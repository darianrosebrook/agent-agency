/**
 * Evidence Aggregator
 *
 * Aggregates and weighs evidence from multiple sources and arguments.
 * Implements credibility scoring and conflict detection.
 *
 * @author @darianrosebrook
 * @module reasoning/EvidenceAggregator
 */
// @ts-nocheck


import { Argument, Evidence, EvidenceAggregation } from "@/types/reasoning";

/**
 * Evidence conflict detection result
 */
export interface EvidenceConflict {
  evidence1: Evidence;
  evidence2: Evidence;
  conflictType: "contradictory" | "incompatible" | "disputed";
  severity: "low" | "medium" | "high";
  description: string;
}

/**
 * Aggregates and analyzes evidence across multiple arguments
 */
export class EvidenceAggregator {
  /**
   * Aggregates evidence from multiple arguments
   */
  public static aggregateEvidence(args: Argument[]): EvidenceAggregation {
    const allEvidence: Evidence[] = [];
    const sourceMap = new Map<string, number>();

    // Collect all evidence
    args.forEach((arg) => {
      arg.evidence.forEach((evidence) => {
        allEvidence.push(evidence);

        // Track sources
        const count = sourceMap.get(evidence.source) ?? 0;
        sourceMap.set(evidence.source, count + 1);
      });
    });

    // Calculate aggregate metrics
    const totalEvidence = allEvidence.length;
    const averageCredibility =
      totalEvidence > 0
        ? allEvidence.reduce((sum, e) => sum + e.credibilityScore, 0) /
          totalEvidence
        : 0;

    const verifiedCount = allEvidence.filter(
      (e) => e.verificationStatus === "verified"
    ).length;

    const disputedCount = allEvidence.filter(
      (e) => e.verificationStatus === "disputed"
    ).length;

    const sources = Array.from(sourceMap.keys());

    // Generate summary
    const summary = this.generateEvidenceSummary(
      allEvidence,
      verifiedCount,
      disputedCount
    );

    return {
      totalEvidence,
      averageCredibility,
      verifiedCount,
      disputedCount,
      sources,
      summary,
    };
  }

  /**
   * Weighs evidence by credibility and verification status
   */
  public static weighEvidence(evidence: Evidence[]): Map<string, number> {
    const weights = new Map<string, number>();

    evidence.forEach((e) => {
      let weight = e.credibilityScore;

      // Adjust weight based on verification status
      if (e.verificationStatus === "verified") {
        weight *= 1.2; // Boost verified evidence
      } else if (e.verificationStatus === "disputed") {
        weight *= 0.5; // Reduce disputed evidence
      }

      // Ensure weight is in [0, 1] range
      weight = Math.max(0, Math.min(1, weight));

      weights.set(e.id, weight);
    });

    return weights;
  }

  /**
   * Detects conflicts between evidence items
   */
  public static detectConflicts(evidence: Evidence[]): EvidenceConflict[] {
    const conflicts: EvidenceConflict[] = [];

    // Compare each pair of evidence items
    for (let i = 0; i < evidence.length; i++) {
      for (let j = i + 1; j < evidence.length; j++) {
        const conflict = this.compareEvidence(evidence[i], evidence[j]);
        if (conflict) {
          conflicts.push(conflict);
        }
      }
    }

    return conflicts;
  }

  /**
   * Compares two evidence items for conflicts
   */
  private static compareEvidence(
    e1: Evidence,
    e2: Evidence
  ): EvidenceConflict | null {
    // Check if both are disputed
    if (
      e1.verificationStatus === "disputed" &&
      e2.verificationStatus === "disputed"
    ) {
      return {
        evidence1: e1,
        evidence2: e2,
        conflictType: "disputed",
        severity: "medium",
        description: "Both evidence items are disputed",
      };
    }

    // Check for contradictory content (simple heuristic)
    const negationWords = ["not", "no", "never", "cannot", "false"];
    const e1Lower = e1.content.toLowerCase();
    const e2Lower = e2.content.toLowerCase();

    const e1HasNegation = negationWords.some((word) => e1Lower.includes(word));
    const e2HasNegation = negationWords.some((word) => e2Lower.includes(word));

    if (e1HasNegation !== e2HasNegation) {
      // Check for shared terms indicating same topic
      const e1Words = new Set(e1Lower.split(/\s+/).filter((w) => w.length > 4));
      const e2Words = new Set(e2Lower.split(/\s+/).filter((w) => w.length > 4));

      let sharedWords = 0;
      e1Words.forEach((word) => {
        if (e2Words.has(word)) {
          sharedWords++;
        }
      });

      const minWords = Math.min(e1Words.size, e2Words.size);
      if (minWords > 0 && sharedWords / minWords > 0.4) {
        return {
          evidence1: e1,
          evidence2: e2,
          conflictType: "contradictory",
          severity: "high",
          description: "Evidence items appear to contradict each other",
        };
      }
    }

    return null;
  }

  /**
   * Generates evidence summary text
   */
  private static generateEvidenceSummary(
    evidence: Evidence[],
    verifiedCount: number,
    disputedCount: number
  ): string {
    const total = evidence.length;

    if (total === 0) {
      return "No evidence provided";
    }

    const verifiedPercent = Math.round((verifiedCount / total) * 100);
    const disputedPercent = Math.round((disputedCount / total) * 100);
    const avgCredibility = (
      evidence.reduce((sum, e) => sum + e.credibilityScore, 0) / total
    ).toFixed(2);

    return (
      `Total: ${total} items, ` +
      `Verified: ${verifiedPercent}%, ` +
      `Disputed: ${disputedPercent}%, ` +
      `Average credibility: ${avgCredibility}`
    );
  }

  /**
   * Filters evidence by minimum credibility threshold
   */
  public static filterByCredibility(
    evidence: Evidence[],
    minCredibility: number
  ): Evidence[] {
    return evidence.filter((e) => e.credibilityScore >= minCredibility);
  }

  /**
   * Groups evidence by source
   */
  public static groupBySource(evidence: Evidence[]): Map<string, Evidence[]> {
    const groups = new Map<string, Evidence[]>();

    evidence.forEach((e) => {
      const existing = groups.get(e.source) ?? [];
      groups.set(e.source, [...existing, e]);
    });

    return groups;
  }

  /**
   * Calculates source diversity (0-1 score)
   */
  public static calculateSourceDiversity(evidence: Evidence[]): number {
    if (evidence.length === 0) {
      return 0;
    }

    const uniqueSources = new Set(evidence.map((e) => e.source));
    return Math.min(1, uniqueSources.size / evidence.length);
  }

  /**
   * Identifies most credible evidence
   */
  public static getMostCredibleEvidence(
    evidence: Evidence[],
    count: number = 5
  ): Evidence[] {
    return evidence
      .sort((a, b) => b.credibilityScore - a.credibilityScore)
      .slice(0, count);
  }

  /**
   * Validates evidence quality
   */
  public static validateEvidenceQuality(evidence: Evidence[]): {
    valid: boolean;
    issues: string[];
  } {
    const issues: string[] = [];

    // Check minimum evidence count
    if (evidence.length === 0) {
      issues.push("No evidence provided");
    }

    // Check for low-credibility evidence
    const lowCredCount = evidence.filter(
      (e) => e.credibilityScore < 0.3
    ).length;
    if (lowCredCount > evidence.length * 0.5) {
      issues.push("More than 50% of evidence has low credibility");
    }

    // Check for disputed evidence
    const disputedCount = evidence.filter(
      (e) => e.verificationStatus === "disputed"
    ).length;
    if (disputedCount > evidence.length * 0.3) {
      issues.push("More than 30% of evidence is disputed");
    }

    // Check source diversity
    const diversity = this.calculateSourceDiversity(evidence);
    if (diversity < 0.3) {
      issues.push("Low source diversity (< 30%)");
    }

    return {
      valid: issues.length === 0,
      issues,
    };
  }
}
