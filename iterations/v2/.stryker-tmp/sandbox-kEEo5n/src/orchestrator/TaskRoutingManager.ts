/**
 * @fileoverview Task Routing Manager - Intelligent Agent Selection (ARBITER-002)
 *
 * Implements intelligent task-to-agent routing using multi-armed bandit algorithms,
 * capability matching, and load balancing to optimize task execution outcomes.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { MultiArmedBandit } from "../rl/MultiArmedBandit";
import { PerformanceTracker } from "../rl/PerformanceTracker";
import type { AgentRegistry } from "../types/agent-registry";
import {
  AgentQuery,
  AgentQueryResult,
  TaskType,
} from "../types/agent-registry";
import { RoutingStrategy } from "../types/agentic-rl";
import { RoutingDecision, Task } from "../types/arbiter-orchestration";

// Re-export commonly used types
export { VerificationPriority } from "../types/verification";

/**
 * Task Routing Manager Configuration
 */
export interface TaskRoutingConfig {
  /** Enable multi-armed bandit routing */
  enableBandit: boolean;

  /** Minimum agents required for routing */
  minAgentsRequired: number;

  /** Maximum agents to consider per routing */
  maxAgentsToConsider: number;

  /** Default routing strategy fallback */
  defaultStrategy: RoutingStrategy;

  /** Maximum routing decision time (ms) */
  maxRoutingTimeMs: number;

  /** Load balancing weight (0-1) */
  loadBalancingWeight: number;

  /** Capability matching weight (0-1) */
  capabilityMatchWeight: number;
}

/**
 * Routing metrics for monitoring and optimization
 */
export interface RoutingMetrics {
  totalRoutingDecisions: number;
  averageRoutingTimeMs: number;
  explorationRate: number;
  exploitationRate: number;
  capabilityMismatchRate: number;
  loadBalancingEffectiveness: number;
  successRate: number;
}

/**
 * Routing outcome for feedback loop
 */
export interface RoutingOutcome {
  routingDecision: RoutingDecision;
  success: boolean;
  qualityScore: number;
  latencyMs: number;
  errorReason?: string;
}

/**
 * Task Routing Manager
 *
 * Coordinates intelligent task-to-agent routing using multiple strategies:
 * 1. Multi-armed bandit for exploration/exploitation balance
 * 2. Capability matching for task requirements
 * 3. Load balancing for optimal resource utilization
 */
export class TaskRoutingManager {
  private config: TaskRoutingConfig;
  private agentRegistry: AgentRegistry;
  private performanceTracker?: PerformanceTracker;
  private multiArmedBandit: MultiArmedBandit | null = null;
  private metrics: RoutingMetrics;
  private routingHistory: Map<string, RoutingDecision> = new Map();

  constructor(
    agentRegistry: AgentRegistry,
    config?: Partial<TaskRoutingConfig>,
    performanceTracker?: PerformanceTracker
  ) {
    this.agentRegistry = agentRegistry;
    this.performanceTracker = performanceTracker;
    this.config = {
      enableBandit: true,
      minAgentsRequired: 1,
      maxAgentsToConsider: 10,
      defaultStrategy: "multi-armed-bandit",
      maxRoutingTimeMs: 100,
      loadBalancingWeight: 0.3,
      capabilityMatchWeight: 0.7,
      ...config,
    };

    // Initialize multi-armed bandit if enabled
    if (this.config.enableBandit) {
      this.multiArmedBandit = new MultiArmedBandit({
        explorationRate: 0.1,
        decayFactor: 0.995,
        minSampleSize: 5,
        useUCB: true,
        ucbConstant: 2.0,
      });
    }

    // Initialize metrics
    this.metrics = {
      totalRoutingDecisions: 0,
      averageRoutingTimeMs: 0,
      explorationRate: 0,
      exploitationRate: 0,
      capabilityMismatchRate: 0,
      loadBalancingEffectiveness: 0,
      successRate: 0,
    };
  }

  /**
   * Set the performance tracker for performance-aware routing.
   *
   * @param tracker - Performance tracker instance
   */
  setPerformanceTracker(tracker: PerformanceTracker): void {
    this.performanceTracker = tracker;
  }

  /**
   * Route a task to the optimal agent
   *
   * @param task - Task to route
   * @returns Routing decision with selected agent and rationale
   */
  async routeTask(task: Task): Promise<RoutingDecision> {
    const startTime = Date.now();

    try {
      // Step 1: Find candidate agents based on task requirements
      const candidateAgents = await this.findCandidateAgents(task);

      if (candidateAgents.length === 0) {
        throw new Error(
          `No agents available for task type: ${task.type}. ` +
            `Required capabilities not found in agent registry.`
        );
      }

      if (candidateAgents.length < this.config.minAgentsRequired) {
        throw new Error(
          `Insufficient agents (${candidateAgents.length}/${this.config.minAgentsRequired}) ` +
            `available for task type: ${task.type}`
        );
      }

      // Step 2: Get performance context for routing decision
      const performanceContext = this.performanceTracker
        ? await this.getPerformanceContext(task, candidateAgents)
        : null;

      // Step 3: Apply routing strategy with performance context
      const routingDecision = await this.applyRoutingStrategy(
        task,
        candidateAgents,
        performanceContext
      );

      // Step 4: Record routing decision
      this.recordRoutingDecision(routingDecision);

      // Step 5: Update metrics
      const routingTimeMs = Date.now() - startTime;
      this.updateMetrics(routingTimeMs, routingDecision);

      // Validate routing time is within SLA
      if (routingTimeMs > this.config.maxRoutingTimeMs) {
        console.warn(
          `Routing decision took ${routingTimeMs}ms, exceeding SLA of ${this.config.maxRoutingTimeMs}ms`
        );
      }

      return routingDecision;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      // Create error routing decision
      const errorDecision: RoutingDecision = {
        id: `routing-error-${Date.now()}`,
        taskId: task.id,
        selectedAgent: null as any, // No agent selected
        confidence: 0,
        reason: `Routing failed: ${errorMessage}`,
        strategy: this.config.defaultStrategy as any,
        alternatives: [],
        timestamp: new Date(),
      };

      this.metrics.totalRoutingDecisions++;
      throw error;
    }
  }

  /**
   * Find candidate agents that match task requirements
   */
  private async findCandidateAgents(task: Task): Promise<AgentQueryResult[]> {
    // Build agent query from task requirements
    const query: AgentQuery = {
      taskType: task.type as TaskType,
      languages: task.requiredCapabilities?.languages,
      specializations: task.requiredCapabilities?.specializations,
      maxUtilization: 90, // Don't overload agents
      minSuccessRate: 0.0, // Consider all agents (bandit will optimize)
    };

    // Query agent registry
    const candidates = await this.agentRegistry.getAgentsByCapability(query);

    // Limit to max agents to consider
    return candidates.slice(0, this.config.maxAgentsToConsider);
  }

  /**
   * Apply routing strategy to select optimal agent
   */
  private async applyRoutingStrategy(
    task: Task,
    candidates: AgentQueryResult[],
    performanceContext?: any
  ): Promise<RoutingDecision> {
    // Use multi-armed bandit strategy if enabled
    if (
      this.config.enableBandit &&
      this.multiArmedBandit &&
      this.config.defaultStrategy === "multi-armed-bandit"
    ) {
      return await this.routeWithBandit(task, candidates, performanceContext);
    }

    // Fallback to capability-match strategy
    return this.routeByCapability(task, candidates, performanceContext);
  }

  /**
   * Route using multi-armed bandit algorithm
   */
  private async routeWithBandit(
    task: Task,
    candidates: AgentQueryResult[],
    performanceContext?: any
  ): Promise<RoutingDecision> {
    if (!this.multiArmedBandit) {
      throw new Error("Multi-armed bandit not initialized");
    }

    // Select agent using bandit algorithm
    const selectedAgent = await this.multiArmedBandit.select(
      candidates,
      task.type as TaskType
    );

    // Create detailed routing decision
    const routingDecision = this.multiArmedBandit.createRoutingDecision(
      task.id,
      candidates,
      selectedAgent,
      task.type as TaskType
    );

    return {
      id: `routing-${task.id}-${Date.now()}`,
      taskId: task.id,
      selectedAgent,
      confidence: routingDecision.confidence,
      reason: routingDecision.rationale,
      strategy: "epsilon-greedy" as const, // Bandit uses epsilon-greedy
      alternatives: routingDecision.alternativesConsidered.map((alt) => ({
        agent: candidates.find((c) => c.agent.id === alt.agentId)?.agent!,
        score: alt.score,
        reason: alt.reason,
      })),
      timestamp: new Date(),
    };
  }

  /**
   * Get performance context for routing decision.
   *
   * @param task - Task being routed
   * @param candidates - Candidate agents
   * @returns Performance context with agent metrics
   */
  private async getPerformanceContext(
    task: Task,
    candidates: AgentQueryResult[]
  ): Promise<any> {
    if (!this.performanceTracker) {
      return null;
    }

    try {
      // Get performance stats to understand overall system performance
      const stats = this.performanceTracker.getStats();

      // Create basic performance context for each candidate
      const agentMetrics: Record<string, any> = {};
      for (const candidate of candidates) {
        // For now, use basic heuristics based on agent capabilities and system stats
        // In a full implementation, this would query historical performance data
        const capabilityScore = candidate.matchScore || 0.5;
        const systemSuccessRate = 0.8; // Default success rate assumption

        // Combine capability match with system performance
        const performanceScore =
          capabilityScore * 0.7 + systemSuccessRate * 0.3;

        agentMetrics[candidate.agent.id] = {
          overallScore: performanceScore,
          confidence: 0.5, // Low confidence for now
          recentMetrics: [], // Would be populated with actual historical data
        };
      }

      return {
        taskId: task.id,
        taskType: task.type,
        agentMetrics,
        systemStats: stats,
      };
    } catch (error) {
      console.warn("Failed to get performance context for routing:", error);
      return null;
    }
  }

  /**
   * Route by capability matching (fallback strategy)
   */
  private routeByCapability(
    task: Task,
    candidates: AgentQueryResult[],
    performanceContext?: any
  ): RoutingDecision {
    let selectedCandidate: AgentQueryResult;
    let strategy: RoutingStrategy = "capability-match";

    if (performanceContext?.agentMetrics) {
      // Use performance-weighted scoring (still using capability-match strategy)
      strategy = "capability-match";

      const scoredCandidates = candidates.map((candidate) => {
        const capabilityScore = candidate.matchScore;
        const performanceScore =
          performanceContext.agentMetrics[candidate.agent.id]?.overallScore ||
          0.5;
        const confidence =
          performanceContext.agentMetrics[candidate.agent.id]?.confidence ||
          0.5;

        // Weight: 70% capability, 30% performance history
        const weightedScore = capabilityScore * 0.7 + performanceScore * 0.3;

        return {
          ...candidate,
          weightedScore,
          performanceScore,
          confidence,
        };
      });

      // Sort by weighted score
      scoredCandidates.sort((a, b) => b.weightedScore - a.weightedScore);
      selectedCandidate = scoredCandidates[0];

      return {
        id: `routing-${task.id}-${Date.now()}`,
        taskId: task.id,
        selectedAgent: selectedCandidate.agent,
        confidence: selectedCandidate.matchScore,
        reason: `Performance-weighted selection: capability ${selectedCandidate.matchScore.toFixed(
          2
        )}`,
        strategy,
        alternatives: [],
        timestamp: new Date(),
      };
    } else {
      // Fallback to pure capability matching
      selectedCandidate = candidates[0];

      return {
        id: `routing-${task.id}-${Date.now()}`,
        taskId: task.id,
        selectedAgent: selectedCandidate.agent,
        confidence: selectedCandidate.matchScore,
        reason: `Best capability match: ${selectedCandidate.matchReason}`,
        strategy,
        alternatives: candidates.slice(1, 3).map((alt) => ({
          agent: alt.agent,
          score: alt.matchScore,
          reason: alt.matchReason,
        })),
        timestamp: new Date(),
      };
    }
  }

  /**
   * Record routing decision for history and analysis
   */
  private recordRoutingDecision(decision: RoutingDecision): void {
    this.routingHistory.set(decision.id, decision);

    // Keep history size manageable (last 1000 decisions)
    if (this.routingHistory.size > 1000) {
      const firstKey = this.routingHistory.keys().next().value;
      if (firstKey) {
        this.routingHistory.delete(firstKey);
      }
    }
  }

  /**
   * Update routing metrics
   */
  private updateMetrics(
    routingTimeMs: number,
    decision: RoutingDecision
  ): void {
    // Increment count first
    this.metrics.totalRoutingDecisions++;
    const count = this.metrics.totalRoutingDecisions;

    // Update average routing time (incremental average)
    this.metrics.averageRoutingTimeMs =
      (this.metrics.averageRoutingTimeMs * (count - 1) + routingTimeMs) / count;

    // Track exploration vs exploitation
    if (decision.strategy === "epsilon-greedy") {
      // Check if this was exploration (lower confidence often means exploration)
      if (decision.confidence < 0.8) {
        this.metrics.explorationRate =
          (this.metrics.explorationRate * (count - 1) + 1) / count;
      } else {
        this.metrics.exploitationRate =
          (this.metrics.exploitationRate * (count - 1) + 1) / count;
      }
    }
  }

  /**
   * Record routing outcome for feedback loop
   *
   * @param outcome - Routing outcome with success metrics
   */
  async recordRoutingOutcome(outcome: RoutingOutcome): Promise<void> {
    // Update agent performance in registry
    await this.agentRegistry.updatePerformance(
      outcome.routingDecision.selectedAgent.id,
      {
        success: outcome.success,
        qualityScore: outcome.qualityScore,
        latencyMs: outcome.latencyMs,
        taskType: outcome.routingDecision.selectedAgent.capabilities
          .taskTypes[0] as TaskType,
      }
    );

    // Update bandit with outcome
    if (this.multiArmedBandit) {
      this.multiArmedBandit.updateWithOutcome(
        outcome.routingDecision.selectedAgent.id,
        outcome.success,
        outcome.qualityScore,
        outcome.latencyMs
      );
    }

    // Update success rate metric
    const count = this.metrics.totalRoutingDecisions;
    const successValue = outcome.success ? 1 : 0;
    this.metrics.successRate =
      (this.metrics.successRate * count + successValue) / (count + 1);
  }

  /**
   * Get current routing metrics
   */
  getMetrics(): RoutingMetrics {
    return { ...this.metrics };
  }

  /**
   * Get routing statistics for monitoring
   */
  async getRoutingStats(): Promise<{
    metrics: RoutingMetrics;
    banditStats?: any;
    recentDecisions: Array<{
      taskId: string;
      agentId: string;
      strategy: string;
      confidence: number;
      timestamp: Date;
    }>;
  }> {
    // Get recent decisions (last 10)
    const recentDecisions = Array.from(this.routingHistory.values())
      .slice(-10)
      .map((decision) => ({
        taskId: decision.taskId,
        agentId: decision.selectedAgent.id,
        strategy: decision.strategy,
        confidence: decision.confidence,
        timestamp: decision.timestamp,
      }));

    const stats: any = {
      metrics: this.getMetrics(),
      recentDecisions,
    };

    // Include bandit stats if available
    if (this.multiArmedBandit) {
      stats.banditStats = this.multiArmedBandit.getStats();
    }

    return stats;
  }

  /**
   * Reset routing metrics (useful for testing)
   */
  resetMetrics(): void {
    this.metrics = {
      totalRoutingDecisions: 0,
      averageRoutingTimeMs: 0,
      explorationRate: 0,
      exploitationRate: 0,
      capabilityMismatchRate: 0,
      loadBalancingEffectiveness: 0,
      successRate: 0,
    };
  }

  /**
   * Reset multi-armed bandit state (useful for testing)
   */
  resetBandit(): void {
    if (this.multiArmedBandit) {
      this.multiArmedBandit.reset();
    }
  }
}

/**
 * Default Task Routing Manager Configuration
 */
export const defaultTaskRoutingConfig: TaskRoutingConfig = {
  enableBandit: true,
  minAgentsRequired: 1,
  maxAgentsToConsider: 10,
  defaultStrategy: "multi-armed-bandit",
  maxRoutingTimeMs: 100,
  loadBalancingWeight: 0.3,
  capabilityMatchWeight: 0.7,
};
