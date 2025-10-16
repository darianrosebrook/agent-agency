/**
 * @fileoverview Type definitions for Runtime Optimization Engine (INFRA-003)
 *
 * Defines types for performance monitoring, bottleneck detection, and
 * optimization recommendations.
 *
 * @author @darianrosebrook
 */

/**
 * Performance metric types
 */
export enum MetricType {
  CPU = "cpu",
  MEMORY = "memory",
  NETWORK = "network",
  LATENCY = "latency",
  THROUGHPUT = "throughput",
  ERROR_RATE = "error_rate",
  CACHE_HIT_RATE = "cache_hit_rate",
}

/**
 * Performance metric data point
 */
export interface PerformanceMetric {
  /** Metric type */
  type: MetricType;

  /** Metric value */
  value: number;

  /** Unit of measurement (ms, %, bytes, etc.) */
  unit: string;

  /** Timestamp of measurement */
  timestamp: Date;

  /** Source component or operation */
  source: string;

  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Severity levels for bottlenecks
 */
export enum BottleneckSeverity {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

/**
 * Bottleneck detection result
 */
export interface Bottleneck {
  /** Unique identifier */
  id: string;

  /** Component or operation causing bottleneck */
  component: string;

  /** Severity level */
  severity: BottleneckSeverity;

  /** Metric type affected */
  metricType: MetricType;

  /** Current value */
  currentValue: number;

  /** Threshold value */
  threshold: number;

  /** Impact description */
  impact: string;

  /** First detected timestamp */
  detectedAt: Date;

  /** Last observed timestamp */
  lastObservedAt: Date;

  /** Frequency of occurrence */
  occurrenceCount: number;
}

/**
 * Optimization recommendation types
 */
export enum RecommendationType {
  CACHE_OPTIMIZATION = "cache_optimization",
  RESOURCE_ALLOCATION = "resource_allocation",
  QUERY_OPTIMIZATION = "query_optimization",
  CONCURRENCY_TUNING = "concurrency_tuning",
  MEMORY_MANAGEMENT = "memory_management",
  NETWORK_OPTIMIZATION = "network_optimization",
}

/**
 * Optimization recommendation
 */
export interface OptimizationRecommendation {
  /** Unique identifier */
  id: string;

  /** Recommendation type */
  type: RecommendationType;

  /** Priority level */
  priority: "low" | "medium" | "high";

  /** Target component */
  component: string;

  /** Recommendation title */
  title: string;

  /** Detailed description */
  description: string;

  /** Expected impact description */
  expectedImpact: string;

  /** Estimated improvement percentage */
  estimatedImprovementPct?: number;

  /** Implementation difficulty */
  implementationDifficulty: "easy" | "moderate" | "hard";

  /** Related bottleneck ID */
  relatedBottleneckId?: string;

  /** Generated timestamp */
  generatedAt: Date;
}

/**
 * Cache performance statistics
 */
export interface CacheStatistics {
  /** Cache name or identifier */
  cacheId: string;

  /** Total requests */
  totalRequests: number;

  /** Cache hits */
  hits: number;

  /** Cache misses */
  misses: number;

  /** Hit rate percentage */
  hitRate: number;

  /** Average response time for hits (ms) */
  avgHitTimeMs: number;

  /** Average response time for misses (ms) */
  avgMissTimeMs: number;

  /** Cache size (bytes) */
  cacheSizeBytes: number;

  /** Eviction count */
  evictionCount: number;

  /** Time window for statistics */
  windowStartTime: Date;

  /** Window end time */
  windowEndTime: Date;
}

/**
 * Performance trend data
 */
export interface PerformanceTrend {
  /** Metric type */
  metricType: MetricType;

  /** Component being tracked */
  component: string;

  /** Trend direction */
  direction: "improving" | "stable" | "degrading";

  /** Average value over period */
  averageValue: number;

  /** Minimum value */
  minValue: number;

  /** Maximum value */
  maxValue: number;

  /** Standard deviation */
  standardDeviation: number;

  /** Trend start time */
  startTime: Date;

  /** Trend end time */
  endTime: Date;

  /** Data points used for trend */
  dataPointCount: number;
}

/**
 * Optimization engine configuration
 */
export interface OptimizationEngineConfig {
  /** Enable/disable the optimization engine */
  enabled: boolean;

  /** Metric collection interval (ms) */
  collectionIntervalMs: number;

  /** Analysis window duration (ms) */
  analysisWindowMs: number;

  /** Maximum overhead percentage allowed */
  maxOverheadPct: number;

  /** Bottleneck detection thresholds */
  thresholds: {
    [_key in MetricType]?: number;
  };

  /** Enable cache optimization */
  enableCacheOptimization: boolean;

  /** Enable trend analysis */
  enableTrendAnalysis: boolean;

  /** Minimum data points for trend analysis */
  minDataPointsForTrend: number;
}

/**
 * Optimization analysis result
 */
export interface OptimizationAnalysis {
  /** Analysis timestamp */
  timestamp: Date;

  /** Analysis window */
  windowMs: number;

  /** Detected bottlenecks */
  bottlenecks: Bottleneck[];

  /** Generated recommendations */
  recommendations: OptimizationRecommendation[];

  /** Performance trends */
  trends: PerformanceTrend[];

  /** Cache statistics */
  cacheStats: CacheStatistics[];

  /** Overall system health score (0-100) */
  healthScore: number;

  /** Analysis duration (ms) */
  analysisDurationMs: number;
}

/**
 * Performance monitor interface
 */
export interface IPerformanceMonitor {
  /**
   * Record a performance metric
   */
  recordMetric(_metric: PerformanceMetric): Promise<void>;

  /**
   * Get metrics for a time window
   */
  getMetrics(
    _startTime: Date,
    _endTime: Date,
    _metricType?: MetricType
  ): Promise<PerformanceMetric[]>;

  /**
   * Get latest metrics
   */
  getLatestMetrics(
    _count: number,
    _metricType?: MetricType
  ): Promise<PerformanceMetric[]>;

  /**
   * Clear old metrics
   */
  clearMetrics(_olderThan: Date): Promise<void>;
}

/**
 * Bottleneck detector interface
 */
export interface IBottleneckDetector {
  /**
   * Detect bottlenecks from metrics
   */
  detectBottlenecks(_metrics: PerformanceMetric[]): Promise<Bottleneck[]>;

  /**
   * Update bottleneck thresholds
   */
  updateThresholds(_thresholds: Partial<Record<MetricType, number>>): void;

  /**
   * Get active bottlenecks
   */
  getActiveBottlenecks(): Bottleneck[];

  /**
   * Clear resolved bottlenecks
   */
  clearResolvedBottlenecks(_olderThan: Date): Promise<void>;
}

/**
 * Runtime optimizer interface
 */
export interface IRuntimeOptimizer {
  /**
   * Initialize the optimizer
   */
  initialize(): Promise<void>;

  /**
   * Start optimization monitoring
   */
  start(): Promise<void>;

  /**
   * Stop optimization monitoring
   */
  stop(): Promise<void>;

  /**
   * Perform analysis and generate recommendations
   */
  analyze(): Promise<OptimizationAnalysis>;

  /**
   * Get cache statistics
   */
  getCacheStatistics(): Promise<CacheStatistics[]>;

  /**
   * Get performance trends
   */
  getPerformanceTrends(): Promise<PerformanceTrend[]>;

  /**
   * Get current configuration
   */
  getConfig(): OptimizationEngineConfig;

  /**
   * Update configuration
   */
  updateConfig(_config: Partial<OptimizationEngineConfig>): void;

  /**
   * Get health status
   */
  getHealthStatus(): {
    isRunning: boolean;
    lastAnalysisTime?: Date;
    metricsCollected: number;
    bottlenecksDetected: number;
    recommendationsGenerated: number;
  };
}
