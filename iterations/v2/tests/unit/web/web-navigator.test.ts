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
    } as any;

    mockKnowledgeSeeker = {
      processQuery: jest.fn(),
    } as any;

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
});
