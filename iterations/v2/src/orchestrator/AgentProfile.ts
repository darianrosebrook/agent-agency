/**
 * Agent Profile Management
 *
 * @author @darianrosebrook
 * @module orchestrator/AgentProfile
 *
 * Helper class for managing agent profiles with immutable updates
 * and running average performance calculations.
 */

import type {
  AgentProfile,
  CurrentLoad,
  PerformanceHistory,
  PerformanceMetrics,
  Timestamp,
} from "../types/agent-registry";

/**
 * Helper class for agent profile operations.
 * Ensures immutability and correct running average calculations.
 */
export class AgentProfileHelper {
  /**
   * Update performance history with new metrics using running averages.
   *
   * @param history - Current performance history
   * @param metrics - New performance metrics from completed task
   * @returns Updated performance history with new running averages
   *
   * @remarks
   * Uses incremental averaging formula to avoid storing all historical data:
   * newAverage = oldAverage + (newValue - oldAverage) / (count + 1)
   */
  static updatePerformanceHistory(
    history: PerformanceHistory,
    metrics: PerformanceMetrics
  ): PerformanceHistory {
    const newCount = history.taskCount + 1;

    // Incremental average updates
    const successValue = metrics.success ? 1.0 : 0.0;
    const newSuccessRate =
      history.successRate + (successValue - history.successRate) / newCount;

    const newAverageQuality =
      history.averageQuality +
      (metrics.qualityScore - history.averageQuality) / newCount;

    const newAverageLatency =
      history.averageLatency +
      (metrics.latencyMs - history.averageLatency) / newCount;

    return {
      successRate: newSuccessRate,
      averageQuality: newAverageQuality,
      averageLatency: newAverageLatency,
      taskCount: newCount,
    };
  }

  /**
   * Create initial performance history for a new agent.
   * Uses optimistic initialization to encourage exploration.
   *
   * @returns Initial performance history with optimistic values
   */
  static createInitialPerformanceHistory(): PerformanceHistory {
    return {
      successRate: 0.8, // Optimistic initialization
      averageQuality: 0.7, // Moderate quality assumption
      averageLatency: 5000, // 5 second baseline
      taskCount: 0,
    };
  }

  /**
   * Update current load by incrementing active tasks.
   *
   * @param load - Current load state
   * @param maxConcurrentTasks - Maximum concurrent tasks allowed
   * @returns Updated load with incremented active tasks
   */
  static incrementActiveTask(
    load: CurrentLoad,
    maxConcurrentTasks: number
  ): CurrentLoad {
    const newActiveTasks = load.activeTasks + 1;
    const utilizationPercent = (newActiveTasks / maxConcurrentTasks) * 100;

    return {
      activeTasks: newActiveTasks,
      queuedTasks: load.queuedTasks,
      utilizationPercent: Math.min(100, utilizationPercent),
    };
  }

  /**
   * Update current load by decrementing active tasks.
   *
   * @param load - Current load state
   * @param maxConcurrentTasks - Maximum concurrent tasks allowed
   * @returns Updated load with decremented active tasks
   */
  static decrementActiveTask(
    load: CurrentLoad,
    maxConcurrentTasks: number
  ): CurrentLoad {
    const newActiveTasks = Math.max(0, load.activeTasks - 1);
    const utilizationPercent = (newActiveTasks / maxConcurrentTasks) * 100;

    return {
      activeTasks: newActiveTasks,
      queuedTasks: load.queuedTasks,
      utilizationPercent,
    };
  }

  /**
   * Update queued tasks count.
   *
   * @param load - Current load state
   * @param queuedTasks - New queued tasks count
   * @returns Updated load with new queue size
   */
  static updateQueuedTasks(
    load: CurrentLoad,
    queuedTasks: number
  ): CurrentLoad {
    return {
      ...load,
      queuedTasks: Math.max(0, queuedTasks),
    };
  }

  /**
   * Create initial current load for a new agent.
   *
   * @returns Initial load state with zero tasks
   */
  static createInitialLoad(): CurrentLoad {
    return {
      activeTasks: 0,
      queuedTasks: 0,
      utilizationPercent: 0,
    };
  }

  /**
   * Update agent's last active timestamp.
   *
   * @param profile - Current agent profile
   * @param timestamp - New timestamp
   * @returns Updated profile with new lastActiveAt
   */
  static updateLastActive(
    profile: AgentProfile,
    timestamp: Timestamp = new Date().toISOString()
  ): AgentProfile {
    return {
      ...profile,
      lastActiveAt: timestamp,
    };
  }

  /**
   * Check if an agent is considered stale (inactive for too long).
   *
   * @param profile - Agent profile to check
   * @param staleThresholdMs - Threshold in milliseconds
   * @param currentTime - Current timestamp (defaults to now)
   * @returns True if agent is stale
   */
  static isStale(
    profile: AgentProfile,
    staleThresholdMs: number,
    currentTime: Timestamp = new Date().toISOString()
  ): boolean {
    const lastActive = new Date(profile.lastActiveAt).getTime();
    const current = new Date(currentTime).getTime();
    return current - lastActive > staleThresholdMs;
  }

  /**
   * Calculate confidence interval for success rate based on task count.
   * Used for Upper Confidence Bound (UCB) calculations in routing.
   *
   * @param history - Performance history
   * @param totalTasks - Total tasks across all agents (for UCB calculation)
   * @returns Confidence interval bonus value
   */
  static calculateConfidenceInterval(
    history: PerformanceHistory,
    totalTasks: number
  ): number {
    if (history.taskCount === 0) {
      return 1.0; // Maximum exploration bonus for new agents
    }

    // UCB exploration bonus formula
    return Math.sqrt((2 * Math.log(totalTasks)) / history.taskCount);
  }

  /**
   * Validate agent profile data for required fields and constraints.
   *
   * @param profile - Profile to validate
   * @throws Error if validation fails
   */
  static validateProfile(profile: Partial<AgentProfile>): void {
    if (!profile.id || profile.id.trim() === "") {
      throw new Error("Agent ID is required");
    }

    if (!profile.name || profile.name.trim() === "") {
      throw new Error("Agent name is required");
    }

    if (!profile.modelFamily) {
      throw new Error("Model family is required");
    }

    if (
      !profile.capabilities ||
      !profile.capabilities.taskTypes ||
      profile.capabilities.taskTypes.length === 0
    ) {
      throw new Error("At least one task type capability is required");
    }

    // Validate performance history ranges
    if (profile.performanceHistory) {
      const { successRate, averageQuality } = profile.performanceHistory;

      if (successRate < 0 || successRate > 1) {
        throw new Error("Success rate must be between 0 and 1");
      }

      if (averageQuality < 0 || averageQuality > 1) {
        throw new Error("Average quality must be between 0 and 1");
      }
    }

    // Validate current load ranges
    if (profile.currentLoad) {
      const { activeTasks, queuedTasks, utilizationPercent } =
        profile.currentLoad;

      if (activeTasks < 0) {
        throw new Error("Active tasks cannot be negative");
      }

      if (queuedTasks < 0) {
        throw new Error("Queued tasks cannot be negative");
      }

      if (utilizationPercent < 0 || utilizationPercent > 100) {
        throw new Error("Utilization percent must be between 0 and 100");
      }
    }
  }

  /**
   * Create a deep clone of an agent profile for immutable updates.
   *
   * @param profile - Profile to clone
   * @returns Deep clone of the profile
   */
  static cloneProfile(profile: AgentProfile): AgentProfile {
    return {
      ...profile,
      capabilities: {
        ...profile.capabilities,
        taskTypes: [...profile.capabilities.taskTypes],
        languages: [...profile.capabilities.languages],
        specializations: [...profile.capabilities.specializations],
      },
      performanceHistory: { ...profile.performanceHistory },
      currentLoad: { ...profile.currentLoad },
    };
  }
}
