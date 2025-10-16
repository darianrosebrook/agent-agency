/**
 * Consensus Engine
 *
 * Implements multiple consensus algorithms for debate resolution:
 * - Simple majority
 * - Weighted majority (by agent credibility/weight)
 * - Unanimous
 * - Supermajority (2/3+)
 *
 * @author @darianrosebrook
 * @module reasoning/ConsensusEngine
 */
// @ts-nocheck


import {
  ConsensusAlgorithm,
  ConsensusImpossibleError,
  ConsensusResult,
  DebateParticipant,
  DebateVote,
} from "@/types/reasoning";

/**
 * Consensus calculation options
 */
export interface ConsensusOptions {
  algorithm: ConsensusAlgorithm;
  minimumParticipation: number; // 0-1, minimum % of participants that must vote
  confidenceThreshold: number; // 0-1, minimum average confidence for consensus
  supermajorityThreshold?: number; // 0-1, for supermajority algorithm
}

/**
 * Implements various consensus formation algorithms
 */
export class ConsensusEngine {
  private static readonly DEFAULT_OPTIONS: ConsensusOptions = {
    algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
    minimumParticipation: 0.67, // 2/3 must vote
    confidenceThreshold: 0.6, // Average confidence must be > 60%
    supermajorityThreshold: 0.67, // 2/3 for supermajority
  };

  /**
   * Attempts to form consensus based on votes and algorithm
   */
  public static formConsensus(
    votes: DebateVote[],
    participants: DebateParticipant[],
    options: Partial<ConsensusOptions> = {}
  ): ConsensusResult {
    const opts = { ...this.DEFAULT_OPTIONS, ...options };

    // Validate participation
    this.validateParticipation(votes, participants, opts.minimumParticipation);

    // Tally votes
    const tally = this.tallyVotes(votes, participants, opts.algorithm);

    // Calculate consensus based on algorithm
    const result = this.calculateConsensus(tally, votes, opts);

    return result;
  }

  /**
   * Validates that sufficient participants have voted
   */
  private static validateParticipation(
    votes: DebateVote[],
    participants: DebateParticipant[],
    minimumParticipation: number
  ): void {
    const uniqueVoters = new Set(votes.map((v) => v.agentId));
    const participationRate = uniqueVoters.size / participants.length;

    if (participationRate < minimumParticipation) {
      throw new ConsensusImpossibleError(
        "unknown",
        `Insufficient participation: ${(participationRate * 100).toFixed(
          1
        )}% ` + `(minimum: ${(minimumParticipation * 100).toFixed(1)}%)`
      );
    }
  }

  /**
   * Tallies votes with optional weighting
   */
  private static tallyVotes(
    votes: DebateVote[],
    participants: DebateParticipant[],
    algorithm: ConsensusAlgorithm
  ): { for: number; against: number; abstain: number } {
    const participantMap = new Map(participants.map((p) => [p.agentId, p]));

    let forCount = 0;
    let againstCount = 0;
    let abstainCount = 0;

    votes.forEach((vote) => {
      const participant = participantMap.get(vote.agentId);
      const weight =
        algorithm === ConsensusAlgorithm.WEIGHTED_MAJORITY
          ? participant?.weight ?? 1
          : 1;

      switch (vote.position) {
        case "for":
          forCount += weight;
          break;
        case "against":
          againstCount += weight;
          break;
        case "abstain":
          abstainCount += weight;
          break;
      }
    });

    return {
      for: forCount,
      against: againstCount,
      abstain: abstainCount,
    };
  }

  /**
   * Calculates consensus based on algorithm and tally
   */
  private static calculateConsensus(
    tally: { for: number; against: number; abstain: number },
    votes: DebateVote[],
    options: ConsensusOptions
  ): ConsensusResult {
    const total = tally.for + tally.against + tally.abstain;
    const votingTotal = tally.for + tally.against; // Exclude abstentions from percentage

    let reached = false;
    let outcome: "accepted" | "rejected" | "modified" = "rejected";

    switch (options.algorithm) {
      case ConsensusAlgorithm.SIMPLE_MAJORITY:
        reached = votingTotal > 0 && tally.for > tally.against;
        outcome = reached ? "accepted" : "rejected";
        break;

      case ConsensusAlgorithm.WEIGHTED_MAJORITY:
        reached = votingTotal > 0 && tally.for > tally.against;
        outcome = reached ? "accepted" : "rejected";
        break;

      case ConsensusAlgorithm.UNANIMOUS:
        reached = tally.against === 0 && tally.for > 0;
        outcome = reached ? "accepted" : "rejected";
        break;

      case ConsensusAlgorithm.SUPERMAJORITY:
        const threshold = options.supermajorityThreshold ?? 0.67;
        const forPercentage = votingTotal > 0 ? tally.for / votingTotal : 0;
        reached = forPercentage >= threshold;
        outcome = reached ? "accepted" : "rejected";
        break;
    }

    // Calculate confidence
    const averageConfidence =
      votes.length > 0
        ? votes.reduce((sum, v) => sum + v.confidence, 0) / votes.length
        : 0;

    // Check confidence threshold
    if (reached && averageConfidence < options.confidenceThreshold) {
      // Consensus reached but confidence too low - mark as modified
      outcome = "modified";
    }

    // Generate reasoning
    const reasoning = this.generateConsensusReasoning(
      tally,
      options.algorithm,
      averageConfidence,
      reached
    );

    // Calculate overall confidence (combines vote confidence and margin)
    const margin = Math.abs(tally.for - tally.against) / votingTotal;
    const confidence = (averageConfidence + margin) / 2;

    return {
      reached,
      algorithm: options.algorithm,
      outcome,
      confidence: Math.min(1, confidence),
      votingBreakdown: tally,
      reasoning,
      timestamp: new Date(),
    };
  }

  /**
   * Generates human-readable consensus reasoning
   */
  private static generateConsensusReasoning(
    tally: { for: number; against: number; abstain: number },
    algorithm: ConsensusAlgorithm,
    averageConfidence: number,
    reached: boolean
  ): string {
    const total = tally.for + tally.against + tally.abstain;
    const votingTotal = tally.for + tally.against;

    const forPercent =
      votingTotal > 0 ? ((tally.for / votingTotal) * 100).toFixed(1) : "0";
    const againstPercent =
      votingTotal > 0 ? ((tally.against / votingTotal) * 100).toFixed(1) : "0";

    let reasoning = `Using ${algorithm} algorithm: `;
    reasoning += `${forPercent}% for (${tally.for}), `;
    reasoning += `${againstPercent}% against (${tally.against}), `;
    reasoning += `${tally.abstain} abstentions. `;

    if (reached) {
      reasoning += `Consensus reached with ${(averageConfidence * 100).toFixed(
        1
      )}% average confidence.`;
    } else {
      reasoning += `Consensus not reached. `;

      if (algorithm === ConsensusAlgorithm.UNANIMOUS && tally.against > 0) {
        reasoning += `Unanimous consensus required but ${tally.against} voted against.`;
      } else if (algorithm === ConsensusAlgorithm.SUPERMAJORITY) {
        reasoning += `Supermajority threshold not met.`;
      } else {
        reasoning += `Majority not achieved.`;
      }
    }

    return reasoning;
  }

  /**
   * Checks if consensus is mathematically possible given remaining votes
   */
  public static canReachConsensus(
    currentVotes: DebateVote[],
    totalParticipants: number,
    algorithm: ConsensusAlgorithm
  ): boolean {
    const tally = this.tallyVotes(currentVotes, [], algorithm);
    const remainingVotes = totalParticipants - currentVotes.length;

    switch (algorithm) {
      case ConsensusAlgorithm.SIMPLE_MAJORITY:
      case ConsensusAlgorithm.WEIGHTED_MAJORITY:
        // Can reach if for + remaining > against
        return tally.for + remainingVotes > tally.against;

      case ConsensusAlgorithm.UNANIMOUS:
        // Cannot reach if any vote against
        return tally.against === 0;

      case ConsensusAlgorithm.SUPERMAJORITY:
        // Check if for + remaining can meet threshold
        const threshold = 0.67;
        const maxFor = tally.for + remainingVotes;
        const total = totalParticipants - tally.abstain;
        return maxFor / total >= threshold;

      default:
        return false;
    }
  }

  /**
   * Predicts likely consensus outcome given current votes
   */
  public static predictOutcome(
    currentVotes: DebateVote[],
    totalParticipants: number,
    algorithm: ConsensusAlgorithm
  ): "likely_accepted" | "likely_rejected" | "uncertain" {
    const tally = this.tallyVotes(currentVotes, [], algorithm);
    const remainingVotes = totalParticipants - currentVotes.length;
    const votedCount = currentVotes.length;

    // If less than 50% voted, too uncertain
    if (votedCount / totalParticipants < 0.5) {
      return "uncertain";
    }

    // Check current trend
    const currentLeader = tally.for > tally.against ? "for" : "against";
    const margin = Math.abs(tally.for - tally.against);

    // If margin greater than remaining votes, outcome is locked
    if (margin > remainingVotes) {
      return currentLeader === "for" ? "likely_accepted" : "likely_rejected";
    }

    // Otherwise uncertain
    return "uncertain";
  }

  /**
   * Validates that a consensus result is legitimate
   */
  public static validateConsensusResult(
    result: ConsensusResult,
    votes: DebateVote[],
    participants: DebateParticipant[]
  ): boolean {
    // Verify vote counts match
    const forCount = votes.filter((v) => v.position === "for").length;
    const againstCount = votes.filter((v) => v.position === "against").length;
    const abstainCount = votes.filter((v) => v.position === "abstain").length;

    // Note: For weighted algorithms, breakdown represents weights not counts
    if (result.algorithm === ConsensusAlgorithm.SIMPLE_MAJORITY) {
      if (
        result.votingBreakdown.for !== forCount ||
        result.votingBreakdown.against !== againstCount ||
        result.votingBreakdown.abstain !== abstainCount
      ) {
        return false;
      }
    }

    // Verify consensus logic
    if (
      result.algorithm === ConsensusAlgorithm.SIMPLE_MAJORITY ||
      result.algorithm === ConsensusAlgorithm.WEIGHTED_MAJORITY
    ) {
      const shouldBeReached =
        result.votingBreakdown.for > result.votingBreakdown.against;
      if (result.reached !== shouldBeReached) {
        return false;
      }
    }

    if (result.algorithm === ConsensusAlgorithm.UNANIMOUS) {
      const shouldBeReached =
        result.votingBreakdown.against === 0 && result.votingBreakdown.for > 0;
      if (result.reached !== shouldBeReached) {
        return false;
      }
    }

    return true;
  }
}
