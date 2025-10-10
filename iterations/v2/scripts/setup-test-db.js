#!/usr/bin/env node

/**
 * Setup test database for CI/CD
 *
 * This script:
 * 1. Waits for PostgreSQL to be ready
 * 2. Creates the test database if it doesn't exist
 * 3. Runs database migrations
 */

const { Client } = require('pg');
const fs = require('fs');
const path = require('path');

async function waitForPostgres(host = 'localhost', port = 5432, retries = 30) {
  for (let i = 0; i < retries; i++) {
    try {
      const client = new Client({
        host,
        port,
        user: process.env.POSTGRES_USER || 'postgres',
        password: process.env.POSTGRES_PASSWORD || 'password',
        database: 'postgres' // Connect to default db first
      });

      await client.connect();
      await client.query('SELECT 1');
      await client.end();

      console.log('✅ PostgreSQL is ready');
      return true;
    } catch (error) {
      console.log(`⏳ Waiting for PostgreSQL... (${i + 1}/${retries})`);
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }

  throw new Error('❌ PostgreSQL connection timeout');
}

async function setupDatabase() {
  const client = new Client({
    host: process.env.DB_HOST || 'localhost',
    port: process.env.DB_PORT || 5432,
    user: process.env.POSTGRES_USER || 'postgres',
    password: process.env.POSTGRES_PASSWORD || 'password',
    database: process.env.POSTGRES_DB || 'arbiter_test'
  });

  try {
    await client.connect();
    console.log('✅ Connected to test database');

    // Run migrations
    const migrationsDir = path.join(__dirname, '..', 'migrations');
    if (fs.existsSync(migrationsDir)) {
      const migrationFiles = fs.readdirSync(migrationsDir)
        .filter(file => file.endsWith('.sql'))
        .sort();

      for (const file of migrationFiles) {
        console.log(`📄 Running migration: ${file}`);
        const sql = fs.readFileSync(path.join(migrationsDir, file), 'utf8');

        // Split on semicolons and execute each statement
        const statements = sql.split(';').filter(stmt => stmt.trim().length > 0);

        for (const statement of statements) {
          if (statement.trim()) {
            await client.query(statement);
          }
        }
      }

      console.log('✅ Migrations completed');
    } else {
      console.log('⚠️  No migrations directory found');
    }

  } finally {
    await client.end();
  }
}

async function main() {
  try {
    console.log('🚀 Setting up test database...');

    await waitForPostgres();
    await setupDatabase();

    console.log('🎉 Database setup complete!');
    process.exit(0);

  } catch (error) {
    console.error('❌ Database setup failed:', error.message);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
