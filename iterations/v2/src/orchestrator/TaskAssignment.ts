/**
 * @fileoverview Task Assignment implementation for Arbiter Orchestration (ARBITER-005)
 *
 * Manages the assignment of tasks to agents based on routing decisions.
 * Tracks assignment lifecycle, timeouts, and provides reassignment capabilities.
 *
 * @author @darianrosebrook
 */

import {
  RoutingDecision,
  Task,
  TaskAssignment,
  TaskExecution,
  TaskResult,
} from "../types/arbiter-orchestration";
import { DatabaseClientFactory, IDatabaseClient } from "./DatabaseClient";

/**
 * Assignment Configuration
 */
export interface AssignmentConfig {
  /** Maximum time to wait for agent acknowledgment */
  acknowledgmentTimeoutMs: number;

  /** Maximum assignment duration */
  maxAssignmentDurationMs: number;

  /** Enable automatic reassignment on failure */
  autoReassignmentEnabled: boolean;

  /** Maximum number of reassignment attempts */
  maxReassignmentAttempts: number;

  /** Progress check interval */
  progressCheckIntervalMs: number;

  /** Enable persistence */
  persistenceEnabled: boolean;

  /** Database client for persistence */
  databaseClient?: IDatabaseClient;
}

/**
 * Assignment Status Updates
 */
export interface AssignmentStatusUpdate {
  status?: string;
  acknowledgedAt?: Date;
  startedAt?: Date;
  completedAt?: Date;
  progress?: number;
  errorMessage?: string;
  errorCode?: string;
}

/**
 * Assignment Statistics
 */
export interface AssignmentStats {
  /** Total assignments created */
  totalCreated: number;

  /** Currently active assignments */
  activeCount: number;

  /** Successful completions */
  completedCount: number;

  /** Failed assignments */
  failedCount: number;

  /** Timeout assignments */
  timeoutCount: number;

  /** Reassigned tasks */
  reassignedCount: number;

  /** Average assignment duration */
  averageDurationMs: number;

  /** Assignment success rate (0-1) */
  successRate: number;
}

/**
 * Task Assignment Manager
 *
 * Handles the lifecycle of task assignments from creation to completion.
 * Provides monitoring, timeout handling, and reassignment capabilities.
 */
export class TaskAssignmentManager {
  private assignments: Map<string, TaskAssignment> = new Map();
  private executions: Map<string, TaskExecution> = new Map();
  private config: AssignmentConfig;
  private stats: AssignmentStats;
  private timeouts: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private progressChecks: Map<string, ReturnType<typeof setInterval>> =
    new Map();
  private dbClient?: IDatabaseClient;
  private initialized: boolean = false;

  constructor(config: Partial<AssignmentConfig> = {}) {
    this.config = {
      acknowledgmentTimeoutMs: 5000,
      maxAssignmentDurationMs: 300000, // 5 minutes
      autoReassignmentEnabled: true,
      maxReassignmentAttempts: 3,
      progressCheckIntervalMs: 30000, // 30 seconds
      persistenceEnabled: false,
      ...config,
    };

    // Initialize database client if persistence is enabled
    if (this.config.persistenceEnabled) {
      this.dbClient =
        this.config.databaseClient || DatabaseClientFactory.createMockClient();
    }

    this.stats = {
      totalCreated: 0,
      activeCount: 0,
      completedCount: 0,
      failedCount: 0,
      timeoutCount: 0,
      reassignedCount: 0,
      averageDurationMs: 0,
      successRate: 0,
    };
  }

  /**
   * Initialize the assignment manager (connect to database)
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      // Connect to database if persistence is enabled
      if (this.config.persistenceEnabled && this.dbClient) {
        await this.dbClient.connect();
      }

      this.initialized = true;
      console.log("TaskAssignmentManager initialized successfully");
    } catch (error) {
      console.error("Failed to initialize TaskAssignmentManager:", error);
      throw error;
    }
  }

  /**
   * Persist assignment to database
   */
  private async persistAssignment(assignment: TaskAssignment): Promise<void> {
    if (!this.dbClient) {
      return;
    }

    try {
      await this.dbClient.query(
        `
        INSERT INTO task_assignments (
          assignment_id, task_id, agent_id, agent_name, agent_model_family,
          assigned_at, deadline, assignment_timeout_ms, routing_confidence,
          routing_strategy, routing_reason, status, acknowledged_at,
          started_at, completed_at, progress, last_progress_update,
          error_message, error_code, assignment_metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        ON CONFLICT (assignment_id) DO UPDATE SET
          status = EXCLUDED.status,
          acknowledged_at = EXCLUDED.acknowledged_at,
          started_at = EXCLUDED.started_at,
          completed_at = EXCLUDED.completed_at,
          progress = EXCLUDED.progress,
          last_progress_update = EXCLUDED.last_progress_update,
          error_message = EXCLUDED.error_message,
          error_code = EXCLUDED.error_code,
          updated_at = NOW()
      `,
        [
          assignment.id,
          assignment.task.id,
          assignment.agent.id,
          assignment.agent.name,
          assignment.agent.modelFamily,
          assignment.assignedAt,
          assignment.deadline,
          300000, // assignmentTimeoutMs - default 5 minutes
          assignment.routingDecision.confidence,
          assignment.routingDecision.strategy,
          assignment.routingDecision.reason,
          "pending", // status - default
          null, // acknowledgedAt
          null, // startedAt
          null, // completedAt
          0, // progress
          null, // lastProgressUpdate
          null, // errorMessage
          null, // errorCode
          JSON.stringify({}),
        ]
      );
    } catch (error) {
      console.error(`Failed to persist assignment ${assignment.id}:`, error);
      // Don't throw - assignment should continue working even if persistence fails
    }
  }

  /**
   * Update assignment status in database
   */
  private async updateAssignmentStatusInDb(
    assignmentId: string,
    updates: AssignmentStatusUpdate
  ): Promise<void> {
    if (!this.config.persistenceEnabled || !this.dbClient) {
      return;
    }

    try {
      const setParts: string[] = [];
      const values: any[] = [];
      let paramIndex = 1;

      if (updates.status !== undefined) {
        setParts.push(`status = $${paramIndex++}`);
        values.push(updates.status);
      }
      if (updates.acknowledgedAt !== undefined) {
        setParts.push(`acknowledged_at = $${paramIndex++}`);
        values.push(updates.acknowledgedAt);
      }
      if (updates.startedAt !== undefined) {
        setParts.push(`started_at = $${paramIndex++}`);
        values.push(updates.startedAt);
      }
      if (updates.completedAt !== undefined) {
        setParts.push(`completed_at = $${paramIndex++}`);
        values.push(updates.completedAt);
      }
      if (updates.progress !== undefined) {
        setParts.push(
          `progress = $${paramIndex++}, last_progress_update = NOW()`
        );
        values.push(updates.progress);
      }
      if (updates.errorMessage !== undefined) {
        setParts.push(`error_message = $${paramIndex++}`);
        values.push(updates.errorMessage);
      }
      if (updates.errorCode !== undefined) {
        setParts.push(`error_code = $${paramIndex++}`);
        values.push(updates.errorCode);
      }

      if (setParts.length === 0) {
        return; // Nothing to update
      }

      values.push(assignmentId); // Add assignment_id at the end

      await this.dbClient.query(
        `
        UPDATE task_assignments
        SET ${setParts.join(", ")}, updated_at = NOW()
        WHERE assignment_id = $${paramIndex}
      `,
        values
      );
    } catch (error) {
      console.error(
        `Failed to update assignment status ${assignmentId}:`,
        error
      );
    }
  }

  /**
   * Create a new task assignment
   */
  async createAssignment(
    task: Task,
    routingDecision: RoutingDecision,
    onAssignmentTimeout?: (assignment: TaskAssignment) => void,
    onProgressTimeout?: (assignment: TaskAssignment) => void
  ): Promise<TaskAssignment> {
    const assignment: TaskAssignment = {
      id: `assignment-${task.id}-${Date.now()}`,
      task,
      agent: routingDecision.selectedAgent,
      routingDecision,
      assignedAt: new Date(),
      deadline: new Date(Date.now() + this.config.maxAssignmentDurationMs),
    };

    // Store assignment
    this.assignments.set(assignment.id, assignment);
    this.stats.totalCreated++;
    this.stats.activeCount++;

    // Set acknowledgment timeout
    const ackTimeout = setTimeout(() => {
      this.handleAcknowledgmentTimeout(assignment, onAssignmentTimeout);
    }, this.config.acknowledgmentTimeoutMs);

    this.timeouts.set(`${assignment.id}-ack`, ackTimeout);

    // Set progress check interval
    const progressCheck = setInterval(() => {
      this.checkProgressTimeout(assignment, onProgressTimeout);
    }, this.config.progressCheckIntervalMs);

    this.progressChecks.set(assignment.id, progressCheck);

    // Persist assignment to database if enabled
    if (this.config.persistenceEnabled) {
      // Need to extend assignment with additional properties for database
      const dbAssignment = assignment as any;
      dbAssignment.taskId = task.id;
      dbAssignment.agentId = routingDecision.selectedAgent.id;
      dbAssignment.agentName = routingDecision.selectedAgent.name;
      dbAssignment.agentModelFamily = routingDecision.selectedAgent.modelFamily;
      dbAssignment.assignmentTimeoutMs = this.config.maxAssignmentDurationMs;
      dbAssignment.status = "assigned";
      dbAssignment.progress = 0;
      dbAssignment.lastProgressUpdate = new Date();

      await this.persistAssignment(dbAssignment);
    }

    return assignment;
  }

  /**
   * Acknowledge assignment (agent confirmed receipt)
   */
  async acknowledgeAssignment(assignmentId: string): Promise<boolean> {
    const assignment = this.assignments.get(assignmentId);
    if (!assignment) {
      return false;
    }

    // Clear acknowledgment timeout
    const ackTimeoutKey = `${assignmentId}-ack`;
    const ackTimeout = this.timeouts.get(ackTimeoutKey);
    if (ackTimeout) {
      clearTimeout(ackTimeout);
      this.timeouts.delete(ackTimeoutKey);
    }

    // Create execution record
    const execution: TaskExecution = {
      id: `execution-${assignment.task.id}-${Date.now()}`,
      assignment,
      startedAt: new Date(),
      status: "running",
      progress: 0,
      metadata: {},
    };

    this.executions.set(assignmentId, execution);

    // Update status in database
    if (this.config.persistenceEnabled) {
      await this.updateAssignmentStatusInDb(assignmentId, {
        acknowledgedAt: new Date(),
        startedAt: new Date(),
      });
    }

    return true;
  }

  /**
   * Update execution progress
   */
  async updateProgress(
    assignmentId: string,
    progress: number,
    status: TaskExecution["status"] = "running",
    metadata?: Record<string, any>
  ): Promise<boolean> {
    const execution = this.executions.get(assignmentId);
    if (!execution) {
      return false;
    }

    execution.progress = Math.max(0, Math.min(1, progress));
    execution.status = status;

    if (metadata) {
      execution.metadata = { ...execution.metadata, ...metadata };
    }

    // Reset progress timeout on any update
    this.resetProgressTimeout(assignmentId);

    // Update progress in database
    if (this.config.persistenceEnabled) {
      await this.updateAssignmentStatusInDb(assignmentId, {
        progress: execution.progress,
      });
    }

    return true;
  }

  /**
   * Complete assignment with result
   */
  completeAssignment(
    assignmentId: string,
    result: TaskResult,
    onCompletion?: (assignment: TaskAssignment, result: TaskResult) => void
  ): boolean {
    const assignment = this.assignments.get(assignmentId);
    const execution = this.executions.get(assignmentId);

    if (!assignment || !execution) {
      return false;
    }

    // Update execution
    execution.status = "completed";
    execution.progress = 1;

    // Calculate duration
    const duration = Date.now() - assignment.assignedAt.getTime();

    // Update statistics
    this.stats.activeCount--;
    this.stats.completedCount++;
    this.updateAverageDuration(duration);
    this.updateSuccessRate();

    // Clean up timers
    this.cleanupAssignmentTimers(assignmentId);

    // Remove from active tracking
    this.assignments.delete(assignmentId);
    this.executions.delete(assignmentId);

    // Notify completion
    if (onCompletion) {
      onCompletion(assignment, result);
    }

    return true;
  }

  /**
   * Fail assignment
   */
  failAssignment(
    assignmentId: string,
    error: string,
    canRetry: boolean = true,
    onFailure?: (assignment: TaskAssignment, error: string) => void
  ): boolean {
    const assignment = this.assignments.get(assignmentId);
    const execution = this.executions.get(assignmentId);

    if (!assignment) {
      return false;
    }

    // Update execution if exists
    if (execution) {
      execution.status = "failed";
    }

    // Update statistics
    this.stats.activeCount--;
    this.stats.failedCount++;

    // Clean up timers
    this.cleanupAssignmentTimers(assignmentId);

    // Handle reassignment if enabled and possible
    let reassigned = false;
    if (
      canRetry &&
      this.config.autoReassignmentEnabled &&
      assignment.task.attempts < assignment.task.maxAttempts
    ) {
      reassigned = this.attemptReassignment();
    }

    // Remove from active tracking if not reassigned
    if (!reassigned) {
      this.assignments.delete(assignmentId);
      this.executions.delete(assignmentId);
    }

    // Notify failure
    if (onFailure) {
      onFailure(assignment, error);
    }

    return true;
  }

  /**
   * Get assignment by ID
   */
  getAssignment(assignmentId: string): TaskAssignment | null {
    return this.assignments.get(assignmentId) || null;
  }

  /**
   * Get execution by assignment ID
   */
  getExecution(assignmentId: string): TaskExecution | null {
    return this.executions.get(assignmentId) || null;
  }

  /**
   * Get all active assignments
   */
  getActiveAssignments(): TaskAssignment[] {
    return Array.from(this.assignments.values());
  }

  /**
   * Get assignment statistics
   */
  getStats(): AssignmentStats {
    return { ...this.stats };
  }

  /**
   * Force timeout an assignment
   */
  timeoutAssignment(
    assignmentId: string,
    onTimeout?: (assignment: TaskAssignment) => void
  ): boolean {
    const assignment = this.assignments.get(assignmentId);
    if (!assignment) {
      return false;
    }

    // Update statistics
    this.stats.activeCount--;
    this.stats.timeoutCount++;

    // Clean up timers
    this.cleanupAssignmentTimers(assignmentId);

    // Remove from tracking
    this.assignments.delete(assignmentId);
    this.executions.delete(assignmentId);

    // Notify timeout
    if (onTimeout) {
      onTimeout(assignment);
    }

    return true;
  }

  /**
   * Clean shutdown - cancel all active assignments
   */
  async shutdown(): Promise<void> {
    // Clear all timers
    for (const timeout of Array.from(this.timeouts.values())) {
      clearTimeout(timeout);
    }
    this.timeouts.clear();

    for (const interval of Array.from(this.progressChecks.values())) {
      clearInterval(interval);
    }
    this.progressChecks.clear();

    // Cancel all active assignments
    const activeIds = Array.from(this.assignments.keys());
    for (const assignmentId of activeIds) {
      this.failAssignment(assignmentId, "System shutdown", false);
    }
  }

  /**
   * Handle acknowledgment timeout
   */
  private handleAcknowledgmentTimeout(
    assignment: TaskAssignment,
    onTimeout?: (assignment: TaskAssignment) => void
  ): void {
    // Agent didn't acknowledge within timeout
    this.failAssignment(
      assignment.id,
      "Acknowledgment timeout",
      true,
      (assignment) => {
        if (onTimeout) {
          onTimeout(assignment);
        }
      }
    );
  }

  /**
   * Check for progress timeout
   */
  private checkProgressTimeout(
    assignment: TaskAssignment,
    onTimeout?: (assignment: TaskAssignment) => void
  ): void {
    const execution = this.executions.get(assignment.id);
    if (!execution) {
      return;
    }

    const timeSinceLastUpdate = Date.now() - execution.startedAt.getTime();
    if (timeSinceLastUpdate > this.config.maxAssignmentDurationMs) {
      this.timeoutAssignment(assignment.id, onTimeout);
    }
  }

  /**
   * Reset progress timeout
   */
  private resetProgressTimeout(assignmentId: string): void {
    const progressCheck = this.progressChecks.get(assignmentId);
    if (progressCheck) {
      clearInterval(progressCheck);

      const newProgressCheck = setInterval(() => {
        const assignment = this.assignments.get(assignmentId);
        if (assignment) {
          this.checkProgressTimeout(assignment);
        }
      }, this.config.progressCheckIntervalMs);

      this.progressChecks.set(assignmentId, newProgressCheck);
    }
  }

  /**
   * Attempt to reassign a failed task
   */
  private attemptReassignment(): boolean {
    // This would typically call back to the routing system
    // For now, we'll just mark it as reassigned in statistics
    this.stats.reassignedCount++;
    return true; // Assume reassignment was successful
  }

  /**
   * Update average duration statistic
   */
  private updateAverageDuration(duration: number): void {
    const totalCompletions = this.stats.completedCount;
    if (totalCompletions === 1) {
      this.stats.averageDurationMs = duration;
    } else {
      const prevAverage = this.stats.averageDurationMs;
      this.stats.averageDurationMs =
        (prevAverage * (totalCompletions - 1) + duration) / totalCompletions;
    }
  }

  /**
   * Update success rate statistic
   */
  private updateSuccessRate(): void {
    const totalResolved =
      this.stats.completedCount +
      this.stats.failedCount +
      this.stats.timeoutCount;
    if (totalResolved > 0) {
      this.stats.successRate = this.stats.completedCount / totalResolved;
    }
  }

  /**
   * Clean up timers for an assignment
   */
  private cleanupAssignmentTimers(assignmentId: string): void {
    // Clear acknowledgment timeout
    const ackTimeoutKey = `${assignmentId}-ack`;
    const ackTimeout = this.timeouts.get(ackTimeoutKey);
    if (ackTimeout) {
      clearTimeout(ackTimeout);
      this.timeouts.delete(ackTimeoutKey);
    }

    // Clear progress check
    const progressCheck = this.progressChecks.get(assignmentId);
    if (progressCheck) {
      clearInterval(progressCheck);
      this.progressChecks.delete(assignmentId);
    }
  }
}

/**
 * Task Assignment Factory
 *
 * Provides utilities for creating and managing task assignments.
 */
export class TaskAssignmentFactory {
  private manager: TaskAssignmentManager;

  constructor(config?: Partial<AssignmentConfig>) {
    this.manager = new TaskAssignmentManager(config);
  }

  /**
   * Create assignment from routing decision
   */
  async createFromRouting(
    task: Task,
    routingDecision: RoutingDecision,
    callbacks?: {
      onAcknowledgmentTimeout?: (assignment: TaskAssignment) => void;
      onProgressTimeout?: (assignment: TaskAssignment) => void;
    }
  ): Promise<TaskAssignment> {
    return this.manager.createAssignment(
      task,
      routingDecision,
      callbacks?.onAcknowledgmentTimeout,
      callbacks?.onProgressTimeout
    );
  }

  /**
   * Get assignment manager instance
   */
  getManager(): TaskAssignmentManager {
    return this.manager;
  }
}
