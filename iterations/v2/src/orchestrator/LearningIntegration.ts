/**
 * Learning Integration Layer
 *
 * Connects Multi-Turn Learning Coordinator to the Arbiter Orchestrator,
 * enabling automated learning from task execution patterns, errors, and outcomes.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import type { Pool } from "pg";
import { LearningDatabaseClient } from "../database/LearningDatabaseClient.js";
import type {
  LearningResult,
  LearningTask,
} from "../learning/MultiTurnLearningCoordinator.js";
import { MultiTurnLearningCoordinator } from "../learning/MultiTurnLearningCoordinator.js";
import type { LearningSessionConfig } from "../types/learning-coordination.js";
import { LearningCoordinatorEvent } from "../types/learning-coordination.js";

/**
 * Task completion event from orchestrator
 */
export interface TaskCompletionEvent {
  taskId: string;
  agentId: string;
  success: boolean;
  duration: number;
  errorMessage?: string;
  qualityScore?: number;
  context: unknown;
}

/**
 * Learning trigger configuration
 */
export interface LearningTriggerConfig {
  enableAutoLearning: boolean;
  minErrorCount: number;
  minQualityThreshold: number;
  learningSessionConfig?: Partial<LearningSessionConfig>;
}

/**
 * Performance metrics for learning
 */
export interface PerformanceMetrics {
  taskId: string;
  agentId: string;
  executionTimeMs: number;
  successRate: number;
  errorCount: number;
  averageQualityScore: number;
  timestamp: Date;
}

/**
 * Learning Integration
 *
 * Bridges orchestrator events to learning coordinator,
 * triggering learning sessions based on task outcomes.
 */
export class LearningIntegration extends EventEmitter {
  private coordinator: MultiTurnLearningCoordinator;
  private triggerConfig: LearningTriggerConfig;
  private performanceHistory: Map<string, PerformanceMetrics[]>;
  private activeLearningTasks: Set<string>;

  constructor(dbPool: Pool, triggerConfig?: Partial<LearningTriggerConfig>) {
    super();

    const dbClient = new LearningDatabaseClient(dbPool);
    this.coordinator = new MultiTurnLearningCoordinator(dbClient);

    this.triggerConfig = {
      enableAutoLearning: true,
      minErrorCount: 2,
      minQualityThreshold: 0.7,
      ...triggerConfig,
    };

    this.performanceHistory = new Map();
    this.activeLearningTasks = new Set();

    this.setupCoordinatorEventHandlers();
  }

  /**
   * Initialize integration layer
   */
  async initialize(): Promise<void> {
    await this.coordinator.initialize();
    this.emit("learning-integration:initialized", { timestamp: new Date() });
  }

  /**
   * Handle task completion event from orchestrator
   *
   * @param event - Task completion event
   */
  async handleTaskCompletion(event: TaskCompletionEvent): Promise<void> {
    // Record performance metrics
    this.recordPerformanceMetrics(event);

    // Check if learning should be triggered
    if (this.shouldTriggerLearning(event)) {
      await this.triggerLearningSession(event);
    }
  }

  /**
   * Manually trigger learning session for a task
   *
   * @param taskId - Task ID
   * @param agentId - Agent ID
   * @param context - Task context
   * @param qualityEvaluator - Quality evaluation function
   * @param executor - Task execution function
   * @returns Learning result
   */
  async triggerLearningSession(
    event: TaskCompletionEvent
  ): Promise<LearningResult | null> {
    // Prevent duplicate learning sessions
    const key = `${event.taskId}_${event.agentId}`;
    if (this.activeLearningTasks.has(key)) {
      return null;
    }

    this.activeLearningTasks.add(key);

    try {
      // Create learning task from completion event
      const learningTask: LearningTask = {
        taskId: event.taskId,
        agentId: event.agentId,
        initialContext: event.context,
        qualityEvaluator: async (_result: unknown) => {
          // Simple quality evaluation based on context
          // In production, this should integrate with actual quality metrics
          return event.qualityScore || 0.5;
        },
        executor: async (context: unknown, iterationNumber: number) => {
          // In production, this should re-execute the task with modified context
          // For now, return the context with iteration marker
          if (typeof context === "object" && context !== null) {
            return {
              ...(context as Record<string, unknown>),
              iteration: iterationNumber,
              timestamp: new Date(),
            };
          }
          return {
            originalContext: context,
            iteration: iterationNumber,
            timestamp: new Date(),
          };
        },
      };

      const result = await this.coordinator.startSession(
        learningTask,
        this.triggerConfig.learningSessionConfig
      );

      this.emit("learning-integration:session-completed", {
        taskId: event.taskId,
        agentId: event.agentId,
        result,
      });

      return result;
    } finally {
      this.activeLearningTasks.delete(key);
    }
  }

  /**
   * Record performance metrics for a task
   *
   * @param event - Task completion event
   */
  private recordPerformanceMetrics(event: TaskCompletionEvent): void {
    const metrics: PerformanceMetrics = {
      taskId: event.taskId,
      agentId: event.agentId,
      executionTimeMs: event.duration,
      successRate: event.success ? 1 : 0,
      errorCount: event.success ? 0 : 1,
      averageQualityScore: event.qualityScore || 0,
      timestamp: new Date(),
    };

    const key = `${event.taskId}_${event.agentId}`;
    const history = this.performanceHistory.get(key) || [];
    history.push(metrics);

    // Keep only last 100 metrics per task-agent pair
    if (history.length > 100) {
      history.shift();
    }

    this.performanceHistory.set(key, history);
  }

  /**
   * Check if learning should be triggered for a task
   *
   * @param event - Task completion event
   * @returns Whether to trigger learning
   */
  private shouldTriggerLearning(event: TaskCompletionEvent): boolean {
    if (!this.triggerConfig.enableAutoLearning) {
      return false;
    }

    const key = `${event.taskId}_${event.agentId}`;
    const history = this.performanceHistory.get(key);

    if (!history || history.length < 3) {
      return false; // Need at least 3 data points
    }

    // Trigger on repeated errors
    const recentErrors = history
      .slice(-5)
      .filter((m) => m.errorCount > 0).length;

    if (recentErrors >= this.triggerConfig.minErrorCount) {
      return true;
    }

    // Trigger on low quality scores
    const avgQuality =
      history.slice(-5).reduce((sum, m) => sum + m.averageQualityScore, 0) /
      Math.min(5, history.length);

    if (avgQuality < this.triggerConfig.minQualityThreshold) {
      return true;
    }

    return false;
  }

  /**
   * Get performance metrics for a task-agent pair
   *
   * @param taskId - Task ID
   * @param agentId - Agent ID
   * @returns Performance metrics history
   */
  getPerformanceMetrics(taskId: string, agentId: string): PerformanceMetrics[] {
    const key = `${taskId}_${agentId}`;
    return this.performanceHistory.get(key) || [];
  }

  /**
   * Get aggregated performance statistics
   *
   * @param taskId - Task ID
   * @param agentId - Agent ID
   * @returns Aggregated statistics
   */
  getPerformanceStatistics(
    taskId: string,
    agentId: string
  ): {
    totalExecutions: number;
    averageExecutionTime: number;
    successRate: number;
    totalErrors: number;
    averageQualityScore: number;
  } {
    const metrics = this.getPerformanceMetrics(taskId, agentId);

    if (metrics.length === 0) {
      return {
        totalExecutions: 0,
        averageExecutionTime: 0,
        successRate: 0,
        totalErrors: 0,
        averageQualityScore: 0,
      };
    }

    const totalExecutions = metrics.length;
    const averageExecutionTime =
      metrics.reduce((sum, m) => sum + m.executionTimeMs, 0) / totalExecutions;
    const successCount = metrics.filter((m) => m.successRate === 1).length;
    const successRate = successCount / totalExecutions;
    const totalErrors = metrics.reduce((sum, m) => sum + m.errorCount, 0);
    const averageQualityScore =
      metrics.reduce((sum, m) => sum + m.averageQualityScore, 0) /
      totalExecutions;

    return {
      totalExecutions,
      averageExecutionTime,
      successRate,
      totalErrors,
      averageQualityScore,
    };
  }

  /**
   * Setup event handlers for coordinator events
   */
  private setupCoordinatorEventHandlers(): void {
    // Forward learning events to integration listeners
    const eventsToForward = [
      LearningCoordinatorEvent.SESSION_STARTED,
      LearningCoordinatorEvent.SESSION_COMPLETED,
      LearningCoordinatorEvent.SESSION_FAILED,
      LearningCoordinatorEvent.ITERATION_STARTED,
      LearningCoordinatorEvent.ITERATION_COMPLETED,
      LearningCoordinatorEvent.ERROR_DETECTED,
      LearningCoordinatorEvent.PATTERN_RECOGNIZED,
      LearningCoordinatorEvent.FEEDBACK_GENERATED,
      LearningCoordinatorEvent.PROMPT_MODIFIED,
      LearningCoordinatorEvent.QUALITY_THRESHOLD_MET,
    ];

    for (const eventType of eventsToForward) {
      this.coordinator.on(eventType, (payload) => {
        this.emit(eventType, payload);
      });
    }
  }

  /**
   * Check if learning session is active for task-agent pair
   *
   * @param taskId - Task ID
   * @param agentId - Agent ID
   * @returns Whether learning is active
   */
  isLearningActive(taskId: string, agentId: string): boolean {
    const key = `${taskId}_${agentId}`;
    return this.activeLearningTasks.has(key);
  }

  /**
   * Update learning trigger configuration
   *
   * @param config - New configuration
   */
  updateTriggerConfig(config: Partial<LearningTriggerConfig>): void {
    this.triggerConfig = {
      ...this.triggerConfig,
      ...config,
    };
  }

  /**
   * Clear performance history for task-agent pair
   *
   * @param taskId - Task ID
   * @param agentId - Agent ID
   */
  clearPerformanceHistory(taskId: string, agentId: string): void {
    const key = `${taskId}_${agentId}`;
    this.performanceHistory.delete(key);
  }

  /**
   * Get all active learning tasks
   *
   * @returns Set of active task-agent keys
   */
  getActiveLearningTasks(): string[] {
    return Array.from(this.activeLearningTasks);
  }
}
