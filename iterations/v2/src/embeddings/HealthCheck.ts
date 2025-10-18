/**
 * @fileoverview Health Check Endpoints for Embedding Services
 *
 * Provides comprehensive health monitoring for embedding infrastructure
 * including circuit breaker status, rate limiting, and service availability.
 *
 * @author @darianrosebrook
 */

import { CircuitBreaker } from "./CircuitBreaker.js";
import { EmbeddingService } from "./EmbeddingService.js";
import { EmbeddingRateLimiter } from "./RateLimiter.js";

export interface HealthStatus {
  status: "healthy" | "degraded" | "unhealthy";
  timestamp: string;
  version: string;
  uptime: number;
  checks: {
    ollama: ServiceHealth;
    circuitBreaker: CircuitBreakerHealth;
    rateLimiter: RateLimiterHealth;
    cache: CacheHealth;
    memory: MemoryHealth;
  };
}

export interface ServiceHealth {
  status: "up" | "down" | "degraded";
  responseTime?: number;
  error?: string;
  lastChecked: string;
}

export interface CircuitBreakerHealth {
  status: "closed" | "open" | "half-open";
  failures: number;
  successes: number;
  consecutiveFailures: number;
  stateChanges: number;
  lastFailureTime?: string;
  lastSuccessTime?: string;
}

export interface RateLimiterHealth {
  status: "healthy" | "warning" | "critical";
  totalRequests: number;
  allowedRequests: number;
  blockedRequests: number;
  blockRate: number; // percentage
  activeKeys: number;
}

export interface CacheHealth {
  status: "healthy" | "warning";
  size: number;
  maxSize: number;
  utilization: number; // percentage
  hitRate?: number;
  missRate?: number;
}

export interface MemoryHealth {
  status: "healthy" | "warning" | "critical";
  rss: number;
  heapTotal: number;
  heapUsed: number;
  external: number;
  utilization: number; // percentage of heap used
}

/**
 * Health check service for embedding infrastructure
 */
export class EmbeddingHealthCheck {
  private startTime: number;
  private version: string;
  private ollamaChecks: ServiceHealth[] = [];
  private maxChecks = 10; // Keep last 10 checks

  constructor(version: string = "1.0.0") {
    this.startTime = Date.now();
    this.version = version;
  }

  /**
   * Perform comprehensive health check
   */
  async checkHealth(
    embeddingService: EmbeddingService,
    options: {
      ollamaEndpoint?: string;
      circuitBreaker?: CircuitBreaker;
      rateLimiter?: EmbeddingRateLimiter;
      includeDeepChecks?: boolean;
    } = {}
  ): Promise<HealthStatus> {
    const checks = await this.performAllChecks(embeddingService, options);

    // Determine overall status
    const overallStatus = this.determineOverallStatus(checks);

    return {
      status: overallStatus,
      timestamp: new Date().toISOString(),
      version: this.version,
      uptime: Date.now() - this.startTime,
      checks,
    };
  }

  /**
   * Perform all health checks
   */
  private async performAllChecks(
    embeddingService: EmbeddingService,
    options: Parameters<typeof this.checkHealth>[1]
  ): Promise<HealthStatus["checks"]> {
    const [
      ollamaHealth,
      circuitBreakerHealth,
      rateLimiterHealth,
      cacheHealth,
      memoryHealth,
    ] = await Promise.allSettled([
      this.checkOllamaHealth(
        options?.ollamaEndpoint || "http://localhost:11434"
      ),
      this.checkCircuitBreakerHealth(options?.circuitBreaker),
      this.checkRateLimiterHealth(options?.rateLimiter),
      this.checkCacheHealth(embeddingService),
      this.checkMemoryHealth(),
    ]);

    return {
      ollama: this.extractResult(ollamaHealth, {
        status: "down",
        lastChecked: new Date().toISOString(),
      }),
      circuitBreaker: this.extractResult(circuitBreakerHealth, {
        status: "closed",
        failures: 0,
        successes: 0,
        consecutiveFailures: 0,
        stateChanges: 0,
      }),
      rateLimiter: this.extractResult(rateLimiterHealth, {
        status: "healthy",
        totalRequests: 0,
        allowedRequests: 0,
        blockedRequests: 0,
        blockRate: 0,
        activeKeys: 0,
      }),
      cache: this.extractResult(cacheHealth, {
        status: "healthy",
        size: 0,
        maxSize: 1000,
        utilization: 0,
      }),
      memory: this.extractResult(memoryHealth, {
        status: "healthy",
        rss: 0,
        heapTotal: 0,
        heapUsed: 0,
        external: 0,
        utilization: 0,
      }),
    };
  }

  /**
   * Check Ollama service health
   */
  private async checkOllamaHealth(endpoint: string): Promise<ServiceHealth> {
    const startTime = Date.now();

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000); // 5 second timeout

      const response = await fetch(`${endpoint}/api/tags`, {
        method: "GET",
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      const responseTime = Date.now() - startTime;

      const health: ServiceHealth = {
        status: response.ok ? "up" : "degraded",
        responseTime,
        lastChecked: new Date().toISOString(),
      };

      if (!response.ok) {
        health.error = `HTTP ${response.status}: ${response.statusText}`;
      }

      // Store check result
      this.ollamaChecks.push(health);
      if (this.ollamaChecks.length > this.maxChecks) {
        this.ollamaChecks.shift();
      }

      return health;
    } catch (error) {
      const responseTime = Date.now() - startTime;

      return {
        status: "down",
        responseTime,
        error: error instanceof Error ? error.message : "Unknown error",
        lastChecked: new Date().toISOString(),
      };
    }
  }

  /**
   * Check circuit breaker health
   */
  private checkCircuitBreakerHealth(
    circuitBreaker?: CircuitBreaker
  ): CircuitBreakerHealth {
    if (!circuitBreaker) {
      return {
        status: "closed",
        failures: 0,
        successes: 0,
        consecutiveFailures: 0,
        stateChanges: 0,
      };
    }

    const stats = circuitBreaker.getStats();

    return {
      status: circuitBreaker.getState() as CircuitBreakerHealth["status"],
      failures: stats.totalFailures,
      successes: stats.totalSuccesses,
      consecutiveFailures: stats.consecutiveFailures,
      stateChanges: 0, // Not tracked in current implementation
      lastFailureTime: stats.lastFailureTime
        ? new Date(stats.lastFailureTime).toISOString()
        : undefined,
      lastSuccessTime: stats.lastSuccessTime
        ? new Date(stats.lastSuccessTime).toISOString()
        : undefined,
    };
  }

  /**
   * Check rate limiter health
   */
  private checkRateLimiterHealth(
    rateLimiter?: EmbeddingRateLimiter
  ): RateLimiterHealth {
    if (!rateLimiter) {
      return {
        status: "healthy",
        totalRequests: 0,
        allowedRequests: 0,
        blockedRequests: 0,
        blockRate: 0,
        activeKeys: 0,
      };
    }

    const stats = rateLimiter.getStats();
    let totalRequests = 0;
    let allowedRequests = 0;
    let blockedRequests = 0;
    let activeKeys = 0;

    if (stats instanceof Map) {
      for (const keyStats of stats.values()) {
        totalRequests += keyStats.totalRequests;
        allowedRequests += keyStats.allowedRequests;
        blockedRequests += keyStats.blockedRequests;
        activeKeys++;
      }
    } else {
      totalRequests = stats.totalRequests;
      allowedRequests = stats.allowedRequests;
      blockedRequests = stats.blockedRequests;
      activeKeys = 1;
    }

    const blockRate =
      totalRequests > 0 ? (blockedRequests / totalRequests) * 100 : 0;

    let status: RateLimiterHealth["status"] = "healthy";
    if (blockRate > 50) {
      status = "critical";
    } else if (blockRate > 20) {
      status = "warning";
    }

    return {
      status,
      totalRequests,
      allowedRequests,
      blockedRequests,
      blockRate,
      activeKeys,
    };
  }

  /**
   * Check cache health
   */
  private checkCacheHealth(embeddingService: EmbeddingService): CacheHealth {
    // Access private cache size (this is a limitation of the current design)
    // In a real implementation, we'd expose cache stats through the interface
    const size = (embeddingService as any).cache?.size || 0;
    const maxSize = (embeddingService as any).maxCacheSize || 1000;
    const utilization = maxSize > 0 ? (size / maxSize) * 100 : 0;

    let status: CacheHealth["status"] = "healthy";
    if (utilization > 90) {
      status = "warning";
    }

    return {
      status,
      size,
      maxSize,
      utilization,
    };
  }

  /**
   * Check memory health
   */
  private checkMemoryHealth(): MemoryHealth {
    const usage = process.memoryUsage();
    const heapUtilization =
      usage.heapTotal > 0 ? (usage.heapUsed / usage.heapTotal) * 100 : 0;

    let status: MemoryHealth["status"] = "healthy";
    if (heapUtilization > 85) {
      status = "critical";
    } else if (heapUtilization > 70) {
      status = "warning";
    }

    return {
      status,
      rss: usage.rss,
      heapTotal: usage.heapTotal,
      heapUsed: usage.heapUsed,
      external: usage.external,
      utilization: heapUtilization,
    };
  }

  /**
   * Extract result from PromiseSettledResult
   */
  private extractResult<T>(result: PromiseSettledResult<T>, fallback: T): T {
    if (result.status === "fulfilled") {
      return result.value;
    }
    console.error("Health check failed:", result.reason);
    return fallback;
  }

  /**
   * Determine overall health status
   */
  private determineOverallStatus(
    checks: HealthStatus["checks"]
  ): HealthStatus["status"] {
    // Critical failures
    if (
      checks.ollama.status === "down" ||
      checks.circuitBreaker.status === "open"
    ) {
      return "unhealthy";
    }

    // Warning conditions
    if (
      checks.rateLimiter.status === "critical" ||
      checks.memory.status === "critical" ||
      checks.cache.status === "warning"
    ) {
      return "degraded";
    }

    // All good
    return "healthy";
  }

  /**
   * Get recent Ollama health history
   */
  getOllamaHealthHistory(): ServiceHealth[] {
    return [...this.ollamaChecks];
  }

  /**
   * Reset health check state
   */
  reset(): void {
    this.ollamaChecks = [];
    this.startTime = Date.now();
  }
}

/**
 * HTTP handlers for health check endpoints
 */
export class HealthCheckHandlers {
  private healthCheck: EmbeddingHealthCheck;

  constructor(healthCheck: EmbeddingHealthCheck) {
    this.healthCheck = healthCheck;
  }

  /**
   * Basic health check for load balancers
   */
  async basicHealth(
    req: any,
    res: any,
    embeddingService: EmbeddingService
  ): Promise<void> {
    try {
      const health = await this.healthCheck.checkHealth(embeddingService, {
        includeDeepChecks: false,
      });

      const statusCode =
        health.status === "healthy"
          ? 200
          : health.status === "degraded"
          ? 200
          : 503; // unhealthy

      res.status(statusCode).json({
        status: health.status,
        timestamp: health.timestamp,
      });
    } catch (error) {
      res.status(503).json({
        status: "unhealthy",
        timestamp: new Date().toISOString(),
        error: "Health check failed",
      });
    }
  }

  /**
   * Detailed health check for monitoring systems
   */
  async detailedHealth(
    req: any,
    res: any,
    embeddingService: EmbeddingService
  ): Promise<void> {
    try {
      const health = await this.healthCheck.checkHealth(embeddingService, {
        includeDeepChecks: true,
      });

      const statusCode =
        health.status === "healthy"
          ? 200
          : health.status === "degraded"
          ? 200
          : 503;

      res.status(statusCode).json(health);
    } catch (error) {
      res.status(503).json({
        status: "unhealthy",
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : "Health check failed",
      });
    }
  }

  /**
   * Readiness check for Kubernetes
   */
  async readiness(
    req: any,
    res: any,
    embeddingService: EmbeddingService
  ): Promise<void> {
    try {
      // Quick check - just verify service is initialized
      const isReady =
        embeddingService &&
        typeof embeddingService.generateEmbedding === "function";

      res.status(isReady ? 200 : 503).json({
        status: isReady ? "ready" : "not ready",
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      res.status(503).json({
        status: "not ready",
        timestamp: new Date().toISOString(),
        error: "Readiness check failed",
      });
    }
  }
}
