/**
 * @fileoverview Database Client Interface for Arbiter Orchestration (ARBITER-005)
 *
 * Provides a clean abstraction over database operations with connection pooling,
 * transaction support, and error handling.
 *
 * Uses centralized ConnectionPoolManager for connection sharing and multi-tenant support.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { ConnectionPoolManager } from "../database/ConnectionPoolManager";

export interface QueryResult<T = any> {
  rows: T[];
  rowCount: number;
  command: string;
}

export interface Transaction {
  query<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>>; // eslint-disable-line no-unused-vars
  commit(): Promise<void>;
  rollback(): Promise<void>;
}

/**
 * Database Client Interface
 */
export interface IDatabaseClient {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  isConnected(): boolean;

  query<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>>; // eslint-disable-line no-unused-vars
  transaction<T>(callback: (_tx: Transaction) => Promise<T>): Promise<T>; // eslint-disable-line no-unused-vars

  healthCheck(): Promise<boolean>;
  getStats(): Promise<DatabaseStats>;
}

/**
 * Database Statistics
 */
export interface DatabaseStats {
  totalConnections: number;
  activeConnections: number;
  idleConnections: number;
  waitingClients: number;
  uptimeMs: number;
  lastHealthCheck: Date;
  totalQueries: number;
  totalTransactions: number;
  averageQueryTimeMs: number;
}

/**
 * Database Error Types
 */
export class DatabaseError extends Error {
  constructor(message: string, public _code: string, public _details?: any) {
    super(message);
    this.name = "DatabaseError";
  }
}

export class ConnectionError extends DatabaseError {
  constructor(message: string, details?: any) {
    super(message, "CONNECTION_ERROR", details);
  }
}

export class QueryError extends DatabaseError {
  constructor(message: string, sql?: string, params?: any[], details?: any) {
    super(message, "QUERY_ERROR", { sql, params, ...details });
  }
}

export class TransactionError extends DatabaseError {
  constructor(message: string, details?: any) {
    super(message, "TRANSACTION_ERROR", details);
  }
}

/**
 * Simple PostgreSQL Database Client
 * Uses centralized ConnectionPoolManager for connection sharing
 */
export class PostgresDatabaseClient implements IDatabaseClient {
  private poolManager: ConnectionPoolManager;
  private stats: DatabaseStats;
  private connected: boolean = false;

  constructor() {
    // Use centralized pool manager
    this.poolManager = ConnectionPoolManager.getInstance();
    this.stats = {
      totalConnections: 0,
      activeConnections: 0,
      idleConnections: 0,
      waitingClients: 0,
      uptimeMs: 0,
      lastHealthCheck: new Date(),
      totalQueries: 0,
      totalTransactions: 0,
      averageQueryTimeMs: 0,
    };
  }

  async connect(): Promise<void> {
    try {
      // Verify centralized pool is initialized and accessible
      const client = await this.poolManager.getPool().connect();
      try {
        await client.query("SELECT 1");
        console.log("Connected to PostgreSQL via centralized pool");
      } finally {
        client.release();
      }

      this.connected = true;
      this.stats.uptimeMs = Date.now();
      console.log("Database connection established");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new ConnectionError(
        `Failed to connect to database: ${message}`,
        error
      );
    }
  }

  async initializeSchema(): Promise<void> {
    if (!this.connected) {
      throw new ConnectionError("Database not connected");
    }

    // Create task_assignments table for TaskAssignment persistence
    await this.query(`
      CREATE TABLE IF NOT EXISTS task_assignments (
        assignment_id VARCHAR(255) PRIMARY KEY,
        task_id VARCHAR(255) NOT NULL,
        agent_id VARCHAR(255) NOT NULL,
        agent_name VARCHAR(255),
        agent_model_family VARCHAR(100),
        assigned_at TIMESTAMP NOT NULL,
        deadline TIMESTAMP,
        assignment_timeout_ms INTEGER,
        routing_confidence DECIMAL(3,2),
        routing_strategy VARCHAR(50),
        routing_reason TEXT,
        status VARCHAR(50) NOT NULL DEFAULT 'pending',
        acknowledged_at TIMESTAMP,
        started_at TIMESTAMP,
        completed_at TIMESTAMP,
        progress DECIMAL(5,2) DEFAULT 0,
        last_progress_update TIMESTAMP,
        error_message TEXT,
        error_code VARCHAR(100),
        assignment_metadata JSONB,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );

      CREATE INDEX IF NOT EXISTS idx_task_assignments_task_id ON task_assignments(task_id);
      CREATE INDEX IF NOT EXISTS idx_task_assignments_agent_id ON task_assignments(agent_id);
      CREATE INDEX IF NOT EXISTS idx_task_assignments_status ON task_assignments(status);
      CREATE INDEX IF NOT EXISTS idx_task_assignments_created_at ON task_assignments(created_at);
    `);

    console.log("Database schema initialized for task assignments");
  }

  async disconnect(): Promise<void> {
    try {
      // Note: Pool lifecycle is managed by ConnectionPoolManager
      // We just mark this client as disconnected
      this.connected = false;
      this.stats.totalConnections = 0;
      this.stats.activeConnections = 0;
      this.stats.idleConnections = 0;

      console.log("Database client disconnected");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new ConnectionError(`Failed to disconnect: ${message}`, error);
    }
  }

  isConnected(): boolean {
    return this.connected;
  }

  async query<T = any>(
    sql: string,
    params: any[] = []
  ): Promise<QueryResult<T>> {
    if (!this.connected) {
      throw new ConnectionError("Database not connected");
    }

    const startTime = Date.now();

    try {
      console.log(
        `Executing query: ${sql.substring(0, 100)}${
          sql.length > 100 ? "..." : ""
        }`
      );
      console.log(`Parameters: ${params.length} params`);

      // Execute real PostgreSQL query via centralized pool
      const result = await this.poolManager.getPool().query(sql, params);

      const executionTime = Date.now() - startTime;
      this.stats.totalQueries++;
      this.updateAverageQueryTime(executionTime);

      return {
        rows: result.rows as T[],
        rowCount: result.rowCount || 0,
        command: result.command || sql.split(" ")[0].toUpperCase(),
      };
    } catch (error) {
      const executionTime = Date.now() - startTime;
      const message = error instanceof Error ? error.message : String(error);
      console.error(`Query failed after ${executionTime}ms:`, error);
      throw new QueryError(
        `Query execution failed: ${message}`,
        sql,
        params,
        error
      );
    }
  }

  async transaction<T>(callback: (_tx: Transaction) => Promise<T>): Promise<T> {
    if (!this.connected) {
      throw new ConnectionError("Database not connected");
    }

    this.stats.totalTransactions++;

    // Start real PostgreSQL transaction
    console.log("Starting database transaction");

    const client = await this.poolManager.getPool().connect();

    try {
      await client.query("BEGIN");

      const tx: Transaction = {
        query: async <T = any>(sql: string, params?: any[]) => {
          const result = await client.query(sql, params || []);
          return {
            rows: result.rows as T[],
            rowCount: result.rowCount || 0,
            command: result.command || sql.split(" ")[0].toUpperCase(),
          };
        },
        commit: async () => {
          console.log("Committing transaction");
          await client.query("COMMIT");
        },
        rollback: async () => {
          console.log("Rolling back transaction");
          await client.query("ROLLBACK");
        },
      };

      const result = await callback(tx);
      await tx.commit();

      console.log("Transaction committed successfully");
      return result;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      console.log("Transaction failed, rolling back");
      throw new TransactionError(`Transaction failed: ${message}`, error);
    }
  }

  async healthCheck(): Promise<boolean> {
    try {
      if (!this.connected) {
        return false;
      }

      // Simple health check query
      await this.query("SELECT 1 as health_check");

      this.stats.lastHealthCheck = new Date();
      return true;
    } catch (error) {
      console.error("Health check failed:", error);
      return false;
    }
  }

  getStats(): Promise<DatabaseStats> {
    // Update stats with real pool information from centralized manager
    if (this.connected) {
      const poolStats = this.poolManager.getStats();
      this.stats.totalConnections = poolStats.totalCount || 0;
      this.stats.idleConnections = poolStats.idleCount || 0;
      this.stats.waitingClients = poolStats.waitingCount || 0;
    }

    this.stats.uptimeMs = this.connected ? Date.now() - this.stats.uptimeMs : 0;
    return Promise.resolve({ ...this.stats });
  }

  /**
   * Simulate database queries for development/testing
   * In production, this would be replaced with actual PostgreSQL queries
   */
  private async simulateQuery<T>(_sql: string): Promise<QueryResult<T>> {
    // Simulate query execution time
    await new Promise((resolve) =>
      setTimeout(resolve, Math.random() * 50 + 10)
    );

    // Parse the SQL to determine what kind of operation this is
    const sqlLower = _sql.toLowerCase().trim();

    if (sqlLower.startsWith("select")) {
      return this.simulateSelect<T>();
    } else if (sqlLower.startsWith("insert")) {
      return this.simulateInsert<T>();
    } else if (sqlLower.startsWith("update")) {
      return this.simulateUpdate<T>();
    } else if (sqlLower.startsWith("delete")) {
      return this.simulateDelete<T>();
    } else {
      // Default empty result
      return { rows: [], rowCount: 0, command: "UNKNOWN" };
    }
  }

  private simulateSelect<T>(): QueryResult<T> {
    // Mock implementation - in real implementation, this would parse the SQL
    // For now, return mock configuration data
    return {
      rows: [
        { config_key: "max_capacity", config_value: "1000" },
        { config_key: "default_timeout_ms", config_value: "30000" },
        { config_key: "max_retries", config_value: "3" },
        { config_key: "priority_mode", config_value: '"priority"' },
        { config_key: "persistence_enabled", config_value: "true" },
      ] as T[],
      rowCount: 5,
      command: "SELECT",
    };
  }

  private simulateInsert<T>(): QueryResult<T> {
    return {
      rows: [] as T[],
      rowCount: 1,
      command: "INSERT",
    };
  }

  private simulateUpdate<T>(): QueryResult<T> {
    return {
      rows: [] as T[],
      rowCount: 1,
      command: "UPDATE",
    };
  }

  private simulateDelete<T>(): QueryResult<T> {
    return {
      rows: [] as T[],
      rowCount: 1,
      command: "DELETE",
    };
  }

  private updateAverageQueryTime(executionTime: number): void {
    const totalQueries = this.stats.totalQueries;
    if (totalQueries === 1) {
      this.stats.averageQueryTimeMs = executionTime;
    } else {
      const prevAverage = this.stats.averageQueryTimeMs;
      this.stats.averageQueryTimeMs =
        (prevAverage * (totalQueries - 1) + executionTime) / totalQueries;
    }
  }
}

/**
 * Database Client Factory
 */
export class DatabaseClientFactory {
  static createPostgresClient(): IDatabaseClient {
    return new PostgresDatabaseClient();
  }

  static createMockClient(): IDatabaseClient {
    return new MockDatabaseClient();
  }
}

/**
 * Mock Database Client for Testing
 */
export class MockDatabaseClient implements IDatabaseClient {
  private connected: boolean = false;
  private queryResults: Map<string, any[]> = new Map();

  async connect(): Promise<void> {
    this.connected = true;
  }

  async disconnect(): Promise<void> {
    this.connected = false;
  }

  isConnected(): boolean {
    return this.connected;
  }

  async query<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>> {
    if (!this.connected) {
      throw new ConnectionError("Database not connected");
    }

    const key = this.getQueryKey(sql, params);
    const mockRows = this.queryResults.get(key) || [];

    return {
      rows: mockRows as T[],
      rowCount: mockRows.length,
      command: sql.split(" ")[0].toUpperCase(),
    };
  }

  async transaction<T>(callback: (_tx: Transaction) => Promise<T>): Promise<T> {
    if (!this.connected) {
      throw new ConnectionError("Database not connected");
    }

    const tx: Transaction = {
      query: async <T = any>(sql: string, params?: any[]) =>
        this.query<T>(sql, params),
      commit: async () => {},
      rollback: async () => {},
    };

    return callback(tx);
  }

  async healthCheck(): Promise<boolean> {
    return this.connected;
  }

  getStats(): Promise<DatabaseStats> {
    return Promise.resolve({
      totalConnections: 1,
      activeConnections: this.connected ? 1 : 0,
      idleConnections: this.connected ? 1 : 0,
      waitingClients: 0,
      uptimeMs: 1000,
      lastHealthCheck: new Date(),
      totalQueries: 0,
      totalTransactions: 0,
      averageQueryTimeMs: 10,
    });
  }

  // Test helper methods
  setMockResult(key: string, result: any[]): void {
    this.queryResults.set(key, result);
  }

  private getQueryKey(sql: string, params?: any[]): string {
    return `${sql}:${JSON.stringify(params || [])}`;
  }
}
