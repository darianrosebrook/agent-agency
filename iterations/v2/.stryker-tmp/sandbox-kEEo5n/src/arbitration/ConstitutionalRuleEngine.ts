/**
 * Constitutional Rule Engine
 *
 * @author @darianrosebrook
 *
 * Interprets and enforces CAWS constitutional rules with precedent-based consistency.
 * Evaluates agent actions against constitutional constraints and provides rule interpretation.
 *
 * Features:
 * - Constitutional rule loading and management
 * - Violation detection with evidence requirements
 * - Precedent-based rule interpretation
 * - Rule versioning and expiration
 * - Context-aware evaluation
 */
// @ts-nocheck


import {
  ArbitrationError,
  ConstitutionalRule,
  ConstitutionalViolation,
  Precedent,
  RuleCategory,
  RuleEngineConfig,
  ViolationSeverity,
} from "@/types/arbitration";

/**
 * Evaluation context for rule checking
 */
export interface EvaluationContext {
  /** Action being evaluated */
  action: string;

  /** Actor performing the action */
  actor: string;

  /** Action parameters */
  parameters: Record<string, any>;

  /** Environmental context */
  environment: Record<string, any>;

  /** Timestamp */
  timestamp: Date;
}

/**
 * Rule evaluation result
 */
export interface RuleEvaluationResult {
  /** Rule ID */
  ruleId: string;

  /** Whether rule was violated */
  violated: boolean;

  /** Violation details (if violated) */
  violation?: ConstitutionalViolation;

  /** Explanation of evaluation */
  explanation: string;

  /** Confidence in evaluation (0-1) */
  confidence: number;

  /** Applied precedents */
  precedentsApplied: string[];

  /** Evaluation time in ms */
  evaluationTimeMs: number;
}

/**
 * Constitutional Rule Engine
 */
export class ConstitutionalRuleEngine {
  /** Active constitutional rules */
  private rules: Map<string, ConstitutionalRule> = new Map();

  /** Precedents */
  private precedents: Map<string, Precedent> = new Map();

  /** Configuration */
  private config: RuleEngineConfig;

  /** Rule evaluation cache */
  private evaluationCache: Map<string, RuleEvaluationResult> = new Map();

  constructor(config?: Partial<RuleEngineConfig>) {
    this.config = {
      strictMode: true,
      usePrecedents: true,
      evaluationTimeoutMs: 200,
      requireEvidence: true,
      enableWaivers: true,
      ...config,
    };
  }

  /**
   * Load a constitutional rule
   */
  public loadRule(rule: ConstitutionalRule): void {
    // Validate rule
    if (!rule.id || !rule.version) {
      throw new ArbitrationError(
        "Rule must have ID and version",
        "INVALID_RULE"
      );
    }

    // Check expiration
    if (rule.expirationDate && rule.expirationDate < new Date()) {
      throw new ArbitrationError(
        `Rule ${rule.id} has expired`,
        "EXPIRED_RULE",
        undefined,
        rule.id
      );
    }

    // Store rule
    this.rules.set(rule.id, rule);

    // Clear cache for this rule
    this.clearCacheForRule(rule.id);
  }

  /**
   * Load multiple rules
   */
  public loadRules(rules: ConstitutionalRule[]): void {
    rules.forEach((rule) => this.loadRule(rule));
  }

  /**
   * Get a rule by ID
   */
  public getRule(ruleId: string): ConstitutionalRule | undefined {
    return this.rules.get(ruleId);
  }

  /**
   * Get rules by category
   */
  public getRulesByCategory(category: RuleCategory): ConstitutionalRule[] {
    return Array.from(this.rules.values()).filter(
      (rule) => rule.category === category
    );
  }

  /**
   * Get all active rules
   */
  public getAllRules(): ConstitutionalRule[] {
    return Array.from(this.rules.values());
  }

  /**
   * Load a precedent
   */
  public loadPrecedent(precedent: Precedent): void {
    this.precedents.set(precedent.id, precedent);
  }

  /**
   * Evaluate action against constitutional rules
   */
  public async evaluateAction(
    context: EvaluationContext,
    ruleIds?: string[]
  ): Promise<RuleEvaluationResult[]> {
    const startTime = Date.now();

    // Determine rules to evaluate
    const rulesToEvaluate = ruleIds
      ? ruleIds
          .map((id) => this.rules.get(id))
          .filter((r): r is ConstitutionalRule => r !== undefined)
      : Array.from(this.rules.values());

    if (rulesToEvaluate.length === 0) {
      return [];
    }

    // Evaluate each rule
    const results: RuleEvaluationResult[] = [];

    for (const rule of rulesToEvaluate) {
      // Check timeout
      if (Date.now() - startTime > this.config.evaluationTimeoutMs) {
        throw new ArbitrationError(
          "Rule evaluation timeout exceeded",
          "EVALUATION_TIMEOUT"
        );
      }

      const result = await this.evaluateRule(rule, context);
      results.push(result);
    }

    return results;
  }

  /**
   * Evaluate a single rule
   */
  private async evaluateRule(
    rule: ConstitutionalRule,
    context: EvaluationContext
  ): Promise<RuleEvaluationResult> {
    const startTime = Date.now();

    // Check cache
    const cacheKey = this.getCacheKey(rule.id, context);
    const cached = this.evaluationCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    // Check if rule has expired
    if (rule.expirationDate && rule.expirationDate < context.timestamp) {
      return {
        ruleId: rule.id,
        violated: false,
        explanation: "Rule has expired and is no longer enforced",
        confidence: 1.0,
        precedentsApplied: [],
        evaluationTimeMs: Date.now() - startTime,
      };
    }

    // Find applicable precedents
    const applicablePrecedents = this.findApplicablePrecedents(rule, context);

    // Evaluate rule condition
    const violated = this.evaluateCondition(
      rule,
      context,
      applicablePrecedents
    );

    // Build result
    const result: RuleEvaluationResult = {
      ruleId: rule.id,
      violated,
      explanation: this.generateExplanation(
        rule,
        context,
        violated,
        applicablePrecedents
      ),
      confidence: this.calculateConfidence(rule, applicablePrecedents),
      precedentsApplied: applicablePrecedents.map((p) => p.id),
      evaluationTimeMs: Date.now() - startTime,
    };

    // If violated, create violation record
    if (violated) {
      result.violation = this.createViolation(rule, context);
    }

    // Cache result (short-lived cache)
    this.evaluationCache.set(cacheKey, result);

    return result;
  }

  /**
   * Evaluate rule condition
   */
  private evaluateCondition(
    rule: ConstitutionalRule,
    context: EvaluationContext,
    precedents: Precedent[]
  ): boolean {
    // In strict mode, any missing required evidence is a violation
    if (this.config.strictMode && this.config.requireEvidence) {
      const hasRequiredEvidence = rule.requiredEvidence.every(
        (evidenceType) => context.parameters[`evidence_${evidenceType}`]
      );

      if (!hasRequiredEvidence) {
        return true; // Violated due to missing evidence
      }
    }

    // Apply precedent-based interpretation
    if (this.config.usePrecedents && precedents.length > 0) {
      return this.evaluateWithPrecedents(rule, context, precedents);
    }

    // Default rule evaluation logic
    return this.evaluateRuleLogic(rule, context);
  }

  /**
   * Evaluate with precedents
   */
  private evaluateWithPrecedents(
    rule: ConstitutionalRule,
    context: EvaluationContext,
    precedents: Precedent[]
  ): boolean {
    // Find most similar precedent
    const mostSimilar = precedents[0]; // Simplified: take first precedent

    // Apply precedent reasoning
    // In a real implementation, this would use ML/NLP to match context to precedent
    return mostSimilar.verdict.outcome === "rejected";
  }

  /**
   * Evaluate rule logic (simplified)
   */
  private evaluateRuleLogic(
    rule: ConstitutionalRule,
    context: EvaluationContext
  ): boolean {
    // Simplified rule evaluation
    // In production, this would parse and evaluate the rule.condition
    // For now, check some common patterns

    // Code quality rules
    if (rule.category === RuleCategory.CODE_QUALITY) {
      if (context.action === "commit" && !context.parameters.linted) {
        return true; // Violated
      }
    }

    // Testing rules
    if (rule.category === RuleCategory.TESTING) {
      const coverage = context.parameters.coverage as number | undefined;
      if (coverage !== undefined && coverage < 80) {
        return true; // Violated
      }
    }

    // Security rules
    if (rule.category === RuleCategory.SECURITY) {
      if (context.parameters.hasVulnerabilities) {
        return true; // Violated
      }
    }

    // Budget rules
    if (rule.category === RuleCategory.BUDGET) {
      const filesChanged = context.parameters.filesChanged as
        | number
        | undefined;
      const maxFiles = context.environment.maxFiles as number | undefined;
      if (
        filesChanged !== undefined &&
        maxFiles !== undefined &&
        filesChanged > maxFiles
      ) {
        return true; // Violated
      }
    }

    return false; // Not violated
  }

  /**
   * Find applicable precedents for a rule
   */
  private findApplicablePrecedents(
    rule: ConstitutionalRule,
    _context: EvaluationContext
  ): Precedent[] {
    if (!this.config.usePrecedents) {
      return [];
    }

    return Array.from(this.precedents.values())
      .filter((precedent) => {
        // Check if precedent involves this rule
        if (!precedent.rulesInvolved.includes(rule.id)) {
          return false;
        }

        // Check category match
        if (precedent.applicability.category !== rule.category) {
          return false;
        }

        // Check severity compatibility
        const severityOrder = [
          ViolationSeverity.MINOR,
          ViolationSeverity.MODERATE,
          ViolationSeverity.MAJOR,
          ViolationSeverity.CRITICAL,
        ];
        const precedentSeverityIndex = severityOrder.indexOf(
          precedent.applicability.severity
        );
        const ruleSeverityIndex = severityOrder.indexOf(rule.severity);

        // Precedent should be same or higher severity
        return precedentSeverityIndex >= ruleSeverityIndex;
      })
      .sort((a, b) => b.citationCount - a.citationCount) // Most cited first
      .slice(0, 3); // Top 3 precedents
  }

  /**
   * Generate explanation
   */
  private generateExplanation(
    rule: ConstitutionalRule,
    context: EvaluationContext,
    violated: boolean,
    precedents: Precedent[]
  ): string {
    let explanation = `Rule "${rule.title}" (${rule.id})`;

    if (violated) {
      explanation += ` was violated by action "${context.action}"`;
    } else {
      explanation += ` was not violated by action "${context.action}"`;
    }

    if (precedents.length > 0) {
      explanation += `. Applied ${precedents.length} precedent(s): ${precedents
        .map((p) => p.id)
        .join(", ")}`;
    }

    return explanation;
  }

  /**
   * Calculate confidence in evaluation
   */
  private calculateConfidence(
    rule: ConstitutionalRule,
    precedents: Precedent[]
  ): number {
    let confidence = 0.7; // Base confidence

    // Increase confidence with precedents
    if (precedents.length > 0) {
      confidence += precedents.length * 0.1;
    }

    // Increase confidence in strict mode
    if (this.config.strictMode) {
      confidence += 0.1;
    }

    return Math.min(confidence, 1.0);
  }

  /**
   * Create violation record
   */
  private createViolation(
    rule: ConstitutionalRule,
    context: EvaluationContext
  ): ConstitutionalViolation {
    return {
      id: `violation-${Date.now()}-${Math.random()
        .toString(36)
        .substring(2, 9)}`,
      ruleId: rule.id,
      severity: rule.severity,
      description: `Violation of rule "${rule.title}": ${rule.description}`,
      evidence: this.extractEvidence(context),
      location: this.extractLocation(context),
      detectedAt: context.timestamp,
      violator: context.actor,
      context: {
        action: context.action,
        parameters: context.parameters,
        environment: context.environment,
      },
    };
  }

  /**
   * Extract evidence from context
   */
  private extractEvidence(context: EvaluationContext): string[] {
    const evidence: string[] = [];

    // Extract evidence from parameters
    Object.entries(context.parameters).forEach(([key, value]) => {
      if (key.startsWith("evidence_")) {
        evidence.push(`${key}: ${JSON.stringify(value)}`);
      }
    });

    // Add action as evidence
    evidence.push(`Action: ${context.action}`);

    return evidence;
  }

  /**
   * Extract location from context
   */
  private extractLocation(
    context: EvaluationContext
  ): ConstitutionalViolation["location"] {
    if (context.parameters.file) {
      return {
        file: context.parameters.file as string,
        line: context.parameters.line as number | undefined,
        function: context.parameters.function as string | undefined,
      };
    }
    return undefined;
  }

  /**
   * Get cache key
   */
  private getCacheKey(ruleId: string, context: EvaluationContext): string {
    return `${ruleId}:${context.action}:${context.actor}:${JSON.stringify(
      context.parameters
    )}`;
  }

  /**
   * Clear cache for a specific rule
   */
  private clearCacheForRule(ruleId: string): void {
    const keysToDelete: string[] = [];

    for (const key of this.evaluationCache.keys()) {
      if (key.startsWith(`${ruleId}:`)) {
        keysToDelete.push(key);
      }
    }

    keysToDelete.forEach((key) => this.evaluationCache.delete(key));
  }

  /**
   * Clear all evaluation cache
   */
  public clearCache(): void {
    this.evaluationCache.clear();
  }

  /**
   * Get evaluation statistics
   */
  public getStatistics(): {
    totalRules: number;
    activeRules: number;
    totalPrecedents: number;
    cacheSize: number;
    rulesByCategory: Record<RuleCategory, number>;
  } {
    const rulesByCategory = Array.from(this.rules.values()).reduce(
      (counts, rule) => {
        counts[rule.category] = (counts[rule.category] || 0) + 1;
        return counts;
      },
      {} as Record<RuleCategory, number>
    );

    const activeRules = Array.from(this.rules.values()).filter(
      (rule) => !rule.expirationDate || rule.expirationDate > new Date()
    ).length;

    return {
      totalRules: this.rules.size,
      activeRules,
      totalPrecedents: this.precedents.size,
      cacheSize: this.evaluationCache.size,
      rulesByCategory,
    };
  }
}
