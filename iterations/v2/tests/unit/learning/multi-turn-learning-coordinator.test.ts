/**
 * Unit Tests: MultiTurnLearningCoordinator
 *
 * Tests the main orchestration layer for multi-turn learning,
 * including session management, iteration execution, and completion.
 *
 * @author @darianrosebrook
 */

import type { LearningDatabaseClient } from "../../../src/database/LearningDatabaseClient.js";
import {
  MultiTurnLearningCoordinator,
  type LearningTask,
} from "../../../src/learning/MultiTurnLearningCoordinator.js";
import {
  LearningCoordinatorEvent,
  LearningSessionStatus,
} from "../../../src/types/learning-coordination.js";

// Mock dependencies
const createMockDbClient = (): jest.Mocked<LearningDatabaseClient> =>
  ({
    createSession: jest.fn(),
    getSession: jest.fn(),
    updateSession: jest.fn(),
    getSessionsByTask: jest.fn(),
    createIteration: jest.fn(),
    getIterations: jest.fn(),
    upsertErrorPattern: jest.fn(),
    getErrorPatterns: jest.fn(),
    saveSnapshot: jest.fn(),
    getSnapshot: jest.fn(),
    transaction: jest.fn(),
  } as unknown as jest.Mocked<LearningDatabaseClient>);

describe("MultiTurnLearningCoordinator", () => {
  let mockDbClient: jest.Mocked<LearningDatabaseClient>;
  let coordinator: MultiTurnLearningCoordinator;

  beforeEach(() => {
    mockDbClient = createMockDbClient();
    coordinator = new MultiTurnLearningCoordinator(mockDbClient);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Session Initialization", () => {
    it("should initialize a learning session with default config", async () => {
      const task: LearningTask = {
        taskId: "test-task-1",
        agentId: "agent-1",
        initialContext: { value: 0 },
        qualityEvaluator: jest.fn().mockResolvedValue(0.9),
        executor: jest.fn().mockResolvedValue({ value: 10 }),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task);

      expect(result.success).toBe(true);
      expect(mockDbClient.createSession).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: "test-task-1",
          agentId: "agent-1",
          status: LearningSessionStatus.ACTIVE,
        })
      );
    });

    it("should use custom session configuration", async () => {
      const task: LearningTask = {
        taskId: "test-task-2",
        agentId: "agent-2",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.8),
        executor: jest.fn().mockResolvedValue({}),
      };

      const customConfig = {
        maxIterations: 5,
        qualityThreshold: 0.95,
        resourceBudgetMB: 200,
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      await coordinator.startSession(task, customConfig);

      expect(mockDbClient.createSession).toHaveBeenCalledWith(
        expect.objectContaining({
          config: expect.objectContaining({
            maxIterations: 5,
            qualityThreshold: 0.95,
            resourceBudgetMB: 200,
          }),
        })
      );
    });

    it("should emit session started event", async () => {
      const task: LearningTask = {
        taskId: "test-task-3",
        agentId: "agent-3",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.9),
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const eventSpy = jest.fn();
      coordinator.on(LearningCoordinatorEvent.SESSION_STARTED, eventSpy);

      await coordinator.startSession(task);

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          eventType: LearningCoordinatorEvent.SESSION_STARTED,
        })
      );
    });
  });

  describe("Iteration Execution", () => {
    it("should execute task and evaluate quality", async () => {
      const mockExecutor = jest
        .fn()
        .mockResolvedValueOnce({ value: 50 })
        .mockResolvedValueOnce({ value: 90 });

      const mockEvaluator = jest
        .fn()
        .mockResolvedValueOnce(0.5)
        .mockResolvedValueOnce(0.9);

      const task: LearningTask = {
        taskId: "test-task-4",
        agentId: "agent-4",
        initialContext: { value: 0 },
        qualityEvaluator: mockEvaluator,
        executor: mockExecutor,
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      await coordinator.startSession(task, { qualityThreshold: 0.85 });

      expect(mockExecutor).toHaveBeenCalled();
      expect(mockEvaluator).toHaveBeenCalled();
    });

    it("should track iteration improvement delta", async () => {
      const mockExecutor = jest.fn().mockResolvedValue({ value: 100 });
      const mockEvaluator = jest
        .fn()
        .mockResolvedValueOnce(0.6)
        .mockResolvedValueOnce(0.75)
        .mockResolvedValueOnce(0.9);

      const task: LearningTask = {
        taskId: "test-task-5",
        agentId: "agent-5",
        initialContext: {},
        qualityEvaluator: mockEvaluator,
        executor: mockExecutor,
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task, {
        qualityThreshold: 0.85,
      });

      expect(result.success).toBe(true);
      expect(result.iterationsCompleted).toBeGreaterThan(1);
    });

    it("should emit iteration completed events", async () => {
      const task: LearningTask = {
        taskId: "test-task-6",
        agentId: "agent-6",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.9),
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const eventSpy = jest.fn();
      coordinator.on(LearningCoordinatorEvent.ITERATION_COMPLETED, eventSpy);

      await coordinator.startSession(task);

      expect(eventSpy).toHaveBeenCalled();
    });
  });

  describe("Quality Threshold Evaluation", () => {
    it("should complete when quality threshold is met", async () => {
      const mockEvaluator = jest.fn().mockResolvedValue(0.92);

      const task: LearningTask = {
        taskId: "test-task-7",
        agentId: "agent-7",
        initialContext: {},
        qualityEvaluator: mockEvaluator,
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task, {
        qualityThreshold: 0.9,
        maxIterations: 10,
      });

      expect(result.success).toBe(true);
      expect(result.finalQualityScore).toBeGreaterThanOrEqual(0.9);
      expect(result.summary).toBeDefined();
    });

    it("should emit quality threshold met event", async () => {
      const task: LearningTask = {
        taskId: "test-task-8",
        agentId: "agent-8",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const eventSpy = jest.fn();
      coordinator.on(LearningCoordinatorEvent.QUALITY_THRESHOLD_MET, eventSpy);

      await coordinator.startSession(task, { qualityThreshold: 0.9 });

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          eventType: LearningCoordinatorEvent.QUALITY_THRESHOLD_MET,
        })
      );
    });

    it("should continue iterations until threshold is met or limit reached", async () => {
      const mockEvaluator = jest
        .fn()
        .mockResolvedValueOnce(0.5)
        .mockResolvedValueOnce(0.6)
        .mockResolvedValueOnce(0.7)
        .mockResolvedValueOnce(0.8)
        .mockResolvedValueOnce(0.91);

      const task: LearningTask = {
        taskId: "test-task-9",
        agentId: "agent-9",
        initialContext: {},
        qualityEvaluator: mockEvaluator,
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task, {
        qualityThreshold: 0.9,
        maxIterations: 10,
      });

      expect(result.success).toBe(true);
      expect(result.iterationsCompleted).toBe(5);
      expect(mockEvaluator).toHaveBeenCalledTimes(5);
    });
  });

  describe("Error Handling", () => {
    it("should detect and record errors during iteration", async () => {
      const mockExecutor = jest
        .fn()
        .mockRejectedValueOnce(new Error("Test error"))
        .mockResolvedValueOnce({ recovered: true });

      const task: LearningTask = {
        taskId: "test-task-10",
        agentId: "agent-10",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.9),
        executor: mockExecutor,
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task);

      expect(result.success).toBe(true);
      expect(mockDbClient.createIteration).toHaveBeenCalledWith(
        expect.objectContaining({
          errorDetected: true,
        })
      );
    });

    it("should emit error detected events", async () => {
      const task: LearningTask = {
        taskId: "test-task-11",
        agentId: "agent-11",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.9),
        executor: jest.fn().mockRejectedValue(new Error("Test error")),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const eventSpy = jest.fn();
      coordinator.on(LearningCoordinatorEvent.ERROR_DETECTED, eventSpy);

      await coordinator.startSession(task, { maxIterations: 1 });

      expect(eventSpy).toHaveBeenCalled();
    });

    it("should continue after recoverable errors", async () => {
      const mockExecutor = jest
        .fn()
        .mockRejectedValueOnce(new Error("Temporary error"))
        .mockResolvedValueOnce({ value: 50 })
        .mockResolvedValueOnce({ value: 90 });

      const mockEvaluator = jest
        .fn()
        .mockResolvedValueOnce(0.5)
        .mockResolvedValueOnce(0.9);

      const task: LearningTask = {
        taskId: "test-task-12",
        agentId: "agent-12",
        initialContext: {},
        qualityEvaluator: mockEvaluator,
        executor: mockExecutor,
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task, {
        qualityThreshold: 0.85,
        maxIterations: 5,
      });

      expect(result.success).toBe(true);
      expect(mockExecutor).toHaveBeenCalledTimes(3);
    });
  });

  describe("Session Completion", () => {
    it("should complete session successfully when threshold met", async () => {
      const task: LearningTask = {
        taskId: "test-task-13",
        agentId: "agent-13",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: jest.fn().mockResolvedValue({ success: true }),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task, {
        qualityThreshold: 0.9,
      });

      expect(result.success).toBe(true);
      expect(result.finalResult).toBeDefined();
      expect(result.summary).toBeDefined();
      expect(mockDbClient.updateSession).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          status: LearningSessionStatus.COMPLETED,
        })
      );
    });

    it("should emit session completed event", async () => {
      const task: LearningTask = {
        taskId: "test-task-14",
        agentId: "agent-14",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const eventSpy = jest.fn();
      coordinator.on(LearningCoordinatorEvent.SESSION_COMPLETED, eventSpy);

      await coordinator.startSession(task, { qualityThreshold: 0.9 });

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          eventType: LearningCoordinatorEvent.SESSION_COMPLETED,
        })
      );
    });

    it("should generate learning summary with improvement metrics", async () => {
      const mockEvaluator = jest
        .fn()
        .mockResolvedValueOnce(0.5)
        .mockResolvedValueOnce(0.7)
        .mockResolvedValueOnce(0.92);

      const task: LearningTask = {
        taskId: "test-task-15",
        agentId: "agent-15",
        initialContext: {},
        qualityEvaluator: mockEvaluator,
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task, {
        qualityThreshold: 0.9,
      });

      expect(result.summary).toBeDefined();
      expect(result.summary.initialQualityScore).toBe(0.5);
      expect(result.summary.finalQualityScore).toBeGreaterThanOrEqual(0.9);
      expect(result.summary.improvementRate).toBeGreaterThan(0);
    });
  });

  describe("Learning Summary Generation", () => {
    it("should include key insights in summary", async () => {
      const task: LearningTask = {
        taskId: "test-task-16",
        agentId: "agent-16",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task);

      expect(result.summary.keyInsights).toBeDefined();
      expect(result.summary.keyInsights.length).toBeGreaterThan(0);
    });

    it("should note significant improvement in summary", async () => {
      const mockEvaluator = jest
        .fn()
        .mockResolvedValueOnce(0.3)
        .mockResolvedValueOnce(0.95);

      const task: LearningTask = {
        taskId: "test-task-17",
        agentId: "agent-17",
        initialContext: {},
        qualityEvaluator: mockEvaluator,
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task, {
        qualityThreshold: 0.9,
      });

      const improvementInsight = result.summary.keyInsights.find((insight) =>
        insight.toLowerCase().includes("improvement")
      );
      expect(improvementInsight).toBeDefined();
    });

    it("should note error patterns if present", async () => {
      const mockExecutor = jest
        .fn()
        .mockRejectedValueOnce(new Error("Error 1"))
        .mockRejectedValueOnce(new Error("Error 2"))
        .mockResolvedValueOnce({ recovered: true });

      const task: LearningTask = {
        taskId: "test-task-18",
        agentId: "agent-18",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: mockExecutor,
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task);

      const errorInsight = result.summary.keyInsights.find((insight) =>
        insight.toLowerCase().includes("error")
      );
      expect(errorInsight).toBeDefined();
    });
  });

  describe("Session State Management", () => {
    it("should track active sessions", async () => {
      const task: LearningTask = {
        taskId: "test-task-19",
        agentId: "agent-19",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const sessionPromise = coordinator.startSession(task);

      // Session should be tracked during execution
      const result = await sessionPromise;

      // Verify session was created and completed
      expect(result.success).toBe(true);
      expect(result.sessionId).toBeDefined();
    });

    it("should clean up session state after completion", async () => {
      const task: LearningTask = {
        taskId: "test-task-20",
        agentId: "agent-20",
        initialContext: {},
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: jest.fn().mockResolvedValue({}),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      const result = await coordinator.startSession(task);

      expect(result.success).toBe(true);
      // Active session should be cleaned up after completion
      expect(mockDbClient.updateSession).toHaveBeenCalled();
    });
  });

  describe("Resource Tracking", () => {
    it("should track resource usage across iterations", async () => {
      const task: LearningTask = {
        taskId: "test-task-21",
        agentId: "agent-21",
        initialContext: { data: new Array(100).fill("test") },
        qualityEvaluator: jest.fn().mockResolvedValue(0.95),
        executor: jest
          .fn()
          .mockResolvedValue({ data: new Array(100).fill("test") }),
      };

      mockDbClient.createSession.mockResolvedValue();
      mockDbClient.createIteration.mockResolvedValue();
      mockDbClient.updateSession.mockResolvedValue();

      await coordinator.startSession(task);

      expect(mockDbClient.createIteration).toHaveBeenCalledWith(
        expect.objectContaining({
          resourceUsageMB: expect.any(Number),
        })
      );
    });
  });
});
