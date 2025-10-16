/**
 * Distributed Cache Client
 *
 * Provides Redis-based distributed caching for federated learning,
 * verification results, and other cross-instance data sharing.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";

export interface DistributedCacheConfig {
  enabled: boolean;
  provider: {
    type: "redis" | "mock";
    redis?: {
      host: string;
      port: number;
      password?: string;
      database?: number;
      keyPrefix?: string;
      ttl?: number; // Default TTL in seconds
    };
  };
  retry: {
    maxAttempts: number;
    delayMs: number;
    backoffMultiplier: number;
  };
  serialization: {
    format: "json" | "msgpack";
    compression: boolean;
  };
}

export interface CacheEntry<T = any> {
  key: string;
  value: T;
  ttl?: number; // TTL in seconds
  metadata?: {
    createdAt: Date;
    lastAccessed: Date;
    accessCount: number;
    tenantId?: string;
    topicKey?: string;
    aggregatedAt?: Date;
    insightsCount?: number;
  };
}

export interface TenantContribution {
  tenantId: string;
  topicKey: string;
  contributionCount: number;
  lastContribution: Date;
  insightsCount: number;
}

export class DistributedCacheClient {
  private logger: Logger;
  private config: DistributedCacheConfig;
  private redisClient?: any; // Redis client instance
  private isConnected = false;

  constructor(config: DistributedCacheConfig, logger: Logger) {
    this.config = config;
    this.logger = logger;
  }

  /**
   * Initialize the cache client
   */
  async initialize(): Promise<void> {
    if (!this.config.enabled) {
      this.logger.info("Distributed cache is disabled");
      return;
    }

    try {
      if (this.config.provider.type === "redis") {
        await this.initializeRedis();
      } else {
        this.logger.info("Using mock distributed cache");
      }

      this.logger.info("Distributed cache client initialized", {
        provider: this.config.provider.type,
        enabled: this.config.enabled,
      });
    } catch (error) {
      this.logger.error("Failed to initialize distributed cache client", {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  /**
   * Store a value in the distributed cache
   */
  async set<T>(
    key: string,
    value: T,
    ttl?: number,
    metadata?: Partial<CacheEntry<T>["metadata"]>
  ): Promise<void> {
    if (!this.config.enabled) {
      return;
    }

    const cacheEntry: CacheEntry<T> = {
      key,
      value,
      ttl: ttl || this.config.provider.redis?.ttl,
      metadata: {
        createdAt: new Date(),
        lastAccessed: new Date(),
        accessCount: 0,
        ...metadata,
      },
    };

    try {
      if (this.config.provider.type === "redis") {
        await this.setRedis(key, cacheEntry);
      } else {
        // Mock implementation - store in memory
        this.setMock(key, cacheEntry);
      }

      this.logger.debug("Stored value in distributed cache", {
        key,
        ttl: cacheEntry.ttl,
        hasMetadata: !!metadata,
      });
    } catch (error) {
      this.logger.error("Failed to store value in distributed cache", {
        key,
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  /**
   * Retrieve a value from the distributed cache
   */
  async get<T>(key: string): Promise<T | null> {
    if (!this.config.enabled) {
      return null;
    }

    try {
      let cacheEntry: CacheEntry<T> | null = null;

      if (this.config.provider.type === "redis") {
        cacheEntry = await this.getRedis<T>(key);
      } else {
        cacheEntry = this.getMock<T>(key);
      }

      if (!cacheEntry) {
        return null;
      }

      // Update access metadata
      if (cacheEntry.metadata) {
        cacheEntry.metadata.lastAccessed = new Date();
        cacheEntry.metadata.accessCount++;
      }

      this.logger.debug("Retrieved value from distributed cache", {
        key,
        hasValue: !!cacheEntry.value,
        accessCount: cacheEntry.metadata?.accessCount,
      });

      return cacheEntry.value;
    } catch (error) {
      this.logger.error("Failed to retrieve value from distributed cache", {
        key,
        error: error instanceof Error ? error.message : String(error),
      });
      return null;
    }
  }

  /**
   * Delete a value from the distributed cache
   */
  async delete(key: string): Promise<boolean> {
    if (!this.config.enabled) {
      return false;
    }

    try {
      let deleted = false;

      if (this.config.provider.type === "redis") {
        deleted = await this.deleteRedis(key);
      } else {
        deleted = this.deleteMock(key);
      }

      this.logger.debug("Deleted value from distributed cache", {
        key,
        deleted,
      });

      return deleted;
    } catch (error) {
      this.logger.error("Failed to delete value from distributed cache", {
        key,
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
  }

  /**
   * Check if a key exists in the cache
   */
  async exists(key: string): Promise<boolean> {
    if (!this.config.enabled) {
      return false;
    }

    try {
      if (this.config.provider.type === "redis") {
        return await this.existsRedis(key);
      } else {
        return this.existsMock(key);
      }
    } catch (error) {
      this.logger.error("Failed to check key existence in distributed cache", {
        key,
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
  }

  /**
   * Get all keys matching a pattern
   */
  async keys(pattern: string): Promise<string[]> {
    if (!this.config.enabled) {
      return [];
    }

    try {
      if (this.config.provider.type === "redis") {
        return await this.keysRedis(pattern);
      } else {
        return this.keysMock(pattern);
      }
    } catch (error) {
      this.logger.error("Failed to get keys from distributed cache", {
        pattern,
        error: error instanceof Error ? error.message : String(error),
      });
      return [];
    }
  }

  /**
   * Track tenant contribution to a topic
   */
  async trackTenantContribution(
    tenantId: string,
    topicKey: string,
    insightsCount: number
  ): Promise<void> {
    const contributionKey = `tenant_contribution:${tenantId}:${topicKey}`;
    const contribution: TenantContribution = {
      tenantId,
      topicKey,
      contributionCount: 1,
      lastContribution: new Date(),
      insightsCount,
    };

    // Get existing contribution if any
    const existing = await this.get<TenantContribution>(contributionKey);
    if (existing) {
      contribution.contributionCount = existing.contributionCount + 1;
      contribution.insightsCount += existing.insightsCount;
    }

    await this.set(contributionKey, contribution, 86400 * 30); // 30 days TTL

    this.logger.debug("Tracked tenant contribution", {
      tenantId,
      topicKey,
      insightsCount,
      totalContributions: contribution.contributionCount,
    });
  }

  /**
   * Get source tenants for a topic
   */
  async getSourceTenants(topicKey: string): Promise<string[]> {
    const pattern = `tenant_contribution:*:${topicKey}`;
    const keys = await this.keys(pattern);

    const tenantIds: string[] = [];
    for (const key of keys) {
      const contribution = await this.get<TenantContribution>(key);
      if (contribution) {
        tenantIds.push(contribution.tenantId);
      }
    }

    return [...new Set(tenantIds)]; // Remove duplicates
  }

  /**
   * Get contribution statistics for a tenant
   */
  async getTenantContributions(
    tenantId: string
  ): Promise<TenantContribution[]> {
    const pattern = `tenant_contribution:${tenantId}:*`;
    const keys = await this.keys(pattern);

    const contributions: TenantContribution[] = [];
    for (const key of keys) {
      const contribution = await this.get<TenantContribution>(key);
      if (contribution) {
        contributions.push(contribution);
      }
    }

    return contributions;
  }

  /**
   * Health check for the cache client
   */
  async healthCheck(): Promise<{ healthy: boolean; latency?: number }> {
    if (!this.config.enabled) {
      return { healthy: true };
    }

    try {
      const startTime = Date.now();

      if (this.config.provider.type === "redis") {
        await this.redisClient?.ping();
      }

      const latency = Date.now() - startTime;
      return { healthy: true, latency };
    } catch (error) {
      this.logger.error("Cache health check failed", {
        error: error instanceof Error ? error.message : String(error),
      });
      return { healthy: false };
    }
  }

  /**
   * Shutdown the cache client
   */
  async shutdown(): Promise<void> {
    if (this.config.provider.type === "redis" && this.redisClient) {
      try {
        await this.redisClient.quit();
        this.isConnected = false;
        this.logger.info("Redis client disconnected");
      } catch (error) {
        this.logger.error("Error disconnecting Redis client", {
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }
  }

  // Private methods for Redis implementation
  private async initializeRedis(): Promise<void> {
    try {
      // Import Redis client dynamically to avoid dependency issues
      const { createClient } = await import('redis');
      
      const redisConfig = this.config.provider.redis;
      if (!redisConfig) {
        throw new Error("Redis configuration is required when using Redis provider");
      }

      // Create Redis client with configuration
      this.redisClient = createClient({
        socket: {
          host: redisConfig.host,
          port: redisConfig.port,
          reconnectStrategy: (retries) => Math.min(retries * 50, 500),
        },
        password: redisConfig.password,
        database: redisConfig.database || 0,
      });

      // Handle connection events
      this.redisClient.on('error', (err: Error) => {
        this.logger.error('Redis client error', { error: err.message });
        this.isConnected = false;
      });

      this.redisClient.on('connect', () => {
        this.logger.info('Redis client connected');
        this.isConnected = true;
      });

      this.redisClient.on('ready', () => {
        this.logger.info('Redis client ready');
        this.isConnected = true;
      });

      this.redisClient.on('end', () => {
        this.logger.info('Redis client disconnected');
        this.isConnected = false;
      });

      // Connect to Redis
      await this.redisClient.connect();
      
      this.logger.info("Redis client initialized successfully", {
        host: redisConfig.host,
        port: redisConfig.port,
        database: redisConfig.database || 0,
      });
    } catch (error) {
      this.logger.error("Failed to initialize Redis client", {
        error: error instanceof Error ? error.message : String(error),
      });
      
      // Fall back to mock mode if Redis fails
      this.logger.warn("Falling back to mock cache mode");
      this.config.provider.type = "mock";
      this.isConnected = true;
    }
  }

  private async setRedis<T>(key: string, entry: CacheEntry<T>): Promise<void> {
    if (!this.isConnected) {
      throw new Error("Redis client not connected");
    }

    const serialized = JSON.stringify(entry);
    const fullKey = this.getFullKey(key);

    try {
      // Use Redis SETEX to store with TTL
      if (entry.ttl) {
        await this.redisClient.setex(fullKey, entry.ttl, serialized);
      } else {
        await this.redisClient.set(fullKey, serialized);
      }
      
      this.logger.debug("Stored in Redis", { key: fullKey, ttl: entry.ttl });
    } catch (error) {
      this.logger.error("Failed to store in Redis", {
        key: fullKey,
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  private async getRedis<T>(key: string): Promise<CacheEntry<T> | null> {
    if (!this.isConnected) {
      return null;
    }

    const fullKey = this.getFullKey(key);

    try {
      const serialized = await this.redisClient.get(fullKey);
      
      if (!serialized) {
        this.logger.debug("No data found in Redis", { key: fullKey });
        return null;
      }

      const entry = JSON.parse(serialized) as CacheEntry<T>;
      this.logger.debug("Retrieved from Redis", { key: fullKey });
      return entry;
    } catch (error) {
      this.logger.error("Failed to retrieve from Redis", {
        key: fullKey,
        error: error instanceof Error ? error.message : String(error),
      });
      return null;
    }
  }

  private async deleteRedis(key: string): Promise<boolean> {
    if (!this.isConnected) {
      return false;
    }

    const fullKey = this.getFullKey(key);

    try {
      const result = await this.redisClient.del(fullKey);
      const deleted = result > 0;
      
      this.logger.debug("Deleted from Redis", { key: fullKey, deleted });
      return deleted;
    } catch (error) {
      this.logger.error("Failed to delete from Redis", {
        key: fullKey,
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
  }

  private async existsRedis(key: string): Promise<boolean> {
    if (!this.isConnected) {
      return false;
    }

    const fullKey = this.getFullKey(key);

    try {
      const exists = await this.redisClient.exists(fullKey);
      this.logger.debug("Checked existence in Redis", { key: fullKey, exists: exists > 0 });
      return exists > 0;
    } catch (error) {
      this.logger.error("Failed to check existence in Redis", {
        key: fullKey,
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
  }

  private async keysRedis(pattern: string): Promise<string[]> {
    if (!this.isConnected) {
      return [];
    }

    const fullPattern = this.getFullKey(pattern);

    try {
      const keys = await this.redisClient.keys(fullPattern);
      this.logger.debug("Retrieved keys from Redis", { pattern: fullPattern, count: keys.length });
      return keys;
    } catch (error) {
      this.logger.error("Failed to get keys from Redis", {
        pattern: fullPattern,
        error: error instanceof Error ? error.message : String(error),
      });
      return [];
    }
  }

  private getFullKey(key: string): string {
    const prefix = this.config.provider.redis?.keyPrefix || "federated_cache";
    return `${prefix}:${key}`;
  }

  // Private methods for mock implementation
  private mockCache = new Map<string, CacheEntry>();

  private setMock<T>(key: string, entry: CacheEntry<T>): void {
    this.mockCache.set(key, entry);
  }

  private getMock<T>(key: string): CacheEntry<T> | null {
    return this.mockCache.get(key) || null;
  }

  private deleteMock(key: string): boolean {
    return this.mockCache.delete(key);
  }

  private existsMock(key: string): boolean {
    return this.mockCache.has(key);
  }

  private keysMock(pattern: string): string[] {
    const regex = new RegExp(pattern.replace(/\*/g, ".*"));
    return Array.from(this.mockCache.keys()).filter((key) => regex.test(key));
  }
}
