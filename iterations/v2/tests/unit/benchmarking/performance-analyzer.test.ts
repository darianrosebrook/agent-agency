/**
 * Performance Analyzer Unit Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { PerformanceAnalyzer } from "../../../src/benchmarking/PerformanceAnalyzer";
import {
  AgentPerformanceProfile,
  AnalysisConfig,
  PerformanceAnomaly,
  TrendAnalysisResult,
} from "../../../src/types/performance-tracking";

describe("PerformanceAnalyzer", () => {
  let analyzer: PerformanceAnalyzer;
  let mockProfiles: AgentPerformanceProfile[];

  beforeEach(() => {
    analyzer = new PerformanceAnalyzer();
    mockProfiles = [
      {
        agentId: "agent-1",
        taskType: "coding",
        metrics: {
          latency: { averageMs: 1000, p95Ms: 1200, p99Ms: 1500, minMs: 800, maxMs: 2000 },
          accuracy: { successRate: 0.9, qualityScore: 0.85, violationRate: 0.1, evaluationScore: 0.8 },
          resources: { cpuUtilizationPercent: 70, memoryUtilizationPercent: 60, networkIoKbps: 100, diskIoKbps: 50 },
          compliance: { validationPassRate: 0.95, violationSeverityScore: 0.05, clauseCitationRate: 0.9 },
          cost: { costPerTask: 0.5, efficiencyScore: 0.85, resourceWastePercent: 15 },
          reliability: { mtbfHours: 168, availabilityPercent: 99.5, errorRatePercent: 0.5, recoveryTimeMinutes: 5 },
        },
        sampleSize: 10,
        confidence: 0.8,
        lastUpdated: new Date().toISOString(),
        trend: {
          direction: "stable",
          magnitude: 0.1,
          confidence: 0.7,
          timeWindowHours: 1,
        },
      },
      {
        agentId: "agent-2",
        taskType: "analysis",
        metrics: {
          latency: { averageMs: 800, p95Ms: 1000, p99Ms: 1200, minMs: 600, maxMs: 1500 },
          accuracy: { successRate: 0.95, qualityScore: 0.9, violationRate: 0.05, evaluationScore: 0.85 },
          resources: { cpuUtilizationPercent: 65, memoryUtilizationPercent: 55, networkIoKbps: 90, diskIoKbps: 45 },
          compliance: { validationPassRate: 0.98, violationSeverityScore: 0.02, clauseCitationRate: 0.95 },
          cost: { costPerTask: 0.4, efficiencyScore: 0.9, resourceWastePercent: 10 },
          reliability: { mtbfHours: 200, availabilityPercent: 99.8, errorRatePercent: 0.2, recoveryTimeMinutes: 3 },
        },
        sampleSize: 15,
        confidence: 0.85,
        lastUpdated: new Date().toISOString(),
        trend: {
          direction: "improving",
          magnitude: 0.05,
          confidence: 0.8,
          timeWindowHours: 2,
        },
      },
    ];
  });

  afterEach(() => {
    analyzer.stopAnalysis();
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const analyzer = new PerformanceAnalyzer();
      const stats = analyzer.getAnalysisStats();

      expect(stats.isAnalyzing).toBe(false);
      expect(stats.agentsTracked).toBe(0);
      expect(stats.totalAnomalies).toBe(0);
    });

    it("should create with custom config", () => {
      const customConfig: Partial<AnalysisConfig> = {
        anomalyThresholds: {
          latencySpikeMultiplier: 3.0,
          accuracyDropPercent: 20,
          errorRateIncreasePercent: 15,
          resourceSaturationPercent: 95,
        },
        trendAnalysis: {
          minDataPoints: 25,
          confidenceThreshold: 0.9,
        },
      };

      const analyzer = new PerformanceAnalyzer(customConfig);
      const stats = analyzer.getAnalysisStats();

      expect(stats.config.anomalyThresholds.latencySpikeMultiplier).toBe(3.0);
      expect(stats.config.trendAnalysis.minDataPoints).toBe(25);
    });
  });

  describe("analysis lifecycle", () => {
    it("should start and stop analysis", () => {
      analyzer.startAnalysis();
      expect(analyzer.getAnalysisStats().isAnalyzing).toBe(true);

      analyzer.stopAnalysis();
      expect(analyzer.getAnalysisStats().isAnalyzing).toBe(false);
    });
  });

  describe("performance analysis", () => {
    beforeEach(() => {
      analyzer.startAnalysis();
    });

    it("should analyze performance profiles and detect anomalies", async () => {
      // Create a profile that should trigger latency anomaly
      const anomalousProfile = {
        ...mockProfiles[0],
        metrics: {
          ...mockProfiles[0].metrics,
          latency: { averageMs: 5000, p95Ms: 6000, p99Ms: 7000, minMs: 4000, maxMs: 8000 }, // Very high latency
        },
      };

      const result = await analyzer.analyzePerformance([anomalousProfile]);

      expect(result.trendResults).toHaveLength(1);
      expect(result.newAnomalies).toHaveLength(1);
      expect(result.newAnomalies[0].type).toBe("latency_spike");
      expect(result.newAnomalies[0].severity).toBe("critical");
    });

    it("should handle empty profile arrays", async () => {
      const result = await analyzer.analyzePerformance([]);

      expect(result.trendResults).toHaveLength(0);
      expect(result.newAnomalies).toHaveLength(0);
      expect(result.resolvedAnomalies).toHaveLength(0);
    });

    it("should update analysis state with new profiles", async () => {
      await analyzer.analyzePerformance(mockProfiles);

      const stats = analyzer.getAnalysisStats();
      expect(stats.agentsTracked).toBe(2);
    });
  });

  describe("anomaly detection", () => {
    beforeEach(() => {
      analyzer.startAnalysis();
    });

    describe("latency spike detection", () => {
      it("should detect critical latency spikes", async () => {
        const profileWithSpike = {
          ...mockProfiles[0],
          metrics: {
            ...mockProfiles[0].metrics,
            latency: { averageMs: 6000, p95Ms: 7000, p99Ms: 8000, minMs: 5000, maxMs: 9000 },
          },
        };

        // First establish baseline
        await analyzer.analyzePerformance([mockProfiles[0]]);
        // Then detect anomaly
        const result = await analyzer.analyzePerformance([profileWithSpike]);

        expect(result.newAnomalies).toHaveLength(1);
        const anomaly = result.newAnomalies[0];
        expect(anomaly.type).toBe("latency_spike");
        expect(anomaly.severity).toBe("critical");
        expect(anomaly.agentId).toBe("agent-1");
        expect(anomaly.impact.affectedTasksPerHour).toBeGreaterThan(0);
      });

      it("should not detect anomalies within normal range", async () => {
        const result = await analyzer.analyzePerformance(mockProfiles);

        const latencyAnomalies = result.newAnomalies.filter(a => a.type === "latency_spike");
        expect(latencyAnomalies).toHaveLength(0);
      });
    });

    describe("accuracy drop detection", () => {
      it("should detect significant accuracy drops", async () => {
        const profileWithDrop = {
          ...mockProfiles[0],
          metrics: {
            ...mockProfiles[0].metrics,
            accuracy: { successRate: 0.7, qualityScore: 0.65, violationRate: 0.3, evaluationScore: 0.6 }, // 20% drop
          },
        };

        // Establish baseline
        await analyzer.analyzePerformance([mockProfiles[0]]);
        // Detect anomaly
        const result = await analyzer.analyzePerformance([profileWithDrop]);

        expect(result.newAnomalies.some(a => a.type === "accuracy_drop")).toBe(true);
      });
    });

    describe("error rate increase detection", () => {
      it("should detect error rate increases", async () => {
        const profileWithErrors = {
          ...mockProfiles[0],
          metrics: {
            ...mockProfiles[0].metrics,
            reliability: { mtbfHours: 150, availabilityPercent: 95, errorRatePercent: 5, recoveryTimeMinutes: 10 },
          },
        };

        // Establish baseline
        await analyzer.analyzePerformance([mockProfiles[0]]);
        // Detect anomaly
        const result = await analyzer.analyzePerformance([profileWithErrors]);

        expect(result.newAnomalies.some(a => a.type === "error_rate_increase")).toBe(true);
      });
    });

    describe("resource saturation detection", () => {
      it("should detect resource saturation", async () => {
        const profileWithSaturation = {
          ...mockProfiles[0],
          metrics: {
            ...mockProfiles[0].metrics,
            resources: { cpuUtilizationPercent: 96, memoryUtilizationPercent: 94, networkIoKbps: 100, diskIoKbps: 50 },
          },
        };

        const result = await analyzer.analyzePerformance([profileWithSaturation]);

        expect(result.newAnomalies.some(a => a.type === "resource_saturation")).toBe(true);
      });
    });

    describe("anomaly resolution", () => {
      it("should detect resolved anomalies", async () => {
        // Create anomaly
        const anomalousProfile = {
          ...mockProfiles[0],
          metrics: {
            ...mockProfiles[0].metrics,
            latency: { averageMs: 5000, p95Ms: 6000, p99Ms: 7000, minMs: 4000, maxMs: 8000 },
          },
        };

        // Create anomaly
        await analyzer.analyzePerformance([anomalousProfile]);

        // Resolve anomaly
        const normalProfile = mockProfiles[0];
        const result = await analyzer.analyzePerformance([normalProfile]);

        expect(result.resolvedAnomalies.some(a => a.type === "latency_spike")).toBe(true);
      });
    });
  });

  describe("trend analysis", () => {
    beforeEach(() => {
      analyzer.startAnalysis();
    });

    it("should analyze performance trends", async () => {
      await analyzer.analyzePerformance(mockProfiles);

      const trendResult = analyzer.getTrendAnalysis("agent-1");

      expect(trendResult).toBeDefined();
      if (trendResult) {
        expect(trendResult.agentId).toBe("agent-1");
        expect(trendResult.overallTrend).toBeDefined();
        expect(trendResult.metricTrends).toBeDefined();
        expect(trendResult.confidence).toBeGreaterThan(0);
        expect(trendResult.analysisTimeRange).toBeDefined();
      }
    });

    it("should return null for insufficient data", () => {
      const trendResult = analyzer.getTrendAnalysis("unknown-agent");
      expect(trendResult).toBeNull();
    });

    it("should calculate overall trend from multiple metrics", () => {
      const mockMetrics: AgentPerformanceProfile[] = [
        {
          agentId: "agent-1",
          taskType: "coding",
          metrics: {
            latency: { averageMs: 1000, p95Ms: 1200, p99Ms: 1500, minMs: 800, maxMs: 2000 },
            accuracy: { successRate: 0.8, qualityScore: 0.75, violationRate: 0.2, evaluationScore: 0.7 },
            resources: { cpuUtilizationPercent: 70, memoryUtilizationPercent: 60, networkIoKbps: 100, diskIoKbps: 50 },
            compliance: { validationPassRate: 0.9, violationSeverityScore: 0.1, clauseCitationRate: 0.8 },
            cost: { costPerTask: 0.5, efficiencyScore: 0.8, resourceWastePercent: 20 },
            reliability: { mtbfHours: 150, availabilityPercent: 98, errorRatePercent: 2, recoveryTimeMinutes: 10 },
          },
          sampleSize: 10,
          confidence: 0.8,
          lastUpdated: new Date(Date.now() - 3600000).toISOString(),
          trend: { direction: "stable", magnitude: 0, confidence: 0.5, timeWindowHours: 1 },
        },
        {
          agentId: "agent-1",
          taskType: "coding",
          metrics: {
            latency: { averageMs: 950, p95Ms: 1150, p99Ms: 1450, minMs: 750, maxMs: 1900 },
            accuracy: { successRate: 0.85, qualityScore: 0.8, violationRate: 0.15, evaluationScore: 0.75 },
            resources: { cpuUtilizationPercent: 68, memoryUtilizationPercent: 58, networkIoKbps: 95, diskIoKbps: 48 },
            compliance: { validationPassRate: 0.92, violationSeverityScore: 0.08, clauseCitationRate: 0.82 },
            cost: { costPerTask: 0.48, efficiencyScore: 0.82, resourceWastePercent: 18 },
            reliability: { mtbfHours: 160, availabilityPercent: 98.5, errorRatePercent: 1.5, recoveryTimeMinutes: 8 },
          },
          sampleSize: 10,
          confidence: 0.8,
          lastUpdated: new Date().toISOString(),
          trend: { direction: "improving", magnitude: 0.05, confidence: 0.7, timeWindowHours: 1 },
        },
      ];

      const trend = analyzer["calculateOverallTrend"](mockMetrics);

      expect(trend.direction).toBeDefined();
      expect(trend.magnitude).toBeGreaterThanOrEqual(0);
      expect(trend.confidence).toBeGreaterThanOrEqual(0);
      expect(trend.timeWindowHours).toBeGreaterThan(0);
    });
  });

  describe("anomaly retrieval", () => {
    beforeEach(async () => {
      analyzer.startAnalysis();

      // Create an anomaly
      const anomalousProfile = {
        ...mockProfiles[0],
        metrics: {
          ...mockProfiles[0].metrics,
          latency: { averageMs: 5000, p95Ms: 6000, p99Ms: 7000, minMs: 4000, maxMs: 8000 },
        },
      };

      await analyzer.analyzePerformance([anomalousProfile]);
    });

    it("should retrieve active anomalies", () => {
      const anomalies = analyzer.getActiveAnomalies();

      expect(anomalies).toHaveLength(1);
      expect(anomalies[0].type).toBe("latency_spike");
      expect(anomalies[0].agentId).toBe("agent-1");
    });

    it("should filter anomalies by agent", () => {
      const agentAnomalies = analyzer.getActiveAnomalies("agent-1");
      const otherAnomalies = analyzer.getActiveAnomalies("other-agent");

      expect(agentAnomalies).toHaveLength(1);
      expect(otherAnomalies).toHaveLength(0);
    });

    it("should filter anomalies by severity", () => {
      const criticalAnomalies = analyzer.getActiveAnomalies(undefined, "critical");
      const lowAnomalies = analyzer.getActiveAnomalies(undefined, "low");

      expect(criticalAnomalies).toHaveLength(1);
      expect(lowAnomalies).toHaveLength(0);
    });

    it("should sort anomalies by detection time", () => {
      // Add another anomaly
      const anotherAnomalousProfile = {
        ...mockProfiles[1],
        metrics: {
          ...mockProfiles[1].metrics,
          resources: { cpuUtilizationPercent: 96, memoryUtilizationPercent: 94, networkIoKbps: 100, diskIoKbps: 50 },
        },
      };

      analyzer.analyzePerformance([anotherAnomalousProfile]);

      const anomalies = analyzer.getActiveAnomalies();
      expect(anomalies).toHaveLength(2);

      // Should be sorted by detection time (newest first)
      const timestamps = anomalies.map(a => new Date(a.detectedAt).getTime());
      expect(timestamps[0]).toBeGreaterThanOrEqual(timestamps[1]);
    });
  });

  describe("baseline calculation", () => {
    it("should calculate baseline metrics from historical data", () => {
      const baseline = analyzer["calculateBaselineMetrics"](mockProfiles);

      expect(baseline.latency.averageMs).toBeGreaterThan(0);
      expect(baseline.accuracy.successRate).toBeGreaterThan(0);
      expect(baseline.resources.cpuUtilizationPercent).toBeGreaterThan(0);
      expect(baseline.compliance.validationPassRate).toBeGreaterThan(0);
      expect(baseline.cost.costPerTask).toBeGreaterThan(0);
      expect(baseline.reliability.mtbfHours).toBeGreaterThan(0);
    });

    it("should handle empty profile arrays", () => {
      const baseline = analyzer["calculateBaselineMetrics"]([]);

      expect(baseline.latency.averageMs).toBe(0);
      expect(baseline.accuracy.successRate).toBe(0);
    });
  });

  describe("statistical calculations", () => {
    it("should calculate confidence scores", () => {
      const confidence = analyzer["calculateTrendConfidence"]({
        latency: { direction: "improving", magnitude: 0.1, confidence: 0.8, timeWindowHours: 1 },
        accuracy: { direction: "stable", magnitude: 0.05, confidence: 0.9, timeWindowHours: 1 },
        resources: { direction: "declining", magnitude: 0.08, confidence: 0.7, timeWindowHours: 1 },
        compliance: { direction: "improving", magnitude: 0.03, confidence: 0.85, timeWindowHours: 1 },
        cost: { direction: "stable", magnitude: 0.02, confidence: 0.75, timeWindowHours: 1 },
        reliability: { direction: "improving", magnitude: 0.06, confidence: 0.8, timeWindowHours: 1 },
      });

      expect(confidence).toBeGreaterThan(0);
      expect(confidence).toBeLessThanOrEqual(1);
    });

    it("should calculate linear trends", () => {
      const data = [
        { score: 0.8, timestamp: Date.now() - 3600000 },
        { score: 0.82, timestamp: Date.now() - 1800000 },
        { score: 0.85, timestamp: Date.now() },
      ];

      const trend = analyzer["calculateLinearTrend"](data);

      expect(trend.direction).toBe("improving");
      expect(trend.magnitude).toBeGreaterThan(0);
      expect(trend.confidence).toBeGreaterThan(0);
    });

    it("should handle insufficient data for trend calculation", () => {
      const trend = analyzer["calculateLinearTrend"]([{ score: 0.8, timestamp: Date.now() }]);

      expect(trend.direction).toBe("stable");
      expect(trend.magnitude).toBe(0);
      expect(trend.confidence).toBe(0.5);
    });
  });

  describe("memory management", () => {
    it("should clear analysis data", () => {
      analyzer.startAnalysis();
      analyzer.analyzePerformance(mockProfiles);

      analyzer.clearData();

      const stats = analyzer.getAnalysisStats();
      expect(stats.agentsTracked).toBe(0);
      expect(stats.totalAnomalies).toBe(0);
    });
  });

  describe("configuration management", () => {
    it("should update configuration", () => {
      analyzer.updateConfig({
        anomalyThresholds: {
          latencySpikeMultiplier: 2.5,
          accuracyDropPercent: 18,
          errorRateIncreasePercent: 12,
          resourceSaturationPercent: 92,
        },
      });

      const stats = analyzer.getAnalysisStats();
      expect(stats.config.anomalyThresholds.latencySpikeMultiplier).toBe(2.5);
      expect(stats.config.anomalyThresholds.accuracyDropPercent).toBe(18);
    });

    it("should emit config update events", () => {
      const mockEmitter = jest.fn();
      analyzer.on("config_updated", mockEmitter);

      analyzer.updateConfig({ trendAnalysis: { minDataPoints: 30 } });

      expect(mockEmitter).toHaveBeenCalled();
    });
  });

  describe("event emission", () => {
    it("should emit analysis completed events", async () => {
      const mockEmitter = jest.fn();
      analyzer.on("analysis_completed", mockEmitter);

      analyzer.startAnalysis();
      await analyzer.analyzePerformance(mockProfiles);

      expect(mockEmitter).toHaveBeenCalledWith(
        expect.objectContaining({
          agentsAnalyzed: 2,
          newAnomalies: expect.any(Number),
        })
      );
    });

    it("should emit analysis error events", async () => {
      const mockEmitter = jest.fn();
      analyzer.on("analysis_error", mockEmitter);

      analyzer.startAnalysis();

      // Force an error by passing invalid data
      await analyzer.analyzePerformance([{} as any]);

      expect(mockEmitter).toHaveBeenCalled();
    });
  });

  describe("median calculations", () => {
    it("should calculate median latency metrics", () => {
      const profiles = mockProfiles.slice(0, 2); // Use 2 profiles for testing
      const median = analyzer["calculateMedianLatency"](profiles);

      expect(median.averageMs).toBeGreaterThan(0);
      expect(median.p95Ms).toBeGreaterThanOrEqual(median.averageMs);
    });

    it("should calculate median accuracy metrics", () => {
      const profiles = mockProfiles.slice(0, 2);
      const median = analyzer["calculateMedianAccuracy"](profiles);

      expect(median.successRate).toBeGreaterThan(0);
      expect(median.successRate).toBeLessThanOrEqual(1);
    });

    it("should calculate median resource metrics", () => {
      const profiles = mockProfiles.slice(0, 2);
      const median = analyzer["calculateMedianResources"](profiles);

      expect(median.cpuUtilizationPercent).toBeGreaterThanOrEqual(0);
      expect(median.cpuUtilizationPercent).toBeLessThanOrEqual(100);
    });

    it("should calculate median compliance metrics", () => {
      const profiles = mockProfiles.slice(0, 2);
      const median = analyzer["calculateMedianCompliance"](profiles);

      expect(median.validationPassRate).toBeGreaterThanOrEqual(0);
      expect(median.validationPassRate).toBeLessThanOrEqual(1);
    });

    it("should calculate median cost metrics", () => {
      const profiles = mockProfiles.slice(0, 2);
      const median = analyzer["calculateMedianCost"](profiles);

      expect(median.costPerTask).toBeGreaterThanOrEqual(0);
      expect(median.efficiencyScore).toBeGreaterThanOrEqual(0);
    });

    it("should calculate median reliability metrics", () => {
      const profiles = mockProfiles.slice(0, 2);
      const median = analyzer["calculateMedianReliability"](profiles);

      expect(median.mtbfHours).toBeGreaterThan(0);
      expect(median.availabilityPercent).toBeGreaterThanOrEqual(0);
    });
  });
});
