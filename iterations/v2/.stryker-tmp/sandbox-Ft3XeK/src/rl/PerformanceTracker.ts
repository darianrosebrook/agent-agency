/**
 * Performance Tracker for RL Training Data Collection
 *
 * @author @darianrosebrook
 * @module performance-tracker
 *
 * Collects and stores performance data for reinforcement learning training.
 * Implements data collection for routing decisions, task executions, and evaluation outcomes.
 */
// @ts-nocheck


import {
  PerformanceEvent,
  RoutingDecision,
  TaskOutcome,
  Timestamp,
} from "../types/agentic-rl";

/**
 * Configuration for the performance tracker.
 */
export interface PerformanceTrackerConfig {
  /**
   * Maximum number of events to keep in memory.
   */
  maxEventsInMemory: number;

  /**
   * Whether to enable data collection.
   */
  enabled: boolean;

  /**
   * Data retention period in milliseconds.
   */
  retentionPeriodMs: number;

  /**
   * Batch size for processing events.
   */
  batchSize: number;

  /**
   * Whether to anonymize collected data.
   */
  anonymizeData: boolean;
}

/**
 * Default configuration for the performance tracker.
 */
const DEFAULT_CONFIG: PerformanceTrackerConfig = {
  maxEventsInMemory: 10000,
  enabled: true,
  retentionPeriodMs: 30 * 24 * 60 * 60 * 1000, // 30 days
  batchSize: 100,
  anonymizeData: true,
};

/**
 * Performance data collected for a single task execution.
 */
export interface TaskExecutionData {
  /**
   * Execution tracking ID.
   */
  executionId: string;

  /**
   * Task identifier.
   */
  taskId: string;

  /**
   * Agent that executed the task.
   */
  agentId: string;

  /**
   * Routing decision that led to this agent selection.
   */
  routingDecision: RoutingDecision;

  /**
   * Task outcome and metrics.
   */
  outcome: TaskOutcome;

  /**
   * Timestamp of execution start.
   */
  startedAt: Timestamp;

  /**
   * Timestamp of execution completion.
   */
  completedAt: Timestamp;

  /**
   * Additional context data.
   */
  context?: Record<string, unknown>;
}

/**
 * Statistics about collected performance data.
 */
export interface PerformanceStats {
  /**
   * Total number of routing decisions recorded.
   */
  totalRoutingDecisions: number;

  /**
   * Total number of task executions recorded.
   */
  totalTaskExecutions: number;

  /**
   * Total number of evaluation outcomes recorded.
   */
  totalEvaluationOutcomes: number;

  /**
   * Average task completion time.
   */
  averageCompletionTimeMs: number;

  /**
   * Success rate across all tasks.
   */
  overallSuccessRate: number;

  /**
   * Data collection start time.
   */
  collectionStartedAt: Timestamp;

  /**
   * Last data collection time.
   */
  lastUpdatedAt: Timestamp;
}

/**
 * Performance Tracker for collecting RL training data.
 *
 * This component collects performance data from the arbiter system
 * to provide training data for reinforcement learning algorithms.
 * It stores routing decisions, task executions, and evaluation outcomes.
 */
export class PerformanceTracker {
  private config: PerformanceTrackerConfig;
  private events: PerformanceEvent[] = [];
  private taskExecutions: TaskExecutionData[] = [];
  private isCollecting: boolean = false;

  /**
   * Creates a new performance tracker instance.
   *
   * @param config - Configuration for the tracker. Uses defaults if not provided.
   */
  constructor(config: Partial<PerformanceTrackerConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Starts data collection.
   */
  startCollection(): void {
    if (this.config.enabled) {
      this.isCollecting = true;
    }
  }

  /**
   * Stops data collection.
   */
  stopCollection(): void {
    this.isCollecting = false;
  }

  /**
   * Records a routing decision made by the arbiter.
   *
   * @param decision - The routing decision to record.
   */
  async recordRoutingDecision(decision: RoutingDecision): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const event: PerformanceEvent = {
      type: "routing-decision",
      timestamp: decision.timestamp,
      data: this.anonymizeDataIfNeeded(decision) as unknown as Record<
        string,
        unknown
      >,
    };

    this.addEvent(event);
  }

  /**
   * Records the start of a task execution.
   *
   * @param taskId - Task identifier.
   * @param agentId - Agent identifier.
   * @param routingDecision - Routing decision that led to this execution.
   * @param context - Additional context data.
   * @returns Execution tracking ID.
   */
  startTaskExecution(
    taskId: string,
    agentId: string,
    routingDecision: RoutingDecision,
    context?: Record<string, unknown>
  ): string {
    if (!this.isCollecting || !this.config.enabled) {
      return "";
    }

    const executionId = `${taskId}-${Date.now()}`;
    const execution: TaskExecutionData = {
      executionId,
      taskId,
      agentId,
      routingDecision,
      outcome: {} as TaskOutcome, // Will be filled when completed
      startedAt: new Date().toISOString(),
      completedAt: "", // Will be filled when completed
      context: this.anonymizeDataIfNeeded(context || {}) as unknown as Record<
        string,
        unknown
      >,
    };

    this.taskExecutions.push(execution);
    this.cleanupOldExecutions();

    return executionId;
  }

  /**
   * Records the completion of a task execution.
   *
   * @param executionId - Execution tracking ID from startTaskExecution.
   * @param outcome - Task outcome and metrics.
   */
  async completeTaskExecution(
    executionId: string,
    outcome: TaskOutcome
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const execution = this.taskExecutions.find(
      (exec) => exec.executionId === executionId
    );

    if (execution) {
      execution.outcome = outcome;
      execution.completedAt = new Date().toISOString();

      const event: PerformanceEvent = {
        type: "task-execution",
        timestamp: execution.completedAt,
        data: this.anonymizeDataIfNeeded({
          taskId: execution.taskId,
          agentId: execution.agentId,
          routingDecision: execution.routingDecision,
          outcome,
          durationMs:
            new Date(execution.completedAt).getTime() -
            new Date(execution.startedAt).getTime(),
        }) as unknown as Record<string, unknown>,
      };

      this.addEvent(event);
    }
  }

  /**
   * Records an evaluation outcome.
   *
   * @param taskId - Task identifier.
   * @param evaluation - Evaluation results.
   */
  async recordEvaluationOutcome(
    taskId: string,
    evaluation: {
      passed: boolean;
      score: number;
      rubricScores?: Record<string, number>;
      feedback?: string;
    }
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const event: PerformanceEvent = {
      type: "evaluation-outcome",
      timestamp: new Date().toISOString(),
      data: this.anonymizeDataIfNeeded({
        taskId,
        evaluation,
      }) as Record<string, unknown>,
    };

    this.addEvent(event);
  }

  /**
   * Records a general performance event.
   *
   * @param event - Performance event to record.
   */
  async recordEvent(event: PerformanceEvent): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    this.addEvent(this.anonymizeDataIfNeeded(event) as PerformanceEvent);
  }

  /**
   * Exports collected data for RL training.
   *
   * @param since - Optional timestamp to export data since.
   * @returns Array of performance events ready for training.
   */
  exportTrainingData(since?: Timestamp): PerformanceEvent[] {
    let data = this.events;

    if (since) {
      const sinceTime = new Date(since).getTime();
      data = data.filter(
        (event) => new Date(event.timestamp).getTime() >= sinceTime
      );
    }

    // Return a copy to prevent external modification
    return JSON.parse(JSON.stringify(data));
  }

  /**
   * Gets performance statistics.
   *
   * @returns Current performance statistics.
   */
  getStats(): PerformanceStats {
    const routingDecisions = this.events.filter(
      (e) => e.type === "routing-decision"
    );
    const taskExecutions = this.events.filter(
      (e) => e.type === "task-execution"
    );
    const evaluationOutcomes = this.events.filter(
      (e) => e.type === "evaluation-outcome"
    );

    const completionTimes = taskExecutions
      .map((e) => (e.data as any).durationMs)
      .filter((time) => time !== undefined);

    const averageCompletionTime =
      completionTimes.length > 0
        ? completionTimes.reduce((sum, time) => sum + time, 0) /
          completionTimes.length
        : 0;

    const successfulTasks = taskExecutions.filter(
      (e) => (e.data as any).outcome?.success
    ).length;
    const successRate =
      taskExecutions.length > 0 ? successfulTasks / taskExecutions.length : 0;

    const timestamps = this.events.map((e) => new Date(e.timestamp).getTime());
    const lastUpdatedAt =
      timestamps.length > 0
        ? new Date(Math.max(...timestamps)).toISOString()
        : new Date().toISOString();

    return {
      totalRoutingDecisions: routingDecisions.length,
      totalTaskExecutions: taskExecutions.length,
      totalEvaluationOutcomes: evaluationOutcomes.length,
      averageCompletionTimeMs: averageCompletionTime,
      overallSuccessRate: successRate,
      collectionStartedAt:
        this.events.length > 0
          ? this.events[0].timestamp
          : new Date().toISOString(),
      lastUpdatedAt,
    };
  }

  /**
   * Clears all collected data.
   */
  clearData(): void {
    this.events = [];
    this.taskExecutions = [];
  }

  /**
   * Gets current configuration.
   *
   * @returns Current configuration.
   */
  getConfig(): PerformanceTrackerConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration.
   *
   * @param config - New configuration to apply.
   */
  updateConfig(config: Partial<PerformanceTrackerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Checks if data collection is currently active.
   *
   * @returns True if collecting data.
   */
  isActive(): boolean {
    return this.isCollecting && this.config.enabled;
  }

  /**
   * Adds an event to the collection, maintaining size limits.
   *
   * @param event - Event to add.
   */
  private addEvent(event: PerformanceEvent): void {
    this.events.push(event);
    this.cleanupOldEvents();
  }

  /**
   * Removes old events based on retention policy and size limits.
   */
  private cleanupOldEvents(): void {
    // Remove events older than retention period
    const cutoffTime = Date.now() - this.config.retentionPeriodMs;
    this.events = this.events.filter(
      (event) => new Date(event.timestamp).getTime() > cutoffTime
    );

    // Enforce maximum events in memory
    if (this.events.length > this.config.maxEventsInMemory) {
      // Keep most recent events
      this.events = this.events
        .sort(
          (a, b) =>
            new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
        )
        .slice(0, this.config.maxEventsInMemory);
    }
  }

  /**
   * Removes old task executions that haven't been completed.
   */
  private cleanupOldExecutions(): void {
    const oneHourAgo = Date.now() - 60 * 60 * 1000; // 1 hour ago
    this.taskExecutions = this.taskExecutions.filter(
      (execution) => new Date(execution.startedAt).getTime() > oneHourAgo
    );
  }

  /**
   * Anonymizes data if anonymization is enabled.
   *
   * @param data - Data to potentially anonymize.
   * @returns Anonymized data or original data.
   */
  private anonymizeDataIfNeeded<T>(data: T): T {
    if (!this.config.anonymizeData) {
      return data;
    }

    // Basic anonymization - remove or hash sensitive identifiers
    // In a full implementation, this would use proper anonymization techniques
    const anonymized = JSON.parse(JSON.stringify(data));

    // Anonymize agent IDs and task IDs with hashes
    this.anonymizeObject(anonymized);

    return anonymized as T;
  }

  /**
   * Recursively anonymizes object properties.
   *
   * @param obj - Object to anonymize.
   */
  private anonymizeObject(obj: any): void {
    if (typeof obj !== "object" || obj === null) {
      return;
    }

    for (const key in obj) {
      if (
        (key.toLowerCase().includes("id") ||
          key.toLowerCase().includes("agent") ||
          key.toLowerCase().includes("task")) &&
        typeof obj[key] === "string"
      ) {
        // Simple hash for IDs and agent/task identifiers
        obj[key] = this.simpleHash(obj[key]);
      } else if (typeof obj[key] === "object") {
        this.anonymizeObject(obj[key]);
      }
    }
  }

  /**
   * Simple hash function for anonymization.
   *
   * @param str - String to hash.
   * @returns Hashed string.
   */
  private simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }
}
