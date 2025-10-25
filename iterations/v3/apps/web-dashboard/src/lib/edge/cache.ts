/**
 * Edge Caching System
 * Optimized caching for API responses and static data
 */

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
  etag?: string | undefined;
}

interface CacheConfig {
  defaultTTL: number;
  maxSize: number;
  cleanupInterval: number;
}

class EdgeCache {
  private cache = new Map<string, CacheEntry<any>>();
  private config: CacheConfig;
  private cleanupTimer: NodeJS.Timeout | null = null;

  constructor(config: Partial<CacheConfig> = {}) {
    this.config = {
      defaultTTL: 30000, // 30 seconds
      maxSize: 1000,
      cleanupInterval: 60000, // 1 minute
      ...config,
    };

    this.startCleanup();
  }

  /**
   * Get cached data
   */
  get<T>(key: string): T | null {
    const entry = this.cache.get(key);
    
    if (!entry) {
      return null;
    }

    // Check if expired
    if (Date.now() - entry.timestamp > entry.ttl) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  /**
   * Set cached data
   */
  set<T>(key: string, data: T, ttl?: number, etag?: string): void {
    // Clean up if cache is full
    if (this.cache.size >= this.config.maxSize) {
      this.cleanup();
    }

    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl: ttl || this.config.defaultTTL,
      etag,
    });
  }

  /**
   * Check if key exists and is valid
   */
  has(key: string): boolean {
    const entry = this.cache.get(key);
    return entry ? Date.now() - entry.timestamp <= entry.ttl : false;
  }

  /**
   * Delete specific key
   */
  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Clear all cache
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get cache statistics
   */
  getStats() {
    const now = Date.now();
    const entries = Array.from(this.cache.values());
    
    return {
      totalEntries: this.cache.size,
      validEntries: entries.filter(entry => 
        now - entry.timestamp <= entry.ttl
      ).length,
      expiredEntries: entries.filter(entry => 
        now - entry.timestamp > entry.ttl
      ).length,
      memoryUsage: this.estimateMemoryUsage(),
    };
  }

  /**
   * Clean up expired entries
   */
  private cleanup(): void {
    const now = Date.now();
    const expiredKeys: string[] = [];

    this.cache.forEach((entry, key) => {
      if (now - entry.timestamp > entry.ttl) {
        expiredKeys.push(key);
      }
    });

    expiredKeys.forEach(key => this.cache.delete(key));
  }

  /**
   * Start automatic cleanup
   */
  private startCleanup(): void {
    this.cleanupTimer = setInterval(() => {
      this.cleanup();
    }, this.config.cleanupInterval);
  }

  /**
   * Stop cleanup timer
   */
  destroy(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = null;
    }
    this.clear();
  }

  /**
   * Estimate memory usage
   */
  private estimateMemoryUsage(): number {
    let totalSize = 0;
    
    this.cache.forEach((entry, key) => {
      totalSize += key.length * 2; // Unicode characters
      totalSize += JSON.stringify(entry.data).length * 2;
      totalSize += 32; // Entry overhead
    });
    
    return totalSize;
  }
}

// Cache strategies
export const CacheStrategies = {
  // Short-term cache for frequently changing data
  SHORT: { ttl: 10000, maxSize: 100 },
  
  // Medium-term cache for moderately changing data
  MEDIUM: { ttl: 30000, maxSize: 500 },
  
  // Long-term cache for rarely changing data
  LONG: { ttl: 300000, maxSize: 1000 },
  
  // Real-time cache for live data
  REALTIME: { ttl: 5000, maxSize: 50 },
} as const;

// Specialized caches
export const metricsCache = new EdgeCache({
  ...CacheStrategies.REALTIME,
  cleanupInterval: 30000,
});

export const tasksCache = new EdgeCache({
  ...CacheStrategies.MEDIUM,
  cleanupInterval: 60000,
});

export const alertsCache = new EdgeCache({
  ...CacheStrategies.SHORT,
  cleanupInterval: 30000,
});

export const staticCache = new EdgeCache({
  ...CacheStrategies.LONG,
  cleanupInterval: 300000,
});

// Cache utilities
export function createCacheKey(prefix: string, params: Record<string, any>): string {
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  return `${prefix}:${sortedParams}`;
}

export function withCache<T>(
  cache: EdgeCache,
  key: string,
  fetcher: () => Promise<T>,
  ttl?: number
): Promise<T> {
  // Check cache first
  const cached = cache.get<T>(key);
  if (cached !== null) {
    return Promise.resolve(cached);
  }

  // Fetch and cache
  return fetcher().then(data => {
    cache.set(key, data, ttl);
    return data;
  });
}

export default EdgeCache;
