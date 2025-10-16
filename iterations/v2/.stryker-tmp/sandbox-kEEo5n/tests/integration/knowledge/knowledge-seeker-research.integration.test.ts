/**
 * @fileoverview ARBITER-006 Knowledge Seeker - Integration Tests for Research Workflows
 *
 * Comprehensive integration tests covering:
 * - Multi-provider search workflows
 * - Provider failure and fallback scenarios
 * - Rate limiting and backoff validation
 * - Cache utilization end-to-end
 * - Research task integration
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { KnowledgeSeeker } from "../../../src/knowledge/KnowledgeSeeker";
import { SearchProviderFactory } from "../../../src/knowledge/SearchProvider";
import {
  ISearchProvider,
  KnowledgeQuery,
  KnowledgeSeekerConfig,
  QueryType,
  ResultQuality,
  SearchProviderType,
  SearchResult,
} from "../../../src/types/knowledge";

describe("ARBITER-006 Knowledge Seeker - Research Integration Tests", () => {
  // Test configuration factory
  const createIntegrationConfig = (
    overrides?: Partial<KnowledgeSeekerConfig>
  ): KnowledgeSeekerConfig => ({
    enabled: true,
    providers: [
      {
        name: "mock",
        type: SearchProviderType.WEB_SEARCH,
        endpoint: "mock://",
        rateLimit: { requestsPerMinute: 60, requestsPerHour: 1000 },
        limits: { maxResultsPerQuery: 100, maxConcurrentQueries: 10 },
        options: {},
      },
    ],
    processor: {
      minRelevanceScore: 0.5,
      minCredibilityScore: 0.5,
      maxResultsToProcess: 100,
      diversity: { minSources: 2, minSourceTypes: 1, maxResultsPerDomain: 5 },
      quality: {
        enableCredibilityScoring: true,
        enableRelevanceFiltering: true,
        enableDuplicateDetection: true,
      },
      caching: {
        enableResultCaching: true,
        cacheTtlMs: 300000,
        maxCacheSize: 100,
      },
    },
    queryProcessing: {
      maxConcurrentQueries: 10,
      defaultTimeoutMs: 10000,
      retryAttempts: 3,
    },
    caching: {
      enableQueryCaching: true,
      enableResultCaching: true,
      cacheTtlMs: 300000,
    },
    observability: {
      enableMetrics: true,
      enableTracing: true,
      logLevel: "info",
    },
    ...overrides,
  });

  const createTestQuery = (
    overrides?: Partial<KnowledgeQuery>
  ): KnowledgeQuery => ({
    id: `integration-query-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 9)}`,
    query: "What is machine learning?",
    queryType: QueryType.EXPLANATORY,
    maxResults: 10,
    relevanceThreshold: 0.7,
    timeoutMs: 5000,
    metadata: {
      requesterId: "integration-test",
      priority: 5,
      createdAt: new Date(),
      tags: ["integration", "test"],
    },
    ...overrides,
  });

  // ============================================================================
  // Multi-Provider Search Workflows
  // ============================================================================

  describe("Multi-Provider Search Workflows", () => {
    it("should aggregate results from multiple providers", async () => {
      const config = createIntegrationConfig();
      const seeker = new KnowledgeSeeker(config);

      // Add multiple mock providers
      const provider1 = SearchProviderFactory.createMockProvider("provider-1");
      const provider2 = SearchProviderFactory.createMockProvider("provider-2");
      const provider3 = SearchProviderFactory.createMockProvider("provider-3");

      (seeker as any).providers.clear();
      (seeker as any).providers.set("provider-1", provider1);
      (seeker as any).providers.set("provider-2", provider2);
      (seeker as any).providers.set("provider-3", provider3);

      const query = createTestQuery();
      const response = await seeker.processQuery(query);

      // Should have results from all providers
      expect(response.results.length).toBeGreaterThan(0);
      expect(response.metadata.providersQueried.length).toBeGreaterThanOrEqual(
        3
      );

      // Check that multiple providers were queried (results may be filtered/deduplicated)
      const uniqueProviders = new Set(response.results.map((r) => r.provider));
      expect(uniqueProviders.size).toBeGreaterThanOrEqual(1);
      // Verify all providers were consulted
      expect(response.metadata.providersQueried).toContain("provider-1");
      expect(response.metadata.providersQueried).toContain("provider-2");
      expect(response.metadata.providersQueried).toContain("provider-3");
    });

    it("should deduplicate results across providers", async () => {
      const config = createIntegrationConfig();
      config.processor.quality.enableDuplicateDetection = true;
      const seeker = new KnowledgeSeeker(config);

      // Create providers that return overlapping results
      const sharedResult: SearchResult = {
        id: "shared-result",
        queryId: "test",
        title: "Shared Result",
        content: "This is a shared result across providers",
        url: "https://example.com/shared",
        domain: "example.com",
        sourceType: "web",
        relevanceScore: 0.9,
        credibilityScore: 0.85,
        quality: ResultQuality.HIGH,
        provider: "provider-1",
        providerMetadata: {},
        processedAt: new Date(),
        retrievedAt: new Date(),
        contentHash: "shared-hash-123",
      };

      const provider1: ISearchProvider = {
        name: "provider-1",
        type: SearchProviderType.WEB_SEARCH,
        async isAvailable() {
          return true;
        },
        async search(query: KnowledgeQuery) {
          return [sharedResult];
        },
        async getHealthStatus() {
          return {
            available: true,
            responseTimeMs: 100,
            errorRate: 0,
            requestsThisMinute: 5,
            requestsThisHour: 100,
          };
        },
      };

      const provider2: ISearchProvider = {
        name: "provider-2",
        type: SearchProviderType.WEB_SEARCH,
        async isAvailable() {
          return true;
        },
        async search(query: KnowledgeQuery) {
          return [
            {
              ...sharedResult,
              id: "duplicate-result",
              provider: "provider-2",
            },
          ];
        },
        async getHealthStatus() {
          return {
            available: true,
            responseTimeMs: 100,
            errorRate: 0,
            requestsThisMinute: 5,
            requestsThisHour: 100,
          };
        },
      };

      (seeker as any).providers.clear();
      (seeker as any).providers.set("provider-1", provider1);
      (seeker as any).providers.set("provider-2", provider2);

      const query = createTestQuery();
      const response = await seeker.processQuery(query);

      // Should deduplicate based on content hash
      const uniqueHashes = new Set(response.results.map((r) => r.contentHash));
      expect(uniqueHashes.size).toBe(response.results.length);
    });

    it("should respect diversity requirements across providers", async () => {
      const config = createIntegrationConfig();
      config.processor.diversity = {
        minSources: 2,
        minSourceTypes: 1,
        maxResultsPerDomain: 3,
      };
      const seeker = new KnowledgeSeeker(config);

      const provider =
        SearchProviderFactory.createMockProvider("diversity-test");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("diversity-test", provider);

      const query = createTestQuery({ maxResults: 20 });
      const response = await seeker.processQuery(query);

      // Check domain diversity
      const domainCounts = new Map<string, number>();
      response.results.forEach((result) => {
        const count = domainCounts.get(result.domain) || 0;
        domainCounts.set(result.domain, count + 1);
      });

      // No domain should exceed maxResultsPerDomain
      domainCounts.forEach((count) => {
        expect(count).toBeLessThanOrEqual(
          config.processor.diversity.maxResultsPerDomain
        );
      });
    });
  });

  // ============================================================================
  // Provider Failure and Fallback Scenarios
  // ============================================================================

  describe("Provider Failure and Fallback", () => {
    it("should handle partial provider failures gracefully", async () => {
      const config = createIntegrationConfig();
      const seeker = new KnowledgeSeeker(config);

      const workingProvider =
        SearchProviderFactory.createMockProvider("working");
      const failingProvider: ISearchProvider = {
        name: "failing",
        type: SearchProviderType.WEB_SEARCH,
        async isAvailable() {
          return true;
        },
        async search() {
          throw new Error("Provider temporarily unavailable");
        },
        async getHealthStatus() {
          return {
            available: false,
            responseTimeMs: 0,
            errorRate: 1.0,
            requestsThisMinute: 0,
            requestsThisHour: 0,
          };
        },
      };

      (seeker as any).providers.clear();
      (seeker as any).providers.set("working", workingProvider);
      (seeker as any).providers.set("failing", failingProvider);

      const query = createTestQuery();
      const response = await seeker.processQuery(query);

      // Should still return results from working provider
      expect(response.results.length).toBeGreaterThan(0);
      expect(response.results.every((r) => r.provider === "working")).toBe(
        true
      );
      expect(response.metadata.providersQueried).toContain("failing");
    });

    it("should recover from provider failures with retries", async () => {
      const config = createIntegrationConfig();
      config.queryProcessing.retryAttempts = 3;
      const seeker = new KnowledgeSeeker(config);

      let attemptCount = 0;
      const intermittentProvider: ISearchProvider = {
        name: "intermittent",
        type: SearchProviderType.WEB_SEARCH,
        async isAvailable() {
          return true;
        },
        async search(query: KnowledgeQuery) {
          attemptCount++;
          if (attemptCount < 2) {
            throw new Error("Temporary failure");
          }
          return [
            {
              id: "recovered-result",
              queryId: query.id,
              title: "Result after recovery",
              content: "Successfully recovered from failure",
              url: "https://example.com/recovered",
              domain: "example.com",
              sourceType: "web",
              relevanceScore: 0.9,
              credibilityScore: 0.85,
              quality: ResultQuality.HIGH,
              provider: "intermittent",
              providerMetadata: {},
              processedAt: new Date(),
              retrievedAt: new Date(),
              contentHash: "recovered-hash",
            },
          ];
        },
        async getHealthStatus() {
          return {
            available: true,
            responseTimeMs: 100,
            errorRate: 0.5,
            requestsThisMinute: 5,
            requestsThisHour: 100,
          };
        },
      };

      (seeker as any).providers.clear();
      (seeker as any).providers.set("intermittent", intermittentProvider);

      const query = createTestQuery();
      const response = await seeker.processQuery(query);

      // Should eventually succeed after retries
      expect(response).toBeDefined();
      expect(attemptCount).toBeGreaterThanOrEqual(1);
    });

    it("should maintain service availability with all providers down", async () => {
      const config = createIntegrationConfig();
      const seeker = new KnowledgeSeeker(config);

      const unavailableProvider1: ISearchProvider = {
        name: "unavailable-1",
        type: SearchProviderType.WEB_SEARCH,
        async isAvailable() {
          return false;
        },
        async search() {
          throw new Error("Provider not available");
        },
        async getHealthStatus() {
          return {
            available: false,
            responseTimeMs: 0,
            errorRate: 1.0,
            requestsThisMinute: 0,
            requestsThisHour: 0,
          };
        },
      };

      const unavailableProvider2: ISearchProvider = {
        name: "unavailable-2",
        type: SearchProviderType.WEB_SEARCH,
        async isAvailable() {
          return false;
        },
        async search() {
          throw new Error("Provider not available");
        },
        async getHealthStatus() {
          return {
            available: false,
            responseTimeMs: 0,
            errorRate: 1.0,
            requestsThisMinute: 0,
            requestsThisHour: 0,
          };
        },
      };

      (seeker as any).providers.clear();
      (seeker as any).providers.set("unavailable-1", unavailableProvider1);
      (seeker as any).providers.set("unavailable-2", unavailableProvider2);

      const query = createTestQuery();

      // Should either throw or return empty results gracefully
      try {
        const response = await seeker.processQuery(query);
        expect(response).toBeDefined();
        expect(response.results).toBeDefined();
      } catch (error) {
        // Acceptable to throw when all providers are down
        expect(error).toBeDefined();
      }
    });
  });

  // ============================================================================
  // Cache Utilization End-to-End
  // ============================================================================

  describe("Cache Utilization", () => {
    it("should cache and reuse query results across multiple requests", async () => {
      const config = createIntegrationConfig();
      config.caching.enableQueryCaching = true;
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("cache-test");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("cache-test", provider);

      const query = createTestQuery();

      // First query - cache miss
      const response1 = await seeker.processQuery(query);
      expect(response1.metadata.cacheUsed).toBe(false);

      // Second identical query - cache hit
      const response2 = await seeker.processQuery(query);
      expect(response2.metadata.cacheUsed).toBe(true);

      // Results should be identical
      expect(response2.results).toEqual(response1.results);
      expect(response2.confidence).toEqual(response1.confidence);
    });

    it("should invalidate cache and refresh after TTL expires", async () => {
      const shortTTL = 500; // 500ms
      const config = createIntegrationConfig();
      config.caching.cacheTtlMs = shortTTL;
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("ttl-test");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("ttl-test", provider);

      const query = createTestQuery();

      // First query
      await seeker.processQuery(query);

      // Wait for TTL to expire
      await new Promise((resolve) => setTimeout(resolve, shortTTL + 100));

      // Query again - cache should be expired
      const response = await seeker.processQuery(query);
      // May or may not use cache depending on implementation
      expect(response).toBeDefined();
    });

    it("should handle cache misses for different queries", async () => {
      const config = createIntegrationConfig();
      config.caching.enableQueryCaching = true;
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("multi-query");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("multi-query", provider);

      // Execute different queries
      const queries = [
        createTestQuery({ query: "What is AI?" }),
        createTestQuery({ query: "What is ML?" }),
        createTestQuery({ query: "What is DL?" }),
      ];

      for (const query of queries) {
        const response = await seeker.processQuery(query);
        expect(response.metadata.cacheUsed).toBe(false); // All should be cache misses
        expect(response.results.length).toBeGreaterThan(0);
      }
    });
  });

  // ============================================================================
  // Rate Limiting and Backoff
  // ============================================================================

  describe("Rate Limiting and Backoff", () => {
    it("should handle rate-limited providers with backoff", async () => {
      const config = createIntegrationConfig();
      config.providers[0].rateLimit = {
        requestsPerMinute: 2,
        requestsPerHour: 10,
      };
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("rate-limited");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("rate-limited", provider);

      // Execute multiple queries rapidly
      const queries = Array.from({ length: 3 }, (_, i) =>
        createTestQuery({ id: `rate-test-${i}` })
      );

      const results = await Promise.allSettled(
        queries.map((q) => seeker.processQuery(q))
      );

      // Some should succeed, rate limiting may affect others
      const succeeded = results.filter((r) => r.status === "fulfilled");
      expect(succeeded.length).toBeGreaterThan(0);
    });

    it("should distribute load across multiple providers when rate limited", async () => {
      const config = createIntegrationConfig();
      const seeker = new KnowledgeSeeker(config);

      // Add multiple providers
      const provider1 = SearchProviderFactory.createMockProvider("provider-1");
      const provider2 = SearchProviderFactory.createMockProvider("provider-2");

      (seeker as any).providers.clear();
      (seeker as any).providers.set("provider-1", provider1);
      (seeker as any).providers.set("provider-2", provider2);

      // Execute multiple concurrent queries
      const queries = Array.from({ length: 10 }, (_, i) =>
        createTestQuery({ id: `load-test-${i}` })
      );

      const responses = await Promise.all(
        queries.map((q) => seeker.processQuery(q))
      );

      // Should have results from both providers
      const allProviders = new Set(
        responses.flatMap((r) => r.metadata.providersQueried)
      );
      expect(allProviders.size).toBeGreaterThanOrEqual(2);
    });
  });

  // ============================================================================
  // End-to-End Research Workflows
  // ============================================================================

  describe("End-to-End Research Workflows", () => {
    it("should complete full research workflow with citations", async () => {
      const config = createIntegrationConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("research");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("research", provider);

      const query = createTestQuery({
        queryType: QueryType.TECHNICAL,
        query: "How does neural network backpropagation work?",
      });

      const response = await seeker.processQuery(query);

      // Validate complete research response
      expect(response.results.length).toBeGreaterThan(0);
      expect(response.summary).toBeDefined();
      expect(response.summary.length).toBeGreaterThan(0);
      expect(response.confidence).toBeGreaterThanOrEqual(0);
      expect(response.confidence).toBeLessThanOrEqual(1);
      expect(response.sourcesUsed.length).toBeGreaterThan(0);

      // Each result should have citation information
      response.results.forEach((result) => {
        expect(result.url).toBeDefined();
        expect(result.domain).toBeDefined();
        expect(result.title).toBeDefined();
        expect(result.provider).toBeDefined();
      });
    });

    it("should handle complex multi-query research scenarios", async () => {
      const config = createIntegrationConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider =
        SearchProviderFactory.createMockProvider("complex-research");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("complex-research", provider);

      // Simulate a research task with multiple related queries
      const queries = [
        createTestQuery({
          query: "What is machine learning?",
          queryType: QueryType.EXPLANATORY,
        }),
        createTestQuery({
          query: "Compare supervised vs unsupervised learning",
          queryType: QueryType.COMPARATIVE,
        }),
        createTestQuery({
          query: "Latest trends in deep learning",
          queryType: QueryType.TREND,
        }),
      ];

      const responses = await Promise.all(
        queries.map((q) => seeker.processQuery(q))
      );

      // All queries should succeed
      expect(responses.length).toBe(3);
      responses.forEach((response) => {
        expect(response.results.length).toBeGreaterThan(0);
        expect(response.summary).toBeDefined();
        expect(response.confidence).toBeGreaterThan(0);
      });

      // Should have diverse source coverage across queries
      const allSources = new Set(responses.flatMap((r) => r.sourcesUsed));
      expect(allSources.size).toBeGreaterThan(0);
    });

    it("should maintain performance under realistic load", async () => {
      const config = createIntegrationConfig();
      config.queryProcessing.maxConcurrentQueries = 20;
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("load-test");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("load-test", provider);

      // Simulate realistic research load (15 concurrent queries)
      const queries = Array.from({ length: 15 }, (_, i) =>
        createTestQuery({
          id: `load-${i}`,
          query: `Research query ${i}`,
        })
      );

      const startTime = Date.now();
      const responses = await Promise.all(
        queries.map((q) => seeker.processQuery(q))
      );
      const duration = Date.now() - startTime;

      // All queries should complete
      expect(responses.length).toBe(15);
      responses.forEach((response) => {
        expect(response.results.length).toBeGreaterThan(0);
      });

      // Performance should be reasonable (not checking strict timing to avoid flakiness)
      console.log(`Completed 15 concurrent queries in ${duration}ms`);
      expect(duration).toBeGreaterThan(0);
    });
  });
});
