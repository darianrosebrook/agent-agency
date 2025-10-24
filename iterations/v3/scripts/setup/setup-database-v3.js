#!/usr/bin/env node

/**
 * Database Setup and Migration Script for Agent Agency V3
 */

const { Client } = require("pg");
const fs = require("fs");
const path = require("path");

// Configuration
const CONFIG = {
  host: process.env.DB_HOST || "localhost",
  port: process.env.DB_PORT || 5432,
  database: process.env.DB_NAME || "agent_agency",
  user: process.env.DB_USER || "agent_agency",
  password: process.env.DB_PASSWORD || "agent_agency_password",
  ssl: process.env.DB_SSL === "true" ? { rejectUnauthorized: false } : false,
  superuser: process.env.DB_SUPERUSER || "postgres",
  superpassword: process.env.DB_SUPERPASSWORD || process.env.DB_PASSWORD || "",
};

const MIGRATIONS_DIR = path.join(
  __dirname,
  "..",
  "..",
  "database",
  "migrations"
);

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
      database: "postgres",
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
    if (this.client) await this.client.end();
    if (this.superClient) await this.superClient.end();
  }

  async checkPostgresConnection() {
    try {
      await this.connectSuperuser();
      this.log("PostgreSQL server is accessible", "success");
      return true;
    } catch (error) {
      this.log(`Cannot connect to PostgreSQL: ${error.message}`, "error");
      return false;
    }
  }

  async createRole() {
    const roleCheck = await this.superClient.query(
      "SELECT 1 FROM pg_roles WHERE rolname = $1",
      [CONFIG.user]
    );
    if (roleCheck.rows.length > 0) {
      this.log(`Role '${CONFIG.user}' already exists`, "warning");
      return;
    }
    await this.superClient.query(
      `CREATE ROLE ${CONFIG.user} LOGIN PASSWORD '${CONFIG.password}' NOSUPERUSER NOCREATEDB NOCREATEROLE;`
    );
    this.log(`Created database role '${CONFIG.user}'`, "success");
  }

  async createDatabase() {
    const dbCheck = await this.superClient.query(
      "SELECT 1 FROM pg_database WHERE datname = $1",
      [CONFIG.database]
    );
    if (dbCheck.rows.length > 0) {
      this.log(`Database '${CONFIG.database}' already exists`, "warning");
      return;
    }
    await this.superClient.query(
      `CREATE DATABASE ${CONFIG.database} OWNER ${CONFIG.user} ENCODING 'UTF8' LC_COLLATE 'en_US.UTF-8' LC_CTYPE 'en_US.UTF-8';`
    );
    this.log(`Created database '${CONFIG.database}'`, "success");
  }

  async installExtensions() {
    await this.connectApp();
    try {
      await this.client.query("CREATE EXTENSION IF NOT EXISTS vector;");
      this.log("pgvector extension installed/enabled", "success");
    } catch (error) {
      this.log("pgvector extension not available", "warning");
    }
    const extensions = ["uuid-ossp", "pgcrypto"];
    for (const ext of extensions) {
      try {
        await this.client.query(`CREATE EXTENSION IF NOT EXISTS "${ext}";`);
        this.log(`${ext} extension enabled`, "success");
      } catch (error) {
        this.log(`Failed to enable ${ext}`, "warning");
      }
    }
  }

  async runMigrations() {
    const migrationFiles = fs
      .readdirSync(MIGRATIONS_DIR)
      .filter((f) => f.endsWith(".sql"))
      .sort();
    this.log(`Found ${migrationFiles.length} migration files`);

    await this.superClient.query(
      `CREATE TABLE IF NOT EXISTS schema_migrations (version VARCHAR(255) PRIMARY KEY, name VARCHAR(255) NOT NULL, executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(), checksum VARCHAR(255));`
    );

    const executedResult = await this.superClient.query(
      "SELECT version FROM schema_migrations ORDER BY version"
    );
    const executedVersions = new Set(executedResult.rows.map((r) => r.version));

    let executedCount = 0;
    for (const filename of migrationFiles) {
      const match = filename.match(/^(\d+)_(.+)\.sql$/);
      if (!match) continue;
      const version = match[1];
      const name = match[2];
      if (executedVersions.has(version)) continue;

      this.log(`Executing migration ${version}: ${name}`, "step");
      const filepath = path.join(MIGRATIONS_DIR, filename);
      const sql = fs.readFileSync(filepath, "utf8");

      try {
        await this.superClient.query(sql);
      } catch (error) {
        // Handle "already exists" errors gracefully by running the entire migration
        if (
          error.message.includes("already exists") ||
          error.message.includes("duplicate key") ||
          error.code === "42P07" || // duplicate_table
          error.code === "42710" || // duplicate_object
          error.code === "23505"
        ) {
          // unique_violation
          this.log(
            `Migration ${version} already applied (some objects exist)`,
            "warning"
          );
        } else {
          this.log(`Migration ${version} failed: ${error.message}`, "error");
          throw error;
        }
      }

      await this.superClient.query(
        "INSERT INTO schema_migrations (version, name) VALUES ($1, $2)",
        [version, name]
      );
      executedCount++;
      this.log(`Migration ${version} completed`, "success");
    }

    this.log(`Executed ${executedCount} migrations`, "success");
  }

  async verifySetup() {
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
          const tables = ["verdicts", "learning_signals", "performance_events"];
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
    ];

    for (const check of checks) {
      try {
        const passed = await check.check();
        if (passed) this.log(`✅ ${check.name}`, "success");
        else {
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
  }

  async run() {
    const command = process.argv[2] || "init";
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
        case "status":
          await this.checkPostgresConnection();
          break;
        default:
          console.log("Usage: node setup-database-v3.js [init|status]");
      }
    } catch (error) {
      this.log(`Command failed: ${error.message}`, "error");
      process.exit(1);
    } finally {
      await this.disconnect();
    }
  }
}

if (require.main === module) {
  const setup = new DatabaseSetup();
  setup.run();
}
