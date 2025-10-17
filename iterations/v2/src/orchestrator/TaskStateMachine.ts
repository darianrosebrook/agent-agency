/**
 * Task State Machine
 *
 * Manages task lifecycle with validated state transitions.
 * Emits events for state changes and maintains history.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import {
  TaskState,
  TaskStateHistory,
  TaskStateTransition,
} from "../types/task-state";

// Re-export commonly used types
export { VerificationPriority } from "../types/verification";

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

/**
 * Task state machine with validation and history tracking
 */
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
      .filter(([_taskId, history]) => history.currentState === state)
      .map(([taskId, _history]) => taskId);
  }

  /**
   * Get state statistics
   */
  getStats(): Record<TaskState, number> {
    const stats: Record<string, number> = {};

    for (const state of Object.values(TaskState)) {
      stats[state] = 0;
    }

    for (const history of Array.from(this.states.values())) {
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

  /**
   * Check if task exists
   */
  hasTask(taskId: string): boolean {
    return this.states.has(taskId);
  }

  /**
   * Get total number of tasks
   */
  getTaskCount(): number {
    return this.states.size;
  }
}
