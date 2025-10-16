/**
 * Google Fact Check Tools API Provider
 *
 * Integrates with Google's official fact-checking API to verify claims
 * against published fact-checks from reputable sources.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  FactCheckClaim,
  FactCheckResult,
  VerificationVerdict,
} from "../../types/verification";

export interface GoogleFactCheckConfig {
  apiKey: string;
  baseUrl?: string;
  timeout?: number;
  maxRetries?: number;
}

export interface GoogleFactCheckResponse {
  claims?: Array<{
    text: string;
    claimant: string;
    claimDate: string;
    claimReview: Array<{
      publisher: {
        name: string;
        site: string;
      };
      url: string;
      title: string;
      textualRating: string;
      languageCode: string;
    }>;
  }>;
}

/**
 * Google Fact Check Tools API Provider
 */
export class GoogleFactCheckProvider {
  private config: Required<GoogleFactCheckConfig>;

  constructor(config: GoogleFactCheckConfig) {
    this.config = {
      apiKey: config.apiKey,
      baseUrl: config.baseUrl || "https://factchecktools.googleapis.com",
      timeout: config.timeout || 10000,
      maxRetries: config.maxRetries || 3,
    };
  }

  /**
   * Check a claim against Google's fact-check database
   */
  async checkClaim(claim: FactCheckClaim): Promise<FactCheckResult> {
    const startTime = Date.now();

    try {
      // Query Google's Fact Check Tools API
      const response = await this.queryFactCheckAPI(claim.text);

      if (!response.claims || response.claims.length === 0) {
        return {
          claim,
          verdict: VerificationVerdict.UNVERIFIED,
          confidence: 0.3,
          explanation: "No fact-checks found for this claim",
          sources: [],
          relatedClaims: [],
          processingTimeMs: Date.now() - startTime,
        };
      }

      // Analyze the fact-check results
      const analysis = this.analyzeFactCheckResults(
        response.claims,
        claim.text
      );

      return {
        claim,
        verdict: analysis.verdict,
        confidence: analysis.confidence,
        explanation: analysis.explanation,
        sources: analysis.sources,
        relatedClaims: analysis.relatedClaims || [],
        processingTimeMs: Date.now() - startTime,
      };
    } catch (error) {
      console.warn(`Google Fact Check API error for claim ${claim.id}:`, error);

      return {
        claim,
        verdict: VerificationVerdict.ERROR,
        confidence: 0,
        explanation: `Fact-checking service unavailable: ${
          error instanceof Error ? error.message : "Unknown error"
        }`,
        sources: [],
        relatedClaims: [],
        processingTimeMs: Date.now() - startTime,
      };
    }
  }

  /**
   * Query the Google Fact Check Tools API
   */
  private async queryFactCheckAPI(
    query: string
  ): Promise<GoogleFactCheckResponse> {
    const url = new URL(`${this.config.baseUrl}/v1alpha1/claims:search`);
    url.searchParams.set("query", query);
    url.searchParams.set("key", this.config.apiKey);

    const response = await fetch(url.toString(), {
      method: "GET",
      headers: {
        Accept: "application/json",
      },
      signal: AbortSignal.timeout(this.config.timeout),
    });

    if (!response.ok) {
      throw new Error(
        `Google Fact Check API returned ${response.status}: ${response.statusText}`
      );
    }

    return await response.json();
  }

  /**
   * Analyze fact-check results to determine verdict
   */
  private analyzeFactCheckResults(
    claims: GoogleFactCheckResponse["claims"],
    originalQuery: string
  ) {
    if (!claims || claims.length === 0) {
      return {
        verdict: VerificationVerdict.UNVERIFIED,
        confidence: 0.3,
        explanation: "No fact-checks found for this claim",
        sources: [],
      };
    }

    // Analyze all claim reviews for this query
    const allReviews: Array<{
      rating: string;
      publisher: string;
      url: string;
      title: string;
    }> = [];

    for (const claim of claims) {
      if (claim.claimReview) {
        for (const review of claim.claimReview) {
          allReviews.push({
            rating: review.textualRating,
            publisher: review.publisher.name,
            url: review.url,
            title: review.title,
          });
        }
      }
    }

    // Determine overall verdict based on ratings
    const verdict = this.determineVerdictFromRatings(allReviews);
    const confidence = this.calculateConfidence(allReviews.length);

    return {
      verdict,
      confidence,
      explanation: this.generateExplanation(verdict, allReviews),
      sources: allReviews.map((review) => ({
        url: review.url,
        title: review.title,
        publisher: review.publisher,
        credibilityScore: this.getPublisherCredibility(review.publisher),
      })),
      relatedClaims: [],
    };
  }

  /**
   * Determine verdict from fact-check ratings
   */
  private determineVerdictFromRatings(
    reviews: Array<{ rating: string; publisher: string }>
  ): VerificationVerdict {
    if (reviews.length === 0) {
      return VerificationVerdict.UNVERIFIED;
    }

    // Count ratings (case-insensitive)
    const ratingCounts: Record<string, number> = {};
    for (const review of reviews) {
      const normalizedRating = review.rating.toLowerCase().trim();
      ratingCounts[normalizedRating] =
        (ratingCounts[normalizedRating] || 0) + 1;
    }

    // Determine verdict based on most common rating
    const sortedRatings = Object.entries(ratingCounts).sort(
      ([, a], [, b]) => b - a
    );

    const [topRating] = sortedRatings[0];

    // Map common fact-check ratings to verdicts
    if (topRating.includes("true") || topRating.includes("correct")) {
      return VerificationVerdict.VERIFIED_TRUE;
    } else if (
      topRating.includes("false") ||
      topRating.includes("incorrect") ||
      topRating.includes("lie")
    ) {
      return VerificationVerdict.VERIFIED_FALSE;
    } else if (
      topRating.includes("misleading") ||
      topRating.includes("mostly false")
    ) {
      return VerificationVerdict.VERIFIED_FALSE;
    } else if (topRating.includes("mixture") || topRating.includes("partly")) {
      return VerificationVerdict.MIXED;
    }

    return VerificationVerdict.UNVERIFIED;
  }

  /**
   * Calculate confidence based on number of sources
   */
  private calculateConfidence(numSources: number): number {
    // More sources = higher confidence
    if (numSources >= 5) return 0.9;
    if (numSources >= 3) return 0.8;
    if (numSources >= 2) return 0.7;
    if (numSources >= 1) return 0.6;
    return 0.3;
  }

  /**
   * Generate explanation for the verdict
   */
  private generateExplanation(
    verdict: VerificationVerdict,
    reviews: Array<{ rating: string; publisher: string; title: string }>
  ): string {
    const publisherNames = [...new Set(reviews.map((r) => r.publisher))];

    switch (verdict) {
      case VerificationVerdict.VERIFIED_TRUE:
        return `Verified as true by ${reviews.length} fact-check${
          reviews.length > 1 ? "s" : ""
        } from ${publisherNames.join(", ")}`;

      case VerificationVerdict.VERIFIED_FALSE:
        return `Verified as false by ${reviews.length} fact-check${
          reviews.length > 1 ? "s" : ""
        } from ${publisherNames.join(", ")}`;

      case VerificationVerdict.MIXED:
        return `Mixed results from ${reviews.length} fact-check${
          reviews.length > 1 ? "s" : ""
        } from ${publisherNames.join(", ")}`;

      default:
        return `Unable to verify claim. ${reviews.length} fact-check${
          reviews.length > 1 ? "s" : ""
        } found but inconclusive.`;
    }
  }

  /**
   * Get credibility score for a publisher (0-1 scale)
   */
  private getPublisherCredibility(publisher: string): number {
    // Known reputable fact-checkers get higher scores
    const reputableSources = [
      "factcheck.org",
      "snopes",
      "politifact",
      "washington post",
      "new york times",
      "bbc",
      "reuters",
      "associated press",
      "npr",
      "propublica",
    ];

    const normalizedPublisher = publisher.toLowerCase();

    for (const reputable of reputableSources) {
      if (normalizedPublisher.includes(reputable)) {
        return 0.9;
      }
    }

    // Default credibility for unknown sources
    return 0.7;
  }
}
