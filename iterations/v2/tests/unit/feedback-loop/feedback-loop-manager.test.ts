import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { ConfigManager } from "../../../src/config/ConfigManager";
import { FeedbackAnalyzer } from "../../../src/feedback-loop/FeedbackAnalyzer";
import { FeedbackCollector } from "../../../src/feedback-loop/FeedbackCollector";
import { FeedbackLoopManager } from "../../../src/feedback-loop/FeedbackLoopManager";
import { FeedbackPipeline } from "../../../src/feedback-loop/FeedbackPipeline";
import { ImprovementEngine } from "../../../src/feedback-loop/ImprovementEngine";
import {
  VerificationPriority,
  FeedbackRecommendation,
  FeedbackSource,
  FeedbackType,
} from "../../../src/types/feedback-loop";

// Mock dependencies
jest.mock("../../../src/feedback-loop/FeedbackCollector");
jest.mock("../../../src/feedback-loop/FeedbackAnalyzer");
jest.mock("../../../src/feedback-loop/ImprovementEngine");
jest.mock("../../../src/feedback-loop/FeedbackPipeline");
jest.mock("../../../src/config/ConfigManager");

describe("FeedbackLoopManager", () => {
  let manager: FeedbackLoopManager;
  let mockConfigManager: jest.Mocked<ConfigManager>;
  let mockCollector: jest.Mocked<FeedbackCollector>;
  let mockAnalyzer: jest.Mocked<FeedbackAnalyzer>;
  let mockImprovementEngine: jest.Mocked<ImprovementEngine>;
  let mockPipeline: jest.Mocked<FeedbackPipeline>;

  beforeEach(() => {
    jest.clearAllMocks();

    mockConfigManager = {
      get: jest.fn().mockReturnValue({
        enabled: true,
        collection: {
          enabledSources: [FeedbackSource.PERFORMANCE_METRICS],
          batchSize: 10,
          flushIntervalMs: 1000,
          retentionPeriodDays: 30,
          samplingRate: 1.0,
          filters: {},
        },
        analysis: {
          enabledAnalyzers: ["trend", "anomaly"],
          analysisIntervalMs: 5000,
          anomalyThreshold: 2.0,
          trendWindowHours: 24,
          minDataPoints: 5,
          correlationThreshold: 0.5,
          predictionHorizonHours: 24,
        },
        improvements: {
          autoApplyThreshold: 0.8,
          maxConcurrentImprovements: 5,
          cooldownPeriodMs: 300000,
          improvementTimeoutMs: 300000,
          rollbackOnFailure: true,
          monitoringPeriodMs: 600000,
        },
        pipeline: {
          batchSize: 50,
          processingIntervalMs: 10000,
          dataQualityThreshold: 0.7,
          anonymizationLevel: "partial",
          featureEngineering: {
            timeWindowFeatures: true,
            correlationFeatures: true,
            trendFeatures: true,
          },
          trainingDataFormat: "json",
        },
      }),
      set: jest.fn(),
      getAll: jest.fn().mockReturnValue({}),
      reload: jest.fn(),
      validate: jest.fn().mockReturnValue({ valid: true, errors: [] }),
    } as any;

    mockCollector =
      new (FeedbackCollector as any)() as jest.Mocked<FeedbackCollector>;
    mockAnalyzer =
      new (FeedbackAnalyzer as any)() as jest.Mocked<FeedbackAnalyzer>;
    mockImprovementEngine =
      new (ImprovementEngine as any)() as jest.Mocked<ImprovementEngine>;
    mockPipeline =
      new (FeedbackPipeline as any)() as jest.Mocked<FeedbackPipeline>;

    // Setup mock returns
    mockCollector.getStats.mockReturnValue({
      totalEvents: 100,
      eventsBySource: {
        [FeedbackSource.PERFORMANCE_METRICS]: 50,
        [FeedbackSource.TASK_OUTCOMES]: 0,
        [FeedbackSource.USER_RATINGS]: 0,
        [FeedbackSource.SYSTEM_EVENTS]: 0,
        [FeedbackSource.CONSTITUTIONAL_VIOLATIONS]: 0,
        [FeedbackSource.COMPONENT_HEALTH]: 0,
        [FeedbackSource.ROUTING_DECISIONS]: 0,
        [FeedbackSource.AGENT_FEEDBACK]: 0,
      },
      eventsByType: {
        [FeedbackType.NUMERIC_METRIC]: 50,
        [FeedbackType.CATEGORICAL_EVENT]: 0,
        [FeedbackType.TEXT_FEEDBACK]: 0,
        [FeedbackType.RATING_SCALE]: 0,
        [FeedbackType.BINARY_OUTCOME]: 0,
      },
      bufferSize: 5,
      isRunning: true,
      droppedEvents: 0,
      processingErrors: 0,
      lastFlushTime: new Date().toISOString(),
    });

    mockImprovementEngine.getStats.mockReturnValue({
      activeImprovements: 2,
      totalImprovements: 15,
      successRates: { agent_update: { successRate: 0.8, attempts: 10 } },
      cooldownsActive: 1,
    });

    mockPipeline.getStats.mockReturnValue({
      totalBatchesProcessed: 10,
      totalEventsProcessed: 500,
      averageQualityScore: 0.85,
      batchesByQuality: { high: 7, medium: 2, low: 1 },
      processingErrors: 0,
      lastProcessingTime: new Date().toISOString(),
      pendingBatches: 2,
      processedBatches: 8,
    });

    mockCollector.collectPerformanceMetrics.mockImplementation(() => {});
    mockCollector.collectTaskOutcome.mockImplementation(() => {});
    mockCollector.collectUserRating.mockImplementation(() => {});
    mockCollector.collectSystemEvent.mockImplementation(() => {});
    mockCollector.collectConstitutionalViolation.mockImplementation(() => {});
    mockCollector.collectComponentHealth.mockImplementation(() => {});
    mockCollector.collectRoutingDecision.mockImplementation(() => {});
    mockCollector.collectAgentFeedback.mockImplementation(() => {});

    mockAnalyzer.analyzeEntityFeedback.mockReturnValue({
      id: "analysis-1",
      entityId: "entity-1",
      entityType: "agent",
      timeWindow: {
        start: "2025-01-01T00:00:00Z",
        end: "2025-01-02T00:00:00Z",
      },
      metrics: {
        totalFeedbackEvents: 10,
        performanceTrend: "stable",
        anomalyCount: 0,
        correlationStrength: 0.3,
      },
      insights: [],
      recommendations: [],
      confidence: 0.8,
      generatedAt: new Date().toISOString(),
    });

    mockImprovementEngine.applyRecommendation.mockResolvedValue(true);
    mockImprovementEngine.applyRecommendations.mockResolvedValue({
      applied: [],
      skipped: [],
      failed: [],
    });

    manager = new FeedbackLoopManager(mockConfigManager);
  });

  describe("initialization", () => {
    it("should initialize correctly", async () => {
      await manager.initialize();

      expect(mockCollector.start).toHaveBeenCalled();
      expect(manager).toBeDefined();
    });

    it("should not start collector if feedback loop is disabled", async () => {
      mockConfigManager.get.mockReturnValue({
        enabled: false,
        collection: {
          enabledSources: [FeedbackSource.PERFORMANCE_METRICS],
          batchSize: 10,
          flushIntervalMs: 1000,
          retentionPeriodDays: 30,
          samplingRate: 1.0,
          filters: {},
        },
        analysis: {
          enabledAnalyzers: ["trend", "anomaly"],
          analysisIntervalMs: 5000,
          anomalyThreshold: 2.0,
          trendWindowHours: 24,
          minDataPoints: 5,
          correlationThreshold: 0.5,
          predictionHorizonHours: 24,
        },
        improvements: {
          autoApplyThreshold: 0.8,
          maxConcurrentImprovements: 5,
          cooldownPeriodMs: 300000,
          improvementTimeoutMs: 300000,
          rollbackOnFailure: true,
          monitoringPeriodMs: 600000,
        },
        pipeline: {
          batchSize: 50,
          processingIntervalMs: 10000,
          dataQualityThreshold: 0.7,
          anonymizationLevel: "partial",
          featureEngineering: {
            timeWindowFeatures: true,
            correlationFeatures: true,
            trendFeatures: true,
          },
          trainingDataFormat: "json",
        },
      });

      const disabledManager = new FeedbackLoopManager(mockConfigManager);
      await disabledManager.initialize();

      expect(mockCollector.start).not.toHaveBeenCalled();
    });
  });

  describe("shutdown", () => {
    it("should shutdown correctly", async () => {
      await manager.initialize();
      await manager.shutdown();

      expect(mockCollector.stop).toHaveBeenCalled();
      expect(mockPipeline.flush).toHaveBeenCalled();
    });
  });

  describe("feedback collection", () => {
    beforeEach(async () => {
      await manager.initialize();
    });

    it("should collect performance metrics", () => {
      const metrics = { latencyMs: 100, throughput: 50 };
      manager.collectPerformanceMetrics("agent-1", "agent", metrics);

      expect(mockCollector.collectPerformanceMetrics).toHaveBeenCalledWith(
        "agent-1",
        "agent",
        metrics
      );
    });

    it("should collect task outcomes", () => {
      const outcome = { success: true, qualityScore: 0.9 };
      manager.collectTaskOutcome("task-1", outcome, 5000, 0);

      expect(mockCollector.collectTaskOutcome).toHaveBeenCalledWith(
        "task-1",
        outcome,
        5000,
        0,
        undefined
      );
    });

    it("should collect user ratings", () => {
      const criteria = { accuracy: 5, speed: 4 };
      manager.collectUserRating("agent-1", "agent", 4.5, criteria, "Good work");

      expect(mockCollector.collectUserRating).toHaveBeenCalledWith(
        "agent-1",
        "agent",
        4.5,
        criteria,
        "Good work"
      );
    });

    it("should collect system events", () => {
      const impact = { affectedComponents: ["router"] };
      manager.collectSystemEvent("event-1", "high", "System overload", impact);

      expect(mockCollector.collectSystemEvent).toHaveBeenCalledWith(
        "event-1",
        "high",
        "System overload",
        impact
      );
    });
  });

  describe("analysis", () => {
    it("should analyze entity feedback", () => {
      const analysis = manager.analyzeEntity("agent-1", "agent");

      expect(mockAnalyzer.analyzeEntityFeedback).toHaveBeenCalledWith(
        "agent-1",
        "agent"
      );
      expect(analysis).toBeDefined();
      expect(analysis.entityId).toBe("agent-1");
    });

    it("should analyze all entities", () => {
      mockAnalyzer.analyzeAllEntities.mockReturnValue([]);

      const analyses = manager.analyzeAllEntities();

      expect(mockAnalyzer.analyzeAllEntities).toHaveBeenCalled();
      expect(analyses).toEqual([]);
    });
  });

  describe("improvements", () => {
    it("should apply single recommendation", async () => {
      const recommendation: FeedbackRecommendation = {
        id: "rec-1",
        type: "agent_update",
        priority: VerificationPriority.HIGH,
        description: "Update agent performance profile",
        action: {
          targetEntity: "agent-1",
          operation: "update_performance_profile",
          parameters: { newWeight: 0.8 },
        },
        expectedImpact: {
          metric: "performance_score",
          improvementPercent: 10,
          timeToEffect: "1 hour",
        },
        riskAssessment: {
          riskLevel: "low",
          rollbackPlan: "Revert weights",
          monitoringRequired: true,
        },
        implementationStatus: "pending",
      };

      const result = await manager.applyRecommendation(recommendation);

      expect(mockImprovementEngine.applyRecommendation).toHaveBeenCalledWith(
        recommendation
      );
      expect(result).toBe(true);
    });

    it("should apply multiple recommendations", async () => {
      const recommendations: FeedbackRecommendation[] = [
        {
          id: "rec-1",
          type: "routing_adjustment",
          priority: VerificationPriority.MEDIUM,
          description: "Adjust routing weights",
          action: {
            targetEntity: "router-1",
            operation: "reduce_routing_weight",
            parameters: { reduction_factor: 0.8 },
          },
          expectedImpact: {
            metric: "load_balance",
            improvementPercent: 15,
            timeToEffect: "30 minutes",
          },
          riskAssessment: {
            riskLevel: "medium",
            rollbackPlan: "Restore weights",
            monitoringRequired: true,
          },
          implementationStatus: "pending",
        },
      ];

      const results = await manager.applyRecommendations(recommendations);

      expect(mockImprovementEngine.applyRecommendations).toHaveBeenCalledWith(
        recommendations
      );
      expect(results).toBeDefined();
    });
  });

  describe("statistics", () => {
    it("should return comprehensive stats", () => {
      const stats = manager.getStats();

      expect(stats).toBeDefined();
      expect(stats.totalEvents).toBe(100);
      expect(stats.analysisCount).toBe(0); // Not incremented in this test
      expect(stats.averageProcessingTimeMs).toBeDefined();
      expect(stats.uptimeSeconds).toBeGreaterThan(0);
    });

    it("should clear stats", () => {
      manager.clearStats();

      expect(mockCollector.clearStats).toHaveBeenCalled();
      expect(mockImprovementEngine.clearHistory).toHaveBeenCalled();
    });
  });

  describe("health status", () => {
    it("should return healthy status when running well", () => {
      const health = manager.getHealthStatus();

      expect(health.status).toBe("healthy");
      expect(health.details.isRunning).toBe(true);
    });

    it("should return degraded status when buffer is large", () => {
      mockCollector.getStats.mockReturnValue({
        ...mockCollector.getStats(),
        bufferSize: 25, // Large buffer
      });

      const health = manager.getHealthStatus();

      expect(health.status).toBe("degraded");
    });
  });

  describe("event handling", () => {
    it("should handle feedback batch ready events", async () => {
      await manager.initialize();

      const mockBatch = [
        {
          id: "event-1",
          source: FeedbackSource.PERFORMANCE_METRICS,
          type: FeedbackType.NUMERIC_METRIC,
          entityId: "agent-1",
          entityType: "agent",
          timestamp: new Date().toISOString(),
          value: { latencyMs: 100 },
          context: {},
        },
      ];

      // Simulate batch ready event
      mockCollector.emit("feedback:batch-ready", mockBatch);

      // Wait for event processing
      await new Promise((resolve) => setTimeout(resolve, 10));

      expect(mockCollector.processBatch).toHaveBeenCalledWith(mockBatch);
    });

    it("should emit feedback collected events", async () => {
      await manager.initialize();

      const mockEvent = {
        id: "event-1",
        source: FeedbackSource.PERFORMANCE_METRICS,
        type: FeedbackType.NUMERIC_METRIC,
        entityId: "agent-1",
        entityType: "agent",
        timestamp: new Date().toISOString(),
        value: { latencyMs: 100 },
        context: {},
      };

      let collectedEvent: any = null;
      manager.on("feedback:collected", (event) => {
        collectedEvent = event;
      });

      mockCollector.emit("feedback:collected", mockEvent);

      expect(collectedEvent).toEqual(mockEvent);
    });
  });
});
