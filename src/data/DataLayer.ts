/**
 * @fileoverview Unified Data Layer Orchestrator
 * @author @darianrosebrook
 *
 * Central coordinator for database operations, caching, and data access patterns.
 * Provides unified interface for all data operations with connection pooling and caching.
 */

import { EventEmitter } from "events";
import { Logger } from "../utils/Logger";
import { RedisCache } from "./cache/RedisCache";
import { PostgreSQLConnection } from "./connection/PostgreSQLConnection";
import {
  CacheProvider,
  ConnectionPool,
  DataLayerConfig,
  DataLayerError,
  DataOperationMetrics,
  HealthCheckResult,
} from "./types";

export class DataLayer extends EventEmitter {
  private connection: ConnectionPool;
  private cache?: CacheProvider;
  private logger: Logger;
  private config: DataLayerConfig;
  private initialized: boolean = false;
  private metrics: DataOperationMetrics[] = [];
  private healthCheckInterval?: NodeJS.Timeout;

  constructor(config: DataLayerConfig, logger?: Logger) {
    super();
    this.config = config;
    this.logger = logger || new Logger("DataLayer");

    // Initialize database connection
    this.connection = new PostgreSQLConnection(config.database, this.logger);

    // Initialize cache if enabled
    if (config.enableCache !== false) {
      this.cache = new RedisCache(config.cache, this.logger);
    }
  }

  /**
   * Initialize the data layer components
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      this.logger.warn("Data layer already initialized");
      return;
    }

    try {
      this.logger.info("Initializing data layer...");

      // Initialize database connection
      this.logger.info("Initializing database connection...");
      // PostgreSQL connection is lazy - no explicit init needed

      // Initialize cache if available
      if (this.cache) {
        this.logger.info("Initializing cache...");
        await (this.cache as any).initialize();
      }

      this.initialized = true;

      // Start health monitoring if enabled
      if (this.config.enableMetrics) {
        this.startHealthMonitoring();
      }

      this.logger.info("Data layer initialized successfully");
      this.emit("initialized");
    } catch (error) {
      this.logger.error("Failed to initialize data layer", error);
      throw new DataLayerError(
        "Data layer initialization failed",
        "INITIALIZATION_ERROR",
        "initialize",
        undefined,
        error as Error
      );
    }
  }

  /**
   * Get the database connection pool
   */
  getConnection(): ConnectionPool {
    if (!this.initialized) {
      throw new DataLayerError(
        "Data layer not initialized",
        "NOT_INITIALIZED",
        "getConnection"
      );
    }
    return this.connection;
  }

  /**
   * Get the cache provider
   */
  getCache(): CacheProvider | undefined {
    if (!this.initialized) {
      throw new DataLayerError(
        "Data layer not initialized",
        "NOT_INITIALIZED",
        "getCache"
      );
    }
    return this.cache;
  }

  /**
   * Execute a database query with optional caching
   */
  async query<T = any>(
    text: string,
    params: any[] = [],
    options: {
      cache?: boolean;
      cacheKey?: string;
      cacheTtl?: number;
      timeout?: number;
    } = {}
  ): Promise<{ success: boolean; data?: T; error?: string; cached?: boolean }> {
    const startTime = Date.now();

    try {
      // Check cache first if enabled
      if (options.cache && this.cache && options.cacheKey) {
        const cacheResult = await this.cache.get<T>(options.cacheKey);
        if (
          cacheResult.success &&
          cacheResult.hit &&
          cacheResult.data !== undefined
        ) {
          this.recordMetrics(
            "query",
            "cache_hit",
            Date.now() - startTime,
            true,
            true
          );
          return { success: true, data: cacheResult.data, cached: true };
        }
      }

      // Execute database query
      const result = await this.connection.query<T>(text, params, {
        timeout: options.timeout || this.config.queryTimeout,
      });

      // Cache result if enabled
      if (options.cache && this.cache && options.cacheKey && result.success) {
        await this.cache.set(options.cacheKey, result.data, options.cacheTtl);
      }

      this.recordMetrics(
        "query",
        "database",
        Date.now() - startTime,
        result.success
      );

      return {
        success: result.success,
        data: result.data,
        error: result.error,
        cached: false,
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      this.recordMetrics("query", "database", duration, false);

      throw new DataLayerError(
        `Query failed: ${(error as Error).message}`,
        "QUERY_ERROR",
        "query",
        undefined,
        error as Error
      );
    }
  }

  /**
   * Execute operations within a database transaction
   */
  async transaction<T>(
    callback: (connection: ConnectionPool) => Promise<T>
  ): Promise<T> {
    const startTime = Date.now();

    try {
      const result = await this.connection.transaction(async (client) => {
        // Create a transaction-scoped connection wrapper
        const txConnection: ConnectionPool = {
          connect: () => Promise.resolve(client),
          query: (text: string, params?: any[]) => client.query(text, params),
          transaction: () => {
            throw new Error("Nested transactions not supported");
          },
          healthCheck: () => this.connection.healthCheck(),
          getStats: () => Promise.resolve({}),
          close: () => Promise.resolve(),
        };

        return await callback(txConnection);
      });

      this.recordMetrics(
        "transaction",
        "database",
        Date.now() - startTime,
        true
      );
      return result.data as T;
    } catch (error) {
      const duration = Date.now() - startTime;
      this.recordMetrics("transaction", "database", duration, false);

      throw new DataLayerError(
        `Transaction failed: ${(error as Error).message}`,
        "TRANSACTION_ERROR",
        "transaction",
        undefined,
        error as Error
      );
    }
  }

  /**
   * Get cached value
   */
  async getCached<T>(
    key: string
  ): Promise<{ success: boolean; data?: T; error?: string; hit: boolean }> {
    if (!this.cache) {
      throw new DataLayerError(
        "Cache not available",
        "CACHE_NOT_AVAILABLE",
        "getCached"
      );
    }

    const startTime = Date.now();
    const result = await this.cache.get<T>(key);

    this.recordMetrics(
      "cache_get",
      "cache",
      Date.now() - startTime,
      result.success,
      result.hit
    );

    return {
      success: result.success,
      data: result.data,
      error: result.error,
      hit: result.hit,
    };
  }

  /**
   * Set cached value
   */
  async setCached<T>(
    key: string,
    value: T,
    ttl?: number
  ): Promise<{ success: boolean; error?: string }> {
    if (!this.cache) {
      throw new DataLayerError(
        "Cache not available",
        "CACHE_NOT_AVAILABLE",
        "setCached"
      );
    }

    const startTime = Date.now();
    const result = await this.cache.set(key, value, ttl);

    this.recordMetrics(
      "cache_set",
      "cache",
      Date.now() - startTime,
      result.success
    );

    return {
      success: result.success,
      error: result.error,
    };
  }

  /**
   * Perform comprehensive health check
   */
  async healthCheck(): Promise<HealthCheckResult> {
    const startTime = Date.now();

    try {
      // Check database health
      const dbHealth = await this.connection.healthCheck();

      // Check cache health
      let cacheHealth: HealthCheckResult["cache"];
      if (this.cache) {
        try {
          const cacheStats = await this.cache.getStats();
          cacheHealth = {
            connected: true,
            latency: cacheStats.duration,
          };
        } catch (error) {
          cacheHealth = {
            connected: false,
            error: (error as Error).message,
          };
        }
      }

      // Determine overall status
      const dbStatus = dbHealth.database?.connected ? "healthy" : "unhealthy";
      const cacheStatus = cacheHealth?.connected
        ? "healthy"
        : cacheHealth
        ? "degraded"
        : "healthy";
      const overallStatus =
        dbStatus === "healthy" && cacheStatus === "healthy"
          ? "healthy"
          : dbStatus === "healthy"
          ? "degraded"
          : "unhealthy";

      const result: HealthCheckResult = {
        status: overallStatus as HealthCheckResult["status"],
        database: dbHealth.database,
        cache: cacheHealth,
        details: {
          initialized: this.initialized,
          cacheEnabled: !!this.cache,
          metricsEnabled: this.config.enableMetrics,
          duration: Date.now() - startTime,
        },
      };

      this.logger.debug("Health check completed", result);
      return result;
    } catch (error) {
      this.logger.error("Health check failed", error);

      return {
        status: "unhealthy",
        details: {
          error: (error as Error).message,
          duration: Date.now() - startTime,
        },
      };
    }
  }

  /**
   * Get comprehensive statistics
   */
  async getStats(): Promise<Record<string, any>> {
    const stats: Record<string, any> = {
      initialized: this.initialized,
      config: {
        enableCache: this.config.enableCache,
        enableMetrics: this.config.enableMetrics,
        queryTimeout: this.config.queryTimeout,
      },
    };

    // Database stats
    try {
      stats.database = await this.connection.getStats();
    } catch (error) {
      stats.database = { error: (error as Error).message };
    }

    // Cache stats
    if (this.cache) {
      try {
        const cacheStats = await this.cache.getStats();
        stats.cache = cacheStats.success
          ? cacheStats.data
          : { error: cacheStats.error };
      } catch (error) {
        stats.cache = { error: (error as Error).message };
      }
    }

    // Performance metrics (last 100 operations)
    if (this.config.enableMetrics) {
      const recentMetrics = this.metrics.slice(-100);
      stats.metrics = {
        totalOperations: recentMetrics.length,
        averageDuration:
          recentMetrics.reduce((sum, m) => sum + m.duration, 0) /
            recentMetrics.length || 0,
        successRate:
          (recentMetrics.filter((m) => m.success).length /
            recentMetrics.length) *
          100,
        cacheHitRate:
          (recentMetrics.filter((m) => m.cacheHit).length /
            recentMetrics.length) *
          100,
        operationsByType: recentMetrics.reduce((acc, m) => {
          acc[m.operation] = (acc[m.operation] || 0) + 1;
          return acc;
        }, {} as Record<string, number>),
      };
    }

    return stats;
  }

  /**
   * Gracefully shutdown the data layer
   */
  async shutdown(): Promise<void> {
    this.logger.info("Shutting down data layer...");

    // Stop health monitoring
    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
    }

    // Close cache connection
    if (this.cache) {
      try {
        await this.cache.close();
      } catch (error) {
        this.logger.error("Error closing cache", error);
      }
    }

    // Close database connections
    try {
      await this.connection.close();
    } catch (error) {
      this.logger.error("Error closing database connections", error);
    }

    this.initialized = false;
    this.logger.info("Data layer shutdown complete");
    this.emit("shutdown");
  }

  /**
   * Record operation metrics
   */
  private recordMetrics(
    operation: string,
    entity: string,
    duration: number,
    success: boolean,
    cacheHit?: boolean,
    queryCount?: number,
    errorType?: string
  ): void {
    if (!this.config.enableMetrics) return;

    const metric: DataOperationMetrics = {
      operation,
      entity,
      duration,
      success,
      cacheHit,
      queryCount,
      errorType,
    };

    this.metrics.push(metric);

    // Keep only last 1000 metrics to prevent memory leaks
    if (this.metrics.length > 1000) {
      this.metrics = this.metrics.slice(-500);
    }

    this.emit("metric", metric);
  }

  /**
   * Start periodic health monitoring
   */
  private startHealthMonitoring(): void {
    this.healthCheckInterval = setInterval(async () => {
      try {
        const health = await this.healthCheck();
        if (health.status !== "healthy") {
          this.logger.warn("Health check detected issues", health);
          this.emit("health_issue", health);
        }
      } catch (error) {
        this.logger.error("Health monitoring failed", error);
      }
    }, 30000); // Check every 30 seconds
  }
}
