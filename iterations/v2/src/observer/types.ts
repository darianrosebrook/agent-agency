/**
 * Arbiter Observer Types
 *
 * Shared type definitions for the observer bridge, covering configuration,
 * persistence contracts, API payloads, and streaming semantics.
 *
 * @author @darian
 */

export type ObserverPrivacyMode = "standard" | "strict";

/**
 * Redaction rule definition.
 * `pattern` should be a global regular expression compiled at runtime.
 */
export interface RedactionRule {
  name: string;
  pattern: RegExp;
  replacement?: string;
}

/**
 * Observer configuration resolved from environment/config files.
 */
export interface ObserverConfig {
  bind: string;
  port: number;
  socketPath?: string | null;
  authToken?: string;
  allowedOrigins: Set<string>;
  dataDir: string;
  maxClients: number;
  flushIntervalMs: number;
  maxQueueSize: number;
  rotateMB: number;
  retentionDays: number;
  sampleRates: Record<string, number>;
  redactionRules: RedactionRule[];
  privacyMode: ObserverPrivacyMode;
  heartbeatIntervalMs: number;
}

/**
 * Minimal representation of an observer event persisted to JSONL.
 * Additional metadata (seq, schemaVersion, etc.) will be appended in the
 * persistence layer; server code only needs the typed payload.
 */
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

export interface ObserverStatusSummary {
  status: "running" | "stopped" | "degraded";
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

/**
 * Contract for the data store / persistence layer.
 * The HTTP server depends on these methods; implementations live in the
 * persistence module.
 */
export interface ObserverStore {
  getStatus(): ObserverStatusSummary;
  getMetrics(): ObserverMetricsSnapshot;
  getProgress(): ObserverProgressSummary;
  listEvents(_params: {
    cursor?: string;
    limit?: number;
    since?: Date;
    until?: Date;
    type?: string;
    taskId?: string;
    severity?: "debug" | "info" | "warn" | "error";
  }): Promise<{
    events: ObserverEventPayload[];
    nextCursor?: string;
  }>;
  listChainOfThought(_params: {
    taskId?: string;
    cursor?: string;
    limit?: number;
    since?: Date;
  }): Promise<{
    entries: ChainOfThoughtEntry[];
    nextCursor?: string;
  }>;
  getTask(_taskId: string): Promise<{
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
} | null>;
  appendObservation(_note: {
    message: string;
    taskId?: string;
    author?: string;
  }): Promise<{ id: string; timestamp: string }>;
}

/**
 * Contract for SSE streaming so multiple consumers can subscribe to live
 * observer events. The HTTP server orchestrates subscription/unsubscription
 * but delegates queueing to an implementation.
 */
export interface ObserverSseClient {
  id: string;
  res: import("http").ServerResponse;
  filters: {
    taskId?: string;
    type?: string;
    severity?: "debug" | "info" | "warn" | "error";
  };
  verbose: boolean;
  connectedAt: number;
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

export interface ArbiterController {
  ensureArbiterRunning(): Promise<{ status: "running" | "starting" }>;
  requestArbiterStop(): Promise<{ status: "stopping" | "stopped" }>;
  submitTask(_payload: SubmitTaskPayload): Promise<SubmitTaskResult>;
  executeCommand(_command: string): Promise<{ acknowledged: boolean; note?: string }>;
}
