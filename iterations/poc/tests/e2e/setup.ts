/**
 * E2E Test Setup
 *
 * @author @darianrosebrook
 * @description Setup utilities for end-to-end tests
 */

import { ChildProcess, exec, spawn } from "child_process";
import fs from "fs";
import path from "path";
import { promisify } from "util";

const execAsync = promisify(exec);

export interface E2ETestServices {
  postgres: ChildProcess | null;
  redis: ChildProcess | null;
  ollama: ChildProcess | null;
  mcpServer: ChildProcess | null;
}

export class E2ETestRunner {
  private services: E2ETestServices = {
    postgres: null,
    redis: null,
    ollama: null,
    mcpServer: null,
  };

  private projectRoot = path.resolve(__dirname, "../../");

  /**
   * Start all required services for E2E testing
   */
  async setup(): Promise<void> {
    console.log("🚀 Setting up E2E test environment...");

    try {
      // Start PostgreSQL
      console.log("📊 Starting PostgreSQL...");
      this.services.postgres = spawn(
        "docker",
        [
          "run",
          "-d",
          "--rm",
          "-p",
          "5432:5432",
          "-e",
          "POSTGRES_PASSWORD=testpass",
          "-e",
          "POSTGRES_DB=agent_agency_test",
          "postgres:16-alpine",
        ],
        { stdio: "pipe" }
      );

      // Start Redis
      console.log("🔴 Starting Redis...");
      this.services.redis = spawn(
        "docker",
        ["run", "-d", "--rm", "-p", "6379:6379", "redis:7-alpine"],
        { stdio: "pipe" }
      );

      // Wait for services to be ready
      await this.waitForPostgres();
      await this.waitForRedis();

      // Initialize database
      await this.initializeDatabase();

      // Start Ollama (if available)
      console.log("🤖 Starting Ollama...");
      try {
        this.services.ollama = spawn("ollama", ["serve"], {
          stdio: "pipe",
          detached: true,
        });

        // Wait a bit for Ollama to start
        await new Promise((resolve) => setTimeout(resolve, 2000));

        // Try to pull Gemma model
        try {
          await execAsync("ollama pull gemma:3n", { timeout: 60000 });
          console.log("✅ Gemma model ready");
        } catch (error) {
          console.log(
            "⚠️  Could not pull Gemma model, tests will skip AI features"
          );
        }
      } catch (error) {
        console.log("⚠️  Ollama not available, AI tests will be skipped");
      }

      console.log("✅ E2E environment ready");
    } catch (error) {
      console.error("❌ Failed to setup E2E environment:", error);
      await this.teardown();
      throw error;
    }
  }

  /**
   * Clean up all services
   */
  async teardown(): Promise<void> {
    console.log("🧹 Tearing down E2E test environment...");

    // Stop services in reverse order
    const services = Object.entries(this.services).reverse();

    for (const [name, process] of services) {
      if (process) {
        console.log(`Stopping ${name}...`);
        try {
          if (name === "ollama") {
            // Ollama needs special handling
            process.kill("SIGTERM");
          } else {
            // Docker containers
            process.kill("SIGTERM");
          }

          // Wait for process to exit
          await new Promise((resolve) => {
            const timeout = setTimeout(() => resolve(void 0), 5000);
            process.on("exit", () => {
              clearTimeout(timeout);
              resolve(void 0);
            });
          });
        } catch (error) {
          console.warn(
            `Warning: Could not stop ${name}:`,
            error instanceof Error ? error.message : String(error)
          );
        }
      }
    }

    // Clean up any remaining containers
    try {
      await execAsync(
        "docker stop $(docker ps -q --filter ancestor=postgres:16-alpine --filter ancestor=redis:7-alpine)"
      );
    } catch (error) {
      // Ignore errors if no containers are running
    }

    console.log("✅ E2E environment cleaned up");
  }

  /**
   * Wait for PostgreSQL to be ready
   */
  private async waitForPostgres(): Promise<void> {
    const maxAttempts = 30;
    const delay = 1000;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        await execAsync("pg_isready -h localhost -p 5432", { timeout: 5000 });
        console.log("✅ PostgreSQL ready");
        return;
      } catch (error) {
        if (attempt === maxAttempts) {
          throw new Error("PostgreSQL failed to start");
        }
        console.log(`Waiting for PostgreSQL... (${attempt}/${maxAttempts})`);
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }

  /**
   * Wait for Redis to be ready
   */
  private async waitForRedis(): Promise<void> {
    const maxAttempts = 30;
    const delay = 1000;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        await execAsync("redis-cli -h localhost -p 6379 ping", {
          timeout: 5000,
        });
        console.log("✅ Redis ready");
        return;
      } catch (error) {
        if (attempt === maxAttempts) {
          throw new Error("Redis failed to start");
        }
        console.log(`Waiting for Redis... (${attempt}/${maxAttempts})`);
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }

  /**
   * Initialize the database schema
   */
  private async initializeDatabase(): Promise<void> {
    console.log("📝 Initializing database...");

    // Run migrations
    const migrationPath = path.join(
      this.projectRoot,
      "migrations",
      "001_create_core_schema.sql"
    );
    const migrationSQL = fs.readFileSync(migrationPath, "utf8");

    // Use psql to execute the migration
    const { spawn } = require("child_process");
    const psql = spawn(
      "psql",
      [
        "-h",
        "localhost",
        "-p",
        "5432",
        "-U",
        "postgres",
        "-d",
        "agent_agency_test",
        "-c",
        migrationSQL,
      ],
      {
        env: { ...process.env, PGPASSWORD: "testpass" },
        stdio: "pipe",
      }
    );

    return new Promise((resolve, reject) => {
      psql.on("close", (code: number) => {
        if (code === 0) {
          console.log("✅ Database initialized");
          resolve();
        } else {
          reject(new Error(`Database initialization failed with code ${code}`));
        }
      });

      psql.on("error", reject);
    });
  }

  /**
   * Run a test function with setup/teardown
   */
  async runTest(testFn: () => Promise<void>): Promise<void> {
    await this.setup();
    try {
      await testFn();
    } finally {
      await this.teardown();
    }
  }
}

// Global test runner instance
export const e2eRunner = new E2ETestRunner();
