/**
 * @fileoverview Knowledge Seeker Core Component for ARBITER-006
 *
 * Main orchestrator for intelligent information gathering and research,
 * coordinating search providers, processing results, and managing queries.
 *
 * @author @darianrosebrook
 */

import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient";
import { events } from "../orchestrator/EventEmitter";
import { EventTypes } from "../orchestrator/OrchestratorEvents";
import {
  IInformationProcessor,
  IKnowledgeSeeker,
  ISearchProvider,
  KnowledgeQuery,
  KnowledgeResponse,
  KnowledgeSeekerConfig,
  KnowledgeSeekerStatus,
  QueryType,
} from "../types/knowledge";
import { InformationProcessor } from "./InformationProcessor";
import { SearchProviderFactory } from "./SearchProvider";
import type {
  ConversationContext,
  EvidenceManifest,
} from "../verification/types";

/**
 * Knowledge Seeker implementation
 */
export class KnowledgeSeeker implements IKnowledgeSeeker {
  private config: KnowledgeSeekerConfig;
  private providers: Map<string, ISearchProvider> = new Map();
  private processor: IInformationProcessor;
  private activeQueries: Map<string, Promise<KnowledgeResponse>> = new Map();
  private queryCache: Map<string, KnowledgeResponse> = new Map();
  private resultCache: Map<string, any[]> = new Map();
  private dbClient: KnowledgeDatabaseClient | null = null;
  private verificationEngine: any = null; // VerificationEngineImpl

  constructor(
    config: KnowledgeSeekerConfig,
    dbClient?: KnowledgeDatabaseClient,
    verificationEngine?: any // VerificationEngineImpl - using any to avoid circular dependency
  ) {
    this.config = config;
    this.processor = new InformationProcessor(config.processor);
    this.dbClient = dbClient || null;
    this.verificationEngine = verificationEngine || null;
    this.initializeProviders();
  }

  /**
   * Process a knowledge query
   */
  async processQuery(query: KnowledgeQuery): Promise<KnowledgeResponse> {
    const startTime = Date.now();

    // Validate query
    this.validateQuery(query);

    // Check if query is already being processed
    if (this.activeQueries.has(query.id)) {
      return this.activeQueries.get(query.id)!;
    }

    // Check cache first
    if (this.config.caching.enableQueryCaching) {
      const cachedResponse = await this.checkQueryCache(query);
      if (cachedResponse) {
        events.emit({
          id: `event-${Date.now()}-${Math.random()
            .toString(36)
            .substring(2, 9)}`,
          type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
          timestamp: new Date(),
          severity: "info" as any,
          source: "KnowledgeSeeker",
          taskId: query.id,
          metadata: { cacheHit: true, cacheKey: this.generateCacheKey(query) },
        });
        // Mark response as cached
        cachedResponse.metadata.cacheUsed = true;
        return cachedResponse;
      }
    }

    // Create processing promise
    const processingPromise = this.processQueryInternal(query, startTime);
    this.activeQueries.set(query.id, processingPromise);

    try {
      const response = await processingPromise;
      return response;
    } finally {
      // Clean up active query
      this.activeQueries.delete(query.id);
    }
  }

  /**
   * Get seeker status and health information
   */
  async getStatus(): Promise<KnowledgeSeekerStatus> {
    const providerStatuses = await Promise.all(
      Array.from(this.providers.values()).map(async (provider) => {
        const health = await provider.getHealthStatus();
        return {
          name: provider.name,
          available: await provider.isAvailable(),
          health,
        };
      })
    );

    return {
      enabled: this.config.enabled,
      providers: providerStatuses,
      cacheStats: {
        queryCacheSize: this.queryCache.size,
        resultCacheSize: this.resultCache.size,
        hitRate: this.calculateCacheHitRate(),
      },
      processingStats: {
        activeQueries: this.activeQueries.size,
        queuedQueries: 0, // Not implemented yet
        completedQueries: 0, // Would need persistent tracking
        failedQueries: 0, // Would need persistent tracking
      },
    };
  }

  /**
   * Clear all caches
   */
  async clearCaches(): Promise<void> {
    this.queryCache.clear();
    this.resultCache.clear();

    events.emit({
      id: `event-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
      timestamp: new Date(),
      severity: "info" as any,
      source: "KnowledgeSeeker",
      metadata: { action: "cache_cleared" },
    });
  }

  /**
   * Internal query processing logic
   */
  private async processQueryInternal(
    query: KnowledgeQuery,
    startTime: number
  ): Promise<KnowledgeResponse> {
    try {
      // Store query in database if available
      if (this.dbClient && this.dbClient.isAvailable()) {
        await this.dbClient.storeQuery(query);
      }

      // Emit query received event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "KnowledgeSeeker",
        taskId: query.id,
        metadata: {
          queryType: query.queryType,
          maxResults: query.maxResults,
          timeoutMs: query.timeoutMs,
        },
      });

      // Select appropriate providers
      const selectedProviders = this.selectProviders(query);

      if (selectedProviders.length === 0) {
        throw new Error("No suitable search providers available");
      }

      // Execute searches in parallel with timeout
      const searchPromises = selectedProviders.map((provider) =>
        this.executeSearchWithTimeout(provider, query)
      );

      const searchResults = await Promise.allSettled(searchPromises);

      // Collect successful results
      const allResults: any[] = [];
      const providersQueried: string[] = [];

      searchResults.forEach((result, index) => {
        const provider = selectedProviders[index];
        providersQueried.push(provider.name);

        if (result.status === "fulfilled") {
          allResults.push(...result.value);
        } else {
          console.warn(
            `Search failed for provider ${provider.name}:`,
            result.reason
          );

          events.emit({
            id: `event-${Date.now()}-${Math.random()
              .toString(36)
              .substring(2, 9)}`,
            type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
            timestamp: new Date(),
            severity: "warn" as any,
            source: "KnowledgeSeeker",
            taskId: query.id,
            metadata: {
              provider: provider.name,
              error:
                result.reason instanceof Error
                  ? result.reason.message
                  : String(result.reason),
            },
          });
        }
      });

      // Process and filter results
      const processedResults = await this.processor.processResults(
        query,
        allResults
      );

      // Store results in database if available
      if (this.dbClient && this.dbClient.isAvailable()) {
        await this.dbClient.storeResults(processedResults);
      }

      // Verify results if verification is enabled
      let verificationResults: any[] = [];
      let verifiedResults = processedResults;

      if (this.config.verification?.enabled && this.verificationEngine) {
        try {
          // Auto-verify if enabled and query meets criteria
          const shouldVerify =
            this.config.verification.autoVerify &&
            (query.metadata.priority >= 5 ||
              query.queryType === QueryType.FACTUAL);

          if (shouldVerify) {
            verificationResults = await this.verifyResults(
              query,
              processedResults
            );

            // Filter results based on verification confidence if threshold is set
            if (this.config.verification.minConfidenceThreshold) {
              verifiedResults = this.filterByVerificationConfidence(
                processedResults,
                verificationResults,
                this.config.verification.minConfidenceThreshold
              );
            }
          }
        } catch (error) {
          console.warn(
            "Verification failed, continuing with unverified results:",
            error
          );
        }
      }

      // Generate response
      const response: KnowledgeResponse = {
        query,
        results: verifiedResults,
        summary: this.processor.generateSummary(query, verifiedResults),
        confidence: this.calculateResponseConfidence(verifiedResults),
        sourcesUsed: Array.from(new Set(verifiedResults.map((r) => r.domain))),
        verificationResults:
          verificationResults.length > 0 ? verificationResults : undefined,
        metadata: {
          totalResultsFound: allResults.length,
          resultsFiltered: verifiedResults.length,
          processingTimeMs: Date.now() - startTime,
          cacheUsed: false,
          providersQueried,
          verifiedCount: verificationResults.filter(
            (v) =>
              v.confidence >=
              (this.config.verification?.minConfidenceThreshold ?? 0.5)
          ).length,
        },
        respondedAt: new Date(),
      };

      // Store response in database if available
      if (this.dbClient && this.dbClient.isAvailable()) {
        await this.dbClient.storeResponse(response);
        await this.dbClient.updateQueryStatus(query.id, "completed");
      }

      // Cache response if enabled
      if (this.config.caching.enableQueryCaching) {
        await this.cacheQueryResponse(query, response);
      }

      // Emit response generated event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "KnowledgeSeeker",
        taskId: query.id,
        metadata: {
          resultCount: processedResults.length,
          confidence: response.confidence,
          processingTimeMs: response.metadata.processingTimeMs,
        },
      });

      return response;
    } catch (error) {
      const processingTimeMs = Date.now() - startTime;

      // Update query status to failed in database
      if (this.dbClient && this.dbClient.isAvailable()) {
        await this.dbClient.updateQueryStatus(
          query.id,
          "failed",
          error instanceof Error ? error.message : String(error)
        );
      }

      // Emit error event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "error" as any,
        source: "KnowledgeSeeker",
        taskId: query.id,
        metadata: {
          error: error instanceof Error ? error.message : String(error),
          processingTimeMs,
        },
      });

      throw error;
    }
  }

  /**
   * Initialize search providers from configuration
   */
  private initializeProviders(): void {
    for (const providerConfig of this.config.providers) {
      try {
        const provider = SearchProviderFactory.createProvider(providerConfig);
        this.providers.set(provider.name, provider);
      } catch (error) {
        console.error(
          `Failed to initialize provider ${providerConfig.name}:`,
          error
        );
      }
    }

    // Add mock provider for testing if no providers configured
    if (this.providers.size === 0 && this.config.providers.length === 0) {
      const mockProvider = SearchProviderFactory.createMockProvider();
      this.providers.set(mockProvider.name, mockProvider);
    }
  }

  /**
   * Validate query parameters
   */
  private validateQuery(query: KnowledgeQuery): void {
    if (!query.id || query.id.trim().length === 0) {
      throw new Error("Query ID is required");
    }

    if (!query.query || query.query.trim().length === 0) {
      throw new Error("Query text is required");
    }

    if (query.maxResults <= 0 || query.maxResults > 100) {
      throw new Error("maxResults must be between 1 and 100");
    }

    if (query.relevanceThreshold < 0 || query.relevanceThreshold > 1) {
      throw new Error("relevanceThreshold must be between 0 and 1");
    }

    if (query.timeoutMs <= 0 || query.timeoutMs > 300000) {
      // 5 minutes max
      throw new Error("timeoutMs must be between 1 and 300000");
    }

    if (!Object.values(QueryType).includes(query.queryType as QueryType)) {
      throw new Error(`Invalid queryType: ${query.queryType}`);
    }
  }

  /**
   * Select appropriate providers for the query
   */
  private selectProviders(query: KnowledgeQuery): ISearchProvider[] {
    const availableProviders = Array.from(this.providers.values()).filter(
      async (p) => await p.isAvailable()
    );

    // Filter by query type preferences
    let suitableProviders = availableProviders;

    if (query.preferredSources) {
      // Could implement source type filtering here
      // For now, return all available providers
    }

    // Prioritize based on query type
    switch (query.queryType) {
      case QueryType.TECHNICAL:
        // Prefer documentation and code search
        suitableProviders = suitableProviders.filter(
          (p) => p.type === "documentation_search" || p.type === "web_search"
        );
        break;

      case QueryType.FACTUAL:
      case QueryType.EXPLANATORY:
        // Use general web search
        suitableProviders = suitableProviders.filter(
          (p) => p.type === "web_search"
        );
        break;

      default:
        // Use all available providers
        break;
    }

    // Ensure we have at least one provider
    if (suitableProviders.length === 0) {
      suitableProviders = availableProviders.slice(0, 1);
    }

    // Limit concurrent providers to avoid overwhelming
    const maxConcurrent = Math.min(
      suitableProviders.length,
      this.config.queryProcessing.maxConcurrentQueries
    );

    return suitableProviders.slice(0, maxConcurrent);
  }

  /**
   * Execute search with timeout protection
   */
  private async executeSearchWithTimeout(
    provider: ISearchProvider,
    query: KnowledgeQuery
  ): Promise<any[]> {
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new Error(`Search timeout for provider ${provider.name}`));
      }, query.timeoutMs);

      provider
        .search(query)
        .then((results) => {
          clearTimeout(timeoutId);
          resolve(results);
        })
        .catch((error) => {
          clearTimeout(timeoutId);
          reject(error);
        });
    });
  }

  /**
   * Check query cache for existing response
   */
  private async checkQueryCache(
    query: KnowledgeQuery
  ): Promise<KnowledgeResponse | null> {
    const cacheKey = this.generateCacheKey(query);

    // Try database cache first if available
    if (this.dbClient && this.dbClient.isAvailable()) {
      try {
        const cached = await this.dbClient.getCachedResponse(cacheKey);
        if (cached) {
          return cached;
        }
      } catch (error) {
        console.warn("Database cache lookup failed:", error);
      }
    }

    // Fall back to in-memory cache
    const cached = this.queryCache.get(cacheKey);

    if (cached && this.isCacheValid(cached)) {
      // Update cache access time
      cached.respondedAt = new Date();
      return cached;
    }

    return null;
  }

  /**
   * Cache query response
   */
  private async cacheQueryResponse(
    query: KnowledgeQuery,
    response: KnowledgeResponse
  ): Promise<void> {
    const cacheKey = this.generateCacheKey(query);

    // Store in database cache if available
    if (this.dbClient && this.dbClient.isAvailable()) {
      try {
        await this.dbClient.storeCachedResponse(
          cacheKey,
          response,
          this.config.caching.cacheTtlMs
        );
      } catch (error) {
        console.warn("Failed to cache in database:", error);
      }
    }

    // Also store in memory cache for fast access
    this.queryCache.set(cacheKey, response);

    // Clean up expired entries periodically
    if (this.queryCache.size > 100) {
      // Arbitrary cleanup trigger
      this.cleanupExpiredCache();
    }
  }

  /**
   * Generate cache key for query
   */
  private generateCacheKey(query: KnowledgeQuery): string {
    // Create deterministic key based on query content
    const keyData = {
      query: query.query.toLowerCase().trim(),
      queryType: query.queryType,
      maxResults: query.maxResults,
      relevanceThreshold: query.relevanceThreshold,
    };
    return Buffer.from(JSON.stringify(keyData)).toString("base64");
  }

  /**
   * Check if cached response is still valid
   */
  private isCacheValid(response: KnowledgeResponse): boolean {
    // Cache for 1 hour by default
    const cacheAge = Date.now() - response.respondedAt.getTime();
    return cacheAge < (this.config.caching.cacheTtlMs || 3600000);
  }

  /**
   * Calculate response confidence based on result quality
   */
  private calculateResponseConfidence(results: any[]): number {
    if (results.length === 0) return 0;

    const avgRelevance =
      results.reduce((sum, r) => sum + r.relevanceScore, 0) / results.length;
    const avgCredibility =
      results.reduce((sum, r) => sum + r.credibilityScore, 0) / results.length;
    const qualityBonus =
      results.filter((r) => r.quality === "high").length / results.length;

    return Math.min(
      1.0,
      (avgRelevance + avgCredibility) / 2 + qualityBonus * 0.1
    );
  }

  /**
   * Calculate cache hit rate (simplified)
   */
  private calculateCacheHitRate(): number {
    // This would need proper tracking in production
    // For now, return a placeholder
    return 0.0;
  }

  /**
   * Clean up expired cache entries
   */
  private cleanupExpiredCache(): void {
    const now = Date.now();
    for (const [key, response] of Array.from(this.queryCache.entries())) {
      if (
        now - response.respondedAt.getTime() >
        (this.config.caching.cacheTtlMs || 3600000)
      ) {
        this.queryCache.delete(key);
      }
    }
  }

  /**
   * Verify search results using verification engine
   */
  private async verifyResults(
    query: KnowledgeQuery,
    results: any[]
  ): Promise<any[]> {
    if (!this.verificationEngine || !this.config.verification) {
      return [];
    }

    const verificationRequests = results.map((result, idx) => {
      const conversationContext = this.createVerificationConversationContext(
        query,
        result,
        idx
      );
      const evidenceManifest = this.createVerificationEvidenceManifest(result);

      return {
        id: `${query.id}-verify-${idx}`,
        content: result.content || result.snippet || result.title,
        source: result.url,
        context: query.query,
        priority: this.mapQueryPriorityToVerificationPriority(
          query.metadata.priority
        ),
        verificationTypes: this.config.verification?.verificationTypes || [],
        metadata: {
          resultId: result.id,
          provider: result.provider,
          previousMessages: conversationContext.previousMessages,
        },
        conversationContext,
        evidenceManifest,
      };
    });

    // Verify results in parallel
    const verificationResults = await Promise.allSettled(
      verificationRequests.map((req) => this.verificationEngine.verify(req))
    );

    // Extract successful verification results
    return verificationResults
      .filter((r) => r.status === "fulfilled")
      .map((r: any) => r.value);
  }

  /**
   * Map query priority to verification priority
   */
  private mapQueryPriorityToVerificationPriority(
    queryPriority: number
  ): string {
    if (queryPriority >= 8) return "critical";
    if (queryPriority >= 6) return "high";
    if (queryPriority >= 4) return "medium";
    return "low";
  }

  /**
   * Filter results by verification confidence
   */
  private filterByVerificationConfidence(
    results: any[],
    verificationResults: any[],
    minConfidence: number
  ): any[] {
    if (verificationResults.length === 0) {
      return results;
    }

    // Create map of result IDs to verification results
    const verificationMap = new Map();
    for (const verification of verificationResults) {
      const resultId = verification.requestId?.split("-verify-")[0];
      if (resultId) {
        verificationMap.set(verification.requestId, verification);
      }
    }

    // Filter results based on verification confidence
    return results.filter((result, idx) => {
      const verificationKey = `${result.queryId}-verify-${idx}`;
      const verification = verificationMap.get(verificationKey);

      if (!verification) {
        // If no verification available, keep result by default
        return true;
      }

      // Keep results with sufficient verification confidence
      return verification.confidence >= minConfidence;
    });
  }

  private createVerificationConversationContext(
    query: KnowledgeQuery,
    result: any,
    index: number
  ): ConversationContext {
    const conversationId = `${query.id}-result-${result?.id ?? index}`;
    const tenantId = query.metadata.requesterId ?? "knowledge";

    const previousMessages = [
      query.query,
      typeof result?.title === "string" ? result.title : undefined,
      typeof result?.content === "string"
        ? result.content
        : typeof result?.snippet === "string"
        ? result.snippet
        : undefined,
    ].filter((value): value is string => typeof value === "string");

    return {
      conversationId,
      tenantId: String(tenantId),
      previousMessages,
      metadata: {
        queryMetadata: query.metadata,
        provider: result?.provider,
      },
    };
  }

  private createVerificationEvidenceManifest(result: any): EvidenceManifest {
    const reliability =
      typeof result?.credibilityScore === "number"
        ? result.credibilityScore
        : 0.5;

    const lastUpdated = result?.retrievedAt
      ? result.retrievedAt instanceof Date
        ? result.retrievedAt.toISOString()
        : new Date(result.retrievedAt).toISOString()
      : undefined;

    const sources = [
      {
        name: result?.url ?? `provider:${result?.provider ?? "unknown"}`,
        type: result?.sourceType ?? "web",
        reliability,
        lastUpdated,
        responseTime: 0,
      },
    ].filter((source) => !!source.name);

    const evidenceContent =
      result?.content || result?.snippet || result?.title || "";

    const evidence = evidenceContent
      ? [
          {
            content: evidenceContent,
            source: result?.url ?? result?.provider ?? "unknown",
            strength: reliability,
            timestamp: lastUpdated ?? new Date().toISOString(),
            metadata: {
              title: result?.title,
              provider: result?.provider,
            },
          },
        ]
      : [];

    return {
      sources,
      evidence,
      quality: reliability,
      cawsCompliant: false,
    };
  }
}
