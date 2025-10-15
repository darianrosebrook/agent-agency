/**
 * Argument Structure
 *
 * Models and validates structured arguments with claims, evidence, and reasoning.
 * Implements argument credibility scoring and validation logic.
 *
 * @author @darianrosebrook
 * @module reasoning/ArgumentStructure
 */
// @ts-nocheck


import { Argument, Evidence, InvalidArgumentError } from "@/types/reasoning";

/**
 * Argument validation result
 */
export interface ArgumentValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  credibilityScore: number;
}

/**
 * Manages argument structure, validation, and credibility scoring
 */
export class ArgumentStructure {
  /**
   * Creates a new argument with validation
   */
  public static createArgument(
    agentId: string,
    claim: string,
    evidence: Evidence[],
    reasoning: string
  ): Argument {
    // Validate inputs
    if (!claim || claim.trim().length === 0) {
      throw new InvalidArgumentError("Claim cannot be empty");
    }

    if (!reasoning || reasoning.trim().length === 0) {
      throw new InvalidArgumentError("Reasoning cannot be empty");
    }

    if (claim.length > 1000) {
      throw new InvalidArgumentError(
        "Claim exceeds maximum length of 1000 characters"
      );
    }

    if (reasoning.length > 5000) {
      throw new InvalidArgumentError(
        "Reasoning exceeds maximum length of 5000 characters"
      );
    }

    // Calculate initial credibility score
    const credibilityScore = this.calculateCredibilityScore(
      claim,
      evidence,
      reasoning
    );

    return {
      id: this.generateArgumentId(),
      agentId,
      claim,
      evidence,
      reasoning,
      timestamp: new Date(),
      credibilityScore,
    };
  }

  /**
   * Validates an argument structure
   */
  public static validateArgument(argument: Argument): ArgumentValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Validate claim
    if (!argument.claim || argument.claim.trim().length === 0) {
      errors.push("Claim is empty");
    } else if (argument.claim.length < 10) {
      warnings.push("Claim is very short (< 10 characters)");
    } else if (argument.claim.length > 1000) {
      errors.push("Claim exceeds maximum length");
    }

    // Validate reasoning
    if (!argument.reasoning || argument.reasoning.trim().length === 0) {
      errors.push("Reasoning is empty");
    } else if (argument.reasoning.length < 50) {
      warnings.push("Reasoning is very brief (< 50 characters)");
    } else if (argument.reasoning.length > 5000) {
      errors.push("Reasoning exceeds maximum length");
    }

    // Validate evidence
    if (argument.evidence.length === 0) {
      warnings.push("No evidence provided");
    }

    // Validate evidence items
    argument.evidence.forEach((evidence, index) => {
      if (!evidence.source || evidence.source.trim().length === 0) {
        errors.push(`Evidence ${index + 1} missing source`);
      }
      if (!evidence.content || evidence.content.trim().length === 0) {
        errors.push(`Evidence ${index + 1} missing content`);
      }
      if (evidence.credibilityScore < 0 || evidence.credibilityScore > 1) {
        errors.push(`Evidence ${index + 1} has invalid credibility score`);
      }
    });

    // Calculate credibility
    const credibilityScore =
      errors.length === 0
        ? this.calculateCredibilityScore(
            argument.claim,
            argument.evidence,
            argument.reasoning
          )
        : 0;

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      credibilityScore,
    };
  }

  /**
   * Calculates argument credibility score (0-1)
   */
  public static calculateCredibilityScore(
    claim: string,
    evidence: Evidence[],
    reasoning: string
  ): number {
    let score = 0.5; // Base score

    // Evidence quality contribution (max +0.3)
    if (evidence.length > 0) {
      const avgEvidenceCredibility =
        evidence.reduce((sum, e) => sum + e.credibilityScore, 0) /
        evidence.length;
      score += avgEvidenceCredibility * 0.3;
    } else {
      score -= 0.1; // Penalty for no evidence
    }

    // Verified evidence bonus (max +0.1)
    const verifiedCount = evidence.filter(
      (e) => e.verificationStatus === "verified"
    ).length;
    if (evidence.length > 0) {
      score += (verifiedCount / evidence.length) * 0.1;
    }

    // Reasoning quality contribution (max +0.1)
    if (reasoning.length >= 100) {
      score += 0.05; // Good reasoning length
    }
    if (reasoning.length >= 500) {
      score += 0.05; // Comprehensive reasoning
    }

    // Claim quality contribution (max +0.1)
    if (claim.length >= 50 && claim.length <= 500) {
      score += 0.1; // Well-sized claim
    }

    // Disputed evidence penalty
    const disputedCount = evidence.filter(
      (e) => e.verificationStatus === "disputed"
    ).length;
    if (evidence.length > 0) {
      score -= (disputedCount / evidence.length) * 0.2;
    }

    // Ensure score is in [0, 1] range
    return Math.max(0, Math.min(1, score));
  }

  /**
   * Compares two arguments for strength
   */
  public static compareArguments(a: Argument, b: Argument): number {
    const scoreA =
      a.credibilityScore ??
      this.calculateCredibilityScore(a.claim, a.evidence, a.reasoning);
    const scoreB =
      b.credibilityScore ??
      this.calculateCredibilityScore(b.claim, b.evidence, b.reasoning);

    return scoreB - scoreA; // Higher score first
  }

  /**
   * Extracts key points from argument reasoning
   */
  public static extractKeyPoints(argument: Argument): string[] {
    // Simple extraction: split by sentences and filter meaningful ones
    const sentences = argument.reasoning
      .split(/[.!?]+/)
      .map((s) => s.trim())
      .filter((s) => s.length > 20);

    // Return first 5 key sentences
    return sentences.slice(0, 5);
  }

  /**
   * Generates a summary of the argument
   */
  public static summarizeArgument(argument: Argument): string {
    const evidenceCount = argument.evidence.length;
    const avgCredibility = argument.credibilityScore?.toFixed(2) ?? "N/A";

    return (
      `Claim: "${argument.claim}" ` +
      `(Credibility: ${avgCredibility}, ` +
      `Evidence: ${evidenceCount} items)`
    );
  }

  /**
   * Checks if arguments conflict with each other
   */
  public static detectConflict(a: Argument, b: Argument): boolean {
    // Simple heuristic: claims are opposite if they contain negation words
    const negationWords = ["not", "no", "never", "cannot", "isn't", "aren't"];

    const aLower = a.claim.toLowerCase();
    const bLower = b.claim.toLowerCase();

    // Check if one contains negation and the other doesn't for similar claims
    const aHasNegation = negationWords.some((word) => aLower.includes(word));
    const bHasNegation = negationWords.some((word) => bLower.includes(word));

    // If one has negation and other doesn't, and they share key terms, likely conflict
    if (aHasNegation !== bHasNegation) {
      const aWords = new Set(aLower.split(/\s+/).filter((w) => w.length > 3));
      const bWords = new Set(bLower.split(/\s+/).filter((w) => w.length > 3));

      let sharedWords = 0;
      aWords.forEach((word) => {
        if (bWords.has(word)) {
          sharedWords++;
        }
      });

      // Conflict if they share 30%+ of words
      const minWords = Math.min(aWords.size, bWords.size);
      return sharedWords / minWords > 0.3;
    }

    return false;
  }

  /**
   * Generates unique argument ID
   */
  private static generateArgumentId(): string {
    return `arg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}
