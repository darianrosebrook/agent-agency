/**
 * @fileoverview
 * Bridge between ARBITER-004 Performance Tracker and Model Registry.
 * Enables bidirectional performance data flow for comprehensive tracking.
 *
 * @author @darianrosebrook
 */

import type {
  PerformanceEvent,
  TaskExecutionData,
} from "@/rl/PerformanceTracker";
import type { ComputeCostTracker } from "./ComputeCostTracker";
import type { LocalModelSelector } from "./LocalModelSelector";
import type { ModelRegistry } from "./ModelRegistry";

/**
 * Performance data for a model operation
 */
export interface ModelPerformanceData {
  /** Model ID */
  modelId: string;

  /** Task type */
  taskType: string;

  /** Quality score (0-1) */
  quality: number;

  /** Latency in milliseconds */
  latencyMs: number;

  /** Memory usage in MB */
  memoryMB: number;

  /** Success flag */
  success: boolean;

  /** Timestamp */
  timestamp: Date;

  /** Input token count */
  inputTokens?: number;

  /** Output token count */
  outputTokens?: number;
}

/**
 * Bridge between Performance Tracker and Model Registry
 *
 * This component synchronizes performance data between:
 * - ARBITER-004 Performance Tracker (global system metrics)
 * - Model Registry (model-specific performance tracking)
 *
 * Benefits:
 * - Unified performance view across systems
 * - Model selection informed by real-world performance
 * - RL training data includes model selection context
 * - Cost optimization based on comprehensive metrics
 */
export class PerformanceTrackerBridge {
  private registry: ModelRegistry;
  private selector: LocalModelSelector;
  private costTracker: ComputeCostTracker;

  constructor(
    registry: ModelRegistry,
    selector: LocalModelSelector,
    costTracker: ComputeCostTracker
  ) {
    this.registry = registry;
    this.selector = selector;
    this.costTracker = costTracker;
  }

  /**
   * Records model performance from Performance Tracker event
   *
   * Converts ARBITER-004 performance events into model registry updates
   *
   * @param event Performance event from ARBITER-004
   * @param modelId Model that executed the task
   */
  recordFromPerformanceEvent(event: PerformanceEvent, modelId: string): void {
    // Extract task type from event
    const taskType = this.extractTaskType(event);

    // Calculate quality score based on event outcome
    const quality = this.calculateQualityFromEvent(event);

    // Update model performance history
    this.selector.updatePerformanceHistory(modelId, taskType, {
      quality,
      latencyMs:
        event.metadata?.latencyMs || event.metadata?.responseTimeMs || 0,
      memoryMB: this.estimateMemoryUsage(event),
      success: event.success,
    });

    // Record compute cost if detailed metrics available
    if (event.metadata?.latencyMs) {
      this.costTracker.recordOperation({
        modelId,
        operationId: event.eventId,
        timestamp: event.timestamp,
        wallClockMs: event.metadata.latencyMs,
        cpuTimeMs: event.metadata.latencyMs * 0.8, // Estimate
        peakMemoryMB: this.estimateMemoryUsage(event),
        avgMemoryMB: this.estimateMemoryUsage(event) * 0.8,
        cpuUtilization: event.metadata.cpuUsage || 50,
        inputTokens: event.metadata.inputTokens || 0,
        outputTokens: event.metadata.outputTokens || 0,
        tokensPerSecond: this.calculateTokensPerSecond(event),
      });
    }
  }

  /**
   * Records task execution from Performance Tracker
   *
   * @param execution Task execution data from ARBITER-004
   * @param modelId Model that executed the task
   */
  recordFromTaskExecution(execution: TaskExecutionData, modelId: string): void {
    const taskType = execution.metadata?.taskType || "unknown";

    // Calculate quality from reward/outcome
    const quality = this.calculateQualityFromReward(execution.reward);

    // Update performance history
    this.selector.updatePerformanceHistory(modelId, taskType, {
      quality,
      latencyMs: execution.executionTimeMs,
      memoryMB: this.estimateMemoryFromExecution(execution),
      success: execution.success,
    });

    // Record compute cost
    this.costTracker.recordOperation({
      modelId,
      operationId: execution.executionId,
      timestamp: execution.startedAt,
      wallClockMs: execution.executionTimeMs,
      cpuTimeMs: execution.executionTimeMs * 0.8,
      peakMemoryMB: this.estimateMemoryFromExecution(execution),
      avgMemoryMB: this.estimateMemoryFromExecution(execution) * 0.8,
      cpuUtilization: 60, // Default estimate
      inputTokens: 100, // Would need to be extracted from execution
      outputTokens: 50, // Would need to be extracted from execution
      tokensPerSecond: 50 / (execution.executionTimeMs / 1000),
    });
  }

  /**
   * Records model performance data directly
   *
   * @param data Performance data
   */
  recordModelPerformance(data: ModelPerformanceData): void {
    // Update selector's performance history
    this.selector.updatePerformanceHistory(data.modelId, data.taskType, {
      quality: data.quality,
      latencyMs: data.latencyMs,
      memoryMB: data.memoryMB,
      success: data.success,
    });

    // Record compute cost
    this.costTracker.recordOperation({
      modelId: data.modelId,
      operationId: `perf-${Date.now()}`,
      timestamp: data.timestamp,
      wallClockMs: data.latencyMs,
      cpuTimeMs: data.latencyMs * 0.8,
      peakMemoryMB: data.memoryMB,
      avgMemoryMB: data.memoryMB * 0.8,
      cpuUtilization: 60,
      inputTokens: data.inputTokens || 100,
      outputTokens: data.outputTokens || 50,
      tokensPerSecond: (data.outputTokens || 50) / (data.latencyMs / 1000),
    });
  }

  /**
   * Exports model performance data to Performance Tracker format
   *
   * This allows Performance Tracker to incorporate model selection context
   * into its RL training data.
   *
   * @param modelId Model ID
   * @param taskType Task type
   * @returns Performance history in Performance Tracker format
   */
  exportToPerformanceTracker(
    modelId: string,
    taskType: string
  ): TaskExecutionData[] {
    const history = this.selector.getPerformanceHistory(modelId, taskType);

    if (!history) {
      return [];
    }

    // Convert to TaskExecutionData format
    const executionData: TaskExecutionData = {
      executionId: `model-${modelId}-${taskType}`,
      taskName: taskType,
      success: history.successRate > 0.5,
      executionTimeMs: history.avgLatencyMs,
      startedAt: history.lastUpdated,
      completedAt: history.lastUpdated,
      agentId: modelId,
      reward: history.avgQuality,
      metadata: {
        modelId,
        taskType,
        avgQuality: history.avgQuality,
        avgLatency: history.avgLatencyMs,
        successRate: history.successRate,
        samples: history.samples,
      },
    };

    return [executionData];
  }

  /**
   * Extracts task type from performance event
   *
   * @param event Performance event
   * @returns Task type
   */
  private extractTaskType(event: PerformanceEvent): string {
    // Try to extract from metadata
    if (event.metadata?.taskType) {
      return event.metadata.taskType;
    }

    // Infer from event type
    switch (event.eventType) {
      case "routing_decision":
        return "routing";
      case "task_execution":
        return "execution";
      case "thinking_budget":
        return "thinking";
      case "judgment":
        return "judgment";
      case "minimality_eval":
        return "minimality";
      default:
        return "unknown";
    }
  }

  /**
   * Calculates quality score from performance event
   *
   * @param event Performance event
   * @returns Quality score (0-1)
   */
  private calculateQualityFromEvent(event: PerformanceEvent): number {
    // If event has explicit quality/score
    if (event.metadata?.quality !== undefined) {
      return event.metadata.quality;
    }

    if (event.metadata?.score !== undefined) {
      return event.metadata.score;
    }

    // Calculate based on success and other metrics
    if (!event.success) {
      return 0.3; // Low quality for failures
    }

    // Use latency as proxy (faster = better, within reason)
    const latency = event.metadata?.latencyMs || event.metadata?.responseTimeMs;
    if (latency) {
      // Good quality for fast responses (< 1s)
      if (latency < 1000) {
        return 0.9;
      }
      // Decent quality for moderate responses (< 5s)
      if (latency < 5000) {
        return 0.75;
      }
      // Lower quality for slow responses
      return 0.6;
    }

    // Default to moderate quality
    return 0.7;
  }

  /**
   * Calculates quality from reward value
   *
   * @param reward Reward value
   * @returns Quality score (0-1)
   */
  private calculateQualityFromReward(reward: number): number {
    // Normalize reward to 0-1 range
    // Assumes reward is typically in range [-1, 1] or [0, 1]
    if (reward < 0) {
      return 0.5 + reward / 2; // Map [-1, 0] to [0, 0.5]
    }
    return reward; // Assume already in [0, 1]
  }

  /**
   * Estimates memory usage from performance event
   *
   * @param event Performance event
   * @returns Memory usage in MB
   */
  private estimateMemoryUsage(event: PerformanceEvent): number {
    if (event.metadata?.memoryMB) {
      return event.metadata.memoryMB;
    }

    // Estimate based on event type
    switch (event.eventType) {
      case "judgment":
        return 256; // LLM judgments typically use moderate memory
      case "task_execution":
        return 512; // Task executions may use more
      case "thinking_budget":
        return 128; // Thinking is lightweight
      default:
        return 200; // Default estimate
    }
  }

  /**
   * Estimates memory from task execution
   *
   * @param execution Task execution data
   * @returns Memory usage in MB
   */
  private estimateMemoryFromExecution(execution: TaskExecutionData): number {
    if (execution.metadata?.memoryMB) {
      return execution.metadata.memoryMB;
    }

    // Estimate based on execution time (longer tasks may use more memory)
    if (execution.executionTimeMs > 5000) {
      return 512;
    }
    if (execution.executionTimeMs > 1000) {
      return 256;
    }
    return 128;
  }

  /**
   * Calculates tokens per second from event
   *
   * @param event Performance event
   * @returns Tokens per second
   */
  private calculateTokensPerSecond(event: PerformanceEvent): number {
    const outputTokens = event.metadata?.outputTokens || 50;
    const latencyMs =
      event.metadata?.latencyMs || event.metadata?.responseTimeMs || 1000;
    return outputTokens / (latencyMs / 1000);
  }
}
