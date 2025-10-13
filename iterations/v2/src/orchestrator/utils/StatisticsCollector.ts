/**
 * @fileoverview Statistics Collector - Orchestrator Statistics Collection
 *
 * Collects and reports orchestrator statistics including queue metrics,
 * task counts, throughput, and latency. Extracted from TaskOrchestrator
 * to follow composition pattern.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { TaskState } from "../../types/task-state";
import { TaskQueue } from "../TaskQueue";
import { TaskStateMachine } from "../TaskStateMachine";

/**
 * Statistics Collector Configuration
 */
export interface StatisticsCollectorConfig {
  /** Interval for statistics collection (ms) */
  statsIntervalMs?: number;

  /** Enable automatic statistics emission */
  enableAutoEmit?: boolean;
}

/**
 * Orchestrator Statistics
 */
export interface OrchestratorStats {
  queuedTasks: number;
  processingTasks: number;
  completedTasks: number;
  failedTasks: number;
  cancelledTasks: number;
  throughput: number;
  avgLatency: number;
  timestamp: Date;
}

/**
 * Statistics Collector - Collects orchestrator metrics
 *
 * Provides periodic statistics collection and reporting for orchestrator
 * monitoring and observability.
 */
export class StatisticsCollector extends EventEmitter {
  private statsInterval?: NodeJS.Timeout;
  private startTime: number;
  private latencySum = 0;
  private latencyCount = 0;

  constructor(
    private stateMachine: TaskStateMachine,
    private taskQueue: TaskQueue,
    private config: StatisticsCollectorConfig = {}
  ) {
    super();
    this.startTime = Date.now();
  }

  /**
   * Start automatic statistics collection
   */
  start(): void {
    if (this.statsInterval) {
      return;
    }

    const intervalMs = this.config.statsIntervalMs || 30000;

    this.statsInterval = setInterval(() => {
      const stats = this.collectStats();

      if (this.config.enableAutoEmit) {
        this.emit("orchestrator:stats", stats);
      }
    }, intervalMs);
  }

  /**
   * Stop automatic statistics collection
   */
  stop(): void {
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
      this.statsInterval = undefined;
    }
  }

  /**
   * Collect current statistics
   */
  collectStats(): OrchestratorStats {
    const queueStats = this.taskQueue.getStats();
    const stateStats = this.stateMachine.getStats();

    return {
      queuedTasks: queueStats.queued,
      processingTasks: queueStats.processing,
      completedTasks: stateStats[TaskState.COMPLETED] || 0,
      failedTasks: stateStats[TaskState.FAILED] || 0,
      cancelledTasks: stateStats[TaskState.CANCELLED] || 0,
      throughput: this.calculateThroughput(stateStats),
      avgLatency: this.calculateAverageLatency(),
      timestamp: new Date(),
    };
  }

  /**
   * Record task latency for averaging
   */
  recordLatency(latencyMs: number): void {
    this.latencySum += latencyMs;
    this.latencyCount++;
  }

  /**
   * Get queue statistics
   */
  getQueueStats() {
    return this.taskQueue.getStats();
  }

  /**
   * Get state machine statistics
   */
  getStateStats() {
    return this.stateMachine.getStats();
  }

  /**
   * Reset statistics
   */
  reset(): void {
    this.startTime = Date.now();
    this.latencySum = 0;
    this.latencyCount = 0;
  }

  /**
   * Calculate throughput (tasks per minute)
   */
  private calculateThroughput(stateStats: Record<TaskState, number>): number {
    const completed = stateStats[TaskState.COMPLETED] || 0;
    const timeMinutes = (Date.now() - this.startTime) / 60000;
    return timeMinutes > 0 ? completed / timeMinutes : 0;
  }

  /**
   * Calculate average latency
   */
  private calculateAverageLatency(): number {
    if (this.latencyCount === 0) {
      return 0;
    }
    return this.latencySum / this.latencyCount;
  }
}
