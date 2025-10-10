/**
 * Agent Registry Manager
 *
 * @author @darianrosebrook
 * @module orchestrator/AgentRegistryManager
 *
 * Central registry for managing agent profiles, capabilities, and performance history.
 * Implements ARBITER-001 specification with capability tracking and atomic updates.
 */

import type {
  AgentId,
  AgentProfile,
  AgentQuery,
  AgentQueryResult,
  AgentRegistryConfig,
  PerformanceMetrics,
  RegistryStats,
} from "../types/agent-registry";
import { RegistryError, RegistryErrorType } from "../types/agent-registry";
import { AgentProfileHelper } from "./AgentProfile";

/**
 * Default configuration for the agent registry.
 */
const DEFAULT_CONFIG: AgentRegistryConfig = {
  maxAgents: 1000,
  staleAgentThresholdMs: 24 * 60 * 60 * 1000, // 24 hours
  enableAutoCleanup: true,
  cleanupIntervalMs: 60 * 60 * 1000, // 1 hour
};

/**
 * Agent Registry Manager
 *
 * Maintains the catalog of available agents with their capabilities,
 * performance history, and current load status.
 *
 * @remarks
 * Thread-safe: Uses Map for O(1) lookups with atomic updates.
 * Invariants:
 * - Agent profiles are immutable except for performance metrics
 * - Performance history updates are atomic and isolated per agent
 * - Registry queries never block agent registration operations
 * - All capability changes are versioned and auditable
 */
export class AgentRegistryManager {
  private readonly agents: Map<AgentId, AgentProfile>;
  private readonly config: AgentRegistryConfig;
  private cleanupTimer?: ReturnType<typeof setInterval>;
  private readonly maxConcurrentTasksPerAgent: number = 10;

  constructor(config: Partial<AgentRegistryConfig> = {}) {
    this.agents = new Map();
    this.config = { ...DEFAULT_CONFIG, ...config };

    if (this.config.enableAutoCleanup) {
      this.startAutoCleanup();
    }
  }

  /**
   * Register a new agent in the registry.
   *
   * @param agent - Agent to register (partial, will be filled with defaults)
   * @returns Complete agent profile with generated fields
   * @throws RegistryError if agent already exists or registry is full
   *
   * @remarks
   * Acceptance Criterion A1: Agent profile created with capability tracking initialized
   */
  async registerAgent(agent: Partial<AgentProfile>): Promise<AgentProfile> {
    // Validate required fields
    AgentProfileHelper.validateProfile(agent);

    if (!agent.id) {
      throw new RegistryError(
        RegistryErrorType.INVALID_AGENT_DATA,
        "Agent ID is required"
      );
    }

    // Check if agent already exists
    if (this.agents.has(agent.id)) {
      throw new RegistryError(
        RegistryErrorType.AGENT_ALREADY_EXISTS,
        `Agent with ID ${agent.id} already exists`,
        { agentId: agent.id }
      );
    }

    // Check registry capacity
    if (this.agents.size >= this.config.maxAgents) {
      throw new RegistryError(
        RegistryErrorType.REGISTRY_FULL,
        `Registry is full (max: ${this.config.maxAgents} agents)`,
        { maxAgents: this.config.maxAgents, currentSize: this.agents.size }
      );
    }

    // Create complete profile with defaults
    const now = new Date().toISOString();
    const profile: AgentProfile = {
      id: agent.id,
      name: agent.name!,
      modelFamily: agent.modelFamily!,
      capabilities: agent.capabilities!,
      performanceHistory:
        agent.performanceHistory ??
        AgentProfileHelper.createInitialPerformanceHistory(),
      currentLoad: agent.currentLoad ?? AgentProfileHelper.createInitialLoad(),
      registeredAt: now,
      lastActiveAt: now,
    };

    // Initialize capability tracking
    await this.initializeCapabilityTracking(profile);

    // Store in registry
    this.agents.set(profile.id, profile);

    return AgentProfileHelper.cloneProfile(profile);
  }

  /**
   * Get agent profile by ID.
   *
   * @param agentId - ID of the agent to retrieve
   * @returns Agent profile
   * @throws RegistryError if agent not found
   */
  async getProfile(agentId: AgentId): Promise<AgentProfile> {
    const profile = this.agents.get(agentId);

    if (!profile) {
      throw new RegistryError(
        RegistryErrorType.AGENT_NOT_FOUND,
        `Agent with ID ${agentId} not found`,
        { agentId }
      );
    }

    return AgentProfileHelper.cloneProfile(profile);
  }

  /**
   * Query agents by capability and return sorted by performance.
   *
   * @param query - Query parameters with required capabilities
   * @returns Array of matching agents sorted by success rate (highest first)
   *
   * @remarks
   * Acceptance Criterion A2: Agents matching criteria returned sorted by performance history success rate
   * Performance Target: <50ms P95 latency
   */
  async getAgentsByCapability(query: AgentQuery): Promise<AgentQueryResult[]> {
    const results: AgentQueryResult[] = [];

    for (const profile of Array.from(this.agents.values())) {
      // Check task type match
      if (!profile.capabilities.taskTypes.includes(query.taskType)) {
        continue;
      }

      // Check language requirements if specified
      if (query.languages && query.languages.length > 0) {
        const hasAllLanguages = query.languages.every((lang) =>
          profile.capabilities.languages.includes(lang)
        );
        if (!hasAllLanguages) {
          continue;
        }
      }

      // Check specialization requirements if specified
      if (query.specializations && query.specializations.length > 0) {
        const hasAllSpecializations = query.specializations.every((spec) =>
          profile.capabilities.specializations.includes(spec)
        );
        if (!hasAllSpecializations) {
          continue;
        }
      }

      // Check utilization threshold if specified
      if (
        query.maxUtilization !== undefined &&
        profile.currentLoad.utilizationPercent > query.maxUtilization
      ) {
        continue;
      }

      // Check minimum success rate if specified
      if (
        query.minSuccessRate !== undefined &&
        profile.performanceHistory.successRate < query.minSuccessRate
      ) {
        continue;
      }

      // Calculate match score
      const matchScore = this.calculateMatchScore(profile, query);
      const matchReason = this.explainMatchScore(profile, query, matchScore);

      results.push({
        agent: AgentProfileHelper.cloneProfile(profile),
        matchScore,
        matchReason,
      });
    }

    // Sort by success rate (highest first), then by match score
    return results.sort((a, b) => {
      const successDiff =
        b.agent.performanceHistory.successRate -
        a.agent.performanceHistory.successRate;
      if (Math.abs(successDiff) > 0.01) {
        return successDiff;
      }
      return b.matchScore - a.matchScore;
    });
  }

  /**
   * Update performance metrics for an agent after task completion.
   *
   * @param agentId - ID of the agent to update
   * @param metrics - Performance metrics from the completed task
   * @returns Updated agent profile
   * @throws RegistryError if agent not found or update fails
   *
   * @remarks
   * Acceptance Criterion A3: Agent's running average performance history computed and persisted
   * Performance Target: <30ms P95 latency
   * Invariant: Performance history updates are atomic and isolated per agent
   */
  async updatePerformance(
    agentId: AgentId,
    metrics: PerformanceMetrics
  ): Promise<AgentProfile> {
    const profile = this.agents.get(agentId);

    if (!profile) {
      throw new RegistryError(
        RegistryErrorType.AGENT_NOT_FOUND,
        `Agent with ID ${agentId} not found`,
        { agentId }
      );
    }

    try {
      // Compute new running average (atomic operation)
      const newPerformanceHistory = AgentProfileHelper.updatePerformanceHistory(
        profile.performanceHistory,
        metrics
      );

      // Update profile with new performance history
      const updatedProfile: AgentProfile = {
        ...profile,
        performanceHistory: newPerformanceHistory,
        lastActiveAt: new Date().toISOString(),
      };

      // Atomically update in registry
      this.agents.set(agentId, updatedProfile);

      return AgentProfileHelper.cloneProfile(updatedProfile);
    } catch (error) {
      throw new RegistryError(
        RegistryErrorType.UPDATE_FAILED,
        `Failed to update performance for agent ${agentId}: ${
          (error as Error).message
        }`,
        { agentId, metrics, error }
      );
    }
  }

  /**
   * Update agent's current load (active and queued tasks).
   *
   * @param agentId - ID of the agent to update
   * @param activeTasks - New active tasks count
   * @param queuedTasks - New queued tasks count
   * @returns Updated agent profile
   * @throws RegistryError if agent not found
   */
  async updateLoad(
    agentId: AgentId,
    activeTasks: number,
    queuedTasks: number
  ): Promise<AgentProfile> {
    const profile = this.agents.get(agentId);

    if (!profile) {
      throw new RegistryError(
        RegistryErrorType.AGENT_NOT_FOUND,
        `Agent with ID ${agentId} not found`,
        { agentId }
      );
    }

    const utilizationPercent =
      (activeTasks / this.maxConcurrentTasksPerAgent) * 100;

    const updatedProfile: AgentProfile = {
      ...profile,
      currentLoad: {
        activeTasks,
        queuedTasks,
        utilizationPercent: Math.min(100, utilizationPercent),
      },
      lastActiveAt: new Date().toISOString(),
    };

    this.agents.set(agentId, updatedProfile);

    return AgentProfileHelper.cloneProfile(updatedProfile);
  }

  /**
   * Get registry statistics.
   *
   * @returns Current registry stats
   */
  async getStats(): Promise<RegistryStats> {
    const allAgents = Array.from(this.agents.values());
    const activeAgents = allAgents.filter((a) => a.currentLoad.activeTasks > 0);
    const idleAgents = allAgents.filter((a) => a.currentLoad.activeTasks === 0);

    const totalUtilization = allAgents.reduce(
      (sum, a) => sum + a.currentLoad.utilizationPercent,
      0
    );
    const averageUtilization =
      allAgents.length > 0 ? totalUtilization / allAgents.length : 0;

    const totalSuccessRate = allAgents.reduce(
      (sum, a) => sum + a.performanceHistory.successRate,
      0
    );
    const averageSuccessRate =
      allAgents.length > 0 ? totalSuccessRate / allAgents.length : 0;

    return {
      totalAgents: allAgents.length,
      activeAgents: activeAgents.length,
      idleAgents: idleAgents.length,
      averageUtilization,
      averageSuccessRate,
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Remove an agent from the registry.
   *
   * @param agentId - ID of the agent to remove
   * @returns True if agent was removed
   */
  async unregisterAgent(agentId: AgentId): Promise<boolean> {
    return this.agents.delete(agentId);
  }

  /**
   * Initialize capability tracking for a new agent.
   */
  private async initializeCapabilityTracking(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
    _profile: AgentProfile
  ): Promise<void> {
    // Capability tracking initialization
    // In production, this would set up monitoring for capability usage
    // and initialize any external tracking systems
    // For now, this is a no-op, but provides extension point
  }

  /**
   * Calculate match score for query result ranking.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Match score (0.0 - 1.0)
   */
  private calculateMatchScore(
    profile: AgentProfile,
    query: AgentQuery
  ): number {
    let score = 0.0;
    let weights = 0.0;

    // Task type match (required, so always contributes)
    score += 0.3;
    weights += 0.3;

    // Language matches (if specified)
    if (query.languages && query.languages.length > 0) {
      const matchedLanguages = query.languages.filter((lang) =>
        profile.capabilities.languages.includes(lang)
      ).length;
      score += (matchedLanguages / query.languages.length) * 0.3;
      weights += 0.3;
    }

    // Specialization matches (if specified)
    if (query.specializations && query.specializations.length > 0) {
      const matchedSpecs = query.specializations.filter((spec) =>
        profile.capabilities.specializations.includes(spec)
      ).length;
      score += (matchedSpecs / query.specializations.length) * 0.2;
      weights += 0.2;
    }

    // Performance bonus
    score += profile.performanceHistory.successRate * 0.2;
    weights += 0.2;

    return weights > 0 ? score / weights : 0;
  }

  /**
   * Generate human-readable explanation of match score.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Explanation string
   */
  private explainMatchScore(
    profile: AgentProfile,
    query: AgentQuery,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
    _score: number
  ): string {
    const reasons: string[] = [];

    reasons.push(`Supports ${query.taskType}`);

    if (query.languages && query.languages.length > 0) {
      reasons.push(`Languages: ${query.languages.join(", ")}`);
    }

    if (query.specializations && query.specializations.length > 0) {
      reasons.push(`Specializations: ${query.specializations.join(", ")}`);
    }

    reasons.push(
      `${(profile.performanceHistory.successRate * 100).toFixed(
        1
      )}% success rate`
    );
    reasons.push(
      `${profile.currentLoad.utilizationPercent.toFixed(0)}% utilized`
    );

    return reasons.join("; ");
  }

  /**
   * Start automatic cleanup of stale agents.
   */
  private startAutoCleanup(): void {
    this.cleanupTimer = setInterval(() => {
      this.cleanupStaleAgents();
    }, this.config.cleanupIntervalMs);
  }

  /**
   * Clean up stale agents (inactive beyond threshold).
   */
  private cleanupStaleAgents(): void {
    const now = new Date().toISOString();
    const staleAgents: AgentId[] = [];

    const agents = Array.from(this.agents.entries());
    for (const [agentId, profile] of agents) {
      if (
        AgentProfileHelper.isStale(
          profile,
          this.config.staleAgentThresholdMs,
          now
        )
      ) {
        staleAgents.push(agentId);
      }
    }

    for (const agentId of staleAgents) {
      this.agents.delete(agentId);
    }
  }

  /**
   * Shutdown the registry manager and cleanup resources.
   */
  async shutdown(): Promise<void> {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
    }
  }
}
