/**
 * Task State Machine Types
 *
 * Defines states, transitions, and history tracking for task lifecycle management.
 *
 * @author @darianrosebrook
 */

export enum TaskState {
  _PENDING = "pending",
  _QUEUED = "queued",
  _ASSIGNED = "assigned",
  _RUNNING = "running",
  _SUSPENDED = "suspended",
  _COMPLETED = "completed",
  _FAILED = "failed",
  _CANCELLED = "cancelled",
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
