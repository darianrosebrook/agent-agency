/**
 * @fileoverview TypeScript type definitions for Arbiter Orchestration (ARBITER-005)
 *
 * This file defines all types used by the ArbiterOrchestrator and related components
 * for task routing, CAWS enforcement, and performance tracking.
 *
 * @author @darianrosebrook
 */

import {
  AgentCapabilities,
  AgentProfile,
  PerformanceMetrics,
} from "./agent-registry";

// Re-export for convenience
export type { AgentProfile } from "./agent-registry";

// Re-export commonly used types
export { VerificationPriority } from "./verification";

/**
 * Task type enumeration
 */
export type TaskType =
  | "code-editing"
  | "code-review"
  | "analysis"
  | "research"
  | "validation"
  | "general";

/**
 * Core task lifecycle types
 */

export interface Task {
  /** Unique task identifier */
  id: string;

  /** Human-readable task description */
  description: string;

  /** Task type classification */
  type: TaskType;

  /** Required capabilities for task execution */
  requiredCapabilities: Partial<AgentCapabilities>;

  /** Task priority (1-10, higher = more urgent) */
  priority: number;

  /** Maximum allowed execution time in milliseconds */
  timeoutMs: number;

  /** CAWS budget constraints */
  budget: {
    maxFiles: number;
    maxLoc: number;
  };

  /** Task creation timestamp */
  createdAt: Date;

  /** Additional metadata */
  metadata: Record<string, any>;

  /** Current attempt count */
  attempts: number;

  /** Maximum allowed attempts */
  maxAttempts: number;
}

export interface TaskRequest extends Omit<Task, "id" | "createdAt"> {
  /** Optional client-provided ID */
  requestedId?: string;
}

export enum TaskStatus {
  QUEUED = "queued",
  ROUTING = "routing",
  ASSIGNED = "assigned",
  EXECUTING = "executing",
  VALIDATING = "validating",
  COMPLETED = "completed",
  FAILED = "failed",
  TIMEOUT = "timeout",
  CANCELED = "canceled",
}

export interface TaskState {
  task: Task;
  status: TaskStatus;
  assignedAgentId?: string;
  startedAt?: Date;
  completedAt?: Date;
  attempts: number;
  maxAttempts: number;
  lastError?: string;
  routingHistory: RoutingDecision[];
}

/**
 * Routing and assignment types
 */

export interface RoutingDecision {
  /** Decision identifier */
  id: string;

  /** Task being routed */
  taskId: string;

  /** Selected agent profile */
  selectedAgent: AgentProfile;

  /** Routing confidence score (0-1) */
  confidence: number;

  /** Reason for selection */
  reason: string;

  /** Routing strategy used */
  strategy: "epsilon-greedy" | "ucb" | "capability-match" | "load-balance";

  /** Alternative agents considered */
  alternatives: Array<{
    agent: AgentProfile;
    score: number;
    reason: string;
  }>;

  /** Decision timestamp */
  timestamp: Date;
}

export interface TaskAssignment {
  /** Assignment identifier */
  id: string;

  /** Task being assigned */
  task: Task;

  /** Assigned agent */
  agent: AgentProfile;

  /** Routing decision that led to this assignment */
  routingDecision: RoutingDecision;

  /** Assignment timestamp */
  assignedAt: Date;

  /** Expected completion deadline */
  deadline: Date;
}

/**
 * Execution and result types
 */

export interface TaskExecution {
  /** Execution identifier */
  id: string;

  /** Assignment being executed */
  assignment: TaskAssignment;

  /** Execution start timestamp */
  startedAt: Date;

  /** Current execution status */
  status: "running" | "paused" | "resumed" | "completed" | "failed";

  /** Progress indicator (0-1) */
  progress: number;

  /** Execution metadata */
  metadata: Record<string, any>;
}

export interface TaskResult {
  /** Result identifier */
  id: string;

  /** Task that was executed */
  task: Task;

  /** Assigned agent */
  agent: AgentProfile;

  /** Execution success indicator */
  success: boolean;

  /** Task output/result data */
  output: any;

  /** Performance metrics from execution */
  performance: PerformanceMetrics;

  /** Quality score (0-1) */
  qualityScore: number;

  /** Execution errors (if any) */
  errors: string[];

  /** CAWS validation result */
  cawsValidation?: CAWSValidationResult;

  /** Completion timestamp */
  completedAt: Date;

  /** Total execution time in milliseconds */
  executionTimeMs: number;
}

/**
 * CAWS validation types
 */

export interface CAWSValidationResult {
  /** Validation identifier */
  id: string;

  /** Task result being validated */
  taskResultId: string;

  /** Overall validation outcome */
  passed: boolean;

  /** CAWS version used for validation */
  cawsVersion: string;

  /** Budget compliance check */
  budgetCompliance: {
    passed: boolean;
    filesUsed: number;
    filesLimit: number;
    locUsed: number;
    locLimit: number;
    violations: string[];
  };

  /** Quality gate results */
  qualityGates: Array<{
    name: string;
    passed: boolean;
    score?: number;
    threshold?: number;
    details?: string;
  }>;

  /** Waiver applications */
  waivers: Array<{
    id: string;
    reason: string;
    approved: boolean;
    justification?: string;
  }>;

  /** Validation verdict */
  verdict: "pass" | "fail" | "waiver-required";

  /** Remediation guidance */
  remediation?: string[];

  /** Validation timestamp */
  validatedAt: Date;

  /** Cryptographic signature */
  signature: string;
}

/**
 * Health monitoring types
 */

export interface HealthStatus {
  /** Component identifier */
  component: string;

  /** Overall health status */
  status: "healthy" | "degraded" | "unhealthy" | "unknown";

  /** Health check timestamp */
  checkedAt: Date;

  /** Detailed health metrics */
  metrics: Record<string, number>;

  /** Health issues detected */
  issues: Array<{
    severity: "low" | "medium" | "high" | "critical";
    message: string;
    details?: any;
  }>;

  /** Recovery actions taken */
  recoveryActions: Array<{
    action: string;
    timestamp: Date;
    success: boolean;
    details?: string;
  }>;
}

export interface AgentHealth extends HealthStatus {
  /** Agent identifier */
  agentId: string;

  /** Agent responsiveness score (0-1) */
  responsiveness: number;

  /** Current task load */
  currentTasks: number;

  /** Task success rate (0-1) */
  successRate: number;

  /** Average response time in milliseconds */
  averageResponseTimeMs: number;

  /** Last successful task completion */
  lastSuccessfulTask?: Date;

  /** Consecutive failures */
  consecutiveFailures: number;
}

export interface SystemHealth extends HealthStatus {
  /** Queue health */
  queueDepth: number;
  queueProcessingRate: number;

  /** Resource utilization */
  cpuUtilization: number;
  memoryUtilization: number;
  diskUtilization: number;

  /** Database connectivity */
  databaseHealthy: boolean;
  databaseLatencyMs: number;

  /** External service health */
  externalServices: Record<string, boolean>;

  /** System load */
  systemLoad: number;
  processUptime: number;
}

/**
 * Recovery and resilience types
 */

export interface RecoveryAction {
  /** Action identifier */
  id: string;

  /** Failed component */
  component: string;

  /** Recovery strategy */
  strategy:
    | "restart"
    | "failover"
    | "circuit-breaker"
    | "load-shedding"
    | "reassignment";

  /** Action priority (higher number = higher priority) */
  priority: number;

  /** Action status */
  status: "pending" | "in-progress" | "completed" | "failed";

  /** Recovery parameters */
  parameters: Record<string, any>;

  /** Action creation timestamp */
  createdAt: Date;

  /** Action completion timestamp */
  completedAt?: Date;

  /** Success indicator */
  success?: boolean;

  /** Error details (if failed) */
  error?: string;
}

export interface CircuitBreakerState {
  /** Component identifier */
  component: string;

  /** Current state */
  state: "closed" | "open" | "half-open";

  /** Failure count */
  failureCount: number;

  /** Success count */
  successCount: number;

  /** Last failure timestamp */
  lastFailureAt?: Date;

  /** Last success timestamp */
  lastSuccessAt?: Date;

  /** Configuration */
  config: {
    failureThreshold: number;
    recoveryTimeoutMs: number;
    successThreshold: number;
  };
}

/**
 * Orchestrator configuration types
 */

export interface ArbiterConfiguration {
  /** Orchestrator instance identifier */
  instanceId: string;

  /** Queue configuration */
  queue: {
    maxCapacity: number;
    processingConcurrency: number;
    taskTimeoutMs: number;
    retryAttempts: number;
  };

  /** Routing configuration */
  routing: {
    explorationRate: number;
    confidenceThreshold: number;
    maxAlternatives: number;
    routingTimeoutMs: number;
  };

  /** Health monitoring configuration */
  health: {
    checkIntervalMs: number;
    unhealthyThreshold: number;
    recoveryTimeoutMs: number;
    alertThresholds: Record<string, number>;
  };

  /** CAWS enforcement configuration */
  caws: {
    validationTimeoutMs: number;
    strictMode: boolean;
    autoWaiverApproval: boolean;
    provenanceEnabled: boolean;
  };

  /** Performance tracking configuration */
  performance: {
    collectionEnabled: boolean;
    retentionPeriodDays: number;
    samplingRate: number;
    metricsExportEnabled: boolean;
  };

  /** Security configuration */
  security: {
    authenticationRequired: boolean;
    authorizationEnabled: boolean;
    auditLoggingEnabled: boolean;
    encryptionEnabled: boolean;
  };
}

/**
 * Orchestrator state and statistics
 */

export interface OrchestratorStats {
  /** Statistics timestamp */
  timestamp: Date;

  /** Queue statistics */
  queue: {
    depth: number;
    processingRate: number;
    averageWaitTimeMs: number;
    maxWaitTimeMs: number;
  };

  /** Routing statistics */
  routing: {
    totalDecisions: number;
    averageConfidence: number;
    successRate: number;
    averageDecisionTimeMs: number;
  };

  /** Agent statistics */
  agents: {
    totalRegistered: number;
    activeAgents: number;
    averageUtilization: number;
    averageSuccessRate: number;
  };

  /** Task statistics */
  tasks: {
    totalProcessed: number;
    completionRate: number;
    averageExecutionTimeMs: number;
    failureRate: number;
  };

  /** CAWS compliance statistics */
  caws: {
    validationRate: number;
    complianceRate: number;
    waiverRate: number;
    averageValidationTimeMs: number;
  };

  /** System health */
  health: {
    overallStatus: "healthy" | "degraded" | "unhealthy";
    componentHealth: Record<string, "healthy" | "degraded" | "unhealthy">;
    activeAlerts: number;
  };
}

/**
 * Event and notification types
 */

export interface OrchestratorEvent {
  /** Event identifier */
  id: string;

  /** Event type */
  type:
    | "task_queued"
    | "task_assigned"
    | "task_completed"
    | "task_failed"
    | "agent_registered"
    | "agent_unregistered"
    | "health_changed"
    | "validation_passed"
    | "validation_failed"
    | "recovery_action";

  /** Event timestamp */
  timestamp: Date;

  /** Event data */
  data: Record<string, any>;

  /** Event severity */
  severity: "info" | "warning" | "error" | "critical";

  /** Related entity IDs */
  relatedIds: {
    taskId?: string;
    agentId?: string;
    assignmentId?: string;
    validationId?: string;
  };
}

/**
 * Error types
 */

export class OrchestratorError extends Error {
  constructor(message: string, public code: string, public details?: any) {
    super(message);
    this.name = "OrchestratorError";
  }
}

export class TaskTimeoutError extends OrchestratorError {
  constructor(taskId: string, timeoutMs: number) {
    super(`Task ${taskId} exceeded timeout of ${timeoutMs}ms`, "TASK_TIMEOUT", {
      taskId,
      timeoutMs,
    });
  }
}

export class RoutingFailureError extends OrchestratorError {
  constructor(taskId: string, reason: string) {
    super(`Failed to route task ${taskId}: ${reason}`, "ROUTING_FAILURE", {
      taskId,
      reason,
    });
  }
}

export class ValidationFailureError extends OrchestratorError {
  constructor(taskId: string, violations: string[]) {
    super(`Task ${taskId} failed CAWS validation`, "VALIDATION_FAILURE", {
      taskId,
      violations,
    });
  }
}

export class HealthFailureError extends OrchestratorError {
  constructor(component: string, issues: string[]) {
    super(`Health check failed for ${component}`, "HEALTH_FAILURE", {
      component,
      issues,
    });
  }
}

/**
 * Service interfaces
 */

export interface IArbiterOrchestrator {
  initialize(): Promise<void>;
  shutdown(): Promise<void>;

  submitTask(request: TaskRequest): Promise<Task>;
  getTaskStatus(taskId: string): Promise<TaskState>;
  cancelTask(taskId: string): Promise<boolean>;

  getStats(): Promise<OrchestratorStats>;
  getHealth(): Promise<SystemHealth>;

  registerAgent(profile: AgentProfile): Promise<void>;
  unregisterAgent(agentId: string): Promise<void>;
}

export interface ITaskQueue {
  enqueue(task: Task): Promise<void>;
  dequeue(): Promise<Task | null>;
  peek(): Promise<Task | null>;
  size(): Promise<number>;
  clear(): Promise<void>;
}

export interface ITaskRouter {
  route(task: Task): Promise<RoutingDecision>;
  getRoutingStats(): Promise<Record<string, any>>;
}

export interface ICAWSValidator {
  validate(result: TaskResult): Promise<CAWSValidationResult>;
  getValidationStats(): Promise<Record<string, any>>;
}

export interface IPerformanceTracker {
  trackExecution(execution: TaskExecution): Promise<void>;
  trackResult(result: TaskResult): Promise<void>;
  getMetrics(query: any): Promise<PerformanceMetrics[]>;
}

export interface IHealthMonitor {
  checkHealth(): Promise<SystemHealth>;
  checkAgentHealth(agentId: string): Promise<AgentHealth>;
  startMonitoring(): Promise<void>;
  stopMonitoring(): Promise<void>;
}

export interface IRecoveryManager {
  handleFailure(component: string, error: Error): Promise<RecoveryAction>;
  executeRecovery(action: RecoveryAction): Promise<boolean>;
  getRecoveryHistory(limit?: number): RecoveryAction[];
}
