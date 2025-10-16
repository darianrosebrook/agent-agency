/**
 * Precedent Manager
 *
 * @author @darianrosebrook
 *
 * Manages constitutional precedents for case law and consistency.
 * Provides precedent storage, similarity matching, citation tracking,
 * and applicability assessment.
 *
 * Features:
 * - Precedent storage and retrieval
 * - Case similarity matching
 * - Citation tracking and ranking
 * - Precedent applicability assessment
 * - Precedent overruling and deprecation
 * - Search and filtering
 */
// @ts-nocheck


import {
  Precedent,
  PrecedentApplicability,
  RuleCategory,
  Verdict,
  ViolationSeverity,
} from "@/types/arbitration";

/**
 * Precedent manager configuration
 */
export interface PrecedentManagerConfig {
  /** Minimum similarity score for matching (0-1) */
  minSimilarityScore: number;

  /** Maximum precedents to return in search */
  maxSearchResults: number;

  /** Enable citation weighting */
  enableCitationWeighting: boolean;

  /** Citation decay factor (per year) */
  citationDecayFactor: number;

  /** Enable automatic relevance ranking */
  enableAutoRanking: boolean;
}

/**
 * Precedent search criteria
 */
export interface PrecedentSearchCriteria {
  /** Rule categories to match */
  categories?: RuleCategory[];

  /** Violation severity to match */
  severity?: ViolationSeverity;

  /** Minimum citation count */
  minCitations?: number;

  /** Created after date */
  createdAfter?: Date;

  /** Keywords in title or facts */
  keywords?: string[];

  /** Sort by field */
  sortBy?: "relevance" | "citations" | "date";

  /** Limit results */
  limit?: number;
}

/**
 * Similarity match result
 */
export interface SimilarityMatch {
  /** Matched precedent */
  precedent: Precedent;

  /** Similarity score (0-1) */
  score: number;

  /** Matching factors */
  matchingFactors: string[];
}

/**
 * PrecedentManager - Manages constitutional precedents
 */
export class PrecedentManager {
  /** Configuration */
  private config: PrecedentManagerConfig;

  /** Precedent storage */
  private precedents: Map<string, Precedent> = new Map();

  /** Citation graph (precedent ID -> citing precedent IDs) */
  private citationGraph: Map<string, Set<string>> = new Map();

  constructor(config?: Partial<PrecedentManagerConfig>) {
    this.config = {
      minSimilarityScore: 0.6,
      maxSearchResults: 10,
      enableCitationWeighting: true,
      citationDecayFactor: 0.95,
      enableAutoRanking: true,
      ...config,
    };
  }

  /**
   * Create a new precedent from a verdict
   */
  public createPrecedent(
    verdict: Verdict,
    title: string,
    keyFacts: string[],
    reasoningSummary: string,
    applicability: PrecedentApplicability,
    metadata: Record<string, unknown> = {}
  ): Precedent {
    const precedent: Precedent = {
      id: this.generatePrecedentId(),
      title,
      rulesInvolved: verdict.rulesApplied,
      verdict,
      keyFacts,
      reasoningSummary,
      applicability,
      citationCount: 0,
      createdAt: new Date(),
      metadata,
    };

    // Store precedent
    this.precedents.set(precedent.id, precedent);

    // Initialize citation tracking
    this.citationGraph.set(precedent.id, new Set());

    return precedent;
  }

  /**
   * Get a precedent by ID
   */
  public getPrecedent(id: string): Precedent | undefined {
    return this.precedents.get(id);
  }

  /**
   * Find similar precedents based on case characteristics
   */
  public findSimilarPrecedents(
    category: RuleCategory,
    severity: ViolationSeverity,
    facts: string[],
    rulesInvolved: string[],
    limit: number = this.config.maxSearchResults
  ): SimilarityMatch[] {
    const matches: SimilarityMatch[] = [];

    for (const precedent of this.precedents.values()) {
      const score = this.calculateSimilarity(
        category,
        severity,
        facts,
        rulesInvolved,
        precedent
      );

      if (score >= this.config.minSimilarityScore) {
        const matchingFactors = this.getMatchingFactors(
          category,
          severity,
          facts,
          rulesInvolved,
          precedent
        );

        matches.push({
          precedent,
          score,
          matchingFactors,
        });
      }
    }

    // Sort by score descending
    matches.sort((a, b) => b.score - a.score);

    // Apply citation weighting if enabled
    if (this.config.enableCitationWeighting) {
      matches.forEach((match) => {
        const citationBoost = this.calculateCitationBoost(match.precedent);
        match.score = Math.min(match.score * citationBoost, 1.0);
      });

      // Re-sort after citation weighting
      matches.sort((a, b) => b.score - a.score);
    }

    return matches.slice(0, limit);
  }

  /**
   * Search precedents by criteria
   */
  public searchPrecedents(criteria: PrecedentSearchCriteria): Precedent[] {
    let results = Array.from(this.precedents.values());

    // Filter by categories
    if (criteria.categories && criteria.categories.length > 0) {
      results = results.filter((p) =>
        criteria.categories!.includes(p.applicability.category)
      );
    }

    // Filter by severity
    if (criteria.severity) {
      results = results.filter(
        (p) => p.applicability.severity === criteria.severity
      );
    }

    // Filter by minimum citations
    if (criteria.minCitations !== undefined) {
      results = results.filter(
        (p) => p.citationCount >= criteria.minCitations!
      );
    }

    // Filter by created date
    if (criteria.createdAfter) {
      results = results.filter((p) => p.createdAt >= criteria.createdAfter!);
    }

    // Filter by keywords
    if (criteria.keywords && criteria.keywords.length > 0) {
      results = results.filter((p) => {
        const text = `${p.title} ${p.keyFacts.join(" ")} ${
          p.reasoningSummary
        }`.toLowerCase();
        return criteria.keywords!.some((keyword) =>
          text.includes(keyword.toLowerCase())
        );
      });
    }

    // Sort results
    if (criteria.sortBy) {
      switch (criteria.sortBy) {
        case "citations":
          results.sort((a, b) => b.citationCount - a.citationCount);
          break;
        case "date":
          results.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
          break;
        case "relevance":
          // Already sorted by relevance from findSimilarPrecedents
          break;
      }
    }

    // Limit results
    const limit = criteria.limit || this.config.maxSearchResults;
    return results.slice(0, limit);
  }

  /**
   * Record a citation of a precedent
   */
  public citePrecedent(
    precedentId: string,
    citingPrecedentId: string
  ): boolean {
    const precedent = this.precedents.get(precedentId);
    if (!precedent) {
      return false;
    }

    // Increment citation count
    precedent.citationCount++;

    // Update citation graph
    const citations = this.citationGraph.get(precedentId);
    if (citations) {
      citations.add(citingPrecedentId);
    }

    return true;
  }

  /**
   * Get all citing precedents
   */
  public getCitingPrecedents(precedentId: string): Precedent[] {
    const citingIds = this.citationGraph.get(precedentId);
    if (!citingIds) {
      return [];
    }

    return Array.from(citingIds)
      .map((id) => this.precedents.get(id))
      .filter((p): p is Precedent => p !== undefined);
  }

  /**
   * Overrule a precedent (deprecate it)
   */
  public overrulePrecedent(
    precedentId: string,
    overrulingPrecedentId: string,
    reason: string
  ): boolean {
    const precedent = this.precedents.get(precedentId);
    if (!precedent) {
      return false;
    }

    // Mark as overruled in metadata
    precedent.metadata.overruled = true;
    precedent.metadata.overruledBy = overrulingPrecedentId;
    precedent.metadata.overruledReason = reason;
    precedent.metadata.overruledAt = new Date();

    return true;
  }

  /**
   * Check if a precedent is still valid (not overruled)
   */
  public isValid(precedentId: string): boolean {
    const precedent = this.precedents.get(precedentId);
    if (!precedent) {
      return false;
    }

    return !precedent.metadata.overruled;
  }

  /**
   * Assess applicability of a precedent to a new case
   */
  public assessApplicability(
    precedent: Precedent,
    category: RuleCategory,
    severity: ViolationSeverity,
    conditions: string[]
  ): {
    applicable: boolean;
    confidence: number;
    reasoning: string;
  } {
    // Check if overruled
    if (precedent.metadata.overruled) {
      return {
        applicable: false,
        confidence: 1.0,
        reasoning: `Precedent overruled by ${precedent.metadata.overruledBy}: ${precedent.metadata.overruledReason}`,
      };
    }

    // Check category match
    const categoryMatch = precedent.applicability.category === category;
    if (!categoryMatch) {
      return {
        applicable: false,
        confidence: 0.9,
        reasoning: `Category mismatch: precedent applies to ${precedent.applicability.category}, case is ${category}`,
      };
    }

    // Check severity match
    const severityMatch = precedent.applicability.severity === severity;
    if (!severityMatch) {
      return {
        applicable: true,
        confidence: 0.6,
        reasoning: `Severity mismatch but category matches: precedent ${precedent.applicability.severity}, case ${severity}`,
      };
    }

    // Check conditions
    const conditionsMet = precedent.applicability.conditions.every(
      (condition) =>
        conditions.some((c) =>
          c.toLowerCase().includes(condition.toLowerCase())
        )
    );

    if (!conditionsMet) {
      return {
        applicable: true,
        confidence: 0.7,
        reasoning: "Not all precedent conditions are met in current case",
      };
    }

    return {
      applicable: true,
      confidence: 0.95,
      reasoning: "Strong match: category, severity, and conditions align",
    };
  }

  /**
   * Get precedent statistics
   */
  public getStatistics(): {
    totalPrecedents: number;
    validPrecedents: number;
    overruledPrecedents: number;
    averageCitations: number;
    mostCited: Precedent | undefined;
    byCategory: Record<string, number>;
  } {
    const all = Array.from(this.precedents.values());
    const valid = all.filter((p) => !p.metadata.overruled);
    const overruled = all.filter((p) => p.metadata.overruled);

    const totalCitations = all.reduce((sum, p) => sum + p.citationCount, 0);
    const averageCitations = all.length > 0 ? totalCitations / all.length : 0;

    const mostCited = all.reduce<Precedent | undefined>((max, p) => {
      if (!max || p.citationCount > max.citationCount) {
        return p;
      }
      return max;
    }, undefined);

    const byCategory: Record<string, number> = {};
    for (const precedent of all) {
      const category = precedent.applicability.category;
      byCategory[category] = (byCategory[category] || 0) + 1;
    }

    return {
      totalPrecedents: all.length,
      validPrecedents: valid.length,
      overruledPrecedents: overruled.length,
      averageCitations,
      mostCited,
      byCategory,
    };
  }

  /**
   * Calculate similarity between case and precedent
   */
  private calculateSimilarity(
    category: RuleCategory,
    severity: ViolationSeverity,
    facts: string[],
    rulesInvolved: string[],
    precedent: Precedent
  ): number {
    let score = 0;
    let weights = 0;

    // Category match (weight: 0.3)
    if (precedent.applicability.category === category) {
      score += 0.3;
    }
    weights += 0.3;

    // Severity match (weight: 0.2)
    if (precedent.applicability.severity === severity) {
      score += 0.2;
    }
    weights += 0.2;

    // Fact overlap (weight: 0.3)
    const factOverlap = this.calculateTextOverlap(facts, precedent.keyFacts);
    score += factOverlap * 0.3;
    weights += 0.3;

    // Rule overlap (weight: 0.2)
    const ruleOverlap = this.calculateSetOverlap(
      new Set(rulesInvolved),
      new Set(precedent.rulesInvolved)
    );
    score += ruleOverlap * 0.2;
    weights += 0.2;

    return weights > 0 ? score / weights : 0;
  }

  /**
   * Calculate text overlap between two arrays of strings
   */
  private calculateTextOverlap(arr1: string[], arr2: string[]): number {
    if (arr1.length === 0 || arr2.length === 0) {
      return 0;
    }

    const text1 = arr1.join(" ").toLowerCase();
    const text2 = arr2.join(" ").toLowerCase();

    const words1 = new Set(text1.split(/\s+/));
    const words2 = new Set(text2.split(/\s+/));

    return this.calculateSetOverlap(words1, words2);
  }

  /**
   * Calculate overlap between two sets
   */
  private calculateSetOverlap<T>(set1: Set<T>, set2: Set<T>): number {
    if (set1.size === 0 || set2.size === 0) {
      return 0;
    }

    const intersection = new Set([...set1].filter((x) => set2.has(x)));
    const union = new Set([...set1, ...set2]);

    return intersection.size / union.size;
  }

  /**
   * Get matching factors between case and precedent
   */
  private getMatchingFactors(
    category: RuleCategory,
    severity: ViolationSeverity,
    facts: string[],
    rulesInvolved: string[],
    precedent: Precedent
  ): string[] {
    const factors: string[] = [];

    if (precedent.applicability.category === category) {
      factors.push(`Category match: ${category}`);
    }

    if (precedent.applicability.severity === severity) {
      factors.push(`Severity match: ${severity}`);
    }

    const sharedRules = rulesInvolved.filter((r) =>
      precedent.rulesInvolved.includes(r)
    );
    if (sharedRules.length > 0) {
      factors.push(`Shared rules: ${sharedRules.join(", ")}`);
    }

    const factOverlap = this.calculateTextOverlap(facts, precedent.keyFacts);
    if (factOverlap > 0.5) {
      factors.push(`High fact similarity: ${(factOverlap * 100).toFixed(0)}%`);
    }

    return factors;
  }

  /**
   * Calculate citation boost for ranking
   */
  private calculateCitationBoost(precedent: Precedent): number {
    const yearsSinceCreation =
      (Date.now() - precedent.createdAt.getTime()) /
      (365.25 * 24 * 60 * 60 * 1000);

    const decayedCitations =
      precedent.citationCount *
      Math.pow(this.config.citationDecayFactor, yearsSinceCreation);

    // Boost is between 1.0 and 1.5
    const boost = 1.0 + Math.min(decayedCitations / 100, 0.5);

    return boost;
  }

  /**
   * Generate unique precedent ID
   */
  private generatePrecedentId(): string {
    return `PREC-${Date.now()}-${this.precedents.size + 1}`;
  }

  /**
   * Clear all precedents (for testing)
   */
  public clear(): void {
    this.precedents.clear();
    this.citationGraph.clear();
  }
}
