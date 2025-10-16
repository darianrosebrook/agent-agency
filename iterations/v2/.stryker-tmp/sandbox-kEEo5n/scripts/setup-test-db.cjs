#!/usr/bin/env node
// @ts-nocheck

/**
 * Setup test database for CI/CD
 *
 * This script:
 * 1. Waits for PostgreSQL to be ready
 * 2. Creates the test database if it doesn't exist
 * 3. Runs database migrations
 */

const { Client } = require("pg");
const fs = require("fs");
const path = require("path");

async function waitForPostgres(host = "localhost", port = 5432, retries = 30) {
  for (let i = 0; i < retries; i++) {
    try {
      const client = new Client({
        host,
        port,
        user: process.env.POSTGRES_USER || "postgres",
        password: process.env.POSTGRES_PASSWORD || "password",
        database: "postgres", // Connect to default db first
      });

      await client.connect();
      await client.query("SELECT 1");
      await client.end();

      console.log("✅ PostgreSQL is ready");
      return true;
    } catch (error) {
      console.log(`⏳ Waiting for PostgreSQL... (${i + 1}/${retries})`);
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  }

  throw new Error("❌ PostgreSQL connection timeout");
}

async function setupDatabase() {
  // First connect to default postgres database to create our test database
  const adminClient = new Client({
    host: process.env.DB_HOST || "localhost",
    port: process.env.DB_PORT || 5432,
    user: process.env.POSTGRES_USER || "postgres",
    password: process.env.POSTGRES_PASSWORD || "password",
    database: "postgres",
  });

  const testDbName = process.env.POSTGRES_DB || "arbiter_test";
  let client;

  try {
    await adminClient.connect();
    console.log("✅ Connected to postgres database");

    // Create test database if it doesn't exist
    const dbExists = await adminClient.query(
      "SELECT 1 FROM pg_database WHERE datname = $1",
      [testDbName]
    );

    if (dbExists.rows.length === 0) {
      console.log(`📄 Creating database: ${testDbName}`);
      await adminClient.query(`CREATE DATABASE ${testDbName}`);
      console.log("✅ Database created");
    } else {
      console.log(`✅ Database ${testDbName} already exists`);
    }

    await adminClient.end();

    // Now connect to the test database
    client = new Client({
      host: process.env.DB_HOST || "localhost",
      port: process.env.DB_PORT || 5432,
      user: process.env.POSTGRES_USER || "postgres",
      password: process.env.POSTGRES_PASSWORD || "password",
      database: testDbName,
    });

    await client.connect();
    console.log("✅ Connected to test database");

    // Run migrations
    const migrationsDir = path.join(__dirname, "..", "migrations");
    if (fs.existsSync(migrationsDir)) {
      const migrationFiles = fs
        .readdirSync(migrationsDir)
        .filter((file) => file.endsWith(".sql"))
        .sort();

      for (const file of migrationFiles) {
        console.log(`📄 Running migration: ${file}`);
        const sql = fs.readFileSync(path.join(migrationsDir, file), "utf8");

        // Execute the entire SQL file as one query to handle functions and complex statements
        if (sql.trim()) {
          await client.query(sql);
        }
      }

      console.log("✅ Migrations completed");
    } else {
      console.log("⚠️  No migrations directory found");
    }
  } finally {
    await client.end();
  }
}

async function main() {
  try {
    console.log("🚀 Setting up test database...");

    await waitForPostgres();
    await setupDatabase();

    console.log("🎉 Database setup complete!");
    process.exit(0);
  } catch (error) {
    console.error("❌ Database setup failed:", error.message);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
