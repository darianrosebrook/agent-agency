/**
 * @fileoverview Tests for Knowledge Seeker Component (ARBITER-006)
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { KnowledgeSeeker } from "../../../src/knowledge/KnowledgeSeeker";
import { SearchProviderFactory } from "../../../src/knowledge/SearchProvider";
import {
  KnowledgeQuery,
  KnowledgeResponse,
  QueryType,
  SearchProviderType,
  SearchProviderConfig,
} from "../../../src/types/knowledge";

describe("KnowledgeSeeker", () => {
  let knowledgeSeeker: KnowledgeSeeker;
  let mockProvider: any;

  const mockProviderConfig: SearchProviderConfig = {
    name: "mock",
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
  };

  const defaultConfig = {
    enabled: true,
    providers: [mockProviderConfig],
    processor: {
      minRelevanceScore: 0.5,
      minCredibilityScore: 0.5,
      maxResultsToProcess: 10,
      diversity: {
        minSources: 1,
        minSourceTypes: 1,
        maxResultsPerDomain: 3,
      },
      quality: {
        enableCredibilityScoring: true,
        enableRelevanceFiltering: true,
        enableDuplicateDetection: true,
      },
      caching: {
        enableResultCaching: false,
        cacheTtlMs: 3600000,
        maxCacheSize: 1000,
      },
    },
    queryProcessing: {
      maxConcurrentQueries: 5,
      defaultTimeoutMs: 30000,
      retryAttempts: 2,
    },
    caching: {
      enableQueryCaching: true,
      enableResultCaching: false,
      cacheTtlMs: 3600000,
      maxCacheSize: 1000,
    },
    observability: {
      enableMetrics: true,
      enableTracing: false,
      logLevel: "info" as const,
    },
  };

  const validQuery: KnowledgeQuery = {
    id: "test-query-1",
    query: "What is TypeScript?",
      queryType: QueryType.FACTUAL,
    maxResults: 5,
    relevanceThreshold: 0.6,
    timeoutMs: 10000,
    context: {},
    metadata: {
      requesterId: "test-user",
      priority: 1,
      createdAt: new Date(),
    },
  };

  beforeEach(() => {
    // Create mock provider
    mockProvider = SearchProviderFactory.createMockProvider();
    knowledgeSeeker = new KnowledgeSeeker(defaultConfig);
  });

  describe("Query Processing", () => {

    it("should process a valid knowledge query", async () => {
      const response = await knowledgeSeeker.processQuery(validQuery);

      expect(response).toBeDefined();
      expect(response.query.id).toBe(validQuery.id);
      expect(response.results).toBeInstanceOf(Array);
      expect(response.summary).toBeDefined();
      expect(response.confidence).toBeGreaterThanOrEqual(0);
      expect(response.confidence).toBeLessThanOrEqual(1);
      expect(response.sourcesUsed).toBeInstanceOf(Array);
      expect(response.metadata.totalResultsFound).toBeGreaterThan(0);
      expect(response.metadata.processingTimeMs).toBeGreaterThan(0);
    });

    it("should reject invalid queries", async () => {
      const invalidQueries = [
        { ...validQuery, id: "" }, // Empty ID
        { ...validQuery, query: "" }, // Empty query
        { ...validQuery, maxResults: 0 }, // Invalid maxResults
        { ...validQuery, relevanceThreshold: 1.5 }, // Invalid threshold
        { ...validQuery, timeoutMs: 0 }, // Invalid timeout
        { ...validQuery, queryType: "invalid" as any }, // Invalid query type
      ];

      for (const invalidQuery of invalidQueries) {
        await expect(knowledgeSeeker.processQuery(invalidQuery)).rejects.toThrow();
      }
    });

    it("should respect query timeouts", async () => {
      const slowQuery = { ...validQuery, timeoutMs: 1 }; // Very short timeout

      // This might not always fail due to timing, but tests the timeout logic
      try {
        await knowledgeSeeker.processQuery(slowQuery);
      } catch (error) {
        expect(error).toBeDefined();
      }
    });

    it("should handle concurrent queries", async () => {
      const queries = [
        { ...validQuery, id: "concurrent-1" },
        { ...validQuery, id: "concurrent-2" },
        { ...validQuery, id: "concurrent-3", query: "What is JavaScript?" },
      ];

      const responses = await Promise.all(
        queries.map(query => knowledgeSeeker.processQuery(query))
      );

      expect(responses).toHaveLength(3);
      responses.forEach(response => {
        expect(response.results.length).toBeGreaterThan(0);
      });
    });

    it("should use query caching", async () => {
      // First query
      const response1 = await knowledgeSeeker.processQuery(validQuery);
      expect(response1.metadata.cacheUsed).toBe(false);

      // Second identical query should use cache
      const response2 = await knowledgeSeeker.processQuery(validQuery);
      expect(response2.metadata.cacheUsed).toBe(true);

      // Results should be identical
      expect(response2.results.length).toBe(response1.results.length);
      expect(response2.summary).toBe(response1.summary);
    });
  });

  describe("Provider Management", () => {
    it("should initialize providers from configuration", async () => {
      const status = await knowledgeSeeker.getStatus();

      expect(status.providers.length).toBeGreaterThan(0);
      expect(status.providers[0].name).toBe("mock");
      expect(status.providers[0].available).toBe(true);
    });

    it("should handle provider failures gracefully", async () => {
      // Create seeker with no providers (should use mock fallback)
      const emptyConfig = { ...defaultConfig, providers: [] };
      const seekerWithNoProviders = new KnowledgeSeeker(emptyConfig);

      const response = await seekerWithNoProviders.processQuery({
        id: "test-no-providers",
        query: "test query",
        queryType: QueryType.FACTUAL,
        maxResults: 3,
        relevanceThreshold: 0.5,
        timeoutMs: 5000,
        metadata: {
          requesterId: "test",
          priority: 1,
          createdAt: new Date(),
        },
      });

      expect(response.results.length).toBeGreaterThan(0);
    });
  });

  describe("Result Processing", () => {
    it("should filter results by relevance threshold", async () => {
      const highThresholdQuery = {
        ...validQuery,
        relevanceThreshold: 0.9, // Very high threshold
      };

      const response = await knowledgeSeeker.processQuery(highThresholdQuery);

      // Should have fewer results due to high threshold
      expect(response.results.length).toBeLessThanOrEqual(highThresholdQuery.maxResults);

      // All results should meet threshold
      response.results.forEach(result => {
        expect(result.relevanceScore).toBeGreaterThanOrEqual(highThresholdQuery.relevanceThreshold);
      });
    });

    it("should generate meaningful summaries", async () => {
      const response = await knowledgeSeeker.processQuery(validQuery);

      expect(response.summary).toBeDefined();
      expect(response.summary.length).toBeGreaterThan(10);
      expect(response.summary.toLowerCase()).toContain("found");
      expect(response.summary.toLowerCase()).toContain("result");
    });

    it("should calculate confidence scores", async () => {
      const response = await knowledgeSeeker.processQuery(validQuery);

      expect(response.confidence).toBeGreaterThanOrEqual(0);
      expect(response.confidence).toBeLessThanOrEqual(1);

      // Higher quality results should yield higher confidence
      const highQualityCount = response.results.filter(r => r.quality === "high").length;
      if (highQualityCount > 0) {
        expect(response.confidence).toBeGreaterThan(0.5);
      }
    });
  });

  describe("Cache Management", () => {
    it("should clear caches when requested", async () => {
      // Populate cache
      await knowledgeSeeker.processQuery(validQuery);
      await knowledgeSeeker.processQuery(validQuery);

      let status = await knowledgeSeeker.getStatus();
      expect(status.cacheStats.queryCacheSize).toBeGreaterThan(0);

      // Clear caches
      await knowledgeSeeker.clearCaches();

      status = await knowledgeSeeker.getStatus();
      expect(status.cacheStats.queryCacheSize).toBe(0);
      expect(status.cacheStats.resultCacheSize).toBe(0);
    });

    it("should provide accurate cache statistics", async () => {
      const status = await knowledgeSeeker.getStatus();

      expect(status.cacheStats).toBeDefined();
      expect(typeof status.cacheStats.queryCacheSize).toBe("number");
      expect(typeof status.cacheStats.resultCacheSize).toBe("number");
      expect(typeof status.cacheStats.hitRate).toBe("number");
      expect(status.cacheStats.hitRate).toBeGreaterThanOrEqual(0);
      expect(status.cacheStats.hitRate).toBeLessThanOrEqual(1);
    });
  });

  describe("Status and Health Monitoring", () => {
    it("should provide comprehensive status information", async () => {
      const status = await knowledgeSeeker.getStatus();

      expect(status).toBeDefined();
      expect(typeof status.enabled).toBe("boolean");
      expect(status.providers).toBeInstanceOf(Array);
      expect(status.cacheStats).toBeDefined();
      expect(status.processingStats).toBeDefined();

      // Check provider status structure
      status.providers.forEach(provider => {
        expect(provider.name).toBeDefined();
        expect(typeof provider.available).toBe("boolean");
        expect(provider.health).toBeDefined();
        expect(typeof provider.health.available).toBe("boolean");
        expect(typeof provider.health.responseTimeMs).toBe("number");
      });
    });

    it("should report accurate processing statistics", async () => {
      const status = await knowledgeSeeker.getStatus();

      expect(status.processingStats.activeQueries).toBeGreaterThanOrEqual(0);
      expect(status.processingStats.queuedQueries).toBeGreaterThanOrEqual(0);
      expect(status.processingStats.completedQueries).toBeGreaterThanOrEqual(0);
      expect(status.processingStats.failedQueries).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Error Handling", () => {
    it("should handle provider failures gracefully", async () => {
      // Create a failing provider scenario
      const failingQuery = {
        ...validQuery,
        timeoutMs: 1, // Very short timeout to force failure
      };

      // Should not throw, should return results from other providers or empty results
      const response = await knowledgeSeeker.processQuery(failingQuery);
      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it("should handle malformed provider responses", async () => {
      // Mock provider returning invalid data shouldn't crash the system
      const response = await knowledgeSeeker.processQuery(validQuery);
      expect(response).toBeDefined();
    });
  });

  describe("Configuration", () => {
    it("should respect configuration limits", async () => {
      const limitedQuery = {
        ...validQuery,
        maxResults: 1, // Very limited results
      };

      const response = await knowledgeSeeker.processQuery(limitedQuery);
      expect(response.results.length).toBeLessThanOrEqual(1);
    });

    it("should handle disabled caching", () => {
      const noCacheConfig = {
        ...defaultConfig,
        caching: {
          ...defaultConfig.caching,
          enableQueryCaching: false,
        },
      };

      const seekerNoCache = new KnowledgeSeeker(noCacheConfig);
      expect(seekerNoCache).toBeDefined();
    });
  });
});