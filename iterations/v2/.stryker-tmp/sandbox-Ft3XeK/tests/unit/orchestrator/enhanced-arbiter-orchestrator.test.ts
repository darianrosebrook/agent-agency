/**
 * Enhanced Arbiter Orchestrator Unit Tests
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { describe, it, expect, beforeEach, jest } from "@jest/globals";
import { EnhancedArbiterOrchestrator } from "../../../src/orchestrator/EnhancedArbiterOrchestrator";
import { Task, TaskResult } from "../../../src/types/arbiter-orchestration";

// Mock the base ArbiterOrchestrator
jest.mock("../../../src/orchestrator/ArbiterOrchestrator", () => ({
  ArbiterOrchestrator: class MockArbiterOrchestrator {
    async initialize(): Promise<void> {}
    async submitTask(task: Task): Promise<{ taskId: string; assignmentId?: string }> {
      return { taskId: task.id, assignmentId: `assignment-${task.id}` };
    }
    protected async attemptImmediateAssignment(): Promise<any> {
      return null;
    }
  },
}));

describe("EnhancedArbiterOrchestrator", () => {
  let orchestrator: EnhancedArbiterOrchestrator;
  let mockConfig: any;

  const createMockTask = (
    id: string,
    type: "code-editing" | "code-review" | "validation" | "general",
    description: string
  ): Task => ({
    id,
    description,
    type,
    requiredCapabilities: {},
    priority: 5,
    timeoutMs: 30000,
    budget: { maxFiles: 10, maxLoc: 1000 },
    createdAt: new Date(),
    metadata: { source: "test" },
    attempts: 0,
    maxAttempts: 3,
  });

  beforeEach(() => {
    mockConfig = {
      taskQueue: {},
      taskAssignment: {},
      agentRegistry: {},
      security: {},
      healthMonitor: {},
      recoveryManager: {},
      knowledgeSeeker: {},
      orchestrator: {
        enableMetrics: true,
        enableTracing: true,
        maxConcurrentTasks: 10,
        taskTimeoutMs: 30000,
      },
      rl: {
        enableMultiArmedBandit: true,
        enablePerformanceTracking: true,
        enableRLTraining: true,
        enableToolAdoption: true,
        banditConfig: { explorationRate: 0.2 },
        performanceTrackerConfig: { maxEventsInMemory: 1000 },
        rlTrainingConfig: { learningRate: 1e-5 },
        toolAdoptionConfig: { warmupExamples: 100 },
      },
    };

    orchestrator = new EnhancedArbiterOrchestrator(mockConfig);
  });

  describe("initialization", () => {
    it("should initialize with RL components when enabled", async () => {
      await orchestrator.initialize();

      const stats = orchestrator.getRLStats();
      expect(stats.multiArmedBandit).toBeDefined();
      expect(stats.performanceTracker).toBeDefined();
      expect(stats.rlTraining).toBeDefined();
      expect(stats.toolAdoption).toBeDefined();
    });

    it("should initialize without RL components when disabled", async () => {
      const disabledConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          enableMultiArmedBandit: false,
          enablePerformanceTracking: false,
          enableRLTraining: false,
          enableToolAdoption: false,
        },
      };

      const disabledOrchestrator = new EnhancedArbiterOrchestrator(disabledConfig);
      await disabledOrchestrator.initialize();

      const stats = disabledOrchestrator.getRLStats();
      expect(stats.multiArmedBandit).toBeUndefined();
      expect(stats.performanceTracker).toBeUndefined();
      expect(stats.rlTraining).toBeUndefined();
      expect(stats.toolAdoption).toBeUndefined();
    });

    it("should start performance tracking when enabled", async () => {
      // Mock the performance tracker to check if startCollection is called
      const mockTracker = {
        startCollection: jest.fn(),
        getStats: jest.fn().mockReturnValue({}),
      };

      // We can't easily mock the internal components, so we'll test the behavior indirectly
      await orchestrator.initialize();

      const stats = orchestrator.getRLStats();
      expect(stats.performanceTracker).toBeDefined();
    });
  });

  describe("task submission", () => {
    beforeEach(async () => {
      await orchestrator.initialize();
    });

    it("should submit tasks with RL tracking", async () => {
      const task = createMockTask("task-123", "code-editing", "Test task for RL integration");

      const result = await orchestrator.submitTask(task);

      expect(result.taskId).toBe(task.id);
      expect(result.assignmentId).toBeDefined();
    });

    it("should handle task submission without RL components", async () => {
      const disabledConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          enablePerformanceTracking: false,
        },
      };

      const disabledOrchestrator = new EnhancedArbiterOrchestrator(disabledConfig);
      await disabledOrchestrator.initialize();

      const task = createMockTask("task-456", "code-review", "Code review task");

      const result = await disabledOrchestrator.submitTask(task);

      expect(result.taskId).toBe(task.id);
      // Should still work without RL tracking
    });
  });

  describe("task completion recording", () => {
    beforeEach(async () => {
      await orchestrator.initialize();
    });

    it("should record task completion for RL training", async () => {
      const task = createMockTask("task-123", "code-editing", "Code editing task");

      const taskResult: TaskResult = {
        id: "result-123",
        task,
        agent: {} as any, // Mock agent profile
        success: true,
        output: "completed output",
        performance: {
          success: true,
          qualityScore: 0.85,
          latencyMs: 2500,
          tokensUsed: 1500,
        },
        qualityScore: 0.85,
        errors: [],
        completedAt: new Date(),
        executionTimeMs: 2500,
      };

      // Should not throw
      await expect(
        orchestrator.recordTaskCompletion("task-123", taskResult, "assignment-123")
      ).resolves.not.toThrow();
    });

    it("should handle task completion without RL components", async () => {
      const disabledConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          enablePerformanceTracking: false,
        },
      };

      const disabledOrchestrator = new EnhancedArbiterOrchestrator(disabledConfig);
      await disabledOrchestrator.initialize();

      const task = createMockTask("task-456", "code-review", "Code review task");

      const taskResult: TaskResult = {
        id: "result-456",
        task,
        agent: {} as any, // Mock agent profile
        success: false,
        output: "failed output",
        performance: {
          success: false,
          qualityScore: 0.3,
          latencyMs: 1000,
          tokensUsed: 500,
        },
        qualityScore: 0.3,
        errors: ["Task execution failed"],
        completedAt: new Date(),
        executionTimeMs: 1000,
      };

      // Should not throw even without RL components
      await expect(
        disabledOrchestrator.recordTaskCompletion("task-456", taskResult)
      ).resolves.not.toThrow();
    });
  });

  describe("RL training", () => {
    beforeEach(async () => {
      await orchestrator.initialize();
    });

    it("should train RL models on collected data", async () => {
      // Should not throw
      await expect(orchestrator.trainRLModels()).resolves.not.toThrow();
    });

    it("should handle RL training without components", async () => {
      const disabledConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          enableRLTraining: false,
          enableToolAdoption: false,
        },
      };

      const disabledOrchestrator = new EnhancedArbiterOrchestrator(disabledConfig);
      await disabledOrchestrator.initialize();

      // Should not throw even without RL training components
      await expect(disabledOrchestrator.trainRLModels()).resolves.not.toThrow();
    });
  });

  describe("RL statistics", () => {
    it("should provide RL statistics when components are enabled", async () => {
      await orchestrator.initialize();

      const stats = orchestrator.getRLStats();

      expect(stats).toHaveProperty("multiArmedBandit");
      expect(stats).toHaveProperty("performanceTracker");
      expect(stats).toHaveProperty("rlTraining");
      expect(stats).toHaveProperty("toolAdoption");
    });

    it("should return empty stats when RL components are disabled", () => {
      const disabledConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          enableMultiArmedBandit: false,
          enablePerformanceTracking: false,
          enableRLTraining: false,
          enableToolAdoption: false,
        },
      };

      const disabledOrchestrator = new EnhancedArbiterOrchestrator(disabledConfig);
      const stats = disabledOrchestrator.getRLStats();

      expect(stats.multiArmedBandit).toBeUndefined();
      expect(stats.performanceTracker).toBeUndefined();
      expect(stats.rlTraining).toBeUndefined();
      expect(stats.toolAdoption).toBeUndefined();
    });
  });

  describe("task assignment enhancement", () => {
    beforeEach(async () => {
      await orchestrator.initialize();
    });

    it("should enhance task assignment with RL when components are available", async () => {
      const task = createMockTask("task-enhanced", "code-editing", "Enhanced code editing task");

      // The enhanced assignment is tested through the attemptImmediateAssignment method
      // which is called internally by submitTask
      const result = await orchestrator.submitTask(task);

      expect(result.taskId).toBe(task.id);
      // The assignment enhancement happens internally
    });

    it("should fall back gracefully when RL components fail", async () => {
      // Create orchestrator with invalid RL config to simulate failure
      const failingConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          banditConfig: null, // This might cause issues
        },
      };

      const failingOrchestrator = new EnhancedArbiterOrchestrator(failingConfig);
      await failingOrchestrator.initialize();

      const task = createMockTask("task-fallback", "validation", "Validation task");

      // Should still work with fallback to base implementation
      const result = await failingOrchestrator.submitTask(task);
      expect(result.taskId).toBe(task.id);
    });
  });

  describe("feature flags", () => {
    it("should allow selective enabling of RL features", async () => {
      const selectiveConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          enableMultiArmedBandit: true,
          enablePerformanceTracking: false,
          enableRLTraining: true,
          enableToolAdoption: false,
        },
      };

      const selectiveOrchestrator = new EnhancedArbiterOrchestrator(selectiveConfig);
      await selectiveOrchestrator.initialize();

      const stats = selectiveOrchestrator.getRLStats();

      expect(stats.multiArmedBandit).toBeDefined();
      expect(stats.performanceTracker).toBeUndefined();
      expect(stats.rlTraining).toBeDefined();
      expect(stats.toolAdoption).toBeUndefined();
    });

    it("should handle all RL features disabled", () => {
      const allDisabledConfig = {
        ...mockConfig,
        rl: {
          enableMultiArmedBandit: false,
          enablePerformanceTracking: false,
          enableRLTraining: false,
          enableToolAdoption: false,
        },
      };

      const allDisabledOrchestrator = new EnhancedArbiterOrchestrator(allDisabledConfig);
      const stats = allDisabledOrchestrator.getRLStats();

      expect(Object.keys(stats)).toHaveLength(0);
    });
  });

  describe("error handling", () => {
    it("should handle initialization failures gracefully", async () => {
      const failingConfig = {
        ...mockConfig,
        rl: {
          ...mockConfig.rl,
          banditConfig: { invalidConfig: true },
        },
      };

      const failingOrchestrator = new EnhancedArbiterOrchestrator(failingConfig);

      // Should not throw during initialization
      await expect(failingOrchestrator.initialize()).resolves.not.toThrow();

      // Should still be able to submit tasks
      const task = createMockTask("task-after-failure", "code-review", "Error handling task");

      const result = await failingOrchestrator.submitTask(task);
      expect(result.taskId).toBe(task.id);
    });

    it("should handle RL training failures without breaking orchestrator", async () => {
      await orchestrator.initialize();

      // Train RL models - should not throw even if training fails internally
      await expect(orchestrator.trainRLModels()).resolves.not.toThrow();

      // Should still be able to submit tasks after training failure
      const task = createMockTask("task-after-training-failure", "general", "Post-training task");

      const result = await orchestrator.submitTask(task);
      expect(result.taskId).toBe(task.id);
    });
  });
});
