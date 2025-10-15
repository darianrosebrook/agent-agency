/**
 * @fileoverview Performance Monitoring System
 * @author @darianrosebrook
 *
 * Comprehensive performance monitoring for the data layer with alerting,
 * metrics collection, and performance trend analysis.
 * Provides real-time insights into cache performance, query latency, and system health.
 */

import { EventEmitter } from "events";
import { Logger } from "../../utils/Logger";

export interface PerformanceMetrics {
  timestamp: number;
  operation: string;
  entity: string;
  duration: number;
  success: boolean;
  cacheHit?: boolean;
  queryCount?: number;
  errorType?: string;
  metadata?: Record<string, any>;
}

export interface CacheMetrics {
  timestamp: number;
  level: "l1" | "l2" | "overall";
  hitRate: number;
  hits: number;
  misses: number;
  sets: number;
  deletes: number;
  evictions: number;
  size?: number;
  latency?: number;
}

export interface AlertRule {
  id: string;
  name: string;
  condition: (metrics: PerformanceMetrics[]) => boolean;
  severity: "low" | "medium" | "high" | "critical";
  message: string;
  cooldownMs: number; // Prevent alert spam
}

export interface Alert {
  id: string;
  ruleId: string;
  timestamp: number;
  severity: "low" | "medium" | "high" | "critical";
  message: string;
  metrics: Record<string, any>;
}

export class PerformanceMonitor extends EventEmitter {
  private logger: Logger;
  private metrics: PerformanceMetrics[] = [];
  private cacheMetrics: CacheMetrics[] = [];
  private alerts: Alert[] = [];
  private alertRules: AlertRule[] = [];
  private lastAlertTimes: Map<string, number> = new Map();

  private config: {
    maxMetricsHistory: number;
    alertCheckInterval: number;
    enableDetailedLogging: boolean;
  };

  constructor(
    config: Partial<{
      maxMetricsHistory: number;
      alertCheckInterval: number;
      enableDetailedLogging: boolean;
    }> = {},
    logger?: Logger
  ) {
    super();
    this.logger = logger || new Logger("PerformanceMonitor");

    this.config = {
      maxMetricsHistory: 10000,
      alertCheckInterval: 30000, // 30 seconds
      enableDetailedLogging: false,
      ...config,
    };

    this.initializeDefaultAlertRules();
    this.startAlertChecking();
  }

  /**
   * Record a performance metric
   */
  recordMetric(metric: PerformanceMetrics): void {
    this.metrics.push(metric);

    // Keep only recent metrics
    if (this.metrics.length > this.config.maxMetricsHistory) {
      this.metrics = this.metrics.slice(-this.config.maxMetricsHistory);
    }

    // Emit metric event
    this.emit("metric", metric);

    if (this.config.enableDetailedLogging) {
      this.logger.debug("Performance metric recorded", metric);
    }
  }

  /**
   * Record cache performance metrics
   */
  recordCacheMetric(metric: CacheMetrics): void {
    this.cacheMetrics.push(metric);

    // Keep only recent cache metrics (last 1000)
    if (this.cacheMetrics.length > 1000) {
      this.cacheMetrics = this.cacheMetrics.slice(-1000);
    }

    this.emit("cacheMetric", metric);
  }

  /**
   * Add a custom alert rule
   */
  addAlertRule(rule: AlertRule): void {
    this.alertRules.push(rule);
    this.logger.info("Added alert rule", { ruleId: rule.id, name: rule.name });
  }

  /**
   * Remove an alert rule
   */
  removeAlertRule(ruleId: string): boolean {
    const index = this.alertRules.findIndex((rule) => rule.id === ruleId);
    if (index >= 0) {
      this.alertRules.splice(index, 1);
      this.logger.info("Removed alert rule", { ruleId });
      return true;
    }
    return false;
  }

  /**
   * Get current performance statistics
   */
  getPerformanceStats(timeRangeMs: number = 300000): {
    // 5 minutes default
    summary: {
      totalOperations: number;
      successRate: number;
      avgResponseTime: number;
      p95ResponseTime: number;
      p99ResponseTime: number;
      cacheHitRate: number;
      operationsByType: Record<string, number>;
      errorsByType: Record<string, number>;
    };
    recentMetrics: PerformanceMetrics[];
    activeAlerts: Alert[];
  } {
    const cutoff = Date.now() - timeRangeMs;
    const recentMetrics = this.metrics.filter((m) => m.timestamp > cutoff);

    const totalOperations = recentMetrics.length;
    const successfulOperations = recentMetrics.filter((m) => m.success).length;
    const successRate =
      totalOperations > 0
        ? (successfulOperations / totalOperations) * 100
        : 100;

    const responseTimes = recentMetrics
      .map((m) => m.duration)
      .sort((a, b) => a - b);
    const avgResponseTime =
      responseTimes.length > 0
        ? responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length
        : 0;

    const p95Index = Math.floor(responseTimes.length * 0.95);
    const p99Index = Math.floor(responseTimes.length * 0.99);
    const p95ResponseTime = responseTimes[p95Index] || 0;
    const p99ResponseTime = responseTimes[p99Index] || 0;

    const cacheableOperations = recentMetrics.filter(
      (m) => m.cacheHit !== undefined
    );
    const cacheHits = cacheableOperations.filter((m) => m.cacheHit).length;
    const cacheHitRate =
      cacheableOperations.length > 0
        ? (cacheHits / cacheableOperations.length) * 100
        : 0;

    const operationsByType = recentMetrics.reduce((acc, m) => {
      acc[m.operation] = (acc[m.operation] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const errorsByType = recentMetrics
      .filter((m) => !m.success)
      .reduce((acc, m) => {
        const errorType = m.errorType || "unknown";
        acc[errorType] = (acc[errorType] || 0) + 1;
        return acc;
      }, {} as Record<string, number>);

    // Get active alerts (last 5 minutes)
    const activeAlerts = this.alerts.filter(
      (a) => Date.now() - a.timestamp < 300000
    );

    return {
      summary: {
        totalOperations,
        successRate,
        avgResponseTime,
        p95ResponseTime,
        p99ResponseTime,
        cacheHitRate,
        operationsByType,
        errorsByType,
      },
      recentMetrics: recentMetrics.slice(-100), // Last 100 metrics
      activeAlerts,
    };
  }

  /**
   * Get cache performance statistics
   */
  getCacheStats(timeRangeMs: number = 300000): {
    l1Stats: CacheMetrics | null;
    l2Stats: CacheMetrics | null;
    overallStats: CacheMetrics | null;
    trends: {
      hitRateTrend: number; // Change in hit rate over time
      latencyTrend: number; // Change in latency over time
    };
  } {
    const cutoff = Date.now() - timeRangeMs;
    const recentMetrics = this.cacheMetrics.filter((m) => m.timestamp > cutoff);

    const l1Stats = recentMetrics.find((m) => m.level === "l1") || null;
    const l2Stats = recentMetrics.find((m) => m.level === "l2") || null;
    const overallStats =
      recentMetrics.find((m) => m.level === "overall") || null;

    // Calculate trends (simplified - compare first half vs second half)
    const midpoint = cutoff + timeRangeMs / 2;
    const firstHalf = recentMetrics.filter(
      (m) => m.timestamp < midpoint && m.level === "overall"
    );
    const secondHalf = recentMetrics.filter(
      (m) => m.timestamp >= midpoint && m.level === "overall"
    );

    const firstHalfAvgHitRate =
      firstHalf.length > 0
        ? firstHalf.reduce((sum, m) => sum + m.hitRate, 0) / firstHalf.length
        : 0;
    const secondHalfAvgHitRate =
      secondHalf.length > 0
        ? secondHalf.reduce((sum, m) => sum + m.hitRate, 0) / secondHalf.length
        : 0;

    const hitRateTrend = secondHalfAvgHitRate - firstHalfAvgHitRate;

    // For latency trend, we'd need latency data in cache metrics
    const latencyTrend = 0; // Placeholder

    return {
      l1Stats,
      l2Stats,
      overallStats,
      trends: {
        hitRateTrend,
        latencyTrend,
      },
    };
  }

  /**
   * Get all active alerts
   */
  getActiveAlerts(): Alert[] {
    const fiveMinutesAgo = Date.now() - 300000;
    return this.alerts.filter((alert) => alert.timestamp > fiveMinutesAgo);
  }

  /**
   * Clear old metrics and alerts
   */
  cleanup(maxAgeMs: number = 3600000): void {
    // 1 hour default
    const cutoff = Date.now() - maxAgeMs;

    const initialMetricsCount = this.metrics.length;
    this.metrics = this.metrics.filter((m) => m.timestamp > cutoff);

    const initialAlertsCount = this.alerts.length;
    this.alerts = this.alerts.filter((a) => a.timestamp > cutoff);

    const cleanedMetrics = initialMetricsCount - this.metrics.length;
    const cleanedAlerts = initialAlertsCount - this.alerts.length;

    if (cleanedMetrics > 0 || cleanedAlerts > 0) {
      this.logger.info("Cleaned up old monitoring data", {
        cleanedMetrics,
        cleanedAlerts,
        remainingMetrics: this.metrics.length,
        remainingAlerts: this.alerts.length,
      });
    }
  }

  // Private methods

  private initializeDefaultAlertRules(): void {
    // High error rate alert
    this.addAlertRule({
      id: "high_error_rate",
      name: "High Error Rate",
      condition: (metrics) => {
        const recent = metrics.filter((m) => Date.now() - m.timestamp < 300000); // Last 5 minutes
        const errors = recent.filter((m) => !m.success).length;
        return recent.length > 10 && errors / recent.length > 0.1; // >10% error rate
      },
      severity: "high",
      message: "Error rate exceeded 10% in the last 5 minutes",
      cooldownMs: 300000, // 5 minutes
    });

    // Slow response time alert
    this.addAlertRule({
      id: "slow_response_time",
      name: "Slow Response Time",
      condition: (metrics) => {
        const recent = metrics.filter((m) => Date.now() - m.timestamp < 300000);
        if (recent.length < 10) return false;

        const responseTimes = recent
          .map((m) => m.duration)
          .sort((a, b) => b - a);
        const p95 = responseTimes[Math.floor(responseTimes.length * 0.95)];
        return p95 > 5000; // 5 seconds P95 response time
      },
      severity: "medium",
      message: "P95 response time exceeded 5 seconds",
      cooldownMs: 600000, // 10 minutes
    });

    // Low cache hit rate alert
    this.addAlertRule({
      id: "low_cache_hit_rate",
      name: "Low Cache Hit Rate",
      condition: (metrics) => {
        const recent = metrics.filter(
          (m) => Date.now() - m.timestamp < 600000 && m.cacheHit !== undefined
        ); // Last 10 minutes
        if (recent.length < 50) return false;

        const hits = recent.filter((m) => m.cacheHit).length;
        const hitRate = hits / recent.length;
        return hitRate < 0.8; // <80% hit rate
      },
      severity: "low",
      message: "Cache hit rate dropped below 80%",
      cooldownMs: 900000, // 15 minutes
    });

    this.logger.info("Initialized default alert rules", {
      count: this.alertRules.length,
    });
  }

  private startAlertChecking(): void {
    setInterval(() => {
      this.checkAlerts();
    }, this.config.alertCheckInterval);
  }

  private checkAlerts(): void {
    for (const rule of this.alertRules) {
      try {
        // Check cooldown
        const lastAlert = this.lastAlertTimes.get(rule.id) || 0;
        if (Date.now() - lastAlert < rule.cooldownMs) {
          continue;
        }

        // Check condition
        if (rule.condition(this.metrics)) {
          const alert: Alert = {
            id: `alert_${Date.now()}_${Math.random()
              .toString(36)
              .substring(2, 9)}`,
            ruleId: rule.id,
            timestamp: Date.now(),
            severity: rule.severity,
            message: rule.message,
            metrics: this.getPerformanceStats(300000).summary, // Last 5 minutes stats
          };

          this.alerts.push(alert);
          this.lastAlertTimes.set(rule.id, Date.now());

          // Keep only recent alerts
          if (this.alerts.length > 100) {
            this.alerts = this.alerts.slice(-100);
          }

          // Emit alert
          this.emit("alert", alert);

          this.logger.warn("Performance alert triggered", {
            alertId: alert.id,
            ruleId: rule.id,
            severity: rule.severity,
            message: rule.message,
          });
        }
      } catch (error) {
        this.logger.error("Error checking alert rule", {
          ruleId: rule.id,
          error,
        });
      }
    }
  }
}
