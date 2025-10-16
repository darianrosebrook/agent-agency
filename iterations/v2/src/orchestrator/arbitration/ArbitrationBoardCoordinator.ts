/**
 * @fileoverview Arbitration Board Coordinator - ARBITER-025
 *
 * Aggregates pleading decisions with confidence weights to produce final
 * arbitration decisions with comprehensive justification.
 *
 * @author @darianrosebrook
 */

import {
  ArbitrationContext,
  ConfidenceScore,
  ConfidenceScorer,
} from "./ConfidenceScorer";

export interface PleadingDecision {
  id: string;
  workerId: string;
  decision: "approve" | "deny" | "abstain";
  confidence: number;
  reasoning: string;
  evidence: any;
  timestamp: Date;
}

export interface ArbitrationBoardDecision {
  finalDecision: "approve" | "deny";
  confidence: number;
  reasoning: string[];
  participatingWorkers: string[];
  decisionBreakdown: {
    approve: { count: number; totalConfidence: number; workers: string[] };
    deny: { count: number; totalConfidence: number; workers: string[] };
    abstain: { count: number; totalConfidence: number; workers: string[] };
  };
  consensusLevel: "unanimous" | "strong" | "weak" | "contested";
  escalationRequired: boolean;
  metadata: {
    totalDecisions: number;
    averageConfidence: number;
    processingTimeMs: number;
    timestamp: Date;
  };
}

export interface ArbitrationBoardConfig {
  minParticipants: number;
  confidenceThreshold: number;
  escalationThreshold: number;
  consensusWeights: {
    unanimous: number;
    strong: number;
    weak: number;
    contested: number;
  };
}

export class ArbitrationBoardCoordinator {
  private readonly defaultConfig: ArbitrationBoardConfig = {
    minParticipants: 3,
    confidenceThreshold: 0.6,
    escalationThreshold: 0.3,
    consensusWeights: {
      unanimous: 1.0,
      strong: 0.8,
      weak: 0.6,
      contested: 0.4,
    },
  };

  constructor(
    private confidenceScorer: ConfidenceScorer,
    private config: Partial<ArbitrationBoardConfig> = {}
  ) {
    this.config = { ...this.defaultConfig, ...config };
  }

  /**
   * Coordinate arbitration board decision from pleading decisions
   */
  async coordinateDecision(
    pleadingDecisions: PleadingDecision[],
    context: ArbitrationContext
  ): Promise<ArbitrationBoardDecision> {
    const startTime = Date.now();

    // Validate minimum participants
    if (pleadingDecisions.length < this.config.minParticipants!) {
      throw new Error(
        `Insufficient participants: ${pleadingDecisions.length} < ${this.config.minParticipants}`
      );
    }

    // Calculate confidence scores for each decision
    const scoredDecisions = await this.scoreDecisions(
      pleadingDecisions,
      context
    );

    // Analyze decision breakdown
    const breakdown = this.analyzeDecisionBreakdown(scoredDecisions);

    // Determine consensus level
    const consensusLevel = this.determineConsensusLevel(breakdown);

    // Calculate final decision
    const finalDecision = this.calculateFinalDecision(
      breakdown,
      consensusLevel
    );

    // Calculate overall confidence
    const confidence = this.calculateOverallConfidence(
      scoredDecisions,
      breakdown,
      consensusLevel
    );

    // Generate reasoning
    const reasoning = this.generateReasoning(
      breakdown,
      consensusLevel,
      confidence
    );

    // Check if escalation is required
    const escalationRequired = this.shouldEscalate(
      confidence,
      breakdown,
      consensusLevel
    );

    const processingTimeMs = Date.now() - startTime;

    return {
      finalDecision,
      confidence,
      reasoning,
      participatingWorkers: pleadingDecisions.map((d) => d.workerId),
      decisionBreakdown: breakdown,
      consensusLevel,
      escalationRequired,
      metadata: {
        totalDecisions: pleadingDecisions.length,
        averageConfidence:
          scoredDecisions.reduce((sum, d) => sum + d.confidence, 0) /
          scoredDecisions.length,
        processingTimeMs,
        timestamp: new Date(),
      },
    };
  }

  /**
   * Score individual pleading decisions
   */
  private async scoreDecisions(
    pleadingDecisions: PleadingDecision[],
    context: ArbitrationContext
  ): Promise<Array<PleadingDecision & { confidenceScore: ConfidenceScore }>> {
    const scoredDecisions = [];

    for (const decision of pleadingDecisions) {
      // Create context for this specific worker decision
      const workerContext: ArbitrationContext = {
        ...context,
        workerId: decision.workerId,
      };

      const confidenceScore = await this.confidenceScorer.calculateConfidence(
        workerContext
      );

      scoredDecisions.push({
        ...decision,
        confidenceScore,
      });
    }

    return scoredDecisions;
  }

  /**
   * Analyze decision breakdown by category
   */
  private analyzeDecisionBreakdown(
    scoredDecisions: Array<
      PleadingDecision & { confidenceScore: ConfidenceScore }
    >
  ): ArbitrationBoardDecision["decisionBreakdown"] {
    const breakdown = {
      approve: { count: 0, totalConfidence: 0, workers: [] as string[] },
      deny: { count: 0, totalConfidence: 0, workers: [] as string[] },
      abstain: { count: 0, totalConfidence: 0, workers: [] as string[] },
    };

    for (const decision of scoredDecisions) {
      const category = decision.decision;
      breakdown[category].count++;
      breakdown[category].totalConfidence += decision.confidenceScore.overall;
      breakdown[category].workers.push(decision.workerId);
    }

    return breakdown;
  }

  /**
   * Determine consensus level based on decision distribution
   */
  private determineConsensusLevel(
    breakdown: ArbitrationBoardDecision["decisionBreakdown"]
  ): ArbitrationBoardDecision["consensusLevel"] {
    const total =
      breakdown.approve.count + breakdown.deny.count + breakdown.abstain.count;

    if (total === 0) {
      return "contested";
    }

    const approveRatio = breakdown.approve.count / total;
    const denyRatio = breakdown.deny.count / total;

    // Unanimous decision
    if (breakdown.approve.count === total || breakdown.deny.count === total) {
      return "unanimous";
    }

    // Strong consensus (>75% agreement)
    if (approveRatio >= 0.75 || denyRatio >= 0.75) {
      return "strong";
    }

    // Weak consensus (50-75% agreement)
    if (approveRatio >= 0.5 || denyRatio >= 0.5) {
      return "weak";
    }

    // Contested (<50% agreement)
    return "contested";
  }

  /**
   * Calculate final decision based on breakdown and consensus
   */
  private calculateFinalDecision(
    breakdown: ArbitrationBoardDecision["decisionBreakdown"],
    consensusLevel: ArbitrationBoardDecision["consensusLevel"]
  ): "approve" | "deny" {
    // For unanimous decisions, follow the majority
    if (consensusLevel === "unanimous") {
      return breakdown.approve.count > breakdown.deny.count
        ? "approve"
        : "deny";
    }

    // For other consensus levels, use weighted confidence
    const approveConfidence =
      breakdown.approve.count > 0
        ? breakdown.approve.totalConfidence / breakdown.approve.count
        : 0;
    const denyConfidence =
      breakdown.deny.count > 0
        ? breakdown.deny.totalConfidence / breakdown.deny.count
        : 0;

    // Weight by consensus level
    const consensusWeight = this.config.consensusWeights![consensusLevel];
    const weightedApproveConfidence = approveConfidence * consensusWeight;
    const weightedDenyConfidence = denyConfidence * consensusWeight;

    // Also consider count as a tiebreaker
    const countWeight = 0.1;
    const finalApproveScore =
      weightedApproveConfidence + breakdown.approve.count * countWeight;
    const finalDenyScore =
      weightedDenyConfidence + breakdown.deny.count * countWeight;

    return finalApproveScore > finalDenyScore ? "approve" : "deny";
  }

  /**
   * Calculate overall confidence for the final decision
   */
  private calculateOverallConfidence(
    scoredDecisions: Array<
      PleadingDecision & { confidenceScore: ConfidenceScore }
    >,
    breakdown: ArbitrationBoardDecision["decisionBreakdown"],
    consensusLevel: ArbitrationBoardDecision["consensusLevel"]
  ): number {
    // Base confidence from consensus level
    const baseConfidence = this.config.consensusWeights![consensusLevel];

    // Adjust based on average confidence of participants
    const averageConfidence =
      scoredDecisions.reduce((sum, d) => sum + d.confidenceScore.overall, 0) /
      scoredDecisions.length;

    // Adjust based on decision distribution
    const totalDecisions =
      breakdown.approve.count + breakdown.deny.count + breakdown.abstain.count;
    const majorityRatio =
      Math.max(breakdown.approve.count, breakdown.deny.count) / totalDecisions;

    // Combine factors
    const finalConfidence =
      baseConfidence * 0.4 + averageConfidence * 0.4 + majorityRatio * 0.2;

    return Math.max(0, Math.min(1, finalConfidence));
  }

  /**
   * Generate comprehensive reasoning for the decision
   */
  private generateReasoning(
    breakdown: ArbitrationBoardDecision["decisionBreakdown"],
    consensusLevel: ArbitrationBoardDecision["consensusLevel"],
    confidence: number
  ): string[] {
    const reasoning: string[] = [];

    // Consensus analysis
    reasoning.push(`Consensus level: ${consensusLevel}`);

    if (consensusLevel === "unanimous") {
      reasoning.push("All participating workers reached the same decision");
    } else if (consensusLevel === "strong") {
      reasoning.push("Strong majority agreement among workers");
    } else if (consensusLevel === "weak") {
      reasoning.push("Weak consensus with some disagreement");
    } else {
      reasoning.push("Contested decision with significant disagreement");
    }

    // Decision breakdown
    reasoning.push(
      `Decision breakdown: ${breakdown.approve.count} approve, ${breakdown.deny.count} deny, ${breakdown.abstain.count} abstain`
    );

    // Confidence explanation
    if (confidence >= 0.8) {
      reasoning.push("High confidence in the final decision");
    } else if (confidence >= 0.6) {
      reasoning.push("Moderate confidence in the final decision");
    } else if (confidence >= 0.4) {
      reasoning.push("Low confidence in the final decision");
    } else {
      reasoning.push("Very low confidence - decision may be unreliable");
    }

    // Participant analysis
    const totalParticipants =
      breakdown.approve.count + breakdown.deny.count + breakdown.abstain.count;
    reasoning.push(
      `${totalParticipants} workers participated in the arbitration`
    );

    return reasoning;
  }

  /**
   * Determine if escalation is required
   */
  private shouldEscalate(
    confidence: number,
    breakdown: ArbitrationBoardDecision["decisionBreakdown"],
    consensusLevel: ArbitrationBoardDecision["consensusLevel"]
  ): boolean {
    // Escalate if confidence is below threshold
    if (confidence < this.config.escalationThreshold!) {
      return true;
    }

    // Escalate if contested consensus
    if (consensusLevel === "contested") {
      return true;
    }

    // Escalate if too many abstentions
    const totalDecisions =
      breakdown.approve.count + breakdown.deny.count + breakdown.abstain.count;
    const abstentionRatio = breakdown.abstain.count / totalDecisions;
    if (abstentionRatio > 0.5) {
      return true;
    }

    return false;
  }

  /**
   * Get arbitration board statistics
   */
  getStatistics(): {
    config: ArbitrationBoardConfig;
    consensusWeights: Record<string, number>;
  } {
    return {
      config: this.config as ArbitrationBoardConfig,
      consensusWeights: this.config.consensusWeights!,
    };
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig: Partial<ArbitrationBoardConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }
}

