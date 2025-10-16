/**
 * DeadlockResolver
 *
 * Detects and resolves deadlocks in multi-agent debates.
 * Implements multiple resolution strategies and tracks deadlock patterns.
 *
 * @module reasoning/DeadlockResolver
 * @author @darianrosebrook
 */

import {
  AgentRole,
  DeadlockResolutionStrategy,
  DebateSession,
  DebateVote,
  ReasoningEngineError,
} from "@/types/reasoning";

/**
 * Deadlock detection result
 */
export interface DeadlockDetection {
  isDeadlocked: boolean;
  rounds: number;
  votingPattern: string;
  participantsInvolved: string[];
  confidence: number; // 0-1
  suggestedResolution: DeadlockResolutionStrategy;
}

/**
 * Deadlock resolution result
 */
export interface DeadlockResolution {
  strategy: DeadlockResolutionStrategy;
  decision: "for" | "against" | "split" | "escalated";
  reason: string;
  confidence: number;
  mediatorOverride: boolean;
}

/**
 * Deadlock pattern for tracking recurring deadlocks
 */
export interface DeadlockPattern {
  pattern: string;
  occurrences: number;
  lastSeen: Date;
  participantIds: string[];
  avgResolutionTime: number;
}

/**
 * DeadlockResolver configuration
 */
export interface DeadlockResolverConfig {
  deadlockDetectionRounds: number;
  minVotesForDeadlock: number;
  votingPatternThreshold: number; // 0-1, similarity threshold
  enablePatternTracking: boolean;
  defaultResolutionStrategy: DeadlockResolutionStrategy;
}

/**
 * Resolves deadlocks in multi-agent debates
 */
export class DeadlockResolver {
  private deadlockPatterns: Map<string, DeadlockPattern>;
  private config: DeadlockResolverConfig;

  constructor(config: Partial<DeadlockResolverConfig> = {}) {
    this.deadlockPatterns = new Map();

    this.config = {
      deadlockDetectionRounds: config.deadlockDetectionRounds ?? 3,
      minVotesForDeadlock: config.minVotesForDeadlock ?? 2,
      votingPatternThreshold: config.votingPatternThreshold ?? 0.8,
      enablePatternTracking: config.enablePatternTracking ?? true,
      defaultResolutionStrategy:
        config.defaultResolutionStrategy ??
        DeadlockResolutionStrategy.MEDIATOR_DECISION,
    };
  }

  /**
   * Detects if debate is in deadlock
   */
  public detectDeadlock(
    session: DebateSession,
    recentVotes: DebateVote[]
  ): DeadlockDetection {
    if (!recentVotes || recentVotes.length < this.config.minVotesForDeadlock) {
      return {
        isDeadlocked: false,
        rounds: 0,
        votingPattern: "insufficient_data",
        participantsInvolved: [],
        confidence: 0,
        suggestedResolution: this.config.defaultResolutionStrategy,
      };
    }

    // Check for voting pattern repetition
    const patterns = this.extractVotingPatterns(recentVotes);
    const isRepeating = this.isPatternRepeating(
      patterns,
      this.config.deadlockDetectionRounds
    );

    if (!isRepeating) {
      return {
        isDeadlocked: false,
        rounds: 0,
        votingPattern: "progressing",
        participantsInvolved: [],
        confidence: 0,
        suggestedResolution: this.config.defaultResolutionStrategy,
      };
    }

    // Analyze voting split
    const split = this.analyzeVotingSplit(recentVotes);

    // Check if split is irreconcilable
    const isIrreconcilable =
      Math.abs(split.forRatio - 0.5) < 0.1 && // Close to 50/50
      patterns.length >= this.config.deadlockDetectionRounds;

    if (!isIrreconcilable) {
      return {
        isDeadlocked: false,
        rounds: patterns.length,
        votingPattern: "converging",
        participantsInvolved: [],
        confidence: 0.5,
        suggestedResolution: this.config.defaultResolutionStrategy,
      };
    }

    // Deadlock detected
    const participantsInvolved = recentVotes.map((v) => v.agentId);
    const suggestedResolution = this.suggestResolutionStrategy(session, split);

    // Track pattern if enabled
    if (this.config.enablePatternTracking) {
      this.trackPattern(patterns, participantsInvolved);
    }

    return {
      isDeadlocked: true,
      rounds: patterns.length,
      votingPattern: patterns.join(","),
      participantsInvolved: Array.from(new Set(participantsInvolved)),
      confidence: 0.9,
      suggestedResolution,
    };
  }

  /**
   * Resolves a detected deadlock
   */
  public resolveDeadlock(
    session: DebateSession,
    detection: DeadlockDetection,
    strategy?: DeadlockResolutionStrategy
  ): DeadlockResolution {
    const resolutionStrategy =
      strategy ??
      detection.suggestedResolution ??
      this.config.defaultResolutionStrategy;

    switch (resolutionStrategy) {
      case DeadlockResolutionStrategy.MEDIATOR_DECISION:
        return this.resolveMediatorDecision(session);
      case DeadlockResolutionStrategy.TIMEOUT_DEFAULT:
        return this.resolveTimeoutDefault(session);
      case DeadlockResolutionStrategy.WEIGHTED_COMPROMISE:
        return this.resolveWeightedCompromise(session);
      case DeadlockResolutionStrategy.ESCALATE_TO_ADMIN:
        return this.resolveEscalateToAdmin(session);
      case DeadlockResolutionStrategy.SPLIT_DECISION:
        return this.resolveSplitDecision(session);
      default:
        throw new ReasoningEngineError(
          `Unknown resolution strategy: ${resolutionStrategy}`,
          "UNKNOWN_STRATEGY"
        );
    }
  }

  /**
   * Gets deadlock patterns for analysis
   */
  public getDeadlockPatterns(): DeadlockPattern[] {
    return Array.from(this.deadlockPatterns.values());
  }

  /**
   * Clears deadlock pattern tracking
   */
  public clearPatterns(): void {
    this.deadlockPatterns.clear();
  }

  /**
   * Extracts voting patterns from recent votes
   */
  private extractVotingPatterns(votes: DebateVote[]): string[] {
    const patterns: string[] = [];
    const roundSize = votes.length / this.config.deadlockDetectionRounds;

    for (let i = 0; i < this.config.deadlockDetectionRounds; i++) {
      const roundStart = Math.floor(i * roundSize);
      const roundEnd = Math.floor((i + 1) * roundSize);
      const roundVotes = votes.slice(roundStart, roundEnd);

      // Create pattern signature: "for:against:abstain"
      const forCount = roundVotes.filter((v) => v.position === "for").length;
      const againstCount = roundVotes.filter(
        (v) => v.position === "against"
      ).length;
      const abstainCount = roundVotes.filter(
        (v) => v.position === "abstain"
      ).length;

      patterns.push(`${forCount}:${againstCount}:${abstainCount}`);
    }

    return patterns;
  }

  /**
   * Checks if voting pattern is repeating
   */
  private isPatternRepeating(
    patterns: string[],
    minRepetitions: number
  ): boolean {
    if (patterns.length < minRepetitions) {
      return false;
    }

    // Check if all patterns are identical or very similar
    const firstPattern = patterns[0];
    let matchCount = 0;

    for (const pattern of patterns) {
      if (pattern === firstPattern) {
        matchCount++;
      }
    }

    return matchCount >= minRepetitions;
  }

  /**
   * Analyzes voting split
   */
  private analyzeVotingSplit(votes: DebateVote[]): {
    forVotes: number;
    againstVotes: number;
    abstainVotes: number;
    forRatio: number;
  } {
    const forVotes = votes.filter((v) => v.position === "for").length;
    const againstVotes = votes.filter((v) => v.position === "against").length;
    const abstainVotes = votes.filter((v) => v.position === "abstain").length;

    const totalDecisive = forVotes + againstVotes;
    const forRatio = totalDecisive > 0 ? forVotes / totalDecisive : 0.5;

    return {
      forVotes,
      againstVotes,
      abstainVotes,
      forRatio,
    };
  }

  /**
   * Suggests resolution strategy based on session state
   */
  private suggestResolutionStrategy(
    session: DebateSession,
    split: { forRatio: number }
  ): DeadlockResolutionStrategy {
    // Check if mediator is present
    const hasMediator = session.participants.some(
      (p) => p.role === AgentRole.MEDIATOR
    );

    if (hasMediator) {
      return DeadlockResolutionStrategy.MEDIATOR_DECISION;
    }

    // Check if split is close to 50/50
    if (Math.abs(split.forRatio - 0.5) < 0.05) {
      return DeadlockResolutionStrategy.SPLIT_DECISION;
    }

    // Use weighted compromise if split is not extreme
    if (split.forRatio > 0.3 && split.forRatio < 0.7) {
      return DeadlockResolutionStrategy.WEIGHTED_COMPROMISE;
    }

    // Default to timeout
    return DeadlockResolutionStrategy.TIMEOUT_DEFAULT;
  }

  /**
   * Tracks deadlock pattern
   */
  private trackPattern(patterns: string[], participantIds: string[]): void {
    const patternKey = patterns.join("|");

    const existing = this.deadlockPatterns.get(patternKey);

    if (existing) {
      existing.occurrences++;
      existing.lastSeen = new Date();
    } else {
      this.deadlockPatterns.set(patternKey, {
        pattern: patternKey,
        occurrences: 1,
        lastSeen: new Date(),
        participantIds: Array.from(new Set(participantIds)),
        avgResolutionTime: 0,
      });
    }
  }

  /**
   * Resolves via mediator decision
   */
  private resolveMediatorDecision(session: DebateSession): DeadlockResolution {
    const mediator = session.participants.find(
      (p) => p.role === AgentRole.MEDIATOR
    );

    if (!mediator) {
      throw new ReasoningEngineError(
        "No mediator available for resolution",
        "NO_MEDIATOR"
      );
    }

    // Get mediator's vote if they voted
    const mediatorVote = mediator.votesCast[mediator.votesCast.length - 1];

    if (!mediatorVote) {
      // Mediator hasn't voted, default to "for" with low confidence
      return {
        strategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
        decision: "for",
        reason: "Mediator resolution: Default to approval",
        confidence: 0.5,
        mediatorOverride: true,
      };
    }

    return {
      strategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
      decision: mediatorVote.position as "for" | "against",
      reason: `Mediator resolution: ${mediatorVote.reasoning}`,
      confidence: mediatorVote.confidence,
      mediatorOverride: true,
    };
  }

  /**
   * Resolves via timeout default
   */
  private resolveTimeoutDefault(_session: DebateSession): DeadlockResolution {
    // Default to "against" on timeout (conservative approach)
    return {
      strategy: DeadlockResolutionStrategy.TIMEOUT_DEFAULT,
      decision: "against",
      reason: "Timeout resolution: Default to rejection (conservative)",
      confidence: 0.6,
      mediatorOverride: false,
    };
  }

  /**
   * Resolves via weighted compromise
   */
  private resolveWeightedCompromise(
    session: DebateSession
  ): DeadlockResolution {
    // Calculate weighted votes
    let weightedFor = 0;
    let weightedAgainst = 0;
    let totalWeight = 0;

    for (const participant of session.participants) {
      const weight = participant.weight ?? 1.0;
      totalWeight += weight;

      const latestVote =
        participant.votesCast[participant.votesCast.length - 1];
      if (latestVote) {
        if (latestVote.position === "for") {
          weightedFor += weight * latestVote.confidence;
        } else if (latestVote.position === "against") {
          weightedAgainst += weight * latestVote.confidence;
        }
      }
    }

    const decision = weightedFor > weightedAgainst ? "for" : "against";
    const confidence = Math.abs(weightedFor - weightedAgainst) / totalWeight;

    return {
      strategy: DeadlockResolutionStrategy.WEIGHTED_COMPROMISE,
      decision,
      reason: `Weighted compromise: ${decision} (weighted score: ${(
        weightedFor - weightedAgainst
      ).toFixed(2)})`,
      confidence,
      mediatorOverride: false,
    };
  }

  /**
   * Resolves via escalation
   */
  private resolveEscalateToAdmin(_session: DebateSession): DeadlockResolution {
    return {
      strategy: DeadlockResolutionStrategy.ESCALATE_TO_ADMIN,
      decision: "escalated",
      reason: "Escalated to administrator for manual resolution",
      confidence: 1.0,
      mediatorOverride: false,
    };
  }

  /**
   * Resolves via split decision
   */
  private resolveSplitDecision(_session: DebateSession): DeadlockResolution {
    return {
      strategy: DeadlockResolutionStrategy.SPLIT_DECISION,
      decision: "split",
      reason:
        "Split decision: No clear consensus, proceeding with both options",
      confidence: 0.5,
      mediatorOverride: false,
    };
  }
}
