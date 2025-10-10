/**
 * @fileoverview Multi-Level Cache Implementation
 * @author @darianrosebrook
 *
 * Provides multi-level caching with L1 (in-memory LRU) and L2 (Redis) caches.
 * Implements intelligent cache strategies, automatic promotion/demotion,
 * and performance monitoring for >95% hit rates.
 */

import { RedisCache } from "./RedisCache";
import { Logger } from "../../utils/Logger";
import {
  CacheProvider,
  CacheConfig,
  CacheResult,
  DataLayerError
} from "../types";

interface CacheEntry<T = any> {
  data: T;
  timestamp: number;
  ttl: number;
  accessCount: number;
  lastAccessed: number;
  size: number; // Approximate memory size
}

interface CacheStats {
  l1Hits: number;
  l1Misses: number;
  l2Hits: number;
  l2Misses: number;
  sets: number;
  deletes: number;
  evictions: number;
  promotions: number;
  demotions: number;
}

export class MultiLevelCache implements CacheProvider {
  private l1Cache: Map<string, CacheEntry> = new Map();
  private l2Cache: RedisCache;
  private logger: Logger;
  private config: CacheConfig & {
    l1MaxSize?: number;
    l1MaxEntries?: number;
    promotionThreshold?: number;
    demotionThreshold?: number;
    enableMetrics?: boolean;
  };

  private stats: CacheStats = {
    l1Hits: 0,
    l1Misses: 0,
    l2Hits: 0,
    l2Misses: 0,
    sets: 0,
    deletes: 0,
    evictions: 0,
    promotions: 0,
    demotions: 0
  };

  private l1Size: number = 0;
  private accessPattern: Map<string, number> = new Map(); // Track access frequency

  constructor(
    config: CacheConfig & {
      l1MaxSize?: number;
      l1MaxEntries?: number;
      promotionThreshold?: number;
      demotionThreshold?: number;
      enableMetrics?: boolean;
    },
    logger?: Logger
  ) {
    this.config = {
      l1MaxSize: 100 * 1024 * 1024, // 100MB default
      l1MaxEntries: 10000,
      promotionThreshold: 3, // Promote after 3 accesses
      demotionThreshold: 300000, // Demote after 5 minutes of no access
      enableMetrics: true,
      ...config
    };

    this.logger = logger || new Logger("MultiLevelCache");
    this.l2Cache = new RedisCache(config, this.logger);

    // Start cleanup intervals
    this.startMaintenanceTasks();
  }

  /**
   * Initialize the cache system
   */
  async initialize(): Promise<void> {
    try {
      await this.l2Cache.initialize();
      this.logger.info("Multi-level cache initialized successfully");
    } catch (error) {
      this.logger.error("Failed to initialize multi-level cache", error);
      throw error;
    }
  }

  /**
   * Get a value from cache with multi-level lookup
   */
  async get<T>(key: string): Promise<CacheResult<T>> {
    const startTime = Date.now();

    try {
      // Check L1 cache first
      const l1Entry = this.l1Cache.get(key);
      if (l1Entry && !this.isExpired(l1Entry)) {
        // Update access patterns
        l1Entry.accessCount++;
        l1Entry.lastAccessed = Date.now();

        // Check if should promote to L2
        if (l1Entry.accessCount >= (this.config.promotionThreshold || 3)) {
          await this.promoteToL2(key, l1Entry);
        }

        this.stats.l1Hits++;
        this.recordAccessPattern(key);

        return {
          success: true,
          data: l1Entry.data,
          hit: true,
          duration: Date.now() - startTime
        };
      }

      // Check L2 cache
      const l2Result = await this.l2Cache.get<T>(key);
      if (l2Result.success && l2Result.hit && l2Result.data !== undefined) {
        // Promote to L1
        await this.promoteToL1(key, l2Result.data, l2Result.duration || 0);

        this.stats.l2Hits++;

        return {
          success: true,
          data: l2Result.data,
          hit: true,
          duration: Date.now() - startTime
        };
      }

      // If L2 cache operation failed, return the error
      if (!l2Result.success) {
        this.logger.warn("L2 cache operation failed", { key, error: l2Result.error });
        return {
          success: false,
          error: l2Result.error || "L2 cache operation failed",
          hit: false,
          duration: Date.now() - startTime
        };
      }

      // Cache miss
      this.stats.l1Misses++;
      this.stats.l2Misses++;

      return {
        success: true,
        hit: false,
        duration: Date.now() - startTime
      };

    } catch (error) {
      this.logger.error("Cache get operation failed", { key, error });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime
      };
    }
  }

  /**
   * Set a value in cache with intelligent placement
   */
  async set<T>(key: string, value: T, ttl?: number): Promise<CacheResult<boolean>> {
    const startTime = Date.now();

    try {
      const effectiveTtl = ttl || this.config.ttl || 3600;
      const entrySize = this.calculateSize(value);

      // Always set in L2
      const l2Result = await this.l2Cache.set(key, value, effectiveTtl);

      // If L2 set failed, return error
      if (!l2Result.success) {
        this.logger.warn("L2 cache set operation failed", { key, error: l2Result.error });
        return {
          success: false,
          error: l2Result.error || "L2 cache set operation failed",
          hit: false,
          duration: Date.now() - startTime
        };
      }

      // Set in L1 if it fits and isn't too large
      if (entrySize < (this.config.l1MaxSize || 100 * 1024 * 1024) / 10) { // Max 10% of L1 cache
        await this.setInL1(key, value, effectiveTtl, entrySize);
      }

      this.stats.sets++;

      return {
        success: true,
        data: true,
        hit: false,
        duration: Date.now() - startTime
      };

    } catch (error) {
      this.logger.error("Cache set operation failed", { key, error });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime
      };
    }
  }

  /**
   * Delete a value from all cache levels
   */
  async delete(key: string): Promise<CacheResult<boolean>> {
    const startTime = Date.now();

    try {
      // Delete from both levels
      const l1Deleted = this.l1Cache.delete(key);
      const l2Result = await this.l2Cache.delete(key);

      this.accessPattern.delete(key);
      this.stats.deletes++;

      const deleted = l1Deleted || (l2Result.success && l2Result.data);

      return {
        success: true,
        data: !!deleted,
        hit: false,
        duration: Date.now() - startTime
      };

    } catch (error) {
      this.logger.error("Cache delete operation failed", { key, error });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime
      };
    }
  }

  /**
   * Check if a key exists in any cache level
   */
  async exists(key: string): Promise<CacheResult<boolean>> {
    const startTime = Date.now();

    try {
      // Check L1 first
      const l1Entry = this.l1Cache.get(key);
      if (l1Entry && !this.isExpired(l1Entry)) {
        return {
          success: true,
          data: true,
          hit: true,
          duration: Date.now() - startTime
        };
      }

      // Check L2
      const l2Result = await this.l2Cache.exists(key);

      return {
        success: l2Result.success,
        data: l2Result.success && l2Result.data,
        hit: l2Result.hit,
        error: l2Result.error,
        duration: Date.now() - startTime
      };

    } catch (error) {
      this.logger.error("Cache exists operation failed", { key, error });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime
      };
    }
  }

  /**
   * Clear cache keys matching a pattern
   */
  async clear(pattern: string = "*"): Promise<CacheResult<number>> {
    const startTime = Date.now();

    try {
      // Clear L1 cache (simple implementation - clear all for patterns)
      if (pattern === "*") {
        const l1Cleared = this.l1Cache.size;
        this.l1Cache.clear();
        this.l1Size = 0;
        this.accessPattern.clear();
      } else {
        // For specific patterns, we'd need more complex logic
        // For now, just clear L1 completely
        this.l1Cache.clear();
        this.l1Size = 0;
        this.accessPattern.clear();
      }

      // Clear L2 cache
      const l2Result = await this.l2Cache.clear(pattern);

      const totalCleared = (pattern === "*" ? this.l1Cache.size : 0) + (l2Result.data || 0);

      return {
        success: true,
        data: totalCleared,
        hit: false,
        duration: Date.now() - startTime
      };

    } catch (error) {
      this.logger.error("Cache clear operation failed", { pattern, error });

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime
      };
    }
  }

  /**
   * Get comprehensive cache statistics
   */
  async getStats(): Promise<CacheResult<Record<string, any>>> {
    const startTime = Date.now();

    try {
      const l2Stats = await this.l2Cache.getStats();

      const totalRequests = this.stats.l1Hits + this.stats.l1Misses + this.stats.l2Hits + this.stats.l2Misses;
      const l1HitRate = totalRequests > 0 ? (this.stats.l1Hits / (this.stats.l1Hits + this.stats.l1Misses)) * 100 : 0;
      const l2HitRate = totalRequests > 0 ? (this.stats.l2Hits / (this.stats.l2Hits + this.stats.l2Misses)) * 100 : 0;
      const overallHitRate = totalRequests > 0 ? ((this.stats.l1Hits + this.stats.l2Hits) / totalRequests) * 100 : 0;

      const stats = {
        l1: {
          entries: this.l1Cache.size,
          sizeBytes: this.l1Size,
          maxSizeBytes: this.config.l1MaxSize,
          maxEntries: this.config.l1MaxEntries,
          hitRate: `${l1HitRate.toFixed(2)}%`,
          hits: this.stats.l1Hits,
          misses: this.stats.l1Misses
        },
        l2: l2Stats.success ? l2Stats.data : { error: l2Stats.error },
        overall: {
          hitRate: `${overallHitRate.toFixed(2)}%`,
          totalRequests,
          sets: this.stats.sets,
          deletes: this.stats.deletes,
          evictions: this.stats.evictions,
          promotions: this.stats.promotions,
          demotions: this.stats.demotions
        },
        config: {
          promotionThreshold: this.config.promotionThreshold,
          demotionThreshold: this.config.demotionThreshold,
          enableMetrics: this.config.enableMetrics
        }
      };

      return {
        success: true,
        data: stats,
        hit: false,
        duration: Date.now() - startTime
      };

    } catch (error) {
      this.logger.error("Failed to get cache stats", error);

      return {
        success: false,
        error: (error as Error).message,
        hit: false,
        duration: Date.now() - startTime
      };
    }
  }

  /**
   * Close the cache system
   */
  async close(): Promise<void> {
    try {
      await this.l2Cache.close();
      this.l1Cache.clear();
      this.accessPattern.clear();
      this.logger.info("Multi-level cache closed successfully");
    } catch (error) {
      this.logger.error("Error closing multi-level cache", error);
      throw new DataLayerError(
        "Failed to close multi-level cache",
        "CACHE_CLOSE_ERROR",
        "close",
        undefined,
        error as Error
      );
    }
  }

  // Private methods

  private async setInL1<T>(key: string, value: T, ttl: number, size: number): Promise<void> {
    // Check if we need to evict entries
    if (this.l1Size + size > (this.config.l1MaxSize || 100 * 1024 * 1024)) {
      this.evictL1Entries(size);
    }

    if (this.l1Cache.size >= (this.config.l1MaxEntries || 10000)) {
      this.evictL1Entries(1); // Evict at least one entry
    }

    const entry: CacheEntry<T> = {
      data: value,
      timestamp: Date.now(),
      ttl,
      accessCount: 1,
      lastAccessed: Date.now(),
      size
    };

    this.l1Cache.set(key, entry);
    this.l1Size += size;
  }

  private async promoteToL1<T>(key: string, value: T, l2Latency: number): Promise<void> {
    const size = this.calculateSize(value);
    const ttl = this.config.ttl || 3600;

    // Only promote if it's worth it (L2 was slower)
    if (l2Latency > 10) { // If L2 took more than 10ms
      await this.setInL1(key, value, ttl, size);
      this.stats.promotions++;
    }
  }

  private async promoteToL2<T>(key: string, entry: CacheEntry<T>): Promise<void> {
    try {
      await this.l2Cache.set(key, entry.data, entry.ttl);
      this.stats.promotions++;
    } catch (error) {
      this.logger.warn("Failed to promote entry to L2 cache", { key, error });
    }
  }

  private evictL1Entries(spaceNeeded: number): void {
    // Simple LRU eviction - in a real implementation, you'd use a more sophisticated algorithm
    const entries = Array.from(this.l1Cache.entries());

    // Sort by last accessed (oldest first)
    entries.sort(([, a], [, b]) => a.lastAccessed - b.lastAccessed);

    let freedSpace = 0;
    let evicted = 0;

    for (const [key, entry] of entries) {
      if (freedSpace >= spaceNeeded) break;

      this.l1Cache.delete(key);
      freedSpace += entry.size;
      evicted++;
    }

    this.l1Size -= freedSpace;
    this.stats.evictions += evicted;
  }

  private isExpired(entry: CacheEntry): boolean {
    return Date.now() - entry.timestamp > entry.ttl * 1000;
  }

  private calculateSize(value: any): number {
    // Rough estimation of memory usage
    const str = JSON.stringify(value);
    return str.length * 2; // Rough estimate: 2 bytes per character
  }

  private recordAccessPattern(key: string): void {
    const count = this.accessPattern.get(key) || 0;
    this.accessPattern.set(key, count + 1);
  }

  private startMaintenanceTasks(): void {
    // Clean up expired L1 entries every 5 minutes
    setInterval(() => {
      this.cleanupExpiredL1Entries();
    }, 5 * 60 * 1000);

    // Demote infrequently accessed L1 entries every 10 minutes
    setInterval(() => {
      this.demoteColdL1Entries();
    }, 10 * 60 * 1000);
  }

  private cleanupExpiredL1Entries(): void {
    const now = Date.now();
    let cleaned = 0;
    let freedSpace = 0;

    for (const [key, entry] of this.l1Cache.entries()) {
      if (this.isExpired(entry)) {
        this.l1Cache.delete(key);
        freedSpace += entry.size;
        cleaned++;
      }
    }

    if (cleaned > 0) {
      this.l1Size -= freedSpace;
      this.logger.debug(`Cleaned up ${cleaned} expired L1 entries, freed ${freedSpace} bytes`);
    }
  }

  private async demoteColdL1Entries(): Promise<void> {
    const now = Date.now();
    const threshold = this.config.demotionThreshold || 300000; // 5 minutes
    const coldEntries: Array<[string, CacheEntry]> = [];

    for (const [key, entry] of this.l1Cache.entries()) {
      if (now - entry.lastAccessed > threshold) {
        coldEntries.push([key, entry]);
      }
    }

    for (const [key, entry] of coldEntries) {
      // Move to L2 if accessed frequently enough
      if (entry.accessCount >= 2) {
        try {
          await this.l2Cache.set(key, entry.data, entry.ttl);
          this.stats.demotions++;
        } catch (error) {
          this.logger.warn("Failed to demote entry to L2", { key, error });
        }
      }

      // Remove from L1
      this.l1Cache.delete(key);
      this.l1Size -= entry.size;
    }

    if (coldEntries.length > 0) {
      this.logger.debug(`Demoted ${coldEntries.length} cold L1 entries to L2`);
    }
  }
}
