# Phase 1.1: Task State Machine Implementation Plan

**Date**: October 12, 2025  
**Status**: 🔄 In Progress  
**Expected Duration**: 2-3 hours

---

## Overview

Implement a robust, validated task state machine that manages task lifecycle from creation to completion. This is the foundation for task orchestration in ARBITER-005.

---

## Goals

1. **State Management**: Track task progression through defined states
2. **Validation**: Ensure only valid state transitions
3. **Persistence**: Store state changes for audit and recovery
4. **Events**: Emit events for state transitions
5. **Error Handling**: Manage failures and retries

---

## Task States

```
PENDING → QUEUED → ASSIGNED → RUNNING → (COMPLETED | FAILED | CANCELLED)
                                    ↓
                                SUSPENDED
```

### State Definitions

- **PENDING**: Task created, awaiting validation
- **QUEUED**: Validated, waiting for agent assignment
- **ASSIGNED**: Agent assigned, waiting to start
- **RUNNING**: Actively executing
- **SUSPENDED**: Paused (can resume)
- **COMPLETED**: Successfully finished
- **FAILED**: Execution failed
- **CANCELLED**: Manually cancelled

---

## Valid State Transitions

```typescript
const VALID_TRANSITIONS: Record<TaskState, TaskState[]> = {
  PENDING: ["QUEUED", "CANCELLED"],
  QUEUED: ["ASSIGNED", "CANCELLED"],
  ASSIGNED: ["RUNNING", "QUEUED", "CANCELLED"], // Can go back to queue
  RUNNING: ["COMPLETED", "FAILED", "SUSPENDED", "CANCELLED"],
  SUSPENDED: ["RUNNING", "CANCELLED"],
  COMPLETED: [], // Terminal state
  FAILED: ["QUEUED"], // Can retry
  CANCELLED: [], // Terminal state
};
```

---

## Implementation

### 1. Task State Types

```typescript
// src/types/task-state.ts

export enum TaskState {
  PENDING = "pending",
  QUEUED = "queued",
  ASSIGNED = "assigned",
  RUNNING = "running",
  SUSPENDED = "suspended",
  COMPLETED = "completed",
  FAILED = "failed",
  CANCELLED = "cancelled",
}

export interface TaskStateTransition {
  from: TaskState;
  to: TaskState;
  timestamp: Date;
  reason?: string;
  metadata?: Record<string, any>;
}

export interface TaskStateHistory {
  taskId: string;
  transitions: TaskStateTransition[];
  currentState: TaskState;
  createdAt: Date;
  updatedAt: Date;
}

export interface TaskStateValidationError {
  taskId: string;
  fromState: TaskState;
  toState: TaskState;
  reason: string;
}
```

### 2. State Machine Implementation

```typescript
// src/orchestrator/TaskStateMachine.ts

import {
  TaskState,
  TaskStateTransition,
  TaskStateHistory,
} from "../types/task-state";
import { EventEmitter } from "events";

export class TaskStateMachineError extends Error {
  constructor(
    public taskId: string,
    public from: TaskState,
    public to: TaskState,
    message: string
  ) {
    super(message);
    this.name = "TaskStateMachineError";
  }
}

export class TaskStateMachine extends EventEmitter {
  private states: Map<string, TaskStateHistory> = new Map();

  // Valid transitions matrix
  private static readonly VALID_TRANSITIONS: Record<TaskState, TaskState[]> = {
    [TaskState.PENDING]: [TaskState.QUEUED, TaskState.CANCELLED],
    [TaskState.QUEUED]: [TaskState.ASSIGNED, TaskState.CANCELLED],
    [TaskState.ASSIGNED]: [
      TaskState.RUNNING,
      TaskState.QUEUED,
      TaskState.CANCELLED,
    ],
    [TaskState.RUNNING]: [
      TaskState.COMPLETED,
      TaskState.FAILED,
      TaskState.SUSPENDED,
      TaskState.CANCELLED,
    ],
    [TaskState.SUSPENDED]: [TaskState.RUNNING, TaskState.CANCELLED],
    [TaskState.COMPLETED]: [],
    [TaskState.FAILED]: [TaskState.QUEUED], // Can retry
    [TaskState.CANCELLED]: [],
  };

  /**
   * Initialize a new task
   */
  initializeTask(taskId: string): void {
    if (this.states.has(taskId)) {
      throw new Error(`Task ${taskId} already initialized`);
    }

    const history: TaskStateHistory = {
      taskId,
      transitions: [],
      currentState: TaskState.PENDING,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.states.set(taskId, history);
    this.emit("task:initialized", { taskId, state: TaskState.PENDING });
  }

  /**
   * Transition task to new state
   */
  transition(
    taskId: string,
    toState: TaskState,
    reason?: string,
    metadata?: Record<string, any>
  ): void {
    const history = this.states.get(taskId);
    if (!history) {
      throw new Error(`Task ${taskId} not found`);
    }

    const fromState = history.currentState;

    // Validate transition
    if (!this.isValidTransition(fromState, toState)) {
      throw new TaskStateMachineError(
        taskId,
        fromState,
        toState,
        `Invalid transition from ${fromState} to ${toState}`
      );
    }

    // Create transition record
    const transition: TaskStateTransition = {
      from: fromState,
      to: toState,
      timestamp: new Date(),
      reason,
      metadata,
    };

    // Update history
    history.transitions.push(transition);
    history.currentState = toState;
    history.updatedAt = new Date();

    // Emit event
    this.emit("task:transitioned", {
      taskId,
      from: fromState,
      to: toState,
      transition,
    });

    // Emit state-specific events
    this.emit(`task:${toState}`, { taskId, transition });
  }

  /**
   * Check if transition is valid
   */
  private isValidTransition(from: TaskState, to: TaskState): boolean {
    const allowedTransitions = TaskStateMachine.VALID_TRANSITIONS[from];
    return allowedTransitions.includes(to);
  }

  /**
   * Get current state
   */
  getState(taskId: string): TaskState {
    const history = this.states.get(taskId);
    if (!history) {
      throw new Error(`Task ${taskId} not found`);
    }
    return history.currentState;
  }

  /**
   * Get full history
   */
  getHistory(taskId: string): TaskStateHistory {
    const history = this.states.get(taskId);
    if (!history) {
      throw new Error(`Task ${taskId} not found`);
    }
    return { ...history };
  }

  /**
   * Get all transitions
   */
  getTransitions(taskId: string): TaskStateTransition[] {
    const history = this.states.get(taskId);
    if (!history) {
      throw new Error(`Task ${taskId} not found`);
    }
    return [...history.transitions];
  }

  /**
   * Check if task is in terminal state
   */
  isTerminal(taskId: string): boolean {
    const state = this.getState(taskId);
    return (
      state === TaskState.COMPLETED ||
      state === TaskState.FAILED ||
      state === TaskState.CANCELLED
    );
  }

  /**
   * Get all tasks in a specific state
   */
  getTasksByState(state: TaskState): string[] {
    return Array.from(this.states.entries())
      .filter(([_, history]) => history.currentState === state)
      .map(([taskId, _]) => taskId);
  }

  /**
   * Get state statistics
   */
  getStats(): Record<TaskState, number> {
    const stats: Record<string, number> = {};

    for (const state of Object.values(TaskState)) {
      stats[state] = 0;
    }

    for (const history of this.states.values()) {
      stats[history.currentState]++;
    }

    return stats as Record<TaskState, number>;
  }

  /**
   * Clear task history (for cleanup)
   */
  clearTask(taskId: string): void {
    this.states.delete(taskId);
    this.emit("task:cleared", { taskId });
  }

  /**
   * Clear all tasks
   */
  clearAll(): void {
    this.states.clear();
    this.emit("tasks:cleared");
  }
}
```

---

## Testing Strategy

### Unit Tests

```typescript
// tests/unit/orchestrator/task-state-machine.test.ts

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
      expect(() => machine.initializeTask("task-1")).toThrow();
    });
  });

  describe("valid transitions", () => {
    it("should allow PENDING → QUEUED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      expect(machine.getState("task-1")).toBe(TaskState.QUEUED);
    });

    it("should allow QUEUED → ASSIGNED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      expect(machine.getState("task-1")).toBe(TaskState.ASSIGNED);
    });

    it("should allow RUNNING → COMPLETED", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.COMPLETED);
      expect(machine.getState("task-1")).toBe(TaskState.COMPLETED);
    });
  });

  describe("invalid transitions", () => {
    it("should reject PENDING → RUNNING", () => {
      machine.initializeTask("task-1");
      expect(() => machine.transition("task-1", TaskState.RUNNING)).toThrow(
        TaskStateMachineError
      );
    });

    it("should reject COMPLETED → RUNNING", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);
      machine.transition("task-1", TaskState.RUNNING);
      machine.transition("task-1", TaskState.COMPLETED);

      expect(() => machine.transition("task-1", TaskState.RUNNING)).toThrow();
    });
  });

  describe("history tracking", () => {
    it("should record all transitions", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED);
      machine.transition("task-1", TaskState.ASSIGNED);

      const transitions = machine.getTransitions("task-1");
      expect(transitions).toHaveLength(2);
      expect(transitions[0].to).toBe(TaskState.QUEUED);
      expect(transitions[1].to).toBe(TaskState.ASSIGNED);
    });

    it("should include metadata in transitions", () => {
      machine.initializeTask("task-1");
      machine.transition("task-1", TaskState.QUEUED, "validation passed", {
        validatorId: "v1",
      });

      const transitions = machine.getTransitions("task-1");
      expect(transitions[0].metadata).toEqual({ validatorId: "v1" });
    });
  });

  describe("event emission", () => {
    it("should emit transition events", (done) => {
      machine.initializeTask("task-1");

      machine.once("task:transitioned", (event) => {
        expect(event.taskId).toBe("task-1");
        expect(event.to).toBe(TaskState.QUEUED);
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
    });

    it("should list tasks by state", () => {
      machine.initializeTask("task-1");
      machine.initializeTask("task-2");
      machine.transition("task-1", TaskState.QUEUED);

      const queued = machine.getTasksByState(TaskState.QUEUED);
      expect(queued).toEqual(["task-1"]);
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
  });
});
```

---

## Acceptance Criteria

1. ✅ Task state machine enforces valid transitions
2. ✅ Invalid transitions throw `TaskStateMachineError`
3. ✅ All transitions are recorded in history
4. ✅ Events are emitted for state changes
5. ✅ Terminal states cannot transition
6. ✅ Statistics available for monitoring
7. ✅ All tests passing (20+ unit tests)

---

## Implementation Checklist

- [ ] Create task state types
- [ ] Implement TaskStateMachine class
- [ ] Add state validation
- [ ] Implement history tracking
- [ ] Add event emission
- [ ] Create unit tests
- [ ] Add statistics methods
- [ ] Create index exports
- [ ] Update documentation

---

**Status**: Ready to implement!
