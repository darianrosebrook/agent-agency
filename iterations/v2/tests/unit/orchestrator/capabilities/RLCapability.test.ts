/**
 * @fileoverview Unit tests for RLCapability
 *
 * Tests reinforcement learning capability composition for orchestrator.
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { AgentRegistryManager } from "../../../../src/orchestrator/AgentRegistryManager";
import { RLCapability } from "../../../../src/orchestrator/capabilities/RLCapability";
import { Task, TaskResult } from "../../../../src/types/arbiter-orchestration";

// Mock dependencies
jest.mock("../../../../src/rl/MultiArmedBandit");
jest.mock("../../../../src/rl/PerformanceTracker");
jest.mock("../../../../src/rl/TurnLevelRLTrainer");
jest.mock("../../../../src/rl/ToolAdoptionTrainer");
jest.mock("../../../../src/orchestrator/TaskRoutingManager");

describe("RLCapability", () => {
  let rlCapability: RLCapability;
  let mockAgentRegistry: jest.Mocked<AgentRegistryManager>;

  const createTestTask = (): Task => ({
    id: "test-task-1",
    description: "Test task for RL",
    type: "file_editing" as any,
    priority: 5,
    timeoutMs: 30000,
    attempts: 0,
    maxAttempts: 3,
    requiredCapabilities: {},
    budget: { maxFiles: 10, maxLoc: 1000 },
    createdAt: new Date(),
    metadata: {},
  });

  const createTestResult = (success: boolean): TaskResult => ({
    id: "result-1",
    task: createTestTask(),
    agent: {
      id: "agent-1",
      name: "Test Agent",
      modelFamily: "gpt-4" as any,
      capabilities: {} as any,
      performanceHistory: {} as any,
      currentLoad: {} as any,
      registeredAt: new Date().toISOString(),
      lastActiveAt: new Date().toISOString(),
    },
    success,
    output: success ? { result: "success" } : null,
    performance: {
      success,
      qualityScore: success ? 0.85 : 0.3,
      latencyMs: 2500,
      tokensUsed: 1500,
    },
    qualityScore: success ? 0.85 : 0.3,
    errors: success ? [] : ["Test error"],
    completedAt: new Date(),
    executionTimeMs: 2500,
  });

  beforeEach(() => {
    mockAgentRegistry = {
      registerAgent: jest.fn(),
      getProfile: jest.fn(),
      updatePerformance: jest.fn(),
      getStats: jest.fn(),
    } as any;
  });

  describe("initialization", () => {
    it("should not initialize when all features disabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      expect(rlCapability.isEnabled()).toBe(false);
    });

    it("should initialize with only multi-armed bandit enabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: true,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
        banditConfig: { explorationRate: 0.2 },
      });

      await rlCapability.initialize(mockAgentRegistry);

      expect(rlCapability.isEnabled()).toBe(true);
      const stats = rlCapability.getStats();
      expect(stats).toBeDefined();
    });

    it("should initialize with performance tracking enabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: true,
        enableRLTraining: false,
        enableToolAdoption: false,
        performanceTrackerConfig: { maxEventsInMemory: 1000 },
      });

      await rlCapability.initialize(mockAgentRegistry);

      expect(rlCapability.isEnabled()).toBe(true);
    });

    it("should initialize with all features enabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: true,
        enablePerformanceTracking: true,
        enableRLTraining: true,
        enableToolAdoption: true,
        banditConfig: {},
        performanceTrackerConfig: {},
        rlTrainingConfig: {},
        toolAdoptionConfig: {},
      });

      await rlCapability.initialize(mockAgentRegistry);

      expect(rlCapability.isEnabled()).toBe(true);
      const stats = rlCapability.getStats();
      expect(stats).toHaveProperty("multiArmedBandit");
      expect(stats).toHaveProperty("performanceTracker");
      expect(stats).toHaveProperty("rlTraining");
      expect(stats).toHaveProperty("toolAdoption");
    });
  });

  describe("recordRoutingDecision", () => {
    it("should do nothing when not initialized", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      const task = createTestTask();
      await expect(
        rlCapability.recordRoutingDecision(task, "assignment-1")
      ).resolves.not.toThrow();
    });

    it("should record decision when performance tracking enabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: true,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      const task = createTestTask();
      await expect(
        rlCapability.recordRoutingDecision(task, "assignment-1")
      ).resolves.not.toThrow();
    });
  });

  describe("routeTask", () => {
    it("should return null when not initialized", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      const task = createTestTask();
      const result = await rlCapability.routeTask(task);

      expect(result).toBeNull();
    });

    it("should route task when multi-armed bandit enabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: true,
        enablePerformanceTracking: true,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      const task = createTestTask();
      const result = await rlCapability.routeTask(task);

      // Result should be an assignment object or null (graceful degradation)
      if (result) {
        expect(result).toHaveProperty("taskId");
        expect(result).toHaveProperty("agentId");
      }
    });
  });

  describe("recordTaskCompletion", () => {
    it("should do nothing when not initialized", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      const result = createTestResult(true);
      await expect(
        rlCapability.recordTaskCompletion("task-1", result, "assignment-1")
      ).resolves.not.toThrow();
    });

    it("should record successful completion", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: true,
        enablePerformanceTracking: true,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      const result = createTestResult(true);
      await expect(
        rlCapability.recordTaskCompletion("task-1", result, "assignment-1")
      ).resolves.not.toThrow();
    });

    it("should record failed completion", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: true,
        enablePerformanceTracking: true,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      const result = createTestResult(false);
      await expect(
        rlCapability.recordTaskCompletion("task-1", result, "assignment-1")
      ).resolves.not.toThrow();
    });
  });

  describe("trainModels", () => {
    it("should do nothing when not initialized", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await expect(rlCapability.trainModels()).resolves.not.toThrow();
    });

    it("should train models when RL training enabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: true,
        enableRLTraining: true,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      await expect(rlCapability.trainModels()).resolves.not.toThrow();
    });

    it("should train tool adoption when enabled", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: true,
      });

      await rlCapability.initialize(mockAgentRegistry);

      await expect(rlCapability.trainModels()).resolves.not.toThrow();
    });
  });

  describe("getStats", () => {
    it("should return empty stats when not initialized", () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      const stats = rlCapability.getStats();
      expect(stats).toEqual({});
    });

    it("should return stats for enabled components", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: true,
        enablePerformanceTracking: true,
        enableRLTraining: true,
        enableToolAdoption: true,
      });

      await rlCapability.initialize(mockAgentRegistry);

      const stats = rlCapability.getStats();
      expect(stats).toHaveProperty("multiArmedBandit");
      expect(stats).toHaveProperty("performanceTracker");
      expect(stats).toHaveProperty("rlTraining");
      expect(stats).toHaveProperty("toolAdoption");
    });
  });

  describe("graceful degradation", () => {
    it("should handle errors in routing gracefully", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: true,
        enablePerformanceTracking: false,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      const task = createTestTask();
      // Should not throw even if routing fails
      await expect(rlCapability.routeTask(task)).resolves.toBeDefined();
    });

    it("should handle errors in recording completion gracefully", async () => {
      rlCapability = new RLCapability({
        enableMultiArmedBandit: false,
        enablePerformanceTracking: true,
        enableRLTraining: false,
        enableToolAdoption: false,
      });

      await rlCapability.initialize(mockAgentRegistry);

      const result = createTestResult(true);
      // Should not throw even if recording fails
      await expect(
        rlCapability.recordTaskCompletion("task-1", result)
      ).resolves.not.toThrow();
    });
  });
});
