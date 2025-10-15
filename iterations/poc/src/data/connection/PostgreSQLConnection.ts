/**
 * @fileoverview PostgreSQL Connection Pool Implementation
 * @author @darianrosebrook
 *
 * Provides connection pooling, health checks, and metrics for PostgreSQL database operations.
 * Implements the ConnectionPool interface with pg.Pool for efficient connection management.
 */

import { Pool, PoolClient } from "pg";
import { Logger } from "../../utils/Logger";
import {
  ConnectionError,
  ConnectionPool,
  DataLayerError,
  DatabaseConfig,
  HealthCheckResult,
  QueryOptions,
  QueryResult,
} from "../types";

export class PostgreSQLConnection implements ConnectionPool {
  private pool: Pool;
  private logger: Logger;
  private metrics: {
    totalQueries: number;
    activeConnections: number;
    totalConnections: number;
    idleConnections: number;
    waitingClients: number;
  } = {
    totalQueries: 0,
    activeConnections: 0,
    totalConnections: 0,
    idleConnections: 0,
    waitingClients: 0,
  };

  constructor(config: DatabaseConfig, logger?: Logger) {
    this.logger = logger || new Logger("PostgreSQLConnection");

    // Create connection pool with optimized settings
    this.pool = new Pool({
      host: config.host,
      port: config.port,
      database: config.database,
      user: config.username,
      password: config.password,
      ssl: config.ssl,
      max: config.maxConnections || 20,
      idleTimeoutMillis: config.idleTimeoutMillis || 30000,
      connectionTimeoutMillis: config.connectionTimeoutMillis || 2000,
      query_timeout: config.query_timeout || 30000,
      statement_timeout: config.statement_timeout || 30000,
      allowExitOnIdle: true,
    });

    this.setupEventHandlers();
    this.startMetricsCollection();
  }

  /**
   * Get a client from the pool for direct operations
   */
  async connect(): Promise<PoolClient> {
    try {
      const client = await this.pool.connect();
      this.logger.debug("Database client connected", {
        totalCount: this.pool.totalCount,
        idleCount: this.pool.idleCount,
        waitingCount: this.pool.waitingCount,
      });
      return client;
    } catch (error) {
      this.logger.error("Failed to connect to database", error);
      throw new ConnectionError(
        "Failed to establish database connection",
        "connect",
        error as Error
      );
    }
  }

  /**
   * Execute a query with automatic connection management
   */
  async query<T = any>(
    text: string,
    params: any[] = [],
    options: QueryOptions = {}
  ): Promise<QueryResult<T>> {
    const startTime = Date.now();
    const queryId = this.generateQueryId();

    try {
      this.logger.debug("Executing query", {
        queryId,
        text: text.substring(0, 100),
      });

      const result = await this.pool.query(text, params);

      const duration = Date.now() - startTime;
      this.metrics.totalQueries++;

      this.logger.debug("Query completed", {
        queryId,
        duration,
        rowCount: result.rowCount,
        command: result.command,
      });

      return {
        success: true,
        data: result.rows as T,
        duration,
        queryId,
      };
    } catch (error) {
      const duration = Date.now() - startTime;

      this.logger.error("Query failed", {
        queryId,
        duration,
        error: (error as Error).message,
        text: text.substring(0, 100),
      });

      throw new DataLayerError(
        `Query execution failed: ${(error as Error).message}`,
        "QUERY_ERROR",
        "query",
        undefined,
        error as Error
      );
    }
  }

  /**
   * Execute operations within a transaction
   */
  async transaction<T>(
    callback: (client: PoolClient) => Promise<T>
  ): Promise<QueryResult<T>> {
    const startTime = Date.now();
    const client = await this.connect();

    try {
      await client.query("BEGIN");

      const result = await callback(client);

      await client.query("COMMIT");

      const duration = Date.now() - startTime;

      this.logger.debug("Transaction completed", { duration });

      return {
        success: true,
        data: result,
        duration,
        queryId: this.generateQueryId(),
      };
    } catch (error) {
      await client.query("ROLLBACK").catch((rollbackError) => {
        this.logger.error("Rollback failed", rollbackError);
      });

      const duration = Date.now() - startTime;

      this.logger.error("Transaction failed", {
        duration,
        error: (error as Error).message,
      });

      throw new DataLayerError(
        `Transaction failed: ${(error as Error).message}`,
        "TRANSACTION_ERROR",
        "transaction",
        undefined,
        error as Error
      );
    } finally {
      client.release();
    }
  }

  /**
   * Perform health check on database connection
   */
  async healthCheck(): Promise<HealthCheckResult> {
    const startTime = Date.now();

    try {
      const result = await this.query("SELECT 1 as health_check");
      const latency = Date.now() - startTime;

      return {
        status: "healthy",
        database: {
          connected: true,
          latency,
        },
        details: {
          poolStats: this.getStats(),
          latency,
        },
      };
    } catch (error) {
      const latency = Date.now() - startTime;

      this.logger.error("Health check failed", error);

      return {
        status: "unhealthy",
        database: {
          connected: false,
          latency,
          error: (error as Error).message,
        },
      };
    }
  }

  /**
   * Get connection pool statistics
   */
  async getStats(): Promise<Record<string, any>> {
    return {
      totalCount: this.pool.totalCount,
      idleCount: this.pool.idleCount,
      waitingCount: this.pool.waitingCount,
      totalQueries: this.metrics.totalQueries,
      activeConnections: this.metrics.activeConnections,
      totalConnections: this.metrics.totalConnections,
      idleConnections: this.metrics.idleConnections,
      waitingClients: this.metrics.waitingClients,
    };
  }

  /**
   * Gracefully close the connection pool
   */
  async close(): Promise<void> {
    this.logger.info("Closing database connection pool");

    try {
      await this.pool.end();
      this.logger.info("Database connection pool closed successfully");
    } catch (error) {
      this.logger.error("Error closing database connection pool", error);
      throw new ConnectionError(
        "Failed to close database connection pool",
        "close",
        error as Error
      );
    }
  }

  /**
   * Set up event handlers for the connection pool
   */
  private setupEventHandlers(): void {
    this.pool.on("connect", (client) => {
      this.logger.debug("New client connected to database");
    });

    this.pool.on("acquire", (client) => {
      this.metrics.activeConnections++;
      this.logger.debug("Client acquired from pool", {
        active: this.metrics.activeConnections,
      });
    });

    this.pool.on("remove", (client) => {
      this.metrics.activeConnections = Math.max(
        0,
        this.metrics.activeConnections - 1
      );
      this.logger.debug("Client removed from pool", {
        active: this.metrics.activeConnections,
      });
    });

    this.pool.on("error", (error, client) => {
      this.logger.error("Unexpected error on idle client", error);
    });
  }

  /**
   * Start collecting connection pool metrics
   */
  private startMetricsCollection(): void {
    setInterval(() => {
      this.metrics.totalConnections = this.pool.totalCount;
      this.metrics.idleConnections = this.pool.idleCount;
      this.metrics.waitingClients = this.pool.waitingCount;
    }, 5000); // Update every 5 seconds
  }

  /**
   * Generate a unique query ID for tracking
   */
  private generateQueryId(): string {
    return `query_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }
}
