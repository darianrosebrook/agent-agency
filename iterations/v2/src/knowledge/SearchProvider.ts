/**
 * @fileoverview Search Provider Abstraction for ARBITER-006
 *
 * Provides a unified interface for different search providers (Google, Bing, etc.)
 * with rate limiting, error handling, and health monitoring.
 *
 * @author @darianrosebrook
 */

import { events } from "../orchestrator/EventEmitter";
import { EventTypes } from "../orchestrator/OrchestratorEvents";
import {
  ISearchProvider,
  KnowledgeQuery,
  ProviderHealthStatus,
  ResultQuality,
  SearchProviderConfig,
  SearchProviderType,
  SearchResult,
  SourceType,
} from "../types/knowledge";

// Import real search providers
import { BingSearchProvider } from "./providers/BingSearchProvider";
import { DuckDuckGoSearchProvider } from "./providers/DuckDuckGoSearchProvider";
import { GoogleSearchProvider } from "./providers/GoogleSearchProvider";

// Use standard RequestInit from DOM lib
type RequestInit = any; // Simplified for Node.js environment

/**
 * Base Search Provider class with common functionality
 */
export abstract class BaseSearchProvider implements ISearchProvider {
  protected config: SearchProviderConfig;
  protected healthStatus: ProviderHealthStatus;
  protected requestCounts: {
    thisMinute: number;
    thisHour: number;
    lastReset: Date;
  };

  constructor(config: SearchProviderConfig) {
    this.config = config;
    this.healthStatus = {
      available: true,
      responseTimeMs: 0,
      errorRate: 0,
      requestsThisMinute: 0,
      requestsThisHour: 0,
    };
    this.requestCounts = {
      thisMinute: 0,
      thisHour: 0,
      lastReset: new Date(),
    };
  }

  get name(): string {
    return this.config.name;
  }

  get type(): SearchProviderType {
    return this.config.type;
  }

  async isAvailable(): Promise<boolean> {
    // Reset counters if needed
    this.resetCountersIfNeeded();

    // Check rate limits
    if (
      this.requestCounts.thisMinute >= this.config.rateLimit.requestsPerMinute
    ) {
      return false;
    }
    if (this.requestCounts.thisHour >= this.config.rateLimit.requestsPerHour) {
      return false;
    }

    // Check health status
    return this.healthStatus.available;
  }

  async getHealthStatus(): Promise<ProviderHealthStatus> {
    return { ...this.healthStatus };
  }

  abstract search(query: KnowledgeQuery): Promise<SearchResult[]>; // eslint-disable-line no-unused-vars

  /**
   * Execute HTTP request with error handling and metrics
   */
  protected async executeRequest<T>(
    url: string,
    options: RequestInit = {},
    timeoutMs: number = 10000
  ): Promise<T> {
    const startTime = Date.now();
    this.requestCounts.thisMinute++;
    this.requestCounts.thisHour++;

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      const responseTime = Date.now() - startTime;

      // Update health metrics
      this.updateHealthMetrics(responseTime, false);

      return data;
    } catch (error) {
      const responseTime = Date.now() - startTime;

      // Update health metrics
      this.updateHealthMetrics(
        responseTime,
        true,
        error instanceof Error ? error.message : String(error)
      );

      throw error;
    }
  }

  /**
   * Update health metrics after a request
   */
  private updateHealthMetrics(
    responseTime: number,
    hadError: boolean,
    errorMessage?: string
  ): void {
    // Update response time (exponential moving average)
    const alpha = 0.1;
    this.healthStatus.responseTimeMs =
      alpha * responseTime + (1 - alpha) * this.healthStatus.responseTimeMs;

    // Update error rate
    const totalRequests =
      this.healthStatus.requestsThisMinute + this.healthStatus.requestsThisHour;
    if (totalRequests > 0) {
      this.healthStatus.errorRate =
        (this.healthStatus.errorRate * (totalRequests - 1) +
          (hadError ? 1 : 0)) /
        totalRequests;
    }

    // Update availability and error tracking
    if (hadError) {
      this.healthStatus.available = false;
      this.healthStatus.lastError = errorMessage;
    } else {
      this.healthStatus.available = true;
      this.healthStatus.lastError = undefined;
    }

    this.healthStatus.requestsThisMinute = this.requestCounts.thisMinute;
    this.healthStatus.requestsThisHour = this.requestCounts.thisHour;
  }

  /**
   * Reset request counters if time windows have passed
   */
  private resetCountersIfNeeded(): void {
    const now = Date.now();
    const minutesSinceReset =
      (now - this.requestCounts.lastReset.getTime()) / (1000 * 60);

    if (minutesSinceReset >= 1) {
      this.requestCounts.thisMinute = 0;
    }

    if (minutesSinceReset >= 60) {
      this.requestCounts.thisHour = 0;
      this.requestCounts.lastReset = new Date();
    }
  }

  /**
   * Create standardized search result from provider-specific data
   */
  protected createSearchResult(
    queryId: string,
    providerData: any,
    relevanceScore: number = 0.5,
    credibilityScore: number = 0.5
  ): SearchResult {
    const content =
      providerData.snippet ||
      providerData.description ||
      providerData.content ||
      "";
    const url = providerData.url || providerData.link || "";
    const title = providerData.title || "Untitled";

    // Generate content hash for duplicate detection
    const contentHash = this.generateContentHash(title, url, content);

    const now = new Date();

    return {
      id: `${this.name}-${Date.now()}-${Math.random()
        .toString(36)
        .substr(2, 9)}`,
      queryId,
      title,
      content,
      url,
      domain: this.extractDomain(url),
      sourceType: this.inferSourceType(providerData),
      relevanceScore,
      credibilityScore,
      quality: this.assessQuality(relevanceScore, credibilityScore),
      publishedAt: providerData.publishedDate
        ? new Date(providerData.publishedDate)
        : undefined,
      provider: this.name,
      providerMetadata: providerData,
      processedAt: now,
      retrievedAt: now,
      contentHash,
    };
  }

  /**
   * Generate content hash for duplicate detection
   */
  private generateContentHash(
    title: string,
    url: string,
    content: string
  ): string {
    // Simple hash function - in production use a proper hashing library like crypto
    const combined = `${title}|${url}|${content}`;
    let hash = 0;
    for (let i = 0; i < combined.length; i++) {
      const char = combined.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash).toString(16);
  }

  /**
   * Extract domain from URL
   */
  private extractDomain(url: string): string {
    try {
      return new URL(url).hostname;
    } catch {
      return "unknown";
    }
  }

  /**
   * Infer source type from result data
   */
  private inferSourceType(providerData: any): SourceType {
    const url = providerData.url || providerData.link || "";
    const domain = this.extractDomain(url).toLowerCase();

    // Academic sources
    if (
      domain.includes("arxiv") ||
      domain.includes("pubmed") ||
      domain.includes("scholar")
    ) {
      return "academic";
    }

    // News sources
    if (
      domain.includes("news") ||
      domain.includes("cnn") ||
      domain.includes("bbc") ||
      domain.includes("nytimes") ||
      domain.includes("reuters")
    ) {
      return "news";
    }

    // Documentation sources
    if (
      domain.includes("docs") ||
      domain.includes("github") ||
      domain.includes("stackoverflow")
    ) {
      return "documentation";
    }

    // Social media
    if (
      domain.includes("twitter") ||
      domain.includes("facebook") ||
      domain.includes("reddit")
    ) {
      return "social";
    }

    // Default to web
    return "web";
  }

  /**
   * Assess result quality based on scores
   */
  private assessQuality(
    relevanceScore: number,
    credibilityScore: number
  ): ResultQuality {
    const combinedScore = (relevanceScore + credibilityScore) / 2;

    if (combinedScore >= 0.8) return ResultQuality.HIGH;
    if (combinedScore >= 0.6) return ResultQuality.MEDIUM;
    if (combinedScore >= 0.3) return ResultQuality.LOW;
    return ResultQuality.UNRELIABLE;
  }
}

/**
 * Google Custom Search Provider
 */
export class GoogleSearchProvider extends BaseSearchProvider {
  constructor(config: SearchProviderConfig) {
    super({ ...config, type: SearchProviderType.WEB_SEARCH });
  }

  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    if (!this.config.apiKey) {
      throw new Error("Google API key not configured");
    }

    const startTime = Date.now();

    try {
      const searchQuery = encodeURIComponent(query.query);
      const url = `https://www.googleapis.com/customsearch/v1?key=${
        this.config.apiKey
      }&cx=${
        this.config.options.searchEngineId
      }&q=${searchQuery}&num=${Math.min(query.maxResults, 10)}`;

      const data = await this.executeRequest<any>(url);

      // Emit search execution event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED, // Using existing event type for now
        timestamp: new Date(),
        severity: "info" as any,
        source: "KnowledgeSeeker",
        taskId: query.id,
        metadata: {
          provider: this.name,
          resultCount: data.items?.length || 0,
          searchTimeMs: Date.now() - startTime,
        },
      });

      return (data.items || []).map((item: any) =>
        this.createSearchResult(
          query.id,
          {
            ...item,
            title: item.title,
            snippet: item.snippet,
            url: item.link,
          },
          0.7, // Default relevance for Google results
          0.8 // Higher credibility for Google
        )
      );
    } catch (error) {
      console.error(`Google search failed for query ${query.id}:`, error);
      return [];
    }
  }
}

/**
 * DuckDuckGo Search Provider (free, no API key required)
 */
export class DuckDuckGoSearchProvider extends BaseSearchProvider {
  constructor(config: SearchProviderConfig) {
    super({ ...config, type: SearchProviderType.WEB_SEARCH });
  }

  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    const startTime = Date.now();

    try {
      const searchQuery = encodeURIComponent(query.query);
      const url = `https://api.duckduckgo.com/?q=${searchQuery}&format=json&no_html=1&skip_disambig=1`;

      const data = await this.executeRequest<any>(url);

      // DuckDuckGo provides instant answers and related topics
      const results: SearchResult[] = [];

      // Add instant answer if available
      if (data.Answer) {
        results.push(
          this.createSearchResult(
            query.id,
            {
              title: "Instant Answer",
              snippet: data.Answer,
              url: data.AnswerURL || `https://duckduckgo.com/?q=${searchQuery}`,
            },
            0.9, // High relevance for instant answers
            0.7 // Good credibility
          )
        );
      }

      // Add related topics
      if (data.RelatedTopics) {
        data.RelatedTopics.slice(0, query.maxResults - results.length).forEach(
          (topic: any) => {
            if (topic.Text && topic.FirstURL) {
              results.push(
                this.createSearchResult(
                  query.id,
                  {
                    title: topic.Text.split(" - ")[0] || "Related Topic",
                    snippet: topic.Text,
                    url: topic.FirstURL,
                  },
                  0.6, // Medium relevance for related topics
                  0.6 // Medium credibility
                )
              );
            }
          }
        );
      }

      // Emit search execution event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "KnowledgeSeeker",
        taskId: query.id,
        metadata: {
          provider: this.name,
          resultCount: results.length,
          searchTimeMs: Date.now() - startTime,
        },
      });

      return results.slice(0, query.maxResults);
    } catch (error) {
      console.error(`DuckDuckGo search failed for query ${query.id}:`, error);
      return [];
    }
  }
}

/**
 * ArXiv Academic Search Provider
 */
export class ArXivSearchProvider extends BaseSearchProvider {
  constructor(config: SearchProviderConfig) {
    super({ ...config, type: SearchProviderType.ACADEMIC_SEARCH });
  }

  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    const startTime = Date.now();

    try {
      const searchQuery = encodeURIComponent(query.query);
      const url = `http://export.arxiv.org/api/query?search_query=all:${searchQuery}&start=0&max_results=${Math.min(
        query.maxResults,
        10
      )}`;

      const response = await fetch(url);
      const xmlText = await response.text();

      // Parse XML response (simplified)
      const results = this.parseArXivResponse(xmlText, query.id);

      // Emit search execution event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "KnowledgeSeeker",
        taskId: query.id,
        metadata: {
          provider: this.name,
          resultCount: results.length,
          searchTimeMs: Date.now() - startTime,
        },
      });

      return results;
    } catch (error) {
      console.error(`ArXiv search failed for query ${query.id}:`, error);
      return [];
    }
  }

  private parseArXivResponse(xmlText: string, queryId: string): SearchResult[] {
    // Simplified XML parsing - in production use a proper XML parser
    const results: SearchResult[] = [];
    const entryRegex = /<entry>(.*?)<\/entry>/g;
    let match;

    while ((match = entryRegex.exec(xmlText)) !== null && results.length < 10) {
      const entryXml = match[1];

      const titleMatch = entryXml.match(/<title>(.*?)<\/title>/);
      const summaryMatch = entryXml.match(/<summary>(.*?)<\/summary>/);
      const idMatch = entryXml.match(/<id>(.*?)<\/id>/);
      const publishedMatch = entryXml.match(/<published>(.*?)<\/published>/);

      if (titleMatch && summaryMatch && idMatch) {
        results.push(
          this.createSearchResult(
            queryId,
            {
              title: titleMatch[1].replace(/<!\[CDATA\[|\]\]>/g, ""),
              snippet: summaryMatch[1]
                .replace(/<!\[CDATA\[|\]\]>/g, "")
                .substring(0, 500),
              url: idMatch[1],
              publishedDate: publishedMatch ? publishedMatch[1] : undefined,
            },
            0.8, // High relevance for academic papers
            0.9 // High credibility for arXiv
          )
        );
      }
    }

    return results;
  }
}

/**
 * Mock Search Provider for testing
 */
export class MockSearchProvider extends BaseSearchProvider {
  constructor(config: SearchProviderConfig) {
    super(config);
  }

  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    // Return mock results for testing
    const mockResults: SearchResult[] = [];

    for (let i = 0; i < Math.min(query.maxResults, 3); i++) {
      mockResults.push(
        this.createSearchResult(
          query.id,
          {
            title: `Mock Result ${i + 1} for "${query.query}"`,
            snippet: `This is a mock search result ${
              i + 1
            } containing information about ${query.query}.`,
            url: `https://example.com/mock-result-${i + 1}`,
          },
          0.7 - i * 0.1, // Decreasing relevance
          0.8 // Good credibility
        )
      );
    }

    // Simulate some delay
    await new Promise((resolve) => setTimeout(resolve, 100));

    return mockResults;
  }
}

/**
 * Search Provider Factory
 */
export class SearchProviderFactory {
  static createProvider(config: SearchProviderConfig): ISearchProvider {
    switch (config.name.toLowerCase()) {
      case "google":
        return new GoogleSearchProvider(config);
      case "bing":
        return new BingSearchProvider(config);
      case "duckduckgo":
        return new DuckDuckGoSearchProvider(config);
      case "arxiv":
        return new ArXivSearchProvider(config);
      case "mock":
        return new MockSearchProvider(config);
      default:
        throw new Error(`Unknown search provider: ${config.name}`);
    }
  }

  static createMockProvider(name: string = "mock"): ISearchProvider {
    return new MockSearchProvider({
      name,
      type: SearchProviderType.WEB_SEARCH,
      endpoint: "mock://",
      rateLimit: {
        requestsPerMinute: 100,
        requestsPerHour: 1000,
      },
      limits: {
        maxResultsPerQuery: 10,
        maxConcurrentQueries: 5,
      },
      options: {},
    });
  }
}
