/**
 * TurnManager
 *
 * Manages turn scheduling and fairness enforcement in multi-agent debates.
 * Ensures each participant gets equal opportunity to contribute while
 * preventing monopolization and handling timeouts.
 *
 * @module reasoning/TurnManager
 * @author @darianrosebrook
 */

import {
  AgentRole,
  DebateParticipant,
  ReasoningEngineError,
} from "@/types/reasoning";

/**
 * Turn scheduling strategy
 */
export enum TurnSchedulingStrategy {
  ROUND_ROBIN = "round_robin",
  WEIGHTED_FAIR = "weighted_fair",
  PRIORITY_BASED = "priority_based",
  DYNAMIC_ADAPTIVE = "dynamic_adaptive",
}

/**
 * Turn record for tracking participation
 */
export interface TurnRecord {
  agentId: string;
  turnNumber: number;
  timestamp: Date;
  duration: number;
  action: "argument" | "vote" | "evidence" | "rebuttal";
  wasTimeout: boolean;
}

/**
 * Turn allocation result
 */
export interface TurnAllocation {
  agentId: string;
  turnNumber: number;
  maxDuration: number;
  deadline: Date;
  priority: number;
  reason: string;
}

/**
 * Fairness metrics for a debate
 */
export interface FairnessMetrics {
  totalTurns: number;
  turnsPerAgent: Map<string, number>;
  averageTurnsPerAgent: number;
  fairnessScore: number; // 0-1, where 1 is perfectly fair
  participationRate: Map<string, number>; // Percentage of total turns
  timeoutsPerAgent: Map<string, number>;
}

/**
 * TurnManager configuration
 */
export interface TurnManagerConfig {
  schedulingStrategy: TurnSchedulingStrategy;
  defaultTurnTimeout: number; // milliseconds
  maxTurnsPerAgent: number;
  fairnessThreshold: number; // 0-1, minimum acceptable fairness
  enableTimeoutPenalty: boolean;
  timeoutPenaltyMultiplier: number;
}

/**
 * Manages turn scheduling and fairness in debates
 */
export class TurnManager {
  private turnHistory: Map<string, TurnRecord[]>;
  private currentTurn: Map<string, TurnAllocation | null>;
  private config: TurnManagerConfig;

  constructor(config: Partial<TurnManagerConfig> = {}) {
    this.turnHistory = new Map();
    this.currentTurn = new Map();

    this.config = {
      schedulingStrategy:
        config.schedulingStrategy ?? TurnSchedulingStrategy.WEIGHTED_FAIR,
      defaultTurnTimeout: config.defaultTurnTimeout ?? 60000, // 1 minute
      maxTurnsPerAgent: config.maxTurnsPerAgent ?? 10,
      fairnessThreshold: config.fairnessThreshold ?? 0.7,
      enableTimeoutPenalty: config.enableTimeoutPenalty ?? true,
      timeoutPenaltyMultiplier: config.timeoutPenaltyMultiplier ?? 0.5,
    };
  }

  /**
   * Initializes turn tracking for a debate
   */
  public initializeDebate(debateId: string): void {
    if (!debateId || debateId.trim().length === 0) {
      throw new ReasoningEngineError(
        "Debate ID cannot be empty",
        "INVALID_DEBATE_ID"
      );
    }

    this.turnHistory.set(debateId, []);
    this.currentTurn.set(debateId, null);
  }

  /**
   * Allocates the next turn to an agent
   */
  public allocateNextTurn(
    debateId: string,
    participants: DebateParticipant[]
  ): TurnAllocation {
    if (!this.turnHistory.has(debateId)) {
      throw new ReasoningEngineError(
        `Debate ${debateId} not initialized`,
        "DEBATE_NOT_INITIALIZED"
      );
    }

    if (!participants || participants.length === 0) {
      throw new ReasoningEngineError(
        "At least one participant required",
        "NO_PARTICIPANTS"
      );
    }

    // Get current turn history
    const history = this.turnHistory.get(debateId)!;

    // Check if any agent has exceeded max turns
    const turnCounts = this.countTurnsPerAgent(history);
    const availableParticipants = participants.filter(
      (p) => (turnCounts.get(p.agentId) ?? 0) < this.config.maxTurnsPerAgent
    );

    if (availableParticipants.length === 0) {
      throw new ReasoningEngineError(
        "All agents have reached maximum turns",
        "MAX_TURNS_REACHED"
      );
    }

    // Allocate turn based on strategy
    const allocation = this.allocateTurnByStrategy(
      availableParticipants,
      history
    );

    // Store current turn
    this.currentTurn.set(debateId, allocation);

    return allocation;
  }

  /**
   * Records a completed turn
   */
  public recordTurn(
    debateId: string,
    agentId: string,
    action: "argument" | "vote" | "evidence" | "rebuttal",
    duration: number,
    wasTimeout: boolean = false
  ): void {
    if (!this.turnHistory.has(debateId)) {
      throw new ReasoningEngineError(
        `Debate ${debateId} not initialized`,
        "DEBATE_NOT_INITIALIZED"
      );
    }

    const history = this.turnHistory.get(debateId)!;
    const turnNumber = history.length + 1;

    const record: TurnRecord = {
      agentId,
      turnNumber,
      timestamp: new Date(),
      duration,
      action,
      wasTimeout,
    };

    history.push(record);
    this.currentTurn.set(debateId, null);
  }

  /**
   * Checks if current turn has timed out
   */
  public isCurrentTurnTimedOut(debateId: string): boolean {
    const allocation = this.currentTurn.get(debateId);

    if (!allocation) {
      return false;
    }

    return new Date() > allocation.deadline;
  }

  /**
   * Gets current turn allocation
   */
  public getCurrentTurn(debateId: string): TurnAllocation | null {
    return this.currentTurn.get(debateId) ?? null;
  }

  /**
   * Calculates fairness metrics for a debate
   */
  public calculateFairnessMetrics(debateId: string): FairnessMetrics {
    if (!this.turnHistory.has(debateId)) {
      throw new ReasoningEngineError(
        `Debate ${debateId} not initialized`,
        "DEBATE_NOT_INITIALIZED"
      );
    }

    const history = this.turnHistory.get(debateId)!;
    const turnsPerAgent = this.countTurnsPerAgent(history);
    const timeoutsPerAgent = this.countTimeoutsPerAgent(history);

    const totalTurns = history.length;
    const agentCount = turnsPerAgent.size;
    const averageTurnsPerAgent = agentCount > 0 ? totalTurns / agentCount : 0;

    // Calculate fairness score (Gini coefficient-like measure)
    const fairnessScore = this.calculateFairnessScore(
      turnsPerAgent,
      averageTurnsPerAgent
    );

    // Calculate participation rates
    const participationRate = new Map<string, number>();
    for (const [agentId, turns] of turnsPerAgent) {
      participationRate.set(agentId, totalTurns > 0 ? turns / totalTurns : 0);
    }

    return {
      totalTurns,
      turnsPerAgent,
      averageTurnsPerAgent,
      fairnessScore,
      participationRate,
      timeoutsPerAgent,
    };
  }

  /**
   * Validates fairness of debate
   */
  public validateFairness(debateId: string): {
    isValid: boolean;
    issues: string[];
  } {
    const metrics = this.calculateFairnessMetrics(debateId);
    const issues: string[] = [];

    // Check fairness threshold
    if (metrics.fairnessScore < this.config.fairnessThreshold) {
      issues.push(
        `Fairness score ${metrics.fairnessScore.toFixed(2)} below threshold ${
          this.config.fairnessThreshold
        }`
      );
    }

    // Check for agent monopolization (>50% of turns)
    for (const [agentId, rate] of metrics.participationRate) {
      if (rate > 0.5) {
        issues.push(
          `Agent ${agentId} monopolized debate with ${(rate * 100).toFixed(
            1
          )}% of turns`
        );
      }
    }

    // Check for agents with zero participation
    for (const [agentId, turns] of metrics.turnsPerAgent) {
      if (turns === 0) {
        issues.push(`Agent ${agentId} had zero participation`);
      }
    }

    // Check for excessive timeouts
    for (const [agentId, timeouts] of metrics.timeoutsPerAgent) {
      const totalTurns = metrics.turnsPerAgent.get(agentId) ?? 0;
      if (totalTurns > 0 && timeouts / totalTurns > 0.5) {
        issues.push(
          `Agent ${agentId} had ${timeouts}/${totalTurns} timeouts (>50%)`
        );
      }
    }

    return {
      isValid: issues.length === 0,
      issues,
    };
  }

  /**
   * Gets turn history for a debate
   */
  public getTurnHistory(debateId: string): TurnRecord[] {
    return this.turnHistory.get(debateId) ?? [];
  }

  /**
   * Clears turn history for a debate
   */
  public clearDebate(debateId: string): void {
    this.turnHistory.delete(debateId);
    this.currentTurn.delete(debateId);
  }

  /**
   * Allocates turn based on scheduling strategy
   */
  private allocateTurnByStrategy(
    participants: DebateParticipant[],
    history: TurnRecord[]
  ): TurnAllocation {
    switch (this.config.schedulingStrategy) {
      case TurnSchedulingStrategy.ROUND_ROBIN:
        return this.allocateRoundRobin(participants, history);
      case TurnSchedulingStrategy.WEIGHTED_FAIR:
        return this.allocateWeightedFair(participants, history);
      case TurnSchedulingStrategy.PRIORITY_BASED:
        return this.allocatePriorityBased(participants, history);
      case TurnSchedulingStrategy.DYNAMIC_ADAPTIVE:
        return this.allocateDynamicAdaptive(participants, history);
      default:
        throw new ReasoningEngineError(
          `Unknown scheduling strategy: ${this.config.schedulingStrategy}`,
          "UNKNOWN_STRATEGY"
        );
    }
  }

  /**
   * Round-robin turn allocation
   */
  private allocateRoundRobin(
    participants: DebateParticipant[],
    history: TurnRecord[]
  ): TurnAllocation {
    const turnCounts = this.countTurnsPerAgent(history);

    // Find agent with fewest turns
    let selectedAgent = participants[0];
    let minTurns = turnCounts.get(selectedAgent.agentId) ?? 0;

    for (const participant of participants) {
      const turns = turnCounts.get(participant.agentId) ?? 0;
      if (turns < minTurns) {
        minTurns = turns;
        selectedAgent = participant;
      }
    }

    const turnNumber = history.length + 1;
    const deadline = new Date(Date.now() + this.config.defaultTurnTimeout);

    return {
      agentId: selectedAgent.agentId,
      turnNumber,
      maxDuration: this.config.defaultTurnTimeout,
      deadline,
      priority: 1.0,
      reason: `Round-robin (${minTurns} previous turns)`,
    };
  }

  /**
   * Weighted fair turn allocation
   */
  private allocateWeightedFair(
    participants: DebateParticipant[],
    history: TurnRecord[]
  ): TurnAllocation {
    const turnCounts = this.countTurnsPerAgent(history);
    const timeoutCounts = this.countTimeoutsPerAgent(history);

    // Calculate fairness scores for each agent
    const scores = participants.map((p) => {
      const turns = turnCounts.get(p.agentId) ?? 0;
      const timeouts = timeoutCounts.get(p.agentId) ?? 0;

      // Base score: inverse of turn count
      let score = 1 / (turns + 1);

      // Apply weight multiplier
      score *= p.weight ?? 1.0;

      // Apply timeout penalty if enabled
      if (this.config.enableTimeoutPenalty && timeouts > 0) {
        score *= Math.pow(this.config.timeoutPenaltyMultiplier, timeouts);
      }

      return { participant: p, score };
    });

    // Select agent with highest score
    scores.sort((a, b) => b.score - a.score);
    const selected = scores[0];

    const turnNumber = history.length + 1;
    const deadline = new Date(Date.now() + this.config.defaultTurnTimeout);

    return {
      agentId: selected.participant.agentId,
      turnNumber,
      maxDuration: this.config.defaultTurnTimeout,
      deadline,
      priority: selected.score,
      reason: `Weighted fair (score: ${selected.score.toFixed(2)})`,
    };
  }

  /**
   * Priority-based turn allocation
   */
  private allocatePriorityBased(
    participants: DebateParticipant[],
    history: TurnRecord[]
  ): TurnAllocation {
    // Sort by role priority: MEDIATOR > PROPONENT > OPPONENT > OBSERVER
    const rolePriority: Record<AgentRole, number> = {
      [AgentRole.MEDIATOR]: 4,
      [AgentRole.PROPONENT]: 3,
      [AgentRole.OPPONENT]: 2,
      [AgentRole.OBSERVER]: 1,
    };

    const turnCounts = this.countTurnsPerAgent(history);

    // Calculate composite priority
    const priorities = participants.map((p) => {
      const turns = turnCounts.get(p.agentId) ?? 0;
      const rolePrio = rolePriority[p.role] ?? 1;

      // Higher role priority, but penalize for more turns
      const priority = rolePrio / (turns + 1);

      return { participant: p, priority };
    });

    // Select highest priority
    priorities.sort((a, b) => b.priority - a.priority);
    const selected = priorities[0];

    const turnNumber = history.length + 1;
    const deadline = new Date(Date.now() + this.config.defaultTurnTimeout);

    return {
      agentId: selected.participant.agentId,
      turnNumber,
      maxDuration: this.config.defaultTurnTimeout,
      deadline,
      priority: selected.priority,
      reason: `Priority-based (${
        selected.participant.role
      }, priority: ${selected.priority.toFixed(2)})`,
    };
  }

  /**
   * Dynamic adaptive turn allocation
   */
  private allocateDynamicAdaptive(
    participants: DebateParticipant[],
    history: TurnRecord[]
  ): TurnAllocation {
    const turnCounts = this.countTurnsPerAgent(history);
    const timeoutCounts = this.countTimeoutsPerAgent(history);

    // Calculate adaptive scores combining multiple factors
    const scores = participants.map((p) => {
      const turns = turnCounts.get(p.agentId) ?? 0;
      const timeouts = timeoutCounts.get(p.agentId) ?? 0;

      // Factor 1: Fairness (inverse of turns)
      const fairnessScore = 1 / (turns + 1);

      // Factor 2: Weight
      const weightScore = p.weight ?? 1.0;

      // Factor 3: Recent activity (last 3 turns)
      const recentTurns = history
        .slice(-3)
        .filter((t) => t.agentId === p.agentId).length;
      const recencyPenalty = 1 / (recentTurns + 1);

      // Factor 4: Timeout penalty
      const timeoutPenalty =
        this.config.enableTimeoutPenalty && timeouts > 0
          ? Math.pow(this.config.timeoutPenaltyMultiplier, timeouts)
          : 1.0;

      // Combine factors with weights
      const compositeScore =
        fairnessScore * 0.4 +
        weightScore * 0.3 +
        recencyPenalty * 0.2 * timeoutPenalty * 0.1;

      return { participant: p, score: compositeScore };
    });

    // Select highest score
    scores.sort((a, b) => b.score - a.score);
    const selected = scores[0];

    const turnNumber = history.length + 1;
    const deadline = new Date(Date.now() + this.config.defaultTurnTimeout);

    return {
      agentId: selected.participant.agentId,
      turnNumber,
      maxDuration: this.config.defaultTurnTimeout,
      deadline,
      priority: selected.score,
      reason: `Dynamic adaptive (score: ${selected.score.toFixed(2)})`,
    };
  }

  /**
   * Counts turns per agent
   */
  private countTurnsPerAgent(history: TurnRecord[]): Map<string, number> {
    const counts = new Map<string, number>();

    for (const record of history) {
      const current = counts.get(record.agentId) ?? 0;
      counts.set(record.agentId, current + 1);
    }

    return counts;
  }

  /**
   * Counts timeouts per agent
   */
  private countTimeoutsPerAgent(history: TurnRecord[]): Map<string, number> {
    const counts = new Map<string, number>();

    for (const record of history) {
      if (record.wasTimeout) {
        const current = counts.get(record.agentId) ?? 0;
        counts.set(record.agentId, current + 1);
      }
    }

    return counts;
  }

  /**
   * Calculates fairness score (1 - Gini coefficient)
   */
  private calculateFairnessScore(
    turnsPerAgent: Map<string, number>,
    average: number
  ): number {
    if (turnsPerAgent.size === 0 || average === 0) {
      return 1.0; // Perfect fairness if no turns
    }

    const turnCounts = Array.from(turnsPerAgent.values());

    // Calculate sum of absolute differences from average
    let sumDifferences = 0;
    for (const count of turnCounts) {
      sumDifferences += Math.abs(count - average);
    }

    // Normalize by theoretical maximum
    const maxPossibleDifference = 2 * average * turnCounts.length;

    if (maxPossibleDifference === 0) {
      return 1.0;
    }

    // Calculate fairness (1 - normalized Gini)
    const gini = sumDifferences / maxPossibleDifference;
    return 1 - gini;
  }
}
