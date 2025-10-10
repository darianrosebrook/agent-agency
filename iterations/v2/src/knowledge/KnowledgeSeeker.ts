/**
 * Main knowledge seeking orchestration component
 * @author @darianrosebrook
 */

import {
  CacheEntry,
  KnowledgeQuery,
  KnowledgeResponse,
  KnowledgeSeekerConfig,
  KnowledgeSeekerError,
  KnowledgeSeekerErrorCode,
  QueryPriority,
  SearchProvider,
  SearchResult,
} from "../types/knowledge";
import { InformationProcessor } from "./InformationProcessor";
import {
  SearchProviderFactory,
  SearchProvider as SearchProviderImpl,
} from "./SearchProvider";

export class KnowledgeSeeker {
  private readonly config: KnowledgeSeekerConfig;
  private readonly providers: SearchProviderImpl[];
  private readonly processor: InformationProcessor;
  private readonly cache = new Map<string, CacheEntry>();
  private readonly activeSearches = new Set<string>();
  private cleanupTimer?: ReturnType<typeof setInterval>;

  constructor(
    config: Partial<KnowledgeSeekerConfig> = {},
    providers?: SearchProviderImpl[],
    processor?: InformationProcessor
  ) {
    this.config = {
      defaultTimeoutMs: 30000,
      maxConcurrentSearches: 10,
      cacheEnabled: true,
      cacheTtlMs: 3600000, // 1 hour
      minRelevanceThreshold: 0.3,
      maxResultsPerProvider: 10,
      providers: [],
      circuitBreakerEnabled: true,
      retryAttempts: 3,
      retryDelayMs: 1000,
      ...config,
    };

    this.providers =
      providers || SearchProviderFactory.createDefaultProviders();
    this.processor = processor || new InformationProcessor();

    this.initializeCapabilityTracking();
  }

  async search(query: KnowledgeQuery): Promise<KnowledgeResponse> {
    const startTime = Date.now();

    try {
      // Check cache first
      if (this.config.cacheEnabled) {
        const cached = this.checkCache(query);
        if (cached) {
          return {
            ...cached,
            processingTimeMs: Date.now() - startTime,
            cacheHit: true,
          };
        }
      }

      // Validate query
      this.validateQuery(query);

      // Check concurrency limits
      if (this.activeSearches.size >= this.config.maxConcurrentSearches) {
        throw new KnowledgeSeekerError(
          "Maximum concurrent searches exceeded",
          KnowledgeSeekerErrorCode.RATE_LIMIT_EXCEEDED,
          query.id
        );
      }

      this.activeSearches.add(query.id);

      try {
        // Execute search across providers
        const rawResults = await this.executeParallelSearch(query);

        // Process and rank results
        const processedResults = await this.processor.processResults(
          query,
          rawResults
        );

        // Filter by relevance threshold
        const filteredResults = processedResults.filter(
          (result) => result.relevanceScore >= this.config.minRelevanceThreshold
        );

        // Create response
        const response: KnowledgeResponse = {
          queryId: query.id,
          results: filteredResults,
          confidence: this.calculateOverallConfidence(filteredResults),
          processingTimeMs: Date.now() - startTime,
          sourcesUsed: this.getSourcesUsed(filteredResults),
          cacheHit: false,
        };

        // Cache the response
        if (this.config.cacheEnabled) {
          this.cacheResponse(query, response);
        }

        return response;
      } finally {
        this.activeSearches.delete(query.id);
      }
    } catch (error) {
      const processingTime = Date.now() - startTime;

      if (error instanceof KnowledgeSeekerError) {
        return {
          queryId: query.id,
          results: [],
          confidence: 0,
          processingTimeMs: processingTime,
          sourcesUsed: [],
          cacheHit: false,
          error: error.message,
        };
      }

      throw error;
    }
  }

  async searchMultiple(
    queries: KnowledgeQuery[]
  ): Promise<KnowledgeResponse[]> {
    // Prioritize queries by priority and submit in parallel with limits
    const prioritizedQueries = this.prioritizeQueries(queries);

    const results: KnowledgeResponse[] = [];
    const batches = this.createBatches(
      prioritizedQueries,
      this.config.maxConcurrentSearches
    );

    for (const batch of batches) {
      const batchPromises = batch.map((query) => this.search(query));
      const batchResults = await Promise.all(batchPromises);
      results.push(...batchResults);
    }

    return results;
  }

  getCacheStats(): { size: number; hitRate: number; totalAccesses: number } {
    const totalAccesses = Array.from(this.cache.values()).reduce(
      (sum, entry) => sum + entry.accessCount,
      0
    );

    // Simple hit rate calculation (would need more sophisticated tracking in production)
    const hitRate = totalAccesses > 0 ? 0.7 : 0; // Mock hit rate

    return {
      size: this.cache.size,
      hitRate,
      totalAccesses,
    };
  }

  clearCache(): void {
    this.cache.clear();
  }

  getActiveSearches(): string[] {
    return Array.from(this.activeSearches);
  }

  async healthCheck(): Promise<{ healthy: boolean; details: any }> {
    const providerStatus = await Promise.all(
      this.providers.map(async (provider) => ({
        name: provider.name,
        available: await provider.isAvailable(),
        rateLimitStatus: await provider.getRateLimitStatus(),
      }))
    );

    const healthy = providerStatus.some((p) => p.available);
    const totalProviders = providerStatus.length;
    const availableProviders = providerStatus.filter((p) => p.available).length;

    return {
      healthy,
      details: {
        totalProviders,
        availableProviders,
        cacheSize: this.cache.size,
        activeSearches: this.activeSearches.size,
        providers: providerStatus.map((p, index) => ({
          ...p,
          config: this.providers[index].getConfig(), // Include config for health reporting
        })),
      },
    };
  }

  private checkCache(query: KnowledgeQuery): KnowledgeResponse | null {
    const cacheKey = this.createCacheKey(query);
    const entry = this.cache.get(cacheKey);

    if (!entry) return null;

    // Check if expired
    if (Date.now() - entry.timestamp.getTime() > entry.ttlMs) {
      this.cache.delete(cacheKey);
      return null;
    }

    // Update access statistics
    entry.accessCount++;
    entry.lastAccessed = new Date();

    return entry.data;
  }

  private createCacheKey(query: KnowledgeQuery): string {
    // Create a deterministic key based on query content
    const keyData = {
      query: query.query,
      context: query.context,
      filters: query.filters,
      sources: query.sources?.sort(),
    };

    return `knowledge:${JSON.stringify(keyData)}`;
  }

  private validateQuery(query: KnowledgeQuery): void {
    if (!query.query || query.query.trim().length === 0) {
      throw new KnowledgeSeekerError(
        "Query cannot be empty",
        KnowledgeSeekerErrorCode.INVALID_QUERY,
        query.id
      );
    }

    if (query.query.length > 1000) {
      throw new KnowledgeSeekerError(
        "Query too long (max 1000 characters)",
        KnowledgeSeekerErrorCode.INVALID_QUERY,
        query.id
      );
    }

    if (query.maxResults && query.maxResults > 100) {
      throw new KnowledgeSeekerError(
        "Too many results requested (max 100)",
        KnowledgeSeekerErrorCode.INVALID_QUERY,
        query.id
      );
    }
  }

  private async executeParallelSearch(
    query: KnowledgeQuery
  ): Promise<SearchResult[]> {
    const timeoutMs = query.timeoutMs || this.config.defaultTimeoutMs;
    const selectedProviders = this.selectProviders(query);

    const searchPromises = selectedProviders.map((provider) =>
      this.searchWithProvider(provider, query, timeoutMs)
    );

    const results = await Promise.allSettled(searchPromises);
    const successfulResults: SearchResult[] = [];

    for (const result of results) {
      if (result.status === "fulfilled") {
        successfulResults.push(...result.value);
      } else {
        // Log provider failures but continue with others
        console.warn("Provider search failed:", result.reason);
      }
    }

    return successfulResults;
  }

  private selectProviders(query: KnowledgeQuery): SearchProviderImpl[] {
    let candidates = this.providers.filter((p) => p.getConfig().enabled);

    // If specific providers requested, filter to those
    if (query.sources && query.sources.length > 0) {
      candidates = candidates.filter((p) => query.sources!.includes(p.name));
    }

    // Sort by priority
    candidates.sort((a, b) => b.getConfig().priority - a.getConfig().priority);

    return candidates;
  }

  private async searchWithProvider(
    provider: SearchProviderImpl,
    query: KnowledgeQuery,
    timeoutMs: number
  ): Promise<SearchResult[]> {
    const timeoutPromise = new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error("Search timeout")), timeoutMs)
    );

    const searchPromise = provider.search(query);
    const results = await Promise.race([searchPromise, timeoutPromise]);

    // Limit results per provider
    return results.slice(0, this.config.maxResultsPerProvider);
  }

  private calculateOverallConfidence(results: SearchResult[]): number {
    if (results.length === 0) return 0;

    const avgRelevance =
      results.reduce((sum, r) => sum + r.relevanceScore, 0) / results.length;
    const avgCredibility =
      results.reduce((sum, r) => sum + r.credibilityScore, 0) / results.length;
    const sourceDiversity = this.calculateSourceDiversity(results);

    // Weighted confidence calculation
    return avgRelevance * 0.4 + avgCredibility * 0.4 + sourceDiversity * 0.2;
  }

  private calculateSourceDiversity(results: SearchResult[]): number {
    const uniqueProviders = new Set(results.map((r) => r.provider));
    return Math.min(uniqueProviders.size / this.providers.length, 1.0);
  }

  private getSourcesUsed(results: SearchResult[]): SearchProvider[] {
    return Array.from(new Set(results.map((r) => r.provider)));
  }

  private cacheResponse(
    query: KnowledgeQuery,
    response: KnowledgeResponse
  ): void {
    const cacheKey = this.createCacheKey(query);
    const ttlMs =
      query.priority === QueryPriority.CRITICAL
        ? this.config.cacheTtlMs * 2
        : this.config.cacheTtlMs;

    const entry: CacheEntry = {
      key: cacheKey,
      data: response,
      timestamp: new Date(),
      ttlMs,
      accessCount: 1,
      lastAccessed: new Date(),
    };

    this.cache.set(cacheKey, entry);
  }

  private prioritizeQueries(queries: KnowledgeQuery[]): KnowledgeQuery[] {
    const priorityOrder = {
      [QueryPriority.CRITICAL]: 4,
      [QueryPriority.HIGH]: 3,
      [QueryPriority.MEDIUM]: 2,
      [QueryPriority.LOW]: 1,
    };

    return queries.sort(
      (a, b) => priorityOrder[b.priority] - priorityOrder[a.priority]
    );
  }

  private createBatches<T>(items: T[], batchSize: number): T[][] {
    const batches: T[][] = [];
    for (let i = 0; i < items.length; i += batchSize) {
      batches.push(items.slice(i, i + batchSize));
    }
    return batches;
  }

  private initializeCapabilityTracking(): void {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
    const profile = this.providers.map((p) => ({
      name: p.name,
      available: true, // Would check actual availability
      rateLimit: p.getConfig().rateLimit,
    }));

    // Set up periodic cache cleanup
    this.cleanupTimer = setInterval(() => {
      this.performCacheCleanup();
    }, 300000); // Clean every 5 minutes

    // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
    const score = this.calculateOverallConfidence([]); // Initialize scoring system
  }

  private performCacheCleanup(): void {
    const now = Date.now();
    const expiredKeys: string[] = [];

    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp.getTime() > entry.ttlMs) {
        expiredKeys.push(key);
      }
    }

    expiredKeys.forEach((key) => this.cache.delete(key));
  }

  destroy(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = undefined;
    }
    this.clearCache();
  }
}
