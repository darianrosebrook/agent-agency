/**
 * Verdict Generator
 *
 * @author @darianrosebrook
 *
 * Generates comprehensive verdicts for constitutional arbitration decisions.
 * Produces complete reasoning chains with evidence references, rule citations,
 * and precedent application.
 *
 * Features:
 * - Multi-step reasoning chain generation
 * - Evidence validation and citation
 * - Precedent application and citation
 * - Confidence scoring
 * - Audit trail creation
 * - Conditional verdict support
 */

import {
  ArbitrationError,
  ArbitrationSession,
  ConstitutionalRule,
  ReasoningStep,
  Verdict,
  VerdictOutcome,
} from "@/types/arbitration";

/**
 * Verdict generation configuration
 */
export interface VerdictGeneratorConfig {
  /** Minimum confidence for approval (0-1) */
  minConfidenceForApproval: number;

  /** Require precedent citations */
  requirePrecedents: boolean;

  /** Minimum reasoning steps */
  minReasoningSteps: number;

  /** Maximum verdict generation time (ms) */
  maxGenerationTime: number;

  /** Enable conditional verdicts */
  allowConditional: boolean;
}

/**
 * Verdict generation result
 */
export interface VerdictGenerationResult {
  /** Generated verdict */
  verdict: Verdict;

  /** Generation time in ms */
  generationTimeMs: number;

  /** Warnings (if any) */
  warnings: string[];
}

/**
 * VerdictGenerator - Generates comprehensive constitutional verdicts
 */
export class VerdictGenerator {
  /** Configuration */
  private config: VerdictGeneratorConfig;

  /** Verdict counter for ID generation */
  private verdictCounter: number = 0;

  constructor(config?: Partial<VerdictGeneratorConfig>) {
    this.config = {
      minConfidenceForApproval: 0.7,
      requirePrecedents: true,
      minReasoningSteps: 3,
      maxGenerationTime: 150,
      allowConditional: true,
      ...config,
    };
  }

  /**
   * Generate a verdict for an arbitration session
   */
  public async generateVerdict(
    session: ArbitrationSession,
    issuedBy: string
  ): Promise<VerdictGenerationResult> {
    const startTime = Date.now();
    const warnings: string[] = [];

    // Validate session
    this.validateSession(session);

    // Build reasoning chain
    const reasoning = await this.buildReasoningChain(session, warnings);

    // Determine outcome
    const outcome = this.determineOutcome(session, reasoning);

    // Calculate confidence
    const confidence = this.calculateConfidence(session, reasoning);

    // Generate conditions (if conditional)
    const conditions =
      outcome === VerdictOutcome.CONDITIONAL
        ? this.generateConditions(session)
        : undefined;

    // Create verdict
    const verdict: Verdict = {
      id: this.generateVerdictId(),
      sessionId: session.id,
      outcome,
      reasoning,
      rulesApplied: session.rulesEvaluated.map((r) => r.id),
      evidence: session.evidence,
      precedents: session.precedents.map((p) => p.id),
      conditions,
      confidence,
      issuedBy,
      issuedAt: new Date(),
      auditLog: [
        {
          timestamp: new Date(),
          action: "verdict_generated",
          actor: issuedBy,
          details: `Generated ${outcome} verdict with ${reasoning.length} reasoning steps`,
        },
      ],
    };

    // Check time budget
    const generationTimeMs = Date.now() - startTime;
    if (generationTimeMs > this.config.maxGenerationTime) {
      warnings.push(
        `Verdict generation took ${generationTimeMs}ms (exceeded ${this.config.maxGenerationTime}ms budget)`
      );
    }

    return {
      verdict,
      generationTimeMs,
      warnings,
    };
  }

  /**
   * Build complete reasoning chain
   */
  private async buildReasoningChain(
    session: ArbitrationSession,
    warnings: string[]
  ): Promise<ReasoningStep[]> {
    const steps: ReasoningStep[] = [];

    // Step 1: Violation Assessment
    steps.push(
      this.createReasoningStep(
        1,
        `Assessed constitutional violation: ${session.violation.description}`,
        session.evidence,
        [session.violation.ruleId],
        0.9
      )
    );

    // Step 2: Rule Application
    for (let i = 0; i < session.rulesEvaluated.length; i++) {
      const rule = session.rulesEvaluated[i];
      steps.push(
        this.createReasoningStep(
          steps.length + 1,
          `Applied constitutional rule ${rule.id}: ${rule.title}. Severity: ${session.violation.severity}`,
          this.getRelevantEvidence(rule, session.evidence),
          [rule.id],
          0.85
        )
      );
    }

    // Step 3: Precedent Analysis (if applicable)
    if (session.precedents.length > 0) {
      for (const precedent of session.precedents) {
        steps.push(
          this.createReasoningStep(
            steps.length + 1,
            `Analyzed precedent ${precedent.id}: ${precedent.title}. ${precedent.reasoningSummary}`,
            [],
            precedent.rulesInvolved,
            0.8
          )
        );
      }
    } else if (this.config.requirePrecedents) {
      warnings.push("No precedents applied (precedents required by config)");
    }

    // Step 4: Evidence Evaluation
    if (session.evidence.length > 0) {
      steps.push(
        this.createReasoningStep(
          steps.length + 1,
          `Evaluated ${session.evidence.length} pieces of evidence supporting the violation assessment`,
          session.evidence,
          [],
          0.85
        )
      );
    }

    // Step 5: Final Assessment
    steps.push(
      this.createReasoningStep(
        steps.length + 1,
        this.generateFinalAssessment(session),
        session.evidence,
        session.rulesEvaluated.map((r) => r.id),
        this.calculateOverallConfidence(steps)
      )
    );

    // Validate minimum steps
    if (steps.length < this.config.minReasoningSteps) {
      warnings.push(
        `Only ${steps.length} reasoning steps generated (minimum ${this.config.minReasoningSteps} recommended)`
      );
    }

    return steps;
  }

  /**
   * Create a reasoning step
   */
  private createReasoningStep(
    step: number,
    description: string,
    evidence: string[],
    ruleReferences: string[],
    confidence: number
  ): ReasoningStep {
    return {
      step,
      description,
      evidence,
      ruleReferences,
      confidence,
    };
  }

  /**
   * Determine verdict outcome
   */
  private determineOutcome(
    session: ArbitrationSession,
    reasoning: ReasoningStep[]
  ): VerdictOutcome {
    // Check for waiver
    if (session.waiverRequest) {
      return VerdictOutcome.WAIVED;
    }

    // Calculate confidence
    const confidence = this.calculateOverallConfidence(reasoning);

    // Low confidence -> conditional
    if (
      this.config.allowConditional &&
      confidence < this.config.minConfidenceForApproval
    ) {
      return VerdictOutcome.CONDITIONAL;
    }

    // Check violation severity and evidence
    const hasStrongEvidence = session.evidence.length >= 2;
    const isCriticalViolation = session.violation.severity === "critical";

    // Determine approval/rejection
    if (isCriticalViolation || !hasStrongEvidence) {
      return VerdictOutcome.REJECTED;
    }

    // High confidence + evidence -> approved
    if (
      confidence >= this.config.minConfidenceForApproval &&
      hasStrongEvidence
    ) {
      return VerdictOutcome.APPROVED;
    }

    // Default to conditional
    return VerdictOutcome.CONDITIONAL;
  }

  /**
   * Calculate verdict confidence
   */
  private calculateConfidence(
    session: ArbitrationSession,
    reasoning: ReasoningStep[]
  ): number {
    let confidence = this.calculateOverallConfidence(reasoning);

    // Boost for precedents
    if (session.precedents.length > 0) {
      confidence += session.precedents.length * 0.05;
    }

    // Boost for strong evidence
    if (session.evidence.length >= 3) {
      confidence += 0.1;
    }

    // Reduce for waiver requests
    if (session.waiverRequest) {
      confidence *= 0.9;
    }

    return Math.min(confidence, 1.0);
  }

  /**
   * Calculate overall confidence from reasoning steps
   */
  private calculateOverallConfidence(steps: ReasoningStep[]): number {
    if (steps.length === 0) {
      return 0;
    }

    const sum = steps.reduce((acc, step) => acc + step.confidence, 0);
    return sum / steps.length;
  }

  /**
   * Generate conditions for conditional verdict
   */
  private generateConditions(session: ArbitrationSession): string[] {
    const conditions: string[] = [];

    // Add conditions based on violation severity
    if (session.violation.severity === "major") {
      conditions.push("Must address violation within 48 hours");
      conditions.push("Must provide evidence of remediation");
    } else if (session.violation.severity === "moderate") {
      conditions.push("Must address violation within 1 week");
    }

    // Add conditions based on evidence
    if (session.evidence.length < 2) {
      conditions.push("Must provide additional supporting evidence");
    }

    // Add conditions based on rules
    for (const rule of session.rulesEvaluated) {
      if (rule.waivable) {
        conditions.push(`May request waiver for rule ${rule.id}`);
      }
    }

    return conditions;
  }

  /**
   * Generate final assessment text
   */
  private generateFinalAssessment(session: ArbitrationSession): string {
    const ruleCount = session.rulesEvaluated.length;
    const evidenceCount = session.evidence.length;
    const precedentCount = session.precedents.length;

    let assessment = `Final assessment based on ${ruleCount} constitutional rule(s), ${evidenceCount} evidence item(s)`;

    if (precedentCount > 0) {
      assessment += `, and ${precedentCount} precedent(s)`;
    }

    assessment += `. Violation severity: ${session.violation.severity}.`;

    return assessment;
  }

  /**
   * Get relevant evidence for a rule
   */
  private getRelevantEvidence(
    rule: ConstitutionalRule,
    allEvidence: string[]
  ): string[] {
    // Simplified: return all evidence
    // In production, would filter based on rule.requiredEvidence
    return allEvidence.slice(0, 2); // Return subset
  }

  /**
   * Validate session before generating verdict
   */
  private validateSession(session: ArbitrationSession): void {
    if (!session.id) {
      throw new ArbitrationError("Session must have ID", "INVALID_SESSION");
    }

    if (!session.violation) {
      throw new ArbitrationError(
        "Session must have violation",
        "NO_VIOLATION",
        session.id
      );
    }

    if (session.rulesEvaluated.length === 0) {
      throw new ArbitrationError(
        "Session must have evaluated rules",
        "NO_RULES",
        session.id
      );
    }
  }

  /**
   * Generate unique verdict ID
   */
  private generateVerdictId(): string {
    this.verdictCounter++;
    return `VERDICT-${Date.now()}-${this.verdictCounter}`;
  }

  /**
   * Add audit log entry to verdict
   */
  public addAuditEntry(
    verdict: Verdict,
    action: string,
    actor: string,
    details: string
  ): void {
    verdict.auditLog.push({
      timestamp: new Date(),
      action,
      actor,
      details,
    });
  }

  /**
   * Get verdict statistics
   */
  public getStatistics(): {
    totalVerdicts: number;
    averageReasoningSteps: number;
    averageConfidence: number;
  } {
    return {
      totalVerdicts: this.verdictCounter,
      averageReasoningSteps: 0, // Would track in production
      averageConfidence: 0, // Would track in production
    };
  }
}
