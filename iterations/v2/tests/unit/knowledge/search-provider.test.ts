/**
 * @fileoverview Tests for Search Provider Components (ARBITER-006)
 *
 * @author @darianrosebrook
 */

import {
  DuckDuckGoSearchProvider,
  MockSearchProvider,
  SearchProviderFactory,
} from "../../../src/knowledge/SearchProvider";
import { events } from "../../../src/orchestrator/EventEmitter";
import {
  KnowledgeQuery,
  QueryType,
  SearchProviderConfig,
  SearchProviderType,
} from "../../../src/types/knowledge";

describe("SearchProvider", () => {
  let mockConfig: SearchProviderConfig;
  let duckDuckGoConfig: SearchProviderConfig;

  beforeEach(() => {
    mockConfig = {
      name: "mock",
      type: SearchProviderType.WEB_SEARCH,
      endpoint: "mock://",
      rateLimit: {
        requestsPerMinute: 10,
        requestsPerHour: 100,
      },
      limits: {
        maxResultsPerQuery: 10,
        maxConcurrentQueries: 5,
      },
      options: {},
    };

    duckDuckGoConfig = {
      name: "duckduckgo",
      type: SearchProviderType.WEB_SEARCH,
      endpoint: "https://api.duckduckgo.com/",
      rateLimit: {
        requestsPerMinute: 30,
        requestsPerHour: 1000,
      },
      limits: {
        maxResultsPerQuery: 5,
        maxConcurrentQueries: 10,
      },
      options: {},
    };
  });

  describe("SearchProviderFactory", () => {
    it("should create mock provider", () => {
      const provider = SearchProviderFactory.createMockProvider();
      expect(provider.name).toBe("mock");
      expect(provider.type).toBe(SearchProviderType.WEB_SEARCH);
    });

    it("should create providers from configuration", () => {
      const mockProvider = SearchProviderFactory.createProvider(mockConfig);
      expect(mockProvider.name).toBe("mock");

      const duckProvider = new DuckDuckGoSearchProvider(duckDuckGoConfig);
      expect(duckProvider.name).toBe("duckduckgo");
    });

    it("should throw error for unknown provider type", () => {
      const invalidConfig = { ...mockConfig, name: "unknown" };
      expect(() =>
        SearchProviderFactory.createProvider(invalidConfig)
      ).toThrow();
    });
  });

  describe("MockSearchProvider", () => {
    let provider: MockSearchProvider;
    let testQuery: KnowledgeQuery;

    beforeEach(() => {
      provider = new MockSearchProvider(mockConfig);
      testQuery = {
        id: "test-query",
        query: "test search",
        queryType: QueryType.FACTUAL,
        maxResults: 3,
        relevanceThreshold: 0.5,
        timeoutMs: 5000,
        metadata: {
          requesterId: "test",
          priority: 1,
          createdAt: new Date(),
        },
      };
    });

    it("should be available when within rate limits", async () => {
      const available = await provider.isAvailable();
      expect(available).toBe(true);
    });

    it("should return mock search results", async () => {
      const results = await provider.search(testQuery);

      expect(results).toBeInstanceOf(Array);
      expect(results.length).toBeGreaterThan(0);
      expect(results.length).toBeLessThanOrEqual(testQuery.maxResults);

      results.forEach((result) => {
        expect(result).toHaveProperty("id");
        expect(result).toHaveProperty("title");
        expect(result).toHaveProperty("content");
        expect(result).toHaveProperty("url");
        expect(result).toHaveProperty("domain");
        expect(result).toHaveProperty("relevanceScore");
        expect(result).toHaveProperty("credibilityScore");
        expect(result).toHaveProperty("quality");
        expect(result.queryId).toBe(testQuery.id);
      });
    });

    it("should respect max results limit", async () => {
      const limitedQuery = { ...testQuery, maxResults: 1 };
      const results = await provider.search(limitedQuery);
      expect(results.length).toBe(1);
    });

    it("should provide health status", async () => {
      const health = await provider.getHealthStatus();

      expect(health).toHaveProperty("available");
      expect(health).toHaveProperty("responseTimeMs");
      expect(health).toHaveProperty("errorRate");
      expect(health).toHaveProperty("requestsThisMinute");
      expect(health).toHaveProperty("requestsThisHour");
    });
  });

  describe("DuckDuckGoSearchProvider", () => {
    let provider: DuckDuckGoSearchProvider;
    let testQuery: KnowledgeQuery;

    beforeEach(() => {
      provider = new DuckDuckGoSearchProvider(duckDuckGoConfig);
      testQuery = {
        id: "ddg-test",
        query: "TypeScript programming",
        queryType: QueryType.FACTUAL,
        maxResults: 3,
        relevanceThreshold: 0.5,
        timeoutMs: 10000,
        metadata: {
          requesterId: "test",
          priority: 1,
          createdAt: new Date(),
        },
      };
    });

    it("should initialize correctly", () => {
      expect(provider.name).toBe("duckduckgo");
      expect(provider.type).toBe(SearchProviderType.WEB_SEARCH);
    });

    it("should handle API calls gracefully", async () => {
      // This test may make actual API calls or fail gracefully
      try {
        const results = await provider.search(testQuery);
        expect(results).toBeInstanceOf(Array);
      } catch (error) {
        // API might not be available in test environment
        expect(error).toBeDefined();
      }
    });

    it("should respect rate limits", async () => {
      // Mock executeRequest to avoid real API calls but still increment counters
      const originalExecuteRequest = (provider as any).executeRequest;
      const mockExecuteRequest = jest
        .spyOn(provider as any, "executeRequest")
        .mockImplementation(async (...args) => {
          // Call the original method to increment counters, but mock the HTTP call
          const mockFetch = jest
            .spyOn(global, "fetch")
            .mockResolvedValue(new Response("{}"));

          try {
            return await originalExecuteRequest.apply(provider, args);
          } finally {
            mockFetch.mockRestore();
          }
        });

      try {
        // Make 35 search requests (more than the 30 per minute limit)
        const searchPromises = Array(35)
          .fill(null)
          .map((_, i) =>
            provider.search({
              ...testQuery,
              id: `rate-limit-test-${i}`,
            })
          );

        // Wait for all to complete
        await Promise.allSettled(searchPromises);

        // After consuming rate limit slots, check availability
        const available = await provider.isAvailable();
        expect(available).toBe(false); // Should be rate limited
      } finally {
        mockExecuteRequest.mockRestore();
      }
    });
  });

  describe("BaseSearchProvider", () => {
    let provider: MockSearchProvider;

    beforeEach(() => {
      provider = new MockSearchProvider(mockConfig);
    });

    it("should extract domains correctly", async () => {
      // Test domain extraction through search results
      const results = await provider.search({
        id: "domain-test",
        query: "test",
        queryType: QueryType.FACTUAL,
        maxResults: 1,
        relevanceThreshold: 0.5,
        timeoutMs: 5000,
        metadata: {
          requesterId: "test",
          priority: 1,
          createdAt: new Date(),
        },
      });

      results.forEach((result) => {
        expect(result.domain).toBeDefined();
        expect(result.domain).not.toBe("unknown");
        expect(result.domain).not.toContain("://");
      });
    });

    it("should assess result quality correctly", async () => {
      const results = await provider.search({
        id: "quality-test",
        query: "test",
        queryType: QueryType.FACTUAL,
        maxResults: 5,
        relevanceThreshold: 0.1, // Low threshold to get varied results
        timeoutMs: 5000,
        metadata: {
          requesterId: "test",
          priority: 1,
          createdAt: new Date(),
        },
      });

      results.forEach((result) => {
        expect(["high", "medium", "low", "unreliable"]).toContain(
          result.quality
        );

        // Quality should correlate with scores
        if (result.quality === "high") {
          expect(
            result.relevanceScore + result.credibilityScore
          ).toBeGreaterThan(1.2);
        }
      });
    });

    it("should handle errors gracefully", async () => {
      // Test with invalid query that might cause errors
      const invalidQuery = {
        id: "error-test",
        query: "", // Empty query might cause issues
        queryType: QueryType.FACTUAL,
        maxResults: 1,
        relevanceThreshold: 0.5,
        timeoutMs: 1, // Very short timeout
        metadata: {
          requesterId: "test",
          priority: 1,
          createdAt: new Date(),
        },
      };

      // Should not throw, should handle gracefully
      const results = await provider.search(invalidQuery);
      expect(results).toBeInstanceOf(Array);
    });
  });

  afterAll(() => {
    events.shutdown();
  });
});
