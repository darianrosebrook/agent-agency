/**
 * @fileoverview Cross-Reference Validator (ARBITER-007)
 *
 * Searches multiple independent sources and checks for consistency
 * across references to validate claims through consensus.
 *
 * @author @darianrosebrook
 */

import {
  VerificationMethodResult,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "../../types/verification";

/**
 * Configuration for cross-reference validation
 */
export interface CrossReferenceConfig {
  maxSources: number;
  minConsensus: number;
  searchProviders?: string[];
  minSourceQuality?: number;
  timeout?: number; // Timeout for cross-reference operations (ms)
}

/**
 * Reference source found during cross-referencing
 */
interface ReferenceSource {
  url: string;
  title: string;
  snippet: string;
  quality: number;
  supports: boolean;
  confidence: number;
}

/**
 * Extracted claim from content
 */
interface ExtractedClaim {
  text: string;
  type: "factual" | "statistical" | "general";
  keywords: string[];
}

/**
 * Cross-Reference Validator
 *
 * Validates claims by searching multiple independent sources
 * and checking for consensus across references.
 */
export class CrossReferenceValidator {
  private config: CrossReferenceConfig;

  constructor(config: Partial<CrossReferenceConfig> = {}) {
    this.config = {
      maxSources: config.maxSources ?? 5,
      minConsensus: config.minConsensus ?? 0.7,
      searchProviders: config.searchProviders ?? ["mock"],
      minSourceQuality: config.minSourceQuality ?? 0.5,
    };
  }

  /**
   * Verify content through cross-referencing
   */
  async verify(
    request: VerificationRequest
  ): Promise<VerificationMethodResult> {
    const startTime = Date.now();

    try {
      // Extract key claims from content
      const claims = this.extractClaims(request.content);

      if (claims.length === 0) {
        return {
          method: VerificationType.CROSS_REFERENCE,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0,
          reasoning: ["No verifiable claims found in content"],
          processingTimeMs: Date.now() - startTime,
          evidenceCount: 0,
        };
      }

      // Search for references across multiple sources
      const references = await this.searchMultipleSources(
        claims,
        request.context
      );

      if (references.length < 2) {
        return {
          method: VerificationType.CROSS_REFERENCE,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0.3,
          reasoning: [
            `Only ${references.length} reference(s) found, need at least 2 for cross-referencing`,
          ],
          processingTimeMs: Date.now() - startTime,
          evidenceCount: references.length,
        };
      }

      // Analyze consistency across references
      const consistency = this.analyzeConsistency(references);

      // Determine verdict based on consensus
      const verdict = this.determineVerdict(consistency);

      return {
        method: VerificationType.CROSS_REFERENCE,
        verdict: verdict.verdict,
        confidence: verdict.confidence,
        reasoning: verdict.reasoning,
        processingTimeMs: Date.now() - startTime,
        evidenceCount: references.length,
      };
    } catch (error) {
      return {
        method: VerificationType.CROSS_REFERENCE,
        verdict: VerificationVerdict.ERROR,
        confidence: 0,
        reasoning: [
          `Cross-reference validation failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
        ],
        processingTimeMs: Date.now() - startTime,
        evidenceCount: 0,
      };
    }
  }

  /**
   * Extract verifiable claims from content
   */
  private extractClaims(content: string): ExtractedClaim[] {
    const claims: ExtractedClaim[] = [];

    // Split into sentences
    const sentences = content
      .split(/[.!?]+/)
      .map((s) => s.trim())
      .filter((s) => s.length > 10);

    for (const sentence of sentences) {
      // Check if sentence contains factual claims
      const hasNumbers = /\d+/.test(sentence);
      const hasDateIndicators =
        /\b(in|on|during|since|until|before|after)\s+\d{4}\b/i.test(sentence);
      const hasStatisticalTerms =
        /\b(percent|percentage|rate|ratio|average|mean|median)\b/i.test(
          sentence
        );
      const hasFactualIndicators =
        /\b(is|are|was|were|has|have|had|according to|states that|claims that)\b/i.test(
          sentence
        );

      if (
        hasNumbers ||
        hasDateIndicators ||
        hasStatisticalTerms ||
        hasFactualIndicators
      ) {
        const claim: ExtractedClaim = {
          text: sentence,
          type: hasStatisticalTerms
            ? "statistical"
            : hasNumbers
            ? "factual"
            : "general",
          keywords: this.extractKeywords(sentence),
        };
        claims.push(claim);
      }
    }

    return claims.slice(0, 5); // Limit to top 5 claims
  }

  /**
   * Extract keywords from text for searching
   */
  private extractKeywords(text: string): string[] {
    // Remove common words and extract meaningful terms
    const commonWords = new Set([
      "the",
      "a",
      "an",
      "and",
      "or",
      "but",
      "in",
      "on",
      "at",
      "to",
      "for",
      "of",
      "with",
      "by",
      "from",
      "is",
      "are",
      "was",
      "were",
      "be",
      "been",
      "has",
      "have",
      "had",
      "do",
      "does",
      "did",
      "will",
      "would",
      "could",
      "should",
    ]);

    const words = text
      .toLowerCase()
      .replace(/[^\w\s]/g, "")
      .split(/\s+/)
      .filter((w) => w.length > 3 && !commonWords.has(w));

    // Return unique keywords
    return Array.from(new Set(words)).slice(0, 10);
  }

  /**
   * Search multiple sources for references
   */
  private async searchMultipleSources(
    claims: ExtractedClaim[],
    context?: string
  ): Promise<ReferenceSource[]> {
    const references: ReferenceSource[] = [];

    // For each claim, perform mock searches
    for (const claim of claims) {
      const searchQuery = claim.keywords.join(" ");
      const claimReferences = await this.mockSearch(searchQuery, context);
      references.push(...claimReferences);
    }

    // Deduplicate and limit to maxSources
    const uniqueReferences = this.deduplicateReferences(references);
    return uniqueReferences.slice(0, this.config.maxSources);
  }

  /**
   * Mock search function (would be replaced with real search API)
   */
  private async mockSearch(
    query: string,
    context?: string
  ): Promise<ReferenceSource[]> {
    // Simulate search delay
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Generate mock references with varying support levels
    const numReferences = Math.floor(Math.random() * 3) + 2;
    const references: ReferenceSource[] = [];

    for (let i = 0; i < numReferences; i++) {
      const supports = Math.random() > 0.3; // 70% support rate
      references.push({
        url: `https://example.com/source${i + 1}`,
        title: `Reference Source ${i + 1} for: ${query.substring(0, 30)}`,
        snippet: `This source ${
          supports ? "confirms" : "contradicts"
        } the claim about ${query}`,
        quality: 0.5 + Math.random() * 0.5, // 0.5-1.0 quality
        supports,
        confidence: 0.6 + Math.random() * 0.4, // 0.6-1.0 confidence
      });
    }

    return references;
  }

  /**
   * Deduplicate references by URL
   */
  private deduplicateReferences(
    references: ReferenceSource[]
  ): ReferenceSource[] {
    const seen = new Set<string>();
    const unique: ReferenceSource[] = [];

    for (const ref of references) {
      if (!seen.has(ref.url)) {
        seen.add(ref.url);
        unique.push(ref);
      }
    }

    return unique;
  }

  /**
   * Analyze consistency across references
   */
  private analyzeConsistency(references: ReferenceSource[]): {
    consensus: number;
    supporting: number;
    contradicting: number;
    averageQuality: number;
    averageConfidence: number;
  } {
    const supporting = references.filter((r) => r.supports).length;
    const contradicting = references.length - supporting;
    const consensus = supporting / references.length;

    const averageQuality =
      references.reduce((sum, r) => sum + r.quality, 0) / references.length;

    const averageConfidence =
      references.reduce((sum, r) => sum + r.confidence, 0) / references.length;

    return {
      consensus,
      supporting,
      contradicting,
      averageQuality,
      averageConfidence,
    };
  }

  /**
   * Determine verdict based on consistency analysis
   */
  private determineVerdict(consistency: {
    consensus: number;
    supporting: number;
    contradicting: number;
    averageQuality: number;
    averageConfidence: number;
  }): {
    verdict: VerificationVerdict;
    confidence: number;
    reasoning: string[];
  } {
    const reasoning: string[] = [];

    // High consensus supporting
    if (consistency.consensus >= this.config.minConsensus) {
      reasoning.push(
        `Strong consensus (${(consistency.consensus * 100).toFixed(
          1
        )}%) across ${consistency.supporting} sources`
      );
      reasoning.push(
        `Average source quality: ${(consistency.averageQuality * 100).toFixed(
          1
        )}%`
      );

      return {
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: consistency.consensus * consistency.averageConfidence,
        reasoning,
      };
    }

    // High consensus contradicting
    if (consistency.consensus <= 1 - this.config.minConsensus) {
      reasoning.push(
        `Strong consensus (${((1 - consistency.consensus) * 100).toFixed(
          1
        )}%) against claim across ${consistency.contradicting} sources`
      );
      reasoning.push(
        `Average source quality: ${(consistency.averageQuality * 100).toFixed(
          1
        )}%`
      );

      return {
        verdict: VerificationVerdict.VERIFIED_FALSE,
        confidence: (1 - consistency.consensus) * consistency.averageConfidence,
        reasoning,
      };
    }

    // Mixed evidence
    reasoning.push(
      `Mixed evidence: ${consistency.supporting} supporting, ${consistency.contradicting} contradicting`
    );
    reasoning.push(`Consensus: ${(consistency.consensus * 100).toFixed(1)}%`);
    reasoning.push(
      `Need at least ${(this.config.minConsensus * 100).toFixed(
        1
      )}% consensus for verification`
    );

    return {
      verdict: VerificationVerdict.CONTRADICTORY,
      confidence: 0.5,
      reasoning,
    };
  }
}
