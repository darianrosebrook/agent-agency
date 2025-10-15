/**
 * @fileoverview
 * Main evaluator for code diff minimality assessment.
 * Combines AST analysis and scaffolding detection for reward calculation.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import type {
  CodeDiff,
  MinimalityEvaluation,
  MinimalityEvaluatorConfig,
} from "@/types/evaluation";
import { DEFAULT_EVALUATOR_CONFIG } from "@/types/evaluation";
import { ASTDiffAnalyzer } from "./ASTDiffAnalyzer";
import { ScaffoldingDetector } from "./ScaffoldingDetector";

/**
 * Evaluates code diffs for minimality and quality
 */
export class MinimalDiffEvaluator {
  private astAnalyzer: ASTDiffAnalyzer;
  private scaffoldingDetector: ScaffoldingDetector;
  private config: MinimalityEvaluatorConfig;

  /**
   * Creates a new MinimalDiffEvaluator
   *
   * @param config Optional configuration
   */
  constructor(config?: Partial<MinimalityEvaluatorConfig>) {
    this.config = { ...DEFAULT_EVALUATOR_CONFIG, ...config };
    this.astAnalyzer = new ASTDiffAnalyzer();
    this.scaffoldingDetector = new ScaffoldingDetector();
  }

  /**
   * Evaluates a code diff for minimality
   *
   * @param diff Code diff to evaluate
   * @returns Minimality evaluation result
   */
  evaluate(diff: CodeDiff): MinimalityEvaluation {
    const startTime = Date.now();

    // Perform AST analysis
    const astDiff = this.astAnalyzer.analyze(diff);

    // Detect scaffolding
    const scaffolding = this.config.enableScaffoldingDetection
      ? this.scaffoldingDetector.detect(diff)
      : {
          detected: false,
          confidence: 0,
          reasons: [],
          penaltyFactor: 0,
          matchedPatterns: [],
        };

    // Calculate lines changed
    const linesChanged = this.astAnalyzer.calculateLinesChanged(diff);

    // Calculate base minimality from AST similarity
    const baseMinimality = this.calculateBaseMinimality(
      astDiff.similarity,
      linesChanged
    );

    // Apply scaffolding penalty
    const minimalityFactor = this.applyScaffoldingPenalty(
      baseMinimality,
      scaffolding.penaltyFactor
    );

    // Assess overall quality
    const qualityAssessment = this.assessQuality(minimalityFactor);

    const evaluationTimeMs = Date.now() - startTime;

    return {
      minimalityFactor,
      astSimilarity: astDiff.similarity,
      scaffolding,
      linesChanged,
      qualityAssessment,
      evaluationTimeMs,
    };
  }

  /**
   * Calculates base minimality score from AST similarity and lines changed
   *
   * @param similarity AST similarity score
   * @param linesChanged Lines of code changed
   * @returns Base minimality score (0.1-1.0)
   */
  private calculateBaseMinimality(
    similarity: number,
    linesChanged: number
  ): number {
    // High similarity = minimal changes = high score
    const similarityScore = similarity;

    // Fewer lines changed = higher score
    const linesScore = this.calculateLinesScore(linesChanged);

    // Weighted average: similarity is more important
    const rawScore = similarityScore * 0.7 + linesScore * 0.3;

    // Clamp to [0.1, 1.0]
    return Math.max(
      this.config.minMinimalityFactor,
      Math.min(this.config.maxMinimalityFactor, rawScore)
    );
  }

  /**
   * Calculates score based on lines changed
   *
   * @param linesChanged Number of lines changed
   * @returns Score (0-1)
   */
  private calculateLinesScore(linesChanged: number): number {
    // Score decreases as lines changed increases
    // < 10 lines = 1.0
    // 10-50 lines = 0.8-1.0
    // 50-100 lines = 0.5-0.8
    // 100-500 lines = 0.3-0.5
    // > 500 lines = 0.1-0.3

    if (linesChanged < 10) {
      return 1.0;
    } else if (linesChanged < 50) {
      return 0.8 + (0.2 * (50 - linesChanged)) / 40;
    } else if (linesChanged < 100) {
      return 0.5 + (0.3 * (100 - linesChanged)) / 50;
    } else if (linesChanged < 500) {
      return 0.3 + (0.2 * (500 - linesChanged)) / 400;
    } else {
      return 0.1 + (0.2 * Math.min(1000 - linesChanged, 500)) / 500;
    }
  }

  /**
   * Applies scaffolding penalty to minimality score
   *
   * @param baseMinimality Base minimality score
   * @param penaltyFactor Scaffolding penalty factor
   * @returns Adjusted minimality score
   */
  private applyScaffoldingPenalty(
    baseMinimality: number,
    penaltyFactor: number
  ): number {
    // Apply penalty as multiplicative factor
    const penalty = penaltyFactor * this.config.scaffoldingPenaltyWeight;
    const adjustedScore = baseMinimality * (1 - penalty);

    // Ensure we stay within bounds
    return Math.max(
      this.config.minMinimalityFactor,
      Math.min(this.config.maxMinimalityFactor, adjustedScore)
    );
  }

  /**
   * Assesses overall quality based on minimality factor
   *
   * @param minimalityFactor Minimality factor
   * @returns Quality assessment
   */
  private assessQuality(
    minimalityFactor: number
  ): "minimal" | "moderate" | "extensive" {
    if (minimalityFactor >= 0.8) {
      return "minimal";
    } else if (minimalityFactor >= 0.5) {
      return "moderate";
    } else {
      return "extensive";
    }
  }

  /**
   * Gets current configuration
   *
   * @returns Current config
   */
  getConfig(): MinimalityEvaluatorConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration
   *
   * @param config New configuration
   */
  updateConfig(config: Partial<MinimalityEvaluatorConfig>): void {
    this.config = { ...this.config, ...config };
  }
}
