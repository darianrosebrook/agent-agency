/**
 * Performance Tracking Types and Contracts
 *
 * @author @darianrosebrook
 * @module performance-tracking-types
 *
 * Comprehensive type definitions for performance metric collection,
 * benchmark data aggregation, and RL training data pipelines.
 */

import { Timestamp } from "./agent-registry";

/**
 * Core performance event types for different tracking scenarios.
 */
export enum PerformanceEventType {
  _TASK_EXECUTION_START = "task_execution_start",
  _TASK_EXECUTION_COMPLETE = "task_execution_complete",
  _ROUTING_DECISION = "routing_decision",
  _AGENT_SELECTION = "agent_selection",
  _AGENT_REGISTRATION = "agent_registration",
  _AGENT_STATUS_CHANGE = "agent_status_change",
  _CONSTITUTIONAL_VALIDATION = "constitutional_validation",
  _EVALUATION_OUTCOME = "evaluation_outcome",
  _ANOMALY_DETECTED = "anomaly_detected",
  _SYSTEM_LOAD_SPIKE = "system_load_spike",
}

/**
 * Performance metric categories for comprehensive tracking.
 */
export enum MetricCategory {
  _LATENCY = "latency",
  _THROUGHPUT = "throughput",
  _ACCURACY = "accuracy",
  _RESOURCE_UTILIZATION = "resource_utilization",
  _CONSTITUTIONAL_COMPLIANCE = "constitutional_compliance",
  _COST_EFFICIENCY = "cost_efficiency",
  _RELIABILITY = "reliability",
}

/**
 * Agent performance profile with multi-dimensional scoring.
 */
export interface AgentPerformanceProfile {
  /**
   * Agent identifier.
   */
  agentId: string;

  /**
   * Task type this profile applies to.
   */
  taskType: string;

  /**
   * Performance metrics across different dimensions.
   */
  metrics: PerformanceMetrics;

  /**
   * Sample size (number of tasks evaluated).
   */
  sampleSize: number;

  /**
   * Confidence interval for metrics.
   */
  confidence: number;

  /**
   * Last updated timestamp.
   */
  lastUpdated: Timestamp;

  /**
   * Performance trend over time.
   */
  trend: PerformanceTrend;
}

/**
 * Comprehensive performance metrics across multiple dimensions.
 */
export interface PerformanceMetrics {
  /**
   * Response time metrics.
   */
  latency: LatencyMetrics;

  /**
   * Task success and quality metrics.
   */
  accuracy: AccuracyMetrics;

  /**
   * Resource consumption metrics.
   */
  resources: ResourceMetrics;

  /**
   * Constitutional compliance metrics.
   */
  compliance: ComplianceMetrics;

  /**
   * Cost efficiency metrics.
   */
  cost: CostMetrics;

  /**
   * Reliability metrics.
   */
  reliability: ReliabilityMetrics;

  /**
   * Agent-specific metrics (optional, used for agent registration/status events).
   */
  baselineLatencyMs?: number;
  baselineAccuracy?: number;
  baselineCostPerTask?: number;
  baselineReliability?: number;
  status?: string;
  previousStatus?: string;
  reason?: string;
  capabilities?: string[];
}

/**
 * Latency performance metrics.
 */
export interface LatencyMetrics {
  /**
   * Average response time in milliseconds.
   */
  averageMs: number;

  /**
   * 95th percentile response time.
   */
  p95Ms: number;

  /**
   * 99th percentile response time.
   */
  p99Ms: number;

  /**
   * Minimum response time.
   */
  minMs: number;

  /**
   * Maximum response time.
   */
  maxMs: number;
}

/**
 * Accuracy and quality metrics.
 */
export interface AccuracyMetrics {
  /**
   * Task completion success rate (0-1).
   */
  successRate: number;

  /**
   * Average quality score from evaluations (0-1).
   */
  qualityScore: number;

  /**
   * Rate of constitutional violations detected (0-1).
   */
  violationRate: number;

  /**
   * Average evaluation score across all rubrics.
   */
  evaluationScore: number;
}

/**
 * Resource utilization metrics.
 */
export interface ResourceMetrics {
  /**
   * Average CPU utilization percentage.
   */
  cpuUtilizationPercent: number;

  /**
   * Average memory utilization percentage.
   */
  memoryUtilizationPercent: number;

  /**
   * Average network I/O in KB/s.
   */
  networkIoKbps: number;

  /**
   * Average disk I/O in KB/s.
   */
  diskIoKbps: number;
}

/**
 * Constitutional compliance metrics.
 */
export interface ComplianceMetrics {
  /**
   * Rate of passing constitutional validations (0-1).
   */
  validationPassRate: number;

  /**
   * Average severity score of violations detected.
   */
  violationSeverityScore: number;

  /**
   * Overall compliance score (0-1).
   */
  complianceScore?: number;

  /**
   * Rate of CAWS clause citations in responses (0-1).
   */
  clauseCitationRate: number;
}

/**
 * Cost efficiency metrics.
 */
export interface CostMetrics {
  /**
   * Cost per task in processing units.
   */
  costPerTask: number;

  /**
   * Efficiency score (output quality / resource cost).
   */
  efficiencyScore: number;

  /**
   * Resource waste percentage.
   */
  resourceWastePercent: number;
}

/**
 * Reliability metrics.
 */
export interface ReliabilityMetrics {
  /**
   * Mean time between failures in hours.
   */
  mtbfHours: number;

  /**
   * Service availability percentage.
   */
  availabilityPercent: number;

  /**
   * Error rate percentage.
   */
  errorRatePercent: number;

  /**
   * Recovery time from failures in minutes.
   */
  recoveryTimeMinutes: number;
}

/**
 * Performance trend analysis.
 */
export interface PerformanceTrend {
  /**
   * Trend direction.
   */
  direction: "improving" | "declining" | "stable";

  /**
   * Trend magnitude (-1 to 1, where 1 is strongly improving).
   */
  magnitude: number;

  /**
   * Confidence in trend analysis.
   */
  confidence: number;

  /**
   * Time window for trend analysis in hours.
   */
  timeWindowHours: number;
}

/**
 * Individual performance event for tracking.
 */
export interface PerformanceEvent {
  /**
   * Unique event identifier.
   */
  id: string;

  /**
   * Event type.
   */
  type: PerformanceEventType;

  /**
   * Timestamp of event occurrence.
   */
  timestamp: Timestamp;

  /**
   * Whether the operation was successful.
   */
  success?: boolean;

  /**
   * Additional metadata for the event.
   */
  metadata?: Record<string, unknown>;

  /**
   * Agent identifier (if applicable).
   */
  agentId?: string;

  /**
   * Task identifier (if applicable).
   */
  taskId?: string;

  /**
   * Performance metrics captured in this event.
   */
  metrics: Partial<PerformanceMetrics>;

  /**
   * Additional context data.
   */
  context?: Record<string, unknown>;

  /**
   * Model version used for this event (for A/B testing).
   */
  modelVersion?: string;

  /**
   * Data integrity hash.
   */
  integrityHash: string;
}

/**
 * Benchmark dataset for model evaluation.
 */
export interface BenchmarkDataset {
  /**
   * Dataset identifier.
   */
  id: string;

  /**
   * Dataset name.
   */
  name: string;

  /**
   * Task type this dataset evaluates.
   */
  taskType: string;

  /**
   * Number of test cases in the dataset.
   */
  testCaseCount: number;

  /**
   * Dataset creation timestamp.
   */
  createdAt: Timestamp;

  /**
   * Dataset version.
   */
  version: string;

  /**
   * Expected performance baseline.
   */
  baselineMetrics: PerformanceMetrics;
}

/**
 * Benchmark evaluation result.
 */
export interface BenchmarkResult {
  /**
   * Benchmark run identifier.
   */
  id: string;

  /**
   * Agent being evaluated.
   */
  agentId: string;

  /**
   * Dataset used for evaluation.
   */
  datasetId: string;

  /**
   * Evaluation start timestamp.
   */
  startedAt: Timestamp;

  /**
   * Evaluation completion timestamp.
   */
  completedAt: Timestamp;

  /**
   * Overall performance score.
   */
  overallScore: number;

  /**
   * Detailed performance metrics.
   */
  metrics: PerformanceMetrics;

  /**
   * Individual test case results.
   */
  testCaseResults: BenchmarkTestCaseResult[];

  /**
   * Performance comparison to baseline.
   */
  baselineComparison: BaselineComparison;
}

/**
 * Individual test case result within a benchmark.
 */
export interface BenchmarkTestCaseResult {
  /**
   * Test case identifier.
   */
  testCaseId: string;

  /**
   * Whether the test case passed.
   */
  passed: boolean;

  /**
   * Performance score for this test case.
   */
  score: number;

  /**
   * Response time in milliseconds.
   */
  latencyMs: number;

  /**
   * Quality evaluation score.
   */
  qualityScore: number;

  /**
   * Error message (if failed).
   */
  error?: string;
}

/**
 * Comparison of results against baseline performance.
 */
export interface BaselineComparison {
  /**
   * Overall improvement over baseline (-1 to 1).
   */
  improvement: number;

  /**
   * Statistical significance of improvement.
   */
  significance: number;

  /**
   * Key metrics where improvement was observed.
   */
  improvedMetrics: MetricCategory[];

  /**
   * Key metrics where regression was observed.
   */
  regressedMetrics: MetricCategory[];
}

/**
 * RL training data sample.
 */
export interface RLTrainingSample {
  /**
   * Sample identifier.
   */
  id: string;

  /**
   * Agent that generated this sample.
   */
  agentId: string;

  /**
   * Task type context.
   */
  taskType: string;

  /**
   * State representation (anonymized).
   */
  state: Record<string, unknown>;

  /**
   * Action taken.
   */
  action: Record<string, unknown>;

  /**
   * Reward received.
   */
  reward: number;

  /**
   * Next state (anonymized).
   */
  nextState: Record<string, unknown>;

  /**
   * Whether this was a terminal state.
   */
  done: boolean;

  /**
   * Sample generation timestamp.
   */
  timestamp: Timestamp;

  /**
   * Data integrity hash.
   */
  integrityHash: string;
}

/**
 * Batch of RL training data ready for model training.
 */
export interface RLTrainingBatch {
  /**
   * Batch identifier.
   */
  id: string;

  /**
   * Agent this batch is for.
   */
  agentId: string;

  /**
   * Training samples in this batch.
   */
  samples: RLTrainingSample[];

  /**
   * Batch creation timestamp.
   */
  createdAt: Timestamp;

  /**
   * Data quality score for this batch.
   */
  qualityScore: number;

  /**
   * Anonymization level applied.
   */
  anonymizationLevel: "basic" | "differential" | "secure";
}

/**
 * Performance anomaly detection result.
 */
export interface PerformanceAnomaly {
  /**
   * Anomaly identifier.
   */
  id: string;

  /**
   * Type of anomaly detected.
   */
  type:
    | "latency_spike"
    | "accuracy_drop"
    | "resource_saturation"
    | "error_rate_increase";

  /**
   * Severity level.
   */
  severity: "low" | "medium" | "high" | "critical";

  /**
   * Affected agent (if applicable).
   */
  agentId?: string;

  /**
   * Affected task type (if applicable).
   */
  taskType?: string;

  /**
   * Anomaly detection timestamp.
   */
  detectedAt: Timestamp;

  /**
   * Anomaly description.
   */
  description: string;

  /**
   * Impact assessment.
   */
  impact: {
    affectedTasksPerHour: number;
    performanceDegradationPercent: number;
    estimatedRecoveryTimeMinutes: number;
  };

  /**
   * Recommended actions.
   */
  recommendations: string[];
}

/**
 * Data collection configuration.
 */
export interface DataCollectionConfig {
  /**
   * Whether data collection is enabled.
   */
  enabled: boolean;

  /**
   * Sampling rate (0-1, where 1 = collect all events).
   */
  samplingRate: number;

  /**
   * Maximum events to keep in memory buffer.
   */
  maxBufferSize: number;

  /**
   * Batch size for data processing.
   */
  batchSize: number;

  /**
   * Data retention period in days.
   */
  retentionDays: number;

  /**
   * Anonymization settings.
   */
  anonymization: {
    enabled: boolean;
    level: "basic" | "differential" | "secure";
    preserveAgentIds: boolean;
    preserveTaskTypes: boolean;
  };
}

/**
 * Performance analysis configuration.
 */
/**
 * Aggregation configuration for metric processing.
 */
export interface AggregationConfig {
  /**
   * Time windows for different aggregation levels.
   */
  windows: {
    realtime: AggregationWindow;
    short: AggregationWindow;
    medium: AggregationWindow;
    long: AggregationWindow;
  };

  /**
   * Statistical thresholds for outlier detection.
   */
  outlierThresholds: {
    zScoreThreshold: number;
    iqrMultiplier: number;
  };

  /**
   * Trend analysis configuration.
   */
  trendAnalysis: {
    minDataPoints: number;
    confidenceThreshold: number;
  };

  /**
   * Anonymization settings for aggregated data.
   */
  anonymization: {
    enabled: boolean;
    noiseLevel: number;
    preserveAgentIds: boolean;
  };
}

/**
 * Aggregation time window configuration.
 */
export interface AggregationWindow {
  /**
   * Window duration in milliseconds.
   */
  durationMs: number;

  /**
   * Window slide interval in milliseconds.
   */
  slideMs: number;

  /**
   * Minimum sample size required for aggregation.
   */
  minSampleSize: number;
}

/**
 * Performance analysis configuration.
 */
export interface AnalysisConfig {
  /**
   * Anomaly detection thresholds.
   */
  anomalyThresholds: {
    latencySpikeMultiplier: number;
    accuracyDropPercent: number;
    errorRateIncreasePercent: number;
    resourceSaturationPercent: number;
  };

  /**
   * Trend analysis configuration.
   */
  trendAnalysis: {
    minDataPoints: number;
    confidenceThreshold: number;
  };

  /**
   * Alert thresholds.
   */
  alertThresholds: {
    criticalLatencyMs: number;
    criticalErrorRatePercent: number;
    criticalAccuracyDropPercent: number;
  };
}

/**
 * Trend analysis result.
 */
export interface TrendAnalysisResult {
  agentId: string;
  overallTrend: PerformanceTrend;
  metricTrends: Record<MetricCategory, PerformanceTrend>;
  confidence: number;
  analysisTimeRange: {
    start: Timestamp;
    end: Timestamp;
  };
}

/**
 * RL data pipeline configuration.
 */
export interface RLDataPipelineConfig {
  /**
   * Data quality thresholds.
   */
  qualityThresholds: {
    minSampleDiversity: number;
    maxTemporalGapMinutes: number;
    minRewardVariance: number;
    maxDuplicateRatio: number;
  };

  /**
   * Batch configuration.
   */
  batching: {
    maxBatchSize: number;
    maxBatchAgeMinutes: number;
    minBatchSize: number;
  };

  /**
   * Training data retention and cleanup.
   */
  retention: {
    maxSamplesInMemory: number;
    maxBatchesInMemory: number;
    cleanupIntervalMinutes: number;
  };

  /**
   * State representation configuration.
   */
  stateRepresentation: {
    includeHistoricalMetrics: boolean;
    includeAgentLoad: boolean;
    includeTaskContext: boolean;
    temporalWindowSize: number;
  };

  /**
   * Reward function configuration.
   */
  rewardFunction: {
    latencyWeight: number;
    accuracyWeight: number;
    costWeight: number;
    complianceWeight: number;
    temporalDecayFactor: number;
  };
}
