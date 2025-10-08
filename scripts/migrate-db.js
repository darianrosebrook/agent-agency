#!/usr/bin/env node

/**
 * Database Migration Runner
 * Executes SQL migrations in order for the multi-tenant memory system
 *
 * Usage:
 *   node scripts/migrate-db.js [up|down|status] [target_version]
 *
 * Examples:
 *   node scripts/migrate-db.js up          # Run all pending migrations
 *   node scripts/migrate-db.js up 002      # Run migrations up to version 002
 *   node scripts/migrate-db.js down 001    # Rollback to version 001
 *   node scripts/migrate-db.js status      # Show current migration status
 */

const fs = require('fs');
const path = require('path');
const { Client } = require('pg');

// Database configuration
const DB_CONFIG = {
  host: process.env.DB_HOST || 'localhost',
  port: process.env.DB_PORT || 5432,
  database: process.env.DB_NAME || 'agent_agency',
  user: process.env.DB_USER || 'postgres',
  password: process.env.DB_PASSWORD || '',
  ssl: process.env.DB_SSL === 'true' ? { rejectUnauthorized: false } : false
};

const MIGRATIONS_DIR = path.join(__dirname, '..', 'migrations');

class MigrationRunner {
  constructor() {
    this.client = null;
  }

  async connect() {
    this.client = new Client(DB_CONFIG);
    await this.client.connect();
    console.log('✅ Connected to database');
  }

  async disconnect() {
    if (this.client) {
      await this.client.end();
      console.log('✅ Disconnected from database');
    }
  }

  async ensureMigrationsTable() {
    const query = `
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version VARCHAR(255) PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        checksum VARCHAR(255)
      );
    `;

    await this.client.query(query);
  }

  getMigrationFiles() {
    const files = fs.readdirSync(MIGRATIONS_DIR)
      .filter(file => file.endsWith('.sql') && file !== 'README.md')
      .sort();

    return files.map(file => {
      const match = file.match(/^(\d+)_(.+)\.sql$/);
      if (!match) {
        throw new Error(`Invalid migration file name: ${file}`);
      }

      return {
        version: match[1],
        name: match[2],
        filename: file,
        filepath: path.join(MIGRATIONS_DIR, file)
      };
    });
  }

  async getExecutedMigrations() {
    const result = await this.client.query(
      'SELECT version, name, executed_at FROM schema_migrations ORDER BY version'
    );

    return result.rows.map(row => ({
      version: row.version,
      name: row.name,
      executed_at: row.executed_at
    }));
  }

  async executeMigration(migration) {
    console.log(`🔄 Executing migration ${migration.version}: ${migration.name}`);

    const sql = fs.readFileSync(migration.filepath, 'utf8');

    // Split SQL into individual statements (basic approach)
    const statements = sql
      .split(';')
      .map(stmt => stmt.trim())
      .filter(stmt => stmt.length > 0 && !stmt.startsWith('--'));

    for (const statement of statements) {
      if (statement.trim()) {
        await this.client.query(statement);
      }
    }

    // Record the migration
    await this.client.query(
      'INSERT INTO schema_migrations (version, name) VALUES ($1, $2)',
      [migration.version, migration.name]
    );

    console.log(`✅ Migration ${migration.version} completed`);
  }

  async rollbackMigration(version) {
    console.log(`🔄 Rolling back to version ${version}`);

    // Get all migrations that need to be rolled back
    const executedMigrations = await this.getExecutedMigrations();
    const migrationsToRollback = executedMigrations
      .filter(m => m.version > version)
      .sort((a, b) => b.version.localeCompare(a.version));

    if (migrationsToRollback.length === 0) {
      console.log(`ℹ️ No migrations to rollback (already at version ${version})`);
      return;
    }

    console.log(`📋 Will rollback ${migrationsToRollback.length} migration(s):`);
    migrationsToRollback.forEach(m => console.log(`   - ${m.version}: ${m.name}`));

    // Note: In a real implementation, you'd need down migration files
    // For now, we'll just remove from the migrations table
    for (const migration of migrationsToRollback) {
      console.log(`🔄 Removing migration ${migration.version} from tracking`);
      await this.client.query('DELETE FROM schema_migrations WHERE version = $1', [migration.version]);
    }

    console.log(`⚠️ Warning: Rollback only removed migration tracking.`);
    console.log(`⚠️ Database schema changes were NOT reverted.`);
    console.log(`💡 Create down migration files for proper rollback functionality.`);
  }

  async showStatus() {
    const availableMigrations = this.getMigrationFiles();
    const executedMigrations = await this.getExecutedMigrations();

    console.log('\n📊 Migration Status');
    console.log('==================');

    const executedVersions = new Set(executedMigrations.map(m => m.version));

    console.log('\nAvailable Migrations:');
    availableMigrations.forEach(migration => {
      const executed = executedMigrations.find(e => e.version === migration.version);
      const status = executed ? '✅' : '⏳';
      const executedAt = executed ? executed.executed_at.toISOString() : '';
      console.log(`  ${status} ${migration.version}: ${migration.name} ${executedAt}`);
    });

    const pendingCount = availableMigrations.length - executedMigrations.length;
    console.log(`\n📈 Summary:`);
    console.log(`   Executed: ${executedMigrations.length}`);
    console.log(`   Pending: ${pendingCount}`);
    console.log(`   Total: ${availableMigrations.length}`);

    if (executedMigrations.length > 0) {
      const latest = executedMigrations[executedMigrations.length - 1];
      console.log(`   Current Version: ${latest.version} (${latest.name})`);
    }
  }

  async runMigrations(targetVersion = null) {
    const availableMigrations = this.getMigrationFiles();
    const executedMigrations = await this.getExecutedMigrations();
    const executedVersions = new Set(executedMigrations.map(m => m.version));

    const pendingMigrations = availableMigrations.filter(
      migration => !executedVersions.has(migration.version)
    );

    if (pendingMigrations.length === 0) {
      console.log('✅ All migrations are up to date');
      return;
    }

    // Filter migrations if target version specified
    let migrationsToRun = pendingMigrations;
    if (targetVersion) {
      migrationsToRun = pendingMigrations.filter(
        migration => migration.version <= targetVersion
      );
    }

    if (migrationsToRun.length === 0) {
      console.log(`✅ Already at or beyond version ${targetVersion}`);
      return;
    }

    console.log(`🚀 Running ${migrationsToRun.length} migration(s)...`);

    for (const migration of migrationsToRun) {
      await this.executeMigration(migration);
    }

    console.log('🎉 All migrations completed successfully!');
  }

  async run() {
    const command = process.argv[2] || 'status';
    const targetVersion = process.argv[3];

    try {
      await this.connect();
      await this.ensureMigrationsTable();

      switch (command) {
        case 'up':
          await this.runMigrations(targetVersion);
          break;
        case 'down':
          if (!targetVersion) {
            console.error('❌ Target version required for rollback');
            process.exit(1);
          }
          await this.rollbackMigration(targetVersion);
          break;
        case 'status':
          await this.showStatus();
          break;
        default:
          console.error(`❌ Unknown command: ${command}`);
          console.log('Usage: node scripts/migrate-db.js [up|down|status] [target_version]');
          process.exit(1);
      }
    } catch (error) {
      console.error('❌ Migration failed:', error.message);
      process.exit(1);
    } finally {
      await this.disconnect();
    }
  }
}

// Run the migration runner
if (require.main === module) {
  const runner = new MigrationRunner();
  runner.run().catch(error => {
    console.error('❌ Fatal error:', error);
    process.exit(1);
  });
}

module.exports = MigrationRunner;
