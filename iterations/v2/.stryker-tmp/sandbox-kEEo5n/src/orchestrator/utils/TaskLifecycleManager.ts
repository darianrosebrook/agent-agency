/**
 * @fileoverview Task Lifecycle Manager - Event-based Task Lifecycle Management
 *
 * Manages task lifecycle operations including submission, cancellation, suspension,
 * and resumption. Extracted from TaskOrchestrator to follow composition pattern.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { EventEmitter } from "events";
import { Task } from "../../types/arbiter-orchestration";
import { TaskState } from "../../types/task-state";
import { TaskQueue } from "../TaskQueue";
import { TaskStateMachine } from "../TaskStateMachine";

/**
 * Task Lifecycle Manager Configuration
 */
export interface TaskLifecycleConfig {
  /** Enable event emission */
  enableEvents?: boolean;
}

/**
 * Task Lifecycle Manager - Handles task lifecycle operations
 *
 * Provides event-based task lifecycle management with state transitions,
 * queue integration, and event emission.
 */
export class TaskLifecycleManager extends EventEmitter {
  constructor(
    private stateMachine: TaskStateMachine,
    private taskQueue: TaskQueue,
    private config: TaskLifecycleConfig = {}
  ) {
    super();
  }

  /**
   * Initialize a task in the state machine
   */
  initializeTask(taskId: string): void {
    this.stateMachine.initializeTask(taskId);
  }

  /**
   * Submit a task for processing
   */
  async submitTask(task: Task): Promise<void> {
    this.stateMachine.initializeTask(task.id);

    this.taskQueue.enqueue(task);

    if (this.config.enableEvents) {
      this.emit("task:submitted", {
        taskId: task.id,
        task,
        timestamp: new Date(),
      });
    }
  }

  /**
   * Cancel a task
   */
  async cancelTask(
    taskId: string,
    reason: string = "cancelled by user"
  ): Promise<void> {
    this.taskQueue.remove(taskId);

    if (!this.stateMachine.isTerminal(taskId)) {
      this.stateMachine.transition(taskId, TaskState.CANCELLED, reason);
    }

    if (this.config.enableEvents) {
      this.emit("task:cancelled", { taskId, reason, timestamp: new Date() });
    }
  }

  /**
   * Suspend a running task
   */
  async suspendTask(
    taskId: string,
    reason: string = "suspended"
  ): Promise<void> {
    if (this.stateMachine.getState(taskId) === TaskState.RUNNING) {
      this.stateMachine.transition(taskId, TaskState.SUSPENDED, reason);

      if (this.config.enableEvents) {
        this.emit("task:suspended", { taskId, reason, timestamp: new Date() });
      }
    }
  }

  /**
   * Resume a suspended task
   */
  async resumeTask(taskId: string, reason: string = "resumed"): Promise<void> {
    if (this.stateMachine.getState(taskId) === TaskState.SUSPENDED) {
      this.stateMachine.transition(taskId, TaskState.RUNNING, reason);

      if (this.config.enableEvents) {
        this.emit("task:resumed", { taskId, reason, timestamp: new Date() });
      }
    }
  }

  /**
   * Mark task as completed
   */
  async completeTask(taskId: string, result?: any): Promise<void> {
    this.taskQueue.complete(taskId);
    this.stateMachine.transition(
      taskId,
      TaskState.COMPLETED,
      "execution successful"
    );

    if (this.config.enableEvents) {
      this.emit("task:completed", {
        taskId,
        result,
        timestamp: new Date(),
      });
    }
  }

  /**
   * Mark task as failed
   */
  async failTask(taskId: string, error: any): Promise<void> {
    this.taskQueue.complete(taskId);

    const currentState = this.stateMachine.getState(taskId);

    if (currentState === TaskState.RUNNING) {
      this.stateMachine.transition(
        taskId,
        TaskState.FAILED,
        (error as Error).message
      );
    } else if (!this.stateMachine.isTerminal(taskId)) {
      this.stateMachine.transition(
        taskId,
        TaskState.FAILED,
        (error as Error).message
      );
    }

    if (this.config.enableEvents) {
      this.emit("task:failed", {
        taskId,
        error,
        timestamp: new Date(),
      });
    }
  }

  /**
   * Get task status
   */
  getTaskStatus(taskId: string): {
    state: TaskState;
    isTerminal: boolean;
    history: any[];
  } {
    return {
      state: this.stateMachine.getState(taskId),
      isTerminal: this.stateMachine.isTerminal(taskId),
      history: this.stateMachine.getTransitions(taskId),
    };
  }

  /**
   * Check if task is in terminal state
   */
  isTerminal(taskId: string): boolean {
    return this.stateMachine.isTerminal(taskId);
  }

  /**
   * Get current task state
   */
  getState(taskId: string): TaskState {
    return this.stateMachine.getState(taskId);
  }

  /**
   * Transition task to new state
   */
  transition(taskId: string, newState: TaskState, reason?: string): void {
    this.stateMachine.transition(taskId, newState, reason);

    if (this.config.enableEvents) {
      this.emit("state:transitioned", {
        taskId,
        newState,
        reason,
        timestamp: new Date(),
      });
    }
  }
}
