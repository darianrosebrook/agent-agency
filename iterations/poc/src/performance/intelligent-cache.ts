/**
 * Intelligent Cache System
 *
 * Advanced caching with LRU eviction, semantic similarity, and performance optimization.
 * Learns access patterns and predicts future cache needs.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";

export interface CacheEntry<T = any> {
  key: string;
  value: T;
  metadata: {
    created: Date;
    lastAccessed: Date;
    accessCount: number;
    size: number; // bytes
    ttl?: number; // time to live in ms
    tags: string[];
    priority: "low" | "medium" | "high" | "critical";
  };
  accessPattern: {
    frequency: number; // accesses per hour
    temporalPattern: "constant" | "bursty" | "periodic" | "declining";
    predictedNextAccess?: Date;
  };
}

export interface CacheMetrics {
  timestamp: Date;
  totalEntries: number;
  totalSize: number;
  hitRate: number;
  missRate: number;
  evictionRate: number;
  averageAccessTime: number;
  cacheUtilization: number; // percentage
}

export interface CacheStrategy {
  name: string;
  evictionPolicy: "lru" | "lfu" | "size-based" | "ttl-based" | "hybrid";
  maxSize: number; // bytes
  maxEntries: number;
  compressionEnabled: boolean;
  prefetchEnabled: boolean;
}

export interface CachePrediction {
  key: string;
  confidence: number;
  predictedAccessTime: Date;
  reason: string;
}

/**
 * Intelligent Cache with learning and prediction capabilities
 */
export class IntelligentCache<T = any> extends EventEmitter {
  private cache = new Map<string, CacheEntry<T>>();
  private accessHistory: Array<{ key: string; timestamp: Date; hit: boolean }> =
    [];
  private metrics: CacheMetrics[] = [];
  private strategy: CacheStrategy;
  private predictions: CachePrediction[] = [];

  constructor(strategy: Partial<CacheStrategy> = {}) {
    super();

    this.strategy = {
      name: "intelligent-lru",
      evictionPolicy: "hybrid",
      maxSize: 100 * 1024 * 1024, // 100MB
      maxEntries: 10000,
      compressionEnabled: true,
      prefetchEnabled: true,
      ...strategy,
    };

    // Start metrics collection
    this.startMetricsCollection();
  }

  /**
   * Get a value from cache with intelligent prefetching
   */
  async get(key: string): Promise<T | null> {
    const entry = this.cache.get(key);

    if (entry) {
      // Cache hit
      this.recordAccess(key, true);
      this.updateEntryMetadata(entry, true);

      // Check TTL
      if (this.isExpired(entry)) {
        this.cache.delete(key);
        this.emit("expired", { key, entry });
        return null;
      }

      // Trigger prefetching for related items
      if (this.strategy.prefetchEnabled) {
        this.prefetchRelatedItems(key, entry);
      }

      return entry.value;
    }

    // Cache miss
    this.recordAccess(key, false);
    this.emit("miss", { key });

    return null;
  }

  /**
   * Set a value in cache with intelligent metadata
   */
  async set(
    key: string,
    value: T,
    options: Partial<CacheEntry["metadata"]> = {}
  ): Promise<void> {
    const size = this.calculateSize(value);
    const existingEntry = this.cache.get(key);

    // Check if we need to evict entries
    if (!existingEntry) {
      await this.ensureCapacity(size);
    }

    const entry: CacheEntry<T> = {
      key,
      value,
      metadata: {
        created: new Date(),
        lastAccessed: new Date(),
        accessCount: 0,
        size,
        tags: [],
        priority: "medium",
        ...options,
      },
      accessPattern: {
        frequency: 0,
        temporalPattern: "constant",
      },
    };

    this.cache.set(key, entry);
    this.updateEntryMetadata(entry, false);

    // Learn access patterns for this key
    this.learnAccessPattern(key);

    this.emit("set", { key, entry });
  }

  /**
   * Delete a key from cache
   */
  async delete(key: string): Promise<boolean> {
    const deleted = this.cache.delete(key);
    if (deleted) {
      this.emit("delete", { key });
    }
    return deleted;
  }

  /**
   * Clear all cache entries
   */
  async clear(): Promise<void> {
    this.cache.clear();
    this.emit("clear");
  }

  /**
   * Get cache statistics and predictions
   */
  getStats(): {
    entries: number;
    totalSize: number;
    hitRate: number;
    utilization: number;
    predictions: CachePrediction[];
    topAccessed: Array<{ key: string; accesses: number }>;
    evictionCandidates: string[];
  } {
    const entries = Array.from(this.cache.values());
    const totalSize = entries.reduce(
      (sum, entry) => sum + entry.metadata.size,
      0
    );
    const utilization = (totalSize / this.strategy.maxSize) * 100;

    // Calculate hit rate from recent history
    const recentHistory = this.accessHistory.slice(-1000);
    const hits = recentHistory.filter((h) => h.hit).length;
    const hitRate =
      recentHistory.length > 0 ? (hits / recentHistory.length) * 100 : 0;

    // Top accessed entries
    const accessCounts = new Map<string, number>();
    recentHistory.forEach((h) => {
      accessCounts.set(h.key, (accessCounts.get(h.key) || 0) + 1);
    });

    const topAccessed = Array.from(accessCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10)
      .map(([key, accesses]) => ({ key, accesses }));

    // Eviction candidates (least recently used with low priority)
    const evictionCandidates = entries
      .sort((a, b) => {
        // Sort by priority, then by last access time
        const priorityOrder = { critical: 4, high: 3, medium: 2, low: 1 };
        const priorityDiff =
          priorityOrder[b.metadata.priority] -
          priorityOrder[a.metadata.priority];
        if (priorityDiff !== 0) return priorityDiff;
        return (
          a.metadata.lastAccessed.getTime() - b.metadata.lastAccessed.getTime()
        );
      })
      .slice(0, 10)
      .map((entry) => entry.key);

    return {
      entries: this.cache.size,
      totalSize,
      hitRate,
      utilization,
      predictions: this.predictions.slice(0, 5),
      topAccessed,
      evictionCandidates,
    };
  }

  /**
   * Prefetch related items based on access patterns
   */
  private async prefetchRelatedItems(
    key: string,
    entry: CacheEntry<T>
  ): Promise<void> {
    // Find semantically related keys
    const relatedKeys = this.findRelatedKeys(key, entry);

    for (const relatedKey of relatedKeys) {
      if (!this.cache.has(relatedKey)) {
        // Emit prefetch request - actual prefetching logic would be implemented
        // by the cache consumer (e.g., data layer)
        this.emit("prefetch-request", {
          key: relatedKey,
          triggeredBy: key,
          confidence: 0.7,
        });
      }
    }
  }

  /**
   * Find keys related to the current key
   */
  private findRelatedKeys(key: string, entry: CacheEntry<T>): string[] {
    const relatedKeys: string[] = [];

    // Tag-based relationships
    for (const [cacheKey, cacheEntry] of this.cache) {
      if (cacheKey !== key) {
        const commonTags = entry.metadata.tags.filter((tag) =>
          cacheEntry.metadata.tags.includes(tag)
        );
        if (commonTags.length > 0) {
          relatedKeys.push(cacheKey);
        }
      }
    }

    // Pattern-based relationships (e.g., file paths, API endpoints)
    const keyParts = key.split(/[/\-_.]/);
    for (const [cacheKey] of this.cache) {
      if (cacheKey !== key) {
        const cacheKeyParts = cacheKey.split(/[/\-_.]/);
        const commonParts = keyParts.filter((part) =>
          cacheKeyParts.includes(part)
        );
        if (commonParts.length >= 2) {
          // At least 2 common parts
          relatedKeys.push(cacheKey);
        }
      }
    }

    return relatedKeys.slice(0, 5); // Limit prefetch candidates
  }

  /**
   * Learn access patterns for predictive caching
   */
  private learnAccessPattern(key: string): void {
    const history = this.accessHistory.filter((h) => h.key === key).slice(-50); // Last 50 accesses

    if (history.length < 5) return; // Need minimum data

    const intervals: number[] = [];
    for (let i = 1; i < history.length; i++) {
      intervals.push(
        history[i].timestamp.getTime() - history[i - 1].timestamp.getTime()
      );
    }

    const avgInterval = intervals.reduce((a, b) => a + b) / intervals.length;
    const variance =
      intervals.reduce(
        (sum, interval) => sum + Math.pow(interval - avgInterval, 2),
        0
      ) / intervals.length;
    const stdDev = Math.sqrt(variance);

    // Determine temporal pattern
    let pattern: CacheEntry["accessPattern"]["temporalPattern"] = "constant";

    if (stdDev / avgInterval > 0.5) {
      pattern = "bursty"; // High variance = bursty
    } else if (this.detectPeriodicity(intervals)) {
      pattern = "periodic";
    }

    // Calculate access frequency (accesses per hour)
    const timeSpan =
      history[history.length - 1].timestamp.getTime() -
      history[0].timestamp.getTime();
    const hours = timeSpan / (1000 * 60 * 60);
    const frequency = hours > 0 ? history.length / hours : 0;

    // Update entry if it exists
    const entry = this.cache.get(key);
    if (entry) {
      entry.accessPattern.frequency = frequency;
      entry.accessPattern.temporalPattern = pattern;

      // Predict next access
      if (pattern === "periodic" && avgInterval > 0) {
        entry.accessPattern.predictedNextAccess = new Date(
          Date.now() + avgInterval
        );
      }
    }

    // Generate predictions for prefetching
    this.generatePredictions();
  }

  /**
   * Detect periodicity in access intervals
   */
  private detectPeriodicity(intervals: number[]): boolean {
    if (intervals.length < 10) return false;

    // Simple autocorrelation check
    const mean = intervals.reduce((a, b) => a + b) / intervals.length;
    const autocorr =
      intervals
        .slice(1)
        .reduce(
          (sum, interval, i) => sum + (intervals[i] - mean) * (interval - mean),
          0
        ) / intervals.length;

    // If autocorrelation is significant, likely periodic
    return autocorr > mean * 0.3;
  }

  /**
   * Generate predictions for future cache needs
   */
  private generatePredictions(): void {
    this.predictions = [];

    for (const [key, entry] of this.cache) {
      if (entry.accessPattern.predictedNextAccess) {
        const timeUntilAccess =
          entry.accessPattern.predictedNextAccess.getTime() - Date.now();
        const confidence = Math.max(0, 1 - timeUntilAccess / (1000 * 60 * 60)); // Higher confidence for imminent access

        if (confidence > 0.3) {
          this.predictions.push({
            key,
            confidence,
            predictedAccessTime: entry.accessPattern.predictedNextAccess,
            reason: `${entry.accessPattern.temporalPattern} access pattern`,
          });
        }
      }
    }

    // Sort by confidence and recency
    this.predictions.sort((a, b) => {
      if (Math.abs(a.confidence - b.confidence) > 0.1) {
        return b.confidence - a.confidence;
      }
      return a.predictedAccessTime.getTime() - b.predictedAccessTime.getTime();
    });

    this.predictions = this.predictions.slice(0, 20); // Keep top 20 predictions
  }

  /**
   * Ensure cache has capacity for new entry
   */
  private async ensureCapacity(newEntrySize: number): Promise<void> {
    const currentSize = Array.from(this.cache.values()).reduce(
      (sum, entry) => sum + entry.metadata.size,
      0
    );

    // Check size limit
    if (currentSize + newEntrySize > this.strategy.maxSize) {
      await this.evictEntries(
        currentSize + newEntrySize - this.strategy.maxSize
      );
    }

    // Check entry count limit
    if (this.cache.size >= this.strategy.maxEntries) {
      await this.evictEntriesByCount(
        this.cache.size - this.strategy.maxEntries + 1
      );
    }
  }

  /**
   * Evict entries to free up space
   */
  private async evictEntries(bytesNeeded: number): Promise<void> {
    let freedBytes = 0;
    const entriesToEvict: string[] = [];

    // Sort entries by eviction priority
    const sortedEntries = Array.from(this.cache.entries()).sort(
      ([, a], [, b]) =>
        this.calculateEvictionScore(a) - this.calculateEvictionScore(b)
    );

    for (const [key, entry] of sortedEntries) {
      entriesToEvict.push(key);
      freedBytes += entry.metadata.size;

      if (freedBytes >= bytesNeeded) break;
    }

    // Evict selected entries
    for (const key of entriesToEvict) {
      this.cache.delete(key);
      this.emit("evicted", { key, reason: "capacity" });
    }
  }

  /**
   * Evict entries by count
   */
  private async evictEntriesByCount(count: number): Promise<void> {
    const entriesToEvict = Array.from(this.cache.entries())
      .sort(
        ([, a], [, b]) =>
          this.calculateEvictionScore(a) - this.calculateEvictionScore(b)
      )
      .slice(0, count)
      .map(([key]) => key);

    for (const key of entriesToEvict) {
      this.cache.delete(key);
      this.emit("evicted", { key, reason: "count-limit" });
    }
  }

  /**
   * Calculate eviction score (lower = more likely to be evicted)
   */
  private calculateEvictionScore(entry: CacheEntry): number {
    const now = Date.now();
    const age = now - entry.metadata.created.getTime();
    const timeSinceAccess = now - entry.metadata.lastAccessed.getTime();

    let score = 0;

    // Base score from access recency (more recent = higher score)
    score += (1000 * 60 * 60 * 24) / (timeSinceAccess + 1); // Avoid division by zero

    // Adjust for access frequency
    score += entry.accessPattern.frequency * 10;

    // Adjust for priority
    const priorityMultiplier = { low: 0.5, medium: 1, high: 2, critical: 10 };
    score *= priorityMultiplier[entry.metadata.priority];

    // Penalize for size (larger items more likely to be evicted)
    score /= Math.log(entry.metadata.size + 1);

    return score;
  }

  /**
   * Check if entry is expired
   */
  private isExpired(entry: CacheEntry): boolean {
    if (!entry.metadata.ttl) return false;
    return Date.now() - entry.metadata.created.getTime() > entry.metadata.ttl;
  }

  /**
   * Update entry metadata on access
   */
  private updateEntryMetadata(entry: CacheEntry, isHit: boolean): void {
    entry.metadata.lastAccessed = new Date();
    if (isHit) {
      entry.metadata.accessCount++;
    }
  }

  /**
   * Record access for analytics
   */
  private recordAccess(key: string, hit: boolean): void {
    this.accessHistory.push({
      key,
      timestamp: new Date(),
      hit,
    });

    // Keep only recent history
    if (this.accessHistory.length > 10000) {
      this.accessHistory = this.accessHistory.slice(-5000);
    }
  }

  /**
   * Calculate approximate size of value
   */
  private calculateSize(value: any): number {
    try {
      const str = JSON.stringify(value);
      return Buffer.byteLength(str, "utf8");
    } catch {
      // Fallback for non-serializable objects
      return 1000; // 1KB estimate
    }
  }

  /**
   * Start periodic metrics collection
   */
  private startMetricsCollection(): void {
    setInterval(() => {
      const stats = this.getStats();
      const metrics: CacheMetrics = {
        timestamp: new Date(),
        totalEntries: stats.entries,
        totalSize: stats.totalSize,
        hitRate: stats.hitRate,
        missRate: 100 - stats.hitRate,
        evictionRate: 0, // Would need to track evictions over time
        averageAccessTime: 5, // Simulated - would need actual measurement
        cacheUtilization: stats.utilization,
      };

      this.metrics.push(metrics);

      // Keep only recent metrics
      if (this.metrics.length > 1000) {
        this.metrics = this.metrics.slice(-500);
      }

      this.emit("metrics", metrics);
    }, 60000); // Every minute
  }

  /**
   * Get recent metrics
   */
  getRecentMetrics(minutes: number = 5): CacheMetrics[] {
    const cutoff = Date.now() - minutes * 60 * 1000;
    return this.metrics.filter((m) => m.timestamp.getTime() > cutoff);
  }

  /**
   * Optimize cache based on usage patterns
   */
  async optimize(): Promise<{
    optimizations: string[];
    performance: { hitRate: number; utilization: number };
  }> {
    const optimizations: string[] = [];
    const stats = this.getStats();

    // Analyze and suggest optimizations
    if (stats.hitRate < 70) {
      optimizations.push(
        "Consider increasing cache size or adjusting TTL values"
      );
      optimizations.push(
        "Implement more aggressive prefetching for frequently accessed items"
      );
    }

    if (stats.utilization > 90) {
      optimizations.push(
        "Cache is near capacity - consider increasing maxSize or implementing compression"
      );
    }

    if (stats.hitRate > 95 && stats.utilization < 50) {
      optimizations.push(
        "Cache is underutilized - consider reducing maxSize to save memory"
      );
    }

    // Check for patterns that could benefit from optimization
    const burstyEntries = Array.from(this.cache.values()).filter(
      (entry) => entry.accessPattern.temporalPattern === "bursty"
    ).length;

    if (burstyEntries > this.cache.size * 0.3) {
      optimizations.push(
        "High number of bursty access patterns - consider adaptive TTL policies"
      );
    }

    return {
      optimizations,
      performance: {
        hitRate: stats.hitRate,
        utilization: stats.utilization,
      },
    };
  }
}
