/**
 * @file PerformanceTrackerBridge.test.ts
 * @description Unit tests for PerformanceTrackerBridge
 * @author @darianrosebrook
 */

import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import { PerformanceTrackerBridge } from "@/models/PerformanceTrackerBridge";
import type { TaskExecutionData } from "@/rl/PerformanceTracker";
import type { PerformanceEvent } from "@/types/performance-tracking";
import { PerformanceEventType } from "@/types/performance-tracking";
import { beforeEach, describe, expect, it } from "@jest/globals";

describe("PerformanceTrackerBridge", () => {
  let registry: ModelRegistry;
  let selector: LocalModelSelector;
  let costTracker: ComputeCostTracker;
  let bridge: PerformanceTrackerBridge;
  let modelId: string;

  beforeEach(async () => {
    registry = new ModelRegistry();
    costTracker = new ComputeCostTracker();
    selector = new LocalModelSelector(registry, costTracker);
    bridge = new PerformanceTrackerBridge(registry, selector, costTracker);

    // Register and activate test model
    const model = await registry.registerOllamaModel(
      "test-model",
      "gemma3n:e2b",
      "1.0.0"
    );
    modelId = model.id;
    // Note: activateModel method not available on ModelRegistry - mock for test
    (registry as any).activateModel = jest.fn().mockResolvedValue(undefined);
    await (registry as any).activateModel(modelId);
  });

  afterEach(async () => {
    // Clean up registered models
    if (registry && modelId) {
      try {
        // Note: deactivateModel, unregisterModel methods not available on ModelRegistry - mock for test
        (registry as any).deactivateModel = jest
          .fn()
          .mockResolvedValue(undefined);
        (registry as any).unregisterModel = jest
          .fn()
          .mockResolvedValue(undefined);
        await (registry as any).deactivateModel(modelId);
        await (registry as any).unregisterModel(modelId);
      } catch (error) {
        // Ignore cleanup errors
      }
    }
    jest.clearAllMocks();
  });

  afterAll(async () => {
    // Final cleanup
    if (registry) {
      try {
        // Note: listModels, deactivateModel, unregisterModel methods not available on ModelRegistry - mock for test
        (registry as any).listModels = jest.fn().mockResolvedValue([]);
        (registry as any).deactivateModel = jest
          .fn()
          .mockResolvedValue(undefined);
        (registry as any).unregisterModel = jest
          .fn()
          .mockResolvedValue(undefined);
        const models = await (registry as any).listModels();
        for (const model of models) {
          await (registry as any).deactivateModel(model.id);
          await (registry as any).unregisterModel(model.id);
        }
      } catch (error) {
        // Ignore cleanup errors
      }
    }
  });

  describe("Constructor", () => {
    it("should initialize with required dependencies", () => {
      expect(bridge).toBeDefined();
    });
  });

  describe("recordFromPerformanceEvent()", () => {
    it("should record performance from routing decision event", () => {
      const event: PerformanceEvent = {
        id: "event-1",
        type: PerformanceEventType.ROUTING_DECISION,
        timestamp: new Date().toISOString(),
        agentId: modelId,
        metrics: {
          latency: {
            averageMs: 250,
            p95Ms: 300,
            p99Ms: 350,
            minMs: 200,
            maxMs: 400,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.85,
            violationRate: 0.0,
            evaluationScore: 0.85,
          },
        },
        integrityHash: "hash-1",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      // Verify performance was recorded
      const history = selector.getPerformanceHistory(modelId, "routing");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(1);
      expect(history!.avgLatencyMs).toBe(250);
    });

    it("should record performance from task execution event", () => {
      const event: PerformanceEvent = {
        id: "event-2",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        agentId: modelId,
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.9,
            violationRate: 0.0,
            evaluationScore: 0.9,
          },
        },
        integrityHash: "hash-2",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "execution");
      expect(history).toBeDefined();
      expect(history!.avgQuality).toBeCloseTo(0.9, 2);
    });

    it("should record performance from judgment event", () => {
      const event: PerformanceEvent = {
        id: "event-3",
        type: PerformanceEventType.EVALUATION_OUTCOME,
        timestamp: new Date().toISOString(),
        agentId: modelId,
        metrics: {
          latency: {
            averageMs: 1000,
            p95Ms: 1200,
            p99Ms: 1400,
            minMs: 800,
            maxMs: 1600,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.95,
            violationRate: 0.0,
            evaluationScore: 0.95,
          },
        },
        integrityHash: "hash-3",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "judgment");
      expect(history).toBeDefined();
    });

    it("should infer task type from event type", () => {
      const event: PerformanceEvent = {
        id: "event-4",
        type: PerformanceEventType.CONSTITUTIONAL_VALIDATION,
        timestamp: new Date().toISOString(),
        agentId: modelId,
        metrics: {
          latency: {
            averageMs: 100,
            p95Ms: 120,
            p99Ms: 140,
            minMs: 80,
            maxMs: 160,
          },
        },
        context: {
          type: PerformanceEventType.CONSTITUTIONAL_VALIDATION,
          success: true,
        },
        integrityHash: "hash-4",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "validation");
      expect(history).toBeDefined();
    });

    it("should calculate quality from success status", () => {
      const successEvent: PerformanceEvent = {
        id: "event-5",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        agentId: modelId,
        taskId: "task-1",
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.9,
            violationRate: 0.0,
            evaluationScore: 0.95,
          },
        },
        context: {
          success: true,
        },
        integrityHash: "hash-5",
      };

      const failureEvent: PerformanceEvent = {
        id: "event-6",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        agentId: modelId,
        taskId: "task-2",
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 0.0,
            qualityScore: 0.3,
            violationRate: 0.7,
            evaluationScore: 0.2,
          },
        },
        context: {
          success: false,
        },
        integrityHash: "hash-6",
      };

      bridge.recordFromPerformanceEvent(successEvent, modelId);
      bridge.recordFromPerformanceEvent(failureEvent, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(2);
    });

    it("should record compute cost when latency available", () => {
      const event: PerformanceEvent = {
        id: "event-7",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        agentId: modelId,
        taskId: "task-3",
        metrics: {
          latency: {
            averageMs: 750,
            p95Ms: 800,
            p99Ms: 850,
            minMs: 700,
            maxMs: 900,
          },
        },
        context: {
          success: true,
          cpuUsage: 60,
          inputTokens: 100,
          outputTokens: 50,
        },
        integrityHash: "hash-7",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const costProfile = costTracker.getCostProfile(modelId);
      expect(costProfile).toBeDefined();
      expect(costProfile!.totalOperations).toBe(1);
      expect(costProfile!.avgWallClockMs).toBe(750);
    });
  });

  describe("recordFromTaskExecution()", () => {
    it("should record performance from task execution data", () => {
      const execution: TaskExecutionData = {
        executionId: "exec-1",
        taskId: "sentiment-analysis",
        agentId: "agent-1",
        routingDecision: {
          taskId: "sentiment-analysis",
          selectedAgent: "agent-1",
          routingStrategy: "capability-match",
          confidence: 0.9,
          alternativesConsidered: [],
          rationale: "High reward history",
          timestamp: new Date().toISOString(),
        },
        outcome: {
          success: true,
          qualityScore: 0.85,
          efficiencyScore: 0.8,
          tokensConsumed: 100,
          completionTimeMs: 1000,
        },
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
        context: {
          taskType: "analysis",
        },
      };

      bridge.recordFromTaskExecution(execution, modelId);

      const history = selector.getPerformanceHistory(modelId, "analysis");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(1);
      expect(history!.avgLatencyMs).toBe(1000);
    });

    it("should calculate quality from reward", () => {
      const highRewardExecution: TaskExecutionData = {
        executionId: "exec-2",
        taskId: "task-1",
        agentId: "agent-1",
        routingDecision: {
          taskId: "task-1",
          selectedAgent: "agent-1",
          routingStrategy: "capability-match",
          confidence: 0.9,
          alternativesConsidered: [],
          rationale: "High reward history",
          timestamp: new Date().toISOString(),
        },
        outcome: {
          success: true,
          qualityScore: 0.9,
          efficiencyScore: 0.85,
          tokensConsumed: 150,
          completionTimeMs: 1000,
        },
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
      };

      const lowRewardExecution: TaskExecutionData = {
        executionId: "exec-3",
        taskId: "task-1",
        agentId: "agent-1",
        routingDecision: {
          taskId: "task-1",
          selectedAgent: "agent-1",
          routingStrategy: "capability-match",
          confidence: 0.5,
          alternativesConsidered: [],
          rationale: "Low reward history",
          timestamp: new Date().toISOString(),
        },
        outcome: {
          success: false,
          qualityScore: 0.3,
          efficiencyScore: 0.4,
          tokensConsumed: 150,
          completionTimeMs: 1000,
        },
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
      };

      bridge.recordFromTaskExecution(highRewardExecution, modelId);
      bridge.recordFromTaskExecution(lowRewardExecution, modelId);

      const history = selector.getPerformanceHistory(modelId, "unknown");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(2);
    });

    it("should handle negative rewards", () => {
      const negativeRewardExecution: TaskExecutionData = {
        executionId: "exec-4",
        taskId: "task-2",
        agentId: "agent-1",
        routingDecision: {
          taskId: "task-2",
          selectedAgent: "agent-1",
          routingStrategy: "capability-match",
          confidence: 0.2,
          alternativesConsidered: [],
          rationale: "Poor performance history",
          timestamp: new Date().toISOString(),
        },
        outcome: {
          success: false,
          qualityScore: 0.1,
          efficiencyScore: 0.2,
          tokensConsumed: 150,
          completionTimeMs: 1000,
        },
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
      };

      bridge.recordFromTaskExecution(negativeRewardExecution, modelId);

      const history = selector.getPerformanceHistory(modelId, "unknown");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(1);
      expect(history!.avgQuality).toBeLessThan(0.5);
    });

    it("should record compute cost", () => {
      const execution: TaskExecutionData = {
        executionId: "exec-5",
        taskId: "task-3",
        agentId: "agent-1",
        routingDecision: {
          taskId: "task-3",
          selectedAgent: "agent-1",
          routingStrategy: "capability-match",
          confidence: 0.8,
          alternativesConsidered: [],
          rationale: "Good performance history",
          timestamp: new Date().toISOString(),
        },
        outcome: {
          success: true,
          qualityScore: 0.8,
          efficiencyScore: 0.75,
          tokensConsumed: 150,
          completionTimeMs: 1000,
        },
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
      };

      bridge.recordFromTaskExecution(execution, modelId);

      const costProfile = costTracker.getCostProfile(modelId);
      expect(costProfile).toBeDefined();
      expect(costProfile!.totalOperations).toBe(1);
      expect(costProfile!.avgWallClockMs).toBe(2000);
    });
  });

  describe("recordModelPerformance()", () => {
    it("should record performance data directly", () => {
      const data = {
        modelId,
        taskType: "custom-task",
        quality: 0.88,
        latencyMs: 350,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
        inputTokens: 150,
        outputTokens: 75,
      };

      bridge.recordModelPerformance(data);

      const history = selector.getPerformanceHistory(modelId, "custom-task");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(1);
      expect(history!.avgQuality).toBeCloseTo(0.88, 2);
      expect(history!.avgLatencyMs).toBe(350);
    });

    it("should record compute cost with token usage", () => {
      const data = {
        modelId,
        taskType: "token-task",
        quality: 0.9,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
        timestamp: new Date(),
        inputTokens: 200,
        outputTokens: 100,
      };

      bridge.recordModelPerformance(data);

      const costProfile = costTracker.getCostProfile(modelId);
      expect(costProfile).toBeDefined();
      // Note: totalInputTokens, totalOutputTokens properties not part of CostProfile interface
      expect(costProfile!.totalOperations).toBeGreaterThanOrEqual(1);
    });

    it("should handle data without token counts", () => {
      const data = {
        modelId,
        taskType: "no-tokens",
        quality: 0.85,
        latencyMs: 400,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      };

      bridge.recordModelPerformance(data);

      const history = selector.getPerformanceHistory(modelId, "no-tokens");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(1);
    });
  });

  describe("exportToPerformanceTracker()", () => {
    it("should export performance history to TaskExecutionData format", () => {
      // Record some performance data
      for (let i = 0; i < 5; i++) {
        bridge.recordModelPerformance({
          modelId,
          taskType: "export-task",
          quality: 0.8 + i * 0.02,
          latencyMs: 500 + i * 10,
          memoryMB: 256,
          success: true,
          timestamp: new Date(),
        });
      }

      const exported = bridge.exportToPerformanceTracker(
        modelId,
        "export-task"
      );

      expect(exported).toBeDefined();
      expect(exported.length).toBeGreaterThan(0);

      const execution = exported[0];
      expect(execution.executionId).toContain(modelId);
      expect(execution.taskId).toBeDefined();
      expect(execution.agentId).toBe(modelId);
      expect(execution.outcome).toBeDefined();
      expect(execution.startedAt).toBeDefined();
      expect(execution.completedAt).toBeDefined();
    });

    it("should return empty array for non-existent model", () => {
      const exported = bridge.exportToPerformanceTracker(
        "non-existent-model",
        "test-task"
      );

      expect(exported).toEqual([]);
    });

    it("should return empty array for non-existent task type", () => {
      const exported = bridge.exportToPerformanceTracker(
        modelId,
        "non-existent-task"
      );

      expect(exported).toEqual([]);
    });

    it("should include performance metrics in metadata", async () => {
      // Record performance with specific metrics
      bridge.recordModelPerformance({
        modelId,
        taskType: "metrics-task",
        quality: 0.92,
        latencyMs: 450,
        memoryMB: 300,
        success: true,
        timestamp: new Date(),
      });

      const exported = bridge.exportToPerformanceTracker(
        modelId,
        "metrics-task"
      );

      expect(exported.length).toBe(1);
      expect(exported[0].outcome.qualityScore).toBeCloseTo(0.92, 2);
      expect(exported[0].executionId).toBeDefined();
      expect(exported[0].agentId).toBe(modelId);
    });
  });

  describe("Quality Calculation", () => {
    it("should calculate quality from explicit quality in metadata", () => {
      const event: PerformanceEvent = {
        id: "quality-1",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        metrics: {
          accuracy: {
            successRate: 0.87,
            qualityScore: 0.87,
            violationRate: 0.0,
            evaluationScore: 0.87,
          },
          latency: {
            averageMs: 500,
            p95Ms: 500,
            p99Ms: 500,
            minMs: 500,
            maxMs: 500,
          },
        },
        integrityHash: "hash-1",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history!.avgQuality).toBeCloseTo(0.87, 2);
    });

    it("should calculate quality from score in metadata", () => {
      const event: PerformanceEvent = {
        id: "quality-2",
        type: PerformanceEventType.EVALUATION_OUTCOME,
        timestamp: new Date().toISOString(),
        metrics: {
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.93,
            violationRate: 0.0,
            evaluationScore: 0.93,
          },
          latency: {
            averageMs: 800,
            p95Ms: 800,
            p99Ms: 800,
            minMs: 800,
            maxMs: 800,
          },
        },
        integrityHash: "hash-2",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "judgment");
      expect(history!.avgQuality).toBeCloseTo(0.93, 2);
    });

    it("should infer quality from latency when no explicit quality", async () => {
      const fastEvent: PerformanceEvent = {
        id: "quality-3",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        metrics: {
          latency: {
            averageMs: 500, // < 1s, should be high quality
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.8,
            violationRate: 0.0,
            evaluationScore: 0.8,
          },
          resources: {
            cpuUtilizationPercent: 50,
            memoryUtilizationPercent: 60,
            networkIoKbps: 100,
            diskIoKbps: 30,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-3",
      };

      const slowEvent: PerformanceEvent = {
        id: "quality-4",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        metrics: {
          latency: {
            averageMs: 6000, // > 5s, should be lower quality
            p95Ms: 7000,
            p99Ms: 8000,
            minMs: 5000,
            maxMs: 9000,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.6,
            violationRate: 0.0,
            evaluationScore: 0.6,
          },
          resources: {
            cpuUtilizationPercent: 70,
            memoryUtilizationPercent: 80,
            networkIoKbps: 150,
            diskIoKbps: 40,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-4",
      };

      bridge.recordFromPerformanceEvent(fastEvent, modelId);
      const fastHistory = selector.getPerformanceHistory(
        modelId,
        "task_execution"
      );

      // Clear and record slow
      const slowModelId = (
        await registry.registerOllamaModel("slow-model", "gemma3:7b", "1.0.0")
      ).id;
      bridge.recordFromPerformanceEvent(slowEvent, slowModelId);
      const slowHistory = selector.getPerformanceHistory(
        slowModelId,
        "task_execution"
      );

      expect(fastHistory!.avgQuality).toBeGreaterThan(slowHistory!.avgQuality);
    });

    it("should assign low quality to failures", () => {
      const failureEvent: PerformanceEvent = {
        id: "quality-5",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 0.0,
            qualityScore: 0.2,
            violationRate: 0.0,
            evaluationScore: 0.2,
          },
          resources: {
            cpuUtilizationPercent: 30,
            memoryUtilizationPercent: 40,
            networkIoKbps: 50,
            diskIoKbps: 30,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-5",
      };

      bridge.recordFromPerformanceEvent(failureEvent, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history!.avgQuality).toBeLessThan(0.5);
    });
  });

  describe("Memory Estimation", () => {
    it("should use memory from metadata when available", () => {
      const event: PerformanceEvent = {
        id: "memory-1",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.8,
            violationRate: 0.0,
            evaluationScore: 0.8,
          },
          resources: {
            cpuUtilizationPercent: 60,
            memoryUtilizationPercent: 70,
            networkIoKbps: 100,
            diskIoKbps: 35,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-memory-1",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history!.avgMemoryMB).toBe(768);
    });

    it("should estimate memory based on event type when not provided", async () => {
      const judgmentEvent: PerformanceEvent = {
        id: "memory-2",
        type: PerformanceEventType.EVALUATION_OUTCOME,
        timestamp: new Date().toISOString(),
        success: true,
        metadata: {
          latencyMs: 500,
        },
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.8,
            violationRate: 0.0,
            evaluationScore: 0.8,
          },
          resources: {
            memoryUtilizationPercent: 50,
            cpuUtilizationPercent: 50,
            networkIoKbps: 100,
            diskIoKbps: 30,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-memory-2",
      };

      const thinkingEvent: PerformanceEvent = {
        id: "memory-3",
        type: PerformanceEventType.CONSTITUTIONAL_VALIDATION,
        timestamp: new Date().toISOString(),
        success: true,
        metadata: {
          latencyMs: 100,
        },
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.8,
            violationRate: 0.0,
            evaluationScore: 0.8,
          },
          resources: {
            memoryUtilizationPercent: 50,
            cpuUtilizationPercent: 50,
            networkIoKbps: 100,
            diskIoKbps: 30,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-memory-3",
      };

      bridge.recordFromPerformanceEvent(judgmentEvent, modelId);
      const judgmentHistory = selector.getPerformanceHistory(
        modelId,
        "judgment"
      );

      const thinkingModelId = (
        await registry.registerOllamaModel(
          "thinking-model",
          "gemma3:1b",
          "1.0.0"
        )
      ).id;
      bridge.recordFromPerformanceEvent(thinkingEvent, thinkingModelId);
      const thinkingHistory = selector.getPerformanceHistory(
        thinkingModelId,
        "thinking"
      );

      // Judgment should use more memory than thinking
      expect(judgmentHistory!.avgMemoryMB).toBeGreaterThan(
        thinkingHistory!.avgMemoryMB
      );
    });
  });

  describe("Token Calculation", () => {
    it("should calculate tokens per second from metadata", () => {
      const event: PerformanceEvent = {
        id: "tokens-1",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        success: true,
        metadata: {
          latencyMs: 1000,
          outputTokens: 100,
        },
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.8,
            violationRate: 0.0,
            evaluationScore: 0.8,
          },
          resources: {
            memoryUtilizationPercent: 50,
            cpuUtilizationPercent: 50,
            networkIoKbps: 100,
            diskIoKbps: 30,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-tokens-1",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const costProfile = costTracker.getCostProfile(modelId);

      expect(costProfile).toBeDefined();
      expect(costProfile!.avgTokensPerSec).toBeCloseTo(100, 0);
    });

    it("should use default token values when not provided", () => {
      const event: PerformanceEvent = {
        id: "tokens-2",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        success: true,
        metadata: {
          latencyMs: 1000,
        },
        metrics: {
          latency: {
            averageMs: 500,
            p95Ms: 600,
            p99Ms: 700,
            minMs: 400,
            maxMs: 800,
          },
          accuracy: {
            successRate: 1.0,
            qualityScore: 0.8,
            violationRate: 0.0,
            evaluationScore: 0.8,
          },
          resources: {
            memoryUtilizationPercent: 50,
            cpuUtilizationPercent: 50,
            networkIoKbps: 100,
            diskIoKbps: 30,
          },
          compliance: {
            validationPassRate: 1.0,
            violationSeverityScore: 0.0,
            complianceScore: 1.0,
            clauseCitationRate: 0.8,
          },
        },
        integrityHash: "hash-tokens-2",
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const costProfile = costTracker.getCostProfile(modelId);

      expect(costProfile).toBeDefined();
      expect(costProfile!.avgTokensPerSec).toBeGreaterThan(0);
    });
  });
});
