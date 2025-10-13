/**
 * @fileoverview
 * Detects scaffolding patterns in code changes.
 * Identifies boilerplate, excessive comments, and other non-essential code.
 *
 * @author @darianrosebrook
 */

import type {
  CodeDiff,
  ScaffoldingDetection,
  ScaffoldingPattern,
} from "@/types/evaluation";
import { DEFAULT_SCAFFOLDING_PATTERNS } from "@/types/evaluation";

/**
 * Detects scaffolding and boilerplate code in diffs
 */
export class ScaffoldingDetector {
  private patterns: ScaffoldingPattern[];

  /**
   * Creates a new ScaffoldingDetector
   *
   * @param customPatterns Optional custom scaffolding patterns
   */
  constructor(customPatterns?: ScaffoldingPattern[]) {
    this.patterns = customPatterns ?? DEFAULT_SCAFFOLDING_PATTERNS;
  }

  /**
   * Detects scaffolding in a code diff
   *
   * @param diff Code diff to analyze
   * @returns Scaffolding detection result
   */
  detect(diff: CodeDiff): ScaffoldingDetection {
    const matchedPatterns: string[] = [];
    const reasons: string[] = [];
    let totalPenalty = 0;

    // Analyze the "after" code for scaffolding
    const code = diff.after;
    const lines = code.split("\n");
    const totalLines = lines.length;

    // Check each pattern
    for (const pattern of this.patterns) {
      const matches = this.countMatches(code, pattern.pattern);

      if (this.shouldPenalize(pattern, matches, totalLines)) {
        matchedPatterns.push(pattern.name);
        reasons.push(this.buildReason(pattern, matches, totalLines));
        totalPenalty += pattern.penalty;
      }
    }

    // Calculate final penalty factor and confidence
    const detected = matchedPatterns.length > 0;
    const penaltyFactor = Math.min(1.0, totalPenalty);
    const confidence = this.calculateConfidence(
      matchedPatterns.length,
      totalPenalty
    );

    return {
      detected,
      confidence,
      reasons,
      penaltyFactor,
      matchedPatterns,
    };
  }

  /**
   * Counts pattern matches in code
   *
   * @param code Code to search
   * @param pattern Regex pattern
   * @returns Number of matches
   */
  private countMatches(code: string, pattern: RegExp): number {
    const matches = code.match(pattern);
    return matches ? matches.length : 0;
  }

  /**
   * Determines if pattern should trigger penalty
   *
   * @param pattern Scaffolding pattern
   * @param matches Number of matches
   * @param totalLines Total lines of code
   * @returns True if should penalize
   */
  private shouldPenalize(
    pattern: ScaffoldingPattern,
    matches: number,
    totalLines: number
  ): boolean {
    if (matches === 0) {
      return false;
    }

    const matchRatio = matches / totalLines;

    // Category-specific thresholds
    switch (pattern.category) {
      case "comments":
        return matchRatio > 0.5; // > 50% comment lines
      case "boilerplate":
        return matches > 20; // > 20 import lines
      case "whitespace":
        return matchRatio > 0.3; // > 30% blank lines
      case "redundant":
        return matches > 0; // Any redundant code
      default:
        return false;
    }
  }

  /**
   * Builds human-readable reason for penalty
   *
   * @param pattern Pattern that matched
   * @param matches Number of matches
   * @param totalLines Total lines
   * @returns Reason string
   */
  private buildReason(
    pattern: ScaffoldingPattern,
    matches: number,
    totalLines: number
  ): string {
    const matchRatio = ((matches / totalLines) * 100).toFixed(1);

    switch (pattern.category) {
      case "comments":
        return `Excessive comments: ${matchRatio}% of lines are comments`;
      case "boilerplate":
        return `Large boilerplate: ${matches} import/boilerplate lines`;
      case "whitespace":
        return `Excessive whitespace: ${matchRatio}% blank lines`;
      case "redundant":
        return `Redundant code detected: ${matches} redundant blocks`;
      default:
        return `Pattern "${pattern.name}" matched ${matches} times`;
    }
  }

  /**
   * Calculates confidence score for detection
   *
   * @param patternCount Number of patterns matched
   * @param totalPenalty Total penalty accumulated
   * @returns Confidence score (0-1)
   */
  private calculateConfidence(
    patternCount: number,
    totalPenalty: number
  ): number {
    // Base confidence on number of patterns matched and penalty severity
    const patternConfidence = Math.min(
      1.0,
      patternCount / this.patterns.length
    );
    const penaltyConfidence = Math.min(1.0, totalPenalty);

    // Weighted average
    return patternConfidence * 0.4 + penaltyConfidence * 0.6;
  }

  /**
   * Adds a custom scaffolding pattern
   *
   * @param pattern Custom pattern to add
   */
  addPattern(pattern: ScaffoldingPattern): void {
    this.patterns.push(pattern);
  }

  /**
   * Gets all current patterns
   *
   * @returns Array of scaffolding patterns
   */
  getPatterns(): ScaffoldingPattern[] {
    return [...this.patterns];
  }
}
