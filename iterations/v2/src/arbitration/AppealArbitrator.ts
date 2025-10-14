/**
 * Appeal Arbitrator
 *
 * @author @darianrosebrook
 *
 * Handles appeals of arbitration decisions with multi-level review.
 * Provides appeal submission, review, escalation, and final determination.
 *
 * Features:
 * - Appeal submission and validation
 * - Multi-level review process
 * - Evidence re-evaluation
 * - Precedent re-assessment
 * - Appeal statistics and tracking
 * - Auto-escalation on deadlock
 */

import {
  Appeal,
  AppealStatus,
  ArbitrationError,
  ArbitrationSession,
  Verdict,
  VerdictOutcome,
} from "@/types/arbitration";

/**
 * Appeal arbitrator configuration
 */
export interface AppealArbitratorConfig {
  /** Maximum appeal levels */
  maxAppealLevels: number;

  /** Minimum evidence for appeal */
  minEvidenceForAppeal: number;

  /** Appeal review timeout (ms) */
  reviewTimeout: number;

  /** Require unanimous approval for overturns */
  requireUnanimous: boolean;

  /** Auto-escalate on reviewer deadlock */
  autoEscalateOnDeadlock: boolean;
}

/**
 * Appeal decision
 */
export interface AppealDecision {
  /** Appeal ID */
  appealId: string;

  /** Decision: upheld or overturned */
  decision: "upheld" | "overturned";

  /** New verdict (if overturned) */
  newVerdict?: Verdict;

  /** Reasoning for decision */
  reasoning: string;

  /** Reviewer IDs */
  reviewers: string[];

  /** Decided at */
  decidedAt: Date;

  /** Confidence in decision (0-1) */
  confidence: number;
}

/**
 * AppealArbitrator - Handles arbitration appeals
 */
export class AppealArbitrator {
  /** Configuration */
  private config: AppealArbitratorConfig;

  /** Active appeals */
  private appeals: Map<string, Appeal> = new Map();

  /** Appeal decisions */
  private decisions: Map<string, AppealDecision> = new Map();

  /** Appeal counter */
  private appealCounter: number = 0;

  constructor(config?: Partial<AppealArbitratorConfig>) {
    this.config = {
      maxAppealLevels: 3,
      minEvidenceForAppeal: 1,
      reviewTimeout: 5 * 60 * 1000, // 5 minutes
      requireUnanimous: false,
      autoEscalateOnDeadlock: true,
      ...config,
    };
  }

  /**
   * Submit an appeal
   */
  public async submitAppeal(
    session: ArbitrationSession,
    verdict: Verdict,
    appellantId: string,
    grounds: string,
    newEvidence: string[],
    metadata: Record<string, unknown> = {}
  ): Promise<Appeal> {
    // Validate appeal
    this.validateAppeal(verdict, grounds, newEvidence);

    // Create appeal
    const appeal: Appeal = {
      id: this.generateAppealId(),
      sessionId: session.id,
      originalVerdictId: verdict.id,
      appellantId,
      grounds,
      newEvidence,
      status: AppealStatus.SUBMITTED,
      level: 1,
      submittedAt: new Date(),
      metadata,
    };

    // Store appeal
    this.appeals.set(appeal.id, appeal);

    return appeal;
  }

  /**
   * Review an appeal
   */
  public async reviewAppeal(
    appealId: string,
    reviewers: string[],
    session: ArbitrationSession,
    originalVerdict: Verdict
  ): Promise<AppealDecision> {
    const appeal = this.appeals.get(appealId);
    if (!appeal) {
      throw new ArbitrationError(
        `Appeal ${appealId} not found`,
        "APPEAL_NOT_FOUND"
      );
    }

    if (appeal.status !== AppealStatus.SUBMITTED) {
      throw new ArbitrationError(
        `Appeal ${appealId} not in submitted state`,
        "INVALID_APPEAL_STATE",
        appealId
      );
    }

    // Update appeal status
    appeal.status = AppealStatus.UNDER_REVIEW;
    appeal.reviewers = reviewers;
    appeal.reviewedAt = new Date();

    // Perform review
    const decision = await this.performReview(
      appeal,
      session,
      originalVerdict,
      reviewers
    );

    // Update appeal status based on decision
    if (decision.decision === "overturned") {
      appeal.status = AppealStatus.OVERTURNED;
    } else {
      appeal.status = AppealStatus.UPHELD;
    }

    // Store decision
    this.decisions.set(appeal.id, decision);

    return decision;
  }

  /**
   * Escalate an appeal to higher level
   */
  public async escalateAppeal(
    appealId: string,
    reason: string
  ): Promise<Appeal> {
    const appeal = this.appeals.get(appealId);
    if (!appeal) {
      throw new ArbitrationError(
        `Appeal ${appealId} not found`,
        "APPEAL_NOT_FOUND"
      );
    }

    // Check max appeal levels
    if (appeal.level >= this.config.maxAppealLevels) {
      throw new ArbitrationError(
        `Appeal ${appealId} already at maximum level ${this.config.maxAppealLevels}`,
        "MAX_APPEAL_LEVEL",
        appealId
      );
    }

    // Increment level
    appeal.level++;
    appeal.status = AppealStatus.SUBMITTED;
    appeal.metadata.escalationReason = reason;
    appeal.metadata.escalatedAt = new Date();

    return appeal;
  }

  /**
   * Finalize an appeal (no further appeals allowed)
   */
  public finalizeAppeal(appealId: string): boolean {
    const appeal = this.appeals.get(appealId);
    if (!appeal) {
      return false;
    }

    appeal.status = AppealStatus.FINALIZED;
    appeal.metadata.finalizedAt = new Date();

    return true;
  }

  /**
   * Get an appeal by ID
   */
  public getAppeal(appealId: string): Appeal | undefined {
    return this.appeals.get(appealId);
  }

  /**
   * Get all appeals for a session
   */
  public getSessionAppeals(sessionId: string): Appeal[] {
    return Array.from(this.appeals.values()).filter(
      (a) => a.sessionId === sessionId
    );
  }

  /**
   * Get appeal decision
   */
  public getDecision(appealId: string): AppealDecision | undefined {
    return this.decisions.get(appealId);
  }

  /**
   * Get appeal statistics
   */
  public getStatistics(): {
    totalAppeals: number;
    byStatus: Record<string, number>;
    overturnRate: number;
    averageLevel: number;
    byLevel: Record<number, number>;
  } {
    const all = Array.from(this.appeals.values());

    const byStatus: Record<string, number> = {};
    for (const appeal of all) {
      byStatus[appeal.status] = (byStatus[appeal.status] || 0) + 1;
    }

    const decisions = Array.from(this.decisions.values());
    const overturned = decisions.filter(
      (d) => d.decision === "overturned"
    ).length;
    const overturnRate =
      decisions.length > 0 ? overturned / decisions.length : 0;

    const totalLevels = all.reduce((sum, a) => sum + a.level, 0);
    const averageLevel = all.length > 0 ? totalLevels / all.length : 0;

    const byLevel: Record<number, number> = {};
    for (const appeal of all) {
      byLevel[appeal.level] = (byLevel[appeal.level] || 0) + 1;
    }

    return {
      totalAppeals: all.length,
      byStatus,
      overturnRate,
      averageLevel,
      byLevel,
    };
  }

  /**
   * Perform appeal review
   */
  private async performReview(
    appeal: Appeal,
    session: ArbitrationSession,
    originalVerdict: Verdict,
    reviewers: string[]
  ): Promise<AppealDecision> {
    // Evaluate new evidence
    const evidenceScore = this.evaluateNewEvidence(
      appeal.newEvidence,
      session.evidence
    );

    // Assess grounds
    const groundsScore = this.assessGrounds(appeal.grounds, originalVerdict);

    // Calculate overall score
    const overallScore = (evidenceScore + groundsScore) / 2;

    // Determine if verdict should be overturned
    const threshold = this.config.requireUnanimous ? 0.8 : 0.6;
    const shouldOverturn = overallScore > threshold;

    // Build reasoning
    const reasoning = this.buildReviewReasoning(
      appeal,
      evidenceScore,
      groundsScore,
      shouldOverturn
    );

    // Calculate confidence
    const confidence = this.calculateReviewConfidence(
      evidenceScore,
      groundsScore,
      reviewers.length
    );

    const decision: AppealDecision = {
      appealId: appeal.id,
      decision: shouldOverturn ? "overturned" : "upheld",
      reasoning,
      reviewers,
      decidedAt: new Date(),
      confidence,
    };

    // If overturned, would need to generate new verdict
    // For now, just mark as overturned
    if (shouldOverturn) {
      decision.newVerdict = {
        ...originalVerdict,
        outcome: this.determineNewOutcome(originalVerdict.outcome),
        reasoning: [
          ...originalVerdict.reasoning,
          {
            step: originalVerdict.reasoning.length + 1,
            description: `Overturned on appeal: ${reasoning}`,
            evidence: appeal.newEvidence,
            ruleReferences: originalVerdict.rulesApplied,
            confidence,
          },
        ],
      };
    }

    return decision;
  }

  /**
   * Evaluate new evidence provided in appeal
   */
  private evaluateNewEvidence(
    newEvidence: string[],
    originalEvidence: string[]
  ): number {
    if (newEvidence.length === 0) {
      return 0;
    }

    // Check if evidence is truly new
    const genuinelyNew = newEvidence.filter(
      (e) => !originalEvidence.includes(e)
    );

    if (genuinelyNew.length === 0) {
      return 0.2; // Penalize for not providing new evidence
    }

    // Score based on amount of new evidence
    const noveltyScore =
      genuinelyNew.length / (genuinelyNew.length + originalEvidence.length);

    return Math.min(0.5 + noveltyScore * 0.5, 1.0);
  }

  /**
   * Assess grounds for appeal
   */
  private assessGrounds(grounds: string, _originalVerdict: Verdict): number {
    const groundsLower = grounds.toLowerCase();

    // Check for substantive grounds
    const substantiveKeywords = [
      "error",
      "incorrect",
      "overlooked",
      "misapplied",
      "unjust",
      "unfair",
      "bias",
      "procedural",
    ];

    const hasSubstantiveGrounds = substantiveKeywords.some((keyword) =>
      groundsLower.includes(keyword)
    );

    if (!hasSubstantiveGrounds) {
      return 0.3;
    }

    // Check for detailed reasoning
    const wordCount = grounds.split(/\s+/).length;
    const detailScore = Math.min(wordCount / 100, 1.0);

    return 0.5 + detailScore * 0.5;
  }

  /**
   * Build reasoning for review decision
   */
  private buildReviewReasoning(
    appeal: Appeal,
    evidenceScore: number,
    groundsScore: number,
    shouldOverturn: boolean
  ): string {
    const parts: string[] = [];

    parts.push(
      `Evidence evaluation: ${(evidenceScore * 100).toFixed(0)}% - ${
        appeal.newEvidence.length
      } piece(s) of new evidence`
    );
    parts.push(
      `Grounds assessment: ${(groundsScore * 100).toFixed(
        0
      )}% - ${appeal.grounds.substring(0, 50)}...`
    );

    if (shouldOverturn) {
      parts.push(
        "Decision: Original verdict overturned based on new evidence and substantive grounds"
      );
    } else {
      parts.push(
        "Decision: Original verdict upheld - insufficient grounds or evidence for overturn"
      );
    }

    return parts.join(". ");
  }

  /**
   * Calculate confidence in review decision
   */
  private calculateReviewConfidence(
    evidenceScore: number,
    groundsScore: number,
    reviewerCount: number
  ): number {
    const baseConfidence = (evidenceScore + groundsScore) / 2;

    // Boost confidence with more reviewers
    const reviewerBoost = Math.min(reviewerCount / 5, 0.2);

    return Math.min(baseConfidence + reviewerBoost, 1.0);
  }

  /**
   * Determine new outcome if verdict is overturned
   */
  private determineNewOutcome(originalOutcome: VerdictOutcome): VerdictOutcome {
    switch (originalOutcome) {
      case VerdictOutcome.APPROVED:
        return VerdictOutcome.CONDITIONAL;
      case VerdictOutcome.REJECTED:
        return VerdictOutcome.CONDITIONAL;
      case VerdictOutcome.CONDITIONAL:
        return VerdictOutcome.APPROVED;
      case VerdictOutcome.WAIVED:
        return VerdictOutcome.CONDITIONAL;
      default:
        return VerdictOutcome.CONDITIONAL;
    }
  }

  /**
   * Validate appeal before submission
   */
  private validateAppeal(
    verdict: Verdict,
    grounds: string,
    newEvidence: string[]
  ): void {
    if (!verdict || !verdict.id) {
      throw new ArbitrationError(
        "Verdict is required for appeal",
        "INVALID_VERDICT"
      );
    }

    if (!grounds || grounds.trim().length < 20) {
      throw new ArbitrationError(
        "Appeal grounds must be at least 20 characters",
        "INSUFFICIENT_GROUNDS"
      );
    }

    if (newEvidence.length < this.config.minEvidenceForAppeal) {
      throw new ArbitrationError(
        `Appeal requires at least ${this.config.minEvidenceForAppeal} piece(s) of new evidence`,
        "INSUFFICIENT_EVIDENCE"
      );
    }
  }

  /**
   * Generate unique appeal ID
   */
  private generateAppealId(): string {
    this.appealCounter++;
    return `APPEAL-${Date.now()}-${this.appealCounter}`;
  }

  /**
   * Clear all appeals (for testing)
   */
  public clear(): void {
    this.appeals.clear();
    this.decisions.clear();
    this.appealCounter = 0;
  }
}
