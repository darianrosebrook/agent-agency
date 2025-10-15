/**
 * Multi-Armed Bandit Implementation for Task Routing
 *
 * @author @darianrosebrook
 * @module multi-armed-bandit
 *
 * Implements epsilon-greedy and Upper Confidence Bound (UCB) algorithms
 * for intelligent task-to-agent routing in the Arbiter orchestrator.
 */
// @ts-nocheck


import {
  AgentProfile,
  AgentQueryResult,
  TaskType,
} from "../types/agent-registry";
import {
  BanditConfig,
  RLError,
  RLErrorType,
  RoutingDecision,
} from "../types/agentic-rl";

/**
 * Default configuration for multi-armed bandit routing.
 */
const DEFAULT_BANDIT_CONFIG: BanditConfig = {
  explorationRate: 0.2,
  decayFactor: 0.995,
  minSampleSize: 10,
  useUCB: true,
  ucbConstant: 2.0,
};

/**
 * Multi-armed bandit implementation for intelligent task routing.
 *
 * Uses epsilon-greedy strategy with optional Upper Confidence Bound (UCB)
 * scoring to balance exploration (trying new/unproven agents) vs exploitation
 * (using agents with proven performance).
 */
export class MultiArmedBandit {
  private config: BanditConfig;
  private totalTasks: number = 0;

  /**
   * Creates a new multi-armed bandit instance.
   *
   * @param config - Bandit configuration. Uses defaults if not provided.
   */
  constructor(config: Partial<BanditConfig> = {}) {
    this.config = { ...DEFAULT_BANDIT_CONFIG, ...config };
  }

  /**
   * Selects the best agent for a task using multi-armed bandit algorithm.
   *
   * @param candidates - Candidate agents that can handle the task
   * @param taskType - Type of task being routed
   * @returns Selected agent profile
   * @throws {RLError} If no candidates provided or selection fails
   */
  async select(
    candidates: AgentQueryResult[],
    taskType: TaskType
  ): Promise<AgentProfile> {
    if (candidates.length === 0) {
      throw new RLError(
        RLErrorType.INVALID_CONFIG,
        "No candidate agents provided for routing"
      );
    }

    // Increment total task counter for exploration decay
    this.totalTasks++;

    // Calculate decayed exploration rate
    const epsilon = this.calculateEpsilon();

    // Decide between exploration and exploitation
    if (Math.random() < epsilon) {
      return this.selectForExploration(candidates);
    } else {
      return this.selectForExploitation(candidates, taskType);
    }
  }

  /**
   * Creates a routing decision with full context for logging and analysis.
   *
   * @param taskId - Task identifier
   * @param candidates - All candidate agents considered
   * @param selectedAgent - The agent that was selected
   * @param taskType - Type of task being routed
   * @returns Complete routing decision with alternatives and rationale
   */
  createRoutingDecision(
    taskId: string,
    candidates: AgentQueryResult[],
    selectedAgent: AgentProfile,
    taskType: TaskType
  ): RoutingDecision {
    const alternativesConsidered = candidates.map((candidate) => ({
      agentId: candidate.agent.id,
      score: this.calculateUCB(candidate.agent, taskType),
      reason: this.explainScore(candidate.agent, taskType),
    }));

    const confidence = this.calculateConfidence(selectedAgent, taskType);

    return {
      taskId,
      selectedAgent: selectedAgent.id,
      routingStrategy: "multi-armed-bandit",
      confidence,
      alternativesConsidered,
      rationale: this.generateRationale(selectedAgent, taskType, confidence),
      timestamp: new Date().toISOString(),
    };
  }

  /**
   * Updates the bandit with the outcome of a routing decision.
   *
   * @param agentId - Agent that was selected
   * @param success - Whether the routing decision was successful
   * @param qualityScore - Quality score from task evaluation (0-1)
   * @param latencyMs - Task completion latency
   */
  updateWithOutcome(
    _agentId: string,
    _success: boolean,
    _qualityScore: number,
    _latencyMs: number
  ): void {
    // In a full implementation, this would update agent performance history
    // For now, we rely on the AgentRegistryManager to handle performance updates
    // This method provides a hook for future bandit-specific learning
  }

  /**
   * Calculates the current exploration rate with decay.
   *
   * @returns Exploration rate between 0 and 1
   */
  private calculateEpsilon(): number {
    const baseEpsilon = this.config.explorationRate;
    const decayedEpsilon =
      baseEpsilon * Math.pow(this.config.decayFactor, this.totalTasks);
    return Math.max(0.01, decayedEpsilon); // Minimum exploration rate
  }

  /**
   * Selects an agent for exploration (random or underutilized).
   *
   * @param candidates - Candidate agents
   * @returns Selected agent for exploration
   */
  private selectForExploration(candidates: AgentQueryResult[]): AgentProfile {
    // Prefer agents with low utilization for exploration
    const underutilized = candidates.filter(
      (c) => c.agent.currentLoad.utilizationPercent < 50
    );

    if (underutilized.length > 0) {
      // Random selection from underutilized agents
      const randomIndex = Math.floor(Math.random() * underutilized.length);
      return underutilized[randomIndex].agent;
    }

    // Fallback to random selection from all candidates
    const randomIndex = Math.floor(Math.random() * candidates.length);
    return candidates[randomIndex].agent;
  }

  /**
   * Selects the best agent for exploitation using UCB scoring.
   *
   * @param candidates - Candidate agents
   * @param taskType - Task type for scoring context
   * @returns Best performing agent
   */
  private selectForExploitation(
    candidates: AgentQueryResult[],
    taskType: TaskType
  ): AgentProfile {
    if (!this.config.useUCB) {
      // Simple success rate selection
      return candidates.sort(
        (a, b) =>
          b.agent.performanceHistory.successRate -
          a.agent.performanceHistory.successRate
      )[0].agent;
    }

    // Upper Confidence Bound selection
    const scoredCandidates = candidates.map((candidate) => ({
      agent: candidate.agent,
      score: this.calculateUCB(candidate.agent, taskType),
    }));

    // Return agent with highest UCB score
    return scoredCandidates.sort((a, b) => b.score - a.score)[0].agent;
  }

  /**
   * Calculates Upper Confidence Bound score for an agent.
   *
   * UCB formula: mean + exploration_bonus
   * exploration_bonus = sqrt(2 * ln(total_tasks) / attempts)
   *
   * @param agent - Agent profile
   * @param taskType - Task type for context
   * @returns UCB score
   */
  private calculateUCB(agent: AgentProfile, _taskType: TaskType): number {
    const history = agent.performanceHistory;
    const taskCount = history.taskCount;

    if (taskCount < this.config.minSampleSize) {
      // Not enough data, boost exploration
      return history.successRate + 1.0;
    }

    const mean = history.successRate;
    const explorationBonus =
      this.config.ucbConstant *
      Math.sqrt(Math.log(this.totalTasks + 1) / (taskCount + 1));

    return mean + explorationBonus;
  }

  /**
   * Calculates confidence in an agent selection.
   *
   * @param agent - Selected agent
   * @param taskType - Task type
   * @returns Confidence score (0-1)
   */
  private calculateConfidence(agent: AgentProfile, _taskType: TaskType): number {
    const history = agent.performanceHistory;
    const taskCount = history.taskCount;

    if (taskCount === 0) return 0.1; // Very low confidence for new agents

    // Confidence increases with task count and success rate
    const experienceFactor = Math.min(1.0, taskCount / 50); // Caps at 50 tasks
    const performanceFactor = history.successRate;

    // Penalize high utilization (agent might be overloaded)
    const loadFactor = Math.max(
      0.1,
      1.0 - agent.currentLoad.utilizationPercent / 100
    );

    return experienceFactor * 0.4 + performanceFactor * 0.4 + loadFactor * 0.2;
  }

  /**
   * Generates a human-readable explanation for an agent's score.
   *
   * @param agent - Agent profile
   * @param taskType - Task type
   * @returns Explanation string
   */
  private explainScore(agent: AgentProfile, taskType: TaskType): string {
    const history = agent.performanceHistory;
    const ucb = this.calculateUCB(agent, taskType);

    return (
      `${agent.name}: ${history.successRate.toFixed(2)} success rate, ` +
      `${history.taskCount} tasks, UCB score: ${ucb.toFixed(3)}, ` +
      `${agent.currentLoad.utilizationPercent}% utilized`
    );
  }

  /**
   * Generates rationale for a routing decision.
   *
   * @param agent - Selected agent
   * @param taskType - Task type
   * @param confidence - Confidence score
   * @returns Rationale string
   */
  private generateRationale(
    agent: AgentProfile,
    taskType: TaskType,
    confidence: number
  ): string {
    const epsilon = this.calculateEpsilon();
    const explorationUsed = Math.random() < epsilon;

    if (explorationUsed) {
      return (
        `Selected ${agent.name} for exploration (ε=${epsilon.toFixed(3)}) ` +
        `to gather more performance data for ${taskType} tasks`
      );
    } else {
      return (
        `Selected ${agent.name} for exploitation with ${(
          agent.performanceHistory.successRate * 100
        ).toFixed(1)}% ` +
        `success rate on ${
          agent.performanceHistory.taskCount
        } tasks (confidence: ${(confidence * 100).toFixed(1)}%)`
      );
    }
  }

  /**
   * Gets current bandit statistics for monitoring.
   *
   * @returns Bandit statistics
   */
  getStats(): {
    totalTasks: number;
    currentEpsilon: number;
    config: BanditConfig;
  } {
    return {
      totalTasks: this.totalTasks,
      currentEpsilon: this.calculateEpsilon(),
      config: this.config,
    };
  }

  /**
   * Resets the bandit state (useful for testing).
   */
  reset(): void {
    this.totalTasks = 0;
  }
}
