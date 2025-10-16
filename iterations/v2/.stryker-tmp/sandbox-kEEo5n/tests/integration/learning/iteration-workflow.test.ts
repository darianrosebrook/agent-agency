/**
 * Integration Tests: Multi-Turn Learning Iteration Workflow
 *
 * Tests end-to-end iteration workflows including context preservation,
 * error recognition, adaptive prompting, and session completion.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { Pool } from "pg";
import { LearningDatabaseClient } from "../../../src/database/LearningDatabaseClient.js";
import {
  MultiTurnLearningCoordinator,
  type LearningTask,
} from "../../../src/learning/MultiTurnLearningCoordinator.js";
import { LearningSessionStatus } from "../../../src/types/learning-coordination.js";

describe("Multi-Turn Learning Iteration Workflow Integration Tests", () => {
  let dbPool: Pool;
  let dbClient: LearningDatabaseClient;
  let coordinator: MultiTurnLearningCoordinator;

  beforeAll(() => {
    // Create test database pool
    dbPool = new Pool({
      host: process.env.DB_HOST || "localhost",
      port: parseInt(process.env.DB_PORT || "5432"),
      database: process.env.DB_NAME || "arbiter_test",
      user: process.env.DB_USER || "postgres",
      password: process.env.DB_PASSWORD || "postgres",
    });

    dbClient = new LearningDatabaseClient(dbPool);
  });

  beforeEach(() => {
    coordinator = new MultiTurnLearningCoordinator(dbClient);
  });

  afterAll(async () => {
    await dbPool.end();
  });

  describe("Successful Learning Session", () => {
    it("should complete a learning session that meets quality threshold", async () => {
      // Create a task that improves over iterations
      const task: LearningTask = {
        taskId: "test-task-success",
        agentId: "test-agent-1",
        initialContext: { value: 0 },
        qualityEvaluator: async (result: unknown) => {
          const r = result as { value: number };
          return Math.min(r.value / 100, 1.0);
        },
        executor: async (context: unknown, _iterNum: number) => {
          const c = context as { value: number };
          return { value: c.value + 30 }; // Improve by 30 each iteration
        },
      };

      const result = await coordinator.startSession(task, {
        maxIterations: 10,
        qualityThreshold: 0.85,
        enableAdaptivePrompting: true,
        enableErrorRecognition: true,
        progressTimeout: 5000,
        noProgressLimit: 3,
        resourceBudgetMB: 100,
        compressionRatio: 0.7,
      });

      // Assertions
      expect(result.success).toBe(true);
      expect(result.finalQualityScore).toBeGreaterThanOrEqual(0.85);
      expect(result.iterationsCompleted).toBeLessThanOrEqual(4); // Should reach 90 in 3 iterations
      expect(result.summary).toBeDefined();
      expect(result.summary.totalIterations).toBe(result.iterationsCompleted);
      expect(result.summary.improvementRate).toBeGreaterThan(0);

      // Verify session was saved to database
      const session = await dbClient.getSession(result.sessionId);
      expect(session).not.toBeNull();
      expect(session!.status).toBe(LearningSessionStatus.COMPLETED);
      expect(session!.improvementTrajectory.length).toBe(
        result.iterationsCompleted
      );
    }, 30000);

    it("should handle iteration with no errors", async () => {
      const task: LearningTask = {
        taskId: "test-task-no-errors",
        agentId: "test-agent-2",
        initialContext: { count: 0 },
        qualityEvaluator: async (_result: unknown) => 0.9,
        executor: async (context: unknown, iterNum: number) => {
          const c = context as { count: number };
          return { count: c.count + iterNum };
        },
      };

      const result = await coordinator.startSession(task, {
        maxIterations: 2,
        qualityThreshold: 0.85,
        enableErrorRecognition: true,
        progressTimeout: 5000,
        noProgressLimit: 3,
        resourceBudgetMB: 100,
        compressionRatio: 0.7,
      });

      expect(result.success).toBe(true);
      expect(result.summary.errorsDetected).toBe(0);
      expect(result.summary.errorsCorrected).toBe(0);
    }, 30000);
  });

  describe("Learning Session with Iteration Limits", () => {
    it("should stop at maximum iterations", async () => {
      const task: LearningTask = {
        taskId: "test-task-max-iterations",
        agentId: "test-agent-3",
        initialContext: { progress: 0 },
        qualityEvaluator: async (_result: unknown) => 0.5, // Never reaches threshold
        executor: async (context: unknown, _iterNum: number) => {
          const c = context as { progress: number };
          return { progress: c.progress + 0.01 };
        },
      };

      const result = await coordinator.startSession(task, {
        maxIterations: 5,
        qualityThreshold: 0.95,
        enableAdaptivePrompting: false,
        enableErrorRecognition: false,
        progressTimeout: 5000,
        noProgressLimit: 10,
        resourceBudgetMB: 100,
        compressionRatio: 0.7,
      });

      expect(result.success).toBe(true);
      expect(result.iterationsCompleted).toBe(5);
      expect(result.finalQualityScore).toBeLessThan(0.95);
    }, 30000);

    it("should stop when no progress is detected", async () => {
      const task: LearningTask = {
        taskId: "test-task-no-progress",
        agentId: "test-agent-4",
        initialContext: { value: 50 },
        qualityEvaluator: async (_result: unknown) => 0.5, // Constant quality
        executor: async (context: unknown, _iterNum: number) => {
          return context; // No change
        },
      };

      const result = await coordinator.startSession(task, {
        maxIterations: 10,
        qualityThreshold: 0.9,
        enableAdaptivePrompting: false,
        enableErrorRecognition: false,
        progressTimeout: 5000,
        noProgressLimit: 3,
        resourceBudgetMB: 100,
        compressionRatio: 0.7,
      });

      expect(result.success).toBe(true);
      expect(result.iterationsCompleted).toBeLessThanOrEqual(4); // 1 + 3 no-progress
    }, 30000);
  });

  describe("Error Handling and Recovery", () => {
    it("should handle and recover from errors during iterations", async () => {
      let errorCount = 0;
      const task: LearningTask = {
        taskId: "test-task-with-errors",
        agentId: "test-agent-5",
        initialContext: { attempts: 0 },
        qualityEvaluator: async (result: unknown) => {
          const r = result as { attempts: number };
          return Math.min(r.attempts / 10, 1.0);
        },
        executor: async (context: unknown, iterNum: number) => {
          const c = context as { attempts: number };
          // Throw error on first 2 iterations
          if (iterNum <= 2 && errorCount < 2) {
            errorCount++;
            throw new Error("Simulated execution error");
          }
          return { attempts: c.attempts + 2 };
        },
      };

      const result = await coordinator.startSession(task, {
        maxIterations: 8,
        qualityThreshold: 0.85,
        enableErrorRecognition: true,
        enableAdaptivePrompting: false,
        progressTimeout: 5000,
        noProgressLimit: 5,
        resourceBudgetMB: 100,
        compressionRatio: 0.7,
      });

      expect(result.success).toBe(true);
      expect(result.summary.errorsDetected).toBeGreaterThan(0);
      expect(result.iterationsCompleted).toBeGreaterThan(2);
    }, 30000);
  });

  describe("Context Preservation", () => {
    it("should preserve and restore context across iterations", async () => {
      const executedContexts: unknown[] = [];

      const task: LearningTask = {
        taskId: "test-task-context-preservation",
        agentId: "test-agent-6",
        initialContext: { data: "initial", history: [] },
        qualityEvaluator: async (result: unknown) => {
          const r = result as { history: string[] };
          return Math.min(r.history.length / 5, 1.0);
        },
        executor: async (context: unknown, iterNum: number) => {
          executedContexts.push(context);
          const c = context as { data: string; history: string[] };
          return {
            data: `iteration-${iterNum}`,
            history: [...c.history, `step-${iterNum}`],
          };
        },
      };

      const result = await coordinator.startSession(task, {
        maxIterations: 6,
        qualityThreshold: 0.85,
        enableAdaptivePrompting: false,
        enableErrorRecognition: false,
        progressTimeout: 5000,
        noProgressLimit: 10,
        resourceBudgetMB: 100,
        compressionRatio: 0.7,
      });

      expect(result.success).toBe(true);
      expect(executedContexts.length).toBeGreaterThan(0);

      // Verify context was preserved and built upon
      for (let i = 1; i < executedContexts.length; i++) {
        const ctx = executedContexts[i] as { history: string[] };
        expect(ctx.history.length).toBe(i);
      }
    }, 30000);
  });

  describe("Performance and Resource Management", () => {
    it("should track resource usage across iterations", async () => {
      const task: LearningTask = {
        taskId: "test-task-resource-tracking",
        agentId: "test-agent-7",
        initialContext: { data: new Array(1000).fill("x") },
        qualityEvaluator: async (_result: unknown) => 0.9,
        executor: async (context: unknown, _iterNum: number) => {
          return context; // Return same context
        },
      };

      const result = await coordinator.startSession(task, {
        maxIterations: 3,
        qualityThreshold: 0.85,
        enableAdaptivePrompting: false,
        enableErrorRecognition: false,
        progressTimeout: 5000,
        noProgressLimit: 5,
        resourceBudgetMB: 100,
        compressionRatio: 0.7,
      });

      expect(result.success).toBe(true);

      // Verify iterations were tracked with resource usage
      const iterations = await dbClient.getIterations(result.sessionId);
      expect(iterations.length).toBeGreaterThan(0);

      for (const iteration of iterations) {
        expect(iteration.resourceUsageMB).toBeGreaterThan(0);
        expect(iteration.durationMs).toBeGreaterThan(0);
      }
    }, 30000);
  });
});
