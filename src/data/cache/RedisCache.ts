/**
 * @fileoverview Redis Cache Implementation
 * @author @darianrosebrook
 *
 * Provides Redis-based caching with connection pooling, TTL management, and performance metrics.
 * Implements the CacheProvider interface with automatic serialization and error handling.
 */

import { createClient, RedisClientType } from "redis";
import { Logger } from "../../utils/Logger";
import {
  CacheConfig,
  CacheProvider,
  CacheResult,
  ConnectionError,
  DataLayerError,
} from "../types";

export class RedisCache implements CacheProvider {
  private client: RedisClientType;
  private logger: Logger;
  private config: CacheConfig;
  private connected: boolean = false;
  private metrics: {
    hits: number;
    misses: number;
    sets: number;
    deletes: number;
    errors: number;
  } = {
    hits: 0,
    misses: 0,
    sets: 0,
    deletes: 0,
    errors: 0,
  };

  constructor(config: CacheConfig, logger?: Logger) {
    this.config = config;
    this.logger = logger || new Logger("RedisCache");

    this.client = createClient({
      url: `redis://${config.host}:${config.port}`,
      password: config.password,
      database: config.db || 0,
    });

    this.setupEventHandlers();
  }

  /**
   * Initialize the Redis connection
   */
  async initialize(): Promise<void> {
    try {
      await this.client.connect();
      this.connected = true;
      this.logger.info("Redis cache initialized successfully");
    } catch (error) {
      this.logger.error("Failed to initialize Redis cache", error);
      throw new ConnectionError(
        "Failed to connect to Redis",
        "initialize",
        error as Error
      );
    }
  }

  /**
   * Get a value from cache
   */
  async get<T>(key: string): Promise<CacheResult<T>> {
    const startTime = Date.now();
    const fullKey = this.getFullKey(key);

    try {
      if (!this.connected) {
        throw new Error("Redis client not connected");
      }

      const value = await this.client.get(fullKey);

      if (value === null) {
        this.metrics.misses++;
        return {
          success: true,
          hit: false,
          duration: Date.now() - startTime,
        };
      }

      this.metrics.hits++;
      const data = JSON.parse(value) as T;

      return {
        success: true,
        data,
        hit: true,
        duration: Date.now() - startTime,
      };
    } catch (error) {
      this.metrics.errors++;
      this.logger.error("Cache get operation failed", { key: fullKey, error });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime,
      };
    }
  }

  /**
   * Set a value in cache with optional TTL
   */
  async set<T>(
    key: string,
    value: T,
    ttl?: number
  ): Promise<CacheResult<boolean>> {
    const startTime = Date.now();
    const fullKey = this.getFullKey(key);

    try {
      if (!this.connected) {
        throw new Error("Redis client not connected");
      }

      const serializedValue = JSON.stringify(value);
      const effectiveTtl = ttl || this.config.ttl || 3600; // Default 1 hour

      await this.client.setEx(fullKey, effectiveTtl, serializedValue);
      this.metrics.sets++;

      this.logger.debug("Cache set operation completed", {
        key: fullKey,
        ttl: effectiveTtl,
        duration: Date.now() - startTime,
      });

      return {
        success: true,
        data: true,
        hit: false,
        duration: Date.now() - startTime,
      };
    } catch (error) {
      this.metrics.errors++;
      this.logger.error("Cache set operation failed", { key: fullKey, error });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime,
      };
    }
  }

  /**
   * Delete a value from cache
   */
  async delete(key: string): Promise<CacheResult<boolean>> {
    const startTime = Date.now();
    const fullKey = this.getFullKey(key);

    try {
      if (!this.connected) {
        throw new Error("Redis client not connected");
      }

      const result = await this.client.del(fullKey);
      this.metrics.deletes++;

      return {
        success: true,
        data: result > 0,
        hit: false,
        duration: Date.now() - startTime,
      };
    } catch (error) {
      this.metrics.errors++;
      this.logger.error("Cache delete operation failed", {
        key: fullKey,
        error,
      });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime,
      };
    }
  }

  /**
   * Check if a key exists in cache
   */
  async exists(key: string): Promise<CacheResult<boolean>> {
    const startTime = Date.now();
    const fullKey = this.getFullKey(key);

    try {
      if (!this.connected) {
        throw new Error("Redis client not connected");
      }

      const result = await this.client.exists(fullKey);

      return {
        success: true,
        data: result > 0,
        hit: false,
        duration: Date.now() - startTime,
      };
    } catch (error) {
      this.metrics.errors++;
      this.logger.error("Cache exists operation failed", {
        key: fullKey,
        error,
      });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime,
      };
    }
  }

  /**
   * Clear cache keys matching a pattern
   */
  async clear(pattern: string = "*"): Promise<CacheResult<number>> {
    const startTime = Date.now();
    const fullPattern = this.getFullKey(pattern);

    try {
      if (!this.connected) {
        throw new Error("Redis client not connected");
      }

      // Use SCAN for safe pattern deletion
      let cursor = 0;
      let deletedCount = 0;

      do {
        const scanResult = await this.client.scan(cursor as any, {
          MATCH: fullPattern,
          COUNT: 100,
        });

        cursor = Number(scanResult.cursor);

        if (scanResult.keys.length > 0) {
          const deleteResult = await this.client.del(scanResult.keys as any);
          deletedCount += deleteResult;
        }
      } while (cursor !== 0);

      this.logger.info("Cache clear operation completed", {
        pattern: fullPattern,
        deletedCount,
        duration: Date.now() - startTime,
      });

      return {
        success: true,
        data: deletedCount,
        hit: false,
        duration: Date.now() - startTime,
      };
    } catch (error) {
      this.metrics.errors++;
      this.logger.error("Cache clear operation failed", {
        pattern: fullPattern,
        error,
      });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime,
      };
    }
  }

  /**
   * Get cache statistics and health information
   */
  async getStats(): Promise<CacheResult<Record<string, any>>> {
    const startTime = Date.now();

    try {
      if (!this.connected) {
        throw new Error("Redis client not connected");
      }

      const info = await this.client.info();
      const totalRequests =
        this.metrics.hits +
        this.metrics.misses +
        this.metrics.sets +
        this.metrics.deletes;
      const hitRate =
        totalRequests > 0 ? (this.metrics.hits / totalRequests) * 100 : 0;

      const stats = {
        connected: this.connected,
        hitRate: `${hitRate.toFixed(2)}%`,
        hits: this.metrics.hits,
        misses: this.metrics.misses,
        sets: this.metrics.sets,
        deletes: this.metrics.deletes,
        errors: this.metrics.errors,
        info: this.parseRedisInfo(info),
      };

      return {
        success: true,
        data: stats,
        hit: false,
        duration: Date.now() - startTime,
      };
    } catch (error) {
      this.logger.error("Failed to get cache stats", error);

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime,
      };
    }
  }

  /**
   * Close the Redis connection
   */
  async close(): Promise<void> {
    try {
      await this.client.quit();
      this.connected = false;
      this.logger.info("Redis cache connection closed");
    } catch (error) {
      this.logger.error("Error closing Redis connection", error);
      throw new DataLayerError(
        "Failed to close Redis connection",
        "CONNECTION_CLOSE_ERROR",
        "close",
        undefined,
        error as Error
      );
    }
  }

  /**
   * Get the full key with prefix
   */
  private getFullKey(key: string): string {
    const prefix = this.config.keyPrefix || "agent_agency";
    return `${prefix}:${key}`;
  }

  /**
   * Parse Redis INFO command output
   */
  private parseRedisInfo(info: string): Record<string, any> {
    const lines = info.split("\n");
    const parsed: Record<string, any> = {};

    for (const line of lines) {
      if (line.includes(":")) {
        const [key, value] = line.split(":", 2);
        parsed[key] = value;
      }
    }

    return parsed;
  }

  /**
   * Set up Redis client event handlers
   */
  private setupEventHandlers(): void {
    this.client.on("connect", () => {
      this.logger.info("Connected to Redis");
      this.connected = true;
    });

    this.client.on("ready", () => {
      this.logger.info("Redis client ready");
    });

    this.client.on("error", (error) => {
      this.logger.error("Redis client error", error);
      this.connected = false;
    });

    this.client.on("end", () => {
      this.logger.info("Redis connection ended");
      this.connected = false;
    });

    this.client.on("reconnecting", () => {
      this.logger.info("Reconnecting to Redis");
    });
  }
}
