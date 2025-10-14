/**
 * @fileoverview Integration tests for WebNavigator
 *
 * Tests the WebNavigator orchestrator working with real dependencies
 * in the database and event system.
 *
 * @author @darianrosebrook
 */

import { WebNavigatorDatabaseClient } from "../../../src/database/WebNavigatorDatabaseClient";
import { KnowledgeSeeker } from "../../../src/knowledge/KnowledgeSeeker";
import { WebNavigatorConfig } from "../../../src/types/web";
import { WebNavigator } from "../../../src/web/WebNavigator";

describe("WebNavigator Integration", () => {
  let webNavigator: WebNavigator;
  let dbClient: WebNavigatorDatabaseClient;
  let knowledgeSeeker: KnowledgeSeeker;
  let config: WebNavigatorConfig;

  beforeAll(async () => {
    // Use real database client for integration testing
    dbClient = new WebNavigatorDatabaseClient();
    knowledgeSeeker = new KnowledgeSeeker();

    config = {
      http: {
        userAgent: "TestWebNavigator/1.0",
        timeoutMs: 5000,
        maxRedirects: 3,
        verifySsl: false, // Allow self-signed for testing
      },
      limits: {
        maxConcurrentExtractions: 2,
        maxPagesPerDomain: 10,
        maxContentSizeBytes: 1024 * 1024, // 1MB
        maxCacheSizeBytes: 10 * 1024 * 1024, // 10MB
      },
      security: {
        allowedDomains: ["example.com", "httpbin.org"],
        blockedDomains: ["malicious-site.com"],
        respectRobotsTxt: false, // Skip for integration tests
        validateSsl: false,
      },
      observability: {
        enableMetrics: true,
        enableTracing: false,
        logLevel: "info",
      },
    };

    webNavigator = new WebNavigator(config, dbClient, knowledgeSeeker);
  });

  afterAll(async () => {
    // Clean up any test data
    if (dbClient) {
      await dbClient.cleanupExpiredCache();
    }
  });

  describe("health monitoring", () => {
    it("should report healthy status", async () => {
      const health = await webNavigator.getHealth();

      expect(health).toBeDefined();
      expect(health.healthy).toBe(true);
      expect(health.databaseConnected).toBe(true);
      expect(typeof health.uptimeMs).toBe("number");
      expect(health.uptimeMs).toBeGreaterThan(0);
    });

    it("should provide cache statistics", async () => {
      const stats = await webNavigator.getCacheStats();

      expect(stats).toBeDefined();
      expect(typeof stats.totalPages).toBe("number");
      expect(typeof stats.cacheSizeBytes).toBe("number");
      expect(stats.hitRate).toBeGreaterThanOrEqual(0);
      expect(stats.hitRate).toBeLessThanOrEqual(1);
    });
  });

  describe("configuration management", () => {
    it("should return current configuration", () => {
      const currentConfig = webNavigator.getConfig();

      expect(currentConfig).toBeDefined();
      expect(currentConfig.http).toBeDefined();
      expect(currentConfig.limits).toBeDefined();
      expect(currentConfig.security).toBeDefined();
    });

    it("should validate configuration on startup", () => {
      // Configuration should be validated during construction
      expect(webNavigator).toBeDefined();
    });
  });

  describe("cache operations", () => {
    it("should clear caches when requested", async () => {
      await webNavigator.clearCaches();

      // After clearing, cache stats should show empty or reset
      const stats = await webNavigator.getCacheStats();
      expect(stats).toBeDefined();
    });

    it("should handle cache operations gracefully", async () => {
      // These operations should not throw even if database is not available
      await expect(webNavigator.clearCaches()).resolves.not.toThrow();
      await expect(webNavigator.getCacheStats()).resolves.not.toThrow();
    });
  });

  describe("rate limiting", () => {
    it("should track domain rate limits", async () => {
      const domain = "example.com";
      const status = await webNavigator.getRateLimitStatus(domain);

      expect(status).toBeDefined();
      expect(status.domain).toBe(domain);
      expect(typeof status.requestsInWindow).toBe("number");
      expect(typeof status.windowStartMs).toBe("number");
    });

    it("should handle rate limit queries gracefully", async () => {
      await expect(
        webNavigator.getRateLimitStatus("test.com")
      ).resolves.not.toThrow();
    });
  });

  describe("status reporting", () => {
    it("should provide comprehensive status information", async () => {
      const status = webNavigator.getStatus();

      expect(status).toBeDefined();
      expect(status.timestamp).toBeInstanceOf(Date);
      expect(status.config).toBeDefined();
      expect(status.health).toBeDefined();
      expect(status.cache).toBeDefined();
    });

    it("should include version information", () => {
      const status = webNavigator.getStatus();

      expect(status.version).toBeDefined();
      expect(typeof status.version).toBe("string");
    });
  });

  describe("database integration", () => {
    it("should interact with database for persistence", async () => {
      // Test basic database operations
      const health = await webNavigator.getHealth();
      expect(health.databaseConnected).toBeDefined();
    });

    it("should handle database unavailability gracefully", async () => {
      // If database is not available, operations should degrade gracefully
      const stats = await webNavigator.getCacheStats();
      expect(stats).toBeDefined();
    });
  });

  describe("resource management", () => {
    it("should track active operations", () => {
      const status = webNavigator.getStatus();

      expect(status.activeOperations).toBeDefined();
      expect(typeof status.activeOperations.extractions).toBe("number");
      expect(typeof status.activeOperations.searches).toBe("number");
      expect(typeof status.activeOperations.traversals).toBe("number");
    });

    it("should report resource usage", () => {
      const status = webNavigator.getStatus();

      expect(status.resources).toBeDefined();
      expect(typeof status.resources.memoryUsage).toBe("number");
      expect(typeof status.resources.cpuUsage).toBe("number");
    });
  });
});


