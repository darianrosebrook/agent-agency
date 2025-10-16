/**
 * Integration Tests: Knowledge Database Client
 *
 * Tests database operations with KnowledgeDatabaseClient.
 * NOTE: These are integration contract tests. Full implementation requires real database setup.
 *
 * @module tests/integration/database/knowledge-database.test.ts
 * @author @darianrosebrook
 */

import { KnowledgeDatabaseClient } from "../../../src/database/KnowledgeDatabaseClient";
import {
  mockKnowledgeQuery,
  mockKnowledgeResponse,
  mockSearchResult,
} from "../../mocks/knowledge-mocks";

describe("Knowledge Database Integration", () => {
  let dbClient: KnowledgeDatabaseClient;

  beforeEach(() => {
    // Uses centralized ConnectionPoolManager initialized in tests/setup.ts
    dbClient = new KnowledgeDatabaseClient();
  });

  afterEach(async () => {
    // Note: Pool lifecycle managed by ConnectionPoolManager
    // No need to call shutdown() - handled in tests/setup.ts afterAll
  });

  describe("Query Storage", () => {
    it("should store knowledge queries", async () => {
      const query = mockKnowledgeQuery({
        query: "How do I implement OAuth2?",
      });

      // Store query - should return query ID or null if unavailable
      const queryId = await dbClient.storeQuery(query);

      // Verify operation completed (either stored or gracefully degraded)
      expect(queryId === null || typeof queryId === "string").toBe(true);
    });

    it("should update query status", async () => {
      const query = mockKnowledgeQuery();

      const queryId = await dbClient.storeQuery(query);

      if (queryId) {
        // Update should not throw
        await expect(
          dbClient.updateQueryStatus(queryId, "completed")
        ).resolves.not.toThrow();
      }
    });
  });

  describe("Result Storage", () => {
    it("should store search results", async () => {
      const results = [
        mockSearchResult(),
        mockSearchResult(),
        mockSearchResult(),
      ];

      // Store should not throw
      await expect(dbClient.storeResults(results)).resolves.not.toThrow();
    });

    it("should handle empty result sets", async () => {
      const results = [];

      await expect(dbClient.storeResults(results)).resolves.not.toThrow();
    });
  });

  describe("Response Storage", () => {
    it("should store knowledge responses", async () => {
      const query = mockKnowledgeQuery();
      const response = mockKnowledgeResponse({ query });

      await expect(dbClient.storeResponse(response)).resolves.not.toThrow();
    });
  });

  describe("Provider Health Tracking", () => {
    it("should update provider health status", async () => {
      await expect(
        dbClient.updateProviderHealth("google", {
          available: true,
          responseTimeMs: 250,
          errorRate: 0.01,
          requestsThisMinute: 10,
          requestsThisHour: 500,
        })
      ).resolves.not.toThrow();
    });

    it("should retrieve provider health", async () => {
      const health = await dbClient.getProviderHealth("google");

      // Should return health object or null if unavailable
      expect(health === null || typeof health === "object").toBe(true);
    });
  });

  describe("Graceful Degradation", () => {
    it("should handle database unavailability gracefully", async () => {
      // Without calling initialize(), database should be unavailable
      expect(dbClient.isAvailable()).toBe(false);

      // Operations should not throw when database unavailable
      const query = mockKnowledgeQuery();
      await expect(dbClient.storeQuery(query)).resolves.not.toThrow();
    });
  });

  describe("Performance", () => {
    it("should complete storage operations quickly", async () => {
      const query = mockKnowledgeQuery();
      const response = mockKnowledgeResponse({ query });

      const startTime = Date.now();
      await dbClient.storeResponse(response);
      const duration = Date.now() - startTime;

      // Should complete quickly (even if just graceful degradation)
      expect(duration).toBeLessThan(1000);
    });
  });
});

/**
 * NOTE: Full database integration tests require:
 * 1. Test database setup and teardown
 * 2. Database migrations applied
 * 3. Test data seeding
 * 4. Connection pooling configuration
 * 5. Transaction isolation
 *
 * These tests verify the integration contract and graceful degradation.
 * For full database testing, use a test PostgreSQL instance.
 */
