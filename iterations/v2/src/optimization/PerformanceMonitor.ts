/**
 * @fileoverview Performance Monitor for Runtime Optimization Engine
 *
 * Collects and stores performance metrics with minimal overhead.
 * Implements circular buffer for efficient memory management.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import {
  MetricType,
  type IPerformanceMonitor,
  type PerformanceMetric,
} from "@/types/optimization-types";

/**
 * Configuration for Performance Monitor
 */
export interface PerformanceMonitorConfig {
  /** Maximum metrics to store (circular buffer size) */
  maxMetrics: number;

  /** Automatically clean metrics older than (ms) */
  autoCleanOlderThanMs: number;

  /** Enable automatic cleanup */
  enableAutoCleanup: boolean;

  /** Cleanup interval (ms) */
  cleanupIntervalMs: number;
}

/**
 * Default configuration
 */
const DEFAULT_CONFIG: PerformanceMonitorConfig = {
  maxMetrics: 10000,
  autoCleanOlderThanMs: 3600000, // 1 hour
  enableAutoCleanup: true,
  cleanupIntervalMs: 300000, // 5 minutes
};

/**
 * Performance Monitor
 *
 * Implements efficient metric collection with:
 * - Circular buffer for fixed memory usage
 * - Automatic cleanup of old metrics
 * - Fast queries by time range
 * - Minimal locking for concurrent access
 */
export class PerformanceMonitor implements IPerformanceMonitor {
  private logger: Logger;
  private config: PerformanceMonitorConfig;
  private metrics: PerformanceMetric[] = [];
  private cleanupTimer?: ReturnType<typeof setInterval>;
  private metricsLock = false;

  constructor(config: Partial<PerformanceMonitorConfig> = {}) {
    this.logger = new Logger("PerformanceMonitor");
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Start the performance monitor
   */
  async start(): Promise<void> {
    if (this.config.enableAutoCleanup) {
      this.startAutoCleanup();
    }
    this.logger.info("Performance monitor started", {
      maxMetrics: this.config.maxMetrics,
      autoCleanup: this.config.enableAutoCleanup,
    });
  }

  /**
   * Stop the performance monitor
   */
  async stop(): Promise<void> {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = undefined;
    }
    this.logger.info("Performance monitor stopped");
  }

  /**
   * Record a performance metric
   *
   * @param metric Performance metric to record
   */
  async recordMetric(metric: PerformanceMetric): Promise<void> {
    await this.withLock(async () => {
      // Add metric to buffer
      this.metrics.push(metric);

      // If buffer is full, remove oldest metric (circular buffer)
      if (this.metrics.length > this.config.maxMetrics) {
        this.metrics.shift();
      }
    });
  }

  /**
   * Get metrics for a time window
   *
   * @param startTime Window start time
   * @param endTime Window end time
   * @param metricType Optional metric type filter
   * @returns Metrics within time window
   */
  async getMetrics(
    startTime: Date,
    endTime: Date,
    metricType?: MetricType
  ): Promise<PerformanceMetric[]> {
    return this.withLock(async () => {
      return this.metrics.filter((metric) => {
        const inTimeRange =
          metric.timestamp >= startTime && metric.timestamp <= endTime;

        if (!metricType) {
          return inTimeRange;
        }

        return inTimeRange && metric.type === metricType;
      });
    });
  }

  /**
   * Get latest metrics
   *
   * @param count Number of metrics to retrieve
   * @param metricType Optional metric type filter
   * @returns Latest metrics
   */
  async getLatestMetrics(
    count: number,
    metricType?: MetricType
  ): Promise<PerformanceMetric[]> {
    return this.withLock(async () => {
      let filtered = this.metrics;

      if (metricType) {
        filtered = this.metrics.filter((m) => m.type === metricType);
      }

      // Return last N metrics
      return filtered.slice(-count);
    });
  }

  /**
   * Clear metrics older than specified date
   *
   * @param olderThan Clear metrics older than this date
   */
  async clearMetrics(olderThan: Date): Promise<void> {
    await this.withLock(async () => {
      const beforeCount = this.metrics.length;
      this.metrics = this.metrics.filter((m) => m.timestamp >= olderThan);
      const clearedCount = beforeCount - this.metrics.length;

      if (clearedCount > 0) {
        this.logger.debug("Cleared old metrics", {
          clearedCount,
          remainingCount: this.metrics.length,
        });
      }
    });
  }

  /**
   * Get current metric count
   */
  getMetricCount(): number {
    return this.metrics.length;
  }

  /**
   * Get configuration
   */
  getConfig(): PerformanceMonitorConfig {
    return { ...this.config };
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<PerformanceMonitorConfig>): void {
    this.config = { ...this.config, ...config };

    // Restart auto-cleanup if needed
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = undefined;
    }

    if (this.config.enableAutoCleanup) {
      this.startAutoCleanup();
    }

    this.logger.info("Configuration updated", this.config);
  }

  /**
   * Start automatic cleanup
   */
  private startAutoCleanup(): void {
    this.cleanupTimer = setInterval(async () => {
      const cutoffTime = new Date(
        Date.now() - this.config.autoCleanOlderThanMs
      );
      await this.clearMetrics(cutoffTime);
    }, this.config.cleanupIntervalMs);
  }

  /**
   * Simple lock mechanism for concurrent access
   *
   * @param fn Function to execute with lock
   */
  private async withLock<T>(fn: () => T | Promise<T>): Promise<T> {
    // Wait for lock to be available
    while (this.metricsLock) {
      await new Promise((resolve) => setTimeout(resolve, 1));
    }

    this.metricsLock = true;

    try {
      return await fn();
    } finally {
      this.metricsLock = false;
    }
  }
}
