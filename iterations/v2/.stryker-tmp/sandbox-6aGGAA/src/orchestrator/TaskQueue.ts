/**
 * @fileoverview Task Queue implementation for Arbiter Orchestration (ARBITER-005)
 *
 * Manages the queue of tasks waiting for routing and assignment to agents.
 * Provides priority-based queuing, capacity management, and thread-safe operations.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  ITaskQueue,
  Task,
  TaskState,
  TaskStatus,
} from "../types/arbiter-orchestration";
import { DatabaseClientFactory, IDatabaseClient } from "./DatabaseClient";
import { events } from "./EventEmitter";
import { Mutex } from "./Mutex";
import {
  EventSeverity,
  EventTypes,
  TaskDequeuedEvent,
} from "./OrchestratorEvents";
import {
  AuthCredentials,
  Permission,
  SecurityManager,
} from "./SecurityManager";
import { ValidationUtils, validateTask } from "./Validation";

/**
 * Priority queue implementation with efficient insertion and removal
 */
class PriorityQueue<T> {
  private items: Array<{ item: T; priority: number }> = [];

  enqueue(item: T, priority: number): void {
    const queueItem = { item, priority };
    let added = false;

    // Insert in priority order (higher priority first)
    for (let i = 0; i < this.items.length; i++) {
      if (priority > this.items[i].priority) {
        this.items.splice(i, 0, queueItem);
        added = true;
        break;
      }
    }

    if (!added) {
      this.items.push(queueItem);
    }
  }

  dequeue(): T | null {
    return this.items.shift()?.item ?? null;
  }

  peek(): T | null {
    return this.items[0]?.item ?? null;
  }

  size(): number {
    return this.items.length;
  }

  clear(): void {
    this.items = [];
  }

  toArray(): T[] {
    return this.items.map((item) => item.item);
  }
}

/**
 * Task Queue Configuration
 */
export interface TaskQueueConfig {
  /** Maximum number of tasks that can be queued */
  maxCapacity: number;

  /** Default task timeout in milliseconds */
  defaultTimeoutMs: number;

  /** Maximum retry attempts for failed tasks */
  maxRetries: number;

  /** Queue processing priority mode */
  priorityMode: "fifo" | "priority" | "deadline";

  /** Enable queue persistence */
  persistenceEnabled: boolean;

  /** Database client for persistence */
  databaseClient?: IDatabaseClient;

  /** Security manager for authentication/authorization */
  securityManager?: SecurityManager;
}

/**
 * Task Queue Statistics
 */
export interface TaskQueueStats {
  /** Current queue depth */
  depth: number;

  /** Maximum queue depth reached */
  maxDepth: number;

  /** Total tasks enqueued */
  totalEnqueued: number;

  /** Total tasks dequeued */
  totalDequeued: number;

  /** Average wait time in milliseconds */
  averageWaitTimeMs: number;

  /** Tasks by priority distribution */
  priorityDistribution: Record<number, number>;

  /** Tasks by status distribution */
  statusDistribution: Record<TaskStatus, number>;
}

/**
 * Task Queue Implementation
 *
 * Thread-safe task queue with priority management and capacity controls.
 * Supports multiple priority modes and provides comprehensive statistics.
 */
export class TaskQueue implements ITaskQueue {
  private queue: PriorityQueue<Task>;
  private config: TaskQueueConfig;
  private stats: TaskQueueStats;
  private taskStates: Map<string, TaskState> = new Map();
  private mutex: Mutex = new Mutex();
  private dbClient?: IDatabaseClient;
  private securityManager?: SecurityManager;
  private initialized: boolean = false;

  constructor(config: Partial<TaskQueueConfig> = {}) {
    const finalConfig = {
      maxCapacity: 1000,
      defaultTimeoutMs: 30000,
      maxRetries: 3,
      priorityMode: "priority",
      persistenceEnabled: false,
      ...config,
    };

    // Validate configuration
    const configValidation =
      ValidationUtils.validateTaskQueueConfig(finalConfig);
    if (!configValidation.isValid) {
      throw new Error(
        `Invalid TaskQueue configuration:\n${ValidationUtils.formatValidationResult(
          configValidation
        )}`
      );
    }

    this.config = finalConfig as TaskQueueConfig;

    // Initialize database client if persistence is enabled
    if (this.config.persistenceEnabled) {
      this.dbClient =
        this.config.databaseClient || DatabaseClientFactory.createMockClient();
    }

    // Initialize security manager
    this.securityManager = this.config.securityManager;

    this.queue = new PriorityQueue<Task>();
    this.stats = {
      depth: 0,
      maxDepth: 0,
      totalEnqueued: 0,
      totalDequeued: 0,
      averageWaitTimeMs: 0,
      priorityDistribution: {},
      statusDistribution: {
        [TaskStatus.QUEUED]: 0,
        [TaskStatus.ROUTING]: 0,
        [TaskStatus.ASSIGNED]: 0,
        [TaskStatus.EXECUTING]: 0,
        [TaskStatus.VALIDATING]: 0,
        [TaskStatus.COMPLETED]: 0,
        [TaskStatus.FAILED]: 0,
        [TaskStatus.TIMEOUT]: 0,
        [TaskStatus.CANCELED]: 0,
      },
    };
  }

  /**
   * Initialize the task queue (connect to database, load persisted state)
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      // Connect to database if persistence is enabled
      if (this.config.persistenceEnabled && this.dbClient) {
        await this.dbClient.connect();

        // Load persisted tasks and restore queue state
        await this.loadPersistedState();
      }

      this.initialized = true;
      console.log("TaskQueue initialized successfully");
    } catch (error) {
      console.error("Failed to initialize TaskQueue:", error);
      throw error;
    }
  }

  /**
   * Shutdown the task queue (disconnect from database)
   */
  async shutdown(): Promise<void> {
    try {
      if (this.config.persistenceEnabled && this.dbClient) {
        await this.dbClient.disconnect();
      }
      this.initialized = false;
      console.log("TaskQueue shutdown successfully");
    } catch (error) {
      console.error("Error during TaskQueue shutdown:", error);
    }
  }

  /**
   * Load persisted state from database
   */
  private async loadPersistedState(): Promise<void> {
    if (!this.dbClient) {
      return;
    }

    try {
      // Load queued tasks from database
      const result = await this.dbClient.query(`
        SELECT * FROM task_queue
        WHERE status = 'queued'
        ORDER BY priority DESC, created_at ASC
      `);

      for (const row of result.rows) {
        const task: Task = {
          id: row.task_id,
          description: row.description,
          type: row.task_type,
          priority: row.priority,
          timeoutMs: row.timeout_ms,
          attempts: row.attempts,
          maxAttempts: row.max_attempts,
          requiredCapabilities: row.required_capabilities || {},
          budget: {
            maxFiles: row.budget_max_files,
            maxLoc: row.budget_max_loc,
          },
          createdAt: new Date(row.created_at),
          metadata: row.task_metadata || {},
        };

        // Restore task state
        const taskState: TaskState = {
          task,
          status: TaskStatus.QUEUED,
          attempts: row.attempts,
          maxAttempts: row.max_attempts,
          routingHistory: [],
        };

        this.queue.enqueue(task, this.calculatePriority(task));
        this.taskStates.set(task.id, taskState);

        // Update stats for loaded task
        this.stats.depth++;
        this.stats.maxDepth = Math.max(this.stats.maxDepth, this.stats.depth);
        this.stats.statusDistribution[TaskStatus.QUEUED]++;
      }

      console.log(`Loaded ${result.rows.length} persisted tasks from database`);
    } catch (error) {
      console.error("Failed to load persisted state:", error);
      throw error;
    }
  }

  /**
   * Persist a task to the database
   */
  private async persistTask(task: Task, taskState: TaskState): Promise<void> {
    if (!this.dbClient) {
      return;
    }

    try {
      await this.dbClient.query(
        `
        INSERT INTO task_queue (
          task_id, task_type, description, priority, timeout_ms,
          attempts, max_attempts, budget_max_files, budget_max_loc,
          required_capabilities, task_metadata, status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (task_id) DO UPDATE SET
          status = EXCLUDED.status,
          attempts = EXCLUDED.attempts,
          updated_at = NOW()
      `,
        [
          task.id,
          task.type,
          task.description,
          task.priority || 1,
          task.timeoutMs || this.config.defaultTimeoutMs,
          taskState.attempts,
          taskState.maxAttempts,
          task.budget?.maxFiles || 40,
          task.budget?.maxLoc || 1500,
          JSON.stringify(task.requiredCapabilities || {}),
          JSON.stringify(task.metadata || {}),
          "queued",
        ]
      );
    } catch (error) {
      console.error(`Failed to persist task ${task.id}:`, error);
      // Don't throw - queue should continue working even if persistence fails
    }
  }

  /**
   * Update task status in database
   */
  private async updateTaskStatusInDb(
    taskId: string,
    status: TaskStatus
  ): Promise<void> {
    if (!this.config.persistenceEnabled || !this.dbClient) {
      return;
    }

    try {
      await this.dbClient.query(
        `
        UPDATE task_queue
        SET status = $1, updated_at = NOW()
        WHERE task_id = $2
      `,
        [status, taskId]
      );
    } catch (error) {
      console.error(`Failed to update task status ${taskId}:`, error);
    }
  }

  /**
   * Enqueue a task for processing (standard interface method - no auth)
   */
  async enqueue(task: Task): Promise<void> {
    // For backward compatibility - no security checks
    // Validate input
    validateTask(task);

    await this.acquireLock();

    try {
      // Check capacity
      if (this.stats.depth >= this.config.maxCapacity) {
        throw new Error(`Queue capacity exceeded: ${this.config.maxCapacity}`);
      }

      // Set default timeout if not provided
      const timeoutMs = task.timeoutMs || this.config.defaultTimeoutMs;

      // Create task state
      const taskState: TaskState = {
        task: { ...task, timeoutMs },
        status: TaskStatus.QUEUED,
        attempts: 0,
        maxAttempts: this.config.maxRetries,
        routingHistory: [],
      };

      // Store task state
      this.taskStates.set(task.id, taskState);

      // Add to queue with appropriate priority
      const priority = this.calculatePriority(task);
      this.queue.enqueue(task, priority);

      // Persist to database if enabled
      if (this.config.persistenceEnabled && this.dbClient) {
        await this.persistTask(task, taskState);
      }

      // Update statistics
      this.stats.depth++;
      this.stats.maxDepth = Math.max(this.stats.maxDepth, this.stats.depth);
      this.stats.totalEnqueued++;
      this.stats.priorityDistribution[priority] =
        (this.stats.priorityDistribution[priority] || 0) + 1;
      this.stats.statusDistribution[TaskStatus.QUEUED]++;
    } finally {
      this.releaseLock();
    }
  }

  /**
   * Enqueue a task for processing (requires authentication)
   */
  async enqueueWithCredentials(
    task: Task,
    credentials: AuthCredentials
  ): Promise<void> {
    // Validate input
    validateTask(task);

    // Authenticate and authorize
    if (this.securityManager) {
      const context = this.securityManager.authenticate(credentials);
      if (!context) {
        throw new Error("Authentication failed");
      }

      if (!this.securityManager.authorize(context, Permission.SUBMIT_TASK)) {
        throw new Error("Authorization failed: insufficient permissions");
      }

      if (!this.securityManager.checkRateLimit(context, "submitTask")) {
        throw new Error("Rate limit exceeded for task submission");
      }

      // Sanitize input
      this.securityManager.sanitizeInput(context, "enqueue_task", task);
    }

    await this.acquireLock();

    try {
      // Check capacity
      if (this.stats.depth >= this.config.maxCapacity) {
        throw new Error(`Queue capacity exceeded: ${this.config.maxCapacity}`);
      }

      // Set default timeout if not provided
      const timeoutMs = task.timeoutMs || this.config.defaultTimeoutMs;

      // Create task state
      const taskState: TaskState = {
        task: { ...task, timeoutMs },
        status: TaskStatus.QUEUED,
        attempts: 0,
        maxAttempts: this.config.maxRetries,
        routingHistory: [],
      };

      // Store task state
      this.taskStates.set(task.id, taskState);

      // Add to queue with appropriate priority
      const priority = this.calculatePriority(task);
      this.queue.enqueue(task, priority);

      // Persist to database if enabled
      if (this.config.persistenceEnabled && this.dbClient) {
        await this.persistTask(task, taskState);
      }

      // Update statistics
      this.stats.depth++;
      this.stats.maxDepth = Math.max(this.stats.maxDepth, this.stats.depth);
      this.stats.totalEnqueued++;
      this.stats.priorityDistribution[priority] =
        (this.stats.priorityDistribution[priority] || 0) + 1;
      this.stats.statusDistribution[TaskStatus.QUEUED]++;
    } finally {
      this.releaseLock();
    }
  }

  /**
   * Dequeue the next task for processing
   */
  async dequeue(): Promise<Task | null> {
    await this.acquireLock();

    try {
      const task = this.queue.dequeue();

      if (task) {
        // Update task state
        const taskState = this.taskStates.get(task.id);
        if (taskState) {
          taskState.status = TaskStatus.ROUTING;
          taskState.routingHistory = [];
          this.updateStatusStats(TaskStatus.QUEUED, TaskStatus.ROUTING);

          // Update status in database
          await this.updateTaskStatusInDb(task.id, TaskStatus.ROUTING);
        }

        // Update statistics
        this.stats.depth--;
        this.stats.totalDequeued++;

        // Emit task dequeued event
        const dequeuedEvent: TaskDequeuedEvent = {
          id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          type: EventTypes.TASK_DEQUEUED,
          timestamp: new Date(),
          severity: EventSeverity.INFO,
          source: "TaskQueue",
          taskId: task.id,
          queueDepth: this.stats.depth,
          waitTimeMs: this.calculateWaitTime(task.id),
          metadata: {
            taskType: task.type,
            priority: task.priority,
          },
        };
        events.emit(dequeuedEvent);
      }

      return task || null;
    } finally {
      this.releaseLock();
    }
  }

  /**
   * Peek at the next task without removing it
   */
  async peek(): Promise<Task | null> {
    // Peek is read-only, no lock needed
    return this.queue.peek();
  }

  /**
   * Get current queue size
   */
  async size(): Promise<number> {
    return this.stats.depth;
  }

  /**
   * Clear all tasks from the queue
   */
  async clear(): Promise<void> {
    await this.acquireLock();

    try {
      // Cancel all queued tasks
      const queuedTasks = this.queue.toArray();
      for (const task of queuedTasks) {
        const taskState = this.taskStates.get(task.id);
        if (taskState) {
          taskState.status = TaskStatus.CANCELED;
          taskState.lastError = "Queue cleared";
          this.updateStatusStats(TaskStatus.QUEUED, TaskStatus.CANCELED);

          // Update status in database
          await this.updateTaskStatusInDb(task.id, TaskStatus.CANCELED);
        }
      }

      // Clear queue and reset statistics
      this.queue.clear();
      this.stats.depth = 0;
      this.taskStates.clear();
    } finally {
      this.releaseLock();
    }
  }

  /**
   * Get task state by ID
   */
  getTaskState(taskId: string): TaskState | null {
    return this.taskStates.get(taskId) || null;
  }

  /**
   * Update task status
   */
  updateTaskStatus(taskId: string, status: TaskStatus, error?: string): void {
    const taskState = this.taskStates.get(taskId);
    if (taskState) {
      const oldStatus = taskState.status;
      taskState.status = status;

      if (error) {
        taskState.lastError = error;
      }

      this.updateStatusStats(oldStatus, status);

      // Set timestamps
      if (status === TaskStatus.ASSIGNED) {
        taskState.startedAt = new Date();
      } else if (
        [
          TaskStatus.COMPLETED,
          TaskStatus.FAILED,
          TaskStatus.TIMEOUT,
          TaskStatus.CANCELED,
        ].includes(status)
      ) {
        taskState.completedAt = new Date();
      }
    }
  }

  /**
   * Record routing decision
   */
  recordRoutingDecision(taskId: string, decision: any): void {
    const taskState = this.taskStates.get(taskId);
    if (taskState) {
      taskState.routingHistory.push(decision);
    }
  }

  /**
   * Get queue statistics
   */
  getStats(): TaskQueueStats {
    return { ...this.stats };
  }

  /**
   * Get all queued tasks
   */
  getQueuedTasks(): Task[] {
    return this.queue.toArray();
  }

  /**
   * Calculate task priority based on configuration
   */
  private calculatePriority(task: Task): number {
    switch (this.config.priorityMode) {
      case "fifo":
        // Use creation timestamp for FIFO ordering
        return -task.createdAt.getTime();

      case "priority":
        // Use task priority directly
        return task.priority;

      case "deadline": {
        // Calculate urgency based on deadline proximity
        const timeToDeadline =
          task.createdAt.getTime() + task.timeoutMs - Date.now();
        const urgency = Math.max(0, 1 - timeToDeadline / (24 * 60 * 60 * 1000)); // 0-1 over 24 hours
        return task.priority + urgency * 10;
      }

      default:
        return task.priority;
    }
  }

  /**
   * Update status distribution statistics
   */
  private updateStatusStats(
    fromStatus: TaskStatus,
    toStatus: TaskStatus
  ): void {
    this.stats.statusDistribution[fromStatus]--;
    this.stats.statusDistribution[toStatus]++;
  }

  /**
   * Calculate estimated wait time for a task
   */
  private calculateEstimatedWaitTime(task: Task): number {
    // Simple estimation based on queue depth and priority
    const baseTime = 5000; // 5 seconds base
    const depthMultiplier = Math.min(this.stats.depth * 0.1, 2); // Max 2x multiplier
    const priorityMultiplier = task.priority <= 5 ? 1 : 0.5; // High priority waits less
    return Math.round(baseTime * depthMultiplier * priorityMultiplier);
  }

  /**
   * Calculate actual wait time for a dequeued task
   */
  private calculateWaitTime(taskId: string): number {
    const taskState = this.taskStates.get(taskId);
    if (!taskState) return 0;

    const enqueuedAt = (taskState as any).enqueuedAt || new Date();
    return Date.now() - enqueuedAt.getTime();
  }

  /**
   * Acquire exclusive lock for queue operations
   */
  private async acquireLock(): Promise<void> {
    await this.mutex.acquire();
  }

  /**
   * Release exclusive lock
   */
  private releaseLock(): void {
    this.mutex.release();
  }
}

/**
 * Secure Task Queue wrapper that adds authentication/authorization
 */
export class SecureTaskQueue implements ITaskQueue {
  constructor(
    private taskQueue: TaskQueue,
    private securityManager: SecurityManager
  ) {}

  async enqueue(task: Task, credentials?: AuthCredentials): Promise<void> {
    if (!credentials) {
      throw new Error("Authentication credentials required for secure queue");
    }
    return this.taskQueue.enqueueWithCredentials(task, credentials);
  }

  async dequeue(): Promise<Task | null> {
    return this.taskQueue.dequeue();
  }

  async peek(): Promise<Task | null> {
    return this.taskQueue.peek();
  }

  async size(): Promise<number> {
    return this.taskQueue.size();
  }

  async clear(): Promise<void> {
    return this.taskQueue.clear();
  }
}
