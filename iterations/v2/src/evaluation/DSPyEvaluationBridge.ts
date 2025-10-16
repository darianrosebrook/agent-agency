/**
 * Bridge between existing evaluation framework and DSPy service
 *
 * Integrates DSPy-powered rubric optimization and model judges
 * with the existing V2 evaluation infrastructure.
 *
 * @author @darianrosebrook
 */

import {
  DSPyClient,
  JudgeEvaluationRequest,
  RubricOptimizationRequest,
} from "@/dspy-integration";
import { Logger } from "@/observability/Logger";
import type { JudgmentInput } from "@/types/judge";
import { ModelBasedJudge } from "./ModelBasedJudge";

/**
 * Configuration for DSPy evaluation bridge
 */
export interface DSPyEvaluationBridgeConfig {
  /** DSPy service URL */
  dspyServiceUrl: string;
  /** Whether DSPy enhancement is enabled */
  enabled: boolean;
  /** Fallback to existing evaluation if DSPy fails */
  fallbackOnError: boolean;
}

/**
 * Rubric evaluation request
 */
export interface RubricEvaluationRequest {
  taskContext: string;
  agentOutput: string;
  evaluationCriteria: string;
}

/**
 * Rubric evaluation result
 */
export interface RubricEvaluationResult {
  score: number;
  reasoning: string;
  suggestions: string[];
  metadata: {
    enhanced: boolean;
    dspyUsed: boolean;
    [key: string]: unknown;
  };
}

/**
 * Bridge for integrating DSPy with existing evaluation framework
 *
 * Provides seamless integration between:
 * - Existing V2 evaluation components
 * - New DSPy-powered optimization
 */
export class DSPyEvaluationBridge {
  private readonly dspyClient: DSPyClient;
  private readonly config: DSPyEvaluationBridgeConfig;
  private readonly logger: Logger;
  private readonly existingJudge: ModelBasedJudge;

  constructor(
    config: DSPyEvaluationBridgeConfig,
    existingJudge: ModelBasedJudge
  ) {
    this.config = config;
    this.existingJudge = existingJudge;

    this.dspyClient = new DSPyClient({
      baseUrl: config.dspyServiceUrl,
      timeout: 30000,
      maxRetries: 3,
    });

    this.logger = new Logger("DSPyEvaluationBridge");
  }

  /**
   * Evaluate rubric using DSPy-enhanced computation
   *
   * Falls back to existing evaluation if DSPy is disabled or fails.
   *
   * @param request - Rubric evaluation request
   * @returns Enhanced evaluation result
   */
  async evaluateRubric(
    request: RubricEvaluationRequest
  ): Promise<RubricEvaluationResult> {
    // If DSPy is disabled, use existing evaluation
    if (!this.config.enabled) {
      return this.evaluateRubricLegacy(request);
    }

    try {
      // Check DSPy service health
      await this.dspyClient.health();

      // Use DSPy-enhanced evaluation
      const dspyRequest: RubricOptimizationRequest = {
        taskContext: request.taskContext,
        agentOutput: request.agentOutput,
        evaluationCriteria: request.evaluationCriteria,
      };

      const dspyResult = await this.dspyClient.optimizeRubric(dspyRequest);

      this.logger.info("DSPy rubric evaluation successful", {
        score: dspyResult.rewardScore,
      });

      return {
        score: dspyResult.rewardScore,
        reasoning: dspyResult.reasoning,
        suggestions: this.parseSuggestions(dspyResult.improvementSuggestions),
        metadata: {
          enhanced: true,
          dspyUsed: true,
          ...dspyResult.metadata,
        },
      };
    } catch (error) {
      this.logger.warn("DSPy evaluation failed, falling back", {
        error: error instanceof Error ? error.message : String(error),
      });

      if (this.config.fallbackOnError) {
        return this.evaluateRubricLegacy(request);
      }

      throw error;
    }
  }

  /**
   * Evaluate using model judge with DSPy enhancement
   *
   * @param judgeType - Type of judgment to perform
   * @param artifact - Output to evaluate
   * @param groundTruth - Expected output
   * @param context - Task context
   * @returns Judge evaluation result
   */
  async evaluateWithJudge(
    judgeType: "relevance" | "faithfulness" | "minimality" | "safety",
    artifact: string,
    groundTruth: string,
    context: string
  ): Promise<{
    judgment: string;
    confidence: number;
    reasoning: string;
    metadata: { dspyEnhanced: boolean; [key: string]: unknown };
  }> {
    // If DSPy is disabled, use existing judge
    if (!this.config.enabled) {
      return this.evaluateWithJudgeLegacy(
        judgeType,
        artifact,
        groundTruth,
        context
      );
    }

    try {
      // Check DSPy service health
      await this.dspyClient.health();

      // Use DSPy-enhanced judge
      const judgeRequest: JudgeEvaluationRequest = {
        judgeType,
        artifact,
        groundTruth,
        context,
      };

      const judgeResult = await this.dspyClient.evaluateWithJudge(judgeRequest);

      this.logger.info("DSPy judge evaluation successful", {
        judgeType,
        confidence: judgeResult.confidence,
      });

      return {
        judgment: judgeResult.judgment,
        confidence: judgeResult.confidence,
        reasoning: judgeResult.reasoning,
        metadata: {
          dspyEnhanced: true,
          ...judgeResult.metadata,
        },
      };
    } catch (error) {
      this.logger.warn("DSPy judge evaluation failed, falling back", {
        judgeType,
        error: error instanceof Error ? error.message : String(error),
      });

      if (this.config.fallbackOnError) {
        return this.evaluateWithJudgeLegacy(
          judgeType,
          artifact,
          groundTruth,
          context
        );
      }

      throw error;
    }
  }

  /**
   * Legacy rubric evaluation (existing implementation)
   *
   * @param request - Rubric evaluation request
   * @returns Evaluation result
   */
  private async evaluateRubricLegacy(
    request: RubricEvaluationRequest
  ): Promise<RubricEvaluationResult> {
    this.logger.info("Using integrated rubric evaluation with ModelBasedJudge");

    try {
      // Create a model-based judge for rubric evaluation
      const judge = new ModelBasedJudge({
        enableFallback: true,
        timeoutMs: 30000,
      });

      // Prepare judgment input
      const judgmentInput: JudgmentInput = {
        task: request.taskContext,
        output: request.agentOutput,
        context: {
          rubricEvaluation: true,
          timestamp: new Date().toISOString(),
        },
      };

      // Get judgment from the model-based judge
      const judgment = await judge.evaluate(judgmentInput);

      // Convert judgment result to rubric evaluation result
      const score = judgment.overallScore;
      const reasoning = "Evaluated using model-based judge";

      // Extract suggestions from criterion assessments
      const suggestions: string[] = [];
      for (const assessment of judgment.assessments || []) {
        if (assessment.reasoning) {
          suggestions.push(`${assessment.criterion}: ${assessment.reasoning}`);
        }
      }

      return {
        score,
        reasoning,
        suggestions,
        metadata: {
          enhanced: false,
          dspyUsed: false,
          judgeUsed: true,
          confidence: judgment.overallConfidence,
          criteriaEvaluated: judgment.assessments?.length || 0,
        },
      };
    } catch (error) {
      this.logger.error(
        "Rubric evaluation failed, falling back to basic scoring",
        { error }
      );

      // Fallback to basic keyword-based scoring
      const score = this.calculateBasicRubricScore(request);
      const reasoning = "Basic keyword-based evaluation (judge failed)";

      return {
        score,
        reasoning,
        suggestions: ["Consider using more specific evaluation criteria"],
        metadata: {
          enhanced: false,
          dspyUsed: false,
          fallbackUsed: true,
        },
      };
    }
  }

  /**
   * Calculate basic rubric score using keyword matching
   */
  private calculateBasicRubricScore(request: RubricEvaluationRequest): number {
    const { taskContext, agentOutput, evaluationCriteria } = request;

    // Simple keyword-based scoring
    const criteriaWords = evaluationCriteria.toLowerCase().split(/\s+/);
    const outputWords = agentOutput.toLowerCase().split(/\s+/);
    const contextWords = taskContext.toLowerCase().split(/\s+/);

    let matches = 0;
    let totalCriteria = 0;

    for (const criterion of criteriaWords) {
      if (criterion.length > 3) {
        // Ignore short words
        totalCriteria++;
        if (
          outputWords.includes(criterion) ||
          contextWords.includes(criterion)
        ) {
          matches++;
        }
      }
    }

    // Return score between 0.3 and 0.9 based on matches
    const baseScore = totalCriteria > 0 ? matches / totalCriteria : 0.5;
    return Math.max(0.3, Math.min(0.9, baseScore));
  }

  /**
   * Legacy judge evaluation (existing implementation)
   *
   * @param judgeType - Type of judgment
   * @param artifact - Output to evaluate
   * @param groundTruth - Expected output
   * @param context - Task context
   * @returns Judge evaluation result
   */
  private async evaluateWithJudgeLegacy(
    judgeType: string,
    artifact: string,
    groundTruth: string,
    context: string
  ): Promise<{
    judgment: string;
    confidence: number;
    reasoning: string;
    metadata: { dspyEnhanced: boolean; [key: string]: unknown };
  }> {
    // Use existing ModelBasedJudge
    const result = await this.existingJudge.evaluate({
      task: judgeType,
      output: artifact,
      expectedOutput: groundTruth,
      context: { text: context },
    });

    return {
      judgment: result.allCriteriaPass ? "PASS" : "FAIL",
      confidence: result.overallConfidence,
      reasoning: result.assessments
        .map((a) => `${a.criterion}: ${a.reasoning}`)
        .join("; "),
      metadata: {
        dspyEnhanced: false,
        overallScore: result.overallScore,
        assessments: result.assessments,
      },
    };
  }

  /**
   * Parse improvement suggestions from DSPy response
   *
   * @param suggestions - Raw suggestions string
   * @returns Array of parsed suggestions
   */
  private parseSuggestions(suggestions: string): string[] {
    // Split by newlines and filter empty lines
    return suggestions
      .split("\n")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }
}
