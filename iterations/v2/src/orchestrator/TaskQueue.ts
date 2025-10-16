/**
 * Task Queue Management
 *
 * Manages queued tasks and processing state for the orchestrator.
 *
 * @author @darianrosebrook
 */

import { Buffer } from "node:buffer";
import { EventEmitter } from "events";
import { Logger } from "@/observability/Logger";
import type { AuthCredentials, SecurityContext } from "./SecurityManager";
import {
  Permission,
  SecurityError,
  SecurityManager,
  SecurityMiddleware,
} from "./SecurityManager";
import { Task } from "../types/arbiter-orchestration";
import { TaskQueueStats } from "../types/orchestrator-events";

export class TaskQueue extends EventEmitter {
  private queue: Task[] = [];
  private processing: Map<string, Task> = new Map();
  private timestamps: Map<string, Date> = new Map();

  /**
   * Add task to queue
   */
  enqueue(task: Task): void {
    if (this.hasTask(task.id)) {
      throw new Error(`Task ${task.id} is already in queue`);
    }

    this.queue.push(task);
    this.timestamps.set(task.id, new Date());

    this.emit("task:enqueued", { taskId: task.id, task });
  }

  /**
   * Get next task from queue
   */
  dequeue(): Task | undefined {
    const task = this.queue.shift();
    if (task) {
      this.processing.set(task.id, task);
      this.emit("task:dequeued", { taskId: task.id, task });
    }
    return task;
  }

  /**
   * Peek at next task without removing it
   */
  peek(): Task | undefined {
    return this.queue[0];
  }

  /**
   * Remove task from queue or processing
   */
  remove(taskId: string): boolean {
    // Check queue first
    const queueIndex = this.queue.findIndex((t) => t.id === taskId);
    if (queueIndex >= 0) {
      this.queue.splice(queueIndex, 1);
      this.timestamps.delete(taskId);
      this.emit("task:removed", { taskId, from: "queue" });
      return true;
    }

    // Check processing
    if (this.processing.has(taskId)) {
      this.processing.delete(taskId);
      this.timestamps.delete(taskId);
      this.emit("task:removed", { taskId, from: "processing" });
      return true;
    }

    return false;
  }

  /**
   * Check if task exists in queue or processing
   */
  hasTask(taskId: string): boolean {
    return this.isQueued(taskId) || this.isProcessing(taskId);
  }

  /**
   * Check if task is queued
   */
  isQueued(taskId: string): boolean {
    return this.queue.some((t) => t.id === taskId);
  }

  /**
   * Check if task is being processed
   */
  isProcessing(taskId: string): boolean {
    return this.processing.has(taskId);
  }

  /**
   * Mark task as no longer processing (completed/failed)
   */
  complete(taskId: string): boolean {
    if (this.processing.has(taskId)) {
      this.processing.delete(taskId);
      this.timestamps.delete(taskId);
      this.emit("task:completed", { taskId });
      return true;
    }
    return false;
  }

  /**
   * Get queue statistics
   */
  getStats(): TaskQueueStats {
    const queuedTimestamps = this.queue
      .map((task) => this.timestamps.get(task.id))
      .filter(Boolean) as Date[];

    const oldestQueued =
      queuedTimestamps.length > 0
        ? new Date(Math.min(...queuedTimestamps.map((t) => t.getTime())))
        : undefined;

    return {
      queued: this.queue.length,
      processing: this.processing.size,
      total: this.queue.length + this.processing.size,
      oldestQueued,
    };
  }

  /**
   * Get all queued tasks
   */
  getQueuedTasks(): Task[] {
    return [...this.queue];
  }

  /**
   * Get all processing tasks
   */
  getProcessingTasks(): Task[] {
    return Array.from(this.processing.values());
  }

  /**
   * Get task by ID
   */
  getTask(taskId: string): Task | undefined {
    return (
      this.queue.find((t) => t.id === taskId) || this.processing.get(taskId)
    );
  }

  /**
   * Get queue size
   */
  size(): number {
    return this.queue.length;
  }

  /**
   * Check if queue is empty
   */
  isEmpty(): boolean {
    return this.queue.length === 0;
  }

  /**
   * Clear all tasks
   */
  clear(): void {
    const taskIds = [
      ...this.queue.map((t) => t.id),
      ...Array.from(this.processing.keys()),
    ];

    this.queue = [];
    this.processing.clear();
    this.timestamps.clear();

    this.emit("queue:cleared", { taskIds });
  }

  /**
   * Get tasks older than specified duration
   */
  getStaleTasks(maxAgeMs: number): Task[] {
    const now = Date.now();
    const stale: Task[] = [];

    // Check queued tasks
    for (const task of Array.from(this.queue)) {
      const timestamp = this.timestamps.get(task.id);
      if (timestamp && now - timestamp.getTime() > maxAgeMs) {
        stale.push(task);
      }
    }

    // Check processing tasks
    for (const task of Array.from(this.processing.values())) {
      const timestamp = this.timestamps.get(task.id);
      if (timestamp && now - timestamp.getTime() > maxAgeMs) {
        stale.push(task);
      }
    }

    return stale;
  }

  /**
   * Enqueue task with credentials (stub for security integration)
   */
  async enqueueWithCredentials(
    task: Task,
    credentials: AuthCredentials,
    securityOrOptions?: SecurityManager | SecureTaskQueueOptions,
    maybeOptions?: SecureTaskQueueOptions
  ): Promise<void> {
    let securityManager: SecurityManager | undefined;
    let options: SecureTaskQueueOptions | undefined;

    if (securityOrOptions instanceof SecurityManager) {
      securityManager = securityOrOptions;
      options = maybeOptions;
    } else if (securityOrOptions) {
      options = securityOrOptions;
    }

    if (!securityManager) {
      throw new Error(
        "SecurityManager instance required to enqueue with credentials"
      );
    }

    const secureQueue = new SecureTaskQueue(this, securityManager, options);
    await secureQueue.enqueue(task, credentials);
  }

  /**
   * Initialize the task queue
   */
  async initialize(): Promise<void> {
    this.emit("initialized");
  }

  /**
   * Shutdown the task queue
   */
  async shutdown(): Promise<void> {
    this.queue = [];
    this.processing.clear();
    this.timestamps.clear();
    this.emit("shutdown");
  }

  /**
   * Get task state by ID
   */
  getTaskState(taskId: string): { task: Task; status: string } | undefined {
    const queuedTask = this.queue.find((t) => t.id === taskId);
    if (queuedTask) {
      return { task: queuedTask, status: "queued" };
    }

    const processingTask = this.processing.get(taskId);
    if (processingTask) {
      return { task: processingTask, status: "processing" };
    }

    return undefined;
  }

  /** Get queue length (alias for size)
   */
  getQueueLength(): number {
    return this.queue.length;
  }
}

export type TaskQueueAuditAction =
  | "enqueue"
  | "reject"
  | "rate_limited"
  | "sanitized";

export interface TaskQueueAuditRecord {
  taskId: string;
  action: TaskQueueAuditAction;
  actor: string;
  timestamp: Date;
  metadata?: Record<string, any>;
  reason?: string;
}

export interface TaskQueueAuditSink {
  record(event: TaskQueueAuditRecord): Promise<void> | void;
}

export interface SecureTaskQueueOptions {
  auditSink?: TaskQueueAuditSink;
  allowedTaskTypes?: Record<string, string[]>;
  metadataLimitBytes?: number;
  descriptionLimit?: number;
  redactMetadataKeys?: string[];
}

/**
 * Security-aware wrapper around TaskQueue that enforces authentication,
 * authorization, rate limiting, and audit logging for queue operations.
 *
 * All mutations go through the provided SecurityManager to guarantee that
 * task submissions respect security policies and produce an auditable trail.
 */
export class SecureTaskQueue {
  private readonly logger = new Logger("SecureTaskQueue");
  private readonly middleware: SecurityMiddleware;
  private readonly auditSink?: TaskQueueAuditSink;
  private readonly allowedTaskTypes?: Record<string, string[]>;
  private readonly metadataLimitBytes: number;
  private readonly descriptionLimit: number;
  private readonly redactMetadataKeys: string[];

  constructor(
    private readonly queue: TaskQueue,
    private readonly securityManager: SecurityManager,
    options: SecureTaskQueueOptions = {}
  ) {
    this.middleware = new SecurityMiddleware(securityManager);
    this.auditSink = options.auditSink;
    const policies = this.securityManager.getPolicyConfig();

    this.allowedTaskTypes =
      options.allowedTaskTypes &&
      Object.keys(options.allowedTaskTypes).length > 0
        ? options.allowedTaskTypes
        : Object.keys(policies.allowedTaskTypes || {}).length > 0
        ? (policies.allowedTaskTypes as Record<string, string[]>)
        : undefined;

    this.metadataLimitBytes =
      options.metadataLimitBytes ?? policies.maxMetadataSize;
    this.descriptionLimit =
      options.descriptionLimit ?? policies.maxTaskDescriptionLength;
    this.redactMetadataKeys = options.redactMetadataKeys ?? [
      "token",
      "secret",
      "authorization",
      "apiKey",
    ];
  }

  /**
   * Access the underlying queue for read-only operations.
   */
  get innerQueue(): TaskQueue {
    return this.queue;
  }

  /**
   * Enqueue a task after authenticating and authorizing the request.
   */
  async enqueue(
    task: Task,
    credentials: AuthCredentials
  ): Promise<void> {
    const baseClone = this.cloneTask(task);

    try {
      await this.middleware.protect(
        credentials,
        Permission.SUBMIT_TASK,
        "submitTask",
        async (context) => {
          const submissionClone = this.cloneTask(baseClone);

          try {
            // Sanitize payload (throws on suspicious content)
            this.securityManager.sanitizeInput(
              context,
              "task_submission",
              submissionClone
            );
            this.enforcePolicies(context, submissionClone);

            const enrichedTask = this.enrichTask(submissionClone, context);
            this.queue.enqueue(enrichedTask);

            await this.recordAudit("enqueue", enrichedTask, context);
          } catch (error) {
            await this.recordAudit("reject", submissionClone, context, {
              error: error instanceof Error ? error.message : String(error),
            });
            throw error;
          }
        }
      );
    } catch (error) {
      if (error instanceof SecurityError && error.code === "RATE_LIMITED") {
        await this.recordAudit("rate_limited", baseClone, undefined, {
          error: error.message,
          actor: credentials.agentId,
        });
      }
      throw error;
    }
  }

  private enforcePolicies(
    context: SecurityContext,
    task: Task
  ): void {
    if (task.description.length > this.descriptionLimit) {
      throw new SecurityError(
        "Task description exceeds policy limit",
        "TASK_DESCRIPTION_TOO_LONG",
        {
          limit: this.descriptionLimit,
          actual: task.description.length,
        }
      );
    }

    const metadataBytes = Buffer.byteLength(
      JSON.stringify(task.metadata ?? {}),
      "utf8"
    );
    if (metadataBytes > this.metadataLimitBytes) {
      throw new SecurityError(
        "Task metadata exceeds allowed size",
        "TASK_METADATA_TOO_LARGE",
        {
          limit: this.metadataLimitBytes,
          actual: metadataBytes,
        }
      );
    }

    if (
      this.allowedTaskTypes &&
      !this.securityManager.isTrustedAgent(context.agentId)
    ) {
      const agentSpecific =
        this.allowedTaskTypes[context.agentId] ??
        this.allowedTaskTypes[context.tenantId];
      const wildcard = this.allowedTaskTypes["*"];
      const allowed = agentSpecific ?? wildcard;

      if (Array.isArray(allowed) && allowed.length > 0) {
        if (!allowed.includes(task.type)) {
          throw new SecurityError(
            `Task type ${task.type} not permitted for agent ${context.agentId}`,
            "TASK_TYPE_NOT_ALLOWED",
            { allowed }
          );
        }
      }
    }
  }

  private async recordAudit(
    action: TaskQueueAuditAction,
    task: Task,
    context?: SecurityContext,
    details: Record<string, any> = {}
  ): Promise<void> {
    const { actor: overrideActor, ...metadataDetails } = details;
    const actorId = context?.agentId ?? overrideActor ?? "unknown";

    const record: TaskQueueAuditRecord = {
      taskId: task.id,
      action,
      actor: actorId,
      timestamp: new Date(),
      metadata: {
        taskType: task.type,
        tenantId: context?.tenantId,
        securityLevel: context?.securityLevel,
        ...metadataDetails,
      },
    };

    this.logger.info("Task queue action", {
      action,
      taskId: task.id,
      actor: actorId,
    });

    try {
      await this.auditSink?.record(record);
    } catch (error) {
      this.logger.warn("Failed to write task queue audit record", {
        error,
        taskId: task.id,
      });
    }
  }

  private cloneTask(task: Task): Task {
    return {
      ...task,
      requiredCapabilities: { ...(task.requiredCapabilities ?? {}) },
      budget: { ...task.budget },
      metadata: { ...(task.metadata ?? {}) },
      payload: task.payload ? { ...task.payload } : undefined,
      createdAt: task.createdAt ? new Date(task.createdAt) : new Date(),
    };
  }

  private enrichTask(task: Task, context: SecurityContext): Task {
    const metadata = {
      ...(task.metadata ?? {}),
      security: {
        submittedBy: context.agentId,
        sessionId: context.sessionId,
        tenantId: context.tenantId,
        securityLevel: context.securityLevel,
        submittedAt: new Date().toISOString(),
      },
    };

    const redactedMetadata = this.redactSensitiveMetadata(metadata);

    return {
      ...task,
      attempts: task.attempts ?? 0,
      maxAttempts: task.maxAttempts ?? 3,
      metadata: redactedMetadata,
      createdAt: task.createdAt ?? new Date(),
    };
  }

  private redactSensitiveMetadata(metadata: Record<string, any>) {
    const clone: Record<string, any> = {};
    for (const [key, value] of Object.entries(metadata)) {
      if (this.redactMetadataKeys.includes(key.toLowerCase())) {
        clone[key] = "[REDACTED]";
      } else if (value && typeof value === "object" && !Array.isArray(value)) {
        clone[key] = this.redactSensitiveMetadata(value);
      } else {
        clone[key] = value;
      }
    }
    return clone;
  }
}
