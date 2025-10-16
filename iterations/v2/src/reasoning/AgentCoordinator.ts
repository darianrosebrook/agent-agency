/**
 * AgentCoordinator
 *
 * Manages agent participation in debates including role assignment,
 * capability matching, and load balancing.
 *
 * @module reasoning/AgentCoordinator
 * @author @darianrosebrook
 */

import { AgentRole, ReasoningEngineError } from "@/types/reasoning";

/**
 * Agent capability information
 */
export interface AgentCapability {
  agentId: string;
  roles: AgentRole[];
  expertise: string[];
  currentLoad: number;
  maxLoad: number;
  availabilityScore: number;
}

/**
 * Role assignment request
 */
export interface RoleAssignmentRequest {
  requiredRoles: AgentRole[];
  topic: string;
  expertiseKeywords?: string[];
  excludeAgents?: string[];
}

/**
 * Role assignment result
 */
export interface RoleAssignmentResult {
  assignments: Array<{
    agentId: string;
    role: AgentRole;
    matchScore: number;
    reason: string;
  }>;
  confidence: number;
  alternativeAssignments?: Array<{
    agentId: string;
    role: AgentRole;
    matchScore: number;
  }>;
}

/**
 * Load balancing strategy
 */
export enum LoadBalancingStrategy {
  _ROUND_ROBIN = "round_robin",
  _LEAST_LOADED = "least_loaded",
  _CAPABILITY_BASED = "capability_based",
  _HYBRID = "hybrid",
}

/**
 * AgentCoordinator configuration
 */
export interface AgentCoordinatorConfig {
  loadBalancingStrategy: LoadBalancingStrategy;
  maxAgentsPerDebate: number;
  minAgentsPerDebate: number;
  enableCapabilityMatching: boolean;
  expertiseMatchThreshold: number;
}

/**
 * Coordinates agent participation in debates
 */
export class AgentCoordinator {
  private agentCapabilities: Map<string, AgentCapability>;
  private agentDebateHistory: Map<string, string[]>;
  private config: AgentCoordinatorConfig;
  private lastAssignmentIndex: number;

  constructor(config: Partial<AgentCoordinatorConfig> = {}) {
    this.agentCapabilities = new Map();
    this.agentDebateHistory = new Map();
    this.lastAssignmentIndex = 0;

    this.config = {
      loadBalancingStrategy:
        config.loadBalancingStrategy ?? LoadBalancingStrategy.HYBRID,
      maxAgentsPerDebate: config.maxAgentsPerDebate ?? 10,
      minAgentsPerDebate: config.minAgentsPerDebate ?? 2,
      enableCapabilityMatching: config.enableCapabilityMatching ?? true,
      expertiseMatchThreshold: config.expertiseMatchThreshold ?? 0.5,
    };
  }

  /**
   * Registers an agent with their capabilities
   */
  public registerAgent(capability: AgentCapability): void {
    if (!capability.agentId || capability.agentId.trim().length === 0) {
      throw new ReasoningEngineError(
        "Agent ID cannot be empty",
        "INVALID_AGENT_ID"
      );
    }

    if (!capability.roles || capability.roles.length === 0) {
      throw new ReasoningEngineError(
        `Agent ${capability.agentId} must have at least one role`,
        "NO_ROLES"
      );
    }

    if (capability.maxLoad < 1) {
      throw new ReasoningEngineError(
        `Agent ${capability.agentId} maxLoad must be at least 1`,
        "INVALID_MAX_LOAD"
      );
    }

    this.agentCapabilities.set(capability.agentId, {
      ...capability,
      currentLoad: capability.currentLoad ?? 0,
      availabilityScore: this.calculateAvailabilityScore(capability),
    });
  }

  /**
   * Unregisters an agent
   */
  public unregisterAgent(agentId: string): boolean {
    return this.agentCapabilities.delete(agentId);
  }

  /**
   * Gets an agent's current capabilities
   */
  public getAgentCapability(agentId: string): AgentCapability | undefined {
    return this.agentCapabilities.get(agentId);
  }

  /**
   * Assigns roles to agents for a debate
   */
  public assignRoles(request: RoleAssignmentRequest): RoleAssignmentResult {
    // Validate request
    if (!request.requiredRoles || request.requiredRoles.length === 0) {
      throw new ReasoningEngineError(
        "At least one role is required",
        "NO_ROLES_REQUIRED"
      );
    }

    if (request.requiredRoles.length < this.config.minAgentsPerDebate) {
      throw new ReasoningEngineError(
        `Minimum ${this.config.minAgentsPerDebate} agents required`,
        "INSUFFICIENT_AGENTS"
      );
    }

    if (request.requiredRoles.length > this.config.maxAgentsPerDebate) {
      throw new ReasoningEngineError(
        `Maximum ${this.config.maxAgentsPerDebate} agents allowed`,
        "TOO_MANY_AGENTS"
      );
    }

    // Get available agents for each role
    const availableAgents = this.getAvailableAgents();

    if (availableAgents.length === 0) {
      throw new ReasoningEngineError(
        "No agents available for assignment",
        "NO_AGENTS_AVAILABLE"
      );
    }

    // Assign roles based on strategy
    const assignments = this.performRoleAssignment(request, availableAgents);

    // Calculate confidence in assignments
    const confidence = this.calculateAssignmentConfidence(assignments);

    // Generate alternative assignments
    const alternativeAssignments = this.generateAlternatives(
      request,
      availableAgents,
      assignments.map((a) => a.agentId)
    );

    return {
      assignments,
      confidence,
      alternativeAssignments,
    };
  }

  /**
   * Updates an agent's load after debate assignment
   */
  public updateAgentLoad(
    agentId: string,
    debateId: string,
    increment: boolean
  ): void {
    const capability = this.agentCapabilities.get(agentId);

    if (!capability) {
      throw new ReasoningEngineError(
        `Agent ${agentId} not found`,
        "AGENT_NOT_FOUND"
      );
    }

    // Update load
    capability.currentLoad += increment ? 1 : -1;
    capability.currentLoad = Math.max(0, capability.currentLoad);

    // Update availability score
    capability.availabilityScore = this.calculateAvailabilityScore(capability);

    // Track debate history
    if (increment) {
      const history = this.agentDebateHistory.get(agentId) ?? [];
      history.push(debateId);
      this.agentDebateHistory.set(agentId, history);
    }

    this.agentCapabilities.set(agentId, capability);
  }

  /**
   * Gets statistics about agent coordination
   */
  public getCoordinationStats(): {
    totalAgents: number;
    availableAgents: number;
    averageLoad: number;
    utilizationRate: number;
  } {
    const totalAgents = this.agentCapabilities.size;
    const availableAgents = this.getAvailableAgents().length;

    let totalLoad = 0;
    let totalCapacity = 0;

    for (const capability of this.agentCapabilities.values()) {
      totalLoad += capability.currentLoad;
      totalCapacity += capability.maxLoad;
    }

    const averageLoad = totalAgents > 0 ? totalLoad / totalAgents : 0;
    const utilizationRate = totalCapacity > 0 ? totalLoad / totalCapacity : 0;

    return {
      totalAgents,
      availableAgents,
      averageLoad,
      utilizationRate,
    };
  }

  /**
   * Calculates availability score for an agent
   */
  private calculateAvailabilityScore(capability: AgentCapability): number {
    if (capability.currentLoad >= capability.maxLoad) {
      return 0;
    }

    const loadRatio = capability.currentLoad / capability.maxLoad;
    return 1 - loadRatio;
  }

  /**
   * Gets all available agents
   */
  private getAvailableAgents(): AgentCapability[] {
    return Array.from(this.agentCapabilities.values()).filter(
      (cap) => cap.currentLoad < cap.maxLoad
    );
  }

  /**
   * Performs role assignment based on strategy
   */
  private performRoleAssignment(
    request: RoleAssignmentRequest,
    availableAgents: AgentCapability[]
  ): Array<{
    agentId: string;
    role: AgentRole;
    matchScore: number;
    reason: string;
  }> {
    switch (this.config.loadBalancingStrategy) {
      case LoadBalancingStrategy.ROUND_ROBIN:
        return this.assignRoundRobin(request, availableAgents);
      case LoadBalancingStrategy.LEAST_LOADED:
        return this.assignLeastLoaded(request, availableAgents);
      case LoadBalancingStrategy.CAPABILITY_BASED:
        return this.assignCapabilityBased(request, availableAgents);
      case LoadBalancingStrategy.HYBRID:
        return this.assignHybrid(request, availableAgents);
      default:
        throw new ReasoningEngineError(
          `Unknown load balancing strategy: ${this.config.loadBalancingStrategy}`,
          "UNKNOWN_STRATEGY"
        );
    }
  }

  /**
   * Round-robin assignment
   */
  private assignRoundRobin(
    request: RoleAssignmentRequest,
    availableAgents: AgentCapability[]
  ): Array<{
    agentId: string;
    role: AgentRole;
    matchScore: number;
    reason: string;
  }> {
    const assignments: Array<{
      agentId: string;
      role: AgentRole;
      matchScore: number;
      reason: string;
    }> = [];

    for (const role of request.requiredRoles) {
      // Find agents that can fulfill this role
      const capableAgents = availableAgents.filter((agent) =>
        agent.roles.includes(role)
      );

      if (capableAgents.length === 0) {
        throw new ReasoningEngineError(
          `No agents available for role: ${role}`,
          "NO_AGENTS_FOR_ROLE"
        );
      }

      // Round-robin selection
      const selectedAgent =
        capableAgents[this.lastAssignmentIndex % capableAgents.length];
      this.lastAssignmentIndex++;

      assignments.push({
        agentId: selectedAgent.agentId,
        role,
        matchScore: 1.0,
        reason: "Round-robin assignment",
      });
    }

    return assignments;
  }

  /**
   * Least-loaded assignment
   */
  private assignLeastLoaded(
    request: RoleAssignmentRequest,
    availableAgents: AgentCapability[]
  ): Array<{
    agentId: string;
    role: AgentRole;
    matchScore: number;
    reason: string;
  }> {
    const assignments: Array<{
      agentId: string;
      role: AgentRole;
      matchScore: number;
      reason: string;
    }> = [];

    for (const role of request.requiredRoles) {
      // Find agents that can fulfill this role
      const capableAgents = availableAgents.filter((agent) =>
        agent.roles.includes(role)
      );

      if (capableAgents.length === 0) {
        throw new ReasoningEngineError(
          `No agents available for role: ${role}`,
          "NO_AGENTS_FOR_ROLE"
        );
      }

      // Sort by current load (ascending)
      capableAgents.sort((a, b) => a.currentLoad - b.currentLoad);

      const selectedAgent = capableAgents[0];

      assignments.push({
        agentId: selectedAgent.agentId,
        role,
        matchScore: selectedAgent.availabilityScore,
        reason: `Least loaded (load: ${selectedAgent.currentLoad}/${selectedAgent.maxLoad})`,
      });
    }

    return assignments;
  }

  /**
   * Capability-based assignment
   */
  private assignCapabilityBased(
    request: RoleAssignmentRequest,
    availableAgents: AgentCapability[]
  ): Array<{
    agentId: string;
    role: AgentRole;
    matchScore: number;
    reason: string;
  }> {
    const assignments: Array<{
      agentId: string;
      role: AgentRole;
      matchScore: number;
      reason: string;
    }> = [];

    for (const role of request.requiredRoles) {
      // Find agents that can fulfill this role
      const capableAgents = availableAgents.filter((agent) =>
        agent.roles.includes(role)
      );

      if (capableAgents.length === 0) {
        throw new ReasoningEngineError(
          `No agents available for role: ${role}`,
          "NO_AGENTS_FOR_ROLE"
        );
      }

      // Calculate expertise match for each agent
      const agentsWithScores = capableAgents.map((agent) => ({
        agent,
        score: this.calculateExpertiseMatch(
          agent,
          request.expertiseKeywords ?? []
        ),
      }));

      // Sort by expertise match (descending)
      agentsWithScores.sort((a, b) => b.score - a.score);

      const selected = agentsWithScores[0];

      assignments.push({
        agentId: selected.agent.agentId,
        role,
        matchScore: selected.score,
        reason: `Best expertise match (score: ${selected.score.toFixed(2)})`,
      });
    }

    return assignments;
  }

  /**
   * Hybrid assignment (combines load balancing and capability matching)
   */
  private assignHybrid(
    request: RoleAssignmentRequest,
    availableAgents: AgentCapability[]
  ): Array<{
    agentId: string;
    role: AgentRole;
    matchScore: number;
    reason: string;
  }> {
    const assignments: Array<{
      agentId: string;
      role: AgentRole;
      matchScore: number;
      reason: string;
    }> = [];

    for (const role of request.requiredRoles) {
      // Find agents that can fulfill this role
      const capableAgents = availableAgents.filter((agent) =>
        agent.roles.includes(role)
      );

      if (capableAgents.length === 0) {
        throw new ReasoningEngineError(
          `No agents available for role: ${role}`,
          "NO_AGENTS_FOR_ROLE"
        );
      }

      // Calculate hybrid score (availability + expertise)
      const agentsWithScores = capableAgents.map((agent) => {
        const expertiseScore = this.calculateExpertiseMatch(
          agent,
          request.expertiseKeywords ?? []
        );

        // Weighted combination: 50% availability, 50% expertise
        const hybridScore =
          agent.availabilityScore * 0.5 + expertiseScore * 0.5;

        return {
          agent,
          hybridScore,
          expertiseScore,
        };
      });

      // Sort by hybrid score (descending)
      agentsWithScores.sort((a, b) => b.hybridScore - a.hybridScore);

      const selected = agentsWithScores[0];

      assignments.push({
        agentId: selected.agent.agentId,
        role,
        matchScore: selected.hybridScore,
        reason: `Hybrid assignment (availability: ${selected.agent.availabilityScore.toFixed(
          2
        )}, expertise: ${selected.expertiseScore.toFixed(2)})`,
      });
    }

    return assignments;
  }

  /**
   * Calculates expertise match score
   */
  private calculateExpertiseMatch(
    agent: AgentCapability,
    keywords: string[]
  ): number {
    if (keywords.length === 0) {
      return 1.0; // No expertise requirement, all agents match equally
    }

    if (agent.expertise.length === 0) {
      return 0.0; // No expertise declared
    }

    // Count matching keywords
    let matches = 0;

    for (const keyword of keywords) {
      const lowerKeyword = keyword.toLowerCase();
      if (
        agent.expertise.some((exp) => exp.toLowerCase().includes(lowerKeyword))
      ) {
        matches++;
      }
    }

    return matches / keywords.length;
  }

  /**
   * Calculates confidence in assignments
   */
  private calculateAssignmentConfidence(
    assignments: Array<{ matchScore: number }>
  ): number {
    if (assignments.length === 0) {
      return 0;
    }

    const averageScore =
      assignments.reduce((sum, a) => sum + a.matchScore, 0) /
      assignments.length;

    return averageScore;
  }

  /**
   * Generates alternative assignments
   */
  private generateAlternatives(
    request: RoleAssignmentRequest,
    availableAgents: AgentCapability[],
    excludeAgents: string[]
  ): Array<{ agentId: string; role: AgentRole; matchScore: number }> {
    const alternatives: Array<{
      agentId: string;
      role: AgentRole;
      matchScore: number;
    }> = [];

    // For each role, find next best agent not in primary assignments
    for (const role of request.requiredRoles) {
      const capableAgents = availableAgents.filter(
        (agent) =>
          agent.roles.includes(role) && !excludeAgents.includes(agent.agentId)
      );

      if (capableAgents.length > 0) {
        // Use capability-based scoring for alternatives
        const agentsWithScores = capableAgents.map((agent) => ({
          agentId: agent.agentId,
          role,
          matchScore: this.calculateExpertiseMatch(
            agent,
            request.expertiseKeywords ?? []
          ),
        }));

        agentsWithScores.sort((a, b) => b.matchScore - a.matchScore);

        if (agentsWithScores[0]) {
          alternatives.push(agentsWithScores[0]);
        }
      }
    }

    return alternatives;
  }
}
