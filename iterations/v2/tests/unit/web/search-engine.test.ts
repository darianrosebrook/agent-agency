/**
 * @fileoverview Unit tests for SearchEngine
 *
 * Tests search query execution, result processing, and ranking.
 *
 * @author @darianrosebrook
 */

import axios from "axios";
import { SearchEngineConfig, SearchQuery } from "../../../src/types/web";
import { SearchEngine } from "../../../src/web/SearchEngine";

// Mock axios
jest.mock("axios");
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe("SearchEngine", () => {
  let searchEngine: SearchEngine;
  let mockConfig: SearchEngineConfig;

  beforeEach(() => {
    mockConfig = {
      apiKey: "test-api-key",
      searchEngineUrl: "https://api.searchengine.com/search",
      maxResults: 10,
      timeoutMs: 5000,
      retries: 3,
      userAgent: "TestSearchAgent/1.0",
    };

    searchEngine = new SearchEngine(mockConfig);

    // Reset all mocks
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should initialize with valid configuration", () => {
      expect(searchEngine).toBeDefined();
      expect(searchEngine).toBeInstanceOf(SearchEngine);
    });

    it("should throw error with missing API key", () => {
      const invalidConfig = { ...mockConfig, apiKey: "" };
      expect(() => new SearchEngine(invalidConfig)).toThrow(/API key required/);
    });

    it("should configure axios client properly", () => {
      expect(mockedAxios.create).toHaveBeenCalledWith(
        expect.objectContaining({
          timeout: mockConfig.timeoutMs,
          headers: expect.objectContaining({
            "User-Agent": mockConfig.userAgent,
          }),
        })
      );
    });
  });

  describe("search execution", () => {
    const mockQuery: SearchQuery = {
      query: "test search query",
      maxResults: 5,
      language: "en",
      region: "us",
      safeSearch: true,
      excludeDomains: ["spam.com"],
    };

    const mockApiResponse = {
      items: [
        {
          title: "Test Result 1",
          link: "https://example.com/result1",
          snippet: "This is the first test result snippet.",
          displayLink: "example.com",
          formattedUrl: "https://example.com/result1",
          htmlTitle: "<b>Test</b> Result 1",
          htmlSnippet: "This is the <b>first</b> test result snippet.",
          cacheId: "cache1",
          pagemap: {
            metatags: [{ "og:title": "Test Result 1" }],
          },
        },
        {
          title: "Test Result 2",
          link: "https://example.com/result2",
          snippet: "This is the second test result snippet.",
          displayLink: "example.com",
          formattedUrl: "https://example.com/result2",
          htmlTitle: "Test Result 2",
          htmlSnippet: "This is the second test result snippet.",
          cacheId: "cache2",
        },
      ],
      searchInformation: {
        searchTime: 0.5,
        formattedSearchTime: "0.50",
        totalResults: "2",
        formattedTotalResults: "2",
      },
      queries: {
        request: [
          {
            title: "Google Custom Search - test search query",
            searchTerms: "test search query",
            count: 5,
            startIndex: 1,
            inputEncoding: "utf8",
            outputEncoding: "utf8",
            safe: "active",
            cx: "test-cx",
          },
        ],
      },
    };

    beforeEach(() => {
      mockedAxios.get.mockResolvedValue({
        data: mockApiResponse,
        status: 200,
        statusText: "OK",
        headers: {},
        config: {},
      } as any);
    });

    it("should execute search successfully", async () => {
      const results = await searchEngine.search(mockQuery);

      expect(results).toBeDefined();
      expect(results.query).toBe(mockQuery.query);
      expect(results.totalResults).toBe(2);
      expect(results.results.length).toBe(2);
      expect(results.executionTimeMs).toBeGreaterThan(0);
    });

    it("should return properly formatted search results", async () => {
      const results = await searchEngine.search(mockQuery);

      expect(results.results[0]).toEqual(
        expect.objectContaining({
          title: "Test Result 1",
          url: "https://example.com/result1",
          snippet: "This is the first test result snippet.",
          displayUrl: "example.com",
          rank: 1,
        })
      );
    });

    it("should respect maxResults parameter", async () => {
      const limitedQuery = { ...mockQuery, maxResults: 1 };
      const results = await searchEngine.search(limitedQuery);

      expect(results.results.length).toBe(1);
    });

    it("should handle empty search results", async () => {
      mockedAxios.get.mockResolvedValue({
        data: { items: [] },
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search(mockQuery);

      expect(results.totalResults).toBe(0);
      expect(results.results.length).toBe(0);
    });

    it("should handle API errors gracefully", async () => {
      mockedAxios.get.mockRejectedValue(new Error("API Error"));

      await expect(searchEngine.search(mockQuery)).rejects.toThrow();
    });

    it("should respect timeout configuration", async () => {
      mockedAxios.get.mockImplementation(
        () =>
          new Promise((resolve) =>
            setTimeout(
              () =>
                resolve({
                  data: mockApiResponse,
                  status: 200,
                  headers: {},
                  config: {},
                }),
              10000
            )
          )
      );

      const timeoutConfig = { ...mockConfig, timeoutMs: 100 };
      const timeoutSearchEngine = new SearchEngine(timeoutConfig);

      await expect(timeoutSearchEngine.search(mockQuery)).rejects.toThrow();
    });

    it("should implement retry logic", async () => {
      let callCount = 0;
      mockedAxios.get.mockImplementation(() => {
        callCount++;
        if (callCount < 3) {
          return Promise.reject(new Error("Temporary failure"));
        }
        return Promise.resolve({
          data: mockApiResponse,
          status: 200,
          headers: {},
          config: {},
        } as any);
      });

      const results = await searchEngine.search(mockQuery);

      expect(mockedAxios.get).toHaveBeenCalledTimes(3);
      expect(results).toBeDefined();
    });

    it("should filter excluded domains", async () => {
      const responseWithExcluded = {
        ...mockApiResponse,
        items: [
          ...mockApiResponse.items,
          {
            title: "Spam Result",
            link: "https://spam.com/result",
            snippet: "This should be filtered out.",
            displayLink: "spam.com",
            formattedUrl: "https://spam.com/result",
          },
        ],
      };

      mockedAxios.get.mockResolvedValue({
        data: responseWithExcluded,
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search(mockQuery);

      // Should not include the spam.com result
      expect(results.results.some((r) => r.url.includes("spam.com"))).toBe(
        false
      );
    });
  });

  describe("result ranking", () => {
    it("should rank results by relevance", async () => {
      const mockResponse = {
        items: [
          {
            title: "Perfect Match",
            link: "https://example.com/perfect",
            snippet: "This result perfectly matches the search query.",
          },
          {
            title: "Partial Match",
            link: "https://example.com/partial",
            snippet: "This result partially matches some terms.",
          },
          {
            title: "Poor Match",
            link: "https://example.com/poor",
            snippet: "This result barely matches anything.",
          },
        ],
      };

      mockedAxios.get.mockResolvedValue({
        data: mockResponse,
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search({
        query: "perfect match",
        maxResults: 10,
      });

      expect(results.results[0].title).toBe("Perfect Match");
      expect(results.results[0].rank).toBe(1);
      expect(results.results[1].rank).toBe(2);
      expect(results.results[2].rank).toBe(3);
    });

    it("should assign relevance scores", async () => {
      const mockResponse = {
        items: [
          {
            title: "Highly Relevant",
            link: "https://example.com/high",
            snippet: "This contains all the search terms in order.",
          },
        ],
      };

      mockedAxios.get.mockResolvedValue({
        data: mockResponse,
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search({
        query: "highly relevant",
        maxResults: 10,
      });

      expect(results.results[0]).toHaveProperty("relevanceScore");
      expect(results.results[0].relevanceScore).toBeGreaterThan(0);
      expect(results.results[0].relevanceScore).toBeLessThanOrEqual(1);
    });
  });

  describe("search query validation", () => {
    it("should validate search queries", () => {
      expect(() => searchEngine.search({ query: "" })).rejects.toThrow(
        /Query required/
      );
      expect(() => searchEngine.search({ query: "   " })).rejects.toThrow(
        /Query required/
      );
    });

    it("should sanitize search queries", async () => {
      const maliciousQuery = {
        query: '<script>alert("xss")</script> test query',
        maxResults: 5,
      };

      mockedAxios.get.mockResolvedValue({
        data: { items: [] },
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search(maliciousQuery);

      // Verify the query was sanitized before sending to API
      expect(mockedAxios.get).toHaveBeenCalledWith(
        expect.stringContaining("test%20query"),
        expect.any(Object)
      );
      expect(mockedAxios.get).not.toHaveBeenCalledWith(
        expect.stringContaining("<script>"),
        expect.any(Object)
      );
    });

    it("should handle special characters in queries", async () => {
      const specialQuery = {
        query: "C++ programming & web development",
        maxResults: 5,
      };

      mockedAxios.get.mockResolvedValue({
        data: { items: [] },
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search(specialQuery);

      expect(results).toBeDefined();
      expect(mockedAxios.get).toHaveBeenCalledWith(
        expect.stringContaining(
          "C%2B%2B%20programming%20%26%20web%20development"
        ),
        expect.any(Object)
      );
    });
  });

  describe("performance monitoring", () => {
    it("should track search execution time", async () => {
      const startTime = Date.now();
      const results = await searchEngine.search({
        query: "test query",
        maxResults: 5,
      });
      const endTime = Date.now();

      expect(results.executionTimeMs).toBeGreaterThanOrEqual(0);
      expect(results.executionTimeMs).toBeLessThanOrEqual(
        endTime - startTime + 100
      ); // Allow some tolerance
    });

    it("should include performance metadata", async () => {
      const results = await searchEngine.search({
        query: "performance test",
        maxResults: 5,
      });

      expect(results).toHaveProperty("executionTimeMs");
      expect(results).toHaveProperty("timestamp");
      expect(results.timestamp).toBeInstanceOf(Date);
    });
  });

  describe("error handling", () => {
    it("should handle network errors", async () => {
      mockedAxios.get.mockRejectedValue(new Error("Network timeout"));

      await expect(searchEngine.search({ query: "test" })).rejects.toThrow(
        /Network timeout/
      );
    });

    it("should handle API rate limiting", async () => {
      mockedAxios.get.mockResolvedValue({
        data: {},
        status: 429,
        statusText: "Too Many Requests",
        headers: {},
        config: {},
      } as any);

      await expect(searchEngine.search({ query: "test" })).rejects.toThrow(
        /rate limit/i
      );
    });

    it("should handle malformed API responses", async () => {
      mockedAxios.get.mockResolvedValue({
        data: { invalid: "response" },
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search({ query: "test" });

      expect(results.totalResults).toBe(0);
      expect(results.results.length).toBe(0);
    });

    it("should handle partial API failures", async () => {
      const partialResponse = {
        items: [
          { title: "Valid Result", link: "https://example.com" },
          { invalid: "result" }, // Missing required fields
        ],
      };

      mockedAxios.get.mockResolvedValue({
        data: partialResponse,
        status: 200,
        headers: {},
        config: {},
      } as any);

      const results = await searchEngine.search({ query: "test" });

      // Should include valid results and skip invalid ones
      expect(results.results.length).toBe(1);
      expect(results.results[0].title).toBe("Valid Result");
    });
  });
});
