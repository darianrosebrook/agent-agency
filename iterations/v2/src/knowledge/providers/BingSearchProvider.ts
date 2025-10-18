/**
 * @fileoverview Bing Web Search Provider for ARBITER-006
 *
 * Implements real web search using Bing Web Search API v7.
 * Requires BING_SEARCH_API_KEY environment variable.
 *
 * @author @darianrosebrook
 */

import {
  KnowledgeQuery,
  SearchProviderConfig,
  SearchResult,
} from "../../types/knowledge";
import { BaseSearchProvider } from "../SearchProvider";

/**
 * Bing Web Search API Response Types
 */
interface BingSearchResponse {
  _type: string;
  queryContext: {
    originalQuery: string;
    alteredQuery?: string;
  };
  webPages?: {
    webSearchUrl: string;
    totalEstimatedMatches: number;
    value: Array<{
      id: string;
      name: string;
      url: string;
      isFamilyFriendly?: boolean;
      displayUrl: string;
      snippet: string;
      dateLastCrawled?: string;
      language?: string;
      isNavigational?: boolean;
    }>;
  };
  rankingResponse?: {
    mainline: {
      items: Array<{
        answerType: string;
        resultIndex: number;
        value: { id: string };
      }>;
    };
  };
}

/**
 * Bing Web Search Provider
 *
 * Provides web search using Microsoft Bing Web Search API v7.
 * Free tier: 1,000 queries per month.
 * Paid tiers: Up to 10,000,000 queries per month.
 */
export class BingSearchProvider extends BaseSearchProvider {
  private apiKey: string;
  private apiEndpoint = "https://api.bing.microsoft.com/v7.0/search";

  constructor(config: SearchProviderConfig) {
    super(config);

    this.apiKey = process.env.BING_SEARCH_API_KEY || "";

    if (!this.apiKey) {
      throw new Error(
        "BingSearchProvider requires BING_SEARCH_API_KEY environment variable"
      );
    }
  }

  /**
   * Execute search query using Bing Web Search API
   */
  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    const startTime = Date.now();

    try {
      // Build API URL with parameters
      const url = new URL(this.apiEndpoint);
      url.searchParams.set("q", query.query);
      url.searchParams.set("count", Math.min(query.maxResults, 50).toString());
      url.searchParams.set("mkt", "en-US"); // Market - could be configurable
      url.searchParams.set("safeSearch", "Moderate");

      // Add optional filters based on query type
      if (query.queryType === "technical") {
        // Freshness filter for technical content
        url.searchParams.set("freshness", "Month");
      } else if (query.queryType === "trend") {
        url.searchParams.set("freshness", "Day");
      }

      // Execute API request
      const response = await fetch(url.toString(), {
        headers: {
          "Ocp-Apim-Subscription-Key": this.apiKey,
          Accept: "application/json",
          "User-Agent": "ArbiterKnowledgeSeeker/1.0",
        },
        signal: AbortSignal.timeout(query.timeoutMs || 10000),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(
          `Bing Search API error (${response.status}): ${errorText}`
        );
      }

      const data = await response.json() as BingSearchResponse;

      // Parse and transform results
      const results = this.parseBingResults(data, query);

      // Track performance
      const responseTime = Date.now() - startTime;
      console.log(
        `Bing Search completed in ${responseTime}ms: ${results.length} results`
      );

      return results;
    } catch (error) {
      console.error("Bing Search failed:", error);

      // Check for rate limiting
      if (error instanceof Error && error.message.includes("429")) {
        throw new Error("Bing Search rate limit exceeded. Check your quota.");
      }

      // Check for invalid API key
      if (error instanceof Error && error.message.includes("401")) {
        throw new Error("Bing Search API key is invalid or missing.");
      }

      throw error;
    }
  }

  /**
   * Parse Bing Web Search API response into SearchResults
   */
  private parseBingResults(
    data: BingSearchResponse,
    query: KnowledgeQuery
  ): SearchResult[] {
    if (
      !data.webPages ||
      !data.webPages.value ||
      data.webPages.value.length === 0
    ) {
      return [];
    }

    return data.webPages.value.map((item, index) => {
      // Calculate relevance score based on position and ranking
      const positionScore = 1.0 - index * 0.08; // Slightly less aggressive than Google

      // Calculate credibility score based on domain
      const domain = super["extractDomain"](item.url);
      const credibilityScore = this.calculateCredibilityScore(domain);

      return this.createSearchResult(
        query.id,
        {
          title: item.name,
          snippet: item.snippet,
          url: item.url,
          link: item.url,
          displayLink: item.displayUrl,
          id: item.id,
          dateLastCrawled: item.dateLastCrawled,
          language: item.language,
          isNavigational: item.isNavigational,
        },
        Math.max(positionScore, 0.5), // Minimum 0.5 relevance
        credibilityScore
      );
    });
  }

  /**
   * Calculate credibility score based on domain reputation
   */
  private calculateCredibilityScore(domain: string): number {
    const domainLower = domain.toLowerCase();

    // High credibility domains
    const highCredibility = [
      "wikipedia.org",
      "github.com",
      "stackoverflow.com",
      "microsoft.com",
      "mozilla.org",
      "w3.org",
      "ieee.org",
      "acm.org",
      ".edu",
      ".gov",
    ];

    // Medium credibility domains
    const mediumCredibility = [
      "medium.com",
      "dev.to",
      "reddit.com",
      "youtube.com",
      "linkedin.com",
      "docs.",
    ];

    // Low credibility indicators
    const lowCredibility = [
      ".tk",
      ".ml",
      ".ga",
      ".cf",
      ".gq", // Free TLDs often used for spam
    ];

    // Check high credibility
    if (highCredibility.some((trusted) => domainLower.includes(trusted))) {
      return 0.9;
    }

    // Check medium credibility
    if (mediumCredibility.some((known) => domainLower.includes(known))) {
      return 0.7;
    }

    // Check low credibility
    if (lowCredibility.some((suspicious) => domainLower.includes(suspicious))) {
      return 0.3;
    }

    // Default credibility with HTTPS boost
    return domainLower.startsWith("https://") ? 0.6 : 0.5;
  }

  /**
   * Get provider status and health metrics
   */
  async healthCheck(): Promise<{ available: boolean; message?: string }> {
    try {
      // Simple health check - verify API key is set
      if (!this.apiKey) {
        return {
          available: false,
          message: "Missing API key",
        };
      }

      // Optionally: Execute a test query to verify API is responding
      // For now, just verify configuration
      return {
        available: true,
        message: "Bing Search provider configured",
      };
    } catch (error) {
      return {
        available: false,
        message: error instanceof Error ? error.message : "Health check failed",
      };
    }
  }
}
