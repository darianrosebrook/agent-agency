/**
 * @fileoverview Load Balancer for Adaptive Resource Manager
 *
 * Distributes tasks across agents using configurable strategies.
 * Supports round-robin, least-loaded, weighted, and priority-based balancing.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import {
  LoadBalancingStrategy,
  type AgentResourceProfile,
  type ILoadBalancer,
  type LoadBalancingDecision,
  type ResourceAllocationRequest,
} from "@/types/resource-types";
import { ResourceMonitor } from "./ResourceMonitor";

/**
 * Load Balancer
 *
 * Selects optimal agents for task assignment using:
 * - Multiple balancing strategies
 * - Resource-aware decision making
 * - Load distribution tracking
 * - Fast decision times (<50ms)
 */
export class LoadBalancer implements ILoadBalancer {
  private logger: Logger;
  private strategy: LoadBalancingStrategy;
  private resourceMonitor: ResourceMonitor;
  private roundRobinIndex = 0;
  private loadDistribution: Map<string, number> = new Map();

  constructor(
    resourceMonitor: ResourceMonitor,
    strategy: LoadBalancingStrategy = LoadBalancingStrategy.LEAST_LOADED
  ) {
    this.logger = new Logger("LoadBalancer");
    this.resourceMonitor = resourceMonitor;
    this.strategy = strategy;
  }

  /**
   * Select agent for task assignment
   *
   * @param request Resource allocation request
   * @param availableAgents List of available agent IDs
   * @returns Load balancing decision
   */
  async selectAgent(
    request: ResourceAllocationRequest,
    availableAgents: string[]
  ): Promise<LoadBalancingDecision> {
    const startTime = Date.now();

    if (availableAgents.length === 0) {
      throw new Error("No available agents for task assignment");
    }

    // Get agent resource profiles
    const agentProfiles = await this.getAgentProfiles(availableAgents);

    // Select agent based on strategy
    let selectedAgentId: string;
    let rationale: string;
    let alternativesConsidered: string[] = [];

    switch (this.strategy) {
      case LoadBalancingStrategy.ROUND_ROBIN:
        ({ selectedAgentId, rationale, alternativesConsidered } =
          this.selectRoundRobin(availableAgents));
        break;

      case LoadBalancingStrategy.LEAST_LOADED:
        ({ selectedAgentId, rationale, alternativesConsidered } =
          await this.selectLeastLoaded(agentProfiles));
        break;

      case LoadBalancingStrategy.WEIGHTED:
        ({ selectedAgentId, rationale, alternativesConsidered } =
          await this.selectWeighted(agentProfiles));
        break;

      case LoadBalancingStrategy.PRIORITY_BASED:
        ({ selectedAgentId, rationale, alternativesConsidered } =
          await this.selectPriorityBased(request, agentProfiles));
        break;

      case LoadBalancingStrategy.RANDOM:
        ({ selectedAgentId, rationale, alternativesConsidered } =
          this.selectRandom(availableAgents));
        break;

      default:
        throw new Error(`Unknown load balancing strategy: ${this.strategy}`);
    }

    // Get agent load
    const agentProfile = agentProfiles.find(
      (p) => p.agentId === selectedAgentId
    );
    const agentLoad = agentProfile
      ? (agentProfile.currentTaskCount / agentProfile.maxTaskCapacity) * 100
      : 0;

    // Update load distribution
    const currentLoad = this.loadDistribution.get(selectedAgentId) ?? 0;
    this.loadDistribution.set(selectedAgentId, currentLoad + 1);

    const decision: LoadBalancingDecision = {
      selectedAgentId,
      strategy: this.strategy,
      agentLoad,
      timestamp: new Date(),
      rationale,
      alternativesConsidered,
      decisionDurationMs: Date.now() - startTime,
    };

    this.logger.debug("Load balancing decision made", {
      selectedAgent: selectedAgentId,
      strategy: this.strategy,
      load: agentLoad,
      decisionTimeMs: decision.decisionDurationMs,
    });

    return decision;
  }

  /**
   * Update load balancing strategy
   *
   * @param strategy New strategy
   */
  setStrategy(strategy: LoadBalancingStrategy): void {
    this.strategy = strategy;
    this.logger.info("Load balancing strategy updated", { strategy });
  }

  /**
   * Get current strategy
   *
   * @returns Current strategy
   */
  getStrategy(): LoadBalancingStrategy {
    return this.strategy;
  }

  /**
   * Get load distribution statistics
   *
   * @returns Map of agent ID to task count
   */
  async getLoadDistribution(): Promise<Map<string, number>> {
    return new Map(this.loadDistribution);
  }

  /**
   * Reset load distribution statistics
   */
  resetLoadDistribution(): void {
    this.loadDistribution.clear();
    this.logger.info("Load distribution statistics reset");
  }

  /**
   * Select agent using round-robin strategy
   */
  private selectRoundRobin(availableAgents: string[]): {
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  } {
    this.roundRobinIndex = this.roundRobinIndex % availableAgents.length;
    const selectedAgentId = availableAgents[this.roundRobinIndex];
    this.roundRobinIndex++;

    return {
      selectedAgentId,
      rationale: "Round-robin selection",
      alternativesConsidered: availableAgents.filter(
        (id) => id !== selectedAgentId
      ),
    };
  }

  /**
   * Select agent with least load
   */
  private async selectLeastLoaded(
    agentProfiles: AgentResourceProfile[]
  ): Promise<{
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  }> {
    if (agentProfiles.length === 0) {
      throw new Error("No agent profiles available for selection");
    }

    // Calculate load score for each agent
    const agentLoads = agentProfiles.map((profile) => ({
      agentId: profile.agentId,
      load:
        (profile.currentTaskCount / profile.maxTaskCapacity) * 100 +
        (profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent) / 2,
    }));

    // Sort by load (ascending)
    agentLoads.sort((a, b) => a.load - b.load);

    const selectedAgentId = agentLoads[0].agentId;
    const alternativesConsidered = agentLoads.slice(1, 4).map((a) => a.agentId);

    return {
      selectedAgentId,
      rationale: `Least loaded agent (${agentLoads[0].load.toFixed(1)}% load)`,
      alternativesConsidered,
    };
  }

  /**
   * Select agent using weighted strategy
   * Weights: 50% task capacity, 30% CPU, 20% memory
   */
  private async selectWeighted(agentProfiles: AgentResourceProfile[]): Promise<{
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  }> {
    if (agentProfiles.length === 0) {
      throw new Error("No agent profiles available for selection");
    }

    // Calculate weighted score for each agent (lower is better)
    const agentScores = agentProfiles.map((profile) => {
      const taskLoad =
        (profile.currentTaskCount / profile.maxTaskCapacity) * 100;
      const cpuLoad = profile.cpuUsage.usagePercent;
      const memoryLoad = profile.memoryUsage.usagePercent;

      const score = taskLoad * 0.5 + cpuLoad * 0.3 + memoryLoad * 0.2;

      return {
        agentId: profile.agentId,
        score,
      };
    });

    // Sort by score (ascending)
    agentScores.sort((a, b) => a.score - b.score);

    const selectedAgentId = agentScores[0].agentId;
    const alternativesConsidered = agentScores
      .slice(1, 4)
      .map((a) => a.agentId);

    return {
      selectedAgentId,
      rationale: `Weighted selection (score: ${agentScores[0].score.toFixed(
        1
      )})`,
      alternativesConsidered,
    };
  }

  /**
   * Select agent using priority-based strategy
   * High priority tasks get the best available agents
   */
  private async selectPriorityBased(
    request: ResourceAllocationRequest,
    agentProfiles: AgentResourceProfile[]
  ): Promise<{
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  }> {
    if (agentProfiles.length === 0) {
      throw new Error("No agent profiles available for selection");
    }

    // Filter healthy agents for high priority tasks
    const healthyAgents =
      request.priority >= 75
        ? agentProfiles.filter((p) => p.healthStatus === "healthy")
        : agentProfiles;

    if (healthyAgents.length === 0) {
      // Fall back to all agents if no healthy ones
      return this.selectLeastLoaded(agentProfiles);
    }

    // For high priority, select agent with most available capacity
    const agentCapacities = healthyAgents.map((profile) => ({
      agentId: profile.agentId,
      availableCapacity: profile.maxTaskCapacity - profile.currentTaskCount,
      resourceAvailability:
        100 -
        (profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent) / 2,
    }));

    // Sort by available capacity and resource availability
    agentCapacities.sort(
      (a, b) =>
        b.availableCapacity - a.availableCapacity ||
        b.resourceAvailability - a.resourceAvailability
    );

    const selectedAgentId = agentCapacities[0].agentId;
    const alternativesConsidered = agentCapacities
      .slice(1, 4)
      .map((a) => a.agentId);

    return {
      selectedAgentId,
      rationale: `Priority-based selection for ${request.priority} priority task`,
      alternativesConsidered,
    };
  }

  /**
   * Select agent randomly
   */
  private selectRandom(availableAgents: string[]): {
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  } {
    const randomIndex = Math.floor(Math.random() * availableAgents.length);
    const selectedAgentId = availableAgents[randomIndex];

    return {
      selectedAgentId,
      rationale: "Random selection",
      alternativesConsidered: availableAgents.filter(
        (id) => id !== selectedAgentId
      ),
    };
  }

  /**
   * Get agent profiles for available agents
   */
  private async getAgentProfiles(
    agentIds: string[]
  ): Promise<AgentResourceProfile[]> {
    const profiles: AgentResourceProfile[] = [];

    for (const agentId of agentIds) {
      const profile = await this.resourceMonitor.getAgentResources(agentId);
      if (profile) {
        profiles.push(profile);
      }
    }

    return profiles;
  }
}
