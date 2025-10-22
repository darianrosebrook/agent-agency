/**
 * Enhanced Performance Tracker for Detailed Metrics Collection
 *
 * Provides comprehensive performance monitoring and analysis capabilities
 * for the V2 Arbiter system, including task performance, system metrics,
 * and performance trend analysis.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { SystemMetrics } from "./HealthMonitor";

/// <reference types="node" />

export interface PerformanceSnapshot {
  timestamp: number;
  metrics: SystemMetrics;
  taskMetrics?: {
    activeTasks: number;
    completedTasks: number;
    failedTasks: number;
    averageTaskDuration: number;
    taskThroughput: number;
  };
  networkMetrics?: {
    requestsPerSecond: number;
    averageResponseTime: number;
    errorRate: number;
  };
}

export interface PerformanceTrend {
  metric: string;
  values: number[];
  timestamps: number[];
  average: number;
  trend: "increasing" | "decreasing" | "stable";
  standardDeviation: number;
}

export interface PerformanceReport {
  period: {
    start: number;
    end: number;
    duration: number;
  };
  summary: {
    averageCpuUsage: number;
    peakCpuUsage: number;
    averageMemoryUsage: number;
    peakMemoryUsage: number;
    totalTasksProcessed: number;
    taskSuccessRate: number;
    averageTaskDuration: number;
  };
  trends: PerformanceTrend[];
  alerts: string[];
  recommendations: string[];
}

export interface PerformanceTrackerConfig {
  collectionIntervalMs: number;
  retentionPeriodMs: number;
  maxSnapshots: number;
  enableDetailedMetrics: boolean;
  enableTrendAnalysis: boolean;
}

export class PerformanceTracker extends EventEmitter {
  private config: PerformanceTrackerConfig;
  private snapshots: PerformanceSnapshot[] = [];
  private isRunning = false;
  private collectionTimer?: NodeJS.Timeout;
  private lastCollectionTime = Date.now();

  constructor(config: PerformanceTrackerConfig) {
    super();
    this.config = config;
  }

  public start(): void {
    if (this.isRunning) return;

    this.isRunning = true;
    this.lastCollectionTime = Date.now();

    // Start periodic collection
    this.collectionTimer = setInterval(() => {
      this.collectSnapshot();
    }, this.config.collectionIntervalMs);

    // Collect initial snapshot
    this.collectSnapshot();

    this.emit("started");
  }

  public stop(): void {
    if (!this.isRunning) return;

    this.isRunning = false;

    if (this.collectionTimer) {
      clearInterval(this.collectionTimer);
      this.collectionTimer = undefined;
    }

    this.emit("stopped");
  }

  public getLatestSnapshot(): PerformanceSnapshot | null {
    return this.snapshots.length > 0
      ? this.snapshots[this.snapshots.length - 1]
      : null;
  }

  public getSnapshots(since?: number): PerformanceSnapshot[] {
    if (!since) return [...this.snapshots];

    return this.snapshots.filter((snapshot) => snapshot.timestamp >= since);
  }

  public getTrends(metric: string, windowMs?: number): PerformanceTrend | null {
    if (!this.config.enableTrendAnalysis) return null;

    const now = Date.now();
    const window = windowMs || this.config.retentionPeriodMs;
    const cutoff = now - window;

    const relevantSnapshots = this.snapshots.filter(
      (s) => s.timestamp >= cutoff
    );
    if (relevantSnapshots.length < 2) return null;

    const values: number[] = [];
    const timestamps: number[] = [];

    for (const snapshot of relevantSnapshots) {
      let _value: number | undefined;

      // Extract the metric value based on path
      const path = metric.split(".");
      let current: any = snapshot.metrics;

      for (const key of path) {
        if (current && typeof current === "object" && key in current) {
          current = current[key];
        } else {
          current = undefined;
          break;
        }
      }

      if (typeof current === "number") {
        values.push(current);
        timestamps.push(snapshot.timestamp);
      }
    }

    if (values.length < 2) return null;

    // Calculate statistics
    const average = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance =
      values.reduce((sum, val) => sum + Math.pow(val - average, 2), 0) /
      values.length;
    const standardDeviation = Math.sqrt(variance);

    // Calculate trend (simple linear regression slope)
    let trend: "increasing" | "decreasing" | "stable" = "stable";
    if (values.length >= 3) {
      const n = values.length;
      const sumX = timestamps.reduce((sum, t, i) => sum + i, 0);
      const sumY = values.reduce((sum, val) => sum + val, 0);
      const sumXY = timestamps.reduce((sum, t, i) => sum + i * values[i], 0);
      const sumXX = timestamps.reduce((sum, t, i) => sum + i * i, 0);

      const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);

      if (slope > 0.1) trend = "increasing";
      else if (slope < -0.1) trend = "decreasing";
    }

    return {
      metric,
      values,
      timestamps,
      average,
      trend,
      standardDeviation,
    };
  }

  public generateReport(periodMs?: number): PerformanceReport {
    const now = Date.now();
    const period = periodMs || this.config.retentionPeriodMs;
    const startTime = now - period;

    const relevantSnapshots = this.snapshots.filter(
      (s) => s.timestamp >= startTime
    );
    if (relevantSnapshots.length === 0) {
      throw new Error("No performance data available for the specified period");
    }

    // Calculate summary statistics
    const cpuUsages = relevantSnapshots
      .map((s) => s.metrics.cpuUsage)
      .filter((v) => typeof v === "number");
    const memoryUsages = relevantSnapshots.map((s) => {
      if (s.metrics.memoryUsage && s.metrics.memoryUsage.heapUsed) {
        return s.metrics.memoryUsage.heapUsed / s.metrics.memoryUsage.heapTotal;
      }
      return 0;
    });

    const averageCpuUsage =
      cpuUsages.length > 0
        ? cpuUsages.reduce((sum, val) => sum + val, 0) / cpuUsages.length
        : 0;
    const peakCpuUsage = cpuUsages.length > 0 ? Math.max(...cpuUsages) : 0;
    const averageMemoryUsage =
      memoryUsages.length > 0
        ? memoryUsages.reduce((sum, val) => sum + val, 0) / memoryUsages.length
        : 0;
    const peakMemoryUsage =
      memoryUsages.length > 0 ? Math.max(...memoryUsages) : 0;

    // Task metrics (would need integration with task system)
    const totalTasksProcessed = 0; // Placeholder
    const taskSuccessRate = 0; // Placeholder
    const averageTaskDuration = 0; // Placeholder

    // Generate trends for key metrics
    const trends: PerformanceTrend[] = [];
    const keyMetrics = [
      "cpuUsage",
      "memoryUsage.heapUsed",
      "activeConnections",
    ];

    for (const metric of keyMetrics) {
      const trend = this.getTrends(metric, period);
      if (trend) {
        trends.push(trend);
      }
    }

    // Generate alerts based on trends
    const alerts: string[] = [];
    for (const trend of trends) {
      if (trend.trend === "increasing" && trend.average > 0.8) {
        alerts.push(
          `High ${trend.metric} detected (${Math.round(
            trend.average * 100
          )}% average)`
        );
      }
    }

    // Generate recommendations
    const recommendations: string[] = [];
    if (averageCpuUsage > 0.7) {
      recommendations.push(
        "Consider optimizing CPU-intensive operations or scaling horizontally"
      );
    }
    if (averageMemoryUsage > 0.8) {
      recommendations.push(
        "Memory usage is high - consider memory optimization or increasing system memory"
      );
    }
    if (peakMemoryUsage - averageMemoryUsage > 0.2) {
      recommendations.push(
        "High memory usage variance detected - investigate memory leaks"
      );
    }

    return {
      period: {
        start: startTime,
        end: now,
        duration: period,
      },
      summary: {
        averageCpuUsage,
        peakCpuUsage,
        averageMemoryUsage,
        peakMemoryUsage,
        totalTasksProcessed,
        taskSuccessRate,
        averageTaskDuration,
      },
      trends,
      alerts,
      recommendations,
    };
  }

  private collectSnapshot(): void {
    const timestamp = Date.now();

    // This would be integrated with the HealthMonitor to get current metrics
    // For now, we'll collect basic system metrics
    const metrics: SystemMetrics = {
      uptime: process.uptime() * 1000,
      memoryUsage: process.memoryUsage(),
      cpuUsage: process.cpuUsage().user / 1000000,
      activeConnections: 0, // Would need database integration
      taskQueueDepth: 0, // Would need task queue integration
      circuitBreakerStats: {}, // Would need circuit breaker integration
      errorRate: 0, // Would need error tracking integration
      throughput: 0, // Would need task completion tracking
    };

    const snapshot: PerformanceSnapshot = {
      timestamp,
      metrics,
      // taskMetrics and networkMetrics would be populated with actual data
    };

    this.snapshots.push(snapshot);

    // Maintain retention policy
    const cutoff = timestamp - this.config.retentionPeriodMs;
    this.snapshots = this.snapshots.filter((s) => s.timestamp >= cutoff);

    // Limit number of snapshots
    if (this.snapshots.length > this.config.maxSnapshots) {
      this.snapshots = this.snapshots.slice(-this.config.maxSnapshots);
    }

    this.emit("snapshot-collected", snapshot);
  }

  public getStats(): {
    totalSnapshots: number;
    oldestSnapshot: number | null;
    newestSnapshot: number | null;
    averageCollectionInterval: number;
  } {
    const totalSnapshots = this.snapshots.length;
    const oldestSnapshot =
      totalSnapshots > 0 ? this.snapshots[0].timestamp : null;
    const newestSnapshot =
      totalSnapshots > 0 ? this.snapshots[totalSnapshots - 1].timestamp : null;

    let averageCollectionInterval = 0;
    if (totalSnapshots > 1) {
      const intervals = [];
      for (let i = 1; i < totalSnapshots; i++) {
        intervals.push(
          this.snapshots[i].timestamp - this.snapshots[i - 1].timestamp
        );
      }
      averageCollectionInterval =
        intervals.reduce((sum, val) => sum + val, 0) / intervals.length;
    }

    return {
      totalSnapshots,
      oldestSnapshot,
      newestSnapshot,
      averageCollectionInterval,
    };
  }
}
