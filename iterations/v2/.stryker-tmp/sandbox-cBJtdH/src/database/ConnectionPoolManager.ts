/**
 * Centralized Database Connection Pool Manager
 *
 * @fileoverview Singleton connection pool manager to prevent multiple pool instances
 * and provide centralized configuration, monitoring, and tenant context support.
 *
 * @author @darianrosebrook
 * @description
 * This manager ensures only ONE connection pool exists for the entire application,
 * preventing resource waste and connection exhaustion. All database clients should
 * use getPool() instead of creating their own Pool instances.
 *
 * Features:
 * - Singleton pattern (one pool for entire app)
 * - Tenant context support for Row Level Security (RLS)
 * - Connection monitoring and health checks
 * - Graceful shutdown handling
 * - Centralized configuration
 */
// @ts-nocheck


import { Pool, PoolClient, PoolConfig } from "pg";

export interface DatabaseConnectionConfig {
  // Connection
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
  ssl?: boolean | { rejectUnauthorized: boolean };

  // Pool sizing
  min?: number; // Minimum connections to maintain
  max?: number; // Maximum connections allowed
  idleTimeoutMs?: number; // How long idle connections stay open
  connectionTimeoutMs?: number; // Max time to wait for connection

  // Query timeouts
  statementTimeoutMs?: number; // Max query execution time
  queryTimeoutMs?: number; // Deprecated alias for statementTimeout

  // Application metadata
  applicationName?: string; // Appears in pg_stat_activity
}

export interface PoolStats {
  totalCount: number; // Total connections in pool
  idleCount: number; // Idle connections available
  waitingCount: number; // Clients waiting for connection
  activeConnections: number; // Currently executing queries
  createdAt: Date;
  healthCheckStatus: "healthy" | "degraded" | "unhealthy";
  lastHealthCheck?: Date;
}

export interface TenantContext {
  tenantId: string;
  userId?: string;
  sessionId?: string;
}

/**
 * Centralized Connection Pool Manager (Singleton)
 *
 * Usage:
 * ```typescript
 * const pool = ConnectionPoolManager.getInstance().getPool();
 * const result = await pool.query('SELECT * FROM agents');
 *
 * // With tenant context (for RLS):
 * const client = await ConnectionPoolManager.getInstance()
 *   .getClientWithTenantContext('tenant-123');
 * try {
 *   const result = await client.query('SELECT * FROM agent_profiles');
 * } finally {
 *   client.release();
 * }
 * ```
 */
export class ConnectionPoolManager {
  private static instance: ConnectionPoolManager | null = null;
  private pool: Pool | null = null;
  private config: DatabaseConnectionConfig | null = null;
  private createdAt: Date | null = null;
  private isShuttingDown: boolean = false;

  // Private constructor prevents direct instantiation
  private constructor() {
    // Set up process shutdown handlers
    this.setupShutdownHandlers();
  }

  /**
   * Get singleton instance
   */
  static getInstance(): ConnectionPoolManager {
    if (!ConnectionPoolManager.instance) {
      ConnectionPoolManager.instance = new ConnectionPoolManager();
    }
    return ConnectionPoolManager.instance;
  }

  /**
   * Initialize connection pool with configuration
   *
   * Can be called multiple times, but will reuse existing pool if config matches
   */
  initialize(config: DatabaseConnectionConfig): void {
    if (this.pool) {
      console.warn(
        "Connection pool already initialized. Reusing existing pool."
      );
      return;
    }

    this.config = this.normalizeConfig(config);
    this.createdAt = new Date();

    const poolConfig: PoolConfig = {
      host: this.config.host,
      port: this.config.port,
      database: this.config.database,
      user: this.config.user,
      password: this.config.password,
      ssl: this.config.ssl,
      min: this.config.min || 2,
      max: this.config.max || 20,
      idleTimeoutMillis: this.config.idleTimeoutMs || 30000,
      connectionTimeoutMillis: this.config.connectionTimeoutMs || 10000,
      statement_timeout: this.config.statementTimeoutMs || 30000,
      application_name: this.config.applicationName || "v2-arbiter",
    };

    this.pool = new Pool(poolConfig);

    // Set up event handlers for monitoring
    this.pool.on("connect", (_client) => {
      if (!this.isShuttingDown) {
        console.log(
          `[ConnectionPool] New connection established (total: ${this.pool?.totalCount})`
        );
      }
    });

    this.pool.on("acquire", (_client) => {
      if (!this.isShuttingDown) {
        console.log(
          `[ConnectionPool] Connection acquired (active: ${
            this.pool!.totalCount - this.pool!.idleCount
          }, idle: ${this.pool?.idleCount})`
        );
      }
    });

    this.pool.on("remove", (_client) => {
      if (!this.isShuttingDown && this.pool) {
        console.log(
          `[ConnectionPool] Connection removed (total: ${this.pool.totalCount})`
        );
      }
    });

    this.pool.on("error", (err, _client) => {
      console.error("[ConnectionPool] Unexpected pool error:", err);
      // Don't throw - pool should recover automatically
    });

    console.log(
      `[ConnectionPool] Initialized: ${this.config.host}:${this.config.port}/${this.config.database} (min: ${poolConfig.min}, max: ${poolConfig.max})`
    );
  }

  /**
   * Initialize from environment variables
   */
  initializeFromEnv(): void {
    const config: DatabaseConnectionConfig = {
      host: process.env.DB_HOST || "localhost",
      port: parseInt(process.env.DB_PORT || "5432", 10),
      database: process.env.DB_NAME || "agent_agency_v2",
      user: process.env.DB_USER || "postgres",
      password: process.env.DB_PASSWORD || "",
      ssl:
        process.env.DB_SSL === "true" ? { rejectUnauthorized: false } : false,
      min: process.env.DB_POOL_MIN ? parseInt(process.env.DB_POOL_MIN, 10) : 2,
      max: process.env.DB_POOL_MAX ? parseInt(process.env.DB_POOL_MAX, 10) : 20,
      idleTimeoutMs: process.env.DB_IDLE_TIMEOUT_MS
        ? parseInt(process.env.DB_IDLE_TIMEOUT_MS, 10)
        : 30000,
      connectionTimeoutMs: process.env.DB_CONNECTION_TIMEOUT_MS
        ? parseInt(process.env.DB_CONNECTION_TIMEOUT_MS, 10)
        : 10000,
      statementTimeoutMs: process.env.DB_STATEMENT_TIMEOUT_MS
        ? parseInt(process.env.DB_STATEMENT_TIMEOUT_MS, 10)
        : 30000,
      applicationName: process.env.DB_APPLICATION_NAME || "v2-arbiter",
    };

    // Support DATABASE_URL format (postgresql://user:pass@host:port/db)
    if (process.env.DATABASE_URL) {
      const url = new URL(process.env.DATABASE_URL);
      config.host = url.hostname;
      config.port = parseInt(url.port, 10) || 5432;
      config.database = url.pathname.slice(1); // Remove leading /
      config.user = url.username;
      config.password = url.password;
    }

    this.initialize(config);
  }

  /**
   * Get the connection pool
   *
   * @throws Error if pool not initialized
   */
  getPool(): Pool {
    if (!this.pool) {
      throw new Error(
        "Connection pool not initialized. Call initialize() or initializeFromEnv() first."
      );
    }
    if (this.isShuttingDown) {
      throw new Error("Connection pool is shutting down");
    }
    return this.pool;
  }

  /**
   * Get a client with tenant context set for Row Level Security (RLS)
   *
   * Usage:
   * ```typescript
   * const client = await manager.getClientWithTenantContext('tenant-123');
   * try {
   *   // All queries automatically filtered by RLS
   *   const result = await client.query('SELECT * FROM agent_profiles');
   * } finally {
   *   client.release(); // ALWAYS release!
   * }
   * ```
   */
  async getClientWithTenantContext(
    tenantId: string,
    context?: { userId?: string; sessionId?: string }
  ): Promise<PoolClient> {
    const client = await this.getPool().connect();

    try {
      // Set tenant context for RLS policies
      await client.query("SET LOCAL app.current_tenant = $1", [tenantId]);

      // Optionally set additional context
      if (context?.userId) {
        await client.query("SET LOCAL app.current_user = $1", [context.userId]);
      }
      if (context?.sessionId) {
        await client.query("SET LOCAL app.current_session = $1", [
          context.sessionId,
        ]);
      }

      return client;
    } catch (error) {
      // Release client if context setup fails
      client.release();
      throw error;
    }
  }

  /**
   * Execute query with automatic tenant context
   *
   * Helper method that handles client acquisition and release automatically
   */
  async queryWithTenantContext<T = any>(
    tenantId: string,
    sql: string,
    params: any[] = [],
    context?: { userId?: string; sessionId?: string }
  ): Promise<{ rows: T[]; rowCount: number }> {
    const client = await this.getClientWithTenantContext(tenantId, context);
    try {
      const result = await client.query(sql, params);
      return {
        rows: result.rows as T[],
        rowCount: result.rowCount || 0,
      };
    } finally {
      client.release();
    }
  }

  /**
   * Health check - verifies pool can acquire connections
   */
  async healthCheck(): Promise<boolean> {
    try {
      const result = await this.getPool().query("SELECT 1 as health");
      return result.rows[0].health === 1;
    } catch (error) {
      console.error("[ConnectionPool] Health check failed:", error);
      return false;
    }
  }

  /**
   * Get pool statistics
   */
  getStats(): PoolStats {
    if (!this.pool || !this.createdAt) {
      throw new Error("Connection pool not initialized");
    }

    const totalCount = this.pool.totalCount;
    const idleCount = this.pool.idleCount;
    const waitingCount = this.pool.waitingCount;

    // Determine health status based on utilization
    const utilization =
      (totalCount - idleCount) / (this.config?.max || totalCount);
    let healthCheckStatus: "healthy" | "degraded" | "unhealthy" = "healthy";

    if (utilization > 0.9) {
      healthCheckStatus = "unhealthy"; // >90% utilization
    } else if (utilization > 0.75) {
      healthCheckStatus = "degraded"; // >75% utilization
    }

    return {
      totalCount,
      idleCount,
      waitingCount,
      activeConnections: totalCount - idleCount,
      createdAt: this.createdAt,
      healthCheckStatus,
    };
  }

  /**
   * Graceful shutdown - closes all connections
   *
   * Call this on application shutdown to cleanly close connections
   */
  async shutdown(): Promise<void> {
    if (!this.pool) {
      console.warn("[ConnectionPool] No pool to shutdown");
      return;
    }

    if (this.isShuttingDown) {
      console.warn("[ConnectionPool] Shutdown already in progress");
      return;
    }

    this.isShuttingDown = true;
    console.log("[ConnectionPool] Shutting down...");

    try {
      // Remove all event listeners to prevent async logging after shutdown
      this.pool.removeAllListeners();

      await this.pool.end();
      console.log("[ConnectionPool] All connections closed");
    } catch (error) {
      console.error("[ConnectionPool] Error during shutdown:", error);
      throw error;
    } finally {
      this.pool = null;
      this.config = null;
      this.createdAt = null;
      this.isShuttingDown = false;
    }
  }

  /**
   * Get configuration (for debugging)
   */
  getConfig(): DatabaseConnectionConfig | null {
    return this.config;
  }

  /**
   * Check if pool is initialized
   */
  isInitialized(): boolean {
    return this.pool !== null;
  }

  /**
   * Normalize configuration with defaults
   */
  private normalizeConfig(
    config: DatabaseConnectionConfig
  ): DatabaseConnectionConfig {
    return {
      ...config,
      min: config.min ?? 2,
      max: config.max ?? 20,
      idleTimeoutMs: config.idleTimeoutMs ?? 30000,
      connectionTimeoutMs: config.connectionTimeoutMs ?? 10000,
      statementTimeoutMs:
        config.statementTimeoutMs ?? config.queryTimeoutMs ?? 30000,
      applicationName: config.applicationName ?? "v2-arbiter",
    };
  }

  /**
   * Set up graceful shutdown handlers
   */
  private setupShutdownHandlers(): void {
    const shutdownHandler = async (signal: string) => {
      console.log(`[ConnectionPool] Received ${signal}, shutting down...`);
      try {
        await this.shutdown();
        process.exit(0);
      } catch (error) {
        console.error("[ConnectionPool] Error during shutdown:", error);
        process.exit(1);
      }
    };

    process.on("SIGTERM", () => shutdownHandler("SIGTERM"));
    process.on("SIGINT", () => shutdownHandler("SIGINT"));
  }

  /**
   * Reset singleton (for testing only)
   */
  static resetForTesting(): void {
    if (ConnectionPoolManager.instance?.pool) {
      ConnectionPoolManager.instance.pool.end().catch(console.error);
    }
    ConnectionPoolManager.instance = null;
  }
}

/**
 * Convenience export for getting the pool
 */
export function getPool(): Pool {
  return ConnectionPoolManager.getInstance().getPool();
}

/**
 * Convenience export for tenant-scoped queries
 */
export async function withTenantContext<T>(
  tenantId: string,
  callback: (_: PoolClient) => Promise<T>,
  context?: { userId?: string; sessionId?: string }
): Promise<T> {
  const client =
    await ConnectionPoolManager.getInstance().getClientWithTenantContext(
      tenantId,
      context
    );
  try {
    return await callback(client);
  } finally {
    client.release();
  }
}
