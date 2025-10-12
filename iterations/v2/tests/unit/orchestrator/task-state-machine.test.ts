/**
 * Task State Machine Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import {
  TaskStateMachine,
  TaskStateMachineError,
} from "../../../src/orchestrator/TaskStateMachine";
import { TaskState } from "../../../src/types/task-state";

describe("TaskStateMachine", () => {
  let machine: TaskStateMachine;

  beforeEach(() => {
    machine = new TaskStateMachine();
  });

  describe("initialization", () => {
    it("should initialize task in PENDING state", () => {
      machine.initializeTask("task-1");
      expect(machine.getState("task-1")).toBe(TaskState.PENDING);
    });

    it("should throw if task already initialized", () => {
      machine.initializeTask("task-1");
      expect(() => machine.initializeTask("task-1")).toThrow(
        "Task task-1 already initialized"
      );
    });

    it("should emit initialization event", (done) => {
      machine.once("task:initialized", (event) => {
        expect(event.taskId).toBe("task-1");
        expect(event.state).toBe(TaskState.PENDING);
        done();
      });

      machine.initializeTask("task-1");
    });

    it("should create history with timestamps", () => {
      machine.initializeTask("task-1");
      const history = machine.getHistory("task-1");

      expect(history.taskId).toBe("task-1");
      expect(history.createdAt).toBeInstanceOf(Date);
      expect(history.updatedAt).toBeInstanceOf(Date);
      expect(history.transitions).toEqual([]);
    });
  });

  describe("valid transitions", () => {
    it("should allow PENDING → QUEUED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      expect(machine.getState("task-1")).toBe(TaskState.QUEUED);
    });

    it("should allow PENDING → CANCELLED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.CANCELLED);
      expect(machine.getState("task-1")).toBe(TaskState.CANCELLED);
    });

    it("should allow QUEUED → ASSIGNED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      expect(machine.getState("task-1")).toBe(TaskState.ASSIGNED);
    });

    it("should allow ASSIGNED → RUNNING", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      expect(machine.getState("task-1")).toBe(TaskState.RUNNING);
    });

    it("should allow RUNNING → COMPLETED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.COMPLETED);
      expect(machine.getState("task-1")).toBe(TaskState.COMPLETED);
    });

    it("should allow RUNNING → FAILED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.FAILED);
      expect(machine.getState("task-1")).toBe(TaskState.FAILED);
    });

    it("should allow RUNNING → SUSPENDED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.SUSPENDED);
      expect(machine.getState("task-1")).toBe(TaskState.SUSPENDED);
    });

    it("should allow SUSPENDED → RUNNING (resume)", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.SUSPENDED);
      machine.transition("task-1", TaskState.RUNNING);
      expect(machine.getState("task-1")).toBe(TaskState.RUNNING);
    });

    it("should allow FAILED → QUEUED (retry)", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.FAILED);
      machine.transition("task-1", TaskState.QUEUED);
      expect(machine.getState("task-1")).toBe(TaskState.QUEUED);
    });

    it("should allow ASSIGNED → QUEUED (reassignment)", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.QUEUED);
      expect(machine.getState("task-1")).toBe(TaskState.QUEUED);
    });
  });

  describe("invalid transitions", () => {
    it("should reject PENDING → RUNNING", () => {
      machine.initializeTask("task-1");
      expect(() => machine.transition("task-1", TaskState.RUNNING)).toThrow(
        TaskStateMachineError
      );
    });

    it("should reject PENDING → COMPLETED", () => {
      machine.initializeTask("task-1");
      expect(() =>
        machine.transition("task-1", TaskState.COMPLETED)
      ).toThrow(TaskStateMachineError);
    });

    it("should reject QUEUED → RUNNING", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      expect(() => machine.transition("task-1", TaskState.RUNNING)).toThrow(
        TaskStateMachineError
      );
    });

    it("should reject COMPLETED → RUNNING (terminal state)", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.COMPLETED);

      expect(() => machine.transition("task-1", TaskState.RUNNING)).toThrow(
        TaskStateMachineError
      );
    });

    it("should reject CANCELLED → QUEUED (terminal state)", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.CANCELLED);

      expect(() => machine.transition("task-1", TaskState.QUEUED)).toThrow(
        TaskStateMachineError
      );
    });

    it("should throw for non-existent task", () => {
      expect(() => machine.transition("task-999", TaskState.QUEUED)).toThrow(
        "Task task-999 not found"
      );
    });
  });

  describe("history tracking", () => {
    it("should record all transitions", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);

      const transitions = machine.getTransitions("task-1");
      expect(transitions).toHaveLength(2);
      expect(transitions[0].from).toBe(TaskState.PENDING);
      expect(transitions[0].to).toBe(TaskState.QUEUED);
      expect(transitions[1].from).toBe(TaskState.QUEUED);
      expect(transitions[1].to).toBe(TaskState.ASSIGNED);
    });

    it("should include timestamps in transitions", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);

      const transitions = machine.getTransitions("task-1");
      expect(transitions[0].timestamp).toBeInstanceOf(Date);
    });

    it("should include reason in transitions", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED, "validation passed");

      const transitions = machine.getTransitions("task-1");
      expect(transitions[0].reason).toBe("validation passed");
    });

    it("should include metadata in transitions", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED, "validation passed", {
        validatorId: "v1",
        score: 0.95,
      });

      const transitions = machine.getTransitions("task-1");
      expect(transitions[0].metadata).toEqual({
        validatorId: "v1",
        score: 0.95,
      });
    });

    it("should update updatedAt timestamp on transition", () => {
      machine.initializeTask("task-1");
      const history1 = machine.getHistory("task-1");
      const timestamp1 = history1.updatedAt;

      // Small delay to ensure timestamp difference
      setTimeout(() => {
        machine.transition("task-1", TaskState.QUEUED);
        const history2 = machine.getHistory("task-1");
        expect(history2.updatedAt.getTime()).toBeGreaterThan(
          timestamp1.getTime()
        );
      }, 10);
    });
  });

  describe("event emission", () => {
    it("should emit transition events", (done) => {
      machine.initializeTask("task-1");

      machine.once("task:transitioned", (event) => {
        expect(event.taskId).toBe("task-1");
        expect(event.from).toBe(TaskState.PENDING);
        expect(event.to).toBe(TaskState.QUEUED);
        expect(event.transition).toBeDefined();
        done();
      });

      machine.transition("task-1", TaskState.QUEUED);
    });

    it("should emit state-specific events", (done) => {
      machine.initializeTask("task-1");

      machine.once("task:completed", (event) => {
        expect(event.taskId).toBe("task-1");
        done();
      });

      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.COMPLETED);
    });

    it("should emit cleared event", (done) => {
      machine.initializeTask("task-1");

      machine.once("task:cleared", (event) => {
        expect(event.taskId).toBe("task-1");
        done();
      });

      machine.clearTask("task-1");
    });
  });

  describe("statistics", () => {
    it("should provide state distribution", () => {
      machine.initializeTask("task-1");
      machine.initializeTask("task-2");
      machine.initializeTask("task-3");

      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-2", TaskState.QUEUED);

      const stats = machine.getStats();
      expect(stats[TaskState.PENDING]).toBe(1);
      expect(stats[TaskState.QUEUED]).toBe(2);
      expect(stats[TaskState.COMPLETED]).toBe(0);
    });

    it("should list tasks by state", () => {
      machine.initializeTask("task-1");
      machine.initializeTask("task-2");
      machine.initializeTask("task-3");

      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-2", TaskState.QUEUED);

      const queued = machine.getTasksByState(TaskState.QUEUED);
      expect(queued).toHaveLength(2);
      expect(queued).toContain("task-1");
      expect(queued).toContain("task-2");
    });

    it("should return empty array for states with no tasks", () => {
      machine.initializeTask("task-1");

      const completed = machine.getTasksByState(TaskState.COMPLETED);
      expect(completed).toEqual([]);
    });

    it("should count total tasks", () => {
      machine.initializeTask("task-1");
      machine.initializeTask("task-2");

      expect(machine.getTaskCount()).toBe(2);
    });
  });

  describe("terminal states", () => {
    it("should identify completed as terminal", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.COMPLETED);

      expect(machine.isTerminal("task-1")).toBe(true);
    });

    it("should identify failed as terminal", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.FAILED);

      expect(machine.isTerminal("task-1")).toBe(true);
    });

    it("should identify cancelled as terminal", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.CANCELLED);

      expect(machine.isTerminal("task-1")).toBe(true);
    });

    it("should identify running as non-terminal", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);

      expect(machine.isTerminal("task-1")).toBe(false);
    });
  });

  describe("cleanup", () => {
    it("should clear individual task", () => {
      machine.initializeTask("task-1");
      machine.initializeTask("task-2");

      machine.clearTask("task-1");

      expect(machine.hasTask("task-1")).toBe(false);
      expect(machine.hasTask("task-2")).toBe(true);
    });

    it("should clear all tasks", () => {
      machine.initializeTask("task-1");
      machine.initializeTask("task-2");
      machine.initializeTask("task-3");

      machine.clearAll();

      expect(machine.getTaskCount()).toBe(0);
      expect(machine.hasTask("task-1")).toBe(false);
    });

    it("should emit cleared event when clearing all", (done) => {
      machine.initializeTask("task-1");

      machine.once("tasks:cleared", () => {
        done();
      });

      machine.clearAll();
    });
  });

  describe("error handling", () => {
    it("should throw for invalid transitions with detailed error", () => {
      machine.initializeTask("task-1");

      try {
        machine.transition("task-1", TaskState.RUNNING);
        fail("Should have thrown TaskStateMachineError");
      } catch (error) {
        expect(error).toBeInstanceOf(TaskStateMachineError);
        expect((error as TaskStateMachineError).taskId).toBe("task-1");
        expect((error as TaskStateMachineError).from).toBe(
          TaskState.PENDING
        );
        expect((error as TaskStateMachineError).to).toBe(TaskState.RUNNING);
      }
    });

    it("should throw when accessing non-existent task state", () => {
      expect(() => machine.getState("task-999")).toThrow(
        "Task task-999 not found"
      );
    });

    it("should throw when accessing non-existent task history", () => {
      expect(() => machine.getHistory("task-999")).toThrow(
        "Task task-999 not found"
      );
    });

    it("should throw when checking terminal for non-existent task", () => {
      expect(() => machine.isTerminal("task-999")).toThrow(
        "Task task-999 not found"
      );
    });
  });
});

