// API types for the Observer HTTP interface
// Based on the observer bridge types

export type ObserverStatus = "running" | "stopped" | "degraded";

export interface ObserverStatusSummary {
  status: ObserverStatus;
  startedAt: string;
  uptimeMs: number;
  queueDepth: number;
  maxQueueSize: number;
  observerDegraded: boolean;
  lastFlushMs: number;
  activeFile?: string;
  backpressureEvents: number;
  authConfigured: boolean;
}

export interface ObserverMetricsSnapshot {
  timestamp: string;
  reasoningDepthAvg: number;
  reasoningDepthP95: number;
  debateBreadthAvg: number;
  taskSuccessRate: number;
  toolBudgetUtilization: number;
  activeTasks: number;
  queuedTasks: number;
  policyViolations: number;
  observerDegraded: boolean;
  queueDepth: number;
}

export interface ObserverProgressSummary {
  status: "not_started" | "running" | "completed" | "degraded";
  reasoningSteps: {
    observations: number;
    analyses: number;
    plans: number;
    decisions: number;
    executions: number;
    verifications: number;
  };
  totalReasoningSteps: number;
  uptimeMinutes: number;
}

export interface ObserverEventPayload {
  id: string;
  type: string;
  severity: "debug" | "info" | "warn" | "error";
  source: string;
  taskId?: string;
  agentId?: string;
  timestamp: string;
  traceId?: string;
  spanId?: string;
  correlationId?: string;
  metadata?: Record<string, unknown>;
}

export interface ChainOfThoughtEntry {
  id: string;
  taskId: string;
  sessionId?: string;
  phase:
    | "observation"
    | "analysis"
    | "plan"
    | "decision"
    | "execute"
    | "verify"
    | "hypothesis"
    | "critique"
    | "other";
  agentId?: string;
  agentRole?: string;
  timestamp: string;
  confidence?: number;
  content?: string;
  redacted?: boolean;
  hash?: string;
  traceId?: string;
  spanId?: string;
}

export interface Task {
  taskId: string;
  state: string;
  progress: string[];
  lastUpdated: string;
  currentPlan?: string;
  nextActions?: string[];
  redacted?: boolean;
  caws?: {
    passed: boolean;
    verdict: string;
    remediation?: string[];
  };
  verification?: {
    verdict: string;
    confidence: number;
    reasoning: string[];
  };
}

export interface SubmitTaskPayload {
  description: string;
  specPath?: string;
  metadata?: Record<string, unknown>;
}

export interface SubmitTaskResult {
  taskId: string;
  assignmentId?: string;
  queued?: boolean;
  overrideRequired?: string;
}

export interface ObservationResult {
  id: string;
  timestamp: string;
}

export interface CommandResult {
  acknowledged: boolean;
  note?: string;
}

export interface ArbiterControlResult {
  status: "running" | "starting" | "stopping" | "stopped";
}

export interface EventListResult {
  events: ObserverEventPayload[];
  nextCursor?: string;
}

export interface ChainOfThoughtListResult {
  entries: ChainOfThoughtEntry[];
  nextCursor?: string;
}

// API Request/Response types
export interface ApiResponse<T> {
  data: T;
  error?: string;
}

export interface PaginatedParams {
  cursor?: string;
  limit?: number;
}

export interface EventFilters extends PaginatedParams {
  severity?: "debug" | "info" | "warn" | "error";
  type?: string;
  taskId?: string;
  sinceTs?: string;
  untilTs?: string;
}

export interface ChainOfThoughtFilters extends PaginatedParams {
  taskId?: string;
  since?: string;
}
