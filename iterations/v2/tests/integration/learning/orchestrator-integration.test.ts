/**
 * Integration Tests: Orchestrator Learning Integration
 *
 * Tests the integration between LearningIntegration and the orchestrator,
 * including automatic learning triggers and performance metric tracking.
 *
 * @author @darianrosebrook
 */

import { Pool } from "pg";
import type { TaskCompletionEvent } from "../../../src/orchestrator/LearningIntegration.js";
import { LearningIntegration } from "../../../src/orchestrator/LearningIntegration.js";
import { LearningCoordinatorEvent } from "../../../src/types/learning-coordination.js";

describe("Orchestrator Learning Integration Tests", () => {
  let dbPool: Pool;
  let integration: LearningIntegration;

  beforeAll(async () => {
    // Create test database pool
    dbPool = new Pool({
      host: process.env.DB_HOST || "localhost",
      port: parseInt(process.env.DB_PORT || "5432"),
      database: process.env.DB_NAME || "arbiter_test",
      user: process.env.DB_USER || "postgres",
      password: process.env.DB_PASSWORD || "postgres",
    });
  });

  beforeEach(async () => {
    integration = new LearningIntegration(dbPool, {
      enableAutoLearning: true,
      minErrorCount: 2,
      minQualityThreshold: 0.7,
    });

    await integration.initialize();
  });

  afterAll(async () => {
    await dbPool.end();
  });

  describe("Task Completion Handling", () => {
    it("should record performance metrics for task completions", async () => {
      const event: TaskCompletionEvent = {
        taskId: "test-task-1",
        agentId: "agent-1",
        success: true,
        duration: 1500,
        qualityScore: 0.85,
        context: { foo: "bar" },
      };

      await integration.handleTaskCompletion(event);

      const metrics = integration.getPerformanceMetrics(
        event.taskId,
        event.agentId
      );
      expect(metrics.length).toBe(1);
      expect(metrics[0].taskId).toBe(event.taskId);
      expect(metrics[0].agentId).toBe(event.agentId);
      expect(metrics[0].executionTimeMs).toBe(event.duration);
      expect(metrics[0].averageQualityScore).toBe(event.qualityScore);
    });

    it("should aggregate performance statistics correctly", async () => {
      const taskId = "test-task-2";
      const agentId = "agent-2";

      // Submit multiple completions
      for (let i = 0; i < 5; i++) {
        await integration.handleTaskCompletion({
          taskId,
          agentId,
          success: i % 2 === 0, // 3 successes, 2 failures
          duration: 1000 + i * 100,
          qualityScore: 0.5 + i * 0.1,
          context: {},
        });
      }

      const stats = integration.getPerformanceStatistics(taskId, agentId);

      expect(stats.totalExecutions).toBe(5);
      expect(stats.successRate).toBe(0.6); // 3 out of 5
      expect(stats.totalErrors).toBe(2);
      expect(stats.averageExecutionTime).toBeCloseTo(1200, 0);
      expect(stats.averageQualityScore).toBeGreaterThan(0.5);
    });
  });

  describe("Automatic Learning Triggers", () => {
    it("should trigger learning after repeated errors", async () => {
      const taskId = "test-task-3";
      const agentId = "agent-3";

      let sessionStarted = false;
      integration.on(LearningCoordinatorEvent.SESSION_STARTED, (_payload) => {
        sessionStarted = true;
      });

      // Submit 3 successful completions first (no trigger)
      for (let i = 0; i < 3; i++) {
        await integration.handleTaskCompletion({
          taskId,
          agentId,
          success: true,
          duration: 1000,
          qualityScore: 0.9,
          context: { attempt: i },
        });
      }

      expect(sessionStarted).toBe(false);

      // Submit 2 failures to trigger learning
      for (let i = 0; i < 2; i++) {
        await integration.handleTaskCompletion({
          taskId,
          agentId,
          success: false,
          duration: 1000,
          errorMessage: "Test error",
          qualityScore: 0.3,
          context: { attempt: i + 3 },
        });
      }

      // Wait for async processing
      await new Promise((resolve) => setTimeout(resolve, 1000));

      expect(sessionStarted).toBe(true);
    }, 30000);

    it("should trigger learning for low quality scores", async () => {
      const taskId = "test-task-4";
      const agentId = "agent-4";

      let sessionCompleted = false;
      integration.on(LearningCoordinatorEvent.SESSION_COMPLETED, (_payload) => {
        sessionCompleted = true;
      });

      // Submit tasks with consistently low quality
      for (let i = 0; i < 5; i++) {
        await integration.handleTaskCompletion({
          taskId,
          agentId,
          success: true,
          duration: 1000,
          qualityScore: 0.5, // Below threshold of 0.7
          context: { attempt: i },
        });
      }

      // Wait for async processing
      await new Promise((resolve) => setTimeout(resolve, 1000));

      expect(sessionCompleted).toBe(true);
    }, 30000);

    it("should not trigger learning when auto-learning is disabled", async () => {
      const disabledIntegration = new LearningIntegration(dbPool, {
        enableAutoLearning: false,
        minErrorCount: 2,
        minQualityThreshold: 0.7,
      });

      await disabledIntegration.initialize();

      let sessionStarted = false;
      disabledIntegration.on(
        LearningCoordinatorEvent.SESSION_STARTED,
        (_payload) => {
          sessionStarted = true;
        }
      );

      const taskId = "test-task-5";
      const agentId = "agent-5";

      // Submit errors that would normally trigger learning
      for (let i = 0; i < 5; i++) {
        await disabledIntegration.handleTaskCompletion({
          taskId,
          agentId,
          success: false,
          duration: 1000,
          errorMessage: "Test error",
          context: {},
        });
      }

      await new Promise((resolve) => setTimeout(resolve, 500));

      expect(sessionStarted).toBe(false);
    });
  });

  describe("Learning Session Management", () => {
    it("should prevent duplicate learning sessions for same task-agent pair", async () => {
      const taskId = "test-task-6";
      const agentId = "agent-6";

      const event: TaskCompletionEvent = {
        taskId,
        agentId,
        success: false,
        duration: 1000,
        errorMessage: "Test error",
        context: {},
      };

      // Start first session
      const result1 = integration.triggerLearningSession(event);

      // Try to start second session immediately
      const result2 = await integration.triggerLearningSession(event);

      expect(result2).toBeNull(); // Should be rejected

      // Wait for first session to complete
      await result1;

      // Now should be able to start new session
      const result3 = await integration.triggerLearningSession(event);
      expect(result3).not.toBeNull();
    }, 30000);

    it("should track active learning tasks", async () => {
      const taskId = "test-task-7";
      const agentId = "agent-7";

      const event: TaskCompletionEvent = {
        taskId,
        agentId,
        success: true,
        duration: 1000,
        qualityScore: 0.8,
        context: {},
      };

      expect(integration.isLearningActive(taskId, agentId)).toBe(false);

      // Start learning session
      const sessionPromise = integration.triggerLearningSession(event);

      // Should be active during execution
      expect(integration.isLearningActive(taskId, agentId)).toBe(true);

      // Wait for completion
      await sessionPromise;

      // Should be inactive after completion
      expect(integration.isLearningActive(taskId, agentId)).toBe(false);
    }, 30000);
  });

  describe("Event Forwarding", () => {
    it("should forward learning coordinator events to integration listeners", async () => {
      const events: string[] = [];

      // Listen for multiple event types
      integration.on(LearningCoordinatorEvent.SESSION_STARTED, (_payload) => {
        events.push("session_started");
      });

      integration.on(LearningCoordinatorEvent.ITERATION_STARTED, (_payload) => {
        events.push("iteration_started");
      });

      integration.on(LearningCoordinatorEvent.SESSION_COMPLETED, (_payload) => {
        events.push("session_completed");
      });

      // Trigger a learning session
      const event: TaskCompletionEvent = {
        taskId: "test-task-8",
        agentId: "agent-8",
        success: true,
        duration: 1000,
        qualityScore: 0.9,
        context: { test: true },
      };

      await integration.triggerLearningSession(event);

      // Wait for async processing
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Verify events were emitted
      expect(events).toContain("session_started");
      expect(events).toContain("iteration_started");
      expect(events).toContain("session_completed");
    }, 30000);
  });

  describe("Configuration Management", () => {
    it("should allow updating trigger configuration", () => {
      expect(integration["triggerConfig"].minErrorCount).toBe(2);

      integration.updateTriggerConfig({
        minErrorCount: 5,
        minQualityThreshold: 0.5,
      });

      expect(integration["triggerConfig"].minErrorCount).toBe(5);
      expect(integration["triggerConfig"].minQualityThreshold).toBe(0.5);
    });

    it("should clear performance history when requested", async () => {
      const taskId = "test-task-9";
      const agentId = "agent-9";

      // Add some metrics
      await integration.handleTaskCompletion({
        taskId,
        agentId,
        success: true,
        duration: 1000,
        qualityScore: 0.8,
        context: {},
      });

      expect(integration.getPerformanceMetrics(taskId, agentId).length).toBe(1);

      // Clear history
      integration.clearPerformanceHistory(taskId, agentId);

      expect(integration.getPerformanceMetrics(taskId, agentId).length).toBe(0);
    });
  });
});
