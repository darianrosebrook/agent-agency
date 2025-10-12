/**
 * @fileoverview DuckDuckGo Search Provider for ARBITER-006
 *
 * Implements web search using DuckDuckGo Instant Answer API.
 * No API key required, but has limitations compared to Google/Bing.
 *
 * Note: DuckDuckGo's Instant Answer API provides limited results.
 * For production use with high result counts, consider HTML scraping
 * or using their commercial API (if available).
 *
 * @author @darianrosebrook
 */

import { BaseSearchProvider } from "../SearchProvider";
import {
  KnowledgeQuery,
  SearchProviderConfig,
  SearchResult,
} from "../../types/knowledge";

/**
 * DuckDuckGo Instant Answer API Response Types
 */
interface DuckDuckGoResponse {
  Abstract: string;
  AbstractText: string;
  AbstractSource: string;
  AbstractURL: string;
  Image: string;
  Heading: string;
  Answer: string;
  AnswerType: string;
  Definition: string;
  DefinitionSource: string;
  DefinitionURL: string;
  RelatedTopics: Array<{
    FirstURL?: string;
    Icon?: { URL: string; Height: string; Width: string };
    Result: string;
    Text: string;
  } | {
    Name: string;
    Topics: Array<{
      FirstURL: string;
      Icon?: { URL: string; Height: string; Width: string };
      Result: string;
      Text: string;
    }>;
  }>;
  Results: Array<{
    FirstURL: string;
    Icon: { URL: string; Height: string; Width: string };
    Result: string;
    Text: string;
  }>;
  Type: string;
}

/**
 * DuckDuckGo Search Provider
 *
 * Provides web search using DuckDuckGo Instant Answer API.
 * Free to use, no API key required.
 * Limited to instant answers and related topics (typically 3-10 results).
 *
 * Limitations:
 * - Fewer results than Google/Bing
 * - No pagination
 * - Best for factual queries, definitions, and instant answers
 * - Not ideal for broad research or comparative queries
 */
export class DuckDuckGoSearchProvider extends BaseSearchProvider {
  private apiEndpoint = "https://api.duckduckgo.com/";

  constructor(config: SearchProviderConfig) {
    super(config);
    // No API key required for DuckDuckGo
  }

  /**
   * Execute search query using DuckDuckGo Instant Answer API
   */
  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    const startTime = Date.now();

    try {
      // Build API URL with parameters
      const url = new URL(this.apiEndpoint);
      url.searchParams.set("q", query.query);
      url.searchParams.set("format", "json");
      url.searchParams.set("no_html", "1"); // Strip HTML from results
      url.searchParams.set("skip_disambig", "1"); // Skip disambiguation pages

      // Execute API request
      const response = await fetch(url.toString(), {
        headers: {
          "Accept": "application/json",
          "User-Agent": "ArbiterKnowledgeSeeker/1.0",
        },
        signal: AbortSignal.timeout(query.timeoutMs || 10000),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(
          `DuckDuckGo Search API error (${response.status}): ${errorText}`
        );
      }

      const data: DuckDuckGoResponse = await response.json();

      // Parse and transform results
      const results = this.parseDuckDuckGoResults(data, query);

      // Track performance
      const responseTime = Date.now() - startTime;
      console.log(
        `DuckDuckGo Search completed in ${responseTime}ms: ${results.length} results`
      );

      // Log warning if result count is low
      if (results.length < 3 && results.length < query.maxResults) {
        console.warn(
          `DuckDuckGo returned only ${results.length} results. ` +
            `Consider using Google or Bing for more comprehensive results.`
        );
      }

      return results;
    } catch (error) {
      console.error("DuckDuckGo Search failed:", error);
      throw error;
    }
  }

  /**
   * Parse DuckDuckGo Instant Answer API response into SearchResults
   */
  private parseDuckDuckGoResults(
    data: DuckDuckGoResponse,
    query: KnowledgeQuery
  ): SearchResult[] {
    const results: SearchResult[] = [];
    let positionIndex = 0;

    // 1. Add Abstract (main answer) if available
    if (data.AbstractText && data.AbstractURL) {
      const relevance = 0.95; // Abstracts are highly relevant
      const credibility = this.calculateCredibilityScore(data.AbstractSource);

      results.push(
        this.createSearchResult(
          query.id,
          {
            title: data.Heading || data.AbstractSource || "Instant Answer",
            snippet: data.AbstractText,
            url: data.AbstractURL,
            link: data.AbstractURL,
            source: data.AbstractSource,
            image: data.Image,
            type: "abstract",
          },
          relevance,
          credibility
        )
      );
      positionIndex++;
    }

    // 2. Add Definition if available
    if (data.Definition && data.DefinitionURL) {
      const relevance = 0.90;
      const credibility = this.calculateCredibilityScore(data.DefinitionSource);

      results.push(
        this.createSearchResult(
          query.id,
          {
            title: `Definition: ${data.Heading || query.query}`,
            snippet: data.Definition,
            url: data.DefinitionURL,
            link: data.DefinitionURL,
            source: data.DefinitionSource,
            type: "definition",
          },
          relevance,
          credibility
        )
      );
      positionIndex++;
    }

    // 3. Add Results (if any)
    if (data.Results && data.Results.length > 0) {
      for (const result of data.Results) {
        if (positionIndex >= query.maxResults) break;

        const relevance = 0.85 - (positionIndex * 0.05);
        const domain = super["extractDomain"](result.FirstURL);
        const credibility = this.calculateCredibilityScore(domain);

        results.push(
          this.createSearchResult(
            query.id,
            {
              title: this.extractTitle(result.Text),
              snippet: result.Text,
              url: result.FirstURL,
              link: result.FirstURL,
              type: "result",
            },
            Math.max(relevance, 0.5),
            credibility
          )
        );
        positionIndex++;
      }
    }

    // 4. Add Related Topics
    if (data.RelatedTopics && data.RelatedTopics.length > 0) {
      for (const topic of data.RelatedTopics) {
        if (positionIndex >= query.maxResults) break;

        // Handle grouped topics
        if ("Topics" in topic && topic.Topics) {
          for (const subTopic of topic.Topics) {
            if (positionIndex >= query.maxResults) break;

            const relevance = 0.70 - (positionIndex * 0.05);
            const domain = super["extractDomain"](subTopic.FirstURL);
            const credibility = this.calculateCredibilityScore(domain);

            results.push(
              this.createSearchResult(
                query.id,
                {
                  title: this.extractTitle(subTopic.Text),
                  snippet: subTopic.Text,
                  url: subTopic.FirstURL,
                  link: subTopic.FirstURL,
                  category: topic.Name,
                  type: "related",
                },
                Math.max(relevance, 0.5),
                credibility
              )
            );
            positionIndex++;
          }
        }
        // Handle single topics
        else if ("FirstURL" in topic && topic.FirstURL) {
          const relevance = 0.70 - (positionIndex * 0.05);
          const domain = super["extractDomain"](topic.FirstURL);
          const credibility = this.calculateCredibilityScore(domain);

          results.push(
            this.createSearchResult(
              query.id,
              {
                title: this.extractTitle(topic.Text),
                snippet: topic.Text,
                url: topic.FirstURL,
                link: topic.FirstURL,
                type: "related",
              },
              Math.max(relevance, 0.5),
              credibility
            )
          );
          positionIndex++;
        }
      }
    }

    return results;
  }

  /**
   * Extract title from formatted text
   * DuckDuckGo often includes HTML-like formatting
   */
  private extractTitle(text: string): string {
    // Extract first sentence or up to 100 chars
    const firstSentence = text.split(/[.!?]/)[0];
    const title = firstSentence.length > 100
      ? firstSentence.substring(0, 97) + "..."
      : firstSentence;

    return title.trim();
  }

  /**
   * Calculate credibility score based on source/domain
   */
  private calculateCredibilityScore(source: string): number {
    const sourceLower = source.toLowerCase();

    // High credibility sources
    const highCredibility = [
      "wikipedia",
      "britannica",
      "github",
      "stackoverflow",
      "mozilla",
      "w3.org",
      ".edu",
      ".gov",
    ];

    // Medium credibility sources
    const mediumCredibility = [
      "medium",
      "dev.to",
      "reddit",
      "youtube",
      "docs",
    ];

    // Check high credibility
    if (highCredibility.some((trusted) => sourceLower.includes(trusted))) {
      return 0.9;
    }

    // Check medium credibility
    if (mediumCredibility.some((known) => sourceLower.includes(known))) {
      return 0.7;
    }

    // Default credibility
    return 0.6;
  }

  /**
   * Get provider status and health metrics
   */
  async healthCheck(): Promise<{ available: boolean; message?: string }> {
    try {
      // DuckDuckGo doesn't require API key, so always available
      return {
        available: true,
        message: "DuckDuckGo Search provider ready (no API key required)",
      };
    } catch (error) {
      return {
        available: false,
        message: error instanceof Error ? error.message : "Health check failed",
      };
    }
  }
}

