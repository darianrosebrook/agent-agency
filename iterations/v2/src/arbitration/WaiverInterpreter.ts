/**
 * Waiver Interpreter
 *
 * @author @darianrosebrook
 *
 * Evaluates and manages waiver requests for constitutional rule violations.
 * Provides time-bounded waiver approvals with automatic expiration and conditions.
 *
 * Features:
 * - Waiver request evaluation
 * - Evidence-based approval/denial
 * - Time-bounded waivers with auto-expiration
 * - Conditional waiver support
 * - Waiver tracking and history
 * - Automatic revocation
 */

import {
  ConstitutionalRule,
  WaiverDecision,
  WaiverRequest,
  WaiverStatus,
} from "@/types/arbitration";

/**
 * Waiver interpreter configuration
 */
export interface WaiverInterpreterConfig {
  /** Maximum waiver duration (ms) */
  maxWaiverDuration: number;

  /** Default waiver duration (ms) */
  defaultWaiverDuration: number;

  /** Require justification for all waivers */
  requireJustification: boolean;

  /** Minimum evidence count for approval */
  minEvidenceForApproval: number;

  /** Auto-revoke on expiration */
  autoRevokeOnExpiration: boolean;

  /** Allow conditional waivers */
  allowConditionalWaivers: boolean;
}

/**
 * Waiver evaluation result
 */
export interface WaiverEvaluationResult {
  /** Whether waiver should be approved */
  shouldApprove: boolean;

  /** Reasoning for decision */
  reasoning: string;

  /** Recommended duration (if approved) */
  recommendedDuration?: number;

  /** Conditions (if conditional approval) */
  conditions?: string[];

  /** Confidence in decision (0-1) */
  confidence: number;
}

/**
 * WaiverInterpreter - Evaluates and manages waiver requests
 */
export class WaiverInterpreter {
  /** Configuration */
  private config: WaiverInterpreterConfig;

  /** Active waivers by rule ID */
  private activeWaivers: Map<string, WaiverDecision> = new Map();

  /** Waiver history */
  private waiverHistory: WaiverDecision[] = [];

  constructor(config?: Partial<WaiverInterpreterConfig>) {
    this.config = {
      maxWaiverDuration: 7 * 24 * 60 * 60 * 1000, // 7 days
      defaultWaiverDuration: 24 * 60 * 60 * 1000, // 24 hours
      requireJustification: true,
      minEvidenceForApproval: 2,
      autoRevokeOnExpiration: true,
      allowConditionalWaivers: true,
      ...config,
    };
  }

  /**
   * Evaluate a waiver request
   */
  public async evaluateWaiver(
    request: WaiverRequest,
    rule: ConstitutionalRule,
    decidedBy: string
  ): Promise<WaiverEvaluationResult> {
    // Check if rule is waivable
    if (!rule.waivable) {
      return {
        shouldApprove: false,
        reasoning: `Rule ${rule.id} is not waivable`,
        confidence: 1.0,
      };
    }

    // Check justification
    if (
      this.config.requireJustification &&
      (!request.justification || request.justification.length < 10)
    ) {
      return {
        shouldApprove: false,
        reasoning: "Insufficient justification provided",
        confidence: 0.9,
      };
    }

    // Check evidence
    if (request.evidence.length < this.config.minEvidenceForApproval) {
      if (this.config.allowConditionalWaivers) {
        return {
          shouldApprove: true,
          reasoning: `Conditional approval pending additional evidence (${request.evidence.length}/${this.config.minEvidenceForApproval})`,
          recommendedDuration: this.config.defaultWaiverDuration / 2,
          conditions: [
            `Must provide ${
              this.config.minEvidenceForApproval - request.evidence.length
            } additional evidence item(s)`,
            "Waiver subject to review upon evidence submission",
          ],
          confidence: 0.6,
        };
      }

      return {
        shouldApprove: false,
        reasoning: `Insufficient evidence (${request.evidence.length}/${this.config.minEvidenceForApproval} required)`,
        confidence: 0.8,
      };
    }

    // Check requested duration
    if (request.requestedDuration > this.config.maxWaiverDuration) {
      return {
        shouldApprove: true,
        reasoning: `Approved with reduced duration (requested ${this.formatDuration(
          request.requestedDuration
        )}, approved ${this.formatDuration(this.config.maxWaiverDuration)})`,
        recommendedDuration: this.config.maxWaiverDuration,
        conditions: ["May request extension before expiration"],
        confidence: 0.75,
      };
    }

    // Check existing waivers
    const existingWaiver = this.activeWaivers.get(rule.id);
    if (existingWaiver && existingWaiver.status === WaiverStatus.APPROVED) {
      return {
        shouldApprove: false,
        reasoning: `Active waiver already exists for rule ${
          rule.id
        } (expires ${existingWaiver.expiresAt?.toISOString()})`,
        confidence: 1.0,
      };
    }

    // Approve with conditions based on rule severity
    const conditions: string[] = [];
    let duration =
      request.requestedDuration || this.config.defaultWaiverDuration;

    if (rule.severity === "critical" || rule.severity === "major") {
      conditions.push("Must provide weekly progress reports");
      conditions.push("Subject to immediate revocation if conditions violated");
      duration = Math.min(duration, this.config.defaultWaiverDuration);
    }

    return {
      shouldApprove: true,
      reasoning: `Approved based on adequate justification and evidence`,
      recommendedDuration: duration,
      conditions: conditions.length > 0 ? conditions : undefined,
      confidence: 0.85,
    };
  }

  /**
   * Process waiver request and create decision
   */
  public async processWaiver(
    request: WaiverRequest,
    rule: ConstitutionalRule,
    decidedBy: string
  ): Promise<WaiverDecision> {
    // Evaluate waiver
    const evaluation = await this.evaluateWaiver(request, rule, decidedBy);

    // Create decision
    const decision: WaiverDecision = {
      requestId: request.id,
      status: evaluation.shouldApprove
        ? WaiverStatus.APPROVED
        : WaiverStatus.REJECTED,
      reasoning: evaluation.reasoning,
      decidedBy,
      decidedAt: new Date(),
    };

    // Add approval details
    if (evaluation.shouldApprove) {
      const duration =
        evaluation.recommendedDuration ||
        request.requestedDuration ||
        this.config.defaultWaiverDuration;

      decision.approvedDuration = duration;
      decision.expiresAt = new Date(Date.now() + duration);
      decision.conditions = evaluation.conditions;

      // Set auto-revoke
      if (this.config.autoRevokeOnExpiration) {
        decision.autoRevokeAt = decision.expiresAt;
      }

      // Track active waiver
      this.activeWaivers.set(rule.id, decision);
    }

    // Add to history
    this.waiverHistory.push(decision);

    return decision;
  }

  /**
   * Check if a waiver is active for a rule
   */
  public isWaiverActive(ruleId: string): boolean {
    const waiver = this.activeWaivers.get(ruleId);

    if (!waiver) {
      return false;
    }

    // Check expiration
    if (waiver.expiresAt && waiver.expiresAt < new Date()) {
      // Auto-revoke if configured
      if (this.config.autoRevokeOnExpiration) {
        this.revokeWaiver(
          ruleId,
          "system",
          "Automatic revocation on expiration"
        );
      }
      return false;
    }

    return waiver.status === WaiverStatus.APPROVED;
  }

  /**
   * Get active waiver for a rule
   */
  public getActiveWaiver(ruleId: string): WaiverDecision | undefined {
    const waiver = this.activeWaivers.get(ruleId);

    if (!waiver) {
      return undefined;
    }

    // Check expiration
    if (waiver.expiresAt && waiver.expiresAt < new Date()) {
      return undefined;
    }

    return waiver;
  }

  /**
   * Revoke a waiver
   */
  public revokeWaiver(
    ruleId: string,
    revokedBy: string,
    reason: string
  ): boolean {
    const waiver = this.activeWaivers.get(ruleId);

    if (!waiver) {
      return false;
    }

    // Update status
    waiver.status = WaiverStatus.REVOKED;
    waiver.reasoning = `${waiver.reasoning} | Revoked by ${revokedBy}: ${reason}`;

    // Remove from active
    this.activeWaivers.delete(ruleId);

    return true;
  }

  /**
   * Extend a waiver duration
   */
  public extendWaiver(
    ruleId: string,
    extensionDuration: number,
    extendedBy: string,
    justification: string
  ): WaiverDecision | null {
    const waiver = this.activeWaivers.get(ruleId);

    if (!waiver || !waiver.expiresAt) {
      return null;
    }

    // Calculate new expiration
    const newExpiration = new Date(
      waiver.expiresAt.getTime() + extensionDuration
    );

    // Check against max duration
    const totalDuration = newExpiration.getTime() - Date.now();
    if (totalDuration > this.config.maxWaiverDuration) {
      return null;
    }

    // Update waiver
    waiver.expiresAt = newExpiration;
    waiver.reasoning = `${waiver.reasoning} | Extended by ${extendedBy}: ${justification}`;

    if (waiver.autoRevokeAt) {
      waiver.autoRevokeAt = newExpiration;
    }

    return waiver;
  }

  /**
   * Get all active waivers
   */
  public getActiveWaivers(): WaiverDecision[] {
    const now = new Date();

    return Array.from(this.activeWaivers.values()).filter(
      (waiver) =>
        waiver.status === WaiverStatus.APPROVED &&
        (!waiver.expiresAt || waiver.expiresAt > now)
    );
  }

  /**
   * Get waiver history
   */
  public getWaiverHistory(ruleId?: string): WaiverDecision[] {
    if (ruleId) {
      // Would need to track ruleId in WaiverDecision to filter
      return this.waiverHistory;
    }
    return this.waiverHistory;
  }

  /**
   * Clean up expired waivers
   */
  public cleanupExpiredWaivers(): number {
    let cleanedCount = 0;
    const now = new Date();

    for (const [ruleId, waiver] of this.activeWaivers.entries()) {
      if (waiver.expiresAt && waiver.expiresAt < now) {
        if (this.config.autoRevokeOnExpiration) {
          this.revokeWaiver(
            ruleId,
            "system",
            "Automatic cleanup of expired waiver"
          );
        } else {
          this.activeWaivers.delete(ruleId);
        }
        cleanedCount++;
      }
    }

    return cleanedCount;
  }

  /**
   * Get waiver statistics
   */
  public getStatistics(): {
    totalWaivers: number;
    activeWaivers: number;
    approvedCount: number;
    rejectedCount: number;
    revokedCount: number;
    averageDuration: number;
  } {
    const active = this.getActiveWaivers();
    const approved = this.waiverHistory.filter(
      (w) => w.status === WaiverStatus.APPROVED
    );
    const rejected = this.waiverHistory.filter(
      (w) => w.status === WaiverStatus.REJECTED
    );
    const revoked = this.waiverHistory.filter(
      (w) => w.status === WaiverStatus.REVOKED
    );

    const durations = approved
      .filter((w) => w.approvedDuration)
      .map((w) => w.approvedDuration!);

    const averageDuration =
      durations.length > 0
        ? durations.reduce((a, b) => a + b, 0) / durations.length
        : 0;

    return {
      totalWaivers: this.waiverHistory.length,
      activeWaivers: active.length,
      approvedCount: approved.length,
      rejectedCount: rejected.length,
      revokedCount: revoked.length,
      averageDuration,
    };
  }

  /**
   * Format duration for display
   */
  private formatDuration(durationMs: number): string {
    const days = Math.floor(durationMs / (24 * 60 * 60 * 1000));
    const hours = Math.floor(
      (durationMs % (24 * 60 * 60 * 1000)) / (60 * 60 * 1000)
    );

    if (days > 0) {
      return `${days} day(s)${hours > 0 ? ` ${hours} hour(s)` : ""}`;
    }
    return `${hours} hour(s)`;
  }
}
