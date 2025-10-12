/**
 * Task Queue Management
 *
 * Manages queued tasks and processing state for the orchestrator.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
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
    const queueIndex = this.queue.findIndex(t => t.id === taskId);
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
    return this.queue.some(t => t.id === taskId);
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
      .map(task => this.timestamps.get(task.id))
      .filter(Boolean) as Date[];

    const oldestQueued = queuedTimestamps.length > 0
      ? new Date(Math.min(...queuedTimestamps.map(t => t.getTime())))
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
    return this.queue.find(t => t.id === taskId) ||
           this.processing.get(taskId);
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
      ...this.queue.map(t => t.id),
      ...Array.from(this.processing.keys())
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
      if (timestamp && (now - timestamp.getTime()) > maxAgeMs) {
        stale.push(task);
      }
    }

    // Check processing tasks
    for (const task of Array.from(this.processing.values())) {
      const timestamp = this.timestamps.get(task.id);
      if (timestamp && (now - timestamp.getTime()) > maxAgeMs) {
        stale.push(task);
      }
    }

    return stale;
  }
}
