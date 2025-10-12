/**
 * Task State Machine Types
 *
 * Defines states, transitions, and history tracking for task lifecycle management.
 *
 * @author @darianrosebrook
 */

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

