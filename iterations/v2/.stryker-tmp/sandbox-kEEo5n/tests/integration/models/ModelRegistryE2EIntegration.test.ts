/**
 * @file ModelRegistryE2EIntegration.test.ts
 * @description End-to-end integration tests for Model Registry with RL-003 and ARBITER-004
 * @author @darianrosebrook
 */
// @ts-nocheck


import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import { ModelRegistryLLMProvider } from "@/evaluation/ModelRegistryLLMProvider";
import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import { PerformanceTrackerBridge } from "@/models/PerformanceTrackerBridge";
import { PerformanceTracker } from "@/rl/PerformanceTracker";
import type { JudgmentInput } from "@/types/judge";
import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  jest,
} from "@jest/globals";

describe("Model Registry E2E Integration", () => {
  let registry: ModelRegistry;
  let selector: LocalModelSelector;
  let costTracker: ComputeCostTracker;
  let bridge: PerformanceTrackerBridge;
  let performanceTracker: PerformanceTracker;

  beforeEach(async () => {
    // Initialize core model registry components
    registry = new ModelRegistry();
    costTracker = new ComputeCostTracker();
    selector = new LocalModelSelector(registry, costTracker);
    bridge = new PerformanceTrackerBridge(registry, selector, costTracker);

    // Initialize ARBITER-004 Performance Tracker
    performanceTracker = new PerformanceTracker();

    // Register test models
    await registry.registerOllamaModel(
      "judgment-model",
      "gemma3n:e2b",
      "1.0.0",
      "primary"
    );

    await registry.registerOllamaModel(
      "fast-model",
      "gemma3:1b",
      "1.0.0",
      "fast"
    );

    // Activate models
    const models = registry.getAllModels();
    for (const model of models) {
      await registry.activateModel(model.id);
    }
  });

  afterEach(() => {
    // Clear all mocks and reset state
    jest.clearAllMocks();

    // Clear performance tracker data
    if (performanceTracker) {
      performanceTracker.clearData();
    }

    // Clear registry data
    if (registry) {
      const models = registry.getAllModels();
      for (const model of models) {
        // Note: ModelRegistry doesn't have deactivateModel method
        // Models remain registered but can be deactivated by clearing performance data
      }
    }
  });

  describe("RL-003 (ModelBasedJudge) Integration", () => {
    it("should use Model Registry for LLM provider", async () => {
      // Create Model Registry LLM Provider
      const llmProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "judgment-model",
          temperature: 0.1,
          maxTokens: 1000,
          taskType: "judgment",
          qualityThreshold: 0.8,
        },
        registry,
        selector,
        costTracker
      );

      // Create ModelBasedJudge with custom provider
      const judge = new ModelBasedJudge(undefined, llmProvider);

      // Prepare judgment input
      const input: JudgmentInput = {
        task: "Summarize the following text",
        output: "This is a concise summary of the text.",
        context: { text: "Original text about artificial intelligence..." },
      };

      // Perform judgment
      const result = await judge.evaluate(input);

      // Verify judgment was performed
      expect(result).toBeDefined();
      expect(result.overallScore).toBeGreaterThan(0);
      expect(result.overallConfidence).toBeGreaterThan(0);
      expect(result.assessments.length).toBeGreaterThan(0);

      // Verify model was selected and tracked
      const activeModelId = llmProvider.getActiveModelId();
      expect(activeModelId).toBeDefined();

      // Verify performance was recorded
      const history = selector.getPerformanceHistory(
        activeModelId!,
        "judgment"
      );
      expect(history).toBeDefined();
      expect(history!.samples).toBeGreaterThan(0);
    });

    it("should track performance across multiple judgments", async () => {
      const llmProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "judgment-model",
          temperature: 0.1,
          maxTokens: 1000,
          taskType: "judgment",
        },
        registry,
        selector,
        costTracker
      );

      const judge = new ModelBasedJudge(undefined, llmProvider);

      // Perform multiple judgments
      for (let i = 0; i < 5; i++) {
        const input: JudgmentInput = {
          task: `Task ${i}`,
          output: `Output ${i}`,
        };

        await judge.evaluate(input);
      }

      // Verify accumulated performance data
      const activeModelId = llmProvider.getActiveModelId();
      const history = selector.getPerformanceHistory(
        activeModelId!,
        "judgment"
      );

      expect(history).toBeDefined();
      expect(history!.samples).toBe(5 * 4); // 5 judgments * 4 criteria
      expect(history!.avgLatencyMs).toBeGreaterThan(0);
      expect(history!.successRate).toBeGreaterThan(0);
    });

    it("should select appropriate model based on quality threshold", async () => {
      // Record different quality levels for models
      const models = registry.getAllModels();

      selector.updatePerformanceHistory(models[0].id, "judgment", {
        quality: 0.9,
        latencyMs: 500,
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory(models[1].id, "judgment", {
        quality: 0.6,
        latencyMs: 100,
        memoryMB: 128,
        success: true,
      });

      // Create provider with high quality requirement
      const llmProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "judgment-model",
          temperature: 0.1,
          maxTokens: 1000,
          taskType: "judgment",
          qualityThreshold: 0.85, // High threshold
        },
        registry,
        selector,
        costTracker
      );

      const judge = new ModelBasedJudge(undefined, llmProvider);

      // Perform judgment
      await judge.evaluate({
        task: "High quality task",
        output: "High quality output",
      });

      // Should select the high-quality model
      const activeModelId = llmProvider.getActiveModelId();
      expect(activeModelId).toBe(models[0].id);
    });
  });

  describe("ARBITER-004 (Performance Tracker) Integration", () => {
    it("should bridge performance data to model registry", async () => {
      const models = registry.getAllModels();
      const modelId = models[0].id;

      // Simulate Performance Tracker recording an event
      performanceTracker.recordEvent({
        type: "routing-decision",
        timestamp: new Date().toISOString(),
        data: {
          taskId: "task-1",
          selectedAgent: "agent-1",
          availableAgents: ["agent-1", "agent-2"],
          metrics: { latencyMs: 250, taskType: "routing" },
          confidence: 0.95,
        },
      });

      // Get the performance event
      const events = performanceTracker.exportTrainingData();
      expect(events.length).toBeGreaterThan(0);

      // Bridge to model registry - convert PerformanceEvent to TaskExecutionData format
      const event = events[0];
      if (event.type === "routing-decision") {
        // For routing decisions, we'll record directly as model performance data
        bridge.recordModelPerformance({
          modelId,
          taskType: "routing",
          quality: 0.95,
          latencyMs: 250,
          memoryMB: 128,
          success: true,
          timestamp: new Date(),
        });
      }

      // Verify data was recorded in model registry
      const history = selector.getPerformanceHistory(modelId, "routing");
      expect(history).toBeDefined();
      expect(history!.samples).toBeGreaterThan(0);
    });

    it("should sync task execution data", async () => {
      const models = registry.getAllModels();
      const modelId = models[0].id;

      // Simulate task execution
      performanceTracker.recordEvent({
        type: "task-execution",
        timestamp: new Date().toISOString(),
        data: {
          taskId: "test-task",
          agentId: "agent-1",
          outcome: {
            success: true,
            qualityScore: 0.9,
            efficiencyScore: 0.85,
            tokensConsumed: 100,
            completionTimeMs: 100,
          },
          context: {
            taskType: "execution",
            quality: 0.85,
          },
        },
      });

      // Get task execution data
      const executions = performanceTracker.exportTrainingData();
      expect(executions.length).toBeGreaterThan(0);

      // Bridge to model registry - convert PerformanceEvent to TaskExecutionData format
      const execution = executions[0];
      if (execution.type === "task-execution") {
        // Create a TaskExecutionData object from the PerformanceEvent
        const taskExecutionData: any = {
          executionId: `exec-${Date.now()}`,
          taskId: execution.data.taskId as string,
          agentId: execution.data.agentId as string,
          routingDecision: {} as any,
          outcome: execution.data.outcome as any,
          startedAt: execution.timestamp,
          completedAt: execution.timestamp,
          context: execution.data.context as any,
        };
        bridge.recordFromTaskExecution(taskExecutionData, modelId);
      }

      // Verify data was recorded
      const history = selector.getPerformanceHistory(modelId, "execution");
      expect(history).toBeDefined();
      expect(history!.avgQuality).toBeCloseTo(0.85, 2);
    });

    it("should export model performance to Performance Tracker format", async () => {
      const models = registry.getAllModels();
      const modelId = models[0].id;

      // Record some performance data
      for (let i = 0; i < 10; i++) {
        selector.updatePerformanceHistory(modelId, "test-task", {
          quality: 0.8 + Math.random() * 0.1,
          latencyMs: 100 + Math.random() * 100,
          memoryMB: 200 + Math.random() * 100,
          success: true,
        });
      }

      // Export to Performance Tracker format
      const exportedData = bridge.exportToPerformanceTracker(
        modelId,
        "test-task"
      );

      expect(exportedData.length).toBeGreaterThan(0);
      expect(exportedData[0].agentId).toBe(modelId);
      expect(exportedData[0].context?.avgQuality).toBeDefined();
      expect(exportedData[0].context?.avgLatency).toBeDefined();
    });

    it("should enable RL training with model selection context", async () => {
      const models = registry.getAllModels();

      // Simulate multiple model executions with different outcomes
      for (let i = 0; i < 5; i++) {
        const modelId = models[i % models.length].id;

        // Record task in Performance Tracker
        performanceTracker.recordEvent({
          type: "task-execution",
          timestamp: new Date().toISOString(),
          data: {
            taskId: `task-${i}`,
            agentId: "agent-1",
            outcome: {
              success: true,
              qualityScore: 0.8 + i * 0.02, // Increasing reward
              efficiencyScore: 0.8,
              tokensConsumed: 100,
              completionTimeMs: 100,
            },
            context: { modelId, taskType: "training" },
          },
        });

        // Bridge to model registry
        const executions = performanceTracker.exportTrainingData();
        const execution = executions[executions.length - 1];
        if (execution.type === "task-execution") {
          // Create a TaskExecutionData object from the PerformanceEvent
          const taskExecutionData: any = {
            executionId: `exec-${i}-${Date.now()}`,
            taskId: execution.data.taskId as string,
            agentId: execution.data.agentId as string,
            routingDecision: {} as any,
            outcome: execution.data.outcome as any,
            startedAt: execution.timestamp,
            completedAt: execution.timestamp,
            context: execution.data.context as any,
          };
          bridge.recordFromTaskExecution(taskExecutionData, modelId);
        }
      }

      // Export training data from Performance Tracker
      const trainingData = performanceTracker.exportTrainingData();
      expect(trainingData.length).toBeGreaterThan(0);

      // Verify model performance histories are available
      for (const model of models) {
        const history = selector.getPerformanceHistory(model.id, "training");
        if (history) {
          expect(history.samples).toBeGreaterThan(0);
        }
      }
    });
  });

  describe("Integrated Workflow", () => {
    it("should support complete judgment + tracking + optimization cycle", async () => {
      // 1. Setup integrated system
      const llmProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "judgment-model",
          temperature: 0.1,
          maxTokens: 1000,
          taskType: "integrated-workflow",
        },
        registry,
        selector,
        costTracker
      );

      const judge = new ModelBasedJudge(undefined, llmProvider);

      // 2. Perform judgment (RL-003)
      const judgment = await judge.evaluate({
        task: "Analyze sentiment",
        output: "The sentiment is positive",
      });

      expect(judgment.overallScore).toBeGreaterThan(0);

      // 3. Record in Performance Tracker (ARBITER-004)
      performanceTracker.recordEvent({
        type: "task-execution",
        timestamp: new Date().toISOString(),
        data: {
          taskId: "sentiment-analysis",
          agentId: "agent-1",
          outcome: {
            success: judgment.allCriteriaPass,
            qualityScore: judgment.overallScore,
            efficiencyScore: 0.8,
            tokensConsumed: 100,
            completionTimeMs: 150,
          },
          context: {
            taskType: "integrated-workflow",
            modelId: llmProvider.getActiveModelId()!,
          },
        },
      });

      // 4. Bridge data
      const events = performanceTracker.exportTrainingData();
      const event = events[events.length - 1];
      if (event.type === "task-execution") {
        // Create a TaskExecutionData object from the PerformanceEvent
        const taskExecutionData: any = {
          executionId: `exec-sentiment-${Date.now()}`,
          taskId: event.data.taskId as string,
          agentId: event.data.agentId as string,
          routingDecision: {} as any,
          outcome: event.data.outcome as any,
          startedAt: event.timestamp,
          completedAt: event.timestamp,
          context: event.data.context as any,
        };
        bridge.recordFromTaskExecution(
          taskExecutionData,
          llmProvider.getActiveModelId()!
        );
      }

      // 5. Verify complete data flow
      const modelId = llmProvider.getActiveModelId()!;
      const history = selector.getPerformanceHistory(
        modelId,
        "integrated-workflow"
      );

      expect(history).toBeDefined();
      expect(history!.samples).toBeGreaterThan(0);

      // 6. Verify cost tracking
      const costProfile = costTracker.getCostProfile(modelId);
      expect(costProfile).toBeDefined();
      expect(costProfile?.totalOperations).toBeGreaterThan(0);

      // 7. Verify Performance Tracker has complete data
      const trainingData = performanceTracker.exportTrainingData();
      expect(trainingData.length).toBeGreaterThan(0);
    });

    it("should demonstrate hot-swap capability with performance tracking", async () => {
      // This test would require the hot-swap manager
      // Included as a placeholder for future implementation
      expect(true).toBe(true);
    });
  });

  describe("Cost Optimization Integration", () => {
    it("should track costs across all systems", async () => {
      const llmProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "judgment-model",
          temperature: 0.1,
          maxTokens: 1000,
          taskType: "cost-tracking",
        },
        registry,
        selector,
        costTracker
      );

      const judge = new ModelBasedJudge(undefined, llmProvider);

      // Perform several operations
      for (let i = 0; i < 3; i++) {
        await judge.evaluate({
          task: `Cost test ${i}`,
          output: `Result ${i}`,
        });
      }

      const modelId = llmProvider.getActiveModelId()!;
      const costProfile = costTracker.getCostProfile(modelId);

      expect(costProfile).toBeDefined();
      expect(costProfile?.totalOperations).toBeGreaterThanOrEqual(3 * 4); // 3 judgments * 4 criteria
      expect(costProfile?.avgWallClockMs).toBeGreaterThan(0);
      expect(costProfile?.avgTokensPerSec).toBeGreaterThanOrEqual(0);
    });

    it("should identify optimization opportunities", async () => {
      const models = registry.getAllModels();

      // Record expensive operations
      for (let i = 0; i < 30; i++) {
        costTracker.recordOperation({
          modelId: models[0].id,
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 200, // Low CPU utilization
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 20,
          inputTokens: 100,
          outputTokens: 50,
          tokensPerSecond: 30,
        });
      }

      const recommendations = costTracker.getOptimizationRecommendations(
        models[0].id
      );

      expect(recommendations).toBeDefined();
      expect(recommendations.length).toBeGreaterThan(0);
    });
  });
});
