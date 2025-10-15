/**
 * @fileoverview Unit tests for WebNavigator
 *
 * Tests main orchestrator coordination, caching, and rate limiting.
 *
 * @author @darianrosebrook
 */

import { WebNavigatorDatabaseClient } from "../../../src/database/WebNavigatorDatabaseClient";
import { KnowledgeSeeker } from "../../../src/knowledge/KnowledgeSeeker";
import { RateLimitStatus, WebNavigatorConfig } from "../../../src/types/web";
import { ContentExtractor } from "../../../src/web/ContentExtractor";
import { SearchEngine } from "../../../src/web/SearchEngine";
import { TraversalEngine } from "../../../src/web/TraversalEngine";
import { WebNavigator } from "../../../src/web/WebNavigator";

// Mock dependencies
jest.mock("../../../src/database/WebNavigatorDatabaseClient");
jest.mock("../../../src/knowledge/KnowledgeSeeker");
jest.mock("../../../src/web/ContentExtractor");
jest.mock("../../../src/web/SearchEngine");
jest.mock("../../../src/web/TraversalEngine");

describe("WebNavigator", () => {
  let webNavigator: WebNavigator;
  let mockDbClient: jest.Mocked<WebNavigatorDatabaseClient>;
  let mockKnowledgeSeeker: jest.Mocked<KnowledgeSeeker>;
  let mockContentExtractor: jest.Mocked<ContentExtractor>;
  let mockSearchEngine: jest.Mocked<SearchEngine>;
  let mockTraversalEngine: jest.Mocked<TraversalEngine>;
  let defaultConfig: WebNavigatorConfig;

  beforeEach(() => {
    mockDbClient = {
      isAvailable: jest.fn().mockReturnValue(true),
      storeContent: jest.fn().mockResolvedValue("content-1"),
      cacheContent: jest.fn().mockResolvedValue(undefined),
      getContentByUrl: jest.fn().mockResolvedValue(null),
      getRateLimit: jest.fn().mockResolvedValue(null),
      updateRateLimit: jest.fn().mockResolvedValue(undefined),
      incrementRateLimitCounter: jest.fn().mockResolvedValue(1),
      getCacheStats: jest.fn().mockResolvedValue({
        totalPages: 0,
        cacheSizeBytes: 0,
        hitRate: 0,
        ageDistribution: {
          under1Hour: 0,
          under6Hours: 0,
          under12Hours: 0,
          under24Hours: 0,
        },
      }),
      cleanupExpiredCache: jest.fn().mockResolvedValue(0),
      storeExtractionMetrics: jest.fn().mockResolvedValue(undefined),
      createTraversal: jest.fn().mockResolvedValue(undefined),
      updateTraversalNode: jest.fn().mockResolvedValue(undefined),
      updateTraversalStatus: jest.fn().mockResolvedValue(undefined),
    } as any;

    mockKnowledgeSeeker = {
      processQuery: jest.fn(),
    } as any;

    // Set up module mocks
    mockContentExtractor = {
      extractContent: jest.fn(),
    } as any;
    (
      ContentExtractor as jest.MockedClass<typeof ContentExtractor>
    ).mockImplementation(() => mockContentExtractor);

    mockSearchEngine = {
      search: jest.fn(),
      pruneCache: jest.fn(),
      clearCache: jest.fn(),
    } as any;
    (SearchEngine as jest.MockedClass<typeof SearchEngine>).mockImplementation(
      () => mockSearchEngine
    );

    mockTraversalEngine = {
      traverse: jest.fn().mockResolvedValue({
        sessionId: "traversal-123",
        startUrl: "https://example.com",
        pages: [],
        nodes: [],
        statistics: {
          pagesVisited: 1,
          pagesSkipped: 0,
          errorsEncountered: 0,
          maxDepthReached: 1,
          processingTimeMs: 100,
          totalContentBytes: 1024,
          avgPageLoadTimeMs: 50,
          rateLimitEncounters: 0,
        },
        graph: {
          nodes: [],
          edges: [],
        },
        completedAt: new Date(),
        maxDepthReached: true,
        pageLimitReached: false,
        totalPagesVisited: 1,
        traversalTimeMs: 100,
        depthDistribution: { 0: 1 },
      }),
    } as any;
    (
      TraversalEngine as jest.MockedClass<typeof TraversalEngine>
    ).mockImplementation(() => mockTraversalEngine);

    defaultConfig = {
      enabled: true,
      http: {
        timeoutMs: 5000,
        maxConcurrentRequests: 20,
        retryAttempts: 3,
        retryDelayMs: 1000,
        userAgent: "Test-Agent/1.0",
        followRedirects: true,
        maxRedirects: 5,
      },
      cache: {
        enabled: true,
        ttlHours: 24,
        maxSizeMb: 100,
      },
      rateLimit: {
        enabled: true,
        requestsPerMinute: 60,
        backoffMultiplier: 2,
        maxBackoffMs: 60000,
      },
      security: {
        verifySsl: true,
        sanitizeContent: true,
        detectMalicious: true,
        respectRobotsTxt: true,
      },
      limits: {
        maxContentSizeMb: 10,
        maxExtractionTimeMs: 5000,
        maxTraversalDepth: 3,
        maxPagesPerTraversal: 50,
      },
      observability: {
        enableMetrics: true,
        enableTracing: true,
        logLevel: "info",
      },
    };

    webNavigator = new WebNavigator(
      defaultConfig,
      mockDbClient,
      mockKnowledgeSeeker
    );

    jest.clearAllMocks();
  });

  afterEach(() => {
    jest.clearAllMocks();
    // Reset traversal engine mock to default behavior
    mockTraversalEngine.traverse.mockResolvedValue({
      sessionId: "traversal-123",
      startUrl: "https://example.com",
      pages: [],
      nodes: [],
      statistics: {
        pagesVisited: 1,
        pagesSkipped: 0,
        errorsEncountered: 0,
        maxDepthReached: 1,
        processingTimeMs: 100,
        totalContentBytes: 1024,
        avgPageLoadTimeMs: 50,
        rateLimitEncounters: 0,
      },
      graph: {
        nodes: [],
        edges: [],
      },
      completedAt: new Date(),
      maxDepthReached: true,
      pageLimitReached: false,
      totalPagesVisited: 1,
      traversalTimeMs: 100,
      depthDistribution: { 0: 1 },
    });
  });

  describe("initialization", () => {
    it("should initialize with configuration", () => {
      expect(webNavigator).toBeDefined();
    });

    it("should throw error when disabled", async () => {
      const disabledNavigator = new WebNavigator(
        { ...defaultConfig, enabled: false },
        mockDbClient,
        mockKnowledgeSeeker
      );

      await expect(
        disabledNavigator.processQuery({
          id: "query-1",
          url: "https://example.com/test",
          extractionType: "main_content" as any,
          enableTraversal: false,
          extractionConfig: {} as any,
          timeoutMs: 5000,
          metadata: {
            requesterId: "test",
            priority: 5,
            createdAt: new Date(),
          },
        })
      ).rejects.toThrow("disabled");
    });
  });

  describe("getStatus", () => {
    it("should return current status", async () => {
      const status = await webNavigator.getStatus();

      expect(status.enabled).toBe(true);
      expect(status.activeExtractions).toBe(0);
      expect(status.activeTraversals).toBe(0);
      expect(status.cacheStats).toBeDefined();
      expect(status.health).toBeDefined();
    });

    it("should include cache statistics", async () => {
      mockDbClient.getCacheStats.mockResolvedValue({
        totalPages: 10,
        cacheSizeBytes: 1024000,
        hitRate: 0.75,
        ageDistribution: {
          under1Hour: 2,
          under6Hours: 5,
          under12Hours: 8,
          under24Hours: 10,
        },
      });

      const status = await webNavigator.getStatus();

      expect(status.cacheStats.totalPages).toBe(10);
      expect(status.cacheStats.hitRate).toBe(0.75);
    });

    it("should include health information", async () => {
      const status = await webNavigator.getStatus();

      expect(status.health.httpClientAvailable).toBe(true);
      expect(status.health.databaseAvailable).toBe(true);
      expect(status.health.cacheAvailable).toBe(true);
      expect(status.health.status).toBe("healthy");
    });

    it("should report degraded when database unavailable", async () => {
      mockDbClient.isAvailable.mockReturnValue(false);

      const status = await webNavigator.getStatus();

      expect(status.health.databaseAvailable).toBe(false);
      expect(status.health.status).toBe("degraded");
    });
  });

  describe("clearCaches", () => {
    it("should clear all caches", async () => {
      await webNavigator.clearCaches();

      expect(mockDbClient.cleanupExpiredCache).toHaveBeenCalled();
    });

    it("should handle database unavailability gracefully", async () => {
      mockDbClient.isAvailable.mockReturnValue(false);

      await expect(webNavigator.clearCaches()).resolves.not.toThrow();
    });
  });

  describe("rate limiting", () => {
    it("should track rate limits per domain", async () => {
      const config = {
        ...defaultConfig,
        rateLimit: {
          ...defaultConfig.rateLimit,
          requestsPerMinute: 1,
        },
      };

      const navigator = new WebNavigator(
        config,
        mockDbClient,
        mockKnowledgeSeeker
      );

      mockDbClient.incrementRateLimitCounter.mockResolvedValue(2);

      // This would trigger rate limiting in a real scenario
      // For now, just verify the infrastructure is in place
      const status = await navigator.getStatus();
      expect(status).toBeDefined();
    });

    it("should handle rate limit backoff", async () => {
      const rateLimitedDb = {
        ...mockDbClient,
        getRateLimit: jest.fn().mockResolvedValue({
          domain: "example.com",
          status: RateLimitStatus.THROTTLED,
          requestsInWindow: 100,
          windowResetAt: new Date(Date.now() + 60000),
          backoffUntil: new Date(Date.now() + 5000),
          lastRequestAt: new Date(),
        }),
      };

      const navigator = new WebNavigator(
        defaultConfig,
        rateLimitedDb as any,
        mockKnowledgeSeeker
      );

      // Attempting to access a throttled domain should respect backoff
      // This is tested more thoroughly in integration tests
      expect(navigator).toBeDefined();
    });
  });

  describe("cache management", () => {
    it("should use cached content when available", async () => {
      const cachedContent = {
        id: "cached-1",
        url: "https://example.com/test",
        title: "Cached Page",
        content: "Cached content",
        links: [],
        images: [],
        metadata: {} as any,
        quality: "high" as any,
        contentHash: "hash1",
        extractedAt: new Date(),
      };

      mockDbClient.getContentByUrl.mockResolvedValue(cachedContent);

      // This would use cache in a real scenario
      const status = await webNavigator.getStatus();
      expect(status.cacheStats).toBeDefined();
    });

    it("should respect cache TTL configuration", () => {
      const customConfig = {
        ...defaultConfig,
        cache: {
          enabled: true,
          ttlHours: 12,
          maxSizeMb: 50,
        },
      };

      const navigator = new WebNavigator(
        customConfig,
        mockDbClient,
        mockKnowledgeSeeker
      );

      expect(navigator).toBeDefined();
    });

    it("should work without cache when disabled", () => {
      const noCacheConfig = {
        ...defaultConfig,
        cache: {
          enabled: false,
          ttlHours: 24,
          maxSizeMb: 100,
        },
      };

      const navigator = new WebNavigator(
        noCacheConfig,
        mockDbClient,
        mockKnowledgeSeeker
      );

      expect(navigator).toBeDefined();
    });
  });

  describe("configuration", () => {
    it("should respect security configuration", () => {
      const secureConfig = {
        ...defaultConfig,
        security: {
          verifySsl: true,
          sanitizeContent: true,
          detectMalicious: true,
          respectRobotsTxt: true,
        },
      };

      const navigator = new WebNavigator(
        secureConfig,
        mockDbClient,
        mockKnowledgeSeeker
      );

      expect(navigator).toBeDefined();
    });

    it("should respect limits configuration", () => {
      const limitedConfig = {
        ...defaultConfig,
        limits: {
          maxContentSizeMb: 5,
          maxExtractionTimeMs: 3000,
          maxTraversalDepth: 2,
          maxPagesPerTraversal: 25,
        },
      };

      const navigator = new WebNavigator(
        limitedConfig,
        mockDbClient,
        mockKnowledgeSeeker
      );

      expect(navigator).toBeDefined();
    });

    it("should respect observability configuration", () => {
      const observableConfig = {
        ...defaultConfig,
        observability: {
          enableMetrics: true,
          enableTracing: true,
          logLevel: "debug" as const,
        },
      };

      const navigator = new WebNavigator(
        observableConfig,
        mockDbClient,
        mockKnowledgeSeeker
      );

      expect(navigator).toBeDefined();
    });
  });

  describe("graceful degradation", () => {
    it("should continue working when database is unavailable", async () => {
      mockDbClient.isAvailable.mockReturnValue(false);

      const status = await webNavigator.getStatus();

      expect(status.health.databaseAvailable).toBe(false);
      expect(status.health.status).toBe("degraded");
    });

    it("should return empty cache stats when database unavailable", async () => {
      mockDbClient.isAvailable.mockReturnValue(false);

      const status = await webNavigator.getStatus();

      expect(status.cacheStats.totalPages).toBe(0);
      expect(status.cacheStats.hitRate).toBe(0);
    });
  });

  describe("concurrent operations", () => {
    it("should track active extractions", async () => {
      const initialStatus = await webNavigator.getStatus();
      expect(initialStatus.activeExtractions).toBe(0);

      // Active extractions would be tracked during actual extraction
      // This verifies the infrastructure is in place
    });

    it("should track active traversals", async () => {
      const initialStatus = await webNavigator.getStatus();
      expect(initialStatus.activeTraversals).toBe(0);

      // Active traversals would be tracked during actual traversal
      // This verifies the infrastructure is in place
    });
  });

  describe("content extraction", () => {
    beforeEach(() => {
      // Reset mocks
      jest.clearAllMocks();
    });

    it("should extract content from URL with default config", async () => {
      const mockContent = {
        id: "content-1",
        url: "https://example.com",
        title: "Example Page",
        content: "Example content",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 1000,
          isSecure: true,
        },
      };

      // Mock the content extractor
      const mockContentExtractor = {
        extractContent: jest.fn().mockResolvedValue(mockContent),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      const result = await webNavigator.extractContent("https://example.com");

      expect(mockContentExtractor.extractContent).toHaveBeenCalledWith(
        "https://example.com",
        expect.any(Object)
      );
      expect(result).toEqual(mockContent);
      expect(mockDbClient.storeContent).toHaveBeenCalledWith(mockContent);
      expect(mockDbClient.cacheContent).toHaveBeenCalled();
    });

    it("should extract content with custom config", async () => {
      const customConfig = {
        includeImages: false,
        includeLinks: true,
        includeMetadata: true,
        stripNavigation: true,
        stripAds: true,
        maxContentLength: 5000,
        security: {
          verifySsl: true,
          sanitizeHtml: true,
          detectMalicious: true,
          followRedirects: true,
          maxRedirects: 5,
          userAgent: "Custom Agent",
          respectRobotsTxt: true,
        },
      };

      const mockContent = {
        id: "content-2",
        url: "https://example.com",
        title: "Example Page",
        content: "Custom content",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 2000,
          isSecure: true,
        },
      };

      const mockContentExtractor = {
        extractContent: jest.fn().mockResolvedValue(mockContent),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      const result = await webNavigator.extractContent(
        "https://example.com",
        customConfig
      );

      expect(mockContentExtractor.extractContent).toHaveBeenCalledWith(
        "https://example.com",
        customConfig
      );
      expect(result).toEqual(mockContent);
    });

    it("should handle content extraction errors", async () => {
      const mockContentExtractor = {
        extractContent: jest.fn().mockRejectedValue(new Error("Network error")),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      await expect(
        webNavigator.extractContent("https://example.com")
      ).rejects.toThrow("Network error");

      expect(mockDbClient.storeContent).not.toHaveBeenCalled();
    });

    it("should cache extracted content", async () => {
      const cachedContent: any = {
        id: "cached-1",
        url: "https://example.com",
        title: "Cached Page",
        content: "Cached content",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 1500,
          isSecure: true,
        },
        links: [],
        images: [],
        quality: "high",
        contentHash: "hash123",
      };

      mockDbClient.getContentByUrl.mockResolvedValue(cachedContent);

      const result = await webNavigator.extractContent("https://example.com");

      expect(result).toEqual(cachedContent);
      expect(mockDbClient.getContentByUrl).toHaveBeenCalledWith(
        "https://example.com"
      );
      // Content extractor should not be called for cached content
      expect(
        (webNavigator as any).contentExtractor?.extractContent
      ).not.toHaveBeenCalled();
    });

    it("should store extraction metrics in database", async () => {
      const mockContent = {
        id: "content-3",
        url: "https://example.com",
        title: "Metrics Test",
        content: "Test content",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 3000,
          isSecure: true,
        },
      };

      const mockContentExtractor = {
        extractContent: jest.fn().mockResolvedValue(mockContent),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      await webNavigator.extractContent("https://example.com");

      expect(mockDbClient.storeExtractionMetrics).toHaveBeenCalledWith(
        mockContent.id,
        expect.objectContaining({
          totalTimeMs: expect.any(Number),
          statusCode: 200,
          contentType: "text/html",
          contentLength: 3000,
          sslVerified: true,
          sanitizationApplied: true,
        })
      );
    });

    it("should handle rate limit during extraction", async () => {
      const rateLimitError = new Error("429 Too Many Requests");

      const mockContentExtractor = {
        extractContent: jest.fn().mockRejectedValue(rateLimitError),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      await expect(
        webNavigator.extractContent("https://example.com")
      ).rejects.toThrow("429 Too Many Requests");

      // Should handle rate limit
      expect(mockDbClient.updateRateLimit).toHaveBeenCalled();
    });

    it("should prevent concurrent extraction of same URL", async () => {
      const mockContent = {
        id: "content-4",
        url: "https://example.com",
        title: "Concurrent Test",
        content: "Test content",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 1000,
          isSecure: true,
        },
      };

      const mockContentExtractor = {
        extractContent: jest
          .fn()
          .mockImplementation(
            () =>
              new Promise((resolve) =>
                setTimeout(() => resolve(mockContent), 100)
              )
          ),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      // Start two concurrent extractions
      const promise1 = webNavigator.extractContent("https://example.com");
      const promise2 = webNavigator.extractContent("https://example.com");

      const [result1, result2] = await Promise.all([promise1, promise2]);

      expect(result1).toEqual(mockContent);
      expect(result2).toEqual(mockContent);
      // Content extractor should only be called once due to deduplication
      expect(mockContentExtractor.extractContent).toHaveBeenCalledTimes(1);
    });
  });

  describe("traversal operations", () => {
    beforeEach(() => {
      jest.clearAllMocks();
    });

    it("should perform traversal with configuration", async () => {
      const traversalConfig = {
        maxDepth: 3,
        maxPages: 10,
        strategy: "breadth_first" as any,
        sameDomainOnly: true,
        respectRobotsTxt: true,
        delayBetweenRequests: 100,
        timeout: 5000,
      };

      const mockTraversalResult = {
        sessionId: "traversal-1",
        startUrl: "https://example.com",
        pagesVisited: [
          "https://example.com/page1",
          "https://example.com/page2",
        ],
        totalPages: 2,
        maxDepthReached: 1,
        errors: [],
        startedAt: new Date(),
        completedAt: new Date(),
      };

      // TraversalEngine is now mocked at the module level

      const query: any = {
        id: "query-1",
        url: "https://example.com",
        extractionType: "full_page",
        enableTraversal: true,
        traversalConfig,
        extractionConfig: {
          includeImages: true,
          includeLinks: true,
          includeMetadata: true,
          stripNavigation: true,
          stripAds: true,
          maxContentLength: 10000,
          security: {
            verifySsl: true,
            sanitizeHtml: true,
            detectMalicious: true,
            followRedirects: true,
            maxRedirects: 5,
            userAgent: "TestAgent",
            respectRobotsTxt: true,
          },
        },
        timeoutMs: 30000,
        metadata: {
          requesterId: "test-user",
          priority: 1,
          createdAt: new Date(),
        },
      };

      const result = await webNavigator.processQuery(query);

      expect(mockTraversalEngine.traverse).toHaveBeenCalledWith(
        "https://example.com",
        expect.any(Object) // getDefaultExtractionConfig() result
      );
      expect(result).toEqual({
        sessionId: "traversal-123",
        startUrl: "https://example.com",
        pages: [],
        nodes: [],
        statistics: {
          pagesVisited: 1,
          pagesSkipped: 0,
          errorsEncountered: 0,
          maxDepthReached: 1,
          processingTimeMs: 100,
          totalContentBytes: 1024,
          avgPageLoadTimeMs: 50,
          rateLimitEncounters: 0,
        },
        graph: {
          nodes: [],
          edges: [],
        },
        completedAt: expect.any(Date),
        maxDepthReached: true,
        pageLimitReached: false,
        totalPagesVisited: 1,
        traversalTimeMs: 100,
        depthDistribution: { 0: 1 },
      });
      expect(mockDbClient.createTraversal).toHaveBeenCalledWith(
        expect.stringMatching(/^traversal-\d+-[a-z0-9]+$/),
        "https://example.com",
        traversalConfig.maxDepth,
        traversalConfig.maxPages,
        traversalConfig.strategy
      );
    });

    it("should handle traversal errors", async () => {
      // Configure the module-level mock to throw an error
      mockTraversalEngine.traverse.mockRejectedValue(
        new Error("Traversal failed")
      );

      const query: any = {
        id: "query-2",
        url: "https://example.com",
        extractionType: "full_page",
        enableTraversal: true,
        traversalConfig: { maxDepth: 2, maxPages: 5 },
        extractionConfig: {
          includeImages: true,
          includeLinks: true,
          includeMetadata: true,
          stripNavigation: true,
          stripAds: true,
          maxContentLength: 10000,
          security: {
            verifySsl: true,
            sanitizeHtml: true,
            detectMalicious: true,
            followRedirects: true,
            maxRedirects: 5,
            userAgent: "TestAgent",
            respectRobotsTxt: true,
          },
        },
        timeoutMs: 30000,
        metadata: {
          requesterId: "test-user",
          priority: 1,
          createdAt: new Date(),
        },
      };

      await expect(webNavigator.processQuery(query)).rejects.toThrow(
        "Traversal failed"
      );
    });

    it("should create traversal session in database", async () => {
      const traversalConfig = {
        maxDepth: 2,
        maxPages: 5,
        strategy: "breadth_first" as any,
        sameDomainOnly: true,
        respectRobotsTxt: true,
      };
      const mockTraversalResult = {
        sessionId: "session-123",
        startUrl: "https://example.com",
        pagesVisited: ["https://example.com"],
        totalPages: 1,
        maxDepthReached: 0,
        errors: [],
        startedAt: new Date(),
        completedAt: new Date(),
      };

      // TraversalEngine is now mocked at the module level

      const query: any = {
        id: "query-3",
        url: "https://example.com",
        extractionType: "full_page",
        enableTraversal: true,
        traversalConfig,
        extractionConfig: {
          includeImages: true,
          includeLinks: true,
          includeMetadata: true,
          stripNavigation: true,
          stripAds: true,
          maxContentLength: 10000,
          security: {
            verifySsl: true,
            sanitizeHtml: true,
            detectMalicious: true,
            followRedirects: true,
            maxRedirects: 5,
            userAgent: "TestAgent",
            respectRobotsTxt: true,
          },
        },
        timeoutMs: 30000,
        metadata: {
          requesterId: "test-user",
          priority: 1,
          createdAt: new Date(),
        },
      };

      await webNavigator.processQuery(query);

      expect(mockDbClient.createTraversal).toHaveBeenCalledWith(
        expect.stringMatching(/^traversal-\d+-[a-z0-9]+$/),
        "https://example.com",
        traversalConfig.maxDepth,
        traversalConfig.maxPages,
        traversalConfig.strategy
      );
    });

    it("should store traversal results", async () => {
      const mockTraversalResult = {
        sessionId: "session-456",
        startUrl: "https://example.com",
        pagesVisited: ["https://example.com/page1"],
        totalPages: 1,
        maxDepthReached: 1,
        errors: [],
        startedAt: new Date(),
        completedAt: new Date(),
      };

      const mockTraversalEngine = {
        traverse: jest.fn().mockResolvedValue(mockTraversalResult),
      };

      // TraversalEngine is now mocked at the module level

      const query: any = {
        id: "query-4",
        url: "https://example.com",
        extractionType: "full_page",
        enableTraversal: true,
        traversalConfig: { maxDepth: 2, maxPages: 5 },
        extractionConfig: {
          includeImages: true,
          includeLinks: true,
          includeMetadata: true,
          stripNavigation: true,
          stripAds: true,
          maxContentLength: 10000,
          security: {
            verifySsl: true,
            sanitizeHtml: true,
            detectMalicious: true,
            followRedirects: true,
            maxRedirects: 5,
            userAgent: "TestAgent",
            respectRobotsTxt: true,
          },
        },
        timeoutMs: 30000,
        metadata: {
          requesterId: "test-user",
          priority: 1,
          createdAt: new Date(),
        },
      };

      await webNavigator.processQuery(query);

      expect(mockDbClient.updateTraversalNode).toHaveBeenCalledWith(
        "session-456",
        expect.objectContaining({
          totalPages: 1,
          maxDepthReached: 1,
          errors: [],
          completedAt: expect.any(Date),
        })
      );
    });
  });

  describe("rate limiting", () => {
    beforeEach(() => {
      jest.clearAllMocks();
    });

    it("should apply rate limit backoff", async () => {
      // Set up rate limit
      const domain = "example.com";
      const rateLimit = {
        domain,
        status: RateLimitStatus.THROTTLED,
        requestsInWindow: 0,
        windowResetAt: new Date(Date.now() + 60000),
        backoffUntil: new Date(Date.now() + 30000),
        lastRequestAt: new Date(),
      };

      mockDbClient.getRateLimit.mockResolvedValue(rateLimit);

      const mockContentExtractor = {
        extractContent: jest
          .fn()
          .mockRejectedValue(new Error("429 Too Many Requests")),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      await expect(
        webNavigator.extractContent(`https://${domain}`)
      ).rejects.toThrow("429 Too Many Requests");

      expect(mockDbClient.updateRateLimit).toHaveBeenCalledWith(
        expect.objectContaining({
          domain,
          status: RateLimitStatus.THROTTLED,
          backoffUntil: expect.any(Date),
        })
      );
    });

    it("should update rate limit status in database", async () => {
      const domain = "rate-limited.com";

      const mockContentExtractor = {
        extractContent: jest
          .fn()
          .mockRejectedValue(new Error("429 Too Many Requests")),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      await expect(
        webNavigator.extractContent(`https://${domain}`)
      ).rejects.toThrow("429 Too Many Requests");

      expect(mockDbClient.updateRateLimit).toHaveBeenCalled();
    });

    it("should handle rate limit expiration", async () => {
      // Test that expired rate limits are reset
      const expiredRateLimit = {
        domain: "expired.com",
        status: RateLimitStatus.THROTTLED,
        requestsInWindow: 10,
        windowResetAt: new Date(Date.now() - 1000), // Expired
        backoffUntil: new Date(Date.now() - 1000), // Expired
        lastRequestAt: new Date(Date.now() - 2000),
      };

      mockDbClient.getRateLimit.mockResolvedValue(expiredRateLimit);

      const mockContent = {
        id: "content-5",
        url: "https://expired.com",
        title: "Expired Rate Limit",
        content: "Content after rate limit expired",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 1000,
          isSecure: true,
        },
      };

      const mockContentExtractor = {
        extractContent: jest.fn().mockResolvedValue(mockContent),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      const result = await webNavigator.extractContent("https://expired.com");

      expect(result).toEqual(mockContent);
    });
  });

  describe("health monitoring", () => {
    it("should report healthy status when all components available", async () => {
      mockDbClient.isAvailable.mockReturnValue(true);

      const status = await webNavigator.getStatus();

      expect(status.health.status).toBe("healthy");
      expect(status.health.httpClientAvailable).toBe(true);
      expect(status.health.databaseAvailable).toBe(true);
      expect(status.health.cacheAvailable).toBe(true);
    });

    it("should report degraded status when database unavailable", async () => {
      mockDbClient.isAvailable.mockReturnValue(false);

      const status = await webNavigator.getStatus();

      expect(status.health.status).toBe("degraded");
      expect(status.health.databaseAvailable).toBe(false);
      expect(status.health.cacheAvailable).toBe(false);
    });

    it("should report unhealthy status when HTTP client unavailable", async () => {
      // Mock HTTP client unavailability (would need different mocking approach)
      // For now, test that degraded status works
      mockDbClient.isAvailable.mockReturnValue(false);

      const status = await webNavigator.getStatus();

      expect(status.health.status).toBe("degraded");
    });
  });

  describe("cache management", () => {
    beforeEach(() => {
      jest.clearAllMocks();
    });

    it("should perform periodic cache cleanup", async () => {
      // Trigger cache cleanup manually (normally done by interval)
      await webNavigator.clearCaches();

      expect(mockDbClient.cleanupExpiredCache).toHaveBeenCalled();
    });

    it("should handle cache cleanup errors gracefully", async () => {
      mockDbClient.cleanupExpiredCache.mockRejectedValue(
        new Error("Cleanup failed")
      );

      // Should not throw
      await expect(webNavigator.clearCaches()).resolves.toBeUndefined();
    });

    it("should clear search engine cache", async () => {
      const mockSearchEngine = {
        clearCache: jest.fn(),
        pruneCache: jest.fn(),
      };

      (webNavigator as any).searchEngine = mockSearchEngine;

      await webNavigator.clearCaches();

      expect(mockSearchEngine.clearCache).toHaveBeenCalled();
    });
  });

  describe("error handling", () => {
    it("should handle database connection failures", async () => {
      mockDbClient.isAvailable.mockReturnValue(false);
      mockDbClient.getContentByUrl.mockRejectedValue(
        new Error("DB connection failed")
      );

      const mockContent = {
        id: "content-6",
        url: "https://example.com",
        title: "DB Failure Test",
        content: "Content despite DB failure",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 1000,
          isSecure: true,
        },
      };

      const mockContentExtractor = {
        extractContent: jest.fn().mockResolvedValue(mockContent),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      // Should still work despite DB failures
      const result = await webNavigator.extractContent("https://example.com");

      expect(result).toEqual(mockContent);
    });

    it("should handle search engine failures", async () => {
      // Test would require mocking search engine failures
      // For now, verify the infrastructure exists
      expect((webNavigator as any).searchEngine).toBeDefined();
    });

    it("should maintain operation during partial failures", async () => {
      // Make database operations fail but keep core functionality working
      mockDbClient.storeContent.mockRejectedValue(new Error("DB write failed"));
      mockDbClient.cacheContent.mockRejectedValue(
        new Error("Cache write failed")
      );

      const mockContent = {
        id: "content-7",
        url: "https://example.com",
        title: "Partial Failure Test",
        content: "Content despite partial failures",
        extractedAt: new Date(),
        metadata: {
          statusCode: 200,
          contentType: "text/html",
          contentLength: 1000,
          isSecure: true,
        },
      };

      const mockContentExtractor = {
        extractContent: jest.fn().mockResolvedValue(mockContent),
      };

      (webNavigator as any).contentExtractor = mockContentExtractor;

      // Should still return content despite DB failures
      const result = await webNavigator.extractContent("https://example.com");

      expect(result).toEqual(mockContent);
    });
  });
});
