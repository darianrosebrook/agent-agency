/**
 * Metric Aggregator Unit Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { MetricAggregator } from "../../../src/benchmarking/MetricAggregator";
import {
  VerificationPriority,
  AgentPerformanceProfile,
  AggregationConfig,
  PerformanceEvent,
  PerformanceEventType,
} from "../../../src/types/performance-tracking";

describe("MetricAggregator", () => {
  let aggregator: MetricAggregator;
  let mockEvents: PerformanceEvent[];
  let mockProfiles: AgentPerformanceProfile[];

  beforeEach(() => {
    aggregator = new MetricAggregator();
    mockEvents = [
      {
        id: "event-1",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        agentId: "agent-1",
        taskId: "task-1",
        metrics: {
          latency: {
            averageMs: 1000,
            p95Ms: 1200,
            p99Ms: 1500,
            minMs: 800,
            maxMs: 2000,
          },
          accuracy: {
            successRate: 0.9,
            qualityScore: 0.85,
            violationRate: 0.1,
            evaluationScore: 0.8,
          },
          resources: {
            cpuUtilizationPercent: 70,
            memoryUtilizationPercent: 60,
            networkIoKbps: 100,
            diskIoKbps: 50,
          },
          compliance: {
            validationPassRate: 0.95,
            violationSeverityScore: 0.05,
            clauseCitationRate: 0.9,
          },
          cost: {
            costPerTask: 0.5,
            efficiencyScore: 0.85,
            resourceWastePercent: 15,
          },
          reliability: {
            mtbfHours: 168,
            availabilityPercent: 99.5,
            errorRatePercent: 0.5,
            recoveryTimeMinutes: 5,
          },
        },
        integrityHash: "hash1",
      },
      {
        id: "event-2",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date(Date.now() - 60000).toISOString(), // 1 minute ago
        agentId: "agent-1",
        taskId: "task-2",
        metrics: {
          latency: {
            averageMs: 1200,
            p95Ms: 1400,
            p99Ms: 1700,
            minMs: 900,
            maxMs: 2200,
          },
          accuracy: {
            successRate: 0.8,
            qualityScore: 0.75,
            violationRate: 0.2,
            evaluationScore: 0.7,
          },
          resources: {
            cpuUtilizationPercent: 75,
            memoryUtilizationPercent: 65,
            networkIoKbps: 120,
            diskIoKbps: 60,
          },
          compliance: {
            validationPassRate: 0.9,
            violationSeverityScore: 0.1,
            clauseCitationRate: 0.8,
          },
          cost: {
            costPerTask: 0.6,
            efficiencyScore: 0.8,
            resourceWastePercent: 20,
          },
          reliability: {
            mtbfHours: 140,
            availabilityPercent: 98.5,
            errorRatePercent: 1.5,
            recoveryTimeMinutes: 10,
          },
        },
        integrityHash: "hash2",
      },
    ];

    mockProfiles = [
      {
        agentId: "agent-1",
        taskType: "coding",
        metrics: {
          latency: {
            averageMs: 1100,
            p95Ms: 1300,
            p99Ms: 1600,
            minMs: 850,
            maxMs: 2100,
          },
          accuracy: {
            successRate: 0.85,
            qualityScore: 0.8,
            violationRate: 0.15,
            evaluationScore: 0.75,
          },
          resources: {
            cpuUtilizationPercent: 72.5,
            memoryUtilizationPercent: 62.5,
            networkIoKbps: 110,
            diskIoKbps: 55,
          },
          compliance: {
            validationPassRate: 0.925,
            violationSeverityScore: 0.075,
            clauseCitationRate: 0.85,
          },
          cost: {
            costPerTask: 0.55,
            efficiencyScore: 0.825,
            resourceWastePercent: 17.5,
          },
          reliability: {
            mtbfHours: 154,
            availabilityPercent: 99,
            errorRatePercent: 1,
            recoveryTimeMinutes: 7.5,
          },
        },
        sampleSize: 2,
        confidence: 0.8,
        lastUpdated: new Date().toISOString(),
        trend: {
          direction: "stable",
          magnitude: 0.1,
          confidence: 0.7,
          timeWindowHours: 1,
        },
      },
    ];
  });

  afterEach(() => {
    aggregator.stopAggregation();
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const aggregator = new MetricAggregator();
      const stats = aggregator.getStats();

      expect(stats.isAggregating).toBe(false);
      expect(stats.totalAggregations).toBe(0);
    });

    it("should create with custom config", () => {
      const customConfig: Partial<AggregationConfig> = {
        windows: {
          realtime: { durationMs: 300000, slideMs: 30000, minSampleSize: 5 },
          short: { durationMs: 3600000, slideMs: 900000, minSampleSize: 25 },
          medium: {
            durationMs: 86400000,
            slideMs: 14400000,
            minSampleSize: 50,
          },
          long: {
            durationMs: 604800000,
            slideMs: 86400000,
            minSampleSize: 250,
          },
        },
      };

      const aggregator = new MetricAggregator(customConfig);
      const stats = aggregator.getStats();

      expect(stats.config.windows.realtime.minSampleSize).toBe(5);
    });
  });

  describe("aggregation lifecycle", () => {
    it("should start and stop aggregation", () => {
      aggregator.startAggregation();
      expect(aggregator.getStats().isAggregating).toBe(true);

      aggregator.stopAggregation();
      expect(aggregator.getStats().isAggregating).toBe(false);
    });
  });

  describe("event processing", () => {
    beforeEach(() => {
      aggregator.startAggregation();
    });

    it("should process events and create aggregations", async () => {
      aggregator.addEvents(mockEvents);
      await aggregator.performAggregation();

      const stats = aggregator.getStats();
      expect(stats.totalAggregations).toBeGreaterThan(0);

      const profiles = aggregator.getPerformanceProfiles("agent-1");
      expect(profiles).toHaveLength(4); // realtime, short, medium, long windows
    });

    it("should handle empty event batches", async () => {
      aggregator.addEvents([]);
      await aggregator.performAggregation();

      const stats = aggregator.getStats();
      expect(stats.totalAggregations).toBe(0);
    });

    it("should filter out outliers", async () => {
      const normalEvent = { ...mockEvents[0] };
      const outlierEvent = {
        ...mockEvents[0],
        id: "outlier",
        metrics: {
          ...mockEvents[0].metrics,
          accuracy: {
            successRate: -1,
            qualityScore: 0,
            violationRate: 0,
            evaluationScore: 0,
          }, // Invalid
        },
      };

      aggregator.addEvents([normalEvent, outlierEvent]);
      await aggregator.performAggregation();

      const stats = aggregator.getStats();
      expect(stats.totalAggregations).toBeGreaterThan(0);
    });
  });

  describe("performance profile retrieval", () => {
    beforeEach(async () => {
      aggregator.startAggregation();
      aggregator.addEvents(mockEvents);
      await aggregator.performAggregation();
    });

    it("should retrieve performance profiles by agent", () => {
      const profiles = aggregator.getPerformanceProfiles("agent-1");

      expect(profiles).toHaveLength(4);
      profiles.forEach((profile) => {
        expect(profile.agentId).toBe("agent-1");
        expect(profile.metrics).toBeDefined();
        expect(profile.confidence).toBeGreaterThan(0);
        expect(profile.trend).toBeDefined();
      });
    });

    it("should filter profiles by task type", () => {
      // Add events with different task types
      const codingEvent = { ...mockEvents[0], taskId: "coding-1" };
      const analysisEvent = { ...mockEvents[1], taskId: "analysis-1" };

      aggregator.addEvents([codingEvent, analysisEvent]);
      aggregator.performAggregation();

      const codingProfiles = aggregator.getPerformanceProfiles(
        "agent-1",
        "coding"
      );
      const analysisProfiles = aggregator.getPerformanceProfiles(
        "agent-1",
        "analysis"
      );

      expect(codingProfiles.length).toBeGreaterThan(0);
      expect(analysisProfiles.length).toBeGreaterThan(0);
    });

    it("should return empty array for unknown agent", () => {
      const profiles = aggregator.getPerformanceProfiles("unknown-agent");
      expect(profiles).toHaveLength(0);
    });
  });

  describe("benchmark data retrieval", () => {
    beforeEach(async () => {
      aggregator.startAggregation();
      aggregator.addEvents(mockEvents);
      await aggregator.performAggregation();
    });

    it("should retrieve benchmark data within time range", () => {
      const startTime = new Date(Date.now() - 3600000).toISOString(); // 1 hour ago
      const endTime = new Date().toISOString();

      const benchmarkData = aggregator.getBenchmarkData(startTime, endTime);

      expect(benchmarkData.length).toBeGreaterThan(0);
      benchmarkData.forEach((data) => {
        expect(new Date(data.startTime).getTime()).toBeGreaterThanOrEqual(
          new Date(startTime).getTime()
        );
        expect(new Date(data.endTime).getTime()).toBeLessThanOrEqual(
          new Date(endTime).getTime()
        );
      });
    });

    it("should filter benchmark data by agent", () => {
      const startTime = new Date(Date.now() - 3600000).toISOString();
      const endTime = new Date().toISOString();

      const allData = aggregator.getBenchmarkData(startTime, endTime);
      const agentData = aggregator.getBenchmarkData(
        startTime,
        endTime,
        "agent-1"
      );

      expect(agentData.length).toBeLessThanOrEqual(allData.length);
      agentData.forEach((data) => {
        expect(data.agentId).toBe("agent-1");
      });
    });

    it("should filter benchmark data by task type", () => {
      const startTime = new Date(Date.now() - 3600000).toISOString();
      const endTime = new Date().toISOString();

      const allData = aggregator.getBenchmarkData(startTime, endTime);
      const filteredData = aggregator.getBenchmarkData(
        startTime,
        endTime,
        undefined,
        "coding"
      );

      expect(filteredData.length).toBeLessThanOrEqual(allData.length);
    });
  });

  describe("metric aggregation", () => {
    it("should aggregate latency metrics correctly", () => {
      const metrics = aggregator["aggregateMetrics"](mockEvents);

      expect(metrics.latency.averageMs).toBeGreaterThan(0);
      expect(metrics.latency.p95Ms).toBeGreaterThan(metrics.latency.averageMs);
      expect(metrics.latency.p99Ms).toBeGreaterThanOrEqual(
        metrics.latency.p95Ms
      );
      expect(metrics.latency.minMs).toBeLessThanOrEqual(
        metrics.latency.averageMs
      );
      expect(metrics.latency.maxMs).toBeGreaterThanOrEqual(
        metrics.latency.averageMs
      );
    });

    it("should aggregate accuracy metrics correctly", () => {
      const metrics = aggregator["aggregateMetrics"](mockEvents);

      expect(metrics.accuracy.successRate).toBeGreaterThan(0);
      expect(metrics.accuracy.successRate).toBeLessThanOrEqual(1);
      expect(metrics.accuracy.qualityScore).toBeGreaterThanOrEqual(0);
      expect(metrics.accuracy.qualityScore).toBeLessThanOrEqual(1);
      expect(metrics.accuracy.violationRate).toBeGreaterThanOrEqual(0);
      expect(metrics.accuracy.violationRate).toBeLessThanOrEqual(1);
    });

    it("should aggregate resource metrics correctly", () => {
      const metrics = aggregator["aggregateMetrics"](mockEvents);

      expect(metrics.resources.cpuUtilizationPercent).toBeGreaterThanOrEqual(0);
      expect(metrics.resources.cpuUtilizationPercent).toBeLessThanOrEqual(100);
      expect(metrics.resources.memoryUtilizationPercent).toBeGreaterThanOrEqual(
        0
      );
      expect(metrics.resources.memoryUtilizationPercent).toBeLessThanOrEqual(
        100
      );
    });

    it("should aggregate compliance metrics correctly", () => {
      const metrics = aggregator["aggregateMetrics"](mockEvents);

      expect(metrics.compliance.validationPassRate).toBeGreaterThanOrEqual(0);
      expect(metrics.compliance.validationPassRate).toBeLessThanOrEqual(1);
      expect(metrics.compliance.violationSeverityScore).toBeGreaterThanOrEqual(
        0
      );
      expect(metrics.compliance.clauseCitationRate).toBeGreaterThanOrEqual(0);
    });

    it("should aggregate cost metrics correctly", () => {
      const metrics = aggregator["aggregateMetrics"](mockEvents);

      expect(metrics.cost.costPerTask).toBeGreaterThanOrEqual(0);
      expect(metrics.cost.efficiencyScore).toBeGreaterThanOrEqual(0);
      expect(metrics.cost.efficiencyScore).toBeLessThanOrEqual(1);
    });

    it("should aggregate reliability metrics correctly", () => {
      const metrics = aggregator["aggregateMetrics"](mockEvents);

      expect(metrics.reliability.mtbfHours).toBeGreaterThan(0);
      expect(metrics.reliability.availabilityPercent).toBeGreaterThanOrEqual(0);
      expect(metrics.reliability.availabilityPercent).toBeLessThanOrEqual(100);
      expect(metrics.reliability.errorRatePercent).toBeGreaterThanOrEqual(0);
    });
  });

  describe("trend analysis", () => {
    it("should calculate performance trends", () => {
      const mockMetrics: AgentPerformanceProfile[] = [
        {
          agentId: "agent-1",
          taskType: "coding",
          metrics: {
            latency: {
              averageMs: 1000,
              p95Ms: 1200,
              p99Ms: 1500,
              minMs: 800,
              maxMs: 2000,
            },
            accuracy: {
              successRate: 0.8,
              qualityScore: 0.75,
              violationRate: 0.2,
              evaluationScore: 0.7,
            },
            resources: {
              cpuUtilizationPercent: 70,
              memoryUtilizationPercent: 60,
              networkIoKbps: 100,
              diskIoKbps: 50,
            },
            compliance: {
              validationPassRate: 0.9,
              violationSeverityScore: 0.1,
              clauseCitationRate: 0.8,
            },
            cost: {
              costPerTask: 0.5,
              efficiencyScore: 0.8,
              resourceWastePercent: 20,
            },
            reliability: {
              mtbfHours: 150,
              availabilityPercent: 98,
              errorRatePercent: 2,
              recoveryTimeMinutes: 10,
            },
          },
          sampleSize: 10,
          confidence: 0.8,
          lastUpdated: new Date(Date.now() - 3600000).toISOString(), // 1 hour ago
          trend: {
            direction: "stable",
            magnitude: 0,
            confidence: 0.5,
            timeWindowHours: 1,
          },
        },
        {
          agentId: "agent-1",
          taskType: "coding",
          metrics: {
            latency: {
              averageMs: 950,
              p95Ms: 1150,
              p99Ms: 1450,
              minMs: 750,
              maxMs: 1900,
            },
            accuracy: {
              successRate: 0.85,
              qualityScore: 0.8,
              violationRate: 0.15,
              evaluationScore: 0.75,
            },
            resources: {
              cpuUtilizationPercent: 68,
              memoryUtilizationPercent: 58,
              networkIoKbps: 95,
              diskIoKbps: 48,
            },
            compliance: {
              validationPassRate: 0.92,
              violationSeverityScore: 0.08,
              clauseCitationRate: 0.82,
            },
            cost: {
              costPerTask: 0.48,
              efficiencyScore: 0.82,
              resourceWastePercent: 18,
            },
            reliability: {
              mtbfHours: 160,
              availabilityPercent: 98.5,
              errorRatePercent: 1.5,
              recoveryTimeMinutes: 8,
            },
          },
          sampleSize: 10,
          confidence: 0.8,
          lastUpdated: new Date().toISOString(),
          trend: {
            direction: "improving",
            magnitude: 0.05,
            confidence: 0.7,
            timeWindowHours: 1,
          },
        },
      ];

      const trend = aggregator["calculateOverallTrend"](mockMetrics);

      expect(trend.direction).toBeDefined();
      expect(trend.magnitude).toBeGreaterThanOrEqual(0);
      expect(trend.confidence).toBeGreaterThanOrEqual(0);
      expect(trend.confidence).toBeLessThanOrEqual(1);
      expect(trend.timeWindowHours).toBeGreaterThan(0);
    });

    it("should handle insufficient data for trend analysis", () => {
      const insufficientData: AgentPerformanceProfile[] = [
        {
          agentId: "agent-1",
          taskType: "coding",
          metrics: mockProfiles[0].metrics,
          sampleSize: 1,
          confidence: 0.5,
          lastUpdated: new Date().toISOString(),
          trend: {
            direction: "stable",
            magnitude: 0,
            confidence: 0.5,
            timeWindowHours: 1,
          },
        },
      ];

      const trend = aggregator["calculateOverallTrend"](insufficientData);

      expect(trend.direction).toBe("stable");
      expect(trend.magnitude).toBe(0);
      expect(trend.confidence).toBe(0.5);
    });
  });

  describe("outlier detection", () => {
    it("should detect outlier events", () => {
      const normalEvent = mockEvents[0];
      const outlierEvent = {
        ...mockEvents[0],
        id: "outlier",
        metrics: {
          ...mockEvents[0].metrics,
          latency: {
            averageMs: 10000,
            p95Ms: 15000,
            p99Ms: 20000,
            minMs: 5000,
            maxMs: 30000,
          }, // Very high latency
        },
      };

      const outliers = aggregator["detectOutliers"]([
        normalEvent,
        normalEvent,
        outlierEvent,
      ]);

      expect(outliers).toHaveLength(1);
      expect(outliers[0].id).toBe("outlier");
    });

    it("should handle insufficient data for outlier detection", () => {
      const outliers = aggregator["detectOutliers"]([mockEvents[0]]);

      expect(outliers).toHaveLength(0);
    });

    it("should identify invalid events as outliers", () => {
      const invalidEvent = {
        ...mockEvents[0],
        metrics: {
          ...mockEvents[0].metrics,
          accuracy: {
            successRate: -1,
            qualityScore: 0,
            violationRate: 0,
            evaluationScore: 0,
          }, // Invalid
        },
      };

      const isOutlier = aggregator["isOutlier"](invalidEvent);

      expect(isOutlier).toBe(true);
    });
  });

  describe("data anonymization", () => {
    it("should apply noise to aggregated metrics", () => {
      const aggregatorWithAnonymization = new MetricAggregator({
        anonymization: {
          enabled: true,
          noiseLevel: 0.1,
          preserveAgentIds: true,
        },
      });

      const value = 100;
      const anonymizedValue =
        aggregatorWithAnonymization["applyAnonymizationNoise"](value);

      // Should be close to original value but with some noise
      expect(anonymizedValue).toBeGreaterThan(value * 0.9);
      expect(anonymizedValue).toBeLessThan(value * 1.1);
      expect(anonymizedValue).toBeGreaterThanOrEqual(0);
    });

    it("should not apply noise when anonymization disabled", () => {
      const aggregatorWithoutAnonymization = new MetricAggregator({
        anonymization: {
          enabled: false,
          noiseLevel: 0.1,
          preserveAgentIds: true,
        },
      });

      const value = 100;
      const result =
        aggregatorWithoutAnonymization["applyAnonymizationNoise"](value);

      expect(result).toBe(value);
    });
  });

  describe("memory management", () => {
    it("should clean up old aggregated data", async () => {
      aggregator.startAggregation();

      // Add some data
      aggregator.addEvents(mockEvents);
      await aggregator.performAggregation();

      const initialCount = aggregator.getStats().totalAggregations;

      // Simulate cleanup by directly calling the method
      aggregator["cleanupOldData"]();

      // Should still have data (not old enough)
      const finalCount = aggregator.getStats().totalAggregations;
      expect(finalCount).toBe(initialCount);
    });

    it("should clear all data when requested", () => {
      aggregator.startAggregation();
      aggregator.addEvents(mockEvents);

      aggregator.clearData();

      const stats = aggregator.getStats();
      expect(stats.totalAggregations).toBe(0);
      expect(stats.bufferSize).toBe(0);
      expect(stats.isAggregating).toBe(false);
    });
  });

  describe("configuration management", () => {
    it("should update configuration", () => {
      aggregator.updateConfig({
        outlierThresholds: {
          zScoreThreshold: 2.5,
          iqrMultiplier: 1.8,
        },
      });

      const stats = aggregator.getStats();
      expect(stats.config.outlierThresholds.zScoreThreshold).toBe(2.5);
      expect(stats.config.outlierThresholds.iqrMultiplier).toBe(1.8);
    });

    it("should emit config update events", () => {
      const mockEmitter = jest.fn();
      aggregator.on("config_updated", mockEmitter);

      aggregator.updateConfig({
        anonymization: {
          enabled: false,
          noiseLevel: 0,
          preserveAgentIds: true,
        },
      });

      expect(mockEmitter).toHaveBeenCalled();
    });
  });

  describe("event emission", () => {
    it("should emit aggregation completed events", async () => {
      const mockEmitter = jest.fn();
      aggregator.on("aggregation_completed", mockEmitter);

      aggregator.startAggregation();
      aggregator.addEvents(mockEvents);
      await aggregator.performAggregation();

      expect(mockEmitter).toHaveBeenCalledWith(
        expect.objectContaining({
          eventCount: 2,
          processingTimeMs: expect.any(Number),
        })
      );
    });

    it("should emit aggregation error events", async () => {
      const mockEmitter = jest.fn();
      aggregator.on("aggregation_error", mockEmitter);

      // Force an error by passing invalid data
      aggregator["aggregatedData"].set("invalid", null as any);

      aggregator.startAggregation();
      await aggregator.performAggregation();

      expect(mockEmitter).toHaveBeenCalled();
    });
  });

  describe("statistical calculations", () => {
    it("should calculate confidence scores correctly", () => {
      const confidence = aggregator["calculateConfidence"](50, "medium");

      expect(confidence).toBeGreaterThan(0);
      expect(confidence).toBeLessThanOrEqual(1);
    });

    it("should calculate averages correctly", () => {
      const values = [10, 20, 30, 40, 50];
      const average = aggregator["average"](values);

      expect(average).toBe(30);
    });

    it("should handle empty arrays in average calculation", () => {
      const average = aggregator["average"]([]);

      expect(average).toBe(0);
    });
  });
});
