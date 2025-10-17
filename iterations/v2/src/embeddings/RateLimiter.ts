/**
 * @fileoverview Rate Limiting Implementation
 *
 * Prevents API quota exhaustion and ensures fair resource usage
 * through token bucket and sliding window algorithms.
 *
 * @author @darianrosebrook
 */

export interface RateLimitConfig {
  /** Maximum requests per time window */
  maxRequests: number;
  /** Time window in milliseconds */
  windowMs: number;
  /** Whether to use sliding window (more accurate but memory intensive) */
  sliding?: boolean;
  /** Key strategy for identifying clients */
  keyStrategy?: "ip" | "user" | "endpoint" | "custom";
  /** Custom key function */
  keyFn?: (request: any) => string;
}

export interface RateLimitResult {
  allowed: boolean;
  remaining: number;
  resetTime: number;
  limit: number;
  windowMs: number;
}

export interface RateLimitStats {
  totalRequests: number;
  allowedRequests: number;
  blockedRequests: number;
  currentWindowStart: number;
  windowRequests: number;
}

/**
 * Token Bucket Rate Limiter
 * Allows bursts but maintains average rate
 */
export class TokenBucketRateLimiter {
  private buckets: Map<
    string,
    {
      tokens: number;
      lastRefill: number;
      capacity: number;
      refillRate: number; // tokens per millisecond
    }
  > = new Map();

  constructor(private capacity: number, private refillRatePerSecond: number) {}

  /**
   * Check if request is allowed and consume token
   */
  consume(key: string): RateLimitResult {
    const now = Date.now();
    let bucket = this.buckets.get(key);

    if (!bucket) {
      bucket = {
        tokens: this.capacity,
        lastRefill: now,
        capacity: this.capacity,
        refillRate: this.refillRatePerSecond / 1000, // per millisecond
      };
      this.buckets.set(key, bucket);
    }

    // Refill tokens based on time passed
    const timePassed = now - bucket.lastRefill;
    const tokensToAdd = timePassed * bucket.refillRate;
    bucket.tokens = Math.min(bucket.capacity, bucket.tokens + tokensToAdd);
    bucket.lastRefill = now;

    if (bucket.tokens >= 1) {
      bucket.tokens -= 1;
      return {
        allowed: true,
        remaining: Math.floor(bucket.tokens),
        resetTime: now + Math.ceil((1 - bucket.tokens) / bucket.refillRate),
        limit: bucket.capacity,
        windowMs: 1000 / bucket.refillRate,
      };
    }

    return {
      allowed: false,
      remaining: 0,
      resetTime: now + Math.ceil(1 / bucket.refillRate),
      limit: bucket.capacity,
      windowMs: 1000 / bucket.refillRate,
    };
  }

  /**
   * Get current bucket stats
   */
  getStats(key: string): RateLimitStats | null {
    const bucket = this.buckets.get(key);
    if (!bucket) return null;

    const now = Date.now();
    const timePassed = now - bucket.lastRefill;
    const tokensToAdd = timePassed * (this.refillRatePerSecond / 1000);
    const currentTokens = Math.min(
      bucket.capacity,
      bucket.tokens + tokensToAdd
    );

    return {
      totalRequests: bucket.capacity - currentTokens, // approximate
      allowedRequests: bucket.capacity - currentTokens,
      blockedRequests: 0, // not tracked in token bucket
      currentWindowStart: bucket.lastRefill,
      windowRequests: bucket.capacity - currentTokens,
    };
  }

  /**
   * Reset bucket for key
   */
  reset(key: string): void {
    this.buckets.delete(key);
  }

  /**
   * Cleanup old buckets (memory management)
   */
  cleanup(maxAge: number = 3600000): void {
    // 1 hour default
    const now = Date.now();
    for (const [key, bucket] of this.buckets.entries()) {
      if (now - bucket.lastRefill > maxAge) {
        this.buckets.delete(key);
      }
    }
  }
}

/**
 * Fixed Window Rate Limiter
 * Simple but allows bursts at window boundaries
 */
export class FixedWindowRateLimiter {
  private windows: Map<
    string,
    {
      count: number;
      windowStart: number;
    }
  > = new Map();

  constructor(private maxRequests: number, private windowMs: number) {}

  /**
   * Check if request is allowed
   */
  consume(key: string): RateLimitResult {
    const now = Date.now();
    const windowStart = Math.floor(now / this.windowMs) * this.windowMs;
    const windowKey = `${key}:${windowStart}`;

    let window = this.windows.get(windowKey);
    if (!window || window.windowStart !== windowStart) {
      window = { count: 0, windowStart };
      this.windows.set(windowKey, window);
    }

    const allowed = window.count < this.maxRequests;
    if (allowed) {
      window.count++;
    }

    return {
      allowed,
      remaining: Math.max(0, this.maxRequests - window.count),
      resetTime: windowStart + this.windowMs,
      limit: this.maxRequests,
      windowMs: this.windowMs,
    };
  }

  /**
   * Get current window stats
   */
  getStats(key: string): RateLimitStats | null {
    const now = Date.now();
    const windowStart = Math.floor(now / this.windowMs) * this.windowMs;
    const windowKey = `${key}:${windowStart}`;

    const window = this.windows.get(windowKey);
    if (!window) return null;

    return {
      totalRequests: window.count,
      allowedRequests: Math.min(window.count, this.maxRequests),
      blockedRequests: Math.max(0, window.count - this.maxRequests),
      currentWindowStart: window.windowStart,
      windowRequests: window.count,
    };
  }

  /**
   * Reset window for key
   */
  reset(key: string): void {
    const now = Date.now();
    const windowStart = Math.floor(now / this.windowMs) * this.windowMs;
    const windowKey = `${key}:${windowStart}`;
    this.windows.delete(windowKey);
  }

  /**
   * Cleanup old windows (memory management)
   */
  cleanup(maxAge: number = 3600000): void {
    // 1 hour default
    const now = Date.now();
    for (const [key, window] of this.windows.entries()) {
      if (now - window.windowStart > maxAge) {
        this.windows.delete(key);
      }
    }
  }
}

/**
 * Sliding Window Rate Limiter
 * More accurate but memory intensive
 */
export class SlidingWindowRateLimiter {
  private requests: Map<string, number[]> = new Map();

  constructor(private maxRequests: number, private windowMs: number) {}

  /**
   * Check if request is allowed
   */
  consume(key: string): RateLimitResult {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    let timestamps = this.requests.get(key) || [];
    // Remove old timestamps outside the window
    timestamps = timestamps.filter((ts) => ts > windowStart);

    const allowed = timestamps.length < this.maxRequests;
    if (allowed) {
      timestamps.push(now);
      this.requests.set(key, timestamps);
    }

    return {
      allowed,
      remaining: Math.max(0, this.maxRequests - timestamps.length),
      resetTime: now + this.windowMs,
      limit: this.maxRequests,
      windowMs: this.windowMs,
    };
  }

  /**
   * Get current window stats
   */
  getStats(key: string): RateLimitStats | null {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    const timestamps = this.requests.get(key) || [];
    const currentWindow = timestamps.filter((ts) => ts > windowStart);

    return {
      totalRequests: timestamps.length,
      allowedRequests: currentWindow.length,
      blockedRequests: timestamps.length - currentWindow.length,
      currentWindowStart: windowStart,
      windowRequests: currentWindow.length,
    };
  }

  /**
   * Reset data for key
   */
  reset(key: string): void {
    this.requests.delete(key);
  }

  /**
   * Cleanup old timestamps (memory management)
   */
  cleanup(maxAge: number = 3600000): void {
    // 1 hour default
    const cutoff = Date.now() - maxAge;
    for (const [key, timestamps] of this.requests.entries()) {
      const filtered = timestamps.filter((ts) => ts > cutoff);
      if (filtered.length === 0) {
        this.requests.delete(key);
      } else {
        this.requests.set(key, filtered);
      }
    }
  }
}

/**
 * Rate Limiter Factory with presets
 */
export class RateLimiterFactory {
  /** Conservative rate limiting for external APIs */
  static createForExternalAPI(): TokenBucketRateLimiter {
    return new TokenBucketRateLimiter(100, 10); // 100 tokens, refill 10/sec
  }

  /** Balanced rate limiting for embedding services */
  static createForEmbeddings(): TokenBucketRateLimiter {
    return new TokenBucketRateLimiter(1000, 50); // 1000 tokens, refill 50/sec
  }

  /** Aggressive rate limiting for high-throughput services */
  static createForHighThroughput(): TokenBucketRateLimiter {
    return new TokenBucketRateLimiter(5000, 200); // 5000 tokens, refill 200/sec
  }

  /** Strict rate limiting for sensitive operations */
  static createForSensitiveOps(): FixedWindowRateLimiter {
    return new FixedWindowRateLimiter(10, 60000); // 10 requests per minute
  }
}

/**
 * Rate limiting middleware for embedding operations
 */
export class EmbeddingRateLimiter {
  private limiter: TokenBucketRateLimiter;
  private stats: Map<string, RateLimitStats> = new Map();

  constructor(maxRequestsPerSecond: number = 50) {
    this.limiter = new TokenBucketRateLimiter(1000, maxRequestsPerSecond);
  }

  /**
   * Check rate limit for an embedding operation
   */
  checkLimit(key: string = "default"): RateLimitResult {
    const result = this.limiter.consume(key);

    // Update stats
    const currentStats = this.stats.get(key) || {
      totalRequests: 0,
      allowedRequests: 0,
      blockedRequests: 0,
      currentWindowStart: Date.now(),
      windowRequests: 0,
    };

    currentStats.totalRequests++;
    if (result.allowed) {
      currentStats.allowedRequests++;
    } else {
      currentStats.blockedRequests++;
    }
    currentStats.windowRequests = currentStats.totalRequests;

    this.stats.set(key, currentStats);

    return result;
  }

  /**
   * Get rate limiting statistics
   */
  getStats(key?: string): RateLimitStats | Map<string, RateLimitStats> {
    if (key) {
      return (
        this.stats.get(key) || {
          totalRequests: 0,
          allowedRequests: 0,
          blockedRequests: 0,
          currentWindowStart: Date.now(),
          windowRequests: 0,
        }
      );
    }
    return new Map(this.stats);
  }

  /**
   * Reset rate limiter for a key
   */
  reset(key?: string): void {
    if (key) {
      this.limiter.reset(key);
      this.stats.delete(key);
    } else {
      // Reset all (not implemented in TokenBucketRateLimiter)
      this.stats.clear();
    }
  }

  /**
   * Cleanup old rate limit data
   */
  cleanup(): void {
    this.limiter.cleanup();
  }
}

/**
 * Rate limiting error
 */
export class RateLimitError extends Error {
  public readonly resetTime: number;
  public readonly limit: number;
  public readonly remaining: number;

  constructor(result: RateLimitResult) {
    super(
      `Rate limit exceeded. Try again at ${new Date(
        result.resetTime
      ).toISOString()}`
    );
    this.name = "RateLimitError";
    this.resetTime = result.resetTime;
    this.limit = result.limit;
    this.remaining = result.remaining;
  }
}
