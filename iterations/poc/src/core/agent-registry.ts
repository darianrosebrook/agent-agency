/**
 * Agent Registry & Capability Management
 *
 * Manages agent registration, capability profiles, and cross-agent learning.
 * Enables knowledge sharing and collaborative problem solving.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { MultiTenantMemoryManager } from "../memory/MultiTenantMemoryManager";

export interface AgentCapability {
  name: string;
  level: number; // 0.0 to 1.0
  confidence: number; // 0.0 to 1.0
  experience: number; // Number of tasks completed
  lastUsed: Date;
  successRate: number;
}

export interface AgentProfile {
  id: string;
  name: string;
  tenantId: string;
  capabilities: Map<string, AgentCapability>;
  expertise: string[];
  availability: "online" | "busy" | "offline";
  performance: {
    overall: number;
    tasksCompleted: number;
    averageQuality: number;
    specializationScore: number;
  };
  learning: {
    patternsLearned: number;
    knowledgeShared: number;
    collaborations: number;
  };
  created: Date;
  lastActive: Date;
}

export interface KnowledgePattern {
  id: string;
  type: "success-pattern" | "error-pattern" | "best-practice";
  domain: string;
  description: string;
  implementation: any;
  quality: number;
  usage: number;
  sharedBy: string;
  learnedBy: string[];
  created: Date;
  lastUsed: Date;
}

export interface LearningEvent {
  agentId: string;
  eventType:
    | "task-completed"
    | "pattern-learned"
    | "collaboration"
    | "capability-evolved";
  data: any;
  timestamp: Date;
}

/**
 * Agent Registry manages agent capabilities and cross-agent learning
 */
export class AgentRegistry extends EventEmitter {
  private agents = new Map<string, AgentProfile>();
  private knowledgePatterns = new Map<string, KnowledgePattern>();
  private learningHistory: LearningEvent[] = [];
  private memoryManager: MultiTenantMemoryManager;

  constructor(memoryManager: MultiTenantMemoryManager) {
    super();
    this.memoryManager = memoryManager;
  }

  /**
   * Register a new agent with initial capabilities
   */
  async registerAgent(
    profile: Omit<
      AgentProfile,
      "capabilities" | "performance" | "learning" | "created" | "lastActive"
    > & {
      initialCapabilities?: Record<string, number>;
    }
  ): Promise<AgentProfile> {
    const agentProfile: AgentProfile = {
      ...profile,
      capabilities: new Map(),
      performance: {
        overall: 0.5,
        tasksCompleted: 0,
        averageQuality: 0.5,
        specializationScore: 0,
      },
      learning: {
        patternsLearned: 0,
        knowledgeShared: 0,
        collaborations: 0,
      },
      created: new Date(),
      lastActive: new Date(),
    };

    // Initialize capabilities
    if (profile.initialCapabilities) {
      Object.entries(profile.initialCapabilities).forEach(
        ([capability, level]) => {
          agentProfile.capabilities.set(capability, {
            name: capability,
            level,
            confidence: 0.5,
            experience: 0,
            lastUsed: new Date(),
            successRate: 0.5,
          });
        }
      );
    }

    this.agents.set(profile.id, agentProfile);

    // Store in memory for persistence
    await this.memoryManager.storeExperience(profile.tenantId, {
      memoryId: `agent-profile-${profile.id}`,
      relevanceScore: 1.0,
      contextMatch: {
        similarityScore: 1.0,
        keywordMatches: ["agent", "profile", profile.id],
        semanticMatches: ["agent registration", "capability profile"],
        temporalAlignment: 1.0,
      },
      content: {
        type: "agent-profile",
        agentId: profile.id,
        data: agentProfile,
      },
    });

    this.emit("agent-registered", agentProfile);
    return agentProfile;
  }

  /**
   * Find agents with specific capabilities
   */
  findAgentsByCapability(
    capability: string,
    minLevel: number = 0.5,
    tenantId?: string
  ): AgentProfile[] {
    const candidates = tenantId
      ? [this.agents.get(tenantId)].filter(
          (agent): agent is AgentProfile => agent !== undefined
        )
      : Array.from(this.agents.values());

    return candidates.filter((agent) => {
      if (!agent || agent.availability !== "online") return false;
      const cap = agent.capabilities.get(capability);
      return cap && cap.level >= minLevel;
    });
  }

  /**
   * Share knowledge pattern between agents
   */
  async shareKnowledgePattern(
    fromAgentId: string,
    toAgentId: string,
    pattern: Omit<
      KnowledgePattern,
      "id" | "sharedBy" | "learnedBy" | "created" | "lastUsed"
    >
  ): Promise<boolean> {
    const fromAgent = this.agents.get(fromAgentId);
    const toAgent = this.agents.get(toAgentId);

    if (!fromAgent || !toAgent) {
      return false;
    }

    const patternId = `pattern-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 9)}`;
    const knowledgePattern: KnowledgePattern = {
      ...pattern,
      id: patternId,
      sharedBy: fromAgentId,
      learnedBy: [toAgentId],
      created: new Date(),
      lastUsed: new Date(),
    };

    this.knowledgePatterns.set(patternId, knowledgePattern);

    // Update learning stats
    fromAgent.learning.knowledgeShared++;
    toAgent.learning.patternsLearned++;

    // Store in memory
    await this.memoryManager.storeExperience(fromAgent.tenantId, {
      memoryId: `knowledge-pattern-${patternId}`,
      relevanceScore: 0.9,
      contextMatch: {
        similarityScore: 0.9,
        keywordMatches: [pattern.domain, "knowledge", "pattern"],
        semanticMatches: ["knowledge sharing", "best practice", pattern.type],
        temporalAlignment: 1.0,
      },
      content: {
        type: "knowledge-pattern",
        patternId,
        data: knowledgePattern,
      },
    });

    // Record learning event
    this.recordLearningEvent({
      agentId: toAgentId,
      eventType: "pattern-learned",
      data: { patternId, fromAgentId },
      timestamp: new Date(),
    });

    this.emit("knowledge-shared", {
      pattern: knowledgePattern,
      from: fromAgent,
      to: toAgent,
    });
    return true;
  }

  /**
   * Evolve agent capabilities based on task performance
   */
  async evolveCapability(
    agentId: string,
    capability: string,
    taskResult: {
      success: boolean;
      quality: number;
      complexity: "simple" | "medium" | "complex";
      duration: number;
    }
  ): Promise<boolean> {
    const agent = this.agents.get(agentId);
    if (!agent) return false;

    const cap = agent.capabilities.get(capability);
    if (!cap) {
      // Initialize new capability
      agent.capabilities.set(capability, {
        name: capability,
        level: 0.3,
        confidence: 0.5,
        experience: 0,
        lastUsed: new Date(),
        successRate: 0.5,
      });
    }

    const currentCap = agent.capabilities.get(capability)!;

    // Calculate learning rate based on task complexity
    const complexityMultiplier = { simple: 0.05, medium: 0.1, complex: 0.15 };
    const learningRate = complexityMultiplier[taskResult.complexity];

    // Update capability level
    const qualityBonus = taskResult.quality * 0.2;
    const successBonus = taskResult.success ? 0.1 : -0.05;

    currentCap.level = Math.min(
      1.0,
      Math.max(
        0.0,
        currentCap.level + learningRate + qualityBonus + successBonus
      )
    );

    // Update success rate (weighted average)
    const totalTasks = currentCap.experience + 1;
    currentCap.successRate =
      (currentCap.successRate * currentCap.experience +
        (taskResult.success ? 1 : 0)) /
      totalTasks;

    // Update experience and metadata
    currentCap.experience++;
    currentCap.lastUsed = new Date();
    currentCap.confidence = Math.min(1.0, currentCap.confidence + 0.05);

    // Update agent performance metrics
    agent.performance.tasksCompleted++;
    agent.performance.averageQuality =
      (agent.performance.averageQuality *
        (agent.performance.tasksCompleted - 1) +
        taskResult.quality) /
      agent.performance.tasksCompleted;

    // Calculate specialization score (how focused the agent is)
    const capabilities = Array.from(agent.capabilities.values());
    const avgCapability =
      capabilities.reduce((sum, cap) => sum + cap.level, 0) /
      capabilities.length;
    const variance =
      capabilities.reduce(
        (sum, cap) => sum + Math.pow(cap.level - avgCapability, 2),
        0
      ) / capabilities.length;
    agent.performance.specializationScore = 1 - Math.sqrt(variance); // Lower variance = higher specialization

    agent.lastActive = new Date();

    // Record learning event
    this.recordLearningEvent({
      agentId,
      eventType: "capability-evolved",
      data: {
        capability,
        oldLevel: currentCap.level - learningRate,
        newLevel: currentCap.level,
      },
      timestamp: new Date(),
    });

    this.emit("capability-evolved", {
      agent,
      capability: currentCap,
      taskResult,
    });

    return true;
  }

  /**
   * Find best agent for a task based on capabilities and performance
   */
  findBestAgentForTask(task: {
    type: string;
    complexity: "simple" | "medium" | "complex" | "expert";
    requiredCapabilities: string[];
    tenantId?: string;
  }): AgentProfile | null {
    const candidates = task.tenantId
      ? [this.agents.get(task.tenantId)].filter(
          (agent): agent is AgentProfile => agent !== undefined
        )
      : Array.from(this.agents.values());

    let bestAgent: AgentProfile | null = null;
    let bestScore = -1;

    for (const agent of candidates) {
      if (!agent || agent.availability !== "online") continue;

      let score = 0;
      let hasRequiredCapabilities = true;

      // Check required capabilities
      for (const reqCap of task.requiredCapabilities) {
        const cap = agent.capabilities.get(reqCap);
        if (!cap || cap.level < 0.5) {
          hasRequiredCapabilities = false;
          break;
        }
        score += cap.level * cap.confidence;
      }

      if (!hasRequiredCapabilities) continue;

      // Factor in performance and specialization
      score *= agent.performance.overall;
      score *= 1 + agent.performance.specializationScore;

      // Factor in recent activity (prefer recently active agents)
      const hoursSinceActive =
        (Date.now() - agent.lastActive.getTime()) / (1000 * 60 * 60);
      const recencyBonus = Math.max(0, 1 - hoursSinceActive / 24); // Bonus for active within 24h
      score *= 1 + recencyBonus * 0.1;

      if (score > bestScore) {
        bestScore = score;
        bestAgent = agent;
      }
    }

    return bestAgent;
  }

  /**
   * Get learning insights for an agent
   */
  getAgentInsights(agentId: string): {
    strengths: string[];
    weaknesses: string[];
    learningOpportunities: string[];
    collaborationSuggestions: string[];
  } {
    const agent = this.agents.get(agentId);
    if (!agent) {
      return {
        strengths: [],
        weaknesses: [],
        learningOpportunities: [],
        collaborationSuggestions: [],
      };
    }

    const capabilities = Array.from(agent.capabilities.values());

    // Identify strengths (high-level capabilities)
    const strengths = capabilities
      .filter((cap) => cap.level > 0.8)
      .sort((a, b) => b.level - a.level)
      .slice(0, 3)
      .map((cap) => cap.name);

    // Identify weaknesses (low-level capabilities)
    const weaknesses = capabilities
      .filter((cap) => cap.level < 0.4)
      .sort((a, b) => a.level - b.level)
      .slice(0, 3)
      .map((cap) => cap.name);

    // Find learning opportunities from other agents' patterns
    const learningOpportunities = Array.from(this.knowledgePatterns.values())
      .filter((pattern) => !pattern.learnedBy.includes(agentId))
      .filter((pattern) =>
        agent.expertise.some((exp) => pattern.domain.includes(exp))
      )
      .slice(0, 3)
      .map((pattern) => pattern.description);

    // Suggest collaboration partners
    const collaborationSuggestions = Array.from(this.agents.values())
      .filter(
        (other) => other.id !== agentId && other.tenantId === agent.tenantId
      )
      .filter((other) => {
        // Find agents with complementary skills
        const complementarySkills = Array.from(
          other.capabilities.keys()
        ).filter(
          (skill) =>
            !agent.capabilities.has(skill) ||
            agent.capabilities.get(skill)!.level < 0.5
        );
        return complementarySkills.length > 0;
      })
      .slice(0, 3)
      .map(
        (other) =>
          `${other.name} (${Array.from(other.capabilities.keys())
            .slice(0, 2)
            .join(", ")})`
      );

    return {
      strengths,
      weaknesses,
      learningOpportunities,
      collaborationSuggestions,
    };
  }

  /**
   * Record learning event for analytics
   */
  private recordLearningEvent(event: LearningEvent): void {
    this.learningHistory.push(event);

    // Keep only recent history (last 1000 events)
    if (this.learningHistory.length > 1000) {
      this.learningHistory = this.learningHistory.slice(-1000);
    }
  }

  /**
   * Get agent by ID
   */
  getAgent(agentId: string): AgentProfile | undefined {
    return this.agents.get(agentId);
  }

  /**
   * Get all agents (optionally filtered by tenant)
   */
  getAgents(tenantId?: string): AgentProfile[] {
    const agents = Array.from(this.agents.values());
    return tenantId
      ? agents.filter((agent) => agent.tenantId === tenantId)
      : agents;
  }

  /**
   * Get learning analytics
   */
  getLearningAnalytics(tenantId?: string): {
    totalAgents: number;
    totalPatterns: number;
    averageCapabilityLevel: number;
    topCapabilities: Array<{ name: string; averageLevel: number }>;
  } {
    const agents = tenantId
      ? Array.from(this.agents.values()).filter((a) => a.tenantId === tenantId)
      : Array.from(this.agents.values());

    const allCapabilities = new Map<string, number[]>();

    agents.forEach((agent) => {
      agent.capabilities.forEach((cap, name) => {
        if (!allCapabilities.has(name)) {
          allCapabilities.set(name, []);
        }
        allCapabilities.get(name)!.push(cap.level);
      });
    });

    const topCapabilities = Array.from(allCapabilities.entries())
      .map(([name, levels]) => ({
        name,
        averageLevel:
          levels.reduce((sum, level) => sum + level, 0) / levels.length,
      }))
      .sort((a, b) => b.averageLevel - a.averageLevel)
      .slice(0, 5);

    const totalCapabilityLevels = topCapabilities.reduce(
      (sum, cap) => sum + cap.averageLevel,
      0
    );
    const averageCapabilityLevel =
      topCapabilities.length > 0
        ? totalCapabilityLevels / topCapabilities.length
        : 0;

    return {
      totalAgents: agents.length,
      totalPatterns: this.knowledgePatterns.size,
      averageCapabilityLevel,
      topCapabilities,
    };
  }
}
