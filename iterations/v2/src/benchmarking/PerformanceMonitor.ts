/**
 * Performance Monitor for ARBITER-004
 *
 * Production monitoring and observability for the performance tracking system.
 * Tracks system health, performance impact, and provides real-time metrics.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { performance, PerformanceObserver } from "perf_hooks";
import { PERFORMANCE_CONFIG } from "../config/performance-config";
import { DataCollector } from "./DataCollector";
import { MetricAggregator } from "./MetricAggregator";
import { PerformanceAnalyzer } from "./PerformanceAnalyzer";
import { RLDataPipeline } from "./RLDataPipeline";

export interface PerformanceSnapshot {
  timestamp: string;
  collection: {
    eventsCollected: number;
    bufferSize: number;
    averageLatencyMs: number;
    throughputPerSec: number;
  };
  aggregation: {
    aggregationsCompleted: number;
    activeWindows: number;
    memoryUsageMB: number;
    processingTimeMs: number;
  };
  rl: {
    samplesGenerated: number;
    batchesCreated: number;
    pendingBatches: number;
    qualityScore: number;
  };
  analysis: {
    agentsTracked: number;
    anomaliesDetected: number;
    alertsTriggered: number;
    analysisTimeMs: number;
  };
  system: {
    heapUsedMB: number;
    heapTotalMB: number;
    externalMB: number;
    rssMB: number;
    cpuUsagePercent: number;
    uptimeSeconds: number;
  };
}

export interface HealthCheckResult {
  status: "healthy" | "degraded" | "unhealthy";
  timestamp: string;
  checks: {
    collection: { status: "ok" | "warning" | "error"; message?: string };
    aggregation: { status: "ok" | "warning" | "error"; message?: string };
    rl: { status: "ok" | "warning" | "error"; message?: string };
    analysis: { status: "ok" | "warning" | "error"; message?: string };
    system: { status: "ok" | "warning" | "error"; message?: string };
  };
  recommendations: string[];
}

export class PerformanceMonitor extends EventEmitter {
  private isMonitoring: boolean = false;
  private monitoringInterval?: NodeJS.Timeout;
  private healthCheckInterval?: NodeJS.Timeout;
  private performanceObserver?: PerformanceObserver;
  private lastSnapshot?: PerformanceSnapshot;
  private healthHistory: HealthCheckResult[] = [];

  // Component references
  private dataCollector?: DataCollector;
  private metricAggregator?: MetricAggregator;
  private rlDataPipeline?: RLDataPipeline;
  private performanceAnalyzer?: PerformanceAnalyzer;

  constructor(
    private components?: {
      dataCollector?: DataCollector;
      metricAggregator?: MetricAggregator;
      rlDataPipeline?: RLDataPipeline;
      performanceAnalyzer?: PerformanceAnalyzer;
    }
  ) {
    super();

    if (components) {
      this.dataCollector = components.dataCollector;
      this.metricAggregator = components.metricAggregator;
      this.rlDataPipeline = components.rlDataPipeline;
      this.performanceAnalyzer = components.performanceAnalyzer;
    }

    this.setupPerformanceObserver();
  }

  /**
   * Start monitoring with configured intervals
   */
  startMonitoring(): void {
    if (this.isMonitoring) return;

    this.isMonitoring = true;

    // Performance snapshot interval
    this.monitoringInterval = setInterval(() => {
      this.takeSnapshot().catch((error) => {
        this.emit("monitoring_error", error);
      });
    }, PERFORMANCE_CONFIG.analysis.monitoring.performanceSnapshotIntervalMs);

    // Health check interval
    this.healthCheckInterval = setInterval(() => {
      this.performHealthCheck().catch((error) => {
        this.emit("health_check_error", error);
      });
    }, PERFORMANCE_CONFIG.analysis.monitoring.healthCheckIntervalMs);

    this.emit("monitoring_started");
  }

  /**
   * Stop monitoring
   */
  stopMonitoring(): void {
    if (!this.isMonitoring) return;

    this.isMonitoring = false;

    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = undefined;
    }

    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
      this.healthCheckInterval = undefined;
    }

    if (this.performanceObserver) {
      this.performanceObserver.disconnect();
      this.performanceObserver = undefined;
    }

    this.emit("monitoring_stopped");
  }

  /**
   * Take a performance snapshot
   */
  async takeSnapshot(): Promise<PerformanceSnapshot> {
    const timestamp = new Date().toISOString();
    const memUsage = process.memoryUsage();
    const cpuUsage = process.cpuUsage();
    const uptime = process.uptime();

    // Collection metrics
    const collectionStats = this.dataCollector?.getStats() || {
      eventsCollected: 0,
      bufferSize: 0,
      averageCollectionTimeMs: 0,
    };

    // Aggregation metrics
    const aggregationStats = this.metricAggregator?.getStats() || {
      totalAggregations: 0,
      eventBufferSize: 0,
      memoryUsageMB: 0,
    };

    // RL metrics
    const rlStats = this.rlDataPipeline?.getPipelineStats() || {
      totalSamples: 0,
      completedBatches: [],
      pendingBatches: [],
    };

    // Analysis metrics
    const analysisStats = this.performanceAnalyzer?.getAnalysisStats() || {
      agentsTracked: 0,
      totalAnomalies: 0,
    };

    const snapshot: PerformanceSnapshot = {
      timestamp,
      collection: {
        eventsCollected: collectionStats.eventsCollected || 0,
        bufferSize: collectionStats.bufferSize || 0,
        averageLatencyMs: collectionStats.averageCollectionTimeMs || 0,
        throughputPerSec: this.calculateThroughput(collectionStats),
      },
      aggregation: {
        aggregationsCompleted: aggregationStats.totalAggregations || 0,
        activeWindows: 4, // Default window count
        memoryUsageMB: aggregationStats.memoryUsageMB || 0,
        processingTimeMs: aggregationStats.processingTimeMs || 0,
      },
      rl: {
        samplesGenerated: rlStats.totalSamples || 0,
        batchesCreated: rlStats.completedBatches?.length || 0,
        pendingBatches: rlStats.pendingBatches?.length || 0,
        qualityScore: this.calculateAverageQualityScore(
          rlStats.completedBatches || []
        ),
      },
      analysis: {
        agentsTracked: analysisStats.agentsTracked || 0,
        anomaliesDetected: analysisStats.totalAnomalies || 0,
        alertsTriggered: analysisStats.alertsTriggered || 0,
        analysisTimeMs: analysisStats.analysisTimeMs || 0,
      },
      system: {
        heapUsedMB: memUsage.heapUsed / 1024 / 1024,
        heapTotalMB: memUsage.heapTotal / 1024 / 1024,
        externalMB: memUsage.external / 1024 / 1024,
        rssMB: memUsage.rss / 1024 / 1024,
        cpuUsagePercent: this.calculateCpuUsagePercent(cpuUsage),
        uptimeSeconds: uptime,
      },
    };

    this.lastSnapshot = snapshot;
    this.emit("snapshot_taken", snapshot);

    return snapshot;
  }

  /**
   * Perform health check
   */
  async performHealthCheck(): Promise<HealthCheckResult> {
    const timestamp = new Date().toISOString();
    const checks = {
      collection: { status: "ok" as const },
      aggregation: { status: "ok" as const },
      rl: { status: "ok" as const },
      analysis: { status: "ok" as const },
      system: { status: "ok" as const },
    };
    const recommendations: string[] = [];

    // Collection health check
    if (this.dataCollector) {
      const stats = this.dataCollector.getStats();
      if (stats.bufferSize > PERFORMANCE_CONFIG.collection.batchSize * 2) {
        checks.collection = { status: "warning", message: "Buffer size high" };
        recommendations.push("Consider increasing batch processing frequency");
      }
      if (
        stats.averageCollectionTimeMs >
        PERFORMANCE_CONFIG.collection.maxCollectionLatencyMs
      ) {
        checks.collection = {
          status: "error",
          message: "Collection latency exceeded threshold",
        };
        recommendations.push("Investigate collection performance bottleneck");
      }
    }

    // Aggregation health check
    if (this.metricAggregator) {
      const stats = this.metricAggregator.getStats();
      if (stats.memoryUsageMB > 100) {
        checks.aggregation = {
          status: "warning",
          message: "High memory usage",
        };
        recommendations.push("Consider memory optimization or cleanup");
      }
    }

    // RL health check
    if (this.rlDataPipeline) {
      const stats = this.rlDataPipeline.getPipelineStats();
      if (
        stats.pendingBatches &&
        stats.pendingBatches.length >
          PERFORMANCE_CONFIG.rl.batching.maxBatchSize
      ) {
        checks.rl = { status: "warning", message: "High pending batch count" };
        recommendations.push("Check RL pipeline processing capacity");
      }
    }

    // Analysis health check
    if (this.performanceAnalyzer) {
      const stats = this.performanceAnalyzer.getAnalysisStats();
      if (stats.totalAnomalies > 10) {
        checks.analysis = { status: "warning", message: "High anomaly count" };
        recommendations.push("Review system performance issues");
      }
    }

    // System health check
    const memUsage = process.memoryUsage();
    const heapMB = memUsage.heapUsed / 1024 / 1024;

    if (heapMB > 500) {
      checks.system = { status: "warning", message: "High heap usage" };
      recommendations.push("Monitor memory usage and consider optimization");
    }

    if (heapMB > 800) {
      checks.system = { status: "error", message: "Critical heap usage" };
      recommendations.push("Immediate memory optimization required");
    }

    // Determine overall status
    const errorChecks = Object.values(checks).filter(
      (c) => c.status === "error"
    ).length;
    const warningChecks = Object.values(checks).filter(
      (c) => c.status === "warning"
    ).length;

    let status: HealthCheckResult["status"] = "healthy";
    if (errorChecks > 0) status = "unhealthy";
    else if (warningChecks > 0) status = "degraded";

    const result: HealthCheckResult = {
      status,
      timestamp,
      checks,
      recommendations,
    };

    this.healthHistory.push(result);
    this.emit("health_check_completed", result);

    // Keep only recent health history
    if (this.healthHistory.length > 100) {
      this.healthHistory = this.healthHistory.slice(-50);
    }

    return result;
  }

  /**
   * Get current health status
   */
  getHealthStatus(): HealthCheckResult | null {
    return this.healthHistory[this.healthHistory.length - 1] || null;
  }

  /**
   * Get health history
   */
  getHealthHistory(limit: number = 10): HealthCheckResult[] {
    return this.healthHistory.slice(-limit);
  }

  /**
   * Get latest performance snapshot
   */
  getLatestSnapshot(): PerformanceSnapshot | null {
    return this.lastSnapshot || null;
  }

  /**
   * Export monitoring data for external systems
   */
  exportMetrics(): {
    health: HealthCheckResult | null;
    snapshot: PerformanceSnapshot | null;
    config: typeof PERFORMANCE_CONFIG;
  } {
    return {
      health: this.getHealthStatus(),
      snapshot: this.getLatestSnapshot(),
      config: PERFORMANCE_CONFIG,
    };
  }

  /**
   * Setup Node.js performance observer for GC and other metrics
   */
  private setupPerformanceObserver(): void {
    try {
      this.performanceObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        for (const entry of entries) {
          if (entry.entryType === "gc") {
            this.emit("gc_event", {
              type: entry.name,
              duration: entry.duration,
              timestamp: new Date().toISOString(),
            });
          }
        }
      });

      this.performanceObserver.observe({ entryTypes: ["gc"] });
    } catch (error) {
      // Performance observer not available in all environments
      console.warn("Performance observer not available:", error);
    }
  }

  /**
   * Calculate throughput from stats
   */
  private calculateThroughput(stats: any): number {
    if (!stats.eventsCollected || !stats.timeRunningMs) return 0;
    return (stats.eventsCollected / stats.timeRunningMs) * 1000;
  }

  /**
   * Calculate average quality score from batches
   */
  private calculateAverageQualityScore(batches: any[]): number {
    if (batches.length === 0) return 0;
    const totalScore = batches.reduce(
      (sum, batch) => sum + (batch.qualityScore || 0),
      0
    );
    return totalScore / batches.length;
  }

  /**
   * Calculate CPU usage percentage
   */
  private calculateCpuUsagePercent(cpuUsage: NodeJS.CpuUsage): number {
    const total = cpuUsage.user + cpuUsage.system;
    // Rough approximation - would need time-based measurement for accuracy
    return Math.min((total / 1000000) * 100, 100);
  }

  /**
   * Update component references (for dynamic component loading)
   */
  updateComponents(components: {
    dataCollector?: DataCollector;
    metricAggregator?: MetricAggregator;
    rlDataPipeline?: RLDataPipeline;
    performanceAnalyzer?: PerformanceAnalyzer;
  }): void {
    this.dataCollector = components.dataCollector;
    this.metricAggregator = components.metricAggregator;
    this.rlDataPipeline = components.rlDataPipeline;
    this.performanceAnalyzer = components.performanceAnalyzer;

    this.emit("components_updated");
  }
}

// Global performance monitor instance
let globalMonitor: PerformanceMonitor | null = null;

/**
 * Get or create global performance monitor
 */
export function getPerformanceMonitor(components?: {
  dataCollector?: DataCollector;
  metricAggregator?: MetricAggregator;
  rlDataPipeline?: RLDataPipeline;
  performanceAnalyzer?: PerformanceAnalyzer;
}): PerformanceMonitor {
  if (!globalMonitor) {
    globalMonitor = new PerformanceMonitor(components);
  } else if (components) {
    globalMonitor.updateComponents(components);
  }

  return globalMonitor;
}

/**
 * Quick health check utility
 */
export async function quickHealthCheck(components?: {
  dataCollector?: DataCollector;
  metricAggregator?: MetricAggregator;
  rlDataPipeline?: RLDataPipeline;
  performanceAnalyzer?: PerformanceAnalyzer;
}): Promise<HealthCheckResult> {
  const monitor = new PerformanceMonitor(components);
  return await monitor.performHealthCheck();
}

/**
 * Performance impact measurement utility
 */
export class PerformanceImpactMeasurer {
  private baselineMetrics?: PerformanceSnapshot;

  async measureBaseline(): Promise<void> {
    const monitor = new PerformanceMonitor();
    this.baselineMetrics = await monitor.takeSnapshot();
  }

  async measureImpact(operation: () => Promise<void>): Promise<{
    latencyIncreaseMs: number;
    memoryIncreaseMB: number;
    durationMs: number;
  }> {
    if (!this.baselineMetrics) {
      await this.measureBaseline();
    }

    const startTime = performance.now();
    const startMemory = process.memoryUsage();

    await operation();

    const endTime = performance.now();
    const endMemory = process.memoryUsage();

    const monitor = new PerformanceMonitor();
    const afterSnapshot = await monitor.takeSnapshot();

    return {
      latencyIncreaseMs:
        afterSnapshot.collection.averageLatencyMs -
        (this.baselineMetrics?.collection.averageLatencyMs || 0),
      memoryIncreaseMB:
        afterSnapshot.system.heapUsedMB -
        (this.baselineMetrics?.system.heapUsedMB || 0),
      durationMs: endTime - startTime,
    };
  }
}
