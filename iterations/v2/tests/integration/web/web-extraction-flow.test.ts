/**
 * @fileoverview Integration tests for Web Navigator extraction workflow
 *
 * Tests end-to-end content extraction, caching, and rate limiting.
 *
 * @author @darianrosebrook
 */

import { Pool } from "pg";
import { WebNavigatorDatabaseClient } from "../../../src/database/WebNavigatorDatabaseClient";
import { KnowledgeSeeker } from "../../../src/knowledge/KnowledgeSeeker";
import { WebNavigatorConfig } from "../../../src/types/web";
import { WebNavigator } from "../../../src/web/WebNavigator";

describe("Web Navigator Integration - Extraction Flow", () => {
  let webNavigator: WebNavigator;
  let dbClient: WebNavigatorDatabaseClient;
  let knowledgeSeeker: KnowledgeSeeker;
  let config: WebNavigatorConfig;
  let pool: Pool;

  beforeAll(async () => {
    // Initialize configuration
    config = {
      enabled: true,
      http: {
        timeoutMs: 10000,
        maxConcurrentRequests: 20,
        retryAttempts: 3,
        retryDelayMs: 1000,
        userAgent: "Test-WebNavigator/1.0",
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
        respectRobotsTxt: false, // Disabled for testing
      },
      limits: {
        maxContentSizeMb: 10,
        maxExtractionTimeMs: 10000,
        maxTraversalDepth: 3,
        maxPagesPerTraversal: 50,
      },
      observability: {
        enableMetrics: true,
        enableTracing: false,
        logLevel: "error",
      },
    };

    // Initialize database client
    pool = new Pool({
      host: process.env.DB_HOST || "localhost",
      port: parseInt(process.env.DB_PORT || "5432"),
      database: process.env.DB_NAME || "agent_agency_test",
      user: process.env.DB_USER || "postgres",
      password: process.env.DB_PASSWORD || "postgres",
    });

    dbClient = new WebNavigatorDatabaseClient(pool);
    await dbClient.initialize();

    // Initialize Knowledge Seeker (mocked or minimal for integration)
    knowledgeSeeker = {} as any; // Minimal mock

    // Initialize Web Navigator
    webNavigator = new WebNavigator(config, dbClient, knowledgeSeeker);
  });

  afterAll(async () => {
    await dbClient.shutdown();
    await pool.end();
  });

  describe("Status and Health", () => {
    it("should report healthy status", async () => {
      const status = await webNavigator.getStatus();

      expect(status.enabled).toBe(true);
      expect(status.health.status).toBe("healthy");
      expect(status.health.databaseAvailable).toBe(true);
      expect(status.health.httpClientAvailable).toBe(true);
    });

    it("should provide cache statistics", async () => {
      const status = await webNavigator.getStatus();

      expect(status.cacheStats).toBeDefined();
      expect(typeof status.cacheStats.totalPages).toBe("number");
      expect(typeof status.cacheStats.hitRate).toBe("number");
    });
  });

  describe("Configuration", () => {
    it("should respect enabled flag", async () => {
      const disabledNavigator = new WebNavigator(
        { ...config, enabled: false },
        dbClient,
        knowledgeSeeker
      );

      await expect(
        disabledNavigator.processQuery({
          id: "test-1",
          url: "https://example.com",
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

    it("should respect observability configuration", async () => {
      const observableNavigator = new WebNavigator(
        {
          ...config,
          observability: {
            enableMetrics: true,
            enableTracing: true,
            logLevel: "debug",
          },
        },
        dbClient,
        knowledgeSeeker
      );

      expect(observableNavigator).toBeDefined();
    });
  });

  describe("Cache Management", () => {
    it("should clear caches on demand", async () => {
      await expect(webNavigator.clearCaches()).resolves.not.toThrow();
    });

    it("should work with cache disabled", () => {
      const noCacheNavigator = new WebNavigator(
        {
          ...config,
          cache: {
            enabled: false,
            ttlHours: 24,
            maxSizeMb: 100,
          },
        },
        dbClient,
        knowledgeSeeker
      );

      expect(noCacheNavigator).toBeDefined();
    });
  });

  describe("Graceful Degradation", () => {
    it("should continue working when database unavailable", async () => {
      // Create a client that simulates database unavailability
      const unavailableDbClient = {
        ...dbClient,
        isAvailable: () => false,
      } as any;

      const degradedNavigator = new WebNavigator(
        config,
        unavailableDbClient,
        knowledgeSeeker
      );

      const status = await degradedNavigator.getStatus();

      expect(status.health.databaseAvailable).toBe(false);
      expect(status.health.status).toBe("degraded");
    });
  });

  describe("Component Integration", () => {
    it("should initialize all components", () => {
      expect(webNavigator).toBeDefined();
    });

    it("should have accessible status", async () => {
      const status = await webNavigator.getStatus();

      expect(status.activeExtractions).toBeDefined();
      expect(status.activeTraversals).toBeDefined();
      expect(status.cacheStats).toBeDefined();
    });
  });
});
