/**
 * @fileoverview
 * Main LLM-as-judge implementation for subjective quality evaluation.
 * Coordinates LLM provider, confidence scoring, and multi-criteria assessment.
 *
 * @author @darianrosebrook
 */

import type {
  CriterionAssessment,
  EvaluationCriterion,
  JudgeConfig,
  JudgeMetrics,
  JudgmentInput,
  JudgmentResult,
} from "@/types/judge";
import { DEFAULT_JUDGE_CONFIG } from "@/types/judge";
import { validateLLMConfig } from "@/utils/llm-config";
import { ConfidenceScorer } from "./ConfidenceScorer";
import {
  AnthropicProvider,
  LLMProvider,
  MockLLMProvider,
  OllamaProvider,
  OpenAIProvider,
} from "./LLMProvider";

/**
 * LLM-based judge for subjective evaluation
 */
export class ModelBasedJudge {
  private config: JudgeConfig;
  private llmProvider: LLMProvider;
  private confidenceScorer: ConfidenceScorer;
  private metrics: JudgeMetrics;

  /**
   * Creates a new ModelBasedJudge
   *
   * @param config Optional configuration
   * @param llmProvider Optional custom LLM provider (e.g., ModelRegistryLLMProvider)
   */
  constructor(config?: Partial<JudgeConfig>, llmProvider?: LLMProvider) {
    this.config = { ...DEFAULT_JUDGE_CONFIG, ...config };
    this.llmProvider = llmProvider || this.createLLMProvider();
    this.confidenceScorer = new ConfidenceScorer();
    this.metrics = this.initializeMetrics();
  }

  /**
   * Evaluates input across all criteria
   *
   * @param input Judgment input
   * @returns Complete judgment result
   */
  async evaluate(input: JudgmentInput): Promise<JudgmentResult> {
    const startTime = Date.now();

    // Validate input
    this.validateInput(input);

    const claimConfidence = this.extractClaimConfidence(input);

    // Evaluate each criterion
    const criteria: EvaluationCriterion[] = [
      "faithfulness" as EvaluationCriterion,
      "relevance" as EvaluationCriterion,
      "minimality" as EvaluationCriterion,
      "safety" as EvaluationCriterion,
    ];

    const assessments: CriterionAssessment[] = [];

    for (const criterion of criteria) {
      try {
        const assessment = await this.evaluateCriterion(
          input,
          criterion,
          claimConfidence
        );
        assessments.push(assessment);
      } catch (error) {
        // Fallback to safe default if enabled
        if (this.config.enableFallback) {
          assessments.push(this.createFallbackAssessment(criterion));
          this.metrics.fallbackRate =
            (this.metrics.fallbackRate * this.metrics.totalJudgments + 1) /
            (this.metrics.totalJudgments + 1);
        } else {
          throw error;
        }
      }
    }

    // Calculate overall scores
    const overallScore =
      assessments.reduce((sum, a) => sum + a.score, 0) / assessments.length;
    let overallConfidence =
      assessments.reduce((sum, a) => sum + a.confidence, 0) /
      assessments.length;

    if (claimConfidence !== null) {
      overallConfidence = this.blendConfidence(
        overallConfidence,
        claimConfidence
      );
    }
    const allCriteriaPass = assessments.every((a) => a.passes);

    const evaluationTimeMs = Date.now() - startTime;

    // Update metrics
    this.updateMetrics(assessments, evaluationTimeMs);

    return {
      assessments,
      overallScore,
      overallConfidence,
      claimConfidence: claimConfidence ?? undefined,
      claimEvaluation: input.claimEvaluation,
      allCriteriaPass,
      evaluatedAt: new Date(),
      evaluationTimeMs,
    };
  }

  /**
   * Evaluates a single criterion
   *
   * @param input Judgment input
   * @param criterion Criterion to evaluate
   * @returns Criterion assessment
   */
  private async evaluateCriterion(
    input: JudgmentInput,
    criterion: EvaluationCriterion,
    claimConfidence: number | null
  ): Promise<CriterionAssessment> {
    // Get LLM response
    const response = await this.llmProvider.evaluate(input, criterion);

    // Calculate confidence
    let confidence = this.confidenceScorer.calculateConfidence(response);
    let reasoning = response.reasoning;

    if (claimConfidence !== null) {
      confidence = this.blendConfidence(confidence, claimConfidence);
      reasoning = `${reasoning}\nClaim evaluation confidence: ${Math.round(
        claimConfidence * 100
      )}%`;
    }

    // Check if passes threshold
    const threshold = this.config.thresholds[criterion];
    const passes = response.score >= threshold;

    return {
      criterion,
      score: response.score,
      confidence,
      reasoning,
      passes,
    };
  }

  /**
   * Creates fallback assessment for failed evaluation
   *
   * @param criterion Criterion that failed
   * @returns Fallback assessment
   */
  private createFallbackAssessment(
    criterion: EvaluationCriterion
  ): CriterionAssessment {
    return {
      criterion,
      score: 0.5,
      confidence: 0.3,
      reasoning: "Fallback assessment due to evaluation failure",
      passes: false,
    };
  }

  private extractClaimConfidence(input: JudgmentInput): number | null {
    const evaluation = input.claimEvaluation;
    if (!evaluation) {
      return null;
    }

    const components = [
      evaluation.overallQuality,
      evaluation.factualAccuracyScore,
      evaluation.cawsComplianceScore,
    ].filter(
      (value): value is number =>
        typeof value === "number" && Number.isFinite(value)
    );

    if (components.length === 0) {
      return null;
    }

    const average =
      components.reduce((sum, value) => sum + value, 0) / components.length;

    return Math.min(1, Math.max(0, average));
  }

  private blendConfidence(primary: number, claimConfidence: number): number {
    return Math.min(1, Math.max(0, primary * 0.6 + claimConfidence * 0.4));
  }

  /**
   * Validates judgment input
   *
   * @param input Input to validate
   * @throws Error if validation fails
   */
  private validateInput(input: JudgmentInput): void {
    if (!input.task || input.task.trim().length === 0) {
      throw new Error("Task is required");
    }

    if (!input.output || input.output.trim().length === 0) {
      throw new Error("Output is required");
    }
  }

  /**
   * Creates LLM provider based on configuration
   *
   * @returns LLM provider instance
   */
  private createLLMProvider(): LLMProvider {
    const provider = this.config.llm.provider;

    // Validate configuration for the chosen provider
    if (!validateLLMConfig(this.config.llm)) {
      console.warn(
        `LLM configuration validation failed for provider "${provider}", falling back to mock provider.`
      );
      return new MockLLMProvider(this.config.llm);
    }

    // Create provider based on configuration
    switch (provider) {
      case "ollama":
        return new OllamaProvider(this.config.llm);
      case "openai":
        return new OpenAIProvider(this.config.llm);
      case "anthropic":
        return new AnthropicProvider(this.config.llm);
      case "model-registry":
        // Use Ollama as local-first provider for model registry
        // TODO: Implement intelligent LLM provider selection and orchestration
        // - Add provider capability matching based on task requirements
        // - Implement cost-benefit analysis for provider selection
        // - Support provider ensemble and voting mechanisms
        // - Add provider performance history and adaptive selection
        // - Implement provider fallback and degradation strategies
        // - Support multi-provider orchestration for complex tasks
        // - Add provider-specific parameter optimization
        // - Implement provider usage analytics and optimization
        console.info(
          "Using Ollama provider for model registry (local-first approach)"
        );
        return new OllamaProvider(this.config.llm);
      case "mock":
        return new MockLLMProvider(this.config.llm);
      default:
        console.warn(
          `Unknown LLM provider "${provider}", using mock provider. Supported providers: ollama, openai, anthropic, mock, model-registry`
        );
        return new MockLLMProvider(this.config.llm);
    }
  }

  /**
   * Initializes metrics to default values
   *
   * @returns Initial metrics
   */
  private initializeMetrics(): JudgeMetrics {
    return {
      totalJudgments: 0,
      judgmentsByCriterion: {
        faithfulness: 0,
        relevance: 0,
        minimality: 0,
        safety: 0,
      },
      averageEvaluationTimeMs: 0,
      cacheHitRate: 0,
      fallbackRate: 0,
      averageConfidence: 0,
    };
  }

  /**
   * Updates metrics after an evaluation
   *
   * @param assessments Criterion assessments
   * @param evaluationTimeMs Evaluation duration
   */
  private updateMetrics(
    assessments: CriterionAssessment[],
    evaluationTimeMs: number
  ): void {
    // Increment total judgments
    this.metrics.totalJudgments++;

    // Update criterion counts
    assessments.forEach((assessment) => {
      this.metrics.judgmentsByCriterion[assessment.criterion]++;
    });

    // Update running average of evaluation time
    const totalTime =
      this.metrics.averageEvaluationTimeMs * (this.metrics.totalJudgments - 1) +
      evaluationTimeMs;
    this.metrics.averageEvaluationTimeMs =
      totalTime / this.metrics.totalJudgments;

    // Update running average of confidence
    const avgConfidence =
      assessments.reduce((sum, a) => sum + a.confidence, 0) /
      assessments.length;
    const totalConfidence =
      this.metrics.averageConfidence * (this.metrics.totalJudgments - 1) +
      avgConfidence;
    this.metrics.averageConfidence =
      totalConfidence / this.metrics.totalJudgments;
  }

  /**
   * Gets current metrics
   *
   * @returns Current metrics snapshot
   */
  getMetrics(): JudgeMetrics {
    return { ...this.metrics };
  }

  /**
   * Resets metrics to initial state
   */
  resetMetrics(): void {
    this.metrics = this.initializeMetrics();
  }

  /**
   * Gets current configuration
   *
   * @returns Current config
   */
  getConfig(): JudgeConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration
   *
   * @param config New configuration
   */
  updateConfig(config: Partial<JudgeConfig>): void {
    this.config = { ...this.config, ...config };
    // Recreate provider if LLM config changed
    if (config.llm) {
      this.llmProvider = this.createLLMProvider();
    }
  }
}
