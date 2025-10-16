/**
 * Task Runner Types - ARBITER-014
 *
 * Type definitions for task orchestration, worker management,
 * pleading workflows, and execution metrics.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


// Import artifact types
import type { ArtifactManifest } from "../orchestrator/workers/ArtifactSandbox.js";

export enum TaskStatus {
  PENDING = "pending",
  QUEUED = "queued",
  ASSIGNED = "assigned",
  RUNNING = "running",
  SUSPENDED = "suspended",
  COMPLETED = "completed",
  FAILED = "failed",
  CANCELLED = "cancelled",
}

export enum TaskPriority {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

export enum TaskOrchestratorEvents {
  TASK_SUBMITTED = "task_submitted",
  TASK_STARTED = "task_started",
  TASK_COMPLETED = "task_completed",
  TASK_FAILED = "task_failed",
  TASK_RETRY_SCHEDULED = "task_retry_scheduled",
  PLEADING_INITIATED = "pleading_initiated",
  PLEADING_APPROVED = "pleading_approved",
  PLEADING_DENIED = "pleading_denied",
  WORKER_CREATED = "worker_created",
  WORKER_DESTROYED = "worker_destroyed",
}

export interface Task {
  id: string;
  type: "script" | "api_call" | "data_processing" | "ai_inference";
  priority: TaskPriority;
  payload: any;
  metadata?: Record<string, any>;
  assignedAgent?: string;
  createdAt: Date;
  timeout?: number; // milliseconds
  retries?: number;
  dependencies?: string[]; // task IDs this task depends on
  description?: string;
  requiredCapabilities?: Record<string, any>;
  budget?: {
    maxFiles: number;
    maxLoc: number;
  };
  traceId?: string;
  artifactRoot?: string;
}

export interface TaskResult {
  taskId: string;
  status: TaskStatus;
  result?: any;
  error?: string;
  metrics: TaskMetrics;
  completedAt: Date;
}

export interface TaskExecution {
  executionId: string;
  taskId: string;
  agentId: string;
  startedAt: Date;
  completedAt?: Date;
  status: "running" | "completed" | "failed" | "cancelled";
  attempts: number;
  result?: WorkerExecutionResult;
  error?: string;
  retryScheduled?: Date;
  artifacts?: {
    manifest: ArtifactManifest;
    rootPath: string;
  };
}

export interface TaskMetrics {
  taskId: string;
  workerId: string;
  startTime: number;
  endTime: number;
  cpuUsage: number;
  memoryUsage: number;
  executionTime: number;
  status: "idle" | "running" | "completed" | "failed";
}

export interface WorkerExecutionResult {
  success: boolean;
  result?: any;
  error?: string;
  metrics: {
    executionTime: number;
    cpuUsage: number;
    memoryUsage: number;
    outputSize?: number;
  };
  logs?: string[];
  artifacts?: {
    manifest: ArtifactManifest;
    rootPath: string;
  };
}

export interface WorkerPoolConfig {
  minPoolSize: number;
  maxPoolSize: number;
  workerCapabilities: string[];
  workerTimeout: number;
  workerMemoryLimit?: number;
  artifactConfig?: {
    rootPath: string;
    maxFileSizeBytes: number;
    maxTotalFiles: number;
    maxPathLength: number;
  };
  workerCpuLimit?: number;
}

export interface PleadingWorkflow {
  workflowId: string;
  taskId: string;
  initiatedAt: Date;
  completedAt?: Date;
  status: "active" | "approved" | "denied";
  context: any;
  decisions: PleadingDecision[];
  requiredApprovals: number;
  currentApprovals: number;
}

export interface PleadingDecision {
  decisionId: string;
  taskId: string;
  workflowId: string;
  approverId: string;
  decision: "approve" | "deny" | "escalate";
  reasoning: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}

export interface TaskOrchestratorCapabilities {
  maxConcurrentTasks: number;
  supportedTaskTypes: string[];
  pleadingSupport: boolean;
  retrySupport: boolean;
  isolationLevel: string;
  monitoringEnabled: boolean;
  metricsEnabled: boolean;
}

export interface TaskOrchestratorConfig {
  workerPool: WorkerPoolConfig;
  queue: {
    maxSize: number;
    priorityLevels: TaskPriority[];
    persistenceEnabled: boolean;
  };
  retry: {
    maxAttempts: number;
    backoffMultiplier: number;
    initialDelay: number;
    maxDelay: number;
  };
  routing: {
    enabled: boolean;
    strategy: "round_robin" | "load_balanced" | "priority_based";
  };
  performance: {
    trackingEnabled: boolean;
    metricsInterval: number;
  };
  pleading: {
    enabled: boolean;
    requiredApprovals: number;
    timeoutHours: number;
  };
}
