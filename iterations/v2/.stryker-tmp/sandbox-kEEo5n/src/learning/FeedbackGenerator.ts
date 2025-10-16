/**
 * Feedback Generator
 *
 * Generates structured, actionable feedback with confidence scoring
 * to guide iterative improvements in multi-turn learning.
 *
 * Target: <200ms P95 generation time
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { randomUUID } from "crypto";
import { EventEmitter } from "events";
import type {
  FeedbackRecommendation,
  IterationFeedback,
  LearningIteration,
} from "../types/learning-coordination.js";
import {
  FeedbackType,
  LearningCoordinatorEvent,
  RecommendationPriority,
} from "../types/learning-coordination.js";

/**
 * Feedback context for generation
 */
export interface FeedbackContext {
  currentIteration: LearningIteration;
  previousIterations: LearningIteration[];
  qualityThreshold: number;
  errorPatterns: string[];
}

/**
 * Feedback Generator
 *
 * Produces actionable feedback with specific recommendations,
 * confidence scores, and tracked success patterns.
 */
export class FeedbackGenerator extends EventEmitter {
  private feedbackHistory: Map<string, IterationFeedback[]>;

  constructor() {
    super();
    this.feedbackHistory = new Map();
  }

  /**
   * Generate feedback for iteration
   *
   * @param context - Feedback generation context
   * @returns Iteration feedback with recommendations
   */
  async generateFeedback(context: FeedbackContext): Promise<IterationFeedback> {
    const startTime = Date.now();
    const { currentIteration, previousIterations, qualityThreshold } = context;

    const recommendations: FeedbackRecommendation[] = [];
    const successPatterns: string[] = [];
    const failurePatterns: string[] = [];

    // Analyze quality trend
    if (currentIteration.qualityScore < qualityThreshold) {
      const qualityRecommendations = this.generateQualityRecommendations(
        currentIteration,
        previousIterations,
        qualityThreshold
      );
      recommendations.push(...qualityRecommendations);
    }

    // Analyze errors if detected
    if (currentIteration.errorDetected && currentIteration.errorCategory) {
      const errorRecommendations =
        this.generateErrorRecommendations(currentIteration);
      recommendations.push(...errorRecommendations);
      failurePatterns.push(currentIteration.errorCategory);
    }

    // Analyze improvement trajectory
    if (previousIterations.length > 0) {
      const performanceRecommendations =
        this.generatePerformanceRecommendations(
          currentIteration,
          previousIterations
        );
      recommendations.push(...performanceRecommendations);
    }

    // Identify success patterns
    const recentSuccesses = previousIterations
      .slice(-3)
      .filter((it) => it.improvementDelta > 0.02);

    if (recentSuccesses.length > 0) {
      successPatterns.push(
        ...recentSuccesses.map(
          (it) =>
            `Quality improved by ${(it.improvementDelta * 100).toFixed(1)}%`
        )
      );
    }

    // Determine feedback type
    const feedbackType = this.determineFeedbackType(
      currentIteration,
      recommendations
    );

    // Calculate confidence based on data quality
    const confidence = this.calculateConfidence(
      currentIteration,
      previousIterations
    );

    const feedback: IterationFeedback = {
      feedbackId: randomUUID(),
      iterationId: currentIteration.iterationId,
      type: feedbackType,
      confidence,
      recommendations,
      successPatterns,
      failurePatterns,
      generatedAt: new Date(),
    };

    // Store feedback history
    const sessionFeedback =
      this.feedbackHistory.get(currentIteration.sessionId) || [];
    sessionFeedback.push(feedback);
    this.feedbackHistory.set(currentIteration.sessionId, sessionFeedback);

    // Emit event
    const generationTime = Date.now() - startTime;
    this.emit(LearningCoordinatorEvent.FEEDBACK_GENERATED, {
      sessionId: currentIteration.sessionId,
      timestamp: new Date(),
      eventType: LearningCoordinatorEvent.FEEDBACK_GENERATED,
      data: {
        feedbackId: feedback.feedbackId,
        type: feedbackType,
        recommendationCount: recommendations.length,
        confidence,
        generationTimeMs: generationTime,
      },
    });

    return feedback;
  }

  /**
   * Generate quality improvement recommendations
   *
   * @param current - Current iteration
   * @param previous - Previous iterations
   * @param threshold - Quality threshold
   * @returns Array of recommendations
   */
  private generateQualityRecommendations(
    current: LearningIteration,
    previous: LearningIteration[],
    threshold: number
  ): FeedbackRecommendation[] {
    const recommendations: FeedbackRecommendation[] = [];
    const gap = threshold - current.qualityScore;
    const gapPercent = (gap * 100).toFixed(1);

    // Gap analysis recommendation
    recommendations.push({
      priority:
        gap > 0.2
          ? RecommendationPriority.CRITICAL
          : RecommendationPriority.HIGH,
      action: `Improve quality by ${gapPercent}% to reach threshold`,
      rationale: `Current score ${(current.qualityScore * 100).toFixed(
        1
      )}% is ${gapPercent}% below target ${(threshold * 100).toFixed(1)}%`,
      expectedImpact: gap,
    });

    // Trend analysis
    if (previous.length >= 2) {
      const recentScores = previous.slice(-3).map((it) => it.qualityScore);
      const trend = this.calculateTrend(recentScores);

      if (trend < -0.01) {
        recommendations.push({
          priority: RecommendationPriority.CRITICAL,
          action: "Address quality degradation immediately",
          rationale: `Quality declining by ${(Math.abs(trend) * 100).toFixed(
            1
          )}% per iteration`,
          expectedImpact: 0.15,
        });
      } else if (trend < 0.005) {
        recommendations.push({
          priority: RecommendationPriority.HIGH,
          action: "Accelerate improvement rate",
          rationale: `Current improvement rate ${(trend * 100).toFixed(
            1
          )}% per iteration is too slow`,
          expectedImpact: 0.1,
        });
      }
    }

    return recommendations;
  }

  /**
   * Generate error-specific recommendations
   *
   * @param current - Current iteration with error
   * @returns Array of recommendations
   */
  private generateErrorRecommendations(
    current: LearningIteration
  ): FeedbackRecommendation[] {
    const recommendations: FeedbackRecommendation[] = [];
    const category = current.errorCategory;

    if (!category) {
      return recommendations;
    }

    // Category-specific guidance
    const errorGuidance: Record<
      string,
      { action: string; rationale: string; impact: number }
    > = {
      syntax_error: {
        action: "Review code syntax and structure carefully",
        rationale:
          "Syntax errors indicate basic structural issues that must be resolved first",
        impact: 0.2,
      },
      type_error: {
        action: "Add type checking and validation",
        rationale:
          "Type errors suggest missing guards or incorrect type assumptions",
        impact: 0.15,
      },
      runtime_error: {
        action: "Add error handling and defensive checks",
        rationale:
          "Runtime errors indicate missing null checks or improper state management",
        impact: 0.15,
      },
      validation_error: {
        action: "Verify input data format and constraints",
        rationale: "Validation failures suggest mismatched data expectations",
        impact: 0.1,
      },
      timeout_error: {
        action: "Optimize performance or increase timeout limits",
        rationale:
          "Timeout errors indicate slow operations or insufficient time limits",
        impact: 0.1,
      },
      resource_error: {
        action: "Reduce resource usage or optimize memory allocation",
        rationale:
          "Resource errors suggest memory leaks or excessive resource consumption",
        impact: 0.12,
      },
      dependency_error: {
        action: "Verify dependencies are properly installed and configured",
        rationale:
          "Dependency errors indicate missing or misconfigured packages",
        impact: 0.08,
      },
      configuration_error: {
        action: "Review configuration settings and environment variables",
        rationale:
          "Configuration errors suggest incorrect setup or missing settings",
        impact: 0.08,
      },
      unknown: {
        action: "Analyze error details and add comprehensive logging",
        rationale:
          "Unknown errors require detailed investigation to identify root cause",
        impact: 0.05,
      },
    };

    const guidance = errorGuidance[category] || errorGuidance.unknown;

    recommendations.push({
      priority: RecommendationPriority.CRITICAL,
      action: guidance.action,
      rationale: guidance.rationale,
      expectedImpact: guidance.impact,
    });

    return recommendations;
  }

  /**
   * Generate performance-based recommendations
   *
   * @param current - Current iteration
   * @param previous - Previous iterations
   * @returns Array of recommendations
   */
  private generatePerformanceRecommendations(
    current: LearningIteration,
    previous: LearningIteration[]
  ): FeedbackRecommendation[] {
    const recommendations: FeedbackRecommendation[] = [];

    // Resource usage analysis
    if (current.resourceUsageMB > 50) {
      recommendations.push({
        priority: RecommendationPriority.MEDIUM,
        action: "Optimize memory usage to reduce resource consumption",
        rationale: `Current usage ${current.resourceUsageMB.toFixed(
          1
        )}MB exceeds recommended limits`,
        expectedImpact: 0.05,
      });
    }

    // Iteration duration analysis
    if (current.durationMs > 10000) {
      recommendations.push({
        priority: RecommendationPriority.MEDIUM,
        action: "Improve iteration performance",
        rationale: `Iteration took ${(current.durationMs / 1000).toFixed(
          1
        )}s, consider optimization`,
        expectedImpact: 0.03,
      });
    }

    // Stagnation detection
    if (previous.length >= 3) {
      const recentImprovements = previous
        .slice(-3)
        .map((it) => it.improvementDelta);
      const avgImprovement =
        recentImprovements.reduce((a, b) => a + b, 0) /
        recentImprovements.length;

      if (Math.abs(avgImprovement) < 0.005) {
        recommendations.push({
          priority: RecommendationPriority.HIGH,
          action: "Try alternative approaches to break stagnation",
          rationale:
            "Minimal progress in last 3 iterations suggests current approach may be suboptimal",
          expectedImpact: 0.15,
        });
      }
    }

    return recommendations;
  }

  /**
   * Determine primary feedback type
   *
   * @param iteration - Current iteration
   * @param recommendations - Generated recommendations
   * @returns Feedback type
   */
  private determineFeedbackType(
    iteration: LearningIteration,
    recommendations: FeedbackRecommendation[]
  ): FeedbackType {
    if (iteration.errorDetected) {
      return FeedbackType.ERROR_CORRECTION;
    }

    if (iteration.improvementDelta < 0) {
      return FeedbackType.PERFORMANCE_IMPROVEMENT;
    }

    const hasCritical = recommendations.some(
      (r) => r.priority === RecommendationPriority.CRITICAL
    );
    if (hasCritical) {
      return FeedbackType.ERROR_CORRECTION;
    }

    if (iteration.improvementDelta > 0.02) {
      return FeedbackType.PATTERN_RECOGNITION;
    }

    return FeedbackType.QUALITY_ENHANCEMENT;
  }

  /**
   * Calculate feedback confidence score
   *
   * @param current - Current iteration
   * @param previous - Previous iterations
   * @returns Confidence score 0-1
   */
  private calculateConfidence(
    current: LearningIteration,
    previous: LearningIteration[]
  ): number {
    let confidence = 0.5; // Base confidence

    // Increase confidence with more history
    if (previous.length >= 3) {
      confidence += 0.2;
    }

    // Increase confidence for clear error signals
    if (current.errorDetected) {
      confidence += 0.15;
    }

    // Increase confidence for significant changes
    if (Math.abs(current.improvementDelta) > 0.05) {
      confidence += 0.15;
    }

    // Decrease confidence for inconsistent patterns
    if (previous.length >= 3) {
      const recentDeltas = previous.slice(-3).map((it) => it.improvementDelta);
      const variance = this.calculateVariance(recentDeltas);

      if (variance > 0.01) {
        confidence -= 0.1;
      }
    }

    return Math.max(0, Math.min(1, confidence));
  }

  /**
   * Calculate trend from values
   *
   * @param values - Array of values
   * @returns Trend (positive = improving)
   */
  private calculateTrend(values: number[]): number {
    if (values.length < 2) {
      return 0;
    }

    let sum = 0;
    for (let i = 1; i < values.length; i++) {
      sum += values[i] - values[i - 1];
    }

    return sum / (values.length - 1);
  }

  /**
   * Calculate variance of values
   *
   * @param values - Array of values
   * @returns Variance
   */
  private calculateVariance(values: number[]): number {
    if (values.length === 0) {
      return 0;
    }

    const mean = values.reduce((a, b) => a + b, 0) / values.length;
    const squaredDiffs = values.map((v) => Math.pow(v - mean, 2));

    return squaredDiffs.reduce((a, b) => a + b, 0) / values.length;
  }

  /**
   * Get feedback history for session
   *
   * @param sessionId - Session ID
   * @returns Array of feedback
   */
  getFeedbackHistory(sessionId: string): IterationFeedback[] {
    return this.feedbackHistory.get(sessionId) || [];
  }

  /**
   * Get feedback statistics
   *
   * @param sessionId - Session ID
   * @returns Feedback statistics
   */
  getStatistics(sessionId: string): {
    totalFeedback: number;
    averageConfidence: number;
    totalRecommendations: number;
    recommendationsByPriority: Record<string, number>;
  } {
    const feedback = this.feedbackHistory.get(sessionId) || [];

    if (feedback.length === 0) {
      return {
        totalFeedback: 0,
        averageConfidence: 0,
        totalRecommendations: 0,
        recommendationsByPriority: {},
      };
    }

    const totalConfidence = feedback.reduce((sum, f) => sum + f.confidence, 0);
    const totalRecommendations = feedback.reduce(
      (sum, f) => sum + f.recommendations.length,
      0
    );

    const recommendationsByPriority: Record<string, number> = {};
    for (const fb of feedback) {
      for (const rec of fb.recommendations) {
        recommendationsByPriority[rec.priority] =
          (recommendationsByPriority[rec.priority] || 0) + 1;
      }
    }

    return {
      totalFeedback: feedback.length,
      averageConfidence: totalConfidence / feedback.length,
      totalRecommendations,
      recommendationsByPriority,
    };
  }

  /**
   * Clean up session data
   *
   * @param sessionId - Session ID
   */
  cleanup(sessionId: string): void {
    this.feedbackHistory.delete(sessionId);
  }
}
