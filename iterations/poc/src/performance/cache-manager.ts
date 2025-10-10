/**
 * Advanced Cache Manager - Multi-level caching with intelligent eviction
 *
 * @author @darianrosebrook
 * @description Enterprise-grade caching with TTL, size limits, and performance monitoring
 */

import { Logger } from "../utils/Logger.js";

export interface CacheConfig {
  enabled: boolean;
  maxSize: number; // Maximum number of entries
  defaultTTL: number; // Default time-to-live in milliseconds
  compressionThreshold: number; // Compress entries larger than this (bytes)
  enableMetrics: boolean;
  evictionPolicy: "lru" | "lfu" | "ttl"; // Least Recently Used, Least Frequently Used, Time-based
}

export interface CacheEntry<T = any> {
  key: string;
  value: T;
  metadata: CacheMetadata;
}

export interface CacheMetadata {
  createdAt: number;
  lastAccessed: number;
  accessCount: number;
  size: number; // Estimated size in bytes
  ttl?: number; // Time-to-live override
  tags?: string[]; // For selective invalidation
  compressed?: boolean;
}

export interface CacheStats {
  totalEntries: number;
  totalSize: number;
  hits: number;
  misses: number;
  evictions: number;
  compressions: number;
  hitRate: number;
  averageAccessTime: number;
}

export class CacheManager {
  private config: CacheConfig;
  private cache: Map<string, CacheEntry> = new Map();
  private logger: Logger;
  private stats: CacheStats;

  constructor(config: CacheConfig, logger?: Logger) {
    this.config = config;
    this.logger = logger || new Logger("CacheManager");
    this.stats = {
      totalEntries: 0,
      totalSize: 0,
      hits: 0,
      misses: 0,
      evictions: 0,
      compressions: 0,
      hitRate: 0,
      averageAccessTime: 0,
    };

    if (config.enabled) {
      this.startMaintenanceCycle();
      this.logger.info("Cache manager initialized", {
        maxSize: config.maxSize,
        defaultTTL: config.defaultTTL,
        evictionPolicy: config.evictionPolicy,
      });
    }
  }

  async get<T = any>(key: string): Promise<T | null> {
    if (!this.config.enabled) return null;

    const startTime = Date.now();
    const entry = this.cache.get(key);

    if (!entry) {
      this.stats.misses++;
      this.updateHitRate();
      return null;
    }

    // Check if entry has expired
    if (this.isExpired(entry)) {
      this.delete(key);
      this.stats.misses++;
      this.updateHitRate();
      return null;
    }

    // Update access metadata
    entry.metadata.lastAccessed = Date.now();
    entry.metadata.accessCount++;

    // Decompress if necessary
    let value = entry.value;
    if (entry.metadata.compressed) {
      value = await this.decompress(entry.value);
    }

    const accessTime = Date.now() - startTime;
    this.stats.hits++;
    this.stats.averageAccessTime =
      (this.stats.averageAccessTime *
        (this.stats.hits + this.stats.misses - 1) +
        accessTime) /
      (this.stats.hits + this.stats.misses);

    this.updateHitRate();

    this.logger.debug("Cache hit", { key, accessTime });
    return value;
  }

  async set<T = any>(
    key: string,
    value: T,
    options: {
      ttl?: number;
      tags?: string[];
      skipCompression?: boolean;
    } = {}
  ): Promise<void> {
    if (!this.config.enabled) return;

    // Check if we need to evict entries to make room
    await this.ensureCapacity();

    // Compress if value is large enough
    let finalValue = value;
    let compressed = false;
    const size = this.estimateSize(value);

    if (!options.skipCompression && size > this.config.compressionThreshold) {
      finalValue = await this.compress(value);
      compressed = true;
      this.stats.compressions++;
      this.logger.debug("Compressed cache entry", { key, originalSize: size });
    }

    const entry: CacheEntry = {
      key,
      value: finalValue,
      metadata: {
        createdAt: Date.now(),
        lastAccessed: Date.now(),
        accessCount: 0,
        size: compressed ? this.estimateSize(finalValue) : size,
        ttl: options.ttl,
        tags: options.tags,
        compressed,
      },
    };

    this.cache.set(key, entry);
    this.stats.totalEntries++;
    this.stats.totalSize += entry.metadata.size;

    this.logger.debug("Cache entry stored", {
      key,
      size: entry.metadata.size,
      compressed,
      ttl: options.ttl,
    });
  }

  delete(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;

    this.cache.delete(key);
    this.stats.totalEntries--;
    this.stats.totalSize -= entry.metadata.size;

    this.logger.debug("Cache entry deleted", { key });
    return true;
  }

  clear(): void {
    this.cache.clear();
    this.stats.totalEntries = 0;
    this.stats.totalSize = 0;
    this.stats.evictions = 0;

    this.logger.info("Cache cleared");
  }

  invalidateByTag(tag: string): number {
    let deletedCount = 0;
    for (const [key, entry] of this.cache.entries()) {
      if (entry.metadata.tags?.includes(tag)) {
        this.delete(key);
        deletedCount++;
      }
    }

    if (deletedCount > 0) {
      this.logger.info("Cache entries invalidated by tag", {
        tag,
        deletedCount,
      });
    }

    return deletedCount;
  }

  invalidateByPattern(pattern: string): number {
    let deletedCount = 0;
    const regex = new RegExp(pattern);

    for (const [key, _entry] of this.cache.entries()) {
      if (regex.test(key)) {
        this.delete(key);
        deletedCount++;
      }
    }

    if (deletedCount > 0) {
      this.logger.info("Cache entries invalidated by pattern", {
        pattern,
        deletedCount,
      });
    }

    return deletedCount;
  }

  getStats(): CacheStats {
    return { ...this.stats };
  }

  getKeys(): string[] {
    return Array.from(this.cache.keys());
  }

  has(key: string): boolean {
    const entry = this.cache.get(key);
    return entry !== undefined && !this.isExpired(entry);
  }

  private isExpired(entry: CacheEntry): boolean {
    const ttl = entry.metadata.ttl || this.config.defaultTTL;
    return Date.now() - entry.metadata.createdAt > ttl;
  }

  private async ensureCapacity(): Promise<void> {
    while (this.cache.size >= this.config.maxSize) {
      await this.evictEntry();
    }
  }

  private async evictEntry(): Promise<void> {
    if (this.cache.size === 0) return;

    let entryToEvict: CacheEntry | null = null;
    let evictKey = "";

    switch (this.config.evictionPolicy) {
      case "lru": // Least Recently Used
        for (const [key, entry] of this.cache.entries()) {
          if (
            !entryToEvict ||
            entry.metadata.lastAccessed < entryToEvict.metadata.lastAccessed
          ) {
            entryToEvict = entry;
            evictKey = key;
          }
        }
        break;

      case "lfu": // Least Frequently Used
        for (const [key, entry] of this.cache.entries()) {
          if (
            !entryToEvict ||
            entry.metadata.accessCount < entryToEvict.metadata.accessCount
          ) {
            entryToEvict = entry;
            evictKey = key;
          }
        }
        break;

      case "ttl": // First to expire
        for (const [key, entry] of this.cache.entries()) {
          const entryExpiry =
            entry.metadata.createdAt +
            (entry.metadata.ttl || this.config.defaultTTL);
          const currentExpiry = entryToEvict
            ? entryToEvict.metadata.createdAt +
              (entryToEvict.metadata.ttl || this.config.defaultTTL)
            : Infinity;

          if (entryExpiry < currentExpiry) {
            entryToEvict = entry;
            evictKey = key;
          }
        }
        break;
    }

    if (entryToEvict) {
      this.delete(evictKey);
      this.stats.evictions++;
      this.logger.debug("Cache entry evicted", {
        key: evictKey,
        policy: this.config.evictionPolicy,
      });
    }
  }

  private estimateSize(value: any): number {
    // Rough estimation of object size in bytes
    const jsonString = JSON.stringify(value);
    return jsonString.length * 2; // Rough estimate: 2 bytes per character for UTF-16
  }

  private async compress(value: any): Promise<any> {
    // Simple compression simulation - in production, use proper compression
    // For now, just return the value (real compression would be implemented here)
    return value;
  }

  private async decompress(value: any): Promise<any> {
    // Simple decompression simulation - in production, use proper decompression
    // For now, just return the value (real decompression would be implemented here)
    return value;
  }

  private updateHitRate(): void {
    const totalRequests = this.stats.hits + this.stats.misses;
    if (totalRequests > 0) {
      this.stats.hitRate = this.stats.hits / totalRequests;
    }
  }

  private startMaintenanceCycle(): void {
    // Run maintenance every 5 minutes
    setInterval(async () => {
      await this.performMaintenance();
    }, 5 * 60 * 1000);
  }

  private async performMaintenance(): Promise<void> {
    this.logger.debug("Starting cache maintenance");

    // Remove expired entries
    const expiredKeys: string[] = [];
    for (const [key, entry] of this.cache.entries()) {
      if (this.isExpired(entry)) {
        expiredKeys.push(key);
      }
    }

    expiredKeys.forEach((key) => this.delete(key));

    if (expiredKeys.length > 0) {
      this.logger.info("Cache maintenance completed", {
        expiredRemoved: expiredKeys.length,
        currentSize: this.cache.size,
      });
    }
  }
}
