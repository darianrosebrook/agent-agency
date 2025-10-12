/**
 * Task Orchestrator Event Types
 *
 * Events emitted during task orchestration lifecycle.
 *
 * @author @darianrosebrook
 */

import { RoutingDecision } from "./agentic-rl";
import { Task } from "./arbiter-orchestration";
import { ValidationResult } from "./caws-types";

export interface TaskSubmittedEvent {
  taskId: string;
  task: Task;
  timestamp: Date;
}

export interface TaskValidatedEvent {
  taskId: string;
  validationResult: ValidationResult;
  timestamp: Date;
}

export interface TaskRoutedEvent {
  taskId: string;
  routing: RoutingDecision;
  timestamp: Date;
}

export interface TaskStartedEvent {
  taskId: string;
  agentId: string;
  timestamp: Date;
}

export interface TaskCompletedEvent {
  taskId: string;
  result: TaskExecutionResult;
  timestamp: Date;
}

export interface TaskFailedEvent {
  taskId: string;
  error: any;
  attempt: number;
  timestamp: Date;
}

export interface TaskRetryEvent {
  taskId: string;
  attempt: number;
  delay: number;
  error: any;
  timestamp: Date;
}

export interface TaskCancelledEvent {
  taskId: string;
  reason: string;
  timestamp: Date;
}

export interface TaskSuspendedEvent {
  taskId: string;
  reason: string;
  timestamp: Date;
}

export interface TaskResumedEvent {
  taskId: string;
  reason: string;
  timestamp: Date;
}

export interface OrchestratorStatsEvent {
  queuedTasks: number;
  processingTasks: number;
  completedTasks: number;
  failedTasks: number;
  throughput: number; // tasks per minute
  avgLatency: number; // milliseconds
  timestamp: Date;
}

export type OrchestratorEvent =
  | TaskSubmittedEvent
  | TaskValidatedEvent
  | TaskRoutedEvent
  | TaskStartedEvent
  | TaskCompletedEvent
  | TaskFailedEvent
  | TaskRetryEvent
  | TaskCancelledEvent
  | TaskSuspendedEvent
  | TaskResumedEvent
  | OrchestratorStatsEvent;

/**
 * Task execution result
 */
export interface TaskExecutionResult {
  taskId: string;
  success: boolean;
  result?: any;
  error?: any;
  latencyMs: number;
  qualityScore?: number;
  metadata?: Record<string, any>;
}

/**
 * Orchestrator configuration
 */
export interface OrchestratorConfig {
  maxConcurrentTasks: number;
  maxRetries: number;
  retryBackoffMs: number;
  taskTimeoutMs: number;
  statsIntervalMs: number;
  enableTracing: boolean;
  enableHealthChecks: boolean;
}

/**
 * Task queue statistics
 */
export interface TaskQueueStats {
  queued: number;
  processing: number;
  total: number;
  oldestQueued?: Date;
}
