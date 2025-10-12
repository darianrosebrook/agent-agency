/**
 * @fileoverview Unit tests for SearchEngine
 *
 * Tests search delegation to Knowledge Seeker and content enrichment.
 *
 * @author @darianrosebrook
 */

import { KnowledgeSeeker } from "../../../src/knowledge/KnowledgeSeeker";
import { QueryType, ResultQuality } from "../../../src/types/knowledge";
import { ContentExtractionConfig } from "../../../src/types/web";
import { ContentExtractor } from "../../../src/web/ContentExtractor";
import { SearchEngine } from "../../../src/web/SearchEngine";

// Mock dependencies
jest.mock("../../../src/knowledge/KnowledgeSeeker");
jest.mock("../../../src/web/ContentExtractor");

describe("SearchEngine", () => {
  let searchEngine: SearchEngine;
  let mockKnowledgeSeeker: jest.Mocked<KnowledgeSeeker>;
  let mockContentExtractor: jest.Mocked<ContentExtractor>;
  let defaultExtractionConfig: ContentExtractionConfig;

  beforeEach(() => {
    // Create mocks
    mockKnowledgeSeeker = {
      processQuery: jest.fn(),
    } as any;

    mockContentExtractor = {
      extractContent: jest.fn(),
    } as any;

    defaultExtractionConfig = {
      includeImages: true,
      includeLinks: true,
      includeMetadata: true,
      stripNavigation: true,
      stripAds: true,
      maxContentLength: 10000,
      security: {
        verifySsl: true,
        sanitizeHtml: true,
        detectMalicious: false,
        followRedirects: true,
        maxRedirects: 5,
        userAgent: "Test-Agent/1.0",
        respectRobotsTxt: false,
      },
    };

    searchEngine = new SearchEngine(mockKnowledgeSeeker, mockContentExtractor, {
      autoExtractContent: false,
      autoExtractCount: 3,
      extractionConfig: defaultExtractionConfig,
    });

    jest.clearAllMocks();
  });

  describe("search", () => {
    it("should delegate search to Knowledge Seeker", async () => {
      const mockResults = [
        {
          id: "result-1",
          queryId: "query-1",
          title: "Test Result",
          content: "Test content",
          url: "https://example.com/page1",
          domain: "example.com",
          sourceType: "web" as const,
          relevanceScore: 0.9,
          credibilityScore: 0.8,
          quality: ResultQuality.HIGH,
          provider: "google",
          providerMetadata: {},
          processedAt: new Date(),
          retrievedAt: new Date(),
          contentHash: "hash1",
        },
      ];

      mockKnowledgeSeeker.processQuery.mockResolvedValue({
        query: {
          id: "query-1",
          query: "test query",
          queryType: QueryType.FACTUAL,
          preferredSources: ["web"],
          maxResults: 10,
          relevanceThreshold: 0.5,
          timeoutMs: 30000,
          metadata: {
            requesterId: "web-navigator",
            priority: 5,
            createdAt: new Date(),
            tags: ["web-search"],
          },
        },
        results: mockResults,
        summary: "Test summary",
        confidence: 0.85,
        sourcesUsed: ["example.com"],
        metadata: {
          totalResultsFound: 1,
          resultsFiltered: 0,
          processingTimeMs: 500,
          cacheUsed: false,
          providersQueried: ["google"],
        },
        respondedAt: new Date(),
      });

      const result = await searchEngine.search("test query");

      expect(mockKnowledgeSeeker.processQuery).toHaveBeenCalledWith(
        expect.objectContaining({
          query: "test query",
          queryType: QueryType.FACTUAL,
        })
      );
      expect(result.query).toBe("test query");
      expect(result.results).toHaveLength(1);
      expect(result.totalFound).toBe(1);
    });

    it("should enrich results with full content when requested", async () => {
      const mockResults = [
        {
          id: "result-1",
          queryId: "query-1",
          title: "Test Result",
          content: "Test content",
          url: "https://example.com/page1",
          domain: "example.com",
          sourceType: "web" as const,
          relevanceScore: 0.9,
          credibilityScore: 0.8,
          quality: ResultQuality.HIGH,
          provider: "google",
          providerMetadata: {},
          processedAt: new Date(),
          retrievedAt: new Date(),
          contentHash: "hash1",
        },
      ];

      mockKnowledgeSeeker.processQuery.mockResolvedValue({
        query: {} as any,
        results: mockResults,
        summary: "Test summary",
        confidence: 0.85,
        sourcesUsed: ["example.com"],
        metadata: {
          totalResultsFound: 1,
          resultsFiltered: 0,
          processingTimeMs: 500,
          cacheUsed: false,
          providersQueried: ["google"],
        },
        respondedAt: new Date(),
      });

      mockContentExtractor.extractContent.mockResolvedValue({
        id: "content-1",
        url: "https://example.com/page1",
        title: "Full Page Title",
        content: "Full page content extracted",
        links: [],
        images: [],
        metadata: {} as any,
        quality: "high" as any,
        contentHash: "hash1",
        extractedAt: new Date(),
      });

      const result = await searchEngine.search("test query", {
        enrichContent: true,
      });

      expect(mockContentExtractor.extractContent).toHaveBeenCalled();
      expect(result.results[0].fullContent).toBeDefined();
      expect(result.results[0].fullContent?.content).toBe(
        "Full page content extracted"
      );
    });

    it("should handle content extraction errors gracefully", async () => {
      const mockResults = [
        {
          id: "result-1",
          queryId: "query-1",
          title: "Test Result",
          content: "Test content",
          url: "https://example.com/page1",
          domain: "example.com",
          sourceType: "web" as const,
          relevanceScore: 0.9,
          credibilityScore: 0.8,
          quality: ResultQuality.HIGH,
          provider: "google",
          providerMetadata: {},
          processedAt: new Date(),
          retrievedAt: new Date(),
          contentHash: "hash1",
        },
      ];

      mockKnowledgeSeeker.processQuery.mockResolvedValue({
        query: {} as any,
        results: mockResults,
        summary: "Test summary",
        confidence: 0.85,
        sourcesUsed: ["example.com"],
        metadata: {
          totalResultsFound: 1,
          resultsFiltered: 0,
          processingTimeMs: 500,
          cacheUsed: false,
          providersQueried: ["google"],
        },
        respondedAt: new Date(),
      });

      mockContentExtractor.extractContent.mockRejectedValue(
        new Error("Extraction failed")
      );

      const result = await searchEngine.search("test query", {
        enrichContent: true,
      });

      expect(result.results[0].extractionError).toBe("Extraction failed");
      expect(result.results[0].fullContent).toBeUndefined();
    });

    it("should use cache for repeated searches", async () => {
      const mockResults = [
        {
          id: "result-1",
          queryId: "query-1",
          title: "Test Result",
          content: "Test content",
          url: "https://example.com/page1",
          domain: "example.com",
          sourceType: "web" as const,
          relevanceScore: 0.9,
          credibilityScore: 0.8,
          quality: ResultQuality.HIGH,
          provider: "google",
          providerMetadata: {},
          processedAt: new Date(),
          retrievedAt: new Date(),
          contentHash: "hash1",
        },
      ];

      mockKnowledgeSeeker.processQuery.mockResolvedValue({
        query: {} as any,
        results: mockResults,
        summary: "Test summary",
        confidence: 0.85,
        sourcesUsed: ["example.com"],
        metadata: {
          totalResultsFound: 1,
          resultsFiltered: 0,
          processingTimeMs: 500,
          cacheUsed: false,
          providersQueried: ["google"],
        },
        respondedAt: new Date(),
      });

      // First search
      await searchEngine.search("test query");
      expect(mockKnowledgeSeeker.processQuery).toHaveBeenCalledTimes(1);

      // Second search - should use cache
      await searchEngine.search("test query");
      expect(mockKnowledgeSeeker.processQuery).toHaveBeenCalledTimes(1);
    });

    it("should support different query types", async () => {
      mockKnowledgeSeeker.processQuery.mockResolvedValue({
        query: {} as any,
        results: [],
        summary: "Test summary",
        confidence: 0.85,
        sourcesUsed: [],
        metadata: {
          totalResultsFound: 0,
          resultsFiltered: 0,
          processingTimeMs: 500,
          cacheUsed: false,
          providersQueried: ["google"],
        },
        respondedAt: new Date(),
      });

      await searchEngine.search("test query", {
        queryType: QueryType.TECHNICAL,
      });

      expect(mockKnowledgeSeeker.processQuery).toHaveBeenCalledWith(
        expect.objectContaining({
          queryType: QueryType.TECHNICAL,
        })
      );
    });
  });

  describe("enrichResults", () => {
    it("should enrich specified number of results", async () => {
      const mockResults = [
        {
          id: "result-1",
          queryId: "query-1",
          title: "Result 1",
          content: "Content 1",
          url: "https://example.com/page1",
          domain: "example.com",
          sourceType: "web" as const,
          relevanceScore: 0.9,
          credibilityScore: 0.8,
          quality: ResultQuality.HIGH,
          provider: "google",
          providerMetadata: {},
          processedAt: new Date(),
          retrievedAt: new Date(),
          contentHash: "hash1",
        },
        {
          id: "result-2",
          queryId: "query-1",
          title: "Result 2",
          content: "Content 2",
          url: "https://example.com/page2",
          domain: "example.com",
          sourceType: "web" as const,
          relevanceScore: 0.8,
          credibilityScore: 0.7,
          quality: ResultQuality.MEDIUM,
          provider: "google",
          providerMetadata: {},
          processedAt: new Date(),
          retrievedAt: new Date(),
          contentHash: "hash2",
        },
      ];

      mockContentExtractor.extractContent.mockResolvedValue({
        id: "content-1",
        url: "https://example.com/page1",
        title: "Full Page Title",
        content: "Full page content",
        links: [],
        images: [],
        metadata: {} as any,
        quality: "high" as any,
        contentHash: "hash1",
        extractedAt: new Date(),
      });

      const enriched = await searchEngine.enrichResults(mockResults, 1);

      expect(mockContentExtractor.extractContent).toHaveBeenCalledTimes(1);
      expect(enriched[0].fullContent).toBeDefined();
      expect(enriched[1].fullContent).toBeUndefined();
    });
  });

  describe("cache management", () => {
    it("should clear cache", async () => {
      mockKnowledgeSeeker.processQuery.mockResolvedValue({
        query: {} as any,
        results: [],
        summary: "Test",
        confidence: 0.8,
        sourcesUsed: [],
        metadata: {
          totalResultsFound: 0,
          resultsFiltered: 0,
          processingTimeMs: 500,
          cacheUsed: false,
          providersQueried: [],
        },
        respondedAt: new Date(),
      });

      // Populate cache
      await searchEngine.search("test query");

      // Clear cache
      searchEngine.clearCache();

      // Should make new request
      await searchEngine.search("test query");
      expect(mockKnowledgeSeeker.processQuery).toHaveBeenCalledTimes(2);
    });

    it("should prune expired cache entries", () => {
      // This is a simple test - actual pruning logic would need time manipulation
      expect(() => searchEngine.pruneCache()).not.toThrow();
    });
  });
});
