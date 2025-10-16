/**
 * Debate Outcome Tracker for RL Training Integration
 *
 * @author @darianrosebrook
 * @module debate-outcome-tracker
 *
 * Tracks all arbitration session and debate outcomes for reinforcement learning training.
 * Bridges ARBITER-015 (Arbitration Protocol) and ARBITER-016 (Reasoning Engine)
 * with the RL training pipeline.
 */
// @ts-nocheck


import type { ArbitrationSession, Verdict } from "../types/arbitration";
import { DebateState, type DebateSession } from "../types/reasoning";
import type { PerformanceTracker } from "./PerformanceTracker";

/**
 * Debate outcome data collected for RL training.
 */
export interface DebateOutcome {
  /**
   * Unique outcome identifier.
   */
  id: string;

  /**
   * Arbitration session ID.
   */
  sessionId: string;

  /**
   * Debate state ID (if applicable).
   */
  debateId?: string;

  /**
   * Participating agents.
   */
  participants: string[];

  /**
   * Debate outcome type.
   */
  outcomeType: "consensus" | "arbitration" | "timeout" | "deadlock_resolved";

  /**
   * Final verdict (if reached).
   */
  verdict?: Verdict;

  /**
   * Debate metrics.
   */
  metrics: {
    /**
     * Total number of arguments presented.
     */
    argumentCount: number;

    /**
     * Number of turns taken.
     */
    turnCount: number;

    /**
     * Time to reach consensus/verdict (milliseconds).
     */
    resolutionTimeMs: number;

    /**
     * Evidence quality score (0-1).
     */
    evidenceQuality: number;

    /**
     * Reasoning coherence score (0-1).
     */
    reasoningCoherence: number;

    /**
     * Constitutional compliance score (0-1).
     */
    complianceScore: number;
  };

  /**
   * Turn-level data for RL training.
   */
  turns: DebateTurn[];

  /**
   * Final quality assessment.
   */
  qualityScore: number;

  /**
   * Timestamp of debate completion.
   */
  timestamp: string;
}

/**
 * Individual debate turn data for turn-level RL training.
 */
export interface DebateTurn {
  /**
   * Turn number in the debate.
   */
  turnNumber: number;

  /**
   * Agent who took this turn.
   */
  agentId: string;

  /**
   * Action taken (argument, evidence, vote, etc.).
   */
  action: {
    type: "argument" | "evidence" | "vote" | "challenge" | "concede";
    content: string;
    metadata?: Record<string, unknown>;
  };

  /**
   * State before the turn.
   */
  stateBefore: {
    argumentCount: number;
    evidenceCount: number;
    consensusLevel: number;
    remainingTurns: number;
  };

  /**
   * State after the turn.
   */
  stateAfter: {
    argumentCount: number;
    evidenceCount: number;
    consensusLevel: number;
    remainingTurns: number;
  };

  /**
   * Immediate reward for this turn.
   */
  reward: number;

  /**
   * Turn quality metrics.
   */
  metrics: {
    /**
     * Argument strength (0-1).
     */
    argumentStrength: number;

    /**
     * Evidence relevance (0-1).
     */
    evidenceRelevance: number;

    /**
     * Persuasiveness (0-1).
     */
    persuasiveness: number;

    /**
     * Constitutional alignment (0-1).
     */
    constitutionalAlignment: number;
  };

  /**
   * Timestamp of the turn.
   */
  timestamp: string;
}

/**
 * Configuration for debate outcome tracking.
 */
export interface DebateOutcomeTrackerConfig {
  /**
   * Whether to collect turn-level data.
   */
  collectTurnData: boolean;

  /**
   * Whether to enable quality scoring.
   */
  enableQualityScoring: boolean;

  /**
   * Maximum outcomes to keep in memory.
   */
  maxOutcomesInMemory: number;

  /**
   * Whether to export outcomes to performance tracker.
   */
  exportToPerformanceTracker: boolean;

  /**
   * Minimum debate length to track (turns).
   */
  minDebateLengthToTrack: number;

  /**
   * Quality score weights.
   */
  qualityWeights: {
    evidenceQuality: number;
    reasoningCoherence: number;
    complianceScore: number;
    resolutionEfficiency: number;
  };
}

/**
 * Default configuration.
 */
const DEFAULT_CONFIG: DebateOutcomeTrackerConfig = {
  collectTurnData: true,
  enableQualityScoring: true,
  maxOutcomesInMemory: 10000,
  exportToPerformanceTracker: true,
  minDebateLengthToTrack: 2,
  qualityWeights: {
    evidenceQuality: 0.3,
    reasoningCoherence: 0.3,
    complianceScore: 0.3,
    resolutionEfficiency: 0.1,
  },
};

/**
 * Debate Outcome Tracker for RL training integration.
 *
 * This component tracks all arbitration sessions and debates, extracting
 * training signals for reinforcement learning. It bridges ARBITER-015 and
 * ARBITER-016 with the RL training pipeline.
 */
export class DebateOutcomeTracker {
  private config: DebateOutcomeTrackerConfig;
  private outcomes: DebateOutcome[] = [];
  private isTracking: boolean = false;
  private performanceTracker?: PerformanceTracker;

  /**
   * Creates a new debate outcome tracker.
   *
   * @param config - Tracker configuration. Uses defaults if not provided.
   * @param performanceTracker - Optional performance tracker for data export.
   */
  constructor(
    config: Partial<DebateOutcomeTrackerConfig> = {},
    performanceTracker?: PerformanceTracker
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.performanceTracker = performanceTracker;
  }

  /**
   * Starts outcome tracking.
   */
  startTracking(): void {
    this.isTracking = true;
  }

  /**
   * Stops outcome tracking.
   */
  stopTracking(): void {
    this.isTracking = false;
  }

  /**
   * Records the outcome of an arbitration session.
   *
   * @param session - Completed arbitration session
   * @param debateSession - Final debate session (if debate occurred)
   * @returns Recorded debate outcome
   */
  async recordArbitrationOutcome(
    session: ArbitrationSession,
    debateSession?: DebateSession
  ): Promise<DebateOutcome> {
    if (!this.isTracking) {
      throw new Error("Outcome tracking is not active");
    }

    // Extract participants from session
    const participants = session.participants || [];

    // Calculate debate metrics
    const metrics = this.calculateDebateMetrics(session, debateSession);

    // Extract turn data if available and configured
    const turns =
      this.config.collectTurnData && debateSession
        ? await this.extractTurnData(debateSession, session)
        : [];

    // Calculate quality score
    const qualityScore = this.config.enableQualityScoring
      ? this.calculateQualityScore(metrics, turns)
      : 0.5;

    // Create outcome record
    const outcome: DebateOutcome = {
      id: `outcome_${session.id}_${Date.now()}`,
      sessionId: session.id,
      debateId: debateSession?.id,
      participants,
      outcomeType: this.determineOutcomeType(session, debateSession),
      verdict: session.verdict,
      metrics,
      turns,
      qualityScore,
      timestamp: new Date().toISOString(),
    };

    // Store outcome
    this.outcomes.push(outcome);
    this.cleanupOldOutcomes();

    // Export to performance tracker if configured
    if (this.config.exportToPerformanceTracker && this.performanceTracker) {
      await this.exportToPerformanceTracker(outcome);
    }

    return outcome;
  }

  /**
   * Records a debate outcome (without full arbitration session).
   *
   * @param debateSession - Completed debate session
   * @returns Recorded debate outcome
   */
  async recordDebateOutcome(
    debateSession: DebateSession
  ): Promise<DebateOutcome> {
    if (!this.isTracking) {
      throw new Error("Outcome tracking is not active");
    }

    // Create a minimal arbitration session representation
    const minimalSession: Partial<ArbitrationSession> = {
      id: debateSession.id,
      participants: debateSession.participants.map((p) => p.agentId),
    };

    return this.recordArbitrationOutcome(
      minimalSession as ArbitrationSession,
      debateSession
    );
  }

  /**
   * Exports collected outcomes for RL training.
   *
   * @param since - Optional timestamp to export outcomes since
   * @param minQualityScore - Minimum quality score to export
   * @returns Array of debate outcomes ready for training
   */
  exportOutcomes(since?: string, minQualityScore?: number): DebateOutcome[] {
    let outcomes = this.outcomes;

    // Filter by timestamp if provided
    if (since) {
      const sinceTime = new Date(since).getTime();
      outcomes = outcomes.filter(
        (outcome) => new Date(outcome.timestamp).getTime() >= sinceTime
      );
    }

    // Filter by quality score if provided
    if (minQualityScore !== undefined) {
      outcomes = outcomes.filter(
        (outcome) => outcome.qualityScore >= minQualityScore
      );
    }

    // Return a copy to prevent external modification
    return JSON.parse(JSON.stringify(outcomes));
  }

  /**
   * Gets statistics about tracked outcomes.
   *
   * @returns Tracking statistics
   */
  getStats(): {
    totalOutcomes: number;
    averageQualityScore: number;
    averageTurnCount: number;
    averageResolutionTimeMs: number;
    outcomeTypeDistribution: Record<string, number>;
    isTracking: boolean;
  } {
    const totalOutcomes = this.outcomes.length;

    if (totalOutcomes === 0) {
      return {
        totalOutcomes: 0,
        averageQualityScore: 0,
        averageTurnCount: 0,
        averageResolutionTimeMs: 0,
        outcomeTypeDistribution: {},
        isTracking: this.isTracking,
      };
    }

    const averageQualityScore =
      this.outcomes.reduce((sum, o) => sum + o.qualityScore, 0) / totalOutcomes;

    const averageTurnCount =
      this.outcomes.reduce((sum, o) => sum + o.metrics.turnCount, 0) /
      totalOutcomes;

    const averageResolutionTimeMs =
      this.outcomes.reduce((sum, o) => sum + o.metrics.resolutionTimeMs, 0) /
      totalOutcomes;

    const outcomeTypeDistribution = this.outcomes.reduce((dist, o) => {
      dist[o.outcomeType] = (dist[o.outcomeType] || 0) + 1;
      return dist;
    }, {} as Record<string, number>);

    return {
      totalOutcomes,
      averageQualityScore,
      averageTurnCount,
      averageResolutionTimeMs,
      outcomeTypeDistribution,
      isTracking: this.isTracking,
    };
  }

  /**
   * Clears all tracked outcomes.
   */
  clearOutcomes(): void {
    this.outcomes = [];
  }

  /**
   * Gets current configuration.
   *
   * @returns Current configuration
   */
  getConfig(): DebateOutcomeTrackerConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<DebateOutcomeTrackerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Calculates debate metrics from session and debate session.
   */
  private calculateDebateMetrics(
    session: ArbitrationSession,
    debateSession?: DebateSession
  ): DebateOutcome["metrics"] {
    const argumentCount = debateSession?.arguments?.length || 0;
    const turnCount = argumentCount; // Use argument count as proxy for turns
    const resolutionTimeMs = session.metadata?.totalDurationMs || 0;

    // Calculate evidence quality (average credibility of evidence)
    const evidenceQuality = debateSession?.arguments
      ? debateSession.arguments.reduce(
          (sum, a) => sum + (a.credibilityScore || 0.5),
          0
        ) / debateSession.arguments.length
      : 0.5;

    // Calculate reasoning coherence (based on argument quality)
    const reasoningCoherence = debateSession?.arguments
      ? debateSession.arguments.reduce(
          (sum, a) => sum + (a.credibilityScore || 0.5),
          0
        ) / debateSession.arguments.length
      : 0.5;

    // Calculate compliance score (from session verdict if available)
    const complianceScore = session.verdict?.confidence || 0.5;

    return {
      argumentCount,
      turnCount,
      resolutionTimeMs,
      evidenceQuality,
      reasoningCoherence,
      complianceScore,
    };
  }

  /**
   * Extracts turn-level data from debate session.
   */
  private async extractTurnData(
    debateSession: DebateSession,
    _session: ArbitrationSession
  ): Promise<DebateTurn[]> {
    const turns: DebateTurn[] = [];

    // Skip if debate is too short
    if (
      !debateSession.arguments ||
      debateSession.arguments.length < this.config.minDebateLengthToTrack
    ) {
      return turns;
    }

    // Extract turn data from arguments
    for (let i = 0; i < debateSession.arguments.length; i++) {
      const argument = debateSession.arguments[i];

      const turn: DebateTurn = {
        turnNumber: i + 1,
        agentId: argument.agentId,
        action: {
          type: "argument",
          content: argument.claim,
          metadata: {
            reasoning: argument.reasoning,
            evidenceCount: argument.evidence?.length || 0,
          },
        },
        stateBefore: {
          argumentCount: i,
          evidenceCount: debateSession.arguments
            .slice(0, i)
            .reduce((sum, a) => sum + (a.evidence?.length || 0), 0),
          consensusLevel: 0, // Would calculate from consensus result
          remainingTurns: debateSession.config.maxParticipants - i,
        },
        stateAfter: {
          argumentCount: i + 1,
          evidenceCount: debateSession.arguments
            .slice(0, i + 1)
            .reduce((sum, a) => sum + (a.evidence?.length || 0), 0),
          consensusLevel: 0, // Would calculate from consensus result
          remainingTurns: debateSession.config.maxParticipants - (i + 1),
        },
        reward: this.calculateTurnReward(argument, debateSession),
        metrics: {
          argumentStrength: argument.credibilityScore || 0.5,
          evidenceRelevance: argument.evidence?.length > 0 ? 0.8 : 0.3,
          persuasiveness: argument.credibilityScore || 0.5,
          constitutionalAlignment: 0.7, // Would check against rules
        },
        timestamp: argument.timestamp.toISOString(),
      };

      turns.push(turn);
    }

    return turns;
  }

  /**
   * Calculates reward for a single turn.
   */
  private calculateTurnReward(
    argument: any,
    _debateSession: DebateSession
  ): number {
    // Reward based on argument quality and impact
    const argumentQuality = argument.credibilityScore || 0.5;
    const evidenceBonus = argument.evidence?.length > 0 ? 0.2 : 0;
    const baseReward = argumentQuality + evidenceBonus;

    return Math.min(1.0, baseReward);
  }

  /**
   * Calculates overall quality score for the debate outcome.
   */
  private calculateQualityScore(
    metrics: DebateOutcome["metrics"],
    turns: DebateTurn[]
  ): number {
    const weights = this.config.qualityWeights;

    // Base score from metrics
    const baseScore =
      metrics.evidenceQuality * weights.evidenceQuality +
      metrics.reasoningCoherence * weights.reasoningCoherence +
      metrics.complianceScore * weights.complianceScore;

    // Resolution efficiency bonus (faster resolution = better)
    const targetResolutionTime = 60000; // 1 minute target
    const efficiencyScore = Math.max(
      0,
      1 - metrics.resolutionTimeMs / (targetResolutionTime * 5)
    );
    const efficiencyComponent = efficiencyScore * weights.resolutionEfficiency;

    // Turn quality bonus (if turn data available)
    let turnQualityBonus = 0;
    if (turns.length > 0) {
      const averageTurnReward =
        turns.reduce((sum, t) => sum + t.reward, 0) / turns.length;
      turnQualityBonus = averageTurnReward * 0.1; // 10% weight
    }

    return Math.min(1.0, baseScore + efficiencyComponent + turnQualityBonus);
  }

  /**
   * Determines the outcome type of the debate.
   */
  private determineOutcomeType(
    session: ArbitrationSession,
    debateSession?: DebateSession
  ): DebateOutcome["outcomeType"] {
    // Check for consensus
    if (debateSession?.consensusResult?.reached) {
      return "consensus";
    }

    // Check for timeout (debate ended without conclusion)
    if (
      debateSession &&
      !debateSession.consensusResult?.reached &&
      debateSession.endTime
    ) {
      return "timeout";
    }

    // Check if it was resolved through deadlock handling
    if (
      debateSession?.state === DebateState.CONSENSUS_REACHED ||
      debateSession?.state === DebateState.COMPLETED
    ) {
      return debateSession.consensusResult?.reached
        ? "consensus"
        : "deadlock_resolved";
    }

    // Default to arbitration
    return "arbitration";
  }

  /**
   * Exports outcome data to the performance tracker.
   */
  private async exportToPerformanceTracker(
    outcome: DebateOutcome
  ): Promise<void> {
    if (!this.performanceTracker) {
      return;
    }

    try {
      // Record debate completion as an evaluation outcome
      await this.performanceTracker.recordEvaluationOutcome(outcome.sessionId, {
        passed: outcome.verdict?.outcome === "approved",
        score: outcome.qualityScore,
        rubricScores: {
          evidenceQuality: outcome.metrics.evidenceQuality,
          reasoningCoherence: outcome.metrics.reasoningCoherence,
          complianceScore: outcome.metrics.complianceScore,
        },
        feedback: `Debate resolved via ${outcome.outcomeType}`,
      });

      // Record turn-level data for RL training
      for (const turn of outcome.turns) {
        await this.performanceTracker.recordEvent({
          type: "debate-turn",
          timestamp: turn.timestamp,
          data: {
            outcomeId: outcome.id,
            sessionId: outcome.sessionId,
            turnNumber: turn.turnNumber,
            agentId: turn.agentId,
            action: turn.action,
            reward: turn.reward,
            metrics: turn.metrics,
          },
        });
      }
    } catch (error) {
      console.warn("Failed to export outcome to performance tracker:", error);
    }
  }

  /**
   * Removes old outcomes based on memory limits.
   */
  private cleanupOldOutcomes(): void {
    if (this.outcomes.length > this.config.maxOutcomesInMemory) {
      // Keep most recent outcomes
      this.outcomes = this.outcomes
        .sort(
          (a, b) =>
            new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
        )
        .slice(0, this.config.maxOutcomesInMemory);
    }
  }
}
