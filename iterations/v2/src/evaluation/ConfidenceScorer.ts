/**
 * @fileoverview
 * Confidence scoring utility for LLM-based judgments.
 * Calculates confidence based on response consistency and quality indicators.
 *
 * @author @darianrosebrook
 */

import type { LLMResponse } from "@/types/judge";

/**
 * Calculates confidence scores for LLM judgments
 */
export class ConfidenceScorer {
  /**
   * Calculates confidence from LLM response
   *
   * @param response LLM response to score
   * @returns Confidence score (0-1)
   */
  calculateConfidence(response: LLMResponse): number {
    // Use explicit confidence if provided
    if (response.confidence !== undefined && response.confidence >= 0) {
      return Math.min(1.0, Math.max(0.0, response.confidence));
    }

    // Fall back to heuristic-based confidence
    return this.calculateHeuristicConfidence(response);
  }

  /**
   * Calculates confidence using heuristics
   *
   * @param response LLM response
   * @returns Confidence score (0-1)
   */
  private calculateHeuristicConfidence(response: LLMResponse): number {
    let confidence = 0.5; // Base confidence

    // Factor 1: Reasoning length (longer is more confident)
    const reasoningLength = response.reasoning?.length ?? 0;
    if (reasoningLength > 100) {
      confidence += 0.2;
    } else if (reasoningLength > 50) {
      confidence += 0.1;
    }

    // Factor 2: Score extremity (more extreme scores suggest higher confidence)
    const scoreDistance = Math.abs(response.score - 0.5);
    confidence += scoreDistance * 0.3;

    // Factor 3: Reasoning quality indicators
    if (this.hasQualityIndicators(response.reasoning)) {
      confidence += 0.1;
    }

    return Math.min(1.0, Math.max(0.0, confidence));
  }

  /**
   * Checks for quality indicators in reasoning
   *
   * @param reasoning Reasoning text
   * @returns True if quality indicators present
   */
  private hasQualityIndicators(reasoning: string): boolean {
    if (!reasoning) {
      return false;
    }

    const qualityIndicators = [
      /because/i,
      /therefore/i,
      /specifically/i,
      /clearly/i,
      /demonstrates/i,
      /evidence/i,
    ];

    return qualityIndicators.some((pattern) => pattern.test(reasoning));
  }

  /**
   * Aggregates multiple confidence scores
   *
   * @param confidences Array of confidence scores
   * @returns Aggregated confidence (average)
   */
  aggregateConfidences(confidences: number[]): number {
    if (confidences.length === 0) {
      return 0;
    }

    const sum = confidences.reduce((acc, conf) => acc + conf, 0);
    return sum / confidences.length;
  }
}
