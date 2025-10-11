/**
 * Performance Analyzer for Trend Analysis and Alerting
 *
 * @author @darianrosebrook
 * @module performance-analyzer
 *
 * Analyzes performance trends, detects anomalies, and generates alerts
 * for proactive performance monitoring and issue detection.
 */
// @ts-nocheck


import { EventEmitter } from "events";
import { Timestamp } from "../types/agent-registry";
import {
  AgentPerformanceProfile,
  AnalysisConfig,
  MetricCategory,
  PerformanceAnomaly,
  PerformanceMetrics,
  PerformanceTrend,
} from "../types/performance-tracking";

/**
 * Default analysis configuration.
 */
const DEFAULT_CONFIG: AnalysisConfig = {
  anomalyThresholds: {
    latencySpikeMultiplier: 2.0,
    accuracyDropPercent: 15,
    errorRateIncreasePercent: 10,
    resourceSaturationPercent: 90,
  },
  trendAnalysis: {
    minDataPoints: 20,
    confidenceThreshold: 0.8,
  },
  alertThresholds: {
    criticalLatencyMs: 5000,
    criticalErrorRatePercent: 5,
    criticalAccuracyDropPercent: 20,
  },
};

/**
 * Internal analysis state for tracking agent performance.
 */
interface AgentAnalysisState {
  agentId: string;
  baselineMetrics: PerformanceMetrics;
  recentMetrics: AgentPerformanceProfile[];
  activeAlerts: PerformanceAnomaly[];
  lastAnalysisTime: Timestamp;
  trendHistory: PerformanceTrend[];
}

/**
 * Trend analysis result.
 */
interface TrendAnalysisResult {
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
 * Performance Analyzer for trend analysis and alerting.
 *
 * This component analyzes performance data to detect trends, anomalies,
 * and issues, providing proactive monitoring and alerting capabilities.
 */
export class PerformanceAnalyzer extends EventEmitter {
  private config: AnalysisConfig;
  private analysisStates: Map<string, AgentAnalysisState> = new Map();
  private activeAnomalies: PerformanceAnomaly[] = [];
  private isAnalyzing = false;
  private analysisTimer?: ReturnType<typeof setTimeout>;

  /**
   * Creates a new Performance Analyzer instance.
   *
   * @param config - Analysis configuration. Uses defaults if not provided.
   */
  constructor(config: Partial<AnalysisConfig> = {}) {
    super();
    this.config = { ...DEFAULT_CONFIG, ...config } as AnalysisConfig;
    this.setupEventHandlers();
  }

  /**
   * Starts performance analysis.
   */
  startAnalysis(): void {
    this.isAnalyzing = true;
    this.scheduleAnalysis();
    this.emit("analysis_started");
  }

  /**
   * Stops performance analysis.
   */
  stopAnalysis(): void {
    this.isAnalyzing = false;
    if (this.analysisTimer) {
      clearTimeout(this.analysisTimer);
      this.analysisTimer = undefined;
    }
    this.emit("analysis_stopped");
  }

  /**
   * Analyzes performance profiles for trends and anomalies.
   *
   * @param profiles - Agent performance profiles to analyze
   * @returns Analysis results and detected anomalies
   */
  async analyzePerformance(profiles: AgentPerformanceProfile[]): Promise<{
    trendResults: TrendAnalysisResult[];
    newAnomalies: PerformanceAnomaly[];
    resolvedAnomalies: PerformanceAnomaly[];
  }> {
    if (!this.isAnalyzing) {
      return { trendResults: [], newAnomalies: [], resolvedAnomalies: [] };
    }

    const startTime = performance.now();
    const trendResults: TrendAnalysisResult[] = [];
    const newAnomalies: PerformanceAnomaly[] = [];
    const resolvedAnomalies: PerformanceAnomaly[] = [];

    try {
      // Update analysis states with new profiles
      this.updateAnalysisStates(profiles);

      // Analyze each agent
      for (const profile of profiles) {
        const state = this.analysisStates.get(profile.agentId);
        if (!state) continue;

        // Perform trend analysis
        const trendResult = this.analyzeTrends(profile, state);
        if (trendResult) {
          trendResults.push(trendResult);
        }

        // Detect anomalies
        const agentAnomalies = this.detectAnomalies(profile, state);
        for (const anomaly of agentAnomalies) {
          // Check if this anomaly is new
          const existingAnomaly = this.activeAnomalies.find(
            (a) => a.id === anomaly.id
          );

          if (!existingAnomaly) {
            newAnomalies.push(anomaly);
            this.activeAnomalies.push(anomaly);
          }
        }

        // Check for resolved anomalies
        const agentResolved = this.checkResolvedAnomalies(profile, state);
        resolvedAnomalies.push(...agentResolved);

        // Remove resolved anomalies from active list
        for (const resolved of agentResolved) {
          const index = this.activeAnomalies.findIndex(
            (a) => a.id === resolved.id
          );
          if (index !== -1) {
            this.activeAnomalies.splice(index, 1);
          }
        }

        state.lastAnalysisTime = new Date().toISOString();
      }

      // Clean up old analysis data
      this.cleanupOldData();

      const analysisTime = performance.now() - startTime;
      this.emit("analysis_completed", {
        agentsAnalyzed: profiles.length,
        trendResults: trendResults.length,
        newAnomalies: newAnomalies.length,
        resolvedAnomalies: resolvedAnomalies.length,
        analysisTimeMs: analysisTime,
      });
    } catch (error) {
      this.emit("analysis_error", error);
    }

    return { trendResults, newAnomalies, resolvedAnomalies };
  }

  /**
   * Gets current active anomalies.
   *
   * @param agentId - Filter by specific agent (optional)
   * @param severity - Filter by severity level (optional)
   * @returns Array of active anomalies
   */
  getActiveAnomalies(
    agentId?: string,
    severity?: "low" | "medium" | "high" | "critical"
  ): PerformanceAnomaly[] {
    let anomalies = this.activeAnomalies;

    if (agentId) {
      anomalies = anomalies.filter((a) => a.agentId === agentId);
    }

    if (severity) {
      anomalies = anomalies.filter((a) => a.severity === severity);
    }

    return anomalies.sort(
      (a, b) =>
        new Date(b.detectedAt).getTime() - new Date(a.detectedAt).getTime()
    );
  }

  /**
   * Gets performance trend analysis for an agent.
   *
   * @param agentId - Agent identifier
   * @returns Current trend analysis or null if not available
   */
  getTrendAnalysis(agentId: string): TrendAnalysisResult | null {
    const state = this.analysisStates.get(agentId);
    if (
      !state ||
      state.recentMetrics.length < this.config.trendAnalysis.minDataPoints
    ) {
      return null;
    }

    const latestProfile = state.recentMetrics[state.recentMetrics.length - 1];
    return this.analyzeTrends(latestProfile, state);
  }

  /**
   * Gets analysis statistics and health metrics.
   */
  getAnalysisStats() {
    const agentsTracked = this.analysisStates.size;
    const totalAnomalies = this.activeAnomalies.length;
    const criticalAnomalies = this.activeAnomalies.filter(
      (a) => a.severity === "critical"
    ).length;

    const anomalyBreakdown = {
      low: this.activeAnomalies.filter((a) => a.severity === "low").length,
      medium: this.activeAnomalies.filter((a) => a.severity === "medium")
        .length,
      high: this.activeAnomalies.filter((a) => a.severity === "high").length,
      critical: criticalAnomalies,
    };

    return {
      isAnalyzing: this.isAnalyzing,
      agentsTracked,
      totalAnomalies,
      criticalAnomalies,
      anomalyBreakdown,
      config: this.config,
    };
  }

  /**
   * Updates analysis configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<AnalysisConfig>): void {
    this.config = { ...this.config, ...config };
    this.emit("config_updated", this.config);
  }

  /**
   * Clears all analysis data and resets state.
   */
  clearData(): void {
    this.analysisStates.clear();
    this.activeAnomalies = [];
    this.emit("data_cleared");
  }

  /**
   * Updates analysis states with new performance profiles.
   */
  private updateAnalysisStates(profiles: AgentPerformanceProfile[]): void {
    for (const profile of profiles) {
      let state = this.analysisStates.get(profile.agentId);

      if (!state) {
        state = {
          agentId: profile.agentId,
          baselineMetrics: profile.metrics,
          recentMetrics: [],
          activeAlerts: [],
          lastAnalysisTime: new Date().toISOString(),
          trendHistory: [],
        };
        this.analysisStates.set(profile.agentId, state);
      }

      // Add new profile to recent metrics
      state.recentMetrics.push(profile);

      // Keep only recent data (last 100 data points)
      if (state.recentMetrics.length > 100) {
        state.recentMetrics = state.recentMetrics.slice(-100);
      }

      // Update baseline metrics periodically (every 24 hours worth of data)
      if (state.recentMetrics.length % 24 === 0) {
        state.baselineMetrics = this.calculateBaselineMetrics(
          state.recentMetrics
        );
      }
    }
  }

  /**
   * Analyzes performance trends for an agent.
   */
  private analyzeTrends(
    profile: AgentPerformanceProfile,
    state: AgentAnalysisState
  ): TrendAnalysisResult | null {
    if (state.recentMetrics.length < this.config.trendAnalysis.minDataPoints) {
      return null;
    }

    const metrics = state.recentMetrics.slice(
      -this.config.trendAnalysis.minDataPoints
    );
    const startTime = metrics[0].lastUpdated;
    const endTime = metrics[metrics.length - 1].lastUpdated;

    // Analyze overall trend
    const overallTrend = this.calculateOverallTrend(metrics);

    // Analyze trends for each metric category
    const metricTrends: Record<MetricCategory, PerformanceTrend> = {
      latency: this.calculateMetricTrend(
        metrics,
        "latency",
        (m) => m.latency.averageMs
      ),
      throughput: this.calculateThroughputTrend(metrics), // Special case
      accuracy: this.calculateMetricTrend(
        metrics,
        "accuracy",
        (m) => m.accuracy.successRate
      ),
      resource_utilization: this.calculateResourceTrend(metrics),
      constitutional_compliance: this.calculateMetricTrend(
        metrics,
        "compliance",
        (m) => m.compliance.validationPassRate
      ),
      cost_efficiency: this.calculateMetricTrend(
        metrics,
        "cost",
        (m) => m.cost.efficiencyScore
      ),
      reliability: this.calculateReliabilityTrend(metrics),
    };

    // Calculate overall confidence
    const confidence = this.calculateTrendConfidence(metricTrends);

    const result: TrendAnalysisResult = {
      agentId: profile.agentId,
      overallTrend,
      metricTrends,
      confidence,
      analysisTimeRange: { start: startTime, end: endTime },
    };

    // Update trend history
    state.trendHistory.push(overallTrend);
    if (state.trendHistory.length > 50) {
      state.trendHistory = state.trendHistory.slice(-50);
    }

    return result;
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
   * Calculates trend for a specific metric.
   */
  private calculateMetricTrend(
    metrics: AgentPerformanceProfile[],
    category: keyof PerformanceMetrics,
    extractor: (m: PerformanceMetrics) => number
  ): PerformanceTrend {
    const values = metrics.map((m) => ({
      value: extractor(m.metrics[category] as any),
      timestamp: new Date(m.lastUpdated).getTime(),
    }));

    return this.calculateLinearTrend(
      values.map((v) => ({ score: v.value, timestamp: v.timestamp }))
    );
  }

  /**
   * Calculates throughput trend (derived metric).
   */
  private calculateThroughputTrend(
    metrics: AgentPerformanceProfile[]
  ): PerformanceTrend {
    // Throughput is derived from latency and success rate
    const values = metrics.map((m) => ({
      score:
        m.metrics.accuracy.successRate /
        (1 + m.metrics.latency.averageMs / 1000),
      timestamp: new Date(m.lastUpdated).getTime(),
    }));

    return this.calculateLinearTrend(values);
  }

  /**
   * Calculates resource utilization trend.
   */
  private calculateResourceTrend(
    metrics: AgentPerformanceProfile[]
  ): PerformanceTrend {
    const values = metrics.map((m) => ({
      score:
        (m.metrics.resources.cpuUtilizationPercent +
          m.metrics.resources.memoryUtilizationPercent +
          m.metrics.resources.networkIoKbps +
          m.metrics.resources.diskIoKbps) /
        400, // Normalize to 0-1
      timestamp: new Date(m.lastUpdated).getTime(),
    }));

    return this.calculateLinearTrend(values);
  }

  /**
   * Calculates reliability trend.
   */
  private calculateReliabilityTrend(
    metrics: AgentPerformanceProfile[]
  ): PerformanceTrend {
    const values = metrics.map((m) => ({
      score:
        (m.metrics.reliability.availabilityPercent / 100 +
          (1 - m.metrics.reliability.errorRatePercent / 100) +
          m.metrics.reliability.mtbfHours / 168) / // Normalize to week
        3,
      timestamp: new Date(m.lastUpdated).getTime(),
    }));

    return this.calculateLinearTrend(values);
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
    const sumY = scores.reduce((sum, y) => sum + y, 0);
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
   * Calculates confidence score for trend analysis.
   */
  private calculateTrendConfidence(
    metricTrends: Record<MetricCategory, PerformanceTrend>
  ): number {
    const confidences = Object.values(metricTrends).map((t) => t.confidence);
    const avgConfidence =
      confidences.reduce((sum, c) => sum + c, 0) / confidences.length;
    const minConfidence = Math.min(...confidences);

    // Return weighted average favoring minimum confidence
    return 0.7 * avgConfidence + 0.3 * minConfidence;
  }

  /**
   * Detects performance anomalies.
   */
  private detectAnomalies(
    profile: AgentPerformanceProfile,
    state: AgentAnalysisState
  ): PerformanceAnomaly[] {
    const anomalies: PerformanceAnomaly[] = [];

    // Check latency spike
    const latencySpike = this.checkLatencySpike(profile, state);
    if (latencySpike) anomalies.push(latencySpike);

    // Check accuracy drop
    const accuracyDrop = this.checkAccuracyDrop(profile, state);
    if (accuracyDrop) anomalies.push(accuracyDrop);

    // Check error rate increase
    const errorIncrease = this.checkErrorRateIncrease(profile, state);
    if (errorIncrease) anomalies.push(errorIncrease);

    // Check resource saturation
    const resourceSaturation = this.checkResourceSaturation(profile, state);
    if (resourceSaturation) anomalies.push(resourceSaturation);

    return anomalies;
  }

  /**
   * Checks for latency performance spikes.
   */
  private checkLatencySpike(
    profile: AgentPerformanceProfile,
    state: AgentAnalysisState
  ): PerformanceAnomaly | null {
    const currentLatency = profile.metrics.latency.p95Ms;
    const baselineLatency = state.baselineMetrics.latency.p95Ms;

    if (
      currentLatency >
      baselineLatency * this.config.anomalyThresholds.latencySpikeMultiplier
    ) {
      const degradationPercent =
        ((currentLatency - baselineLatency) / baselineLatency) * 100;

      return {
        id: `latency_spike_${profile.agentId}_${Date.now()}`,
        type: "latency_spike",
        severity:
          currentLatency > this.config.alertThresholds.criticalLatencyMs
            ? "critical"
            : "high",
        agentId: profile.agentId,
        detectedAt: new Date().toISOString(),
        description: `Latency spike detected: ${currentLatency.toFixed(
          0
        )}ms (baseline: ${baselineLatency.toFixed(
          0
        )}ms, +${degradationPercent.toFixed(1)}%)`,
        impact: {
          affectedTasksPerHour: Math.floor(3600000 / currentLatency), // Rough estimate
          performanceDegradationPercent: degradationPercent,
          estimatedRecoveryTimeMinutes: 30, // Conservative estimate
        },
        recommendations: [
          "Check agent resource utilization",
          "Review recent code changes",
          "Consider scaling agent instances",
          "Monitor for memory leaks",
        ],
      };
    }

    return null;
  }

  /**
   * Checks for accuracy performance drops.
   */
  private checkAccuracyDrop(
    profile: AgentPerformanceProfile,
    state: AgentAnalysisState
  ): PerformanceAnomaly | null {
    const currentAccuracy = profile.metrics.accuracy.successRate * 100;
    const baselineAccuracy = state.baselineMetrics.accuracy.successRate * 100;
    const dropPercent = baselineAccuracy - currentAccuracy;

    if (dropPercent > this.config.anomalyThresholds.accuracyDropPercent) {
      return {
        id: `accuracy_drop_${profile.agentId}_${Date.now()}`,
        type: "accuracy_drop",
        severity:
          dropPercent > this.config.alertThresholds.criticalAccuracyDropPercent
            ? "critical"
            : "high",
        agentId: profile.agentId,
        detectedAt: new Date().toISOString(),
        description: `Accuracy drop detected: ${currentAccuracy.toFixed(
          1
        )}% (baseline: ${baselineAccuracy.toFixed(1)}%, -${dropPercent.toFixed(
          1
        )}%)`,
        impact: {
          affectedTasksPerHour: Math.floor(dropPercent * 10), // Rough estimate
          performanceDegradationPercent: dropPercent,
          estimatedRecoveryTimeMinutes: 60, // May require investigation
        },
        recommendations: [
          "Review recent model updates",
          "Check input data quality",
          "Validate training data integrity",
          "Consider model rollback",
        ],
      };
    }

    return null;
  }

  /**
   * Checks for error rate increases.
   */
  private checkErrorRateIncrease(
    profile: AgentPerformanceProfile,
    state: AgentAnalysisState
  ): PerformanceAnomaly | null {
    const currentErrorRate = profile.metrics.reliability.errorRatePercent;
    const baselineErrorRate =
      state.baselineMetrics.reliability.errorRatePercent;
    const increasePercent = currentErrorRate - baselineErrorRate;

    if (
      increasePercent > this.config.anomalyThresholds.errorRateIncreasePercent
    ) {
      return {
        id: `error_rate_increase_${profile.agentId}_${Date.now()}`,
        type: "error_rate_increase",
        severity:
          currentErrorRate >
          this.config.alertThresholds.criticalErrorRatePercent
            ? "critical"
            : "medium",
        agentId: profile.agentId,
        detectedAt: new Date().toISOString(),
        description: `Error rate increase detected: ${currentErrorRate.toFixed(
          1
        )}% (baseline: ${baselineErrorRate.toFixed(
          1
        )}%, +${increasePercent.toFixed(1)}%)`,
        impact: {
          affectedTasksPerHour: Math.floor(increasePercent * 100), // Rough estimate
          performanceDegradationPercent: increasePercent,
          estimatedRecoveryTimeMinutes: 15, // Often quick fixes
        },
        recommendations: [
          "Check application logs",
          "Review recent deployments",
          "Validate input validation",
          "Monitor external service health",
        ],
      };
    }

    return null;
  }

  /**
   * Checks for resource saturation.
   */
  private checkResourceSaturation(
    profile: AgentPerformanceProfile,
    state: AgentAnalysisState
  ): PerformanceAnomaly | null {
    const resources = profile.metrics.resources;
    const saturationPercent = Math.max(
      resources.cpuUtilizationPercent,
      resources.memoryUtilizationPercent
    );

    if (
      saturationPercent >
      this.config.anomalyThresholds.resourceSaturationPercent
    ) {
      return {
        id: `resource_saturation_${profile.agentId}_${Date.now()}`,
        type: "resource_saturation",
        severity: saturationPercent > 95 ? "critical" : "medium",
        agentId: profile.agentId,
        detectedAt: new Date().toISOString(),
        description: `Resource saturation detected: ${saturationPercent.toFixed(
          1
        )}% utilization`,
        impact: {
          affectedTasksPerHour: Math.floor(saturationPercent), // Rough estimate
          performanceDegradationPercent: saturationPercent - 80, // Degradation above 80%
          estimatedRecoveryTimeMinutes: saturationPercent > 95 ? 120 : 30,
        },
        recommendations: [
          "Scale agent instances horizontally",
          "Optimize resource-intensive operations",
          "Implement request throttling",
          "Review resource allocation",
        ],
      };
    }

    return null;
  }

  /**
   * Checks for resolved anomalies.
   */
  private checkResolvedAnomalies(
    profile: AgentPerformanceProfile,
    state: AgentAnalysisState
  ): PerformanceAnomaly[] {
    const resolved: PerformanceAnomaly[] = [];

    for (const anomaly of this.activeAnomalies) {
      if (anomaly.agentId !== profile.agentId) continue;

      let isResolved = false;

      switch (anomaly.type) {
        case "latency_spike":
          const currentLatency = profile.metrics.latency.p95Ms;
          const baselineLatency = state.baselineMetrics.latency.p95Ms;
          isResolved = currentLatency <= baselineLatency * 1.2; // Within 20% of baseline
          break;

        case "accuracy_drop":
          const currentAccuracy = profile.metrics.accuracy.successRate * 100;
          const baselineAccuracy =
            state.baselineMetrics.accuracy.successRate * 100;
          isResolved = baselineAccuracy - currentAccuracy < 5; // Within 5% of baseline
          break;

        case "error_rate_increase":
          const currentErrorRate = profile.metrics.reliability.errorRatePercent;
          const baselineErrorRate =
            state.baselineMetrics.reliability.errorRatePercent;
          isResolved = currentErrorRate - baselineErrorRate < 2; // Within 2% of baseline
          break;

        case "resource_saturation":
          const resources = profile.metrics.resources;
          const saturationPercent = Math.max(
            resources.cpuUtilizationPercent,
            resources.memoryUtilizationPercent
          );
          isResolved = saturationPercent < 85; // Below 85% utilization
          break;
      }

      if (isResolved) {
        resolved.push(anomaly);
      }
    }

    return resolved;
  }

  /**
   * Calculates baseline metrics from historical data.
   */
  private calculateBaselineMetrics(
    profiles: AgentPerformanceProfile[]
  ): PerformanceMetrics {
    // Use median of recent profiles as baseline
    const sortedProfiles = profiles.sort(
      (a, b) =>
        new Date(b.lastUpdated).getTime() - new Date(a.lastUpdated).getTime()
    );

    const recentProfiles = sortedProfiles.slice(
      0,
      Math.min(20, sortedProfiles.length)
    );

    return {
      latency: this.calculateMedianLatency(recentProfiles),
      accuracy: this.calculateMedianAccuracy(recentProfiles),
      resources: this.calculateMedianResources(recentProfiles),
      compliance: this.calculateMedianCompliance(recentProfiles),
      cost: this.calculateMedianCost(recentProfiles),
      reliability: this.calculateMedianReliability(recentProfiles),
    };
  }

  /**
   * Calculates median latency metrics.
   */
  private calculateMedianLatency(
    profiles: AgentPerformanceProfile[]
  ): import("../types/performance-tracking").LatencyMetrics {
    const latencies = profiles
      .map((p) => p.metrics.latency.averageMs)
      .sort((a, b) => a - b);
    const mid = Math.floor(latencies.length / 2);

    return {
      averageMs: latencies[mid] || 0,
      p95Ms:
        latencies[Math.floor(latencies.length * 0.95)] || latencies[mid] || 0,
      p99Ms:
        latencies[Math.floor(latencies.length * 0.99)] || latencies[mid] || 0,
      minMs: latencies[0] || 0,
      maxMs: latencies[latencies.length - 1] || 0,
    };
  }

  /**
   * Calculates median accuracy metrics.
   */
  private calculateMedianAccuracy(
    profiles: AgentPerformanceProfile[]
  ): import("../types/performance-tracking").AccuracyMetrics {
    const accuracies = profiles
      .map((p) => p.metrics.accuracy.successRate)
      .sort((a, b) => a - b);
    const mid = Math.floor(accuracies.length / 2);
    const medianAccuracy = accuracies[mid] || 0;

    return {
      successRate: medianAccuracy,
      qualityScore: medianAccuracy,
      violationRate: 1 - medianAccuracy,
      evaluationScore: medianAccuracy,
    };
  }

  /**
   * Calculates median resource metrics.
   */
  private calculateMedianResources(
    profiles: AgentPerformanceProfile[]
  ): import("../types/performance-tracking").ResourceMetrics {
    const cpuValues = profiles
      .map((p) => p.metrics.resources.cpuUtilizationPercent)
      .sort((a, b) => a - b);
    const memoryValues = profiles
      .map((p) => p.metrics.resources.memoryUtilizationPercent)
      .sort((a, b) => a - b);
    const networkValues = profiles
      .map((p) => p.metrics.resources.networkIoKbps)
      .sort((a, b) => a - b);
    const diskValues = profiles
      .map((p) => p.metrics.resources.diskIoKbps)
      .sort((a, b) => a - b);

    const mid = Math.floor(profiles.length / 2);

    return {
      cpuUtilizationPercent: cpuValues[mid] || 0,
      memoryUtilizationPercent: memoryValues[mid] || 0,
      networkIoKbps: networkValues[mid] || 0,
      diskIoKbps: diskValues[mid] || 0,
    };
  }

  /**
   * Calculates median compliance metrics.
   */
  private calculateMedianCompliance(
    profiles: AgentPerformanceProfile[]
  ): import("../types/performance-tracking").ComplianceMetrics {
    const passRates = profiles
      .map((p) => p.metrics.compliance.validationPassRate)
      .sort((a, b) => a - b);
    const mid = Math.floor(passRates.length / 2);
    const medianPassRate = passRates[mid] || 0;

    return {
      validationPassRate: medianPassRate,
      violationSeverityScore: 1 - medianPassRate,
      clauseCitationRate: medianPassRate,
    };
  }

  /**
   * Calculates median cost metrics.
   */
  private calculateMedianCost(
    profiles: AgentPerformanceProfile[]
  ): import("../types/performance-tracking").CostMetrics {
    const costs = profiles
      .map((p) => p.metrics.cost.costPerTask)
      .sort((a, b) => a - b);
    const efficiencies = profiles
      .map((p) => p.metrics.cost.efficiencyScore)
      .sort((a, b) => a - b);
    const mid = Math.floor(costs.length / 2);

    return {
      costPerTask: costs[mid] || 0,
      efficiencyScore: efficiencies[mid] || 0,
      resourceWastePercent: (1 - (efficiencies[mid] || 0)) * 100,
    };
  }

  /**
   * Calculates median reliability metrics.
   */
  private calculateMedianReliability(
    profiles: AgentPerformanceProfile[]
  ): import("../types/performance-tracking").ReliabilityMetrics {
    const availabilities = profiles
      .map((p) => p.metrics.reliability.availabilityPercent)
      .sort((a, b) => a - b);
    const errorRates = profiles
      .map((p) => p.metrics.reliability.errorRatePercent)
      .sort((a, b) => a - b);
    const mtbfs = profiles
      .map((p) => p.metrics.reliability.mtbfHours)
      .sort((a, b) => a - b);
    const mid = Math.floor(availabilities.length / 2);

    return {
      mtbfHours: mtbfs[mid] || 0,
      availabilityPercent: availabilities[mid] || 0,
      errorRatePercent: errorRates[mid] || 0,
      recoveryTimeMinutes: Math.max(
        0,
        (100 - (availabilities[mid] || 100)) * 10
      ), // Rough estimate
    };
  }

  /**
   * Schedules periodic analysis.
   */
  private scheduleAnalysis(): void {
    if (!this.isAnalyzing) return;

    // Analyze every 5 minutes
    this.analysisTimer = setTimeout(async () => {
      // This would be called with fresh performance profiles
      // For now, just reschedule
      this.scheduleAnalysis();
    }, 5 * 60 * 1000);
  }

  /**
   * Cleans up old analysis data.
   */
  private cleanupOldData(): void {
    // Clean up old trend history
    for (const state of this.analysisStates.values()) {
      state.trendHistory = state.trendHistory.filter(
        (_trend) =>
          // Keep trends from the last 30 days
          true // Simplified - would need trend timestamps
      );
    }

    // Note: Active anomalies are kept until resolved
  }

  /**
   * Sets up event handlers for internal events.
   */
  private setupEventHandlers(): void {
    this.on("analysis_completed", (stats) => {
      // Could trigger dashboard updates or notifications
    });

    this.on("analysis_error", (error) => {
      // Could trigger alerting for analysis system issues
    });
  }
}
