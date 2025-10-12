/**
 * Performance Tracker Unit Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { PerformanceTracker } from "../../../src/rl/PerformanceTracker";
import { RoutingDecision, TaskOutcome } from "../../../src/types/agentic-rl";

describe("PerformanceTracker", () => {
  let tracker: PerformanceTracker;
  let mockRoutingDecision: RoutingDecision;
  let mockTaskOutcome: TaskOutcome;

  beforeEach(() => {
    tracker = new PerformanceTracker();
    tracker.startCollection();

    mockRoutingDecision = {
      taskId: "task-123",
      selectedAgent: "agent-1",
      routingStrategy: "multi-armed-bandit",
      confidence: 0.8,
      alternativesConsidered: [
        { agentId: "agent-1", score: 0.9, reason: "High success rate" },
        { agentId: "agent-2", score: 0.7, reason: "Medium success rate" },
      ],
      rationale: "Selected best performing agent",
      timestamp: new Date().toISOString(),
    };

    mockTaskOutcome = {
      success: true,
      qualityScore: 0.85,
      efficiencyScore: 0.9,
      tokensConsumed: 1500,
      completionTimeMs: 2500,
    };
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const tracker = new PerformanceTracker();
      const config = tracker.getConfig();

      expect(config.enabled).toBe(true);
      expect(config.maxEventsInMemory).toBe(10000);
      expect(config.anonymizeData).toBe(true);
    });

    it("should override default config", () => {
      const customConfig = {
        enabled: false,
        maxEventsInMemory: 5000,
        anonymizeData: false,
      };

      const tracker = new PerformanceTracker(customConfig);
      const config = tracker.getConfig();

      expect(config.enabled).toBe(false);
      expect(config.maxEventsInMemory).toBe(5000);
      expect(config.anonymizeData).toBe(false);
    });
  });

  describe("collection control", () => {
    it("should start and stop collection", () => {
      expect(tracker.isActive()).toBe(true);

      tracker.stopCollection();
      expect(tracker.isActive()).toBe(false);

      tracker.startCollection();
      expect(tracker.isActive()).toBe(true);
    });

    it("should not collect when disabled", async () => {
      tracker.updateConfig({ enabled: false });

      await tracker.recordRoutingDecision(mockRoutingDecision);
      await tracker.recordEvaluationOutcome("task-123", {
        passed: true,
        score: 0.85,
      });

      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBe(0);
      expect(stats.totalEvaluationOutcomes).toBe(0);
    });
  });

  describe("routing decision recording", () => {
    it("should record routing decisions", async () => {
      await tracker.recordRoutingDecision(mockRoutingDecision);

      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBe(1);

      const trainingData = tracker.exportTrainingData();
      expect(trainingData).toHaveLength(1);
      expect(trainingData[0].type).toBe("routing-decision");
    });

    it("should anonymize routing decisions when enabled", async () => {
      await tracker.recordRoutingDecision(mockRoutingDecision);

      const trainingData = tracker.exportTrainingData();
      const event = trainingData[0];

      // Should anonymize agent IDs
      expect((event.data as any).selectedAgent).not.toBe("agent-1");
      expect(typeof (event.data as any).selectedAgent).toBe("string");
    });

    it("should not anonymize when disabled", async () => {
      tracker.updateConfig({ anonymizeData: false });
      await tracker.recordRoutingDecision(mockRoutingDecision);

      const trainingData = tracker.exportTrainingData();
      const event = trainingData[0];

      expect((event.data as any).selectedAgent).toBe("agent-1");
    });
  });

  describe("task execution tracking", () => {
    it("should track task execution lifecycle", async () => {
      const executionId = tracker.startTaskExecution(
        "task-123",
        "agent-1",
        mockRoutingDecision
      );

      expect(executionId).toContain("task-123");

      await tracker.completeTaskExecution(executionId, mockTaskOutcome);

      const stats = tracker.getStats();
      expect(stats.totalTaskExecutions).toBe(1);
      expect(stats.averageCompletionTimeMs).toBeGreaterThanOrEqual(0);
      expect(stats.overallSuccessRate).toBe(1); // 100% success
    });

    it("should handle task execution context", () => {
      const context = { complexity: "high", priority: "urgent" };

      tracker.startTaskExecution(
        "task-456",
        "agent-2",
        mockRoutingDecision,
        context
      );

      // Context should be stored (implementation detail)
      expect(tracker.getStats().totalTaskExecutions).toBe(0); // Not completed yet
    });

    it("should calculate correct statistics", async () => {
      // Record multiple task executions
      const execution1 = tracker.startTaskExecution(
        "task-1",
        "agent-1",
        mockRoutingDecision
      );
      await tracker.completeTaskExecution(execution1, {
        ...mockTaskOutcome,
        success: true,
      });

      const execution2 = tracker.startTaskExecution(
        "task-2",
        "agent-2",
        mockRoutingDecision
      );
      await tracker.completeTaskExecution(execution2, {
        ...mockTaskOutcome,
        success: false,
        completionTimeMs: 3000,
      });

      const stats = tracker.getStats();
      expect(stats.totalTaskExecutions).toBe(2);
      expect(stats.overallSuccessRate).toBe(0.5); // 50% success
      expect(stats.averageCompletionTimeMs).toBeGreaterThanOrEqual(0);
    });
  });

  describe("evaluation outcome recording", () => {
    it("should record evaluation outcomes", async () => {
      const evaluation = {
        passed: true,
        score: 0.85,
        rubricScores: { quality: 0.8, efficiency: 0.9 },
        feedback: "Good work",
      };

      await tracker.recordEvaluationOutcome("task-123", evaluation);

      const stats = tracker.getStats();
      expect(stats.totalEvaluationOutcomes).toBe(1);

      const trainingData = tracker.exportTrainingData();
      expect(trainingData).toHaveLength(1);
      expect(trainingData[0].type).toBe("evaluation-outcome");
    });

    it("should anonymize evaluation data", async () => {
      await tracker.recordEvaluationOutcome("task-123", {
        passed: true,
        score: 0.85,
      });

      const trainingData = tracker.exportTrainingData();
      const event = trainingData[0];

      // Should anonymize task IDs
      expect((event.data as any).taskId).not.toBe("task-123");
    });
  });

  describe("data export", () => {
    it("should export all training data", async () => {
      await tracker.recordRoutingDecision(mockRoutingDecision);
      await tracker.recordEvaluationOutcome("task-123", {
        passed: true,
        score: 0.8,
      });

      const trainingData = tracker.exportTrainingData();
      expect(trainingData).toHaveLength(2);

      const types = trainingData.map((d) => d.type);
      expect(types).toContain("routing-decision");
      expect(types).toContain("evaluation-outcome");
    });

    it("should export data since timestamp", async () => {
      await tracker.recordRoutingDecision(mockRoutingDecision);

      // Wait a bit to ensure different timestamps
      await new Promise((resolve) => setTimeout(resolve, 10));

      const filterTime = new Date().toISOString();

      // Wait a bit more
      await new Promise((resolve) => setTimeout(resolve, 10));

      await tracker.recordEvaluationOutcome("task-456", {
        passed: true,
        score: 0.9,
      });

      const recentData = tracker.exportTrainingData(filterTime);
      expect(recentData).toHaveLength(1);
      expect(recentData[0].type).toBe("evaluation-outcome");
    });

    it("should return copy of data", () => {
      // Ensure exported data is not modifiable
      const trainingData = tracker.exportTrainingData();
      expect(() => {
        (trainingData as any).push({ type: "test" });
      }).not.toThrow();

      expect(tracker.exportTrainingData()).toHaveLength(0);
    });
  });

  describe("statistics", () => {
    it("should provide accurate statistics", () => {
      const stats = tracker.getStats();

      expect(stats.totalRoutingDecisions).toBe(0);
      expect(stats.totalTaskExecutions).toBe(0);
      expect(stats.totalEvaluationOutcomes).toBe(0);
      expect(stats.overallSuccessRate).toBe(0);
      expect(stats.averageCompletionTimeMs).toBe(0);
      expect(typeof stats.collectionStartedAt).toBe("string");
      expect(typeof stats.lastUpdatedAt).toBe("string");
    });

    it("should update statistics over time", async () => {
      const initialStats = tracker.getStats();

      await tracker.recordRoutingDecision(mockRoutingDecision);
      const updatedStats = tracker.getStats();

      expect(updatedStats.totalRoutingDecisions).toBe(
        initialStats.totalRoutingDecisions + 1
      );
      expect(
        new Date(updatedStats.lastUpdatedAt).getTime()
      ).toBeGreaterThanOrEqual(new Date(initialStats.lastUpdatedAt).getTime());
    });
  });

  describe("data management", () => {
    it("should clear all data", async () => {
      await tracker.recordRoutingDecision(mockRoutingDecision);
      await tracker.recordEvaluationOutcome("task-123", {
        passed: true,
        score: 0.8,
      });

      expect(tracker.getStats().totalRoutingDecisions).toBe(1);

      tracker.clearData();

      expect(tracker.getStats().totalRoutingDecisions).toBe(0);
      expect(tracker.exportTrainingData()).toHaveLength(0);
    });

    it("should enforce memory limits", () => {
      tracker.updateConfig({ maxEventsInMemory: 2 });

      // Add more events than the limit
      for (let i = 0; i < 5; i++) {
        tracker.recordEvent({
          type: "test-event",
          timestamp: new Date().toISOString(),
          data: { index: i },
        });
      }

      const trainingData = tracker.exportTrainingData();
      expect(trainingData.length).toBeLessThanOrEqual(2);
    });
  });

  describe("anonymization", () => {
    it("should anonymize IDs with consistent hashing", () => {
      const tracker = new PerformanceTracker({ anonymizeData: true });

      // Same input should produce same anonymized output
      const hash1 = (tracker as any).simpleHash("agent-1");
      const hash2 = (tracker as any).simpleHash("agent-1");

      expect(hash1).toBe(hash2);
      expect(hash1).not.toBe("agent-1");
    });

    it("should anonymize nested objects", () => {
      const tracker = new PerformanceTracker({ anonymizeData: true });

      const original = {
        agentId: "agent-123",
        metadata: {
          taskId: "task-456",
          otherData: "unchanged",
        },
      };

      const anonymized = (tracker as any).anonymizeDataIfNeeded(original);

      expect((anonymized as any).agentId).not.toBe("agent-123");
      expect((anonymized as any).metadata.taskId).not.toBe("task-456");
      expect((anonymized as any).metadata.otherData).toBe("unchanged");
    });
  });

  describe("general event recording", () => {
    it("should record custom events", () => {
      const customEvent = {
        type: "custom-metric",
        timestamp: new Date().toISOString(),
        data: {
          metricName: "response-time",
          value: 150,
          unit: "ms",
        },
      };

      tracker.recordEvent(customEvent);

      const trainingData = tracker.exportTrainingData();
      expect(trainingData).toHaveLength(1);
      expect(trainingData[0].type).toBe("custom-metric");
    });
  });

  describe("edge cases", () => {
    it("should handle empty data gracefully", () => {
      expect(tracker.exportTrainingData()).toHaveLength(0);
      expect(tracker.getStats().totalRoutingDecisions).toBe(0);
    });

    it("should handle invalid execution IDs", async () => {
      // Should not throw for invalid execution ID
      await expect(
        tracker.completeTaskExecution("invalid-id", mockTaskOutcome)
      ).resolves.not.toThrow();
    });

    it("should handle concurrent operations", async () => {
      const promises = [];

      // Record multiple events concurrently
      for (let i = 0; i < 10; i++) {
        promises.push(
          tracker.recordRoutingDecision({
            ...mockRoutingDecision,
            taskId: `task-${i}`,
          })
        );
      }

      await Promise.all(promises);

      expect(tracker.getStats().totalRoutingDecisions).toBe(10);
    });
  });

  describe("benchmarking integration", () => {
    beforeEach(() => {
      tracker = new PerformanceTracker();
      tracker.startCollection();
    });

    it("should convert legacy TaskOutcome to comprehensive metrics", () => {
      const executionId = tracker.startTaskExecution("task-123", "agent-1");

      const legacyOutcome: TaskOutcome = {
        success: true,
        qualityScore: 0.85,
        efficiencyScore: 0.9,
        tokensConsumed: 1500,
        completionTimeMs: 2500,
      };

      const conversionMethod = (tracker as any)
        .convertOutcomeToPerformanceMetrics;
      const metrics = conversionMethod(legacyOutcome, 2500);

      expect(metrics.latency.averageMs).toBe(2500);
      expect(metrics.accuracy.successRate).toBe(1);
      expect(metrics.accuracy.qualityScore).toBe(0.85);
      expect(metrics.cost.efficiencyScore).toBe(0.85);
    });

    it("should handle failed task outcomes", () => {
      const executionId = tracker.startTaskExecution("task-123", "agent-1");

      const failedOutcome: TaskOutcome = {
        success: false,
        qualityScore: 0.3,
        efficiencyScore: 0.4,
        tokensConsumed: 800,
        completionTimeMs: 1500,
      };

      const conversionMethod = (tracker as any)
        .convertOutcomeToPerformanceMetrics;
      const metrics = conversionMethod(failedOutcome, 1500);

      expect(metrics.accuracy.successRate).toBe(0);
      expect(metrics.accuracy.violationRate).toBe(1);
      expect(metrics.compliance.validationPassRate).toBe(0);
    });

    it("should gracefully handle missing data collector", () => {
      // Remove data collector to test graceful degradation
      (tracker as any).dataCollector = null;

      const executionId = tracker.startTaskExecution("task-123", "agent-1");

      const outcome: TaskOutcome = {
        success: true,
        qualityScore: 0.8,
        efficiencyScore: 0.85,
        tokensConsumed: 1200,
        completionTimeMs: 2000,
      };

      // Should not throw error even without data collector
      expect(() => {
        tracker.completeTaskExecution(executionId, outcome);
      }).not.toThrow();
    });

    it("should integrate with DataCollector for comprehensive tracking", async () => {
      const mockDataCollector = {
        recordTaskStart: jest.fn(),
        recordTaskCompletion: jest.fn(),
        recordRoutingDecision: jest.fn(),
        startCollection: jest.fn(),
        stopCollection: jest.fn(),
      };

      (tracker as any).dataCollector = mockDataCollector;

      // Test task execution integration
      const executionId = tracker.startTaskExecution("task-123", "agent-1", {
        priority: VerificationPriority.HIGH,
      });

      expect(mockDataCollector.recordTaskStart).toHaveBeenCalledWith(
        "task-123",
        "agent-1",
        expect.objectContaining({ priority: VerificationPriority.HIGH })
      );

      // Test routing decision integration
      await tracker.recordRoutingDecision(mockRoutingDecision);

      expect(mockDataCollector.recordRoutingDecision).toHaveBeenCalledWith(
        "task-123",
        "agent-1",
        expect.any(Array),
        expect.objectContaining({
          confidence: 0.8,
          rationale: "Selected best performing agent",
        })
      );

      // Test task completion integration
      const comprehensiveMetrics = {
        latency: {
          averageMs: 2500,
          p95Ms: 3000,
          p99Ms: 3500,
          minMs: 2000,
          maxMs: 4000,
        },
        accuracy: {
          successRate: 0.9,
          qualityScore: 0.85,
          violationRate: 0.1,
          evaluationScore: 0.8,
        },
        resources: {
          cpuUtilizationPercent: 75,
          memoryUtilizationPercent: 65,
          networkIoKbps: 120,
          diskIoKbps: 60,
        },
        compliance: {
          validationPassRate: 0.95,
          violationSeverityScore: 0.05,
          clauseCitationRate: 0.9,
        },
        cost: {
          costPerTask: 0.55,
          efficiencyScore: 0.85,
          resourceWastePercent: 15,
        },
        reliability: {
          mtbfHours: 160,
          availabilityPercent: 99,
          errorRatePercent: 1,
          recoveryTimeMinutes: 7,
        },
      };

      await tracker.completeTaskExecution(executionId, mockTaskOutcome);

      expect(mockDataCollector.recordTaskCompletion).toHaveBeenCalledWith(
        "task-123",
        "agent-1",
        expect.objectContaining({
          latency: expect.any(Object),
          accuracy: expect.any(Object),
          resources: expect.any(Object),
          compliance: expect.any(Object),
          cost: expect.any(Object),
          reliability: expect.any(Object),
        }),
        expect.any(Object)
      );
    });
  });
});
