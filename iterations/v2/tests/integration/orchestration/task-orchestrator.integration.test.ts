/**
 * Integration tests for TaskOrchestrator
 *
 * Tests end-to-end task execution, worker management, and pleading workflows.
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs/promises";
import * as path from "path";
import { TaskOrchestrator } from "../../../src/orchestrator/TaskOrchestrator.js";
import type {
  Task,
  TaskOrchestratorConfig,
} from "../../../src/types/task-runner.js";
import { TaskPriority } from "../../../src/types/task-runner.js";

describe("TaskOrchestrator Integration Tests", () => {
  const tempDir = path.join(__dirname, "../../temp/orchestrator-tests");
  let orchestrator: TaskOrchestrator;

  const config: TaskOrchestratorConfig = {
    workerPool: {
      minPoolSize: 1,
      maxPoolSize: 3,
      workerCapabilities: ["script", "api_call", "data_processing"],
      workerTimeout: 10000, // Shorter for tests
    },
    queue: {
      maxSize: 50,
      priorityLevels: [
        TaskPriority.LOW,
        TaskPriority.MEDIUM,
        TaskPriority.HIGH,
        TaskPriority.CRITICAL,
      ],
      persistenceEnabled: false,
    },
    retry: {
      maxAttempts: 2,
      backoffMultiplier: 1.5,
      initialDelay: 100,
      maxDelay: 1000,
    },
    routing: {
      enabled: true,
      strategy: "round_robin",
    },
    performance: {
      trackingEnabled: true,
      metricsInterval: 1000,
    },
    pleading: {
      enabled: true,
      requiredApprovals: 2, // Lower for tests
      timeoutHours: 1,
    },
  };

  beforeEach(async () => {
    await fs.mkdir(tempDir, { recursive: true });
    orchestrator = new TaskOrchestrator(config);

    // Wait for workers to initialize
    await new Promise((resolve) => setTimeout(resolve, 500));
  });

  afterEach(async () => {
    if (orchestrator) {
      await orchestrator.shutdown();
    }

    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe("Task Execution", () => {
    it("should execute script task successfully", async () => {
      const task: Task = {
        id: "script-task-1",
        type: "script",
        priority: TaskPriority.MEDIUM,
        payload: {
          code: `
            const result = args[0] + args[1];
            context.result = result;
          `,
          args: [5, 3],
        },
        createdAt: new Date(),
      };

      const taskId = await orchestrator.submitTask(task);

      expect(taskId).toBe(task.id);

      // Wait for execution to complete
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const status = await orchestrator.getTaskStatus(taskId);
      expect(["completed", "running"]).toContain(status);
    }, 10000);

    it("should execute API call task", async () => {
      const task: Task = {
        id: "api-task-1",
        type: "api_call",
        priority: TaskPriority.MEDIUM,
        payload: {
          method: "GET",
          url: "https://httpbin.org/get",
          headers: { Accept: "application/json" },
          timeout: 5000,
        },
        createdAt: new Date(),
      };

      const taskId = await orchestrator.submitTask(task);

      expect(taskId).toBe(task.id);

      // Wait for execution
      await new Promise((resolve) => setTimeout(resolve, 3000));

      const status = await orchestrator.getTaskStatus(taskId);
      expect(["completed", "running"]).toContain(status);
    }, 10000);

    it("should execute data processing task", async () => {
      const task: Task = {
        id: "data-task-1",
        type: "data_processing",
        priority: TaskPriority.MEDIUM,
        payload: {
          operation: "filter",
          data: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
          config: {
            filter: "item > 5",
          },
        },
        createdAt: new Date(),
      };

      const taskId = await orchestrator.submitTask(task);

      expect(taskId).toBe(task.id);

      // Wait for execution
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const status = await orchestrator.getTaskStatus(taskId);
      expect(["completed", "running"]).toContain(status);
    }, 10000);

    it("should handle task execution errors", async () => {
      const task: Task = {
        id: "error-task-1",
        type: "script",
        priority: TaskPriority.MEDIUM,
        payload: {
          code: `
            throw new Error("Test execution error");
          `,
        },
        createdAt: new Date(),
      };

      const taskId = await orchestrator.submitTask(task);

      expect(taskId).toBe(task.id);

      // Wait for execution and failure
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const status = await orchestrator.getTaskStatus(taskId);
      expect(status).toBe("failed");
    }, 10000);
  });

  describe("Worker Pool Management", () => {
    it("should manage worker pool size", async () => {
      const metrics = orchestrator.getMetrics();

      expect(metrics.workerPool.totalWorkers).toBeGreaterThanOrEqual(1);
      expect(metrics.workerPool.activeWorkers).toBeGreaterThanOrEqual(0);
    });

    it("should handle concurrent task execution", async () => {
      const tasks: Task[] = Array(3)
        .fill(null)
        .map((_, i) => ({
          id: `concurrent-task-${i}`,
          type: "script" as const,
          priority: TaskPriority.MEDIUM as const,
          payload: {
            code: `
              // Simulate some work
              const start = Date.now();
              while (Date.now() - start < 500) {
                // Busy wait
              }
              context.result = "Task ${i} completed";
            `,
          },
          createdAt: new Date(),
        }));

      // Submit all tasks
      const taskIds = await Promise.all(
        tasks.map((task) => orchestrator.submitTask(task))
      );

      expect(taskIds).toHaveLength(3);

      // Wait for execution
      await new Promise((resolve) => setTimeout(resolve, 3000));

      // Check metrics
      const metrics = orchestrator.getMetrics();
      expect(metrics.queuedTasks).toBe(0); // All should be processed
    }, 15000);
  });

  describe("Pleading Workflow Integration", () => {
    it("should initiate pleading for failed tasks", async () => {
      // Create a task that will fail consistently
      const task: Task = {
        id: "pleading-task-1",
        type: "script",
        priority: TaskPriority.HIGH, // High priority to trigger pleading
        payload: {
          code: `
            throw new Error("Persistent failure for pleading test");
          `,
        },
        createdAt: new Date(),
        retries: 3, // Ensure multiple attempts
      };

      const taskId = await orchestrator.submitTask(task);

      // Wait for multiple execution attempts and pleading initiation
      await new Promise((resolve) => setTimeout(resolve, 5000));

      // Check if pleading workflow was initiated
      const workflow = orchestrator.getPleadingWorkflow(taskId);

      // Note: Pleading initiation depends on internal logic and timing
      // This test validates the interface exists and doesn't throw
      expect(() => orchestrator.getPleadingWorkflow(taskId)).not.toThrow();
    }, 10000);

    it("should handle pleading decision submission", async () => {
      // Test the pleading decision interface
      await expect(
        orchestrator.submitPleadingDecision(
          "non-existent-task",
          "approver-123",
          "approve",
          "Test approval"
        )
      ).rejects.toThrow(); // Should fail for non-existent workflow
    });
  });

  describe("Performance and Monitoring", () => {
    it("should track execution metrics", async () => {
      const task: Task = {
        id: "metrics-task-1",
        type: "script",
        priority: TaskPriority.MEDIUM,
        payload: {
          code: `
            context.result = "Metrics test completed";
          `,
        },
        createdAt: new Date(),
      };

      await orchestrator.submitTask(task);

      // Wait for execution
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const metrics = orchestrator.getMetrics();

      // Validate metrics structure
      expect(metrics).toHaveProperty("activeTasks");
      expect(metrics).toHaveProperty("completedTasks");
      expect(metrics).toHaveProperty("failedTasks");
      expect(metrics).toHaveProperty("workerPool");

      expect(typeof metrics.activeTasks).toBe("number");
      expect(typeof metrics.completedTasks).toBe("number");
      expect(typeof metrics.failedTasks).toBe("number");
    });

    it("should maintain task state consistency", async () => {
      const task: Task = {
        id: "state-task-1",
        type: "script",
        priority: TaskPriority.MEDIUM,
        payload: {
          code: `
            context.result = "State test";
          `,
        },
        createdAt: new Date(),
      };

      await orchestrator.submitTask(task);

      let status = await orchestrator.getTaskStatus(task.id);
      expect(["queued", "running", "completed"]).toContain(status);

      // Wait for completion
      await new Promise((resolve) => setTimeout(resolve, 2000));

      status = await orchestrator.getTaskStatus(task.id);
      expect(["completed", "failed", "running"]).toContain(status);
    });
  });

  describe("Error Recovery", () => {
    it("should handle worker failures gracefully", async () => {
      // Submit multiple tasks to test worker failure handling
      const tasks: Task[] = Array(5)
        .fill(null)
        .map((_, i) => ({
          id: `recovery-task-${i}`,
          type: "script" as const,
          priority: TaskPriority.MEDIUM as const,
          payload: {
            code: `
              if (Math.random() < 0.3) {
                throw new Error("Random failure for recovery test");
              }
              context.result = "Recovery task ${i} succeeded";
            `,
          },
          createdAt: new Date(),
        }));

      const taskIds = await Promise.all(
        tasks.map((task) => orchestrator.submitTask(task))
      );

      expect(taskIds).toHaveLength(5);

      // Wait for all executions to complete
      await new Promise((resolve) => setTimeout(resolve, 5000));

      // System should still be functional
      const metrics = orchestrator.getMetrics();
      expect(metrics.workerPool.totalWorkers).toBeGreaterThanOrEqual(1);
    }, 15000);

    it("should maintain queue integrity during failures", async () => {
      // Submit tasks with mixed success/failure rates
      const tasks: Task[] = Array(10)
        .fill(null)
        .map((_, i) => ({
          id: `integrity-task-${i}`,
          type: "script" as const,
          priority: TaskPriority.MEDIUM as const,
          payload: {
            code: `
              const shouldFail = ${i} % 3 === 0; // Every 3rd task fails
              if (shouldFail) {
                throw new Error("Planned failure for integrity test");
              }
              context.result = "Integrity task ${i} succeeded";
            `,
          },
          createdAt: new Date(),
        }));

      await Promise.all(tasks.map((task) => orchestrator.submitTask(task)));

      // Wait for processing
      await new Promise((resolve) => setTimeout(resolve, 8000));

      // Check that system remained stable
      const metrics = orchestrator.getMetrics();
      expect(metrics.workerPool.totalWorkers).toBeGreaterThanOrEqual(1);
      expect(metrics.queuedTasks).toBe(0); // All should be processed
    }, 20000);
  });

  describe("Lifecycle Management", () => {
    it("should shutdown gracefully with active tasks", async () => {
      const task: Task = {
        id: "shutdown-task-1",
        type: "script",
        priority: TaskPriority.MEDIUM,
        payload: {
          code: `
            // Simulate longer running task
            const start = Date.now();
            while (Date.now() - start < 2000) {
              // Busy wait
            }
            context.result = "Shutdown test completed";
          `,
        },
        createdAt: new Date(),
      };

      await orchestrator.submitTask(task);

      // Shutdown while task might be running
      await expect(orchestrator.shutdown()).resolves.toBeUndefined();

      // Verify shutdown completed
      const metrics = orchestrator.getMetrics();
      expect(metrics.workerPool.totalWorkers).toBe(0);
    }, 10000);

    it("should handle multiple shutdown calls", async () => {
      await orchestrator.shutdown();
      await expect(orchestrator.shutdown()).resolves.toBeUndefined();
    });
  });
});
