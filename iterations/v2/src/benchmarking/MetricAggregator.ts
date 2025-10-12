/**
 * Metric Aggregator for Benchmark Data Aggregation and Anonymization
 *
 * @author @darianrosebrook
 * @module metric-aggregator
 *
 * Aggregates performance metrics into comprehensive benchmark datasets
 * with statistical analysis and privacy-preserving anonymization.
 */

import { EventEmitter } from "events";
import { Timestamp } from "../types/agent-registry";
import {
  AccuracyMetrics,
  AgentPerformanceProfile,
  ComplianceMetrics,
  CostMetrics,
  LatencyMetrics,
  PerformanceEvent,
  PerformanceMetrics,
  PerformanceTrend,
  ReliabilityMetrics,
  ResourceMetrics,
} from "../types/performance-tracking";

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
 * Aggregation configuration.
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
 * Default aggregation configuration.
 */
const DEFAULT_CONFIG: AggregationConfig = {
  windows: {
    realtime: {
      durationMs: 5 * 60 * 1000,
      slideMs: 60 * 1000,
      minSampleSize: 10,
    }, // 5 min window, 1 min slide
    short: {
      durationMs: 60 * 60 * 1000,
      slideMs: 15 * 60 * 1000,
      minSampleSize: 50,
    }, // 1 hour window, 15 min slide
    medium: {
      durationMs: 24 * 60 * 60 * 1000,
      slideMs: 4 * 60 * 60 * 1000,
      minSampleSize: 100,
    }, // 24 hour window, 4 hour slide
    long: {
      durationMs: 7 * 24 * 60 * 60 * 1000,
      slideMs: 24 * 60 * 60 * 1000,
      minSampleSize: 500,
    }, // 7 day window, 1 day slide
  },
  outlierThresholds: {
    zScoreThreshold: 3.0,
    iqrMultiplier: 1.5,
  },
  trendAnalysis: {
    minDataPoints: 20,
    confidenceThreshold: 0.8,
  },
  anonymization: {
    enabled: true,
    noiseLevel: 0.1,
    preserveAgentIds: true,
  },
};

/**
 * Aggregated performance data for a time window.
 */
interface AggregatedData {
  agentId: string;
  taskType: string;
  window: keyof AggregationConfig["windows"];
  startTime: Timestamp;
  endTime: Timestamp;
  sampleCount: number;
  metrics: PerformanceMetrics;
  trend: PerformanceTrend;
  outliers: PerformanceEvent[];
  confidence: number;
}

/**
 * Metric Aggregator for performance data aggregation and analysis.
 *
 * This component processes raw performance events into aggregated benchmark
 * data with statistical analysis, trend detection, and privacy preservation.
 */
export class MetricAggregator extends EventEmitter {
  private config: AggregationConfig;
  private aggregatedData: Map<string, AggregatedData[]> = new Map();
  private eventBuffer: PerformanceEvent[] = [];
  private isAggregating = false;
  private lastAggregationTime = 0;

  /**
   * Creates a new Metric Aggregator instance.
   *
   * @param config - Aggregation configuration. Uses defaults if not provided.
   */
  constructor(config: Partial<AggregationConfig> = {}) {
    super();
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.setupEventHandlers();
  }

  /**
   * Starts metric aggregation.
   */
  startAggregation(): void {
    this.isAggregating = true;
    this.scheduleAggregation();
    this.emit("aggregation_started");
  }

  /**
   * Stops metric aggregation.
   */
  stopAggregation(): void {
    this.isAggregating = false;
    this.emit("aggregation_stopped");
  }

  /**
   * Adds performance events for aggregation.
   *
   * @param events - Performance events to process
   */
  addEvents(events: PerformanceEvent[]): void {
    if (!this.isAggregating) {
      return;
    }

    // Filter out outliers before adding to buffer
    const filteredEvents = events.filter((event) => !this.isOutlier(event));
    this.eventBuffer.push(...filteredEvents);

    // Emit event for new data availability
    this.emit("events_added", filteredEvents.length);
  }

  /**
   * Gets aggregated performance profiles for an agent.
   *
   * @param agentId - Agent identifier
   * @param taskType - Task type filter (optional)
   * @returns Array of performance profiles
   */
  getPerformanceProfiles(
    agentId: string,
    taskType?: string
  ): AgentPerformanceProfile[] {
    const agentData = this.aggregatedData.get(agentId) || [];

    return agentData
      .filter((data) => !taskType || data.taskType === taskType)
      .map((data) => this.convertToProfile(data));
  }

  /**
   * Gets benchmark-ready aggregated metrics for a time range.
   *
   * @param startTime - Start timestamp
   * @param endTime - End timestamp
   * @param agentId - Agent filter (optional)
   * @param taskType - Task type filter (optional)
   * @returns Array of aggregated benchmark data
   */
  getBenchmarkData(
    startTime: Timestamp,
    endTime: Timestamp,
    agentId?: string,
    taskType?: string
  ): AggregatedData[] {
    const allData: AggregatedData[] = [];

    for (const [agent, data] of this.aggregatedData) {
      if (agentId && agent !== agentId) continue;

      const filteredData = data.filter((item) => {
        if (taskType && item.taskType !== taskType) return false;

        const itemStart = new Date(item.startTime).getTime();
        const itemEnd = new Date(item.endTime).getTime();
        const queryStart = new Date(startTime).getTime();
        const queryEnd = new Date(endTime).getTime();

        return itemStart <= queryEnd && itemEnd >= queryStart;
      });

      allData.push(...filteredData);
    }

    return allData.sort(
      (a, b) => new Date(b.endTime).getTime() - new Date(a.endTime).getTime()
    );
  }

  /**
   * Performs real-time aggregation for all configured windows.
   */
  async performAggregation(): Promise<void> {
    if (!this.isAggregating || this.eventBuffer.length === 0) {
      return;
    }

    const startTime = performance.now();

    try {
      // Group events by agent and task type
      const groupedEvents = this.groupEventsByAgentAndTask();

      // Aggregate for each window size
      for (const windowType of Object.keys(this.config.windows) as Array<
        keyof AggregationConfig["windows"]
      >) {
        await this.aggregateForWindow(groupedEvents, windowType);
      }

      // Clean up old data
      this.cleanupOldData();

      const duration = performance.now() - startTime;
      this.lastAggregationTime = duration;

      this.emit("aggregation_completed", {
        durationMs: duration,
        eventsProcessed: this.eventBuffer.length,
        aggregationsCreated: this.countTotalAggregations(),
      });

      // Clear processed events
      this.eventBuffer = [];
    } catch (error) {
      this.emit("aggregation_error", error);
    }
  }

  /**
   * Gets aggregation statistics and health metrics.
   */
  getStats() {
    const totalAggregations = this.countTotalAggregations();
    const oldestData = this.getOldestDataTimestamp();
    const newestData = this.getNewestDataTimestamp();
    const memUsage = process.memoryUsage();

    return {
      isAggregating: this.isAggregating,
      bufferSize: this.eventBuffer.length,
      totalAggregations,
      dataTimeRange: {
        oldest: oldestData,
        newest: newestData,
      },
      lastAggregationTimeMs: this.lastAggregationTime,
      memoryUsageMB: memUsage.heapUsed / 1024 / 1024,
      processingTimeMs: this.lastAggregationTime,
      config: this.config,
    };
  }

  /**
   * Updates aggregation configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<AggregationConfig>): void {
    this.config = { ...this.config, ...config };
    this.emit("config_updated", this.config);
  }

  /**
   * Clears all aggregated data.
   */
  clearData(): void {
    this.aggregatedData.clear();
    this.eventBuffer = [];
    this.emit("data_cleared");
  }

  /**
   * Groups events by agent and task type for aggregation.
   */
  private groupEventsByAgentAndTask(): Map<
    string,
    Map<string, PerformanceEvent[]>
  > {
    const grouped = new Map<string, Map<string, PerformanceEvent[]>>();

    for (const event of this.eventBuffer) {
      const agentId = event.agentId || "unknown";
      const taskType = this.extractTaskType(event);

      if (!grouped.has(agentId)) {
        grouped.set(agentId, new Map());
      }

      const agentEvents = grouped.get(agentId)!;
      if (!agentEvents.has(taskType)) {
        agentEvents.set(taskType, []);
      }

      agentEvents.get(taskType)!.push(event);
    }

    return grouped;
  }

  /**
   * Aggregates data for a specific time window.
   */
  private async aggregateForWindow(
    groupedEvents: Map<string, Map<string, PerformanceEvent[]>>,
    windowType: keyof AggregationConfig["windows"]
  ): Promise<void> {
    const window = this.config.windows[windowType];
    const now = new Date();
    const windowStart = new Date(now.getTime() - window.durationMs);

    for (const [agentId, taskEvents] of groupedEvents) {
      for (const [taskType, events] of taskEvents) {
        // Filter events to current window
        const windowEvents = events.filter((event) => {
          const eventTime = new Date(event.timestamp);
          return eventTime >= windowStart && eventTime <= now;
        });

        if (windowEvents.length < window.minSampleSize) {
          continue; // Not enough data for reliable aggregation
        }

        const aggregatedData = await this.createAggregatedData(
          agentId,
          taskType,
          windowEvents,
          windowType,
          windowStart.toISOString(),
          now.toISOString()
        );

        // Store aggregated data
        if (!this.aggregatedData.has(agentId)) {
          this.aggregatedData.set(agentId, []);
        }

        const agentData = this.aggregatedData.get(agentId)!;
        agentData.push(aggregatedData);

        // Keep only recent aggregations (limit memory usage)
        if (agentData.length > 100) {
          agentData.sort(
            (a, b) =>
              new Date(b.endTime).getTime() - new Date(a.endTime).getTime()
          );
          agentData.splice(100);
        }
      }
    }
  }

  /**
   * Creates aggregated data from a set of events.
   */
  private async createAggregatedData(
    agentId: string,
    taskType: string,
    events: PerformanceEvent[],
    window: keyof AggregationConfig["windows"],
    startTime: Timestamp,
    endTime: Timestamp
  ): Promise<AggregatedData> {
    const metrics = this.aggregateMetrics(events);
    const trend = this.calculateTrend(events);
    const outliers = this.detectOutliers(events);
    const confidence = this.calculateConfidence(events.length, window);

    return {
      agentId,
      taskType,
      window,
      startTime,
      endTime,
      sampleCount: events.length,
      metrics,
      trend,
      outliers,
      confidence,
    };
  }

  /**
   * Aggregates performance metrics from multiple events.
   */
  private aggregateMetrics(events: PerformanceEvent[]): PerformanceMetrics {
    const latencyValues = events
      .map((e) => e.metrics.latency)
      .filter((l) => l && l.averageMs > 0)
      .map((l) => l!.averageMs);

    const accuracyValues = events
      .map((e) => e.metrics.accuracy)
      .filter((a) => a && a.successRate >= 0)
      .map((a) => a!.successRate);

    const resourceValues = events
      .map((e) => e.metrics.resources)
      .filter((r) => r && r.cpuUtilizationPercent >= 0);

    const complianceValues = events
      .map((e) => e.metrics.compliance)
      .filter((c) => c && c.validationPassRate >= 0);

    return {
      latency: this.aggregateLatencyMetrics(latencyValues),
      accuracy: this.aggregateAccuracyMetrics(accuracyValues),
      resources: this.aggregateResourceMetrics(resourceValues),
      compliance: this.aggregateComplianceMetrics(complianceValues),
      cost: this.aggregateCostMetrics(events),
      reliability: this.aggregateReliabilityMetrics(events),
    };
  }

  /**
   * Aggregates latency metrics with statistical analysis.
   */
  private aggregateLatencyMetrics(latencyValues: number[]): LatencyMetrics {
    if (latencyValues.length === 0) {
      return { averageMs: 0, p95Ms: 0, p99Ms: 0, minMs: 0, maxMs: 0 };
    }

    const sorted = latencyValues.sort((a, b) => a - b);
    const average =
      latencyValues.reduce((sum, val) => sum + val, 0) / latencyValues.length;
    const p95Index = Math.floor(sorted.length * 0.95);
    const p99Index = Math.floor(sorted.length * 0.99);

    return {
      averageMs: this.applyAnonymizationNoise(average),
      p95Ms: this.applyAnonymizationNoise(sorted[p95Index] || average),
      p99Ms: this.applyAnonymizationNoise(sorted[p99Index] || average),
      minMs: this.applyAnonymizationNoise(sorted[0]),
      maxMs: this.applyAnonymizationNoise(sorted[sorted.length - 1]),
    };
  }

  /**
   * Aggregates accuracy metrics.
   */
  private aggregateAccuracyMetrics(accuracyValues: number[]): AccuracyMetrics {
    if (accuracyValues.length === 0) {
      return {
        successRate: 0,
        qualityScore: 0,
        violationRate: 0,
        evaluationScore: 0,
      };
    }

    const averageSuccessRate =
      accuracyValues.reduce((sum, val) => sum + val, 0) / accuracyValues.length;

    return {
      successRate: this.applyAnonymizationNoise(averageSuccessRate),
      qualityScore: this.applyAnonymizationNoise(averageSuccessRate), // Simplified mapping
      violationRate: this.applyAnonymizationNoise(1 - averageSuccessRate),
      evaluationScore: this.applyAnonymizationNoise(averageSuccessRate),
    };
  }

  /**
   * Aggregates resource metrics.
   */
  private aggregateResourceMetrics(
    resourceEvents: (ResourceMetrics | undefined)[]
  ): ResourceMetrics {
    const validEvents = resourceEvents.filter(
      (r): r is ResourceMetrics => r !== undefined
    );
    if (validEvents.length === 0) {
      return {
        cpuUtilizationPercent: 0,
        memoryUtilizationPercent: 0,
        networkIoKbps: 0,
        diskIoKbps: 0,
      };
    }

    const cpuValues = validEvents.map((r) => r.cpuUtilizationPercent);
    const memoryValues = validEvents.map((r) => r.memoryUtilizationPercent);
    const networkValues = validEvents.map((r) => r.networkIoKbps);
    const diskValues = validEvents.map((r) => r.diskIoKbps);

    return {
      cpuUtilizationPercent: this.applyAnonymizationNoise(
        this.average(cpuValues)
      ),
      memoryUtilizationPercent: this.applyAnonymizationNoise(
        this.average(memoryValues)
      ),
      networkIoKbps: this.applyAnonymizationNoise(this.average(networkValues)),
      diskIoKbps: this.applyAnonymizationNoise(this.average(diskValues)),
    };
  }

  /**
   * Aggregates compliance metrics.
   */
  private aggregateComplianceMetrics(
    complianceEvents: (ComplianceMetrics | undefined)[]
  ): ComplianceMetrics {
    const validEvents = complianceEvents.filter(
      (c): c is ComplianceMetrics => c !== undefined
    );
    if (validEvents.length === 0) {
      return {
        validationPassRate: 0,
        violationSeverityScore: 0,
        clauseCitationRate: 0,
      };
    }

    const passRates = validEvents.map((c) => c.validationPassRate);
    const severityScores = validEvents.map((c) => c.violationSeverityScore);
    const citationRates = validEvents.map((c) => c.clauseCitationRate);

    return {
      validationPassRate: this.applyAnonymizationNoise(this.average(passRates)),
      violationSeverityScore: this.applyAnonymizationNoise(
        this.average(severityScores)
      ),
      clauseCitationRate: this.applyAnonymizationNoise(
        this.average(citationRates)
      ),
    };
  }

  /**
   * Aggregates cost metrics.
   */
  private aggregateCostMetrics(events: PerformanceEvent[]): CostMetrics {
    // Simplified cost calculation based on resource usage
    const resourceEvents = events.filter((e) => e.metrics.resources);
    if (resourceEvents.length === 0) {
      return { costPerTask: 0, efficiencyScore: 0, resourceWastePercent: 0 };
    }

    const avgCpu = this.average(
      resourceEvents.map((e) => e.metrics.resources!.cpuUtilizationPercent)
    );
    const avgMemory = this.average(
      resourceEvents.map((e) => e.metrics.resources!.memoryUtilizationPercent)
    );

    // Simplified cost model: cost increases with resource usage
    const costPerTask = (avgCpu + avgMemory) / 100; // Scale to 0-1
    const efficiencyScore = Math.max(0, 1 - costPerTask); // Higher efficiency = lower cost

    return {
      costPerTask: this.applyAnonymizationNoise(costPerTask),
      efficiencyScore: this.applyAnonymizationNoise(efficiencyScore),
      resourceWastePercent: this.applyAnonymizationNoise(costPerTask * 100),
    };
  }

  /**
   * Aggregates reliability metrics.
   */
  private aggregateReliabilityMetrics(
    events: PerformanceEvent[]
  ): ReliabilityMetrics {
    const taskEvents = events.filter(
      (e) => e.type === "task_execution_complete"
    );
    const successfulTasks = taskEvents.filter(
      (e) => e.metrics.accuracy?.successRate === 1
    );

    const successRate =
      taskEvents.length > 0 ? successfulTasks.length / taskEvents.length : 0;
    const availabilityPercent = successRate * 100;

    return {
      mtbfHours: 24 * successRate, // Simplified: higher success rate = higher MTBF
      availabilityPercent: this.applyAnonymizationNoise(availabilityPercent),
      errorRatePercent: this.applyAnonymizationNoise((1 - successRate) * 100),
      recoveryTimeMinutes: this.applyAnonymizationNoise(
        Math.max(0, 60 * (1 - successRate))
      ), // Simplified
    };
  }

  /**
   * Calculates overall performance trend across all metrics.
   */
  private calculateOverallTrend(
    metrics: AgentPerformanceProfile[]
  ): PerformanceTrend {
    // Simplified: weight different aspects of performance
    const weights = {
      accuracy: 0.4,
      latency: 0.3,
      compliance: 0.2,
      cost: 0.1,
    };

    const scores = metrics.map((m) => ({
      accuracy: m.metrics.accuracy.successRate,
      latency: 1 / (1 + m.metrics.latency.averageMs / 1000), // Normalize latency
      compliance: m.metrics.compliance.validationPassRate,
      cost: m.metrics.cost.efficiencyScore,
      timestamp: new Date(m.lastUpdated).getTime(),
    }));

    // Calculate weighted score for each data point
    const weightedScores = scores.map((s) => ({
      score:
        weights.accuracy * s.accuracy +
        weights.latency * s.latency +
        weights.compliance * s.compliance +
        weights.cost * s.cost,
      timestamp: s.timestamp,
    }));

    return this.calculateLinearTrend(weightedScores);
  }

  /**
   * Calculates linear trend from time series data.
   */
  private calculateLinearTrend(
    data: Array<{ score: number; timestamp: number }>
  ): PerformanceTrend {
    if (data.length < 2) {
      return {
        direction: "stable",
        magnitude: 0,
        confidence: 0.5,
        timeWindowHours: 1,
      };
    }

    const n = data.length;
    const timestamps = data.map((d) => d.timestamp);
    const scores = data.map((d) => d.score);

    // Normalize timestamps to hours from start
    const startTime = Math.min(...timestamps);
    const hours = timestamps.map((t) => (t - startTime) / (1000 * 60 * 60));

    // Calculate linear regression
    const sumX = hours.reduce((sum, x) => sum + x, 0);
    const sumY = scores.reduce((sum, score) => sum + score, 0);
    const sumXY = hours.reduce((sum, x, i) => sum + x * scores[i], 0);
    const sumXX = hours.reduce((sum, x) => sum + x * x, 0);

    const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;

    // Calculate R-squared for confidence
    const yMean = sumY / n;
    const ssRes = scores.reduce((sum, score, i) => {
      const predicted = slope * hours[i] + intercept;
      return sum + Math.pow(score - predicted, 2);
    }, 0);
    const ssTot = scores.reduce(
      (sum, score) => sum + Math.pow(score - yMean, 2),
      0
    );
    const rSquared = ssTot > 0 ? 1 - ssRes / ssTot : 0;

    const direction =
      slope > 0.001 ? "improving" : slope < -0.001 ? "declining" : "stable";
    const magnitude = Math.abs(slope);
    const confidence = Math.sqrt(Math.max(0, rSquared));
    const timeWindowHours =
      (Math.max(...timestamps) - startTime) / (1000 * 60 * 60);

    return {
      direction: direction as "improving" | "declining" | "stable",
      magnitude,
      confidence,
      timeWindowHours,
    };
  }

  /**
   * Calculates performance trend from historical data.
   */
  private calculateTrend(events: PerformanceEvent[]): PerformanceTrend {
    if (events.length < this.config.trendAnalysis.minDataPoints) {
      return {
        direction: "stable",
        magnitude: 0,
        confidence: 0.5,
        timeWindowHours: 1,
      };
    }

    // Sort events by time
    const sortedEvents = events.sort(
      (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    );

    // Extract performance scores over time
    const scores = sortedEvents.map((event) => {
      const accuracy = event.metrics.accuracy?.successRate || 0;
      const latency = event.metrics.latency?.averageMs || 0;
      // Simplified score: higher accuracy, lower latency = better performance
      return accuracy * (1 / (1 + latency / 1000)); // Normalize latency impact
    });

    // Calculate linear trend
    const n = scores.length;
    const sumX = (n * (n - 1)) / 2;
    const sumY = scores.reduce((sum, score) => sum + score, 0);
    const sumXY = scores.reduce((sum, score, index) => sum + score * index, 0);
    const sumXX = (n * (n - 1) * (2 * n - 1)) / 6;

    const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;

    // Determine direction and magnitude
    const direction =
      slope > 0.01 ? "improving" : slope < -0.01 ? "declining" : "stable";
    const magnitude = Math.abs(slope);

    // Calculate confidence using R-squared
    const yMean = sumY / n;
    const ssRes = scores.reduce((sum, score, index) => {
      const predicted = slope * index + intercept;
      return sum + Math.pow(score - predicted, 0);
    }, 0);
    const ssTot = scores.reduce(
      (sum, score) => sum + Math.pow(score - yMean, 2),
      0
    );
    const rSquared = ssTot > 0 ? 1 - ssRes / ssTot : 0;
    const confidence = Math.sqrt(Math.max(0, rSquared));

    const timeRange = sortedEvents[sortedEvents.length - 1].timestamp;
    const timeWindowHours =
      (new Date(timeRange).getTime() -
        new Date(sortedEvents[0].timestamp).getTime()) /
      (1000 * 60 * 60);

    return {
      direction: direction as "improving" | "declining" | "stable",
      magnitude,
      confidence,
      timeWindowHours,
    };
  }

  /**
   * Detects outlier events using statistical methods.
   */
  private detectOutliers(events: PerformanceEvent[]): PerformanceEvent[] {
    if (events.length < 10) return []; // Need minimum sample size

    // Extract performance scores
    const scores = events.map((event) => {
      const accuracy = event.metrics.accuracy?.successRate || 0;
      const latency = event.metrics.latency?.averageMs || 0;
      return accuracy * (1 / (1 + latency / 1000));
    });

    // Calculate mean and standard deviation
    const mean = this.average(scores);
    const stdDev = Math.sqrt(
      scores.reduce((sum, score) => sum + Math.pow(score - mean, 2), 0) /
        scores.length
    );

    // Detect outliers using Z-score
    const outliers: PerformanceEvent[] = [];
    for (let i = 0; i < events.length; i++) {
      const zScore = Math.abs((scores[i] - mean) / stdDev);
      if (zScore > this.config.outlierThresholds.zScoreThreshold) {
        outliers.push(events[i]);
      }
    }

    return outliers;
  }

  /**
   * Determines if an event is an outlier before aggregation.
   */
  private isOutlier(event: PerformanceEvent): boolean {
    // Simplified outlier detection for incoming events
    // In production, this would use more sophisticated methods
    const accuracy = event.metrics.accuracy?.successRate;
    const latency = event.metrics.latency?.averageMs;

    if (accuracy !== undefined && (accuracy < 0 || accuracy > 1)) return true;
    if (latency !== undefined && latency < 0) return true;

    return false;
  }

  /**
   * Calculates confidence score for aggregated data.
   */
  private calculateConfidence(
    sampleCount: number,
    window: keyof AggregationConfig["windows"]
  ): number {
    const windowConfig = this.config.windows[window];
    const sampleRatio = Math.min(1, sampleCount / windowConfig.minSampleSize);
    const timeFactor =
      window === "long"
        ? 1.0
        : window === "medium"
        ? 0.8
        : window === "short"
        ? 0.6
        : 0.4;

    return sampleRatio * timeFactor;
  }

  /**
   * Converts aggregated data to agent performance profile.
   */
  private convertToProfile(data: AggregatedData): AgentPerformanceProfile {
    return {
      agentId: data.agentId,
      taskType: data.taskType,
      metrics: data.metrics,
      sampleSize: data.sampleCount,
      confidence: data.confidence,
      lastUpdated: data.endTime,
      trend: data.trend,
    };
  }

  /**
   * Applies anonymization noise to numerical values.
   */
  private applyAnonymizationNoise(value: number): number {
    if (!this.config.anonymization.enabled) return value;

    // Add Laplace noise for differential privacy
    const noise =
      (Math.random() - 0.5) * 2 * this.config.anonymization.noiseLevel;
    return Math.max(0, value + noise);
  }

  /**
   * Extracts task type from performance event.
   */
  private extractTaskType(event: PerformanceEvent): string {
    // Try to extract from context or use default
    if (event.context?.taskType) return event.context.taskType as string;
    if (event.taskId) {
      // Extract task type from task ID pattern (e.g., "task-123" -> "task")
      const match = event.taskId.match(/^([^-]+)-/);
      if (match) return match[1];
    }
    return "unknown";
  }

  /**
   * Calculates average of numerical array.
   */
  private average(values: number[]): number {
    if (values.length === 0) return 0;
    return values.reduce((sum, val) => sum + val, 0) / values.length;
  }

  /**
   * Schedules periodic aggregation.
   */
  private scheduleAggregation(): void {
    if (!this.isAggregating) return;

    // Aggregate every minute
    setTimeout(async () => {
      await this.performAggregation();
      this.scheduleAggregation();
    }, 60 * 1000);
  }

  /**
   * Cleans up old aggregated data to manage memory.
   */
  private cleanupOldData(): void {
    const maxAge = 30 * 24 * 60 * 60 * 1000; // 30 days
    const cutoffTime = Date.now() - maxAge;

    for (const [agentId, data] of this.aggregatedData) {
      const filteredData = data.filter(
        (item) => new Date(item.endTime).getTime() > cutoffTime
      );
      this.aggregatedData.set(agentId, filteredData);
    }
  }

  /**
   * Counts total number of aggregations across all agents.
   */
  private countTotalAggregations(): number {
    let total = 0;
    for (const data of this.aggregatedData.values()) {
      total += data.length;
    }
    return total;
  }

  /**
   * Gets timestamp of oldest data.
   */
  private getOldestDataTimestamp(): Timestamp | null {
    let oldest: Timestamp | null = null;

    for (const data of this.aggregatedData.values()) {
      for (const item of data) {
        if (
          !oldest ||
          new Date(item.startTime).getTime() < new Date(oldest).getTime()
        ) {
          oldest = item.startTime;
        }
      }
    }

    return oldest;
  }

  /**
   * Gets timestamp of newest data.
   */
  private getNewestDataTimestamp(): Timestamp | null {
    let newest: Timestamp | null = null;

    for (const data of this.aggregatedData.values()) {
      for (const item of data) {
        if (
          !newest ||
          new Date(item.endTime).getTime() > new Date(newest).getTime()
        ) {
          newest = item.endTime;
        }
      }
    }

    return newest;
  }

  /**
   * Sets up event handlers for internal events.
   */
  private setupEventHandlers(): void {
    this.on("aggregation_completed", (_stats) => {
      // Could trigger downstream processing
    });

    this.on("aggregation_error", (_error) => {
      // Could trigger alerting or recovery
    });
  }
}
