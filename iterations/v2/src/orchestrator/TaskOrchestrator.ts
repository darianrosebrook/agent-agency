/**
 * @fileoverview Task Orchestrator - ARBITER-014
 *
 * Main task execution engine that manages task lifecycle, worker isolation,
 * pleading workflows, and integrates with the full orchestration pipeline.
 *
 * @author @darianrosebrook
 */

import * as crypto from "crypto";
import { EventEmitter } from "events";
import * as path from "path";
import { Worker } from "worker_threads";
import { PerformanceTracker } from "../rl/PerformanceTracker";
import {
  PleadingDecision,
  PleadingWorkflow,
  Task,
  TaskExecution,
  TaskMetrics,
  TaskOrchestratorCapabilities,
  TaskOrchestratorConfig,
  TaskOrchestratorEvents,
  TaskStatus,
  WorkerExecutionResult,
  WorkerPoolConfig,
} from "../types/task-runner";
import { TaskState } from "../types/task-state";
import { EventSeverity, events } from "./EventEmitter";
import { EventTypes } from "./OrchestratorEvents";
import { TaskQueue } from "./TaskQueue";
import { TaskRetryHandler } from "./TaskRetryHandler";
import { TaskRoutingManager } from "./TaskRoutingManager";
import { TaskStateMachine } from "./TaskStateMachine";

/**
 * Worker Pool Manager
 */
class WorkerPoolManager extends EventEmitter {
  private workers: Map<string, Worker> = new Map();
  private activeTasks: Map<string, string> = new Map(); // taskId -> workerId
  private workerMetrics: Map<string, TaskMetrics> = new Map();
  private config: WorkerPoolConfig;
  private artifactConfig: WorkerPoolConfig["artifactConfig"];

  constructor(config: WorkerPoolConfig) {
    super();
    this.config = config;
    this.artifactConfig = config.artifactConfig;
    this.initializePool();
  }

  private initializePool(): void {
    for (let i = 0; i < this.config.minPoolSize; i++) {
      this.createWorker();
    }
  }

  private createWorker(): string {
    const workerId = crypto.randomUUID();

    const worker = new Worker(path.join(__dirname, "task-worker.js"), {
      workerData: {
        workerId,
        capabilities: this.config.workerCapabilities,
        artifactConfig: this.artifactConfig,
      },
    });

    worker.on("message", (message) => {
      this.handleWorkerMessage(workerId, message);
    });

    worker.on("error", (error) => {
      this.handleWorkerError(workerId, error);
    });

    worker.on("exit", (code) => {
      this.handleWorkerExit(workerId, code);
    });

    this.workers.set(workerId, worker);
    this.workerMetrics.set(workerId, {
      taskId: "",
      workerId,
      startTime: 0,
      endTime: 0,
      cpuUsage: 0,
      memoryUsage: 0,
      executionTime: 0,
      status: "idle",
    });

    return workerId;
  }

  private handleWorkerMessage(workerId: string, message: any): void {
    switch (message.type) {
      case "task_completed":
        this.handleTaskCompletion(workerId, message.result);
        break;
      case "task_failed":
        this.handleTaskFailure(workerId, message.error);
        break;
      case "worker_ready":
        this.emit("worker_ready", workerId);
        break;
      case "worker_metrics":
        this.updateWorkerMetrics(workerId, message.metrics);
        break;
    }
  }

  private handleWorkerError(workerId: string, error: Error): void {
    console.error(`Worker ${workerId} error:`, error);

    // Find task that was running on this worker
    const taskId = this.activeTasks.get(workerId);
    if (taskId) {
      this.activeTasks.delete(workerId);
      this.emit("task_failed", taskId, error);
    }

    // Remove failed worker and create replacement
    this.workers.delete(workerId);
    this.workerMetrics.delete(workerId);
    this.createWorker();
  }

  private handleWorkerExit(workerId: string, code: number): void {
    console.log(`Worker ${workerId} exited with code ${code}`);

    // Clean up worker resources
    this.workers.delete(workerId);
    this.workerMetrics.delete(workerId);

    // If we still need workers, create a replacement
    if (this.workers.size < this.config.minPoolSize) {
      this.createWorker();
    }
  }

  private handleTaskCompletion(
    workerId: string,
    result: WorkerExecutionResult
  ): void {
    const taskId = this.activeTasks.get(workerId);
    if (taskId) {
      this.activeTasks.delete(workerId);
      this.updateWorkerMetrics(workerId, {
        status: "completed",
        endTime: Date.now(),
      });
      this.emit("task_completed", taskId, result);
    }
  }

  private handleTaskFailure(workerId: string, error: any): void {
    const taskId = this.activeTasks.get(workerId);
    if (taskId) {
      this.activeTasks.delete(workerId);
      this.updateWorkerMetrics(workerId, {
        status: "failed",
        endTime: Date.now(),
      });
      this.emit("task_failed", taskId, error);
    }
  }

  private updateWorkerMetrics(
    workerId: string,
    metrics: Partial<TaskMetrics>
  ): void {
    const existing = this.workerMetrics.get(workerId);
    if (existing) {
      this.workerMetrics.set(workerId, { ...existing, ...metrics });
    }
  }

  async executeTask(task: Task): Promise<void> {
    const availableWorker = this.findAvailableWorker();
    if (!availableWorker) {
      const workerCount = this.workers.size;
      const workerStatuses = Array.from(this.workerMetrics.values()).map(
        (m) => `${m.workerId}:${m.status}`
      );
      throw new Error(
        `No available workers. Total workers: ${workerCount}, Statuses: ${workerStatuses.join(
          ", "
        )}`
      );
    }

    this.activeTasks.set(task.id, availableWorker);

    const worker = this.workers.get(availableWorker);
    if (!worker) {
      throw new Error(`Worker ${availableWorker} not found`);
    }

    // Update metrics
    this.updateWorkerMetrics(availableWorker, {
      taskId: task.id,
      startTime: Date.now(),
      status: "running",
    });

    // Send task to worker
    worker.postMessage({
      type: "execute_task",
      task,
    });
  }

  private findAvailableWorker(): string | null {
    for (const [workerId, metrics] of this.workerMetrics) {
      if (metrics.status === "idle") {
        return workerId;
      }
    }
    return null;
  }

  getMetrics(): {
    activeWorkers: number;
    totalWorkers: number;
    activeTasks: number;
  } {
    return {
      activeWorkers: this.workers.size,
      totalWorkers: this.workers.size,
      activeTasks: this.activeTasks.size,
    };
  }

  async shutdown(): Promise<void> {
    const shutdownPromises = Array.from(this.workers.entries()).map(
      ([workerId, worker]) => {
        return new Promise<void>((resolve) => {
          const timeoutId = setTimeout(() => worker.terminate(), 5000); // Force terminate after 5s
          worker.once("exit", () => {
            clearTimeout(timeoutId);
            resolve();
          });
          worker.postMessage({ type: "shutdown" });
        });
      }
    );

    await Promise.all(shutdownPromises);
    this.workers.clear();
    this.workerMetrics.clear();
    this.activeTasks.clear();
  }
}

/**
 * Pleading Workflow Manager
 */
class PleadingWorkflowManager extends EventEmitter {
  private activeWorkflows: Map<string, PleadingWorkflow> = new Map();
  private workflowHistory: Map<string, PleadingDecision[]> = new Map();

  async initiatePleading(
    taskId: string,
    context: any
  ): Promise<PleadingWorkflow> {
    const workflow: PleadingWorkflow = {
      workflowId: crypto.randomUUID(),
      taskId,
      initiatedAt: new Date(),
      status: "active",
      context,
      decisions: [],
      requiredApprovals: 3, // Configurable
      currentApprovals: 0,
    };

    this.activeWorkflows.set(taskId, workflow);
    this.workflowHistory.set(taskId, []);

    this.emit("pleading_initiated", workflow);

    return workflow;
  }

  async submitDecision(
    taskId: string,
    approverId: string,
    decision: "approve" | "deny" | "escalate",
    reasoning: string
  ): Promise<PleadingDecision> {
    const workflow = this.activeWorkflows.get(taskId);
    if (!workflow) {
      throw new Error(`No active pleading workflow for task ${taskId}`);
    }

    const pleadingDecision: PleadingDecision = {
      decisionId: crypto.randomUUID(),
      taskId,
      approverId,
      decision,
      reasoning,
      timestamp: new Date(),
      workflowId: workflow.workflowId,
    };

    workflow.decisions.push(pleadingDecision);
    this.workflowHistory.get(taskId)!.push(pleadingDecision);

    if (decision === "approve") {
      workflow.currentApprovals++;
    }

    // Check if workflow is complete
    if (workflow.currentApprovals >= workflow.requiredApprovals) {
      workflow.status = "approved";
      workflow.completedAt = new Date();
      this.emit("pleading_approved", workflow);
    } else if (workflow.decisions.length >= 10) {
      // Max decisions
      workflow.status = "denied";
      workflow.completedAt = new Date();
      this.emit("pleading_denied", workflow);
    }

    return pleadingDecision;
  }

  getWorkflow(taskId: string): PleadingWorkflow | null {
    return this.activeWorkflows.get(taskId) || null;
  }

  getWorkflowHistory(taskId: string): PleadingDecision[] {
    return this.workflowHistory.get(taskId) || [];
  }
}

/**
 * Main Task Orchestrator
 */
export class TaskOrchestrator extends EventEmitter {
  private config: TaskOrchestratorConfig;
  private workerPool: WorkerPoolManager;
  private pleadingManager: PleadingWorkflowManager;
  private taskQueue: TaskQueue;
  private stateMachine: TaskStateMachine;
  private retryHandler: TaskRetryHandler;
  private routingManager: TaskRoutingManager;
  private performanceTracker: PerformanceTracker;
  private activeExecutions: Map<string, TaskExecution> = new Map();
  private metrics: Map<string, TaskMetrics> = new Map();

  constructor(
    config: TaskOrchestratorConfig,
    agentRegistry?: any // Optional dependency injection
  ) {
    super();
    this.config = config;

    // Initialize components
    this.workerPool = new WorkerPoolManager({
      minPoolSize: config.workerPool.minPoolSize,
      maxPoolSize: config.workerPool.maxPoolSize,
      workerCapabilities: config.workerPool.workerCapabilities,
      workerTimeout: config.workerPool.workerTimeout,
    });
    this.pleadingManager = new PleadingWorkflowManager();
    this.taskQueue = new TaskQueue();
    this.stateMachine = new TaskStateMachine();
    this.retryHandler = new TaskRetryHandler({
      maxRetries: config.retry.maxAttempts,
      initialBackoffMs: config.retry.initialDelay,
      maxBackoffMs: config.retry.maxDelay,
      backoffMultiplier: config.retry.backoffMultiplier,
      jitter: true,
    });

    // Use injected agent registry or create a mock for testing
    const registry = agentRegistry || {
      getAgent: async () => null,
      getAgentByCapability: async () => [],
      getAgentsByCapability: async () => [], // Add missing method
      updateAgentPerformance: async () => {},
    };

    this.routingManager = new TaskRoutingManager(registry, {
      enableBandit: true,
      minAgentsRequired: 1,
      maxAgentsToConsider: 10,
      defaultStrategy: "multi-armed-bandit",
      maxRoutingTimeMs: 100,
      loadBalancingWeight: 0.3,
      capabilityMatchWeight: 0.7,
      ...config.routing,
    });

    this.performanceTracker = new PerformanceTracker({
      enabled: config.performance.trackingEnabled,
      maxEventsInMemory: 10000,
    });

    this.setupEventHandlers();
  }

  private setupEventHandlers(): void {
    // Worker pool events
    this.workerPool.on("task_completed", (taskId, result) => {
      this.handleTaskCompletion(taskId, result);
    });

    this.workerPool.on("task_failed", (taskId, error) => {
      this.handleTaskFailure(taskId, error);
    });

    // Pleading workflow events
    this.pleadingManager.on("pleading_initiated", (workflow) => {
      this.emit(TaskOrchestratorEvents.PLEADING_INITIATED, workflow);
      events.emit({
        id: `event-${Date.now()}-${crypto.randomUUID()}`,
        type: EventTypes.TASK_PLEADING_INITIATED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "TaskOrchestrator",
        taskId: workflow.taskId,
        metadata: { workflowId: workflow.workflowId },
      });
    });

    this.pleadingManager.on("pleading_approved", (workflow) => {
      this.emit(TaskOrchestratorEvents.PLEADING_APPROVED, workflow);
      this.handlePleadingApproval(workflow);
    });

    this.pleadingManager.on("pleading_denied", (workflow) => {
      this.emit(TaskOrchestratorEvents.PLEADING_DENIED, workflow);
      this.handlePleadingDenial(workflow);
    });
  }

  /**
   * Submit a task for execution
   */
  async submitTask(task: any): Promise<string> {
    // Validate task
    this.validateTask(task);

    // Route task to appropriate agent
    const routingDecision = await this.routingManager.routeTask(task);
    task.assignedAgent = routingDecision.selectedAgent.id;

    // Add to queue
    this.taskQueue.enqueue(task);

    // Initialize task state first
    this.stateMachine.initializeTask(task.id);

    // Then transition to queued state
    await this.stateMachine.transition(
      task.id,
      TaskState.QUEUED,
      "Task submitted"
    );

    // Track performance
    const performanceRoutingDecision = {
      taskId: task.id,
      selectedAgent: routingDecision.selectedAgent.id,
      routingStrategy: routingDecision.strategy as any,
      confidence: routingDecision.confidence,
      alternativesConsidered: routingDecision.alternatives.map((alt: any) => ({
        agentId: alt.agent.id,
        score: routingDecision.confidence * 0.8,
        reason: alt.reason,
      })),
      rationale: routingDecision.reason,
      timestamp: new Date().toISOString(),
    };

    const executionId = this.performanceTracker.startTaskExecution(
      task.id,
      routingDecision.selectedAgent.id,
      performanceRoutingDecision,
      { taskType: task.type, priority: task.priority }
    );

    this.emit(TaskOrchestratorEvents.TASK_SUBMITTED, task);

    events.emit({
      id: `event-${Date.now()}-${crypto.randomUUID()}`,
      type: EventTypes.TASK_SUBMITTED,
      timestamp: new Date(),
      severity: EventSeverity.INFO,
      source: "TaskOrchestrator",
      taskId: task.id,
      metadata: { agentId: task.assignedAgent },
    });

    // Start processing if queue allows
    this.processQueue();

    return task.id;
  }

  /**
   * Process tasks from queue
   */
  private async processQueue(): Promise<void> {
    try {
      while (true) {
        const task = await this.taskQueue.dequeue();
        if (!task) break;

        await this.executeTask(task);
      }
    } catch (error) {
      console.error("Queue processing error:", error);
    }
  }

  /**
   * Execute a task
   */
  private async executeTask(task: any): Promise<void> {
    try {
      // First transition to assigned
      await this.stateMachine.transition(
        task.id,
        TaskState.ASSIGNED,
        "Task assigned to agent"
      );

      // Then transition to running
      await this.stateMachine.transition(
        task.id,
        TaskState.RUNNING,
        "Task execution started"
      );

      // Create execution record
      const execution: TaskExecution = {
        executionId: crypto.randomUUID(),
        taskId: task.id,
        agentId: task.assignedAgent!,
        startedAt: new Date(),
        status: "running",
        attempts: 1,
      };

      this.activeExecutions.set(task.id, execution);

      // Execute via worker pool
      await this.workerPool.executeTask(task);

      this.emit(TaskOrchestratorEvents.TASK_STARTED, execution);
    } catch (error) {
      console.error(`Task execution failed for ${task.id}:`, error);
      await this.handleTaskFailure(task.id, error as Error);
    }
  }

  /**
   * Handle task completion
   */
  private async handleTaskCompletion(
    taskId: string,
    result: WorkerExecutionResult
  ): Promise<void> {
    const execution = this.activeExecutions.get(taskId);
    if (!execution) return;

    execution.completedAt = new Date();
    execution.status = "completed";
    execution.result = result;

    // Handle artifact metadata if present
    if (result.artifacts) {
      execution.artifacts = result.artifacts;
    }

    // Transition to completed
    await this.stateMachine.transition(
      taskId,
      TaskState.COMPLETED,
      "Task completed successfully"
    );

    // Track performance
    // await this.performanceTracker.recordTaskCompletion(execution, result);

    // Update metrics
    this.updateTaskMetrics(taskId, {
      status: "completed",
      endTime: Date.now(),
      executionTime:
        execution.completedAt.getTime() - execution.startedAt.getTime(),
    });

    this.activeExecutions.delete(taskId);

    this.emit(TaskOrchestratorEvents.TASK_COMPLETED, execution);

    events.emit({
      id: `event-${Date.now()}-${crypto.randomUUID()}`,
      type: EventTypes.TASK_COMPLETED,
      timestamp: new Date(),
      severity: EventSeverity.INFO,
      source: "TaskOrchestrator",
      taskId,
      metadata: { executionId: execution.executionId },
    });
  }

  /**
   * Handle task failure
   */
  private async handleTaskFailure(taskId: string, error: Error): Promise<void> {
    const execution = this.activeExecutions.get(taskId);
    if (!execution) return;

    execution.status = "failed";
    execution.error = error.message;

    // Check if we should retry
    // const shouldRetry = await this.retryHandler.shouldRetry(execution, error);
    // if (shouldRetry) {
    //   execution.attempts++;
    //   await this.retryHandler.scheduleRetry(execution);
    //   this.emit(TaskOrchestratorEvents.TASK_RETRY_SCHEDULED, execution);
    //   return;
    // }

    // Transition to failed
    await this.stateMachine.transition(
      taskId,
      TaskState.FAILED,
      `Task failed: ${error.message}`
    );

    // Check if pleading is needed
    if (this.shouldInitiatePleading(execution, error)) {
      await this.initiatePleading(taskId, { error: error.message, execution });
    } else {
      // Final failure
      this.activeExecutions.delete(taskId);
      this.emit(TaskOrchestratorEvents.TASK_FAILED, execution);
    }

    // Track performance
    // await this.performanceTracker.recordTaskFailure(execution, error);
  }

  /**
   * Initiate pleading workflow
   */
  private async initiatePleading(taskId: string, context: any): Promise<void> {
    try {
      const workflow = await this.pleadingManager.initiatePleading(
        taskId,
        context
      );
      // Pleading events are handled by event listeners
    } catch (error) {
      console.error(`Failed to initiate pleading for ${taskId}:`, error);
    }
  }

  /**
   * Handle pleading approval
   */
  private async handlePleadingApproval(
    workflow: PleadingWorkflow
  ): Promise<void> {
    const execution = this.activeExecutions.get(workflow.taskId);
    if (execution) {
      // Retry the task
      execution.attempts++;
      // await this.retryHandler.scheduleRetry(execution);
      this.emit(TaskOrchestratorEvents.TASK_RETRY_SCHEDULED, execution);
    }
  }

  /**
   * Handle pleading denial
   */
  private async handlePleadingDenial(
    workflow: PleadingWorkflow
  ): Promise<void> {
    const execution = this.activeExecutions.get(workflow.taskId);
    if (execution) {
      // Final failure
      this.activeExecutions.delete(workflow.taskId);
      this.emit(TaskOrchestratorEvents.TASK_FAILED, execution);
    }
  }

  /**
   * Submit pleading decision
   */
  async submitPleadingDecision(
    taskId: string,
    approverId: string,
    decision: "approve" | "deny" | "escalate",
    reasoning: string
  ): Promise<void> {
    await this.pleadingManager.submitDecision(
      taskId,
      approverId,
      decision,
      reasoning
    );
  }

  /**
   * Get task status
   */
  async getTaskStatus(taskId: string): Promise<TaskStatus | null> {
    const state = this.stateMachine.getState(taskId);
    return state as unknown as TaskStatus;
  }

  /**
   * Get pleading workflow
   */
  getPleadingWorkflow(taskId: string): PleadingWorkflow | null {
    return this.pleadingManager.getWorkflow(taskId);
  }

  /**
   * Get orchestrator capabilities
   */
  getCapabilities(): TaskOrchestratorCapabilities {
    return {
      maxConcurrentTasks: this.config.workerPool.maxPoolSize,
      supportedTaskTypes: [
        "script",
        "api_call",
        "data_processing",
        "ai_inference",
      ],
      pleadingSupport: true,
      retrySupport: true,
      isolationLevel: "worker_thread",
      monitoringEnabled: true,
      metricsEnabled: true,
    };
  }

  /**
   * Get orchestrator metrics
   */
  getMetrics(): {
    activeTasks: number;
    queuedTasks: number;
    completedTasks: number;
    failedTasks: number;
    workerPool: any;
  } {
    return {
      activeTasks: this.activeExecutions.size,
      queuedTasks: this.taskQueue.size(),
      completedTasks: this.metrics.size, // Simplified
      failedTasks: Array.from(this.metrics.values()).filter(
        (m) => m.status === "failed"
      ).length,
      workerPool: this.workerPool.getMetrics(),
    };
  }

  private validateTask(task: any): void {
    if (!task.id || !task.type || !task.payload) {
      throw new Error("Invalid task: missing required fields");
    }

    if (
      !["script", "api_call", "data_processing", "ai_inference"].includes(
        task.type
      )
    ) {
      throw new Error(`Unsupported task type: ${task.type}`);
    }
  }

  private shouldInitiatePleading(
    execution: TaskExecution,
    error: Error
  ): boolean {
    // Simple logic: initiate pleading for high-priority tasks that failed
    return execution.attempts >= 2; // After at least 2 attempts
  }

  private updateTaskMetrics(
    taskId: string,
    updates: Partial<TaskMetrics>
  ): void {
    const existing = this.metrics.get(taskId);
    if (existing) {
      this.metrics.set(taskId, { ...existing, ...updates });
    } else {
      this.metrics.set(taskId, {
        taskId,
        workerId: "",
        startTime: 0,
        endTime: 0,
        cpuUsage: 0,
        memoryUsage: 0,
        executionTime: 0,
        status: "idle",
        ...updates,
      });
    }
  }

  /**
   * Shutdown orchestrator
   */
  async shutdown(): Promise<void> {
    await this.workerPool.shutdown();
    // TaskQueue doesn't have a shutdown method, just clear its state
    this.taskQueue = new TaskQueue();
    this.activeExecutions.clear();
    this.metrics.clear();
  }
}

// Export types for external use
export type {
  PleadingDecision,
  PleadingWorkflow,
  Task,
  TaskExecution,
  TaskMetrics,
  TaskOrchestratorCapabilities,
  TaskOrchestratorConfig,
  TaskOrchestratorEvents,
  TaskPriority,
  TaskResult,
  TaskStatus,
  WorkerExecutionResult,
  WorkerPoolConfig,
} from "../types/task-runner";
