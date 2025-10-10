/**
 * @fileoverview Fact Checker Component (ARBITER-007)
 *
 * Handles fact-checking verification by checking claims against
 * reliable fact-checking sources and databases.
 *
 * @author @darianrosebrook
 */

import {
  FactCheckClaim,
  FactCheckResult,
  FactCheckSource,
  RelatedClaim,
  VerificationMethodConfig,
  VerificationMethodResult,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "../types/verification";

/**
 * Fact Checker Implementation
 */
export class FactChecker {
  private methodConfigs: VerificationMethodConfig[];

  constructor(methodConfigs: VerificationMethodConfig[]) {
    this.methodConfigs = methodConfigs;
  }

  /**
   * Execute fact-checking verification
   */
  async verify(
    request: VerificationRequest
  ): Promise<VerificationMethodResult> {
    const startTime = Date.now();

    try {
      // Extract claims from the request content
      const claims = this.extractClaims(request);

      if (claims.length === 0) {
        return {
          method: VerificationType.FACT_CHECKING,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0,
          reasoning: ["No verifiable claims found in content"],
          processingTimeMs: Math.max(1, Date.now() - startTime),
          evidenceCount: 0,
        };
      }

      // Check claims against fact-checking sources
      const factCheckResults = await Promise.all(
        claims.map((claim) => this.checkClaim(claim))
      );

      // Aggregate results
      const aggregatedResult = this.aggregateFactCheckResults(factCheckResults);

      return {
        method: VerificationType.FACT_CHECKING,
        verdict: aggregatedResult.verdict,
        confidence: aggregatedResult.confidence,
        reasoning: aggregatedResult.explanations,
        processingTimeMs: Math.max(1, Date.now() - startTime),
        evidenceCount: aggregatedResult.evidenceCount,
      };
    } catch (error) {
      return {
        method: VerificationType.FACT_CHECKING,
        verdict: VerificationVerdict.UNVERIFIED,
        confidence: 0,
        reasoning: [
          `Fact-checking failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
        ],
        processingTimeMs: Math.max(1, Date.now() - startTime),
        evidenceCount: 0,
      };
    }
  }

  /**
   * Extract verifiable claims from request content
   */
  private extractClaims(request: VerificationRequest): FactCheckClaim[] {
    const content = request.content.toLowerCase();

    // Simple claim extraction - look for factual statements
    // In production, this would use NLP to extract claims
    const claims: FactCheckClaim[] = [];

    // Look for statements that could be fact-checked
    const sentences = content
      .split(/[.!?]+/)
      .filter((s) => s.trim().length > 10);

    for (const sentence of sentences) {
      const trimmed = sentence.trim();

      // Check if sentence contains verifiable information
      if (this.isVerifiableClaim(trimmed)) {
        claims.push({
          text: trimmed,
          context: request.context,
          language: "en", // Assume English
          category: this.categorizeClaim(trimmed),
        });
      }
    }

    // Limit to reasonable number of claims
    return claims.slice(0, 5);
  }

  /**
   * Check if a sentence contains a verifiable claim
   */
  private isVerifiableClaim(text: string): boolean {
    // Look for indicators of factual claims
    const factIndicators = [
      /\d+/, // Numbers
      /\b\d{4}\b/, // Years
      /\b(?:said|stated|claimed|reported|according to)\b/i,
      /\b(?:percent|million|billion|trillion)\b/i,
      /\b(?:first|last|only|most|least)\b/i,
      /\b(?:died|born|created|invented)\b/i,
      /\b(?:earth|sun|moon|mars|venus|mercury|jupiter|saturn)\b/i, // Astronomical bodies
      /\b(?:revolves|orbits|rotates|spins)\b/i, // Astronomical terms
    ];

    return factIndicators.some((pattern) => pattern.test(text));
  }

  /**
   * Categorize a claim for better fact-checking
   */
  private categorizeClaim(text: string): string | undefined {
    if (/\b\d{4}\b/.test(text)) return "historical";
    if (/\b(?:percent|million|billion)\b/i.test(text)) return "statistical";
    if (/\b(?:died|born|age)\b/i.test(text)) return "biographical";
    if (/\b(?:invented|created|discovered)\b/i.test(text)) return "scientific";
    return "general";
  }

  /**
   * Check a single claim against fact-checking sources
   */
  private async checkClaim(claim: FactCheckClaim): Promise<FactCheckResult> {
    // In production, this would call actual fact-checking APIs
    // For now, we'll simulate fact-checking with mock responses

    // Simulate API delay
    await new Promise((resolve) =>
      setTimeout(resolve, 100 + Math.random() * 200)
    );

    // Generate mock fact-check result based on claim content
    return this.generateMockFactCheckResult(claim);
  }

  /**
   * Generate mock fact-check result (for development/testing)
   */
  private generateMockFactCheckResult(claim: FactCheckClaim): FactCheckResult {
    // Simple heuristic-based mock responses
    const text = claim.text.toLowerCase();

    let verdict = VerificationVerdict.UNVERIFIED;
    let confidence = 0.5;
    let explanation = "Claim could not be verified";

    // Mock some common verifiable claims
    if (
      text.includes("earth is round") ||
      text.includes("earth revolves around") ||
      text.includes("revolves around sun")
    ) {
      verdict = VerificationVerdict.VERIFIED_TRUE;
      confidence = 0.95;
      explanation = "This is a well-established scientific fact";
    } else if (text.includes("moon landing") && text.includes("fake")) {
      verdict = VerificationVerdict.VERIFIED_FALSE;
      confidence = 0.9;
      explanation =
        "The Apollo moon landings are well-documented historical events";
    } else if (text.includes("vaccine") && text.includes("autism")) {
      verdict = VerificationVerdict.VERIFIED_FALSE;
      confidence = 0.85;
      explanation =
        "Multiple studies have shown no link between vaccines and autism";
    } else if (/\b\d{4}\b/.test(text) && text.includes("born")) {
      // Birth year claims - often verifiable
      verdict = VerificationVerdict.UNVERIFIED;
      confidence = 0.6;
      explanation =
        "Birth date claims require specific verification against reliable sources";
    }

    // Mock sources
    const sources: FactCheckSource[] = [
      {
        url: "https://example.com/fact-check-1",
        title: "Fact Check: " + claim.text.substring(0, 50) + "...",
        publisher: "Example Fact Check",
        credibilityScore: 0.85,
        publishDate: new Date(),
        excerpt: explanation,
      },
    ];

    // Mock related claims
    const relatedClaims: RelatedClaim[] = [];

    return {
      claim,
      verdict,
      confidence,
      explanation,
      sources,
      relatedClaims,
    };
  }

  /**
   * Aggregate results from multiple fact-checks
   */
  private aggregateFactCheckResults(results: FactCheckResult[]): {
    verdict: VerificationVerdict;
    confidence: number;
    explanations: string[];
    evidenceCount: number;
  } {
    if (results.length === 0) {
      return {
        verdict: VerificationVerdict.INSUFFICIENT_DATA,
        confidence: 0,
        explanations: ["No fact-check results available"],
        evidenceCount: 0,
      };
    }

    // Count verdicts
    const verdictCounts = new Map<VerificationVerdict, number>();
    let totalConfidence = 0;
    const explanations: string[] = [];
    let totalEvidence = 0;

    for (const result of results) {
      verdictCounts.set(
        result.verdict,
        (verdictCounts.get(result.verdict) || 0) + 1
      );
      totalConfidence += result.confidence;
      explanations.push(result.explanation);
      totalEvidence += result.sources.length;
    }

    const averageConfidence = totalConfidence / results.length;

    // Find most common verdict
    let consensusVerdict = VerificationVerdict.UNVERIFIED;
    let maxCount = 0;

    for (const [verdict, count] of verdictCounts.entries()) {
      if (count > maxCount) {
        maxCount = count;
        consensusVerdict = verdict;
      }
    }

    // Adjust confidence based on consensus
    const consensusRatio = maxCount / results.length;
    const adjustedConfidence = averageConfidence * (0.5 + 0.5 * consensusRatio);

    return {
      verdict: consensusVerdict,
      confidence: Math.min(1.0, adjustedConfidence),
      explanations,
      evidenceCount: totalEvidence,
    };
  }

  /**
   * Get method configuration
   */
  private getMethodConfig(): VerificationMethodConfig | undefined {
    return this.methodConfigs.find(
      (config) => config.type === VerificationType.FACT_CHECKING
    );
  }

  /**
   * Check if method is available
   */
  async isAvailable(): Promise<boolean> {
    const config = this.getMethodConfig();
    return config?.enabled ?? false;
  }

  /**
   * Get method health status
   */
  getHealth(): { available: boolean; responseTime: number; errorRate: number } {
    // Mock health status
    return {
      available: true,
      responseTime: 150,
      errorRate: 0.02,
    };
  }
}
