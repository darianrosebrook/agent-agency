/**
 * @fileoverview Orchestrator Event Types (ARBITER-005)
 *
 * Defines all event types emitted by orchestrator components for comprehensive
 * observability, monitoring, and debugging.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { BaseEvent, EventSeverity } from "./EventEmitter";

// Re-export EventSeverity for convenience
export { EventSeverity } from "./EventEmitter";

/**
 * Task Queue Events
 */
export interface TaskEnqueuedEvent extends BaseEvent {
  type: "task.enqueued";
  taskId: string;
  agentId?: string;
  priority: number;
  queueDepth: number;
  estimatedWaitTimeMs?: number;
}

export interface TaskDequeuedEvent extends BaseEvent {
  type: "task.dequeued";
  taskId: string;
  agentId?: string;
  queueDepth: number;
  waitTimeMs: number;
}

export interface TaskQueueFullEvent extends BaseEvent {
  type: "task.queue_full";
  taskId: string;
  agentId?: string;
  capacity: number;
  rejectedTasks: number;
}

export interface TaskQueueClearedEvent extends BaseEvent {
  type: "task.queue_cleared";
  clearedCount: number;
  agentId?: string;
  reason?: string;
}

/**
 * Task Assignment Events
 */
export interface TaskAssignedEvent extends BaseEvent {
  type: "task.assigned";
  taskId: string;
  assignmentId: string;
  agentId: string;
  routingConfidence: number;
  deadline: Date;
  estimatedDurationMs?: number;
}

export interface TaskAssignmentAcknowledgedEvent extends BaseEvent {
  type: "task.assignment_acknowledged";
  taskId: string;
  assignmentId: string;
  agentId: string;
  acknowledgmentTimeMs: number;
}

export interface TaskProgressUpdatedEvent extends BaseEvent {
  type: "task.progress_updated";
  taskId: string;
  assignmentId: string;
  agentId: string;
  progress: number;
  previousProgress: number;
  estimatedTimeRemainingMs?: number;
}

export interface TaskCompletedEvent extends BaseEvent {
  type: "task.completed";
  taskId: string;
  assignmentId: string;
  agentId: string;
  durationMs: number;
  success: boolean;
  result?: any;
  qualityScore?: number;
}

export interface TaskFailedEvent extends BaseEvent {
  type: "task.failed";
  taskId: string;
  assignmentId: string;
  agentId: string;
  durationMs: number;
  error: string;
  errorCode?: string;
  retryCount: number;
  canRetry: boolean;
}

export interface TaskTimeoutEvent extends BaseEvent {
  type: "task.timeout";
  taskId: string;
  assignmentId: string;
  agentId: string;
  timeoutType: "acknowledgment" | "execution" | "progress";
  timeoutMs: number;
  deadline: Date;
}

export interface TaskReassignedEvent extends BaseEvent {
  type: "task.reassigned";
  taskId: string;
  oldAssignmentId: string;
  newAssignmentId: string;
  oldAgentId: string;
  newAgentId: string;
  reason: string;
  attemptNumber: number;
}

/**
 * Security Events
 */
export interface AgentAuthenticatedEvent extends BaseEvent {
  type: "security.authenticated";
  agentId: string;
  sessionId: string;
  securityLevel: string;
  ipAddress?: string;
  userAgent?: string;
}

export interface AuthenticationFailedEvent extends BaseEvent {
  type: "security.auth_failed";
  agentId?: string;
  reason: string;
  ipAddress?: string;
  attempts: number;
}

export interface AuthorizationFailedEvent extends BaseEvent {
  type: "security.authz_failed";
  agentId?: string;
  permission: string;
  resource?: string;
  reason: string;
}

export interface RateLimitExceededEvent extends BaseEvent {
  type: "security.rate_limit_exceeded";
  agentId?: string;
  action: string;
  limit: number;
  windowMs: number;
  blockedUntil: Date;
}

export interface SecurityViolationEvent extends BaseEvent {
  type: "security.violation";
  agentId?: string;
  violationType: string;
  details: Record<string, any>;
  severity: EventSeverity;
}

export interface SessionExpiredEvent extends BaseEvent {
  type: "security.session_expired";
  agentId?: string;
  sessionId: string;
  durationMs: number;
}

/**
 * System Health Events
 */
export interface SystemHealthCheckEvent extends BaseEvent {
  type: "system.health_check";
  component: string;
  status: "healthy" | "degraded" | "unhealthy";
  metrics: Record<string, any>;
  responseTimeMs: number;
}

export interface SystemResourceAlertEvent extends BaseEvent {
  type: "system.resource_alert";
  component: string;
  resource: string;
  currentValue: number;
  threshold: number;
  severity: EventSeverity;
}

export interface SystemFailureEvent extends BaseEvent {
  type: "system.failure";
  component: string;
  failureType: string;
  error: string;
  recoveryAction?: string;
  impact: "low" | "medium" | "high" | "critical";
}

export interface SystemRecoveryEvent extends BaseEvent {
  type: "system.recovery";
  component: string;
  failureType: string;
  recoveryAction: string;
  durationMs: number;
  success: boolean;
}

/**
 * Database Events
 */
export interface DatabaseOperationEvent extends BaseEvent {
  type: "database.operation";
  operation: string;
  table: string;
  durationMs: number;
  rowsAffected?: number;
  success: boolean;
  error?: string;
}

export interface DatabaseConnectionEvent extends BaseEvent {
  type: "database.connection";
  operation: "connect" | "disconnect" | "reconnect";
  success: boolean;
  durationMs?: number;
  error?: string;
}

export interface DatabaseMigrationEvent extends BaseEvent {
  type: "database.migration";
  migrationId: string;
  direction: "up" | "down";
  success: boolean;
  durationMs: number;
  error?: string;
}

/**
 * Performance Events
 */
export interface PerformanceMetricEvent extends BaseEvent {
  type: "performance.metric";
  component: string;
  metric: string;
  value: number;
  unit: string;
  threshold?: number;
  anomaly?: boolean;
}

export interface SlowOperationEvent extends BaseEvent {
  type: "performance.slow_operation";
  component: string;
  operation: string;
  durationMs: number;
  thresholdMs: number;
  parameters?: Record<string, any>;
}

/**
 * Configuration Events
 */
export interface ConfigurationChangedEvent extends BaseEvent {
  type: "config.changed";
  component: string;
  key: string;
  oldValue?: any;
  newValue: any;
  changedBy?: string;
}

export interface ConfigurationValidationEvent extends BaseEvent {
  type: "config.validation";
  component: string;
  isValid: boolean;
  errors?: string[];
  warnings?: string[];
}

/**
 * CAWS Events
 */
export interface CAWSValidationEvent extends BaseEvent {
  type: "caws.validation";
  taskId?: string;
  specId?: string;
  verdict: "pass" | "fail" | "waiver";
  violations?: string[];
  waivers?: string[];
  confidence?: number;
}

export interface CAWSComplianceEvent extends BaseEvent {
  type: "caws.compliance";
  component: string;
  rule: string;
  compliant: boolean;
  details?: Record<string, any>;
}

/**
 * Orchestrator Lifecycle Events
 */
export interface OrchestratorStartedEvent extends BaseEvent {
  type: "orchestrator.started";
  version: string;
  config: Record<string, any>;
  components: string[];
}

export interface OrchestratorShutdownEvent extends BaseEvent {
  type: "orchestrator.shutdown";
  uptimeMs: number;
  reason?: string;
  cleanShutdown: boolean;
}

export interface OrchestratorRestartEvent extends BaseEvent {
  type: "orchestrator.restart";
  reason: string;
  downtimeMs?: number;
}

/**
 * Union type of all orchestrator events
 */
export type OrchestratorEvent =
  // Task events
  | TaskEnqueuedEvent
  | TaskDequeuedEvent
  | TaskQueueFullEvent
  | TaskQueueClearedEvent
  | TaskAssignedEvent
  | TaskAssignmentAcknowledgedEvent
  | TaskProgressUpdatedEvent
  | TaskCompletedEvent
  | TaskFailedEvent
  | TaskTimeoutEvent
  | TaskReassignedEvent
  // Security events
  | AgentAuthenticatedEvent
  | AuthenticationFailedEvent
  | AuthorizationFailedEvent
  | RateLimitExceededEvent
  | SecurityViolationEvent
  | SessionExpiredEvent
  // System events
  | SystemHealthCheckEvent
  | SystemResourceAlertEvent
  | SystemFailureEvent
  | SystemRecoveryEvent
  // Database events
  | DatabaseOperationEvent
  | DatabaseConnectionEvent
  | DatabaseMigrationEvent
  // Performance events
  | PerformanceMetricEvent
  | SlowOperationEvent
  // Configuration events
  | ConfigurationChangedEvent
  | ConfigurationValidationEvent
  // CAWS events
  | CAWSValidationEvent
  | CAWSComplianceEvent
  // Lifecycle events
  | OrchestratorStartedEvent
  | OrchestratorShutdownEvent
  | OrchestratorRestartEvent;

/**
 * Event type registry for easy access
 */
export const EventTypes = {
  // Task events
  TASK_ENQUEUED: "task.enqueued" as const,
  TASK_DEQUEUED: "task.dequeued" as const,
  TASK_QUEUE_FULL: "task.queue_full" as const,
  TASK_QUEUE_CLEARED: "task.queue_cleared" as const,
  TASK_ASSIGNED: "task.assigned" as const,
  TASK_ASSIGNMENT_ACKNOWLEDGED: "task.assignment_acknowledged" as const,
  TASK_PROGRESS_UPDATED: "task.progress_updated" as const,
  TASK_COMPLETED: "task.completed" as const,
  TASK_FAILED: "task.failed" as const,
  TASK_TIMEOUT: "task.timeout" as const,
  TASK_REASSIGNED: "task.reassigned" as const,

  // Security events
  AGENT_AUTHENTICATED: "security.authenticated" as const,
  AUTHENTICATION_FAILED: "security.auth_failed" as const,
  AUTHORIZATION_FAILED: "security.authz_failed" as const,
  RATE_LIMIT_EXCEEDED: "security.rate_limit_exceeded" as const,
  SECURITY_VIOLATION: "security.violation" as const,
  SESSION_EXPIRED: "security.session_expired" as const,

  // System events
  SYSTEM_HEALTH_CHECK: "system.health_check" as const,
  SYSTEM_RESOURCE_ALERT: "system.resource_alert" as const,
  SYSTEM_FAILURE: "system.failure" as const,
  SYSTEM_RECOVERY: "system.recovery" as const,

  // Database events
  DATABASE_OPERATION: "database.operation" as const,
  DATABASE_CONNECTION: "database.connection" as const,
  DATABASE_MIGRATION: "database.migration" as const,

  // Performance events
  PERFORMANCE_METRIC: "performance.metric" as const,
  SLOW_OPERATION: "performance.slow_operation" as const,

  // Configuration events
  CONFIG_CHANGED: "config.changed" as const,
  CONFIG_VALIDATION: "config.validation" as const,

  // CAWS events
  CAWS_VALIDATION: "caws.validation" as const,
  CAWS_COMPLIANCE: "caws.compliance" as const,

  // Lifecycle events
  ORCHESTRATOR_STARTED: "orchestrator.started" as const,
  ORCHESTRATOR_SHUTDOWN: "orchestrator.shutdown" as const,
  ORCHESTRATOR_RESTART: "orchestrator.restart" as const,
} as const;
