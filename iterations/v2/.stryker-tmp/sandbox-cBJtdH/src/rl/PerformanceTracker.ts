/**
 * Performance Tracker for RL Training Data Collection
 *
 * @author @darianrosebrook
 * @module performance-tracker
 *
 * Collects and stores performance data for reinforcement learning training.
 * Implements data collection for routing decisions, task executions, and evaluation outcomes.
 * Now integrates with comprehensive benchmarking system (ARBITER-004).
 */
// @ts-nocheck


import { DataCollector } from "../benchmarking/DataCollector";
import {
  PerformanceEvent,
  RoutingDecision,
  TaskOutcome,
  Timestamp,
} from "../types/agentic-rl";
import {
  AccuracyMetrics,
  ComplianceMetrics,
  CostMetrics,
  DataCollectionConfig,
  LatencyMetrics,
  ReliabilityMetrics,
  ResourceMetrics,
} from "../types/performance-tracking";

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
 *
 * Now integrates with ARBITER-004 benchmarking system for comprehensive
 * performance tracking, multi-dimensional scoring, and automated evaluation.
 */
export class PerformanceTracker {
  private config: PerformanceTrackerConfig;
  private events: PerformanceEvent[] = [];
  private taskExecutions: TaskExecutionData[] = [];
  private isCollecting: boolean = false;
  private dataCollector?: DataCollector;

  /**
   * Creates a new performance tracker instance.
   *
   * @param config - Configuration for the tracker. Uses defaults if not provided.
   * @param dataCollector - Optional external data collector (for testing).
   */
  constructor(
    config: Partial<PerformanceTrackerConfig> = {},
    dataCollector?: DataCollector
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.dataCollector = dataCollector; // Use provided collector if available
    this.initializeBenchmarkingIntegration();
  }

  /**
   * Initializes integration with ARBITER-004 benchmarking system.
   */
  private initializeBenchmarkingIntegration(): void {
    // Only create DataCollector if one wasn't provided externally (e.g., for testing)
    if (this.dataCollector) {
      return;
    }

    try {
      // Convert legacy config to new format
      const dataCollectorConfig: Partial<DataCollectionConfig> = {
        enabled: this.config.enabled,
        samplingRate: 1.0, // Legacy system collected all events
        maxBufferSize: this.config.maxEventsInMemory,
        batchSize: this.config.batchSize,
        retentionDays: Math.ceil(
          this.config.retentionPeriodMs / (24 * 60 * 60 * 1000)
        ),
        anonymization: {
          enabled: this.config.anonymizeData,
          level: "basic", // Legacy system used basic anonymization
          preserveAgentIds: true,
          preserveTaskTypes: true,
        },
      };

      this.dataCollector = new DataCollector(dataCollectorConfig);
    } catch (error) {
      // Graceful degradation - continue with legacy system if new system fails
      console.warn(
        "Failed to initialize benchmarking integration, using legacy system:",
        error
      );
    }
  }

  /**
   * Starts data collection.
   */
  startCollection(): void {
    if (this.config.enabled) {
      this.isCollecting = true;
      // Also start the benchmarking data collector
      if (this.dataCollector) {
        this.dataCollector.startCollection();
      }
    }
  }

  /**
   * Stops data collection.
   */
  stopCollection(): void {
    this.isCollecting = false;
    // Also stop the benchmarking data collector
    if (this.dataCollector) {
      this.dataCollector.stopCollection();
    }
  }

  /**
   * Records agent registration for performance baseline tracking.
   *
   * @param agentId - Agent identifier.
   * @param agentData - Agent registration data including capabilities and baseline metrics.
   */
  async recordAgentRegistration(
    agentId: string,
    agentData: {
      capabilities: string[];
      baselineMetrics: {
        latencyMs: number;
        accuracy: number;
        costPerTask: number;
        reliability: number;
      };
      registrationTimestamp: string;
    }
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    // Create agent registration event
    const event: PerformanceEvent = {
      type: "agent-registration",
      timestamp: agentData.registrationTimestamp,
      data: {
        agentId,
        capabilities: agentData.capabilities,
        baselineMetrics: agentData.baselineMetrics,
        eventType: "agent_registration",
      },
    };

    this.addEvent(event);

    // Forward to data collector if available
    if (this.dataCollector) {
      try {
        await this.dataCollector.recordAgentRegistration(agentId, {
          capabilities: agentData.capabilities,
          baselineMetrics: agentData.baselineMetrics,
          registrationTimestamp: agentData.registrationTimestamp,
        });
      } catch (error) {
        // Graceful degradation - log but don't fail
        console.warn(
          "Failed to record agent registration in benchmarking system:",
          error
        );
      }
    }
  }

  /**
   * Records agent status changes for availability tracking.
   *
   * @param agentId - Agent identifier.
   * @param status - New availability status.
   * @param context - Additional context about the status change.
   */
  async recordAgentStatusChange(
    agentId: string,
    status: "available" | "busy" | "offline" | "maintenance",
    context: { previousStatus?: string; reason?: string }
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    // Create agent status change event
    const event: PerformanceEvent = {
      type: "agent-status-change",
      timestamp: new Date().toISOString(),
      data: {
        agentId,
        status,
        previousStatus: context.previousStatus,
        reason: context.reason,
        eventType: "agent_status_change",
      },
    };

    this.addEvent(event);

    // Forward to data collector if available
    if (this.dataCollector) {
      try {
        await this.dataCollector.recordAgentStatusChange(
          agentId,
          status,
          context
        );
      } catch (error) {
        // Graceful degradation - log but don't fail
        console.warn(
          "Failed to record agent status change in benchmarking system:",
          error
        );
      }
    }
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

    // Legacy event format for backward compatibility
    const event: PerformanceEvent = {
      type: "routing-decision",
      timestamp: decision.timestamp,
      data: this.anonymizeDataIfNeeded(decision) as unknown as Record<
        string,
        unknown
      >,
    };

    this.addEvent(event);

    // Also send to new benchmarking system
    if (this.dataCollector) {
      try {
        const alternatives =
          decision.alternativesConsidered?.map((alt) => ({
            agentId: alt.agentId,
            score: alt.score,
          })) || [];

        await this.dataCollector.recordRoutingDecision(
          decision.taskId,
          decision.selectedAgent,
          alternatives,
          {
            confidence: decision.confidence,
            rationale: decision.rationale,
            alternativesConsidered: decision.alternativesConsidered,
          }
        );
      } catch (error) {
        // Graceful degradation - log but don't fail
        console.warn(
          "Failed to record routing decision in benchmarking system:",
          error
        );
      }
    }
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

    // Also record in new benchmarking system
    if (this.dataCollector) {
      try {
        this.dataCollector.recordTaskStart(taskId, agentId, context);
      } catch (error) {
        console.warn(
          "Failed to record task start in benchmarking system:",
          error
        );
      }
    }

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

      const durationMs =
        new Date(execution.completedAt).getTime() -
        new Date(execution.startedAt).getTime();

      // Legacy event format for backward compatibility
      const event: PerformanceEvent = {
        type: "task-execution",
        timestamp: execution.completedAt,
        data: this.anonymizeDataIfNeeded({
          taskId: execution.taskId,
          agentId: execution.agentId,
          routingDecision: execution.routingDecision,
          outcome,
          durationMs,
        }) as unknown as Record<string, unknown>,
      };

      this.addEvent(event);

      // Also send comprehensive metrics to new benchmarking system
      if (this.dataCollector) {
        try {
          // Convert legacy outcome to comprehensive performance metrics
          const performanceMetrics = this.convertOutcomeToPerformanceMetrics(
            outcome,
            durationMs
          );

          await this.dataCollector.recordTaskCompletion(
            execution.taskId,
            execution.agentId,
            performanceMetrics,
            {
              executionId,
              routingDecision: execution.routingDecision,
              startedAt: execution.startedAt,
              context: execution.context,
            }
          );
        } catch (error) {
          console.warn(
            "Failed to record task completion in benchmarking system:",
            error
          );
        }
      }
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
   * Records constitutional validation results for compliance tracking.
   *
   * @param validationData - CAWS validation result data
   */
  async recordConstitutionalValidation(validationData: {
    taskId: string;
    agentId: string;
    validationResult: {
      valid: boolean;
      violations: Array<{
        severity: "low" | "medium" | "high" | "critical";
        message: string;
        rule?: string;
      }>;
      complianceScore: number;
      processingTimeMs: number;
      ruleCount: number;
    };
  }): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    // Create constitutional validation event
    const event: PerformanceEvent = {
      type: "constitutional-validation",
      timestamp: new Date().toISOString(),
      data: this.anonymizeDataIfNeeded({
        taskId: validationData.taskId,
        agentId: validationData.agentId,
        validationResult: validationData.validationResult,
        eventType: "constitutional_validation",
      }) as Record<string, unknown>,
    };

    this.addEvent(event);

    // Forward to data collector if available
    if (this.dataCollector) {
      try {
        await this.dataCollector.recordConstitutionalValidation(validationData);
      } catch (error) {
        // Graceful degradation - log but don't fail
        console.warn(
          "Failed to record constitutional validation in benchmarking system:",
          error
        );
      }
    }
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
   * Records thinking budget allocation for RL training.
   *
   * @param taskId - Task identifier.
   * @param budget - Budget allocation data.
   */
  async recordThinkingBudget(
    taskId: string,
    budget: {
      allocatedTokens: number;
      complexityLevel: string;
      confidence: number;
    }
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const event: PerformanceEvent = {
      type: "thinking-budget-allocation",
      timestamp: new Date().toISOString(),
      data: this.anonymizeDataIfNeeded({
        taskId,
        allocatedTokens: budget.allocatedTokens,
        complexityLevel: budget.complexityLevel,
        confidence: budget.confidence,
        eventType: "thinking_budget",
      }) as Record<string, unknown>,
    };

    this.addEvent(event);
  }

  /**
   * Records thinking budget usage for completed task.
   *
   * @param taskId - Task identifier.
   * @param usage - Budget usage data.
   */
  async recordBudgetUsage(
    taskId: string,
    usage: {
      tokensUsed: number;
      tokensAllocated: number;
      utilizationRate: number;
    }
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const event: PerformanceEvent = {
      type: "thinking-budget-usage",
      timestamp: new Date().toISOString(),
      data: this.anonymizeDataIfNeeded({
        taskId,
        tokensUsed: usage.tokensUsed,
        tokensAllocated: usage.tokensAllocated,
        utilizationRate: usage.utilizationRate,
        eventType: "budget_usage",
      }) as Record<string, unknown>,
    };

    this.addEvent(event);
  }

  /**
   * Records minimality evaluation for code changes.
   *
   * @param taskId - Task identifier.
   * @param evaluation - Minimality evaluation data.
   */
  async recordMinimalityEvaluation(
    taskId: string,
    evaluation: {
      minimalityFactor: number;
      astSimilarity: number;
      scaffoldingPenalty: number;
      linesChanged: number;
      qualityAssessment: string;
    }
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const event: PerformanceEvent = {
      type: "minimality-evaluation",
      timestamp: new Date().toISOString(),
      data: this.anonymizeDataIfNeeded({
        taskId,
        minimalityFactor: evaluation.minimalityFactor,
        astSimilarity: evaluation.astSimilarity,
        scaffoldingPenalty: evaluation.scaffoldingPenalty,
        linesChanged: evaluation.linesChanged,
        qualityAssessment: evaluation.qualityAssessment,
        eventType: "minimality_eval",
      }) as Record<string, unknown>,
    };

    this.addEvent(event);
  }

  /**
   * Records LLM-based judgment for subjective evaluation.
   *
   * @param taskId - Task identifier.
   * @param judgment - Judgment data.
   */
  async recordJudgment(
    taskId: string,
    judgment: {
      overallScore: number;
      overallConfidence: number;
      allCriteriaPass: boolean;
      criteriaScores: Record<string, number>;
      evaluationTimeMs: number;
    }
  ): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const event: PerformanceEvent = {
      type: "model-judgment",
      timestamp: new Date().toISOString(),
      data: this.anonymizeDataIfNeeded({
        taskId,
        overallScore: judgment.overallScore,
        overallConfidence: judgment.overallConfidence,
        allCriteriaPass: judgment.allCriteriaPass,
        criteriaScores: judgment.criteriaScores,
        evaluationTimeMs: judgment.evaluationTimeMs,
        eventType: "model_judgment",
      }) as Record<string, unknown>,
    };

    this.addEvent(event);
  }

  /**
   * Records RL training metrics for monitoring.
   *
   * @param metrics - RL training metrics.
   */
  async recordRLTrainingMetrics(metrics: {
    trajectoriesProcessed: number;
    averageReward: number;
    policyLoss: number;
    valueLoss: number;
    klDivergence: number;
    trainingTimeMs: number;
  }): Promise<void> {
    if (!this.isCollecting || !this.config.enabled) {
      return;
    }

    const event: PerformanceEvent = {
      type: "rl-training-metrics",
      timestamp: new Date().toISOString(),
      data: {
        trajectoriesProcessed: metrics.trajectoriesProcessed,
        averageReward: metrics.averageReward,
        policyLoss: metrics.policyLoss,
        valueLoss: metrics.valueLoss,
        klDivergence: metrics.klDivergence,
        trainingTimeMs: metrics.trainingTimeMs,
        eventType: "rl_training",
      },
    };

    this.addEvent(event);
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

  /**
   * Converts legacy TaskOutcome to comprehensive performance metrics.
   *
   * @param outcome - Legacy task outcome
   * @param durationMs - Task execution duration
   * @returns Comprehensive performance metrics
   */
  private convertOutcomeToPerformanceMetrics(
    outcome: TaskOutcome,
    durationMs: number
  ): Partial<import("../types/performance-tracking").PerformanceMetrics> {
    const success = outcome.success !== false; // Default to true if not specified
    const qualityScore =
      typeof outcome.qualityScore === "number"
        ? outcome.qualityScore
        : success
        ? 0.8
        : 0.2;

    // Basic latency metrics
    const latencyMetrics: Partial<LatencyMetrics> = {
      averageMs: durationMs,
      minMs: durationMs,
      maxMs: durationMs,
      p95Ms: durationMs,
      p99Ms: durationMs,
    };

    // Accuracy metrics derived from outcome
    const accuracyMetrics: Partial<AccuracyMetrics> = {
      successRate: success ? 1 : 0,
      qualityScore,
      evaluationScore: qualityScore,
      violationRate: success ? 0 : 1,
    };

    // Basic resource metrics (placeholder - would be collected separately)
    const resourceMetrics: Partial<ResourceMetrics> = {
      cpuUtilizationPercent: 50, // Placeholder
      memoryUtilizationPercent: 60, // Placeholder
      networkIoKbps: 100, // Placeholder
      diskIoKbps: 50, // Placeholder
    };

    // Compliance metrics (basic - would be enhanced with CAWS validation)
    const complianceMetrics: Partial<ComplianceMetrics> = {
      validationPassRate: success ? 1 : 0,
      violationSeverityScore: success ? 0 : 0.5,
      clauseCitationRate: success ? 0.8 : 0.2,
    };

    // Cost metrics (simplified)
    const costMetrics: Partial<CostMetrics> = {
      costPerTask: durationMs / 1000, // Rough proxy: longer tasks cost more
      efficiencyScore: qualityScore,
      resourceWastePercent: success ? 10 : 30,
    };

    // Reliability metrics (basic)
    const reliabilityMetrics: Partial<ReliabilityMetrics> = {
      availabilityPercent: 100, // Single task - assume available
      errorRatePercent: success ? 0 : 100,
      mtbfHours: success ? 168 : 1, // Rough estimate
      recoveryTimeMinutes: success ? 0 : 5,
    };

    return {
      latency: latencyMetrics as LatencyMetrics,
      accuracy: accuracyMetrics as AccuracyMetrics,
      resources: resourceMetrics as ResourceMetrics,
      compliance: complianceMetrics as ComplianceMetrics,
      cost: costMetrics as CostMetrics,
      reliability: reliabilityMetrics as ReliabilityMetrics,
    };
  }
}
