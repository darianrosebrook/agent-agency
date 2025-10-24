#!/usr/bin/env node

/**
 * Database Setup and Migration Script for Agent Agency V3
 *
 * This script handles complete database setup for the V3 iteration including:
 * - PostgreSQL role creation
 * - Database creation
 * - pgvector extension installation
 * - Schema migrations
 * - Verification
 *
 * Usage:
 *   node scripts/setup/setup-database-v3.js [command]
 *
 * Commands:
 *   init      - Full setup (role, database, extensions, migrations)
 *   migrate   - Run migrations only (assumes DB exists)
 *   verify    - Verify database setup is complete
 *   clean     - Clean up (drop database and role)
 *   status    - Show current setup status
 */

const { execSync, spawn } = require("child_process");
const fs = require("fs");
const path = require("path");
const { Client } = require("pg");

// Configuration
const CONFIG = {
  host: process.env.DB_HOST || "localhost",
  port: process.env.DB_PORT || 5432,
  database: process.env.DB_NAME || "agent_agency",
  user: process.env.DB_USER || "agent_agency",
  password: process.env.DB_PASSWORD || "agent_agency_password",
  ssl: process.env.DB_SSL === "true" ? { rejectUnauthorized: false } : false,

  // Setup-specific config
  superuser: process.env.DB_SUPERUSER || "postgres",
  superpassword: process.env.DB_SUPERPASSWORD || process.env.DB_PASSWORD || "",
};

const MIGRATIONS_DIR =
  "/Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3/database/migrations";

class DatabaseSetup {
  constructor() {
    this.client = null;
    this.superClient = null;
  }

  log(message, type = "info") {
    const timestamp = new Date().toISOString();
    const prefix =
      {
        info: "ℹ️ ",
        success: "✅ ",
        warning: "⚠️ ",
        error: "❌ ",
        step: "🔄 ",
      }[type] || "";

    console.log(`[${timestamp}] ${prefix}${message}`);
  }

  async connectSuperuser() {
    this.superClient = new Client({
      host: CONFIG.host,
      port: CONFIG.port,
      database: "postgres", // Connect to default database
      user: CONFIG.superuser,
      password: CONFIG.superpassword,
      ssl: CONFIG.ssl,
    });

    await this.superClient.connect();
    this.log("Connected to PostgreSQL as superuser");
  }

  async connectApp() {
    this.client = new Client(CONFIG);
    await this.client.connect();
    this.log(`Connected to ${CONFIG.database} database`);
  }

  async disconnect() {
    if (this.client) {
      await this.client.end();
      this.log("Disconnected from app database");
    }
    if (this.superClient) {
      await this.superClient.end();
      this.log("Disconnected from superuser connection");
    }
  }

  async checkPostgresConnection() {
    try {
      await this.connectSuperuser();
      this.log("PostgreSQL server is accessible", "success");
      return true;
    } catch (error) {
      this.log(`Cannot connect to PostgreSQL: ${error.message}`, "error");
      this.log(
        "Make sure PostgreSQL is running and credentials are correct",
        "error"
      );
      return false;
    }
  }

  async createRole() {
    try {
      // Check if role exists
      const roleCheck = await this.superClient.query(
        "SELECT 1 FROM pg_roles WHERE rolname = $1",
        [CONFIG.user]
      );

      if (roleCheck.rows.length > 0) {
        this.log(`Role '${CONFIG.user}' already exists`, "warning");
        return;
      }

      // Create role
      await this.superClient.query(`
        CREATE ROLE ${CONFIG.user} LOGIN
        PASSWORD '${CONFIG.password}'
        NOSUPERUSER NOCREATEDB NOCREATEROLE;
      `);

      this.log(`Created database role '${CONFIG.user}'`, "success");
    } catch (error) {
      throw new Error(`Failed to create role: ${error.message}`);
    }
  }

  async createDatabase() {
    try {
      // Check if database exists
      const dbCheck = await this.superClient.query(
        "SELECT 1 FROM pg_database WHERE datname = $1",
        [CONFIG.database]
      );

      if (dbCheck.rows.length > 0) {
        this.log(`Database '${CONFIG.database}' already exists`, "warning");
        return;
      }

      // Create database
      await this.superClient.query(`
        CREATE DATABASE ${CONFIG.database}
        OWNER ${CONFIG.user}
        ENCODING 'UTF8';
      `);

      this.log(`Created database '${CONFIG.database}'`, "success");
    } catch (error) {
      throw new Error(`Failed to create database: ${error.message}`);
    }
  }

  async installExtensions() {
    try {
      // Switch to app database connection
      await this.connectApp();

      // Check if pgvector is available
      try {
        await this.client.query("CREATE EXTENSION IF NOT EXISTS vector;");
        this.log("pgvector extension installed/enabled", "success");
      } catch (error) {
        this.log(
          "pgvector extension not available - install from https://github.com/pgvector/pgvector",
          "warning"
        );
        this.log("Some features may not work without pgvector", "warning");
      }

      // Enable other required extensions
      const extensions = ["uuid-ossp", "pgcrypto"];
      for (const ext of extensions) {
        try {
          await this.client.query(`CREATE EXTENSION IF NOT EXISTS "${ext}";`);
          this.log(`${ext} extension enabled`, "success");
        } catch (error) {
          this.log(`Failed to enable ${ext}: ${error.message}`, "warning");
        }
      }
    } catch (error) {
      throw new Error(`Failed to install extensions: ${error.message}`);
    }
  }

  async runMigrations() {
    try {
      console.log(`DEBUG: MIGRATIONS_DIR = ${MIGRATIONS_DIR}`);
      this.log(`Migrations directory: ${MIGRATIONS_DIR}`);
      const migrationFiles = fs
        .readdirSync(MIGRATIONS_DIR)
        .filter((file) => file.endsWith(".sql") && file !== "README.md")
        .sort();

      this.log(`Found ${migrationFiles.length} migration files`);

      // Ensure migrations table exists
      await this.client.query(`
        CREATE TABLE IF NOT EXISTS schema_migrations (
          version VARCHAR(255) PRIMARY KEY,
          name VARCHAR(255) NOT NULL,
          executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
          checksum VARCHAR(255)
        );
      `);

      // Get executed migrations
      const executedResult = await this.client.query(
        "SELECT version FROM schema_migrations ORDER BY version"
      );
      const executedVersions = new Set(
        executedResult.rows.map((r) => r.version)
      );

      // Run pending migrations
      let executedCount = 0;
      for (const filename of migrationFiles) {
        const match = filename.match(/^(\d+)_(.+)\.sql$/);
        if (!match) continue;

        const version = match[1];
        const name = match[2];

        if (executedVersions.has(version)) {
          this.log(`Migration ${version} already executed, skipping`);
          continue;
        }

        this.log(`Executing migration ${version}: ${name}`, "step");

        const filepath = path.join(MIGRATIONS_DIR, filename);
        const sql = fs.readFileSync(filepath, "utf8");

        // Split and execute statements
        const statements = sql
          .split(";")
          .map((stmt) => stmt.trim())
          .filter((stmt) => stmt.length > 0 && !stmt.startsWith("--"));

        for (const statement of statements) {
          if (statement.trim()) {
            await this.client.query(statement);
          }
        }

        // Record migration
        await this.client.query(
          "INSERT INTO schema_migrations (version, name) VALUES ($1, $2)",
          [version, name]
        );

        executedCount++;
        this.log(`Migration ${version} completed`, "success");
      }

      if (executedCount === 0) {
        this.log("All migrations already executed", "success");
      } else {
        this.log(`Executed ${executedCount} migrations`, "success");
      }
    } catch (error) {
      throw new Error(`Migration failed: ${error.message}`);
    }
  }

  async verifySetup() {
    try {
      await this.connectApp();

      const checks = [
        {
          name: "Database connection",
          check: async () => {
            await this.client.query("SELECT 1");
            return true;
          },
        },
        {
          name: "Migrations table",
          check: async () => {
            const result = await this.client.query(
              "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'schema_migrations')"
            );
            return result.rows[0].exists;
          },
        },
        {
          name: "Core tables",
          check: async () => {
            const tables = [
              "verdicts",
              "learning_signals",
              "performance_events",
            ];
            for (const table of tables) {
              const result = await this.client.query(
                "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)",
                [table]
              );
              if (!result.rows[0].exists) return false;
            }
            return true;
          },
        },
        {
          name: "RLS policies",
          check: async () => {
            const result = await this.client.query(`
              SELECT COUNT(*) as policy_count
              FROM pg_policies
              WHERE schemaname = 'public'
            `);
            return result.rows[0].policy_count > 0;
          },
        },
      ];

      this.log("Running verification checks...", "step");

      for (const check of checks) {
        try {
          const passed = await check.check();
          if (passed) {
            this.log(`✅ ${check.name}`, "success");
          } else {
            this.log(`❌ ${check.name} failed`, "error");
            return false;
          }
        } catch (error) {
          this.log(`❌ ${check.name} error: ${error.message}`, "error");
          return false;
        }
      }

      this.log("All verification checks passed!", "success");
      return true;
    } catch (error) {
      this.log(`Verification failed: ${error.message}`, "error");
      return false;
    }
  }

  async showStatus() {
    try {
      this.log("Database Setup Status", "info");
      this.log("====================", "info");

      // Check PostgreSQL connection
      const pgConnected = await this.checkPostgresConnection();
      if (!pgConnected) {
        this.log("PostgreSQL server not accessible", "error");
        return;
      }

      // Check role
      try {
        const roleResult = await this.superClient.query(
          "SELECT 1 FROM pg_roles WHERE rolname = $1",
          [CONFIG.user]
        );
        this.log(
          `Role '${CONFIG.user}': ${roleResult.rows.length > 0 ? "✅" : "❌"}`
        );
      } catch (error) {
        this.log(`Role check failed: ${error.message}`, "error");
      }

      // Check database
      try {
        const dbResult = await this.superClient.query(
          "SELECT 1 FROM pg_database WHERE datname = $1",
          [CONFIG.database]
        );
        this.log(
          `Database '${CONFIG.database}': ${
            dbResult.rows.length > 0 ? "✅" : "❌"
          }`
        );
      } catch (error) {
        this.log(`Database check failed: ${error.message}`, "error");
      }

      // Check app connection and schema
      try {
        await this.connectApp();

        // Check migrations
        const migrationResult = await this.client.query(
          "SELECT COUNT(*) as count FROM schema_migrations"
        );
        const migrationCount = migrationResult.rows[0].count;
        this.log(`Migrations executed: ${migrationCount}`);

        // Check core tables
        const tables = ["verdicts", "learning_signals", "performance_events"];
        let tableCount = 0;
        for (const table of tables) {
          const result = await this.client.query(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)",
            [table]
          );
          if (result.rows[0].exists) tableCount++;
        }
        this.log(`Core tables: ${tableCount}/${tables.length}`);

        // Check extensions
        const extensions = ["vector", "uuid-ossp", "pgcrypto"];
        let extCount = 0;
        for (const ext of extensions) {
          try {
            const result = await this.client.query(
              "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = $1)",
              [ext]
            );
            if (result.rows[0].exists) extCount++;
          } catch (error) {
            // Extension check failed, continue
          }
        }
        this.log(`Extensions: ${extCount}/${extensions.length}`);
      } catch (error) {
        this.log(`App database check failed: ${error.message}`, "error");
      }
    } catch (error) {
      this.log(`Status check failed: ${error.message}`, "error");
    }
  }

  async cleanup() {
    try {
      await this.connectSuperuser();

      // Terminate active connections to the database
      await this.superClient.query(
        `
        SELECT pg_terminate_backend(pid)
        FROM pg_stat_activity
        WHERE datname = $1 AND pid <> pg_backend_pid()
      `,
        [CONFIG.database]
      );

      // Drop database
      try {
        await this.superClient.query(
          `DROP DATABASE IF EXISTS ${CONFIG.database}`
        );
        this.log(`Dropped database '${CONFIG.database}'`, "success");
      } catch (error) {
        this.log(`Failed to drop database: ${error.message}`, "warning");
      }

      // Drop role
      try {
        await this.superClient.query(`DROP ROLE IF EXISTS ${CONFIG.user}`);
        this.log(`Dropped role '${CONFIG.user}'`, "success");
      } catch (error) {
        this.log(`Failed to drop role: ${error.message}`, "warning");
      }
    } catch (error) {
      throw new Error(`Cleanup failed: ${error.message}`);
    }
  }

  printUsage() {
    console.log(`
Database Setup Script for Agent Agency V3

Usage:
  node scripts/setup/setup-database-v3.js [command]

Commands:
  init      - Full setup (role, database, extensions, migrations)
  migrate   - Run migrations only (assumes DB exists)
  verify    - Verify database setup is complete
  clean     - Clean up (drop database and role)
  status    - Show current setup status

Environment Variables:
  DB_HOST              - PostgreSQL host (default: localhost)
  DB_PORT              - PostgreSQL port (default: 5432)
  DB_NAME              - Database name (default: agent_agency)
  DB_USER              - App username (default: agent_agency)
  DB_PASSWORD          - App password (default: agent_agency_password)
  DB_SUPERUSER         - Superuser for setup (default: postgres)
  DB_SUPERPASSWORD     - Superuser password (default: DB_PASSWORD)
  DB_SSL               - Enable SSL (default: false)

Examples:
  # Full setup with defaults
  node scripts/setup/setup-database-v3.js init

  # Setup with custom database
  DB_NAME=my_app node scripts/setup/setup-database-v3.js init

  # Check current status
  node scripts/setup/setup-database-v3.js status

  # Clean up everything
  node scripts/setup/setup-database-v3.js clean
`);
  }

  async run() {
    const command = process.argv[2] || "help";

    if (command === "help" || command === "--help" || command === "-h") {
      this.printUsage();
      return;
    }

    try {
      switch (command) {
        case "init":
          this.log("Starting full database setup...", "step");
          if (!(await this.checkPostgresConnection())) return;

          await this.createRole();
          await this.createDatabase();
          await this.installExtensions();
          await this.runMigrations();

          this.log("Database setup completed successfully!", "success");
          await this.verifySetup();
          break;

        case "migrate":
          this.log("Running migrations...", "step");
          await this.connectApp();
          await this.runMigrations();
          this.log("Migrations completed successfully!", "success");
          break;

        case "verify":
          this.log("Verifying database setup...", "step");
          const verified = await this.verifySetup();
          if (verified) {
            this.log("Database setup verified successfully!", "success");
          } else {
            this.log("Database setup verification failed!", "error");
            process.exit(1);
          }
          break;

        case "status":
          await this.showStatus();
          break;

        case "clean":
          this.log("Starting cleanup...", "step");
          await this.cleanup();
          this.log("Cleanup completed!", "success");
          break;

        default:
          this.log(`Unknown command: ${command}`, "error");
          this.printUsage();
          process.exit(1);
      }
    } catch (error) {
      this.log(`Command failed: ${error.message}`, "error");
      process.exit(1);
    } finally {
      await this.disconnect();
    }
  }
}

// Run the setup
if (require.main === module) {
  const setup = new DatabaseSetup();
  setup.run().catch((error) => {
    console.error("Fatal error:", error);
    process.exit(1);
  });
}

module.exports = DatabaseSetup;
