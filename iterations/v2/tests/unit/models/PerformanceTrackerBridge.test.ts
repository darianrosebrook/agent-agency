/**
 * @file PerformanceTrackerBridge.test.ts
 * @description Unit tests for PerformanceTrackerBridge
 * @author @darianrosebrook
 */

import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import { PerformanceTrackerBridge } from "@/models/PerformanceTrackerBridge";
import type {
  PerformanceEvent,
  TaskExecutionData,
} from "@/rl/PerformanceTracker";
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
    await registry.activateModel(modelId);
  });

  describe("Constructor", () => {
    it("should initialize with required dependencies", () => {
      expect(bridge).toBeDefined();
    });
  });

  describe("recordFromPerformanceEvent()", () => {
    it("should record performance from routing decision event", () => {
      const event: PerformanceEvent = {
        eventId: "event-1",
        eventType: "routing_decision",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 250,
          taskType: "routing",
          quality: 0.85,
        },
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
        eventId: "event-2",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 500,
          taskType: "execution",
          quality: 0.9,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "execution");
      expect(history).toBeDefined();
      expect(history!.avgQuality).toBeCloseTo(0.9, 2);
    });

    it("should record performance from judgment event", () => {
      const event: PerformanceEvent = {
        eventId: "event-3",
        eventType: "judgment",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 1000,
          score: 0.95,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "judgment");
      expect(history).toBeDefined();
    });

    it("should infer task type from event type", () => {
      const event: PerformanceEvent = {
        eventId: "event-4",
        eventType: "thinking_budget",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 100,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "thinking");
      expect(history).toBeDefined();
    });

    it("should calculate quality from success status", () => {
      const successEvent: PerformanceEvent = {
        eventId: "event-5",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 500,
        },
      };

      const failureEvent: PerformanceEvent = {
        eventId: "event-6",
        eventType: "task_execution",
        timestamp: new Date(),
        success: false,
        metadata: {
          latencyMs: 500,
        },
      };

      bridge.recordFromPerformanceEvent(successEvent, modelId);
      bridge.recordFromPerformanceEvent(failureEvent, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history).toBeDefined();
      expect(history!.samples).toBe(2);
    });

    it("should record compute cost when latency available", () => {
      const event: PerformanceEvent = {
        eventId: "event-7",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 750,
          cpuUsage: 60,
          inputTokens: 100,
          outputTokens: 50,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const costProfile = costTracker.getCostProfile(modelId);
      expect(costProfile).toBeDefined();
      expect(costProfile!.totalOperations).toBe(1);
      expect(costProfile!.totalWallClockMs).toBe(750);
    });
  });

  describe("recordFromTaskExecution()", () => {
    it("should record performance from task execution data", () => {
      const execution: TaskExecutionData = {
        executionId: "exec-1",
        taskName: "sentiment-analysis",
        success: true,
        executionTimeMs: 1000,
        startedAt: new Date(),
        completedAt: new Date(),
        agentId: "agent-1",
        reward: 0.85,
        metadata: {
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
        taskName: "task-1",
        success: true,
        executionTimeMs: 500,
        startedAt: new Date(),
        completedAt: new Date(),
        agentId: "agent-1",
        reward: 0.9,
      };

      const lowRewardExecution: TaskExecutionData = {
        executionId: "exec-3",
        taskName: "task-1",
        success: true,
        executionTimeMs: 500,
        startedAt: new Date(),
        completedAt: new Date(),
        agentId: "agent-1",
        reward: 0.3,
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
        taskName: "task-2",
        success: false,
        executionTimeMs: 500,
        startedAt: new Date(),
        completedAt: new Date(),
        agentId: "agent-1",
        reward: -0.5,
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
        taskName: "task-3",
        success: true,
        executionTimeMs: 2000,
        startedAt: new Date(),
        completedAt: new Date(),
        agentId: "agent-1",
        reward: 0.8,
      };

      bridge.recordFromTaskExecution(execution, modelId);

      const costProfile = costTracker.getCostProfile(modelId);
      expect(costProfile).toBeDefined();
      expect(costProfile!.totalOperations).toBe(1);
      expect(costProfile!.totalWallClockMs).toBe(2000);
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
      expect(costProfile!.totalInputTokens).toBeGreaterThanOrEqual(200);
      expect(costProfile!.totalOutputTokens).toBeGreaterThanOrEqual(100);
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
      expect(execution.taskName).toBe("export-task");
      expect(execution.agentId).toBe(modelId);
      expect(execution.metadata).toBeDefined();
      expect(execution.metadata.avgQuality).toBeDefined();
      expect(execution.metadata.avgLatency).toBeDefined();
      expect(execution.metadata.successRate).toBeDefined();
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

    it("should include performance metrics in metadata", () => {
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
      expect(exported[0].metadata.avgQuality).toBeCloseTo(0.92, 2);
      expect(exported[0].metadata.avgLatency).toBe(450);
      expect(exported[0].metadata.samples).toBe(1);
    });
  });

  describe("Quality Calculation", () => {
    it("should calculate quality from explicit quality in metadata", () => {
      const event: PerformanceEvent = {
        eventId: "quality-1",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          quality: 0.87,
          latencyMs: 500,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history!.avgQuality).toBeCloseTo(0.87, 2);
    });

    it("should calculate quality from score in metadata", () => {
      const event: PerformanceEvent = {
        eventId: "quality-2",
        eventType: "judgment",
        timestamp: new Date(),
        success: true,
        metadata: {
          score: 0.93,
          latencyMs: 800,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "judgment");
      expect(history!.avgQuality).toBeCloseTo(0.93, 2);
    });

    it("should infer quality from latency when no explicit quality", () => {
      const fastEvent: PerformanceEvent = {
        eventId: "quality-3",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 500, // < 1s, should be high quality
        },
      };

      const slowEvent: PerformanceEvent = {
        eventId: "quality-4",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 6000, // > 5s, should be lower quality
        },
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
        eventId: "quality-5",
        eventType: "task_execution",
        timestamp: new Date(),
        success: false,
        metadata: {
          latencyMs: 500,
        },
      };

      bridge.recordFromPerformanceEvent(failureEvent, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history!.avgQuality).toBeLessThan(0.5);
    });
  });

  describe("Memory Estimation", () => {
    it("should use memory from metadata when available", () => {
      const event: PerformanceEvent = {
        eventId: "memory-1",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 500,
          memoryMB: 768,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const history = selector.getPerformanceHistory(modelId, "task_execution");
      expect(history!.avgMemoryMB).toBe(768);
    });

    it("should estimate memory based on event type when not provided", () => {
      const judgmentEvent: PerformanceEvent = {
        eventId: "memory-2",
        eventType: "judgment",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 500,
        },
      };

      const thinkingEvent: PerformanceEvent = {
        eventId: "memory-3",
        eventType: "thinking_budget",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 100,
        },
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
        eventId: "tokens-1",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 1000,
          outputTokens: 100,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const costProfile = costTracker.getCostProfile(modelId);

      expect(costProfile).toBeDefined();
      expect(costProfile!.avgTokensPerSec).toBeCloseTo(100, 0);
    });

    it("should use default token values when not provided", () => {
      const event: PerformanceEvent = {
        eventId: "tokens-2",
        eventType: "task_execution",
        timestamp: new Date(),
        success: true,
        metadata: {
          latencyMs: 1000,
        },
      };

      bridge.recordFromPerformanceEvent(event, modelId);

      const costProfile = costTracker.getCostProfile(modelId);

      expect(costProfile).toBeDefined();
      expect(costProfile!.avgTokensPerSec).toBeGreaterThan(0);
    });
  });
});
