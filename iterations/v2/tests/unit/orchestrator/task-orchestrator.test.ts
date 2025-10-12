/**
 * Task Orchestrator Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { SpecValidator } from "../../../src/caws-validator/validation/SpecValidator";
import { HealthMonitor } from "../../../src/health/HealthMonitor";
import { TracingProvider } from "../../../src/observability/TracingProvider";
import { AgentRegistryManager } from "../../../src/orchestrator/AgentRegistryManager";
import { TaskOrchestrator } from "../../../src/orchestrator/TaskOrchestrator";
import { TaskQueue } from "../../../src/orchestrator/TaskQueue";
import { TaskRetryHandler } from "../../../src/orchestrator/TaskRetryHandler";
import { TaskRoutingManager } from "../../../src/orchestrator/TaskRoutingManager";
import { TaskStateMachine } from "../../../src/orchestrator/TaskStateMachine";
import { PerformanceTracker } from "../../../src/rl/PerformanceTracker";

import { TaskState } from "../../../src/types/task-state";
import { createMinimalTask } from "../../helpers/test-fixtures";

describe("TaskOrchestrator", () => {
  let orchestrator: TaskOrchestrator;

  // Mocks
  let stateMachine: jest.Mocked<TaskStateMachine>;
  let taskQueue: jest.Mocked<TaskQueue>;
  let retryHandler: jest.Mocked<TaskRetryHandler>;
  let agentRegistry: jest.Mocked<AgentRegistryManager>;
  let taskRouter: jest.Mocked<TaskRoutingManager>;
  let cawsValidator: jest.Mocked<SpecValidator>;
  let performanceTracker: jest.Mocked<PerformanceTracker>;
  let tracing: jest.Mocked<TracingProvider>;
  let healthMonitor: jest.Mocked<HealthMonitor>;

  beforeEach(() => {
    // Create mocks
    stateMachine = {
      initializeTask: jest.fn(),
      transition: jest.fn(),
      getState: jest.fn(),
      isTerminal: jest.fn(),
      getTransitions: jest.fn(),
      getStats: jest.fn(),
      on: jest.fn(),
      emit: jest.fn(),
    } as any;

    taskQueue = {
      enqueue: jest.fn(),
      dequeue: jest.fn(),
      remove: jest.fn(),
      complete: jest.fn(),
      hasTask: jest.fn(),
      getStats: jest.fn(),
      on: jest.fn(),
      emit: jest.fn(),
    } as any;

    retryHandler = {
      executeWithRetry: jest.fn(),
      on: jest.fn(),
      emit: jest.fn(),
    } as any;

    agentRegistry = {} as any;
    taskRouter = {
      routeTask: jest.fn(),
    } as any;
    cawsValidator = {
      validateWorkingSpec: jest.fn(),
    } as any;
    performanceTracker = {
      startTaskExecution: jest.fn(),
      completeTaskExecution: jest.fn(),
      recordRoutingDecision: jest.fn(),
    } as any;
    tracing = {
      traceOperation: jest.fn(),
      startSpan: jest.fn(),
    } as any;
    healthMonitor = {
      registerCheck: jest.fn(),
    } as any;

    // Setup default mock behaviors
    stateMachine.isTerminal.mockReturnValue(false);
    stateMachine.getState.mockReturnValue(TaskState.PENDING);
    taskQueue.hasTask.mockReturnValue(false);
    taskQueue.getStats.mockReturnValue({
      queued: 0,
      processing: 0,
      total: 0,
    });
    stateMachine.getStats.mockReturnValue({
      [TaskState.PENDING]: 0,
      [TaskState.QUEUED]: 0,
      [TaskState.ASSIGNED]: 0,
      [TaskState.RUNNING]: 0,
      [TaskState.SUSPENDED]: 0,
      [TaskState.COMPLETED]: 0,
      [TaskState.FAILED]: 0,
      [TaskState.CANCELLED]: 0,
    });
    retryHandler.getStats.mockReturnValue({
      activeRetries: 0,
      totalAttempts: 0,
      averageRetries: 0,
    });

    tracing.traceOperation.mockImplementation(async (name, fn) => fn());

    orchestrator = new TaskOrchestrator(
      stateMachine,
      taskQueue,
      retryHandler,
      agentRegistry,
      taskRouter,
      cawsValidator,
      performanceTracker,
      tracing,
      healthMonitor
    );
  });

  describe("initialization", () => {
    it("should register health checks", () => {
      expect(healthMonitor.registerCheck).toHaveBeenCalledWith(
        "orchestrator",
        expect.any(Function)
      );
    });

    it("should setup event handlers", () => {
      expect(stateMachine.on).toHaveBeenCalledWith(
        "task:transitioned",
        expect.any(Function)
      );
      expect(retryHandler.on).toHaveBeenCalledWith(
        "task:retry",
        expect.any(Function)
      );
    });
  });

  describe("task submission", () => {
    const validTask = createMinimalTask();
    const invalidTask = createMinimalTask();

    beforeEach(() => {
      cawsValidator.validateWorkingSpec.mockImplementation(async (task) => {
        if (task.id === validTask.id) {
          return { valid: true, errors: [], warnings: [] };
        } else {
          return {
            valid: false,
            errors: [{ field: "type", message: "Invalid type" }],
            warnings: [],
          };
        }
      });
    });

    it("should accept valid task", async () => {
      const result = await orchestrator.submitTask(validTask);

      expect(stateMachine.initializeTask).toHaveBeenCalledWith(validTask.id);
      expect(cawsValidator.validateWorkingSpec).toHaveBeenCalledWith(validTask);
      expect(taskQueue.enqueue).toHaveBeenCalledWith(validTask);
      expect(result.taskId).toBe(validTask.id);
      expect(result.success).toBe(true);
    });

    it("should reject invalid task", async () => {
      await expect(orchestrator.submitTask(invalidTask)).rejects.toThrow();

      expect(stateMachine.initializeTask).toHaveBeenCalledWith(invalidTask.id);
      expect(stateMachine.transition).toHaveBeenCalledWith(
        invalidTask.id,
        TaskState.CANCELLED,
        "validation failed"
      );
    });

    it("should emit submission event", async () => {
      const eventSpy = jest.fn();
      orchestrator.on("task:submitted", eventSpy);

      await orchestrator.submitTask(validTask);

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: validTask.id,
          task: validTask,
          timestamp: expect.any(Date),
        })
      );
    });

    it("should trace task submission", async () => {
      await orchestrator.submitTask(validTask);

      expect(tracing.traceOperation).toHaveBeenCalledWith(
        "orchestrator:submitTask",
        expect.any(Function)
      );
    });
  });

  describe("task cancellation", () => {
    it("should cancel queued task", async () => {
      taskQueue.hasTask.mockReturnValue(true);
      stateMachine.isTerminal.mockReturnValue(false);

      await orchestrator.cancelTask("task-1", "user cancelled");

      expect(taskQueue.remove).toHaveBeenCalledWith("task-1");
      expect(stateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.CANCELLED,
        "user cancelled"
      );
    });

    it("should not cancel terminal task", async () => {
      stateMachine.isTerminal.mockReturnValue(true);

      await orchestrator.cancelTask("task-1");

      expect(stateMachine.transition).not.toHaveBeenCalled();
    });

    it("should emit cancellation event", async () => {
      const eventSpy = jest.fn();
      orchestrator.on("task:cancelled", eventSpy);

      await orchestrator.cancelTask("task-1", "test reason");

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: "task-1",
          reason: "test reason",
          timestamp: expect.any(Date),
        })
      );
    });
  });

  describe("task suspension and resumption", () => {
    it("should suspend running task", async () => {
      stateMachine.getState.mockReturnValue(TaskState.RUNNING);

      await orchestrator.suspendTask("task-1", "maintenance");

      expect(stateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.SUSPENDED,
        "maintenance"
      );
    });

    it("should resume suspended task", async () => {
      stateMachine.getState.mockReturnValue(TaskState.SUSPENDED);

      await orchestrator.resumeTask("task-1", "maintenance complete");

      expect(stateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.RUNNING,
        "maintenance complete"
      );
    });

    it("should not suspend non-running task", async () => {
      stateMachine.getState.mockReturnValue(TaskState.QUEUED);

      await orchestrator.suspendTask("task-1");

      expect(stateMachine.transition).not.toHaveBeenCalled();
    });
  });

  describe("task status", () => {
    it("should return task status", () => {
      stateMachine.getState.mockReturnValue(TaskState.RUNNING);
      stateMachine.isTerminal.mockReturnValue(false);
      stateMachine.getTransitions.mockReturnValue([]);

      const status = orchestrator.getTaskStatus("task-1");

      expect(status.state).toBe(TaskState.RUNNING);
      expect(status.isTerminal).toBe(false);
      expect(status.history).toEqual([]);
    });
  });

  describe("statistics", () => {
    it("should return orchestrator statistics", () => {
      const stats = orchestrator.getStats();

      expect(stats).toEqual({
        queue: expect.any(Object),
        stateMachine: expect.any(Object),
        retryHandler: expect.any(Object),
        isRunning: false,
      });

      expect(taskQueue.getStats).toHaveBeenCalled();
      expect(stateMachine.getStats).toHaveBeenCalled();
      expect(retryHandler.getStats).toHaveBeenCalled();
    });
  });

  describe("start/stop", () => {
    it("should start orchestrator", async () => {
      await orchestrator.start();

      expect(orchestrator.getStats().isRunning).toBe(true);
    });

    it("should stop orchestrator", async () => {
      await orchestrator.start();
      await orchestrator.stop();

      expect(orchestrator.getStats().isRunning).toBe(false);
    });

    it("should handle multiple start/stop calls", async () => {
      await orchestrator.start();
      await orchestrator.start(); // Should be no-op
      await orchestrator.stop();
      await orchestrator.stop(); // Should be no-op

      expect(orchestrator.getStats().isRunning).toBe(false);
    });
  });

  describe("private methods", () => {
    describe("validateTask", () => {
      it("should validate task successfully", async () => {
        const task = createMinimalTask();
        cawsValidator.validateWorkingSpec.mockResolvedValue({
          valid: true,
          errors: [],
          warnings: [],
        });

        // Access private method for testing
        await (orchestrator as any).validateTask(task);

        expect(cawsValidator.validateWorkingSpec).toHaveBeenCalledWith(task);
      });

      it("should throw on validation failure", async () => {
        const task = createMinimalTask();
        cawsValidator.validateWorkingSpec.mockResolvedValue({
          valid: false,
          errors: [{ field: "type", message: "Invalid" }],
          warnings: [],
        });

        await expect(
          (orchestrator as any).validateTask(task)
        ).rejects.toThrow();
      });
    });

    describe("routeTask", () => {
      it("should route task successfully", async () => {
        const task = createMinimalTask();
        const routing = { selectedAgent: { id: "agent-1" } };

        taskRouter.routeTask.mockResolvedValue(routing as any);

        const result = await (orchestrator as any).routeTask(task);

        expect(taskRouter.routeTask).toHaveBeenCalledWith(task, {
          agentMetrics: {},
        });
        expect(result).toBe(routing);
      });
    });
  });
});
