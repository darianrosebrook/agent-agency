/**
 * @fileoverview Confidence Scorer - ARBITER-024
 *
 * Scores arbitration decisions based on multiple factors including model reliability,
 * worker agreement, past accuracy, and CAWS compliance with configurable weights.
 *
 * @author @darianrosebrook
 */

export interface ConfidenceScoreFactors {
  verificationSuccessRate: number;
  claimEvidenceQuality: number;
  workerHistory: number;
  arbitrationWins: number;
  cawsCompliance: number;
  modelReliability?: number;
  sourceCredibility?: number;
}

export interface ConfidenceScore {
  overall: number;
  factors: ConfidenceScoreFactors;
  breakdown: {
    verification: number;
    evidence: number;
    history: number;
    arbitration: number;
    compliance: number;
  };
  reasoning: string[];
}

export interface ConfidenceScorerConfig {
  weights: {
    verificationSuccessRate: number; // 40% default
    claimEvidenceQuality: number; // 30% default
    workerHistory: number; // 20% default
    arbitrationWins: number; // 10% default
    cawsCompliance?: number; // 0% default (optional)
  };
  thresholds: {
    high: number; // 0.8 default
    medium: number; // 0.6 default
    low: number; // 0.4 default
  };
}

export interface ArbitrationContext {
  taskId: string;
  workerId: string;
  claimContent: string;
  verificationResults: Array<{
    method: string;
    success: boolean;
    confidence: number;
    evidence: any;
  }>;
  workerMetrics: {
    totalTasks: number;
    successfulTasks: number;
    arbitrationWins: number;
    arbitrationLosses: number;
    averageAccuracy: number;
    cawsViolations: number;
  };
  modelInfo?: {
    name: string;
    version: string;
    reliability: number;
  };
}

export class ConfidenceScorer {
  private readonly defaultConfig: ConfidenceScorerConfig = {
    weights: {
      verificationSuccessRate: 0.4,
      claimEvidenceQuality: 0.3,
      workerHistory: 0.2,
      arbitrationWins: 0.1,
      cawsCompliance: 0.0,
    },
    thresholds: {
      high: 0.8,
      medium: 0.6,
      low: 0.4,
    },
  };

  constructor(private config: Partial<ConfidenceScorerConfig> = {}) {
    this.config = { ...this.defaultConfig, ...config };
  }

  /**
   * Calculate confidence score for arbitration decision
   */
  async calculateConfidence(
    context: ArbitrationContext
  ): Promise<ConfidenceScore> {
    const factors = await this.calculateFactors(context);
    const breakdown = this.calculateBreakdown(factors);
    const overall = this.calculateOverallScore(factors);
    const reasoning = this.generateReasoning(factors, breakdown, overall);

    return {
      overall,
      factors,
      breakdown,
      reasoning,
    };
  }

  /**
   * Calculate individual factor scores
   */
  private async calculateFactors(
    context: ArbitrationContext
  ): Promise<ConfidenceScoreFactors> {
    const { verificationResults, workerMetrics, modelInfo } = context;

    // Verification success rate (40% weight)
    const verificationSuccessRate =
      this.calculateVerificationSuccessRate(verificationResults);

    // Claim evidence quality (30% weight)
    const claimEvidenceQuality = this.calculateClaimEvidenceQuality(
      verificationResults,
      context.claimContent
    );

    // Worker history (20% weight)
    const workerHistory = this.calculateWorkerHistoryScore(workerMetrics);

    // Arbitration wins (10% weight)
    const arbitrationWins = this.calculateArbitrationWinsScore(workerMetrics);

    // CAWS compliance (optional)
    const cawsCompliance = this.calculateCAWSComplianceScore(workerMetrics);

    // Model reliability (optional)
    const modelReliability = modelInfo?.reliability ?? 1.0;

    // Source credibility (optional)
    const sourceCredibility =
      this.calculateSourceCredibilityScore(verificationResults);

    return {
      verificationSuccessRate,
      claimEvidenceQuality,
      workerHistory,
      arbitrationWins,
      cawsCompliance,
      modelReliability,
      sourceCredibility,
    };
  }

  /**
   * Calculate weighted breakdown scores
   */
  private calculateBreakdown(
    factors: ConfidenceScoreFactors
  ): ConfidenceScore["breakdown"] {
    const weights = this.config.weights!;

    return {
      verification:
        factors.verificationSuccessRate * weights.verificationSuccessRate,
      evidence: factors.claimEvidenceQuality * weights.claimEvidenceQuality,
      history: factors.workerHistory * weights.workerHistory,
      arbitration: factors.arbitrationWins * weights.arbitrationWins,
      compliance: factors.cawsCompliance * (weights.cawsCompliance ?? 0),
    };
  }

  /**
   * Calculate overall confidence score
   */
  private calculateOverallScore(factors: ConfidenceScoreFactors): number {
    const breakdown = this.calculateBreakdown(factors);

    let totalWeight = 0;
    let weightedSum = 0;

    // Add weighted scores
    weightedSum += breakdown.verification;
    totalWeight += this.config.weights!.verificationSuccessRate;

    weightedSum += breakdown.evidence;
    totalWeight += this.config.weights!.claimEvidenceQuality;

    weightedSum += breakdown.history;
    totalWeight += this.config.weights!.workerHistory;

    weightedSum += breakdown.arbitration;
    totalWeight += this.config.weights!.arbitrationWins;

    if (this.config.weights!.cawsCompliance) {
      weightedSum += breakdown.compliance;
      totalWeight += this.config.weights!.cawsCompliance;
    }

    // Normalize by total weight
    return totalWeight > 0 ? weightedSum / totalWeight : 0;
  }

  /**
   * Calculate verification success rate
   */
  private calculateVerificationSuccessRate(
    verificationResults: ArbitrationContext["verificationResults"]
  ): number {
    if (verificationResults.length === 0) {
      return 0.5; // Neutral score for no verification
    }

    const successfulVerifications = verificationResults.filter(
      (r) => r.success
    ).length;
    return successfulVerifications / verificationResults.length;
  }

  /**
   * Calculate claim evidence quality score
   */
  private calculateClaimEvidenceQuality(
    verificationResults: ArbitrationContext["verificationResults"],
    claimContent: string
  ): number {
    if (verificationResults.length === 0) {
      return 0.3; // Low score for unverified claims
    }

    // Average confidence of successful verifications
    const successfulResults = verificationResults.filter((r) => r.success);
    if (successfulResults.length === 0) {
      return 0.2; // Very low score for failed verifications
    }

    const averageConfidence =
      successfulResults.reduce((sum, r) => sum + r.confidence, 0) /
      successfulResults.length;

    // Boost score based on evidence richness
    const evidenceRichness = this.assessEvidenceRichness(verificationResults);

    return Math.min(1.0, averageConfidence * evidenceRichness);
  }

  /**
   * Calculate worker history score
   */
  private calculateWorkerHistoryScore(
    workerMetrics: ArbitrationContext["workerMetrics"]
  ): number {
    if (workerMetrics.totalTasks === 0) {
      return 0.5; // Neutral score for new workers
    }

    const successRate =
      workerMetrics.successfulTasks / workerMetrics.totalTasks;
    const accuracyBonus = workerMetrics.averageAccuracy * 0.2; // 20% bonus for accuracy

    return Math.min(1.0, successRate + accuracyBonus);
  }

  /**
   * Calculate arbitration wins score
   */
  private calculateArbitrationWinsScore(
    workerMetrics: ArbitrationContext["workerMetrics"]
  ): number {
    const totalArbitrations =
      workerMetrics.arbitrationWins + workerMetrics.arbitrationLosses;

    if (totalArbitrations === 0) {
      return 0.5; // Neutral score for no arbitration history
    }

    return workerMetrics.arbitrationWins / totalArbitrations;
  }

  /**
   * Calculate CAWS compliance score
   */
  private calculateCAWSComplianceScore(
    workerMetrics: ArbitrationContext["workerMetrics"]
  ): number {
    const totalTasks = workerMetrics.totalTasks;

    if (totalTasks === 0) {
      return 1.0; // Perfect score for no tasks (no violations)
    }

    const violationRate = workerMetrics.cawsViolations / totalTasks;
    return Math.max(0, 1 - violationRate); // Penalize violations
  }

  /**
   * Calculate source credibility score
   */
  private calculateSourceCredibilityScore(
    verificationResults: ArbitrationContext["verificationResults"]
  ): number {
    if (verificationResults.length === 0) {
      return 0.5; // Neutral score
    }

    // Extract credibility indicators from verification results
    const credibilityScores = verificationResults.map((result) => {
      if (result.evidence?.credibility) {
        return result.evidence.credibility;
      }
      // Default credibility based on verification method
      return this.getDefaultCredibilityForMethod(result.method);
    });

    return (
      credibilityScores.reduce((sum, score) => sum + score, 0) /
      credibilityScores.length
    );
  }

  /**
   * Assess evidence richness
   */
  private assessEvidenceRichness(
    verificationResults: ArbitrationContext["verificationResults"]
  ): number {
    let richnessScore = 0;

    for (const result of verificationResults) {
      if (result.success && result.evidence) {
        // Check for different types of evidence
        if (result.evidence.sources) richnessScore += 0.2;
        if (result.evidence.citations) richnessScore += 0.2;
        if (result.evidence.calculations) richnessScore += 0.2;
        if (result.evidence.data) richnessScore += 0.2;
        if (result.evidence.references) richnessScore += 0.2;
      }
    }

    return Math.min(1.0, richnessScore);
  }

  /**
   * Get default credibility for verification method
   */
  private getDefaultCredibilityForMethod(method: string): number {
    const methodCredibility: Record<string, number> = {
      fact_checking: 0.8,
      source_credibility: 0.9,
      cross_reference: 0.7,
      math_verification: 0.95,
      code_verification: 0.9,
      context_verification: 0.6,
      consistency_check: 0.7,
      logical_validation: 0.8,
      statistical_validation: 0.85,
    };

    return methodCredibility[method] ?? 0.5;
  }

  /**
   * Generate human-readable reasoning
   */
  private generateReasoning(
    factors: ConfidenceScoreFactors,
    breakdown: ConfidenceScore["breakdown"],
    overall: number
  ): string[] {
    const reasoning: string[] = [];

    // Overall confidence level
    if (overall >= this.config.thresholds!.high) {
      reasoning.push(
        `High confidence (${(overall * 100).toFixed(
          1
        )}%) based on strong verification and worker history`
      );
    } else if (overall >= this.config.thresholds!.medium) {
      reasoning.push(
        `Medium confidence (${(overall * 100).toFixed(
          1
        )}%) with some uncertainty factors`
      );
    } else if (overall >= this.config.thresholds!.low) {
      reasoning.push(
        `Low confidence (${(overall * 100).toFixed(
          1
        )}%) due to verification issues or worker concerns`
      );
    } else {
      reasoning.push(
        `Very low confidence (${(overall * 100).toFixed(
          1
        )}%) - significant reliability concerns`
      );
    }

    // Factor-specific reasoning
    if (factors.verificationSuccessRate < 0.5) {
      reasoning.push("Verification success rate is below 50%");
    }
    if (factors.claimEvidenceQuality < 0.4) {
      reasoning.push("Claim evidence quality is insufficient");
    }
    if (factors.workerHistory < 0.6) {
      reasoning.push("Worker has concerning task history");
    }
    if (factors.arbitrationWins < 0.4) {
      reasoning.push("Worker has low arbitration success rate");
    }
    if (factors.cawsCompliance < 0.8) {
      reasoning.push("CAWS compliance issues detected");
    }

    // Positive factors
    if (factors.verificationSuccessRate > 0.8) {
      reasoning.push("Strong verification success rate");
    }
    if (factors.claimEvidenceQuality > 0.8) {
      reasoning.push("High-quality evidence provided");
    }
    if (factors.workerHistory > 0.8) {
      reasoning.push("Excellent worker performance history");
    }

    return reasoning;
  }

  /**
   * Get confidence level category
   */
  getConfidenceLevel(
    score: number
  ): "very_low" | "low" | "medium" | "high" | "very_high" {
    if (score >= 0.9) return "very_high";
    if (score >= this.config.thresholds!.high) return "high";
    if (score >= this.config.thresholds!.medium) return "medium";
    if (score >= this.config.thresholds!.low) return "low";
    return "very_low";
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig: Partial<ConfidenceScorerConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Get current configuration
   */
  getConfig(): ConfidenceScorerConfig {
    return this.config as ConfidenceScorerConfig;
  }
}

