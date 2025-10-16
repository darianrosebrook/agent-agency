/**
 * @fileoverview Bottleneck Detector for Runtime Optimization Engine
 *
 * Analyzes performance metrics to identify system bottlenecks.
 * Uses threshold-based detection with severity classification.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { Logger } from "@/observability/Logger";
import {
  BottleneckSeverity,
  MetricType,
  type Bottleneck,
  type IBottleneckDetector,
  type PerformanceMetric,
} from "@/types/optimization-types";
import { v4 as uuidv4 } from "uuid";

/**
 * Default thresholds for bottleneck detection
 */
const DEFAULT_THRESHOLDS: Record<MetricType, number> = {
  [MetricType.CPU]: 80, // 80% CPU usage
  [MetricType.MEMORY]: 85, // 85% memory usage
  [MetricType.NETWORK]: 75, // 75% network utilization
  [MetricType.LATENCY]: 1000, // 1000ms latency
  [MetricType.THROUGHPUT]: 100, // 100 requests/sec minimum
  [MetricType.ERROR_RATE]: 5, // 5% error rate
  [MetricType.CACHE_HIT_RATE]: 70, // 70% cache hit rate minimum
};

/**
 * Bottleneck Detector
 *
 * Identifies performance bottlenecks by:
 * - Comparing metrics against thresholds
 * - Tracking bottleneck frequency
 * - Classifying severity levels
 * - Managing bottleneck lifecycle
 */
export class BottleneckDetector implements IBottleneckDetector {
  private logger: Logger;
  private thresholds: Record<MetricType, number>;
  private activeBottlenecks: Map<string, Bottleneck> = new Map();
  private bottleneckHistory: Map<string, Bottleneck[]> = new Map();

  constructor(thresholds: Partial<Record<MetricType, number>> = {}) {
    this.logger = new Logger("BottleneckDetector");
    this.thresholds = { ...DEFAULT_THRESHOLDS, ...thresholds };
  }

  /**
   * Detect bottlenecks from metrics
   *
   * @param metrics Performance metrics to analyze
   * @returns Detected bottlenecks
   */
  async detectBottlenecks(metrics: PerformanceMetric[]): Promise<Bottleneck[]> {
    const detectedBottlenecks: Bottleneck[] = [];

    // Group metrics by component and type
    const metricsByComponent = this.groupMetricsByComponent(metrics);

    for (const [component, componentMetrics] of metricsByComponent) {
      for (const metric of componentMetrics) {
        const bottleneck = await this.checkMetricThreshold(component, metric);
        if (bottleneck) {
          detectedBottlenecks.push(bottleneck);
        }
      }
    }

    this.logger.debug("Bottleneck detection completed", {
      metricsAnalyzed: metrics.length,
      bottlenecksDetected: detectedBottlenecks.length,
    });

    return detectedBottlenecks;
  }

  /**
   * Update bottleneck thresholds
   *
   * @param thresholds New threshold values
   */
  updateThresholds(thresholds: Partial<Record<MetricType, number>>): void {
    this.thresholds = { ...this.thresholds, ...thresholds };
    this.logger.info("Thresholds updated", this.thresholds);
  }

  /**
   * Get active bottlenecks
   *
   * @returns Currently active bottlenecks
   */
  getActiveBottlenecks(): Bottleneck[] {
    return Array.from(this.activeBottlenecks.values());
  }

  /**
   * Clear resolved bottlenecks
   *
   * @param olderThan Clear bottlenecks older than this date
   */
  async clearResolvedBottlenecks(olderThan: Date): Promise<void> {
    const beforeCount = this.activeBottlenecks.size;

    for (const [key, bottleneck] of this.activeBottlenecks) {
      if (bottleneck.lastObservedAt < olderThan) {
        // Move to history before removing
        const history = this.bottleneckHistory.get(bottleneck.component) ?? [];
        history.push(bottleneck);
        this.bottleneckHistory.set(bottleneck.component, history);

        this.activeBottlenecks.delete(key);
      }
    }

    const clearedCount = beforeCount - this.activeBottlenecks.size;

    if (clearedCount > 0) {
      this.logger.debug("Cleared resolved bottlenecks", {
        clearedCount,
        remainingCount: this.activeBottlenecks.size,
      });
    }
  }

  /**
   * Get bottleneck history for a component
   *
   * @param component Component name
   * @returns Historical bottlenecks
   */
  getBottleneckHistory(component: string): Bottleneck[] {
    return this.bottleneckHistory.get(component) ?? [];
  }

  /**
   * Check if a metric exceeds threshold
   *
   * @param component Component name
   * @param metric Performance metric
   * @returns Bottleneck if threshold exceeded, null otherwise
   */
  private async checkMetricThreshold(
    component: string,
    metric: PerformanceMetric
  ): Promise<Bottleneck | null> {
    const threshold = this.thresholds[metric.type];
    if (threshold === undefined) {
      return null;
    }

    // Check if metric exceeds threshold
    const exceedsThreshold = this.metricsExceedsThreshold(metric, threshold);
    if (!exceedsThreshold) {
      return null;
    }

    // Generate bottleneck key
    const bottleneckKey = `${component}-${metric.type}`;

    // Check if bottleneck already exists
    const existingBottleneck = this.activeBottlenecks.get(bottleneckKey);

    if (existingBottleneck) {
      // Update existing bottleneck
      existingBottleneck.currentValue = metric.value;
      existingBottleneck.lastObservedAt = metric.timestamp;
      existingBottleneck.occurrenceCount += 1;

      // Update severity based on occurrence count and value
      existingBottleneck.severity = this.calculateSeverity(
        metric,
        threshold,
        existingBottleneck.occurrenceCount
      );

      return existingBottleneck;
    }

    // Create new bottleneck
    const severity = this.calculateSeverity(metric, threshold, 1);
    const bottleneck: Bottleneck = {
      id: uuidv4(),
      component,
      severity,
      metricType: metric.type,
      currentValue: metric.value,
      threshold,
      impact: this.generateImpactDescription(component, metric.type, severity),
      detectedAt: metric.timestamp,
      lastObservedAt: metric.timestamp,
      occurrenceCount: 1,
    };

    this.activeBottlenecks.set(bottleneckKey, bottleneck);

    this.logger.warn("New bottleneck detected", {
      component,
      metricType: metric.type,
      severity,
      currentValue: metric.value,
      threshold,
    });

    return bottleneck;
  }

  /**
   * Check if metric exceeds threshold
   *
   * @param metric Performance metric
   * @param threshold Threshold value
   * @returns True if threshold exceeded
   */
  private metricsExceedsThreshold(
    metric: PerformanceMetric,
    threshold: number
  ): boolean {
    // For some metrics, lower is better (e.g., throughput, cache hit rate)
    if (
      metric.type === MetricType.THROUGHPUT ||
      metric.type === MetricType.CACHE_HIT_RATE
    ) {
      return metric.value < threshold;
    }

    // For most metrics, higher is worse (e.g., CPU, memory, latency, error rate)
    return metric.value > threshold;
  }

  /**
   * Calculate bottleneck severity
   *
   * @param metric Performance metric
   * @param threshold Threshold value
   * @param occurrenceCount Number of times observed
   * @returns Severity level
   */
  private calculateSeverity(
    metric: PerformanceMetric,
    threshold: number,
    occurrenceCount: number
  ): BottleneckSeverity {
    const deviation = Math.abs((metric.value - threshold) / threshold);

    // Adjust severity based on deviation and occurrence frequency
    if (deviation >= 0.5 || occurrenceCount >= 10) {
      return BottleneckSeverity.CRITICAL;
    } else if (deviation >= 0.3 || occurrenceCount >= 5) {
      return BottleneckSeverity.HIGH;
    } else if (deviation >= 0.15 || occurrenceCount >= 3) {
      return BottleneckSeverity.MEDIUM;
    } else {
      return BottleneckSeverity.LOW;
    }
  }

  /**
   * Generate impact description for a bottleneck
   *
   * @param component Component name
   * @param metricType Metric type
   * @param severity Severity level
   * @returns Impact description
   */
  private generateImpactDescription(
    component: string,
    metricType: MetricType,
    severity: BottleneckSeverity
  ): string {
    const impactMap: Record<MetricType, string> = {
      [MetricType.CPU]:
        "High CPU usage may cause slow response times and task delays",
      [MetricType.MEMORY]:
        "High memory usage may cause system instability and crashes",
      [MetricType.NETWORK]: "Network congestion may cause communication delays",
      [MetricType.LATENCY]:
        "High latency degrades user experience and system responsiveness",
      [MetricType.THROUGHPUT]:
        "Low throughput limits system capacity and request handling",
      [MetricType.ERROR_RATE]: "High error rate indicates reliability issues",
      [MetricType.CACHE_HIT_RATE]:
        "Low cache hit rate increases backend load and latency",
    };

    const baseImpact =
      impactMap[metricType] ?? "Performance degradation detected";
    const severityPrefix =
      severity === BottleneckSeverity.CRITICAL ||
      severity === BottleneckSeverity.HIGH
        ? "URGENT: "
        : "";

    return `${severityPrefix}${baseImpact} in ${component}`;
  }

  /**
   * Group metrics by component
   *
   * @param metrics Performance metrics
   * @returns Map of component to metrics
   */
  private groupMetricsByComponent(
    metrics: PerformanceMetric[]
  ): Map<string, PerformanceMetric[]> {
    const grouped = new Map<string, PerformanceMetric[]>();

    for (const metric of metrics) {
      const component = metric.source;
      const componentMetrics = grouped.get(component) ?? [];
      componentMetrics.push(metric);
      grouped.set(component, componentMetrics);
    }

    return grouped;
  }
}
