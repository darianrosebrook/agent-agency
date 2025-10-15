/**
 * @fileoverview Bottleneck Detector for Runtime Optimization Engine
 *
 * Analyzes performance metrics to identify system bottlenecks.
 * Uses threshold-based detection with severity classification.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck
function stryNS_9fa48() {
  var g = typeof globalThis === 'object' && globalThis && globalThis.Math === Math && globalThis || new Function("return this")();
  var ns = g.__stryker__ || (g.__stryker__ = {});
  if (ns.activeMutant === undefined && g.process && g.process.env && g.process.env.__STRYKER_ACTIVE_MUTANT__) {
    ns.activeMutant = g.process.env.__STRYKER_ACTIVE_MUTANT__;
  }
  function retrieveNS() {
    return ns;
  }
  stryNS_9fa48 = retrieveNS;
  return retrieveNS();
}
stryNS_9fa48();
function stryCov_9fa48() {
  var ns = stryNS_9fa48();
  var cov = ns.mutantCoverage || (ns.mutantCoverage = {
    static: {},
    perTest: {}
  });
  function cover() {
    var c = cov.static;
    if (ns.currentTestId) {
      c = cov.perTest[ns.currentTestId] = cov.perTest[ns.currentTestId] || {};
    }
    var a = arguments;
    for (var i = 0; i < a.length; i++) {
      c[a[i]] = (c[a[i]] || 0) + 1;
    }
  }
  stryCov_9fa48 = cover;
  cover.apply(null, arguments);
}
function stryMutAct_9fa48(id) {
  var ns = stryNS_9fa48();
  function isActive(id) {
    if (ns.activeMutant === id) {
      if (ns.hitCount !== void 0 && ++ns.hitCount > ns.hitLimit) {
        throw new Error('Stryker: Hit count limit reached (' + ns.hitCount + ')');
      }
      return true;
    }
    return false;
  }
  stryMutAct_9fa48 = isActive;
  return isActive(id);
}
import { Logger } from "@/observability/Logger";
import { BottleneckSeverity, MetricType, type Bottleneck, type IBottleneckDetector, type PerformanceMetric } from "@/types/optimization-types";
import { v4 as uuidv4 } from "uuid";

/**
 * Default thresholds for bottleneck detection
 */
const DEFAULT_THRESHOLDS: Record<MetricType, number> = stryMutAct_9fa48("0") ? {} : (stryCov_9fa48("0"), {
  [MetricType.CPU]: 80,
  // 80% CPU usage
  [MetricType.MEMORY]: 85,
  // 85% memory usage
  [MetricType.NETWORK]: 75,
  // 75% network utilization
  [MetricType.LATENCY]: 1000,
  // 1000ms latency
  [MetricType.THROUGHPUT]: 100,
  // 100 requests/sec minimum
  [MetricType.ERROR_RATE]: 5,
  // 5% error rate
  [MetricType.CACHE_HIT_RATE]: 70 // 70% cache hit rate minimum
});

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
    if (stryMutAct_9fa48("1")) {
      {}
    } else {
      stryCov_9fa48("1");
      this.logger = new Logger("BottleneckDetector");
      this.thresholds = stryMutAct_9fa48("3") ? {} : (stryCov_9fa48("3"), {
        ...DEFAULT_THRESHOLDS,
        ...thresholds
      });
    }
  }

  /**
   * Detect bottlenecks from metrics
   *
   * @param metrics Performance metrics to analyze
   * @returns Detected bottlenecks
   */
  async detectBottlenecks(metrics: PerformanceMetric[]): Promise<Bottleneck[]> {
    if (stryMutAct_9fa48("4")) {
      {}
    } else {
      stryCov_9fa48("4");
      const detectedBottlenecks: Bottleneck[] = [];

      // Group metrics by component and type
      const metricsByComponent = this.groupMetricsByComponent(metrics);
      for (const [component, componentMetrics] of metricsByComponent) {
        if (stryMutAct_9fa48("6")) {
          {}
        } else {
          stryCov_9fa48("6");
          for (const metric of componentMetrics) {
            if (stryMutAct_9fa48("7")) {
              {}
            } else {
              stryCov_9fa48("7");
              const bottleneck = await this.checkMetricThreshold(component, metric);
              if (stryMutAct_9fa48("9") ? false : stryMutAct_9fa48("8") ? true : (stryCov_9fa48("8", "9"), bottleneck)) {
                if (stryMutAct_9fa48("10")) {
                  {}
                } else {
                  stryCov_9fa48("10");
                  detectedBottlenecks.push(bottleneck);
                }
              }
            }
          }
        }
      }
      this.logger.debug("Bottleneck detection completed", stryMutAct_9fa48("12") ? {} : (stryCov_9fa48("12"), {
        metricsAnalyzed: metrics.length,
        bottlenecksDetected: detectedBottlenecks.length
      }));
      return detectedBottlenecks;
    }
  }

  /**
   * Update bottleneck thresholds
   *
   * @param thresholds New threshold values
   */
  updateThresholds(thresholds: Partial<Record<MetricType, number>>): void {
    if (stryMutAct_9fa48("13")) {
      {}
    } else {
      stryCov_9fa48("13");
      this.thresholds = stryMutAct_9fa48("14") ? {} : (stryCov_9fa48("14"), {
        ...this.thresholds,
        ...thresholds
      });
      this.logger.info("Thresholds updated", this.thresholds);
    }
  }

  /**
   * Get active bottlenecks
   *
   * @returns Currently active bottlenecks
   */
  getActiveBottlenecks(): Bottleneck[] {
    if (stryMutAct_9fa48("16")) {
      {}
    } else {
      stryCov_9fa48("16");
      return Array.from(this.activeBottlenecks.values());
    }
  }

  /**
   * Clear resolved bottlenecks
   *
   * @param olderThan Clear bottlenecks older than this date
   */
  async clearResolvedBottlenecks(olderThan: Date): Promise<void> {
    if (stryMutAct_9fa48("17")) {
      {}
    } else {
      stryCov_9fa48("17");
      const beforeCount = this.activeBottlenecks.size;
      for (const [key, bottleneck] of this.activeBottlenecks) {
        if (stryMutAct_9fa48("18")) {
          {}
        } else {
          stryCov_9fa48("18");
          if (stryMutAct_9fa48("22") ? bottleneck.lastObservedAt >= olderThan : stryMutAct_9fa48("21") ? bottleneck.lastObservedAt <= olderThan : stryMutAct_9fa48("20") ? false : stryMutAct_9fa48("19") ? true : (stryCov_9fa48("19", "20", "21", "22"), bottleneck.lastObservedAt < olderThan)) {
            if (stryMutAct_9fa48("23")) {
              {}
            } else {
              stryCov_9fa48("23");
              // Move to history before removing
              const history = stryMutAct_9fa48("24") ? this.bottleneckHistory.get(bottleneck.component) && [] : (stryCov_9fa48("24"), this.bottleneckHistory.get(bottleneck.component) ?? []);
              history.push(bottleneck);
              this.bottleneckHistory.set(bottleneck.component, history);
              this.activeBottlenecks.delete(key);
            }
          }
        }
      }
      const clearedCount = stryMutAct_9fa48("26") ? beforeCount + this.activeBottlenecks.size : (stryCov_9fa48("26"), beforeCount - this.activeBottlenecks.size);
      if (stryMutAct_9fa48("30") ? clearedCount <= 0 : stryMutAct_9fa48("29") ? clearedCount >= 0 : stryMutAct_9fa48("28") ? false : stryMutAct_9fa48("27") ? true : (stryCov_9fa48("27", "28", "29", "30"), clearedCount > 0)) {
        if (stryMutAct_9fa48("31")) {
          {}
        } else {
          stryCov_9fa48("31");
          this.logger.debug("Cleared resolved bottlenecks", stryMutAct_9fa48("33") ? {} : (stryCov_9fa48("33"), {
            clearedCount,
            remainingCount: this.activeBottlenecks.size
          }));
        }
      }
    }
  }

  /**
   * Get bottleneck history for a component
   *
   * @param component Component name
   * @returns Historical bottlenecks
   */
  getBottleneckHistory(component: string): Bottleneck[] {
    if (stryMutAct_9fa48("34")) {
      {}
    } else {
      stryCov_9fa48("34");
      return stryMutAct_9fa48("35") ? this.bottleneckHistory.get(component) && [] : (stryCov_9fa48("35"), this.bottleneckHistory.get(component) ?? []);
    }
  }

  /**
   * Check if a metric exceeds threshold
   *
   * @param component Component name
   * @param metric Performance metric
   * @returns Bottleneck if threshold exceeded, null otherwise
   */
  private async checkMetricThreshold(component: string, metric: PerformanceMetric): Promise<Bottleneck | null> {
    if (stryMutAct_9fa48("37")) {
      {}
    } else {
      stryCov_9fa48("37");
      const threshold = this.thresholds[metric.type];
      if (stryMutAct_9fa48("40") ? threshold !== undefined : stryMutAct_9fa48("39") ? false : stryMutAct_9fa48("38") ? true : (stryCov_9fa48("38", "39", "40"), threshold === undefined)) {
        if (stryMutAct_9fa48("41")) {
          {}
        } else {
          stryCov_9fa48("41");
          return null;
        }
      }

      // Check if metric exceeds threshold
      const exceedsThreshold = this.metricsExceedsThreshold(metric, threshold);
      if (stryMutAct_9fa48("44") ? false : stryMutAct_9fa48("43") ? true : stryMutAct_9fa48("42") ? exceedsThreshold : (stryCov_9fa48("42", "43", "44"), !exceedsThreshold)) {
        if (stryMutAct_9fa48("45")) {
          {}
        } else {
          stryCov_9fa48("45");
          return null;
        }
      }

      // Generate bottleneck key
      const bottleneckKey = `${component}-${metric.type}`;

      // Check if bottleneck already exists
      const existingBottleneck = this.activeBottlenecks.get(bottleneckKey);
      if (stryMutAct_9fa48("48") ? false : stryMutAct_9fa48("47") ? true : (stryCov_9fa48("47", "48"), existingBottleneck)) {
        if (stryMutAct_9fa48("49")) {
          {}
        } else {
          stryCov_9fa48("49");
          // Update existing bottleneck
          existingBottleneck.currentValue = metric.value;
          existingBottleneck.lastObservedAt = metric.timestamp;
          stryMutAct_9fa48("50") ? existingBottleneck.occurrenceCount -= 1 : (stryCov_9fa48("50"), existingBottleneck.occurrenceCount += 1);

          // Update severity based on occurrence count and value
          existingBottleneck.severity = this.calculateSeverity(metric, threshold, existingBottleneck.occurrenceCount);
          return existingBottleneck;
        }
      }

      // Create new bottleneck
      const severity = this.calculateSeverity(metric, threshold, 1);
      const bottleneck: Bottleneck = stryMutAct_9fa48("51") ? {} : (stryCov_9fa48("51"), {
        id: uuidv4(),
        component,
        severity,
        metricType: metric.type,
        currentValue: metric.value,
        threshold,
        impact: this.generateImpactDescription(component, metric.type, severity),
        detectedAt: metric.timestamp,
        lastObservedAt: metric.timestamp,
        occurrenceCount: 1
      });
      this.activeBottlenecks.set(bottleneckKey, bottleneck);
      this.logger.warn("New bottleneck detected", stryMutAct_9fa48("53") ? {} : (stryCov_9fa48("53"), {
        component,
        metricType: metric.type,
        severity,
        currentValue: metric.value,
        threshold
      }));
      return bottleneck;
    }
  }

  /**
   * Check if metric exceeds threshold
   *
   * @param metric Performance metric
   * @param threshold Threshold value
   * @returns True if threshold exceeded
   */
  private metricsExceedsThreshold(metric: PerformanceMetric, threshold: number): boolean {
    if (stryMutAct_9fa48("54")) {
      {}
    } else {
      stryCov_9fa48("54");
      // For some metrics, lower is better (e.g., throughput, cache hit rate)
      if (stryMutAct_9fa48("57") ? metric.type === MetricType.THROUGHPUT && metric.type === MetricType.CACHE_HIT_RATE : stryMutAct_9fa48("56") ? false : stryMutAct_9fa48("55") ? true : (stryCov_9fa48("55", "56", "57"), (stryMutAct_9fa48("59") ? metric.type !== MetricType.THROUGHPUT : stryMutAct_9fa48("58") ? false : (stryCov_9fa48("58", "59"), metric.type === MetricType.THROUGHPUT)) || (stryMutAct_9fa48("61") ? metric.type !== MetricType.CACHE_HIT_RATE : stryMutAct_9fa48("60") ? false : (stryCov_9fa48("60", "61"), metric.type === MetricType.CACHE_HIT_RATE)))) {
        if (stryMutAct_9fa48("62")) {
          {}
        } else {
          stryCov_9fa48("62");
          return stryMutAct_9fa48("66") ? metric.value >= threshold : stryMutAct_9fa48("65") ? metric.value <= threshold : stryMutAct_9fa48("64") ? false : stryMutAct_9fa48("63") ? true : (stryCov_9fa48("63", "64", "65", "66"), metric.value < threshold);
        }
      }

      // For most metrics, higher is worse (e.g., CPU, memory, latency, error rate)
      return stryMutAct_9fa48("70") ? metric.value <= threshold : stryMutAct_9fa48("69") ? metric.value >= threshold : stryMutAct_9fa48("68") ? false : stryMutAct_9fa48("67") ? true : (stryCov_9fa48("67", "68", "69", "70"), metric.value > threshold);
    }
  }

  /**
   * Calculate bottleneck severity
   *
   * @param metric Performance metric
   * @param threshold Threshold value
   * @param occurrenceCount Number of times observed
   * @returns Severity level
   */
  private calculateSeverity(metric: PerformanceMetric, threshold: number, occurrenceCount: number): BottleneckSeverity {
    if (stryMutAct_9fa48("71")) {
      {}
    } else {
      stryCov_9fa48("71");
      const deviation = Math.abs(stryMutAct_9fa48("72") ? (metric.value - threshold) * threshold : (stryCov_9fa48("72"), (stryMutAct_9fa48("73") ? metric.value + threshold : (stryCov_9fa48("73"), metric.value - threshold)) / threshold));

      // Adjust severity based on deviation and occurrence frequency
      if (stryMutAct_9fa48("76") ? deviation >= 0.5 && occurrenceCount >= 10 : stryMutAct_9fa48("75") ? false : stryMutAct_9fa48("74") ? true : (stryCov_9fa48("74", "75", "76"), (stryMutAct_9fa48("79") ? deviation < 0.5 : stryMutAct_9fa48("78") ? deviation > 0.5 : stryMutAct_9fa48("77") ? false : (stryCov_9fa48("77", "78", "79"), deviation >= 0.5)) || (stryMutAct_9fa48("82") ? occurrenceCount < 10 : stryMutAct_9fa48("81") ? occurrenceCount > 10 : stryMutAct_9fa48("80") ? false : (stryCov_9fa48("80", "81", "82"), occurrenceCount >= 10)))) {
        if (stryMutAct_9fa48("83")) {
          {}
        } else {
          stryCov_9fa48("83");
          return BottleneckSeverity.CRITICAL;
        }
      } else if (stryMutAct_9fa48("86") ? deviation >= 0.3 && occurrenceCount >= 5 : stryMutAct_9fa48("85") ? false : stryMutAct_9fa48("84") ? true : (stryCov_9fa48("84", "85", "86"), (stryMutAct_9fa48("89") ? deviation < 0.3 : stryMutAct_9fa48("88") ? deviation > 0.3 : stryMutAct_9fa48("87") ? false : (stryCov_9fa48("87", "88", "89"), deviation >= 0.3)) || (stryMutAct_9fa48("92") ? occurrenceCount < 5 : stryMutAct_9fa48("91") ? occurrenceCount > 5 : stryMutAct_9fa48("90") ? false : (stryCov_9fa48("90", "91", "92"), occurrenceCount >= 5)))) {
        if (stryMutAct_9fa48("93")) {
          {}
        } else {
          stryCov_9fa48("93");
          return BottleneckSeverity.HIGH;
        }
      } else if (stryMutAct_9fa48("96") ? deviation >= 0.15 && occurrenceCount >= 3 : stryMutAct_9fa48("95") ? false : stryMutAct_9fa48("94") ? true : (stryCov_9fa48("94", "95", "96"), (stryMutAct_9fa48("99") ? deviation < 0.15 : stryMutAct_9fa48("98") ? deviation > 0.15 : stryMutAct_9fa48("97") ? false : (stryCov_9fa48("97", "98", "99"), deviation >= 0.15)) || (stryMutAct_9fa48("102") ? occurrenceCount < 3 : stryMutAct_9fa48("101") ? occurrenceCount > 3 : stryMutAct_9fa48("100") ? false : (stryCov_9fa48("100", "101", "102"), occurrenceCount >= 3)))) {
        if (stryMutAct_9fa48("103")) {
          {}
        } else {
          stryCov_9fa48("103");
          return BottleneckSeverity.MEDIUM;
        }
      } else {
        if (stryMutAct_9fa48("104")) {
          {}
        } else {
          stryCov_9fa48("104");
          return BottleneckSeverity.LOW;
        }
      }
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
  private generateImpactDescription(component: string, metricType: MetricType, severity: BottleneckSeverity): string {
    if (stryMutAct_9fa48("105")) {
      {}
    } else {
      stryCov_9fa48("105");
      const impactMap: Record<MetricType, string> = stryMutAct_9fa48("106") ? {} : (stryCov_9fa48("106"), {
        [MetricType.CPU]: "High CPU usage may cause slow response times and task delays",
        [MetricType.MEMORY]: "High memory usage may cause system instability and crashes",
        [MetricType.NETWORK]: "Network congestion may cause communication delays",
        [MetricType.LATENCY]: "High latency degrades user experience and system responsiveness",
        [MetricType.THROUGHPUT]: "Low throughput limits system capacity and request handling",
        [MetricType.ERROR_RATE]: "High error rate indicates reliability issues",
        [MetricType.CACHE_HIT_RATE]: "Low cache hit rate increases backend load and latency"
      });
      const baseImpact = stryMutAct_9fa48("114") ? impactMap[metricType] && "Performance degradation detected" : (stryCov_9fa48("114"), impactMap[metricType] ?? "Performance degradation detected");
      const severityPrefix = (stryMutAct_9fa48("118") ? severity === BottleneckSeverity.CRITICAL && severity === BottleneckSeverity.HIGH : stryMutAct_9fa48("117") ? false : stryMutAct_9fa48("116") ? true : (stryCov_9fa48("116", "117", "118"), (stryMutAct_9fa48("120") ? severity !== BottleneckSeverity.CRITICAL : stryMutAct_9fa48("119") ? false : (stryCov_9fa48("119", "120"), severity === BottleneckSeverity.CRITICAL)) || (stryMutAct_9fa48("122") ? severity !== BottleneckSeverity.HIGH : stryMutAct_9fa48("121") ? false : (stryCov_9fa48("121", "122"), severity === BottleneckSeverity.HIGH)))) ? "URGENT: " : "";
      return `${severityPrefix}${baseImpact} in ${component}`;
    }
  }

  /**
   * Group metrics by component
   *
   * @param metrics Performance metrics
   * @returns Map of component to metrics
   */
  private groupMetricsByComponent(metrics: PerformanceMetric[]): Map<string, PerformanceMetric[]> {
    if (stryMutAct_9fa48("126")) {
      {}
    } else {
      stryCov_9fa48("126");
      const grouped = new Map<string, PerformanceMetric[]>();
      for (const metric of metrics) {
        if (stryMutAct_9fa48("127")) {
          {}
        } else {
          stryCov_9fa48("127");
          const component = metric.source;
          const componentMetrics = stryMutAct_9fa48("128") ? grouped.get(component) && [] : (stryCov_9fa48("128"), grouped.get(component) ?? []);
          componentMetrics.push(metric);
          grouped.set(component, componentMetrics);
        }
      }
      return grouped;
    }
  }
}