/**
 * @fileoverview Search Engine for ARBITER-008
 *
 * Lightweight search execution layer that delegates to Knowledge Seeker
 * for actual search operations and enriches results with full content extraction.
 *
 * @author @darianrosebrook
 */

import { KnowledgeSeeker } from "../knowledge/KnowledgeSeeker";
import { KnowledgeQuery, QueryType, SearchResult } from "../types/knowledge";
import { ContentExtractionConfig, WebContent } from "../types/web";
import { ContentExtractor } from "./ContentExtractor";

/**
 * Search query parameters
 */
export interface SearchQuery {
  query: string;
  maxResults?: number;
  language?: string;
  region?: string;
  safeSearch?: boolean;
  excludeDomains?: string[];
}

/**
 * Search result with enriched content
 */
export interface EnrichedSearchResult extends SearchResult {
  fullContent?: WebContent;
  extractionError?: string;
  rank?: number;
}

/**
 * Search results with metadata
 */
export interface SearchResults {
  query: string;
  results: EnrichedSearchResult[];
  totalFound: number;
  totalResults?: number; // Alias for totalFound
  processingTimeMs: number;
  executionTimeMs?: number; // Alias for processingTimeMs
  cacheUsed: boolean;
  timestamp?: Date;
}

/**
 * Search Engine configuration
 */
export interface SearchEngineConfig {
  /**
   * API key for search services
   */
  apiKey?: string;

  /**
   * Request timeout in milliseconds
   */
  timeoutMs?: number;

  /**
   * User agent string for requests
   */
  userAgent?: string;

  /**
   * Whether to automatically extract full content for top results
   */
  autoExtractContent: boolean;

  /**
   * Number of top results to extract full content for
   */
  autoExtractCount: number;

  /**
   * Content extraction configuration
   */
  extractionConfig: ContentExtractionConfig;
}

/**
 * Search Engine
 *
 * Delegates search operations to Knowledge Seeker and optionally
 * enriches results with full content extraction.
 */
export class SearchEngine {
  private cache: Map<string, { results: SearchResults; expiresAt: Date }>;

  constructor(
    private readonly knowledgeSeeker: KnowledgeSeeker,
    private readonly contentExtractor: ContentExtractor,
    private readonly config: SearchEngineConfig
  ) {
    this.cache = new Map();
  }

  /**
   * Execute search query
   */
  async search(
    query: string | SearchQuery,
    options?: {
      maxResults?: number;
      queryType?: QueryType;
      enrichContent?: boolean;
    }
  ): Promise<SearchResults>;
  async search(
    query: string | SearchQuery,
    options: {
      maxResults?: number;
      queryType?: QueryType;
      enrichContent?: boolean;
    } = {}
  ): Promise<SearchResults> {
    const startTime = Date.now();

    // Extract query string and options from SearchQuery if provided
    const queryString = typeof query === "string" ? query : query.query;

    // Check cache
    const cacheKey = this.generateCacheKey(queryString, options);
    const cached = this.cache.get(cacheKey);
    if (cached && cached.expiresAt > new Date()) {
      return cached.results;
    }
    const queryOptions = typeof query === "object" ? query : options || {};

    // Merge options
    const mergedOptions = { ...options, ...queryOptions };

    // Delegate to Knowledge Seeker
    const knowledgeQuery: KnowledgeQuery = {
      id: `search-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      query: queryString,
      queryType: mergedOptions.queryType || QueryType.FACTUAL,
      preferredSources: ["web"],
      maxResults: mergedOptions.maxResults || 10,
      relevanceThreshold: 0.5,
      timeoutMs: 30000,
      metadata: {
        requesterId: "web-navigator",
        priority: 5,
        createdAt: new Date(),
        tags: ["web-search"],
      },
    };

    const knowledgeResponse = await this.knowledgeSeeker.processQuery(
      knowledgeQuery
    );

    // Build enriched results
    const enrichedResults: EnrichedSearchResult[] = [];

    for (const result of knowledgeResponse.results) {
      const enrichedResult: EnrichedSearchResult = {
        ...result,
      };

      // Optionally extract full content for top results
      if (
        (options.enrichContent ?? this.config.autoExtractContent) &&
        enrichedResults.length < this.config.autoExtractCount
      ) {
        try {
          const fullContent = await this.contentExtractor.extractContent(
            result.url,
            this.config.extractionConfig
          );
          enrichedResult.fullContent = fullContent;
        } catch (error: any) {
          enrichedResult.extractionError = error.message;
        }
      }

      enrichedResults.push(enrichedResult);
    }

    const processingTimeMs = Date.now() - startTime;

    const searchResults: SearchResults = {
      query: queryString,
      results: enrichedResults,
      totalFound: knowledgeResponse.metadata.totalResultsFound,
      totalResults: knowledgeResponse.metadata.totalResultsFound, // Alias
      processingTimeMs,
      executionTimeMs: processingTimeMs, // Alias
      cacheUsed: knowledgeResponse.metadata.cacheUsed,
      timestamp: new Date(),
    };

    // Cache results for 1 hour
    this.cache.set(cacheKey, {
      results: searchResults,
      expiresAt: new Date(Date.now() + 3600000),
    });

    return searchResults;
  }

  /**
   * Enrich existing search results with full content
   */
  async enrichResults(
    results: SearchResult[],
    maxToEnrich: number = 3
  ): Promise<EnrichedSearchResult[]> {
    const enriched: EnrichedSearchResult[] = [];

    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      const enrichedResult: EnrichedSearchResult = {
        ...result,
      };

      // Extract full content for top results
      if (i < maxToEnrich) {
        try {
          const fullContent = await this.contentExtractor.extractContent(
            result.url,
            this.config.extractionConfig
          );
          enrichedResult.fullContent = fullContent;
        } catch (error: any) {
          enrichedResult.extractionError = error.message;
        }
      }

      enriched.push(enrichedResult);
    }

    return enriched;
  }

  /**
   * Clear search cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Remove expired cache entries
   */
  pruneCache(): void {
    const now = new Date();
    for (const [key, value] of this.cache.entries()) {
      if (value.expiresAt <= now) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Generate cache key for search query
   */
  private generateCacheKey(
    query: string,
    options: {
      maxResults?: number;
      queryType?: QueryType;
      enrichContent?: boolean;
    }
  ): string {
    return `${query}:${options.maxResults || 10}:${
      options.queryType || "factual"
    }:${options.enrichContent || false}`;
  }
}
