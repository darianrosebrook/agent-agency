/**
 * @fileoverview Database Migration Manager
 * @author @darianrosebrook
 *
 * Manages database schema migrations with dependency tracking and rollback support.
 * Ensures database schema stays in sync with application requirements.
 */

import * as fs from "fs";
import * as path from "path";
import { Logger } from "../../utils/Logger";
import { DataLayer } from "../DataLayer";
import { DataLayerError, Migration, MigrationResult } from "../types";

export class MigrationManager {
  private dataLayer: DataLayer;
  private logger: Logger;
  private migrationPath: string;
  private migrations: Map<string, Migration> = new Map();

  constructor(
    dataLayer: DataLayer,
    migrationPath: string = "migrations",
    logger?: Logger
  ) {
    this.dataLayer = dataLayer;
    this.migrationPath = migrationPath;
    this.logger = logger || new Logger("MigrationManager");
  }

  /**
   * Load migrations from the filesystem
   */
  async loadMigrations(): Promise<void> {
    try {
      const migrationFiles = fs
        .readdirSync(this.migrationPath)
        .filter((file) => file.endsWith(".sql"))
        .sort(); // Ensure consistent ordering

      this.logger.info(`Loading ${migrationFiles.length} migration files`);

      for (const file of migrationFiles) {
        const migration = await this.loadMigrationFromFile(file);
        this.migrations.set(migration.id, migration);
      }

      this.logger.info(`Loaded ${this.migrations.size} migrations`);
    } catch (error) {
      this.logger.error("Failed to load migrations", error);
      throw new DataLayerError(
        "Failed to load migration files",
        "MIGRATION_LOAD_ERROR",
        "loadMigrations",
        undefined,
        error as Error
      );
    }
  }

  /**
   * Run all pending migrations
   */
  async migrate(): Promise<MigrationResult> {
    const startTime = Date.now();

    try {
      await this.ensureMigrationsTable();

      const executedMigrations = await this.getExecutedMigrations();
      const pendingMigrations = this.getPendingMigrations(executedMigrations);

      if (pendingMigrations.length === 0) {
        this.logger.info("No pending migrations to execute");
        return {
          success: true,
          migrations: [],
          duration: Date.now() - startTime,
        };
      }

      this.logger.info(
        `Executing ${pendingMigrations.length} pending migrations`
      );

      const executed: string[] = [];

      for (const migration of pendingMigrations) {
        await this.executeMigration(migration);
        executed.push(migration.id);
      }

      const duration = Date.now() - startTime;
      this.logger.info(
        `Successfully executed ${executed.length} migrations in ${duration}ms`
      );

      return {
        success: true,
        migrations: executed,
        duration,
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      this.logger.error("Migration failed", error);

      return {
        success: false,
        error: (error as Error).message,
        duration,
      };
    }
  }

  /**
   * Rollback migrations
   */
  async rollback(steps: number = 1): Promise<MigrationResult> {
    const startTime = Date.now();

    try {
      const executedMigrations = await this.getExecutedMigrations();

      if (executedMigrations.length === 0) {
        this.logger.info("No migrations to rollback");
        return {
          success: true,
          migrations: [],
          duration: Date.now() - startTime,
        };
      }

      const toRollback = executedMigrations.slice(-steps);
      this.logger.info(`Rolling back ${toRollback.length} migrations`);

      const rolledBack: string[] = [];

      // Rollback in reverse order
      for (const migrationId of toRollback.reverse()) {
        const migration = this.migrations.get(migrationId);
        if (migration) {
          await this.rollbackMigration(migration);
          rolledBack.push(migration.id);
        }
      }

      const duration = Date.now() - startTime;
      this.logger.info(
        `Successfully rolled back ${rolledBack.length} migrations in ${duration}ms`
      );

      return {
        success: true,
        migrations: rolledBack,
        duration,
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      this.logger.error("Rollback failed", error);

      return {
        success: false,
        error: (error as Error).message,
        duration,
      };
    }
  }

  /**
   * Get migration status
   */
  async status(): Promise<MigrationResult> {
    try {
      const executedMigrations = await this.getExecutedMigrations();
      const pendingMigrations = this.getPendingMigrations(executedMigrations);

      return {
        success: true,
        migrations: executedMigrations,
        duration: 0,
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        duration: 0,
      };
    }
  }

  /**
   * Create a new migration file
   */
  async create(name: string): Promise<string> {
    const timestamp = new Date()
      .toISOString()
      .replace(/[:.]/g, "-")
      .slice(0, -5);
    const filename = `${timestamp}_${name}.sql`;
    const filepath = path.join(this.migrationPath, filename);

    const template = `-- Migration: ${name}
-- Created: ${new Date().toISOString()}

-- Up migration
-- Write your up migration SQL here

-- Down migration
-- Write your down migration SQL here
`;

    fs.writeFileSync(filepath, template);

    this.logger.info(`Created migration file: ${filename}`);
    return filename;
  }

  // Private methods

  private async loadMigrationFromFile(filename: string): Promise<Migration> {
    const filepath = path.join(this.migrationPath, filename);
    const content = fs.readFileSync(filepath, "utf-8");

    // Parse migration file (simple format: -- Up and -- Down sections)
    const upMatch = content.match(
      /-- Up migration\s*\n([\s\S]*?)(?=\n-- Down migration|$)/
    );
    const downMatch = content.match(/-- Down migration\s*\n([\s\S]*?)$/);

    if (!upMatch) {
      throw new Error(`Migration file ${filename} missing up migration`);
    }

    const id = filename.replace(".sql", "");
    const upSql = upMatch[1].trim();
    const downSql = downMatch ? downMatch[1].trim() : "";

    return {
      id,
      name: filename,
      up: async (pool) => {
        if (upSql) {
          await pool.query(upSql);
        }
      },
      down: async (pool) => {
        if (downSql) {
          await pool.query(downSql);
        }
      },
    };
  }

  private async ensureMigrationsTable(): Promise<void> {
    const createTableSql = `
      CREATE TABLE IF NOT EXISTS schema_migrations (
        id VARCHAR(255) PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        checksum VARCHAR(64)
      );
    `;

    await this.dataLayer.query(createTableSql);
  }

  private async getExecutedMigrations(): Promise<string[]> {
    const result = await this.dataLayer.query<{ id: string }[]>(
      "SELECT id FROM schema_migrations ORDER BY executed_at ASC"
    );

    return result.success ? result.data?.map((row: any) => row.id) || [] : [];
  }

  private getPendingMigrations(executedMigrations: string[]): Migration[] {
    const executedSet = new Set(executedMigrations);
    return Array.from(this.migrations.values())
      .filter((migration) => !executedSet.has(migration.id))
      .sort((a, b) => a.id.localeCompare(b.id));
  }

  private async executeMigration(migration: Migration): Promise<void> {
    this.logger.info(`Executing migration: ${migration.id}`);

    await this.dataLayer.transaction(async (connection) => {
      // Run the up migration
      await migration.up(connection);

      // Record the migration
      await connection.query(
        "INSERT INTO schema_migrations (id, name) VALUES ($1, $2)",
        [migration.id, migration.name]
      );
    });

    this.logger.info(`Migration executed successfully: ${migration.id}`);
  }

  private async rollbackMigration(migration: Migration): Promise<void> {
    this.logger.info(`Rolling back migration: ${migration.id}`);

    await this.dataLayer.transaction(async (connection) => {
      // Run the down migration
      await migration.down(connection);

      // Remove the migration record
      await connection.query("DELETE FROM schema_migrations WHERE id = $1", [
        migration.id,
      ]);
    });

    this.logger.info(`Migration rolled back successfully: ${migration.id}`);
  }
}
