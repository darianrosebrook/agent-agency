/**
 * Verdict Quality Scorer for RL Training
 *
 * @author @darianrosebrook
 * @module verdict-quality-scorer
 *
 * Evaluates the quality of arbitration verdicts for reinforcement learning training.
 * Uses ModelBasedJudge for multi-dimensional quality assessment.
 */
// @ts-nocheck


import { ModelBasedJudge } from "../evaluation/ModelBasedJudge";
import type { Verdict } from "../types/arbitration";
import type { JudgmentInput } from "../types/judge";

/**
 * Verdict quality evaluation result.
 */
export interface VerdictQualityEvaluation {
  /**
   * Overall quality score (0-1).
   */
  overallScore: number;

  /**
   * Detailed scores by criterion.
   */
  criteriaScores: {
    /**
     * Reasoning clarity and logic (0-1).
     */
    reasoningClarity: number;

    /**
     * Evidence quality and relevance (0-1).
     */
    evidenceQuality: number;

    /**
     * Constitutional compliance (0-1).
     */
    constitutionalCompliance: number;

    /**
     * Fairness and impartiality (0-1).
     */
    fairness: number;

    /**
     * Actionability and clarity (0-1).
     */
    actionability: number;
  };

  /**
   * Confidence in the quality assessment (0-1).
   */
  confidence: number;

  /**
   * Detailed feedback for each criterion.
   */
  feedback: {
    criterion: string;
    score: number;
    reasoning: string;
  }[];

  /**
   * Overall assessment reasoning.
   */
  assessmentReasoning: string;

  /**
   * Evaluation timestamp.
   */
  timestamp: string;
}

/**
 * Configuration for verdict quality scoring.
 */
export interface VerdictQualityScorerConfig {
  /**
   * Weights for different quality criteria.
   */
  criteriaWeights: {
    reasoningClarity: number;
    evidenceQuality: number;
    constitutionalCompliance: number;
    fairness: number;
    actionability: number;
  };

  /**
   * Minimum confidence threshold for scores.
   */
  minConfidenceThreshold: number;

  /**
   * Whether to enable detailed feedback generation.
   */
  enableDetailedFeedback: boolean;

  /**
   * Judge model configuration.
   */
  judgeConfig: {
    model: string;
    temperature: number;
    maxTokens: number;
  };
}

/**
 * Default configuration.
 */
const DEFAULT_CONFIG: VerdictQualityScorerConfig = {
  criteriaWeights: {
    reasoningClarity: 0.25,
    evidenceQuality: 0.25,
    constitutionalCompliance: 0.25,
    fairness: 0.15,
    actionability: 0.1,
  },
  minConfidenceThreshold: 0.7,
  enableDetailedFeedback: true,
  judgeConfig: {
    model: "gpt-4",
    temperature: 0.2, // Lower temperature for more consistent scoring
    maxTokens: 1500,
  },
};

/**
 * Verdict Quality Scorer for RL training integration.
 *
 * This component evaluates the quality of arbitration verdicts using
 * multi-dimensional criteria, providing training signals for reinforcement learning.
 */
export class VerdictQualityScorer {
  private config: VerdictQualityScorerConfig;
  private judge: ModelBasedJudge;

  /**
   * Creates a new verdict quality scorer.
   *
   * @param config - Scorer configuration. Uses defaults if not provided.
   * @param judge - Optional model-based judge instance.
   */
  constructor(
    config: Partial<VerdictQualityScorerConfig> = {},
    judge?: ModelBasedJudge
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.judge = judge || new ModelBasedJudge();
  }

  /**
   * Evaluates the quality of a verdict.
   *
   * @param verdict - Verdict to evaluate
   * @param context - Additional context for evaluation
   * @returns Quality evaluation result
   */
  async evaluateVerdict(
    verdict: Verdict,
    context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<VerdictQualityEvaluation> {
    // Evaluate each quality criterion
    const criteriaScores = await this.evaluateCriteria(verdict, context);

    // Calculate overall score using weighted average
    const overallScore = this.calculateWeightedScore(criteriaScores);

    // Calculate confidence based on judge confidence levels
    const confidence = this.calculateConfidence(criteriaScores);

    // Generate detailed feedback if enabled
    const feedback = this.config.enableDetailedFeedback
      ? await this.generateDetailedFeedback(verdict, criteriaScores, context)
      : [];

    // Generate overall assessment reasoning
    const assessmentReasoning = this.generateAssessmentReasoning(
      criteriaScores,
      overallScore
    );

    return {
      overallScore,
      criteriaScores,
      confidence,
      feedback,
      assessmentReasoning,
      timestamp: new Date().toISOString(),
    };
  }

  /**
   * Evaluates verdicts in batch.
   *
   * @param verdicts - Array of verdicts to evaluate
   * @param contexts - Optional array of contexts (one per verdict)
   * @returns Array of quality evaluations
   */
  async evaluateVerdictsBatch(
    verdicts: Verdict[],
    contexts?: Array<{
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }>
  ): Promise<VerdictQualityEvaluation[]> {
    const evaluations: VerdictQualityEvaluation[] = [];

    for (let i = 0; i < verdicts.length; i++) {
      const verdict = verdicts[i];
      const context = contexts?.[i];

      const evaluation = await this.evaluateVerdict(verdict, context);
      evaluations.push(evaluation);
    }

    return evaluations;
  }

  /**
   * Gets current configuration.
   *
   * @returns Current configuration
   */
  getConfig(): VerdictQualityScorerConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<VerdictQualityScorerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Evaluates all quality criteria for a verdict.
   */
  private async evaluateCriteria(
    verdict: Verdict,
    context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<VerdictQualityEvaluation["criteriaScores"]> {
    // Evaluate reasoning clarity
    const reasoningClarity = await this.evaluateReasoningClarity(
      verdict,
      context
    );

    // Evaluate evidence quality
    const evidenceQuality = await this.evaluateEvidenceQuality(
      verdict,
      context
    );

    // Evaluate constitutional compliance
    const constitutionalCompliance =
      await this.evaluateConstitutionalCompliance(verdict, context);

    // Evaluate fairness
    const fairness = await this.evaluateFairness(verdict, context);

    // Evaluate actionability
    const actionability = await this.evaluateActionability(verdict, context);

    return {
      reasoningClarity,
      evidenceQuality,
      constitutionalCompliance,
      fairness,
      actionability,
    };
  }

  /**
   * Evaluates reasoning clarity of the verdict.
   */
  private async evaluateReasoningClarity(
    verdict: Verdict,
    context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<number> {
    const judgmentInput: JudgmentInput = {
      task: `Evaluate the reasoning clarity, completeness, and coherence of this arbitration verdict for: ${
        context?.violation || "a constitutional rule violation"
      }`,
      output: verdict.reasoning.join("\n"),
      expectedOutput:
        "Clear, logical reasoning that addresses all relevant points with internal consistency",
      context: {
        verdictId: verdict.id,
        outcome: verdict.outcome,
        rules: context?.rules || [],
        evaluationFocus: "reasoning_clarity",
      },
    };

    const result = await this.judge.evaluate(judgmentInput);

    // Return the overall score from ModelBasedJudge
    return result.overallScore;
  }

  /**
   * Evaluates evidence quality referenced in the verdict.
   */
  private async evaluateEvidenceQuality(
    verdict: Verdict,
    context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<number> {
    const judgmentInput: JudgmentInput = {
      task: `Evaluate the quality, relevance, sufficiency, and credibility of evidence used in this arbitration verdict`,
      output: JSON.stringify({
        reasoning: verdict.reasoning,
        rulesApplied: verdict.rulesApplied || [],
        arguments: context?.arguments || [],
      }),
      expectedOutput:
        "Well-sourced, relevant, and sufficient evidence supporting the verdict conclusion",
      context: {
        verdictId: verdict.id,
        evidenceCount: verdict.rulesApplied?.length || 0,
        evaluationFocus: "evidence_quality",
      },
    };

    const result = await this.judge.evaluate(judgmentInput);

    return result.overallScore;
  }

  /**
   * Evaluates constitutional compliance of the verdict.
   */
  private async evaluateConstitutionalCompliance(
    verdict: Verdict,
    context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<number> {
    const judgmentInput: JudgmentInput = {
      task: `Evaluate constitutional compliance, rule alignment, precedent consistency, and policy adherence of this arbitration verdict`,
      output: JSON.stringify({
        outcome: verdict.outcome,
        reasoning: verdict.reasoning,
        rulesApplied: verdict.rulesApplied,
        confidence: verdict.confidence,
      }),
      expectedOutput:
        "Verdict fully aligns with CAWS constitutional rules, established precedents, and stated policies",
      context: {
        verdictId: verdict.id,
        rules: context?.rules || [],
        evaluationFocus: "constitutional_compliance",
      },
    };

    const result = await this.judge.evaluate(judgmentInput);

    return result.overallScore;
  }

  /**
   * Evaluates fairness and impartiality of the verdict.
   */
  private async evaluateFairness(
    verdict: Verdict,
    context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<number> {
    const judgmentInput: JudgmentInput = {
      task: `Evaluate the fairness, impartiality, proportionality, and balanced consideration of all arguments in this arbitration verdict`,
      output: JSON.stringify({
        outcome: verdict.outcome,
        reasoning: verdict.reasoning,
        rulesApplied: verdict.rulesApplied,
        arguments: context?.arguments || [],
      }),
      expectedOutput:
        "Impartial verdict with proportional conclusions and fair consideration of all arguments and evidence",
      context: {
        verdictId: verdict.id,
        evaluationFocus: "fairness",
      },
    };

    const result = await this.judge.evaluate(judgmentInput);

    return result.overallScore;
  }

  /**
   * Evaluates actionability and clarity of the verdict.
   */
  private async evaluateActionability(
    verdict: Verdict,
    _context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<number> {
    const judgmentInput: JudgmentInput = {
      task: `Evaluate the specificity, measurability, and feasibility of this arbitration verdict's conclusions and recommendations`,
      output: JSON.stringify({
        outcome: verdict.outcome,
        reasoning: verdict.reasoning,
        rulesApplied: verdict.rulesApplied,
      }),
      expectedOutput:
        "Clear, specific, measurable, and achievable verdict conclusions",
      context: {
        verdictId: verdict.id,
        evaluationFocus: "actionability",
      },
    };

    const result = await this.judge.evaluate(judgmentInput);

    return result.overallScore;
  }

  /**
   * Calculates weighted overall score from criteria scores.
   */
  private calculateWeightedScore(
    criteriaScores: VerdictQualityEvaluation["criteriaScores"]
  ): number {
    const weights = this.config.criteriaWeights;

    return (
      criteriaScores.reasoningClarity * weights.reasoningClarity +
      criteriaScores.evidenceQuality * weights.evidenceQuality +
      criteriaScores.constitutionalCompliance *
        weights.constitutionalCompliance +
      criteriaScores.fairness * weights.fairness +
      criteriaScores.actionability * weights.actionability
    );
  }

  /**
   * Calculates confidence in the quality assessment.
   */
  private calculateConfidence(
    criteriaScores: VerdictQualityEvaluation["criteriaScores"]
  ): number {
    // Calculate variance in criteria scores
    const scores = Object.values(criteriaScores);
    const mean = scores.reduce((sum, s) => sum + s, 0) / scores.length;
    const variance =
      scores.reduce((sum, s) => sum + Math.pow(s - mean, 2), 0) / scores.length;

    // Lower variance = higher confidence
    // High variance suggests inconsistent quality across criteria
    const varianceConfidence = 1 - Math.min(1, variance * 2);

    // Also consider if scores are extreme (very high or very low)
    const extremeScores = scores.filter((s) => s < 0.3 || s > 0.9).length;
    const extremeConfidence = 1 - extremeScores / (scores.length * 2);

    // Combined confidence
    return varianceConfidence * 0.7 + extremeConfidence * 0.3;
  }

  /**
   * Generates detailed feedback for each criterion.
   */
  private async generateDetailedFeedback(
    verdict: Verdict,
    criteriaScores: VerdictQualityEvaluation["criteriaScores"],
    _context?: {
      violation?: string;
      arguments?: Array<{ content: string; agentId: string }>;
      rules?: Array<{ id: string; text: string }>;
    }
  ): Promise<VerdictQualityEvaluation["feedback"]> {
    const feedback: VerdictQualityEvaluation["feedback"] = [];

    // Generate feedback for each criterion
    for (const [criterion, score] of Object.entries(criteriaScores)) {
      const reasoning = this.generateCriterionFeedback(
        criterion,
        score,
        verdict
      );

      feedback.push({
        criterion,
        score,
        reasoning,
      });
    }

    return feedback;
  }

  /**
   * Generates feedback for a single criterion.
   */
  private generateCriterionFeedback(
    criterion: string,
    score: number,
    verdict: Verdict
  ): string {
    const performanceLevel =
      score >= 0.8
        ? "Excellent"
        : score >= 0.6
        ? "Good"
        : score >= 0.4
        ? "Fair"
        : "Needs improvement";

    const feedbackMap: Record<string, string> = {
      reasoningClarity: `${performanceLevel}: The verdict's reasoning is ${
        score >= 0.6 ? "clear and logical" : "unclear or difficult to follow"
      }. Reasoning contains ${verdict.reasoning.length} points with ${
        score >= 0.6 ? "strong" : "weak"
      } logical flow.`,
      evidenceQuality: `${performanceLevel}: Evidence quality is ${
        score >= 0.6 ? "sufficient and relevant" : "lacking or irrelevant"
      }. ${verdict.rulesApplied?.length || 0} rules applied ${
        score >= 0.6
          ? "with appropriate justification"
          : "without sufficient support"
      }.`,
      constitutionalCompliance: `${performanceLevel}: Constitutional compliance is ${
        score >= 0.6 ? "strong" : "weak"
      }. Verdict ${
        score >= 0.6 ? "aligns well" : "may not align"
      } with CAWS principles (confidence: ${(verdict.confidence * 100).toFixed(
        0
      )}%).`,
      fairness: `${performanceLevel}: Fairness and impartiality ${
        score >= 0.6 ? "are well-demonstrated" : "could be improved"
      }. Verdict conclusions ${
        score >= 0.6 ? "are proportional" : "may be disproportionate"
      }.`,
      actionability: `${performanceLevel}: Verdict conclusions are ${
        score >= 0.6 ? "clear and actionable" : "vague or unclear"
      }. ${verdict.reasoning.length} reasoning points specified with ${
        score >= 0.6 ? "good" : "poor"
      } clarity.`,
    };

    return (
      feedbackMap[criterion] ||
      `${performanceLevel}: Score of ${score.toFixed(2)} for ${criterion}.`
    );
  }

  /**
   * Generates overall assessment reasoning.
   */
  private generateAssessmentReasoning(
    criteriaScores: VerdictQualityEvaluation["criteriaScores"],
    overallScore: number
  ): string {
    const performanceLevel =
      overallScore >= 0.8
        ? "excellent"
        : overallScore >= 0.6
        ? "good"
        : overallScore >= 0.4
        ? "fair"
        : "poor";

    // Find strongest and weakest criteria
    const sortedCriteria = Object.entries(criteriaScores).sort(
      ([, a], [, b]) => b - a
    );
    const strongest = sortedCriteria[0][0];
    const weakest = sortedCriteria[sortedCriteria.length - 1][0];

    return (
      `This verdict demonstrates ${performanceLevel} overall quality (score: ${overallScore.toFixed(
        2
      )}). ` +
      `Strongest aspect: ${strongest} (${sortedCriteria[0][1].toFixed(2)}). ` +
      `Area for improvement: ${weakest} (${sortedCriteria[
        sortedCriteria.length - 1
      ][1].toFixed(2)}). ` +
      `${
        overallScore >= 0.7
          ? "This verdict is suitable for RL training."
          : "This verdict may require review before use in RL training."
      }`
    );
  }
}
