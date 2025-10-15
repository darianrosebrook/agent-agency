/**
 * AppealHandler - Manages appeal processing in debates
 *
 * @author @darianrosebrook
 *
 * Handles appeal submissions, reviews, and outcome decisions during debates.
 * Provides mechanisms for participants to challenge decisions, request reviews,
 * and escalate contentious points.
 *
 * Features:
 * - Appeal submission with evidence
 * - Multi-level appeal review
 * - Appeal outcome determination
 * - Appeal history tracking
 * - Automatic escalation rules
 */
// @ts-nocheck


import {
  AgentRole,
  DebateSession,
  DebateState,
  ReasoningEngineError,
} from "@/types/reasoning";

/**
 * Appeal submission request
 */
export interface AppealRequest {
  /** Unique appeal ID */
  appealId: string;

  /** Agent submitting the appeal */
  agentId: string;

  /** What decision is being appealed */
  targetDecision: string;

  /** Reason for the appeal */
  reason: string;

  /** Supporting evidence */
  evidence: string[];

  /** When the appeal was submitted */
  timestamp: Date;

  /** Requested review level (1-3) */
  reviewLevel?: number;
}

/**
 * Appeal status during processing
 */
export enum AppealStatus {
  SUBMITTED = "submitted",
  UNDER_REVIEW = "under_review",
  APPROVED = "approved",
  REJECTED = "rejected",
  ESCALATED = "escalated",
  WITHDRAWN = "withdrawn",
}

/**
 * Appeal review outcome
 */
export interface AppealOutcome {
  /** Appeal ID */
  appealId: string;

  /** Final status */
  status: AppealStatus;

  /** Decision (uphold/overturn/modify) */
  decision: "uphold" | "overturn" | "modify";

  /** Reasoning behind the decision */
  reasoning: string;

  /** Confidence in the decision (0-1) */
  confidence: number;

  /** Reviewing agent(s) */
  reviewers: string[];

  /** When the review was completed */
  completedAt: Date;

  /** Any modifications to the original decision */
  modifications?: string;
}

/**
 * Appeal record with full history
 */
export interface AppealRecord {
  /** The original request */
  request: AppealRequest;

  /** Current status */
  status: AppealStatus;

  /** Review history */
  reviews: Array<{
    reviewer: string;
    recommendation: "uphold" | "overturn" | "modify";
    reasoning: string;
    timestamp: Date;
  }>;

  /** Final outcome (if completed) */
  outcome?: AppealOutcome;
}

/**
 * Configuration for appeal handling
 */
export interface AppealHandlerConfig {
  /** Maximum appeals per agent per debate */
  maxAppealsPerAgent: number;

  /** Maximum appeal review levels (1-3) */
  maxReviewLevels: number;

  /** Automatic escalation threshold (0-1) */
  escalationThreshold: number;

  /** Require mediator approval for appeals */
  requireMediatorApproval: boolean;

  /** Allow appeals during voting */
  allowDuringVoting: boolean;

  /** Minimum confidence for appeal approval (0-1) */
  minConfidenceForApproval: number;
}

/**
 * AppealHandler - Manages appeal processing in debates
 */
export class AppealHandler {
  /** Active appeal records by debate ID */
  private appeals: Map<string, AppealRecord[]> = new Map();

  /** Configuration */
  private config: AppealHandlerConfig;

  constructor(config?: Partial<AppealHandlerConfig>) {
    this.config = {
      maxAppealsPerAgent: 3,
      maxReviewLevels: 3,
      escalationThreshold: 0.7,
      requireMediatorApproval: true,
      allowDuringVoting: false,
      minConfidenceForApproval: 0.6,
      ...config,
    };
  }

  /**
   * Submit an appeal during a debate
   */
  public submitAppeal(
    session: DebateSession,
    request: AppealRequest
  ): AppealRecord {
    // Validate debate state
    if (
      !this.config.allowDuringVoting &&
      session.state === DebateState.CONSENSUS_FORMING
    ) {
      throw new ReasoningEngineError(
        "Appeals not allowed during consensus forming",
        "INVALID_STATE",
        session.id
      );
    }

    // Validate participant
    const participant = session.participants.find(
      (p) => p.agentId === request.agentId
    );
    if (!participant) {
      throw new ReasoningEngineError(
        `Agent ${request.agentId} is not a debate participant`,
        "INVALID_PARTICIPANT",
        session.id
      );
    }

    // Check appeal limit
    const debateAppeals = this.appeals.get(session.id) || [];
    const agentAppeals = debateAppeals.filter(
      (a) => a.request.agentId === request.agentId
    );
    if (agentAppeals.length >= this.config.maxAppealsPerAgent) {
      throw new ReasoningEngineError(
        `Agent ${request.agentId} has reached maximum appeals (${this.config.maxAppealsPerAgent})`,
        "MAX_APPEALS_EXCEEDED",
        session.id
      );
    }

    // Create appeal record
    const record: AppealRecord = {
      request,
      status: AppealStatus.SUBMITTED,
      reviews: [],
    };

    // Store appeal
    if (!this.appeals.has(session.id)) {
      this.appeals.set(session.id, []);
    }
    this.appeals.get(session.id)!.push(record);

    return record;
  }

  /**
   * Review an appeal
   */
  public reviewAppeal(
    session: DebateSession,
    appealId: string,
    reviewer: string,
    recommendation: "uphold" | "overturn" | "modify",
    reasoning: string
  ): AppealRecord {
    const appeal = this.getAppeal(session.id, appealId);

    // Validate reviewer
    const reviewerParticipant = session.participants.find(
      (p) => p.agentId === reviewer
    );
    if (!reviewerParticipant) {
      throw new ReasoningEngineError(
        `Agent ${reviewer} is not a debate participant`,
        "INVALID_REVIEWER",
        session.id
      );
    }

    // Check if mediator approval is required
    if (
      this.config.requireMediatorApproval &&
      reviewerParticipant.role !== AgentRole.MEDIATOR
    ) {
      throw new ReasoningEngineError(
        "Appeal review requires mediator role",
        "MEDIATOR_REQUIRED",
        session.id
      );
    }

    // Add review
    appeal.reviews.push({
      reviewer,
      recommendation,
      reasoning,
      timestamp: new Date(),
    });

    // Update status
    appeal.status = AppealStatus.UNDER_REVIEW;

    return appeal;
  }

  /**
   * Finalize an appeal with an outcome
   */
  public finalizeAppeal(
    session: DebateSession,
    appealId: string,
    decision: "uphold" | "overturn" | "modify",
    reasoning: string,
    modifications?: string
  ): AppealOutcome {
    const appeal = this.getAppeal(session.id, appealId);

    // Validate appeal has been reviewed
    if (appeal.reviews.length === 0) {
      throw new ReasoningEngineError(
        "Appeal must be reviewed before finalization",
        "NO_REVIEWS",
        session.id
      );
    }

    // Calculate confidence based on review consensus
    const recommendationCounts = appeal.reviews.reduce((counts, review) => {
      counts[review.recommendation] = (counts[review.recommendation] || 0) + 1;
      return counts;
    }, {} as Record<string, number>);

    const maxCount = Math.max(...Object.values(recommendationCounts));
    const confidence = maxCount / appeal.reviews.length;

    // Determine final status
    let status: AppealStatus;
    if (decision === "overturn") {
      status = AppealStatus.APPROVED;
    } else if (confidence < this.config.minConfidenceForApproval) {
      status = AppealStatus.ESCALATED;
    } else {
      status =
        decision === "uphold" ? AppealStatus.REJECTED : AppealStatus.APPROVED;
    }

    // Create outcome
    const outcome: AppealOutcome = {
      appealId,
      status,
      decision,
      reasoning,
      confidence,
      reviewers: appeal.reviews.map((r) => r.reviewer),
      completedAt: new Date(),
      modifications,
    };

    // Update appeal
    appeal.status = status;
    appeal.outcome = outcome;

    return outcome;
  }

  /**
   * Withdraw an appeal
   */
  public withdrawAppeal(session: DebateSession, appealId: string): void {
    const appeal = this.getAppeal(session.id, appealId);

    if (
      appeal.status === AppealStatus.APPROVED ||
      appeal.status === AppealStatus.REJECTED
    ) {
      throw new ReasoningEngineError(
        "Cannot withdraw completed appeal",
        "APPEAL_COMPLETED",
        session.id
      );
    }

    appeal.status = AppealStatus.WITHDRAWN;
  }

  /**
   * Get all appeals for a debate
   */
  public getAppeals(debateId: string): AppealRecord[] {
    return this.appeals.get(debateId) || [];
  }

  /**
   * Get appeals by agent
   */
  public getAppealsByAgent(debateId: string, agentId: string): AppealRecord[] {
    const debateAppeals = this.appeals.get(debateId) || [];
    return debateAppeals.filter((a) => a.request.agentId === agentId);
  }

  /**
   * Get appeals by status
   */
  public getAppealsByStatus(
    debateId: string,
    status: AppealStatus
  ): AppealRecord[] {
    const debateAppeals = this.appeals.get(debateId) || [];
    return debateAppeals.filter((a) => a.status === status);
  }

  /**
   * Check if appeal should be automatically escalated
   */
  public shouldEscalate(appeal: AppealRecord): boolean {
    // No escalation if no reviews
    if (appeal.reviews.length === 0) {
      return false;
    }

    // Calculate review consensus
    const recommendations = appeal.reviews.map((r) => r.recommendation);
    const uniqueRecommendations = new Set(recommendations);

    // Escalate if low consensus
    if (uniqueRecommendations.size === recommendations.length) {
      return true; // No agreement at all
    }

    // Calculate confidence
    const recommendationCounts = recommendations.reduce((counts, rec) => {
      counts[rec] = (counts[rec] || 0) + 1;
      return counts;
    }, {} as Record<string, number>);

    const maxCount = Math.max(...Object.values(recommendationCounts));
    const confidence = maxCount / recommendations.length;

    return confidence < this.config.escalationThreshold;
  }

  /**
   * Clear all appeals for a debate
   */
  public clearAppeals(debateId: string): void {
    this.appeals.delete(debateId);
  }

  /**
   * Get appeal statistics
   */
  public getAppealStatistics(debateId: string): {
    total: number;
    byStatus: Record<AppealStatus, number>;
    byAgent: Record<string, number>;
    averageReviews: number;
  } {
    const debateAppeals = this.appeals.get(debateId) || [];

    const byStatus = debateAppeals.reduce((counts, appeal) => {
      counts[appeal.status] = (counts[appeal.status] || 0) + 1;
      return counts;
    }, {} as Record<AppealStatus, number>);

    const byAgent = debateAppeals.reduce((counts, appeal) => {
      const agentId = appeal.request.agentId;
      counts[agentId] = (counts[agentId] || 0) + 1;
      return counts;
    }, {} as Record<string, number>);

    const totalReviews = debateAppeals.reduce(
      (sum, appeal) => sum + appeal.reviews.length,
      0
    );
    const averageReviews =
      debateAppeals.length > 0 ? totalReviews / debateAppeals.length : 0;

    return {
      total: debateAppeals.length,
      byStatus,
      byAgent,
      averageReviews,
    };
  }

  /**
   * Get a specific appeal
   */
  private getAppeal(debateId: string, appealId: string): AppealRecord {
    const debateAppeals = this.appeals.get(debateId);
    if (!debateAppeals) {
      throw new ReasoningEngineError(
        "No appeals found for debate",
        "NO_APPEALS",
        debateId
      );
    }

    const appeal = debateAppeals.find((a) => a.request.appealId === appealId);
    if (!appeal) {
      throw new ReasoningEngineError(
        `Appeal ${appealId} not found`,
        "APPEAL_NOT_FOUND",
        debateId
      );
    }

    return appeal;
  }
}
