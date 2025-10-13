/**
 * @fileoverview
 * Analyzes task characteristics to determine complexity level for budget allocation.
 * Uses rule-based heuristics to categorize tasks as trivial, standard, or complex.
 *
 * @author @darianrosebrook
 */

import {
  ComplexityAssessment,
  ComplexityLevel,
  TaskCharacteristics,
} from "@/types/thinking-budget";

/**
 * Analyzes task characteristics to determine complexity level
 */
export class TaskComplexityAnalyzer {
  /**
   * Analyzes task characteristics and returns complexity assessment
   *
   * @param characteristics Task characteristics to analyze
   * @returns Complexity assessment with level, confidence, and reasoning
   */
  analyze(characteristics: TaskCharacteristics): ComplexityAssessment {
    const startTime = Date.now();

    const level = this.assessComplexity(characteristics);
    const confidence = this.calculateConfidence(characteristics, level);
    const reasoning = this.generateReasoning(characteristics, level);

    const assessmentTimeMs = Date.now() - startTime;

    return {
      level,
      confidence,
      reasoning,
      characteristics,
      assessmentTimeMs,
    };
  }

  /**
   * Assesses complexity level based on task characteristics
   *
   * @param characteristics Task characteristics
   * @returns Assessed complexity level
   */
  private assessComplexity(
    characteristics: TaskCharacteristics
  ): ComplexityLevel {
    const { toolCount, contextSize, stepCount, multiTurn, hasExternalCalls } =
      characteristics;

    // Complex tier: high tool usage, large context, many steps, or multi-turn
    if (
      toolCount >= 3 ||
      contextSize >= 5000 ||
      stepCount >= 5 ||
      (multiTurn && hasExternalCalls)
    ) {
      return ComplexityLevel.COMPLEX;
    }

    // Standard tier: moderate tool usage, context, and steps
    if (
      toolCount >= 1 ||
      contextSize >= 1000 ||
      stepCount >= 2 ||
      hasExternalCalls
    ) {
      return ComplexityLevel.STANDARD;
    }

    // Trivial tier: simple queries with minimal requirements
    return ComplexityLevel.TRIVIAL;
  }

  /**
   * Calculates confidence score for the assessment
   *
   * @param characteristics Task characteristics
   * @param level Assessed complexity level
   * @returns Confidence score between 0 and 1
   */
  private calculateConfidence(
    characteristics: TaskCharacteristics,
    level: ComplexityLevel
  ): number {
    const { toolCount, contextSize, stepCount } = characteristics;

    // Calculate how strongly characteristics align with the assessed level
    let confidence = 0.5; // Base confidence

    switch (level) {
      case ComplexityLevel.TRIVIAL:
        if (toolCount === 0 && contextSize < 500 && stepCount <= 1) {
          confidence = 0.95; // Strong alignment
        } else if (toolCount === 0 && contextSize < 1000) {
          confidence = 0.8; // Moderate alignment
        } else {
          confidence = 0.6; // Weak alignment (edge case)
        }
        break;

      case ComplexityLevel.STANDARD:
        if (
          toolCount >= 1 &&
          toolCount <= 2 &&
          contextSize >= 1000 &&
          contextSize < 5000
        ) {
          confidence = 0.9;
        } else if (toolCount >= 1 || contextSize >= 1000) {
          confidence = 0.75;
        } else {
          confidence = 0.65;
        }
        break;

      case ComplexityLevel.COMPLEX:
        if (toolCount >= 3 || contextSize >= 5000 || stepCount >= 5) {
          confidence = 0.95;
        } else {
          confidence = 0.7;
        }
        break;
    }

    return Math.min(1.0, Math.max(0.0, confidence));
  }

  /**
   * Generates human-readable reasoning for the assessment
   *
   * @param characteristics Task characteristics
   * @param level Assessed complexity level
   * @returns Reasoning string
   */
  private generateReasoning(
    characteristics: TaskCharacteristics,
    level: ComplexityLevel
  ): string {
    const reasons: string[] = [];

    if (characteristics.toolCount >= 3) {
      reasons.push(`high tool usage (${characteristics.toolCount} tools)`);
    } else if (characteristics.toolCount >= 1) {
      reasons.push(`moderate tool usage (${characteristics.toolCount} tools)`);
    } else {
      reasons.push("no tools required");
    }

    if (characteristics.contextSize >= 5000) {
      reasons.push(`large context (${characteristics.contextSize} tokens)`);
    } else if (characteristics.contextSize >= 1000) {
      reasons.push(`moderate context (${characteristics.contextSize} tokens)`);
    } else {
      reasons.push(`small context (${characteristics.contextSize} tokens)`);
    }

    if (characteristics.stepCount >= 5) {
      reasons.push(`many steps (${characteristics.stepCount})`);
    } else if (characteristics.stepCount >= 2) {
      reasons.push(`multiple steps (${characteristics.stepCount})`);
    }

    if (characteristics.multiTurn) {
      reasons.push("multi-turn interaction");
    }

    if (characteristics.hasExternalCalls) {
      reasons.push("external API calls");
    }

    return `Assessed as ${level}: ${reasons.join(", ")}`;
  }
}
