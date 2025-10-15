/**
 * @fileoverview Performance Monitor for Runtime Optimization Engine
 *
 * Collects and stores performance metrics with minimal overhead.
 * Implements circular buffer for efficient memory management.
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
import { MetricType, type IPerformanceMonitor, type PerformanceMetric } from "@/types/optimization-types";

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
const DEFAULT_CONFIG: PerformanceMonitorConfig = stryMutAct_9fa48("130") ? {} : (stryCov_9fa48("130"), {
  maxMetrics: 10000,
  autoCleanOlderThanMs: 3600000,
  // 1 hour
  enableAutoCleanup: stryMutAct_9fa48("131") ? false : (stryCov_9fa48("131"), true),
  cleanupIntervalMs: 300000 // 5 minutes
});

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
  private metricsLock = stryMutAct_9fa48("133") ? true : (stryCov_9fa48("133"), false);
  constructor(config: Partial<PerformanceMonitorConfig> = {}) {
    if (stryMutAct_9fa48("134")) {
      {}
    } else {
      stryCov_9fa48("134");
      this.logger = new Logger("PerformanceMonitor");
      this.config = stryMutAct_9fa48("136") ? {} : (stryCov_9fa48("136"), {
        ...DEFAULT_CONFIG,
        ...config
      });
    }
  }

  /**
   * Start the performance monitor
   */
  async start(): Promise<void> {
    if (stryMutAct_9fa48("137")) {
      {}
    } else {
      stryCov_9fa48("137");
      // Clear any existing timer first to prevent multiple timers
      if (stryMutAct_9fa48("139") ? false : stryMutAct_9fa48("138") ? true : (stryCov_9fa48("138", "139"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("140")) {
          {}
        } else {
          stryCov_9fa48("140");
          clearInterval(this.cleanupTimer);
          this.cleanupTimer = undefined;
        }
      }
      if (stryMutAct_9fa48("142") ? false : stryMutAct_9fa48("141") ? true : (stryCov_9fa48("141", "142"), this.config.enableAutoCleanup)) {
        if (stryMutAct_9fa48("143")) {
          {}
        } else {
          stryCov_9fa48("143");
          this.startAutoCleanup();
        }
      }
      this.logger.info("Performance monitor started", stryMutAct_9fa48("145") ? {} : (stryCov_9fa48("145"), {
        maxMetrics: this.config.maxMetrics,
        autoCleanup: this.config.enableAutoCleanup
      }));
    }
  }

  /**
   * Stop the performance monitor
   */
  async stop(): Promise<void> {
    if (stryMutAct_9fa48("146")) {
      {}
    } else {
      stryCov_9fa48("146");
      if (stryMutAct_9fa48("148") ? false : stryMutAct_9fa48("147") ? true : (stryCov_9fa48("147", "148"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("149")) {
          {}
        } else {
          stryCov_9fa48("149");
          clearInterval(this.cleanupTimer);
          this.cleanupTimer = undefined;
        }
      }
      this.logger.info("Performance monitor stopped");
    }
  }

  /**
   * Record a performance metric
   *
   * @param metric Performance metric to record
   */
  async recordMetric(metric: PerformanceMetric): Promise<void> {
    if (stryMutAct_9fa48("151")) {
      {}
    } else {
      stryCov_9fa48("151");
      await this.withLock(async () => {
        if (stryMutAct_9fa48("152")) {
          {}
        } else {
          stryCov_9fa48("152");
          // Add metric to buffer
          this.metrics.push(metric);

          // If buffer is full, remove oldest metric (circular buffer)
          if (stryMutAct_9fa48("156") ? this.metrics.length <= this.config.maxMetrics : stryMutAct_9fa48("155") ? this.metrics.length >= this.config.maxMetrics : stryMutAct_9fa48("154") ? false : stryMutAct_9fa48("153") ? true : (stryCov_9fa48("153", "154", "155", "156"), this.metrics.length > this.config.maxMetrics)) {
            if (stryMutAct_9fa48("157")) {
              {}
            } else {
              stryCov_9fa48("157");
              this.metrics.shift();
            }
          }
        }
      });
    }
  }

  /**
   * Get metrics for a time window
   *
   * @param startTime Window start time
   * @param endTime Window end time
   * @param metricType Optional metric type filter
   * @returns Metrics within time window
   */
  async getMetrics(startTime: Date, endTime: Date, metricType?: MetricType): Promise<PerformanceMetric[]> {
    if (stryMutAct_9fa48("158")) {
      {}
    } else {
      stryCov_9fa48("158");
      return this.withLock(async () => {
        if (stryMutAct_9fa48("159")) {
          {}
        } else {
          stryCov_9fa48("159");
          return stryMutAct_9fa48("160") ? this.metrics : (stryCov_9fa48("160"), this.metrics.filter(metric => {
            if (stryMutAct_9fa48("161")) {
              {}
            } else {
              stryCov_9fa48("161");
              const inTimeRange = stryMutAct_9fa48("164") ? metric.timestamp >= startTime || metric.timestamp <= endTime : stryMutAct_9fa48("163") ? false : stryMutAct_9fa48("162") ? true : (stryCov_9fa48("162", "163", "164"), (stryMutAct_9fa48("167") ? metric.timestamp < startTime : stryMutAct_9fa48("166") ? metric.timestamp > startTime : stryMutAct_9fa48("165") ? true : (stryCov_9fa48("165", "166", "167"), metric.timestamp >= startTime)) && (stryMutAct_9fa48("170") ? metric.timestamp > endTime : stryMutAct_9fa48("169") ? metric.timestamp < endTime : stryMutAct_9fa48("168") ? true : (stryCov_9fa48("168", "169", "170"), metric.timestamp <= endTime)));
              if (stryMutAct_9fa48("173") ? false : stryMutAct_9fa48("172") ? true : stryMutAct_9fa48("171") ? metricType : (stryCov_9fa48("171", "172", "173"), !metricType)) {
                if (stryMutAct_9fa48("174")) {
                  {}
                } else {
                  stryCov_9fa48("174");
                  return inTimeRange;
                }
              }
              return stryMutAct_9fa48("177") ? inTimeRange || metric.type === metricType : stryMutAct_9fa48("176") ? false : stryMutAct_9fa48("175") ? true : (stryCov_9fa48("175", "176", "177"), inTimeRange && (stryMutAct_9fa48("179") ? metric.type !== metricType : stryMutAct_9fa48("178") ? true : (stryCov_9fa48("178", "179"), metric.type === metricType)));
            }
          }));
        }
      });
    }
  }

  /**
   * Get latest metrics
   *
   * @param count Number of metrics to retrieve
   * @param metricType Optional metric type filter
   * @returns Latest metrics
   */
  async getLatestMetrics(count: number, metricType?: MetricType): Promise<PerformanceMetric[]> {
    if (stryMutAct_9fa48("180")) {
      {}
    } else {
      stryCov_9fa48("180");
      return this.withLock(async () => {
        if (stryMutAct_9fa48("181")) {
          {}
        } else {
          stryCov_9fa48("181");
          let filtered = this.metrics;
          if (stryMutAct_9fa48("183") ? false : stryMutAct_9fa48("182") ? true : (stryCov_9fa48("182", "183"), metricType)) {
            if (stryMutAct_9fa48("184")) {
              {}
            } else {
              stryCov_9fa48("184");
              filtered = stryMutAct_9fa48("185") ? this.metrics : (stryCov_9fa48("185"), this.metrics.filter(stryMutAct_9fa48("186") ? () => undefined : (stryCov_9fa48("186"), m => stryMutAct_9fa48("189") ? m.type !== metricType : stryMutAct_9fa48("188") ? false : stryMutAct_9fa48("187") ? true : (stryCov_9fa48("187", "188", "189"), m.type === metricType))));
            }
          }

          // Return last N metrics
          return stryMutAct_9fa48("190") ? filtered : (stryCov_9fa48("190"), filtered.slice(stryMutAct_9fa48("191") ? +count : (stryCov_9fa48("191"), -count)));
        }
      });
    }
  }

  /**
   * Clear metrics older than specified date
   *
   * @param olderThan Clear metrics older than this date
   */
  async clearMetrics(olderThan: Date): Promise<void> {
    if (stryMutAct_9fa48("192")) {
      {}
    } else {
      stryCov_9fa48("192");
      await this.withLock(async () => {
        if (stryMutAct_9fa48("193")) {
          {}
        } else {
          stryCov_9fa48("193");
          const beforeCount = this.metrics.length;
          this.metrics = stryMutAct_9fa48("194") ? this.metrics : (stryCov_9fa48("194"), this.metrics.filter(stryMutAct_9fa48("195") ? () => undefined : (stryCov_9fa48("195"), m => stryMutAct_9fa48("199") ? m.timestamp < olderThan : stryMutAct_9fa48("198") ? m.timestamp > olderThan : stryMutAct_9fa48("197") ? false : stryMutAct_9fa48("196") ? true : (stryCov_9fa48("196", "197", "198", "199"), m.timestamp >= olderThan))));
          const clearedCount = stryMutAct_9fa48("200") ? beforeCount + this.metrics.length : (stryCov_9fa48("200"), beforeCount - this.metrics.length);
          if (stryMutAct_9fa48("204") ? clearedCount <= 0 : stryMutAct_9fa48("203") ? clearedCount >= 0 : stryMutAct_9fa48("202") ? false : stryMutAct_9fa48("201") ? true : (stryCov_9fa48("201", "202", "203", "204"), clearedCount > 0)) {
            if (stryMutAct_9fa48("205")) {
              {}
            } else {
              stryCov_9fa48("205");
              this.logger.debug("Cleared old metrics", stryMutAct_9fa48("207") ? {} : (stryCov_9fa48("207"), {
                clearedCount,
                remainingCount: this.metrics.length
              }));
            }
          }
        }
      });
    }
  }

  /**
   * Get current metric count
   */
  getMetricCount(): number {
    if (stryMutAct_9fa48("208")) {
      {}
    } else {
      stryCov_9fa48("208");
      return this.metrics.length;
    }
  }

  /**
   * Get configuration
   */
  getConfig(): PerformanceMonitorConfig {
    if (stryMutAct_9fa48("209")) {
      {}
    } else {
      stryCov_9fa48("209");
      return stryMutAct_9fa48("210") ? {} : (stryCov_9fa48("210"), {
        ...this.config
      });
    }
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<PerformanceMonitorConfig>): void {
    if (stryMutAct_9fa48("211")) {
      {}
    } else {
      stryCov_9fa48("211");
      this.config = stryMutAct_9fa48("212") ? {} : (stryCov_9fa48("212"), {
        ...this.config,
        ...config
      });

      // Restart auto-cleanup if needed
      if (stryMutAct_9fa48("214") ? false : stryMutAct_9fa48("213") ? true : (stryCov_9fa48("213", "214"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("215")) {
          {}
        } else {
          stryCov_9fa48("215");
          clearInterval(this.cleanupTimer);
          this.cleanupTimer = undefined;
        }
      }
      if (stryMutAct_9fa48("217") ? false : stryMutAct_9fa48("216") ? true : (stryCov_9fa48("216", "217"), this.config.enableAutoCleanup)) {
        if (stryMutAct_9fa48("218")) {
          {}
        } else {
          stryCov_9fa48("218");
          this.startAutoCleanup();
        }
      }
      this.logger.info("Configuration updated", this.config);
    }
  }

  /**
   * Start automatic cleanup
   */
  private startAutoCleanup(): void {
    if (stryMutAct_9fa48("220")) {
      {}
    } else {
      stryCov_9fa48("220");
      // Clear any existing timer first
      if (stryMutAct_9fa48("222") ? false : stryMutAct_9fa48("221") ? true : (stryCov_9fa48("221", "222"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("223")) {
          {}
        } else {
          stryCov_9fa48("223");
          clearInterval(this.cleanupTimer);
          this.cleanupTimer = undefined;
        }
      }
      this.cleanupTimer = setInterval(async () => {
        if (stryMutAct_9fa48("224")) {
          {}
        } else {
          stryCov_9fa48("224");
          const cutoffTime = new Date(stryMutAct_9fa48("225") ? Date.now() + this.config.autoCleanOlderThanMs : (stryCov_9fa48("225"), Date.now() - this.config.autoCleanOlderThanMs));
          await this.clearMetrics(cutoffTime);
        }
      }, this.config.cleanupIntervalMs);
    }
  }

  /**
   * Simple lock mechanism for concurrent access
   *
   * @param fn Function to execute with lock
   */
  private async withLock<T>(fn: () => T | Promise<T>): Promise<T> {
    if (stryMutAct_9fa48("226")) {
      {}
    } else {
      stryCov_9fa48("226");
      // Wait for lock to be available
      while (stryMutAct_9fa48("227") ? false : (stryCov_9fa48("227"), this.metricsLock)) {
        if (stryMutAct_9fa48("228")) {
          {}
        } else {
          stryCov_9fa48("228");
          await new Promise(stryMutAct_9fa48("229") ? () => undefined : (stryCov_9fa48("229"), resolve => setTimeout(resolve, 1)));
        }
      }
      this.metricsLock = stryMutAct_9fa48("230") ? false : (stryCov_9fa48("230"), true);
      try {
        if (stryMutAct_9fa48("231")) {
          {}
        } else {
          stryCov_9fa48("231");
          return await fn();
        }
      } finally {
        if (stryMutAct_9fa48("232")) {
          {}
        } else {
          stryCov_9fa48("232");
          this.metricsLock = stryMutAct_9fa48("233") ? true : (stryCov_9fa48("233"), false);
        }
      }
    }
  }
}