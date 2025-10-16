/**
 * @fileoverview Unit tests for TaskLifecycleManager
 *
 * Tests task lifecycle management utility for orchestrator.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { TaskQueue } from "../../../../src/orchestrator/TaskQueue";
import { TaskStateMachine } from "../../../../src/orchestrator/TaskStateMachine";
import { TaskLifecycleManager } from "../../../../src/orchestrator/utils/TaskLifecycleManager";
import { Task } from "../../../../src/types/arbiter-orchestration";
import { TaskState } from "../../../../src/types/task-state";

describe("TaskLifecycleManager", () => {
  let lifecycleManager: TaskLifecycleManager;
  let mockStateMachine: jest.Mocked<TaskStateMachine>;
  let mockTaskQueue: jest.Mocked<TaskQueue>;

  const createTestTask = (id: string = "test-task-1"): Task => ({
    id,
    description: "Test task for lifecycle",
    type: "code-editing" as any,
    priority: 5,
    timeoutMs: 30000,
    attempts: 0,
    maxAttempts: 3,
    requiredCapabilities: {},
    budget: { maxFiles: 10, maxLoc: 1000 },
    createdAt: new Date(),
    metadata: {},
  });

  beforeEach(() => {
    mockStateMachine = {
      initializeTask: jest.fn(),
      transition: jest.fn(),
      getState: jest.fn(),
      isTerminal: jest.fn(),
      getTransitions: jest.fn(),
      on: jest.fn(),
      emit: jest.fn(),
    } as any;

    mockTaskQueue = {
      enqueue: jest.fn(),
      dequeue: jest.fn(),
      remove: jest.fn(),
      complete: jest.fn(),
      on: jest.fn(),
      emit: jest.fn(),
    } as any;

    mockStateMachine.isTerminal.mockReturnValue(false);
    mockStateMachine.getState.mockReturnValue(TaskState.PENDING);
  });

  describe("initialization", () => {
    it("should create lifecycle manager with events disabled", () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: false }
      );

      expect(lifecycleManager).toBeDefined();
    });

    it("should create lifecycle manager with events enabled", () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: true }
      );

      expect(lifecycleManager).toBeDefined();
    });
  });

  describe("initializeTask", () => {
    it("should initialize task in state machine", () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      lifecycleManager.initializeTask("task-1");

      expect(mockStateMachine.initializeTask).toHaveBeenCalledWith("task-1");
    });
  });

  describe("submitTask", () => {
    it("should initialize and queue task", async () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      const task = createTestTask();
      await lifecycleManager.submitTask(task);

      expect(mockStateMachine.initializeTask).toHaveBeenCalledWith(task.id);
      expect(mockTaskQueue.enqueue).toHaveBeenCalledWith(task);
    });

    it("should emit event when events enabled", async () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: true }
      );

      const emitSpy = jest.fn();
      lifecycleManager.on("task:submitted", emitSpy);

      const task = createTestTask();
      await lifecycleManager.submitTask(task);

      expect(emitSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: task.id,
          task,
        })
      );
    });

    it("should not emit event when events disabled", async () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: false }
      );

      const emitSpy = jest.fn();
      lifecycleManager.on("task:submitted", emitSpy);

      const task = createTestTask();
      await lifecycleManager.submitTask(task);

      expect(emitSpy).not.toHaveBeenCalled();
    });
  });

  describe("cancelTask", () => {
    it("should remove from queue and transition to cancelled", async () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      await lifecycleManager.cancelTask("task-1", "user requested");

      expect(mockTaskQueue.remove).toHaveBeenCalledWith("task-1");
      expect(mockStateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.CANCELLED,
        "user requested"
      );
    });

    it("should not transition if already terminal", async () => {
      mockStateMachine.isTerminal.mockReturnValue(true);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      await lifecycleManager.cancelTask("task-1", "user requested");

      expect(mockTaskQueue.remove).toHaveBeenCalledWith("task-1");
      expect(mockStateMachine.transition).not.toHaveBeenCalled();
    });

    it("should emit event when enabled", async () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: true }
      );

      const emitSpy = jest.fn();
      lifecycleManager.on("task:cancelled", emitSpy);

      await lifecycleManager.cancelTask("task-1", "test reason");

      expect(emitSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: "task-1",
          reason: "test reason",
        })
      );
    });
  });

  describe("suspendTask", () => {
    it("should suspend running task", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.RUNNING);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      await lifecycleManager.suspendTask("task-1", "system overload");

      expect(mockStateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.SUSPENDED,
        "system overload"
      );
    });

    it("should not suspend non-running task", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.PENDING);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      await lifecycleManager.suspendTask("task-1", "system overload");

      expect(mockStateMachine.transition).not.toHaveBeenCalled();
    });

    it("should emit event when suspending and events enabled", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.RUNNING);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: true }
      );

      const emitSpy = jest.fn();
      lifecycleManager.on("task:suspended", emitSpy);

      await lifecycleManager.suspendTask("task-1", "test reason");

      expect(emitSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: "task-1",
          reason: "test reason",
        })
      );
    });
  });

  describe("resumeTask", () => {
    it("should resume suspended task", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.SUSPENDED);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      await lifecycleManager.resumeTask("task-1", "system recovered");

      expect(mockStateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.RUNNING,
        "system recovered"
      );
    });

    it("should not resume non-suspended task", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.RUNNING);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      await lifecycleManager.resumeTask("task-1", "system recovered");

      expect(mockStateMachine.transition).not.toHaveBeenCalled();
    });

    it("should emit event when resuming and events enabled", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.SUSPENDED);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: true }
      );

      const emitSpy = jest.fn();
      lifecycleManager.on("task:resumed", emitSpy);

      await lifecycleManager.resumeTask("task-1", "test reason");

      expect(emitSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: "task-1",
          reason: "test reason",
        })
      );
    });
  });

  describe("completeTask", () => {
    it("should mark task as completed", async () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      const result = { success: true };
      await lifecycleManager.completeTask("task-1", result);

      expect(mockTaskQueue.complete).toHaveBeenCalledWith("task-1");
      expect(mockStateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.COMPLETED,
        "execution successful"
      );
    });

    it("should emit event with result when events enabled", async () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: true }
      );

      const emitSpy = jest.fn();
      lifecycleManager.on("task:completed", emitSpy);

      const result = { success: true, data: "test" };
      await lifecycleManager.completeTask("task-1", result);

      expect(emitSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          taskId: "task-1",
          result,
        })
      );
    });
  });

  describe("failTask", () => {
    it("should mark running task as failed", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.RUNNING);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      const error = new Error("Task execution failed");
      await lifecycleManager.failTask("task-1", error);

      expect(mockTaskQueue.complete).toHaveBeenCalledWith("task-1");
      expect(mockStateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.FAILED,
        "Task execution failed"
      );
    });

    it("should mark non-terminal task as failed", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.QUEUED);
      mockStateMachine.isTerminal.mockReturnValue(false);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      const error = new Error("Validation failed");
      await lifecycleManager.failTask("task-1", error);

      expect(mockStateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.FAILED,
        "Validation failed"
      );
    });

    it("should not transition terminal task", async () => {
      mockStateMachine.getState.mockReturnValue(TaskState.COMPLETED);
      mockStateMachine.isTerminal.mockReturnValue(true);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      const error = new Error("Test error");
      await lifecycleManager.failTask("task-1", error);

      expect(mockTaskQueue.complete).toHaveBeenCalledWith("task-1");
      expect(mockStateMachine.transition).not.toHaveBeenCalled();
    });
  });

  describe("getTaskStatus", () => {
    it("should return task status from state machine", () => {
      const mockHistory = [
        {
          from: TaskState.PENDING,
          to: TaskState.QUEUED,
          timestamp: new Date(),
        },
      ];
      mockStateMachine.getState.mockReturnValue(TaskState.QUEUED);
      mockStateMachine.isTerminal.mockReturnValue(false);
      mockStateMachine.getTransitions.mockReturnValue(mockHistory);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      const status = lifecycleManager.getTaskStatus("task-1");

      expect(status.state).toBe(TaskState.QUEUED);
      expect(status.isTerminal).toBe(false);
      expect(status.history).toEqual(mockHistory);
    });
  });

  describe("utility methods", () => {
    it("should check if task is terminal", () => {
      mockStateMachine.isTerminal.mockReturnValue(true);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      expect(lifecycleManager.isTerminal("task-1")).toBe(true);
    });

    it("should get current task state", () => {
      mockStateMachine.getState.mockReturnValue(TaskState.RUNNING);

      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue
      );

      expect(lifecycleManager.getState("task-1")).toBe(TaskState.RUNNING);
    });

    it("should transition task to new state", () => {
      lifecycleManager = new TaskLifecycleManager(
        mockStateMachine,
        mockTaskQueue,
        { enableEvents: true }
      );

      const emitSpy = jest.fn();
      lifecycleManager.on("state:transitioned", emitSpy);

      lifecycleManager.transition("task-1", TaskState.RUNNING, "started");

      expect(mockStateMachine.transition).toHaveBeenCalledWith(
        "task-1",
        TaskState.RUNNING,
        "started"
      );
      expect(emitSpy).toHaveBeenCalled();
    });
  });
});
