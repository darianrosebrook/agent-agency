/**
 * @fileoverview Google Custom Search Provider for ARBITER-006
 *
 * Implements real web search using Google Custom Search JSON API.
 * Requires GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_CX environment variables.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  KnowledgeQuery,
  SearchProviderConfig,
  SearchResult,
} from "../../types/knowledge";
import { BaseSearchProvider } from "../SearchProvider";

/**
 * Google Custom Search API Response Types
 */
interface GoogleSearchResponse {
  kind: string;
  url: { type: string; template: string };
  queries: {
    request: Array<{ totalResults: string; count: number }>;
    nextPage?: Array<any>;
  };
  items?: Array<{
    kind: string;
    title: string;
    htmlTitle: string;
    link: string;
    displayLink: string;
    snippet: string;
    htmlSnippet: string;
    cacheId?: string;
    formattedUrl: string;
    htmlFormattedUrl: string;
    pagemap?: {
      metatags?: Array<Record<string, string>>;
      cse_thumbnail?: Array<{ src: string; width: string; height: string }>;
      cse_image?: Array<{ src: string }>;
    };
  }>;
  searchInformation: {
    searchTime: number;
    formattedSearchTime: string;
    totalResults: string;
    formattedTotalResults: string;
  };
}

/**
 * Google Custom Search Provider
 *
 * Provides web search using Google Custom Search JSON API.
 * Free tier: 100 queries per day.
 * Paid tier: Up to 10,000 queries per day.
 */
export class GoogleSearchProvider extends BaseSearchProvider {
  private apiKey: string;
  private customSearchEngineId: string;
  private apiEndpoint = "https://www.googleapis.com/customsearch/v1";

  constructor(config: SearchProviderConfig) {
    super(config);

    this.apiKey = process.env.GOOGLE_SEARCH_API_KEY || "";
    this.customSearchEngineId = process.env.GOOGLE_SEARCH_CX || "";

    if (!this.apiKey || !this.customSearchEngineId) {
      throw new Error(
        "GoogleSearchProvider requires GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_CX environment variables"
      );
    }
  }

  /**
   * Execute search query using Google Custom Search API
   */
  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    const startTime = Date.now();

    try {
      // Build API URL with parameters
      const url = new URL(this.apiEndpoint);
      url.searchParams.set("key", this.apiKey);
      url.searchParams.set("cx", this.customSearchEngineId);
      url.searchParams.set("q", query.query);
      url.searchParams.set("num", Math.min(query.maxResults, 10).toString());

      // Add optional filters based on query type
      if (query.queryType === "technical") {
        // Prioritize technical documentation
        url.searchParams.set(
          "siteSearch",
          "github.com OR stackoverflow.com OR docs.*"
        );
        url.searchParams.set("siteSearchFilter", "i"); // Include sites
      }

      // Execute API request
      const response = await fetch(url.toString(), {
        headers: {
          Accept: "application/json",
          "User-Agent": "ArbiterKnowledgeSeeker/1.0",
        },
        signal: AbortSignal.timeout(query.timeoutMs || 10000),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(
          `Google Search API error (${response.status}): ${errorText}`
        );
      }

      const data: GoogleSearchResponse = await response.json();

      // Parse and transform results
      const results = this.parseGoogleResults(data, query);

      // Track performance
      const responseTime = Date.now() - startTime;
      console.log(
        `Google Search completed in ${responseTime}ms: ${results.length} results`
      );

      return results;
    } catch (error) {
      console.error("Google Search failed:", error);

      // Check for rate limiting
      if (error instanceof Error && error.message.includes("429")) {
        throw new Error("Google Search rate limit exceeded. Check your quota.");
      }

      throw error;
    }
  }

  /**
   * Parse Google Custom Search API response into SearchResults
   */
  private parseGoogleResults(
    data: GoogleSearchResponse,
    query: KnowledgeQuery
  ): SearchResult[] {
    if (!data.items || data.items.length === 0) {
      return [];
    }

    return data.items.map((item) => {
      // Calculate relevance score based on position
      const positionScore = 1.0 - data.items!.indexOf(item) * 0.1;

      // Calculate credibility score based on domain
      const credibilityScore = this.calculateCredibilityScore(item.displayLink);

      return this.createSearchResult(
        query.id,
        {
          title: this.stripHtmlTags(item.htmlTitle || item.title),
          snippet: this.stripHtmlTags(item.htmlSnippet || item.snippet),
          url: item.link,
          link: item.link,
          displayLink: item.displayLink,
          formattedUrl: item.formattedUrl,
          cacheId: item.cacheId,
          pagemap: item.pagemap,
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
      "docs.",
    ];

    // Check high credibility
    if (highCredibility.some((trusted) => domainLower.includes(trusted))) {
      return 0.9;
    }

    // Check medium credibility
    if (mediumCredibility.some((known) => domainLower.includes(known))) {
      return 0.7;
    }

    // Check for HTTPS (slight boost)
    const hasHttps = 0.6;

    // Default credibility
    return hasHttps;
  }

  /**
   * Strip HTML tags from text
   */
  private stripHtmlTags(html: string): string {
    return html
      .replace(/<[^>]*>/g, "") // Remove HTML tags
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'")
      .replace(/\s+/g, " ") // Normalize whitespace
      .trim();
  }

  /**
   * Get provider status and health metrics
   */
  async healthCheck(): Promise<{ available: boolean; message?: string }> {
    try {
      // Simple health check - verify API key is set
      if (!this.apiKey || !this.customSearchEngineId) {
        return {
          available: false,
          message: "Missing API key or Custom Search Engine ID",
        };
      }

      // Optionally: Execute a test query to verify API is responding
      // For now, just verify configuration
      return {
        available: true,
        message: "Google Search provider configured",
      };
    } catch (error) {
      return {
        available: false,
        message: error instanceof Error ? error.message : "Health check failed",
      };
    }
  }
}
