/**
 * Distributed Cache Client Tests
 *
 * Tests for Redis-based distributed caching functionality
 * used by the FederatedLearningEngine.
 *
 * @author @darianrosebrook
 */

import {
  DistributedCacheClient,
  DistributedCacheConfig,
} from "@/adapters/DistributedCacheClient";
import { Logger } from "@/observability/Logger";

describe("DistributedCacheClient", () => {
  let cacheClient: DistributedCacheClient;
  let logger: Logger;
  let config: DistributedCacheConfig;

  beforeEach(() => {
    logger = new Logger("test");
    config = {
      enabled: true,
      provider: {
        type: "mock", // Use mock for testing
      },
      retry: {
        maxAttempts: 3,
        delayMs: 100,
        backoffMultiplier: 2,
      },
      serialization: {
        format: "json",
        compression: false,
      },
    };
    cacheClient = new DistributedCacheClient(config, logger);
  });

  afterEach(async () => {
    await cacheClient.shutdown();
  });

  describe("initialization", () => {
    it("should initialize successfully with mock provider", async () => {
      await expect(cacheClient.initialize()).resolves.not.toThrow();
    });

    it("should handle disabled cache gracefully", async () => {
      const disabledConfig = { ...config, enabled: false };
      const disabledClient = new DistributedCacheClient(disabledConfig, logger);

      await expect(disabledClient.initialize()).resolves.not.toThrow();
      await disabledClient.shutdown();
    });
  });

  describe("basic operations", () => {
    beforeEach(async () => {
      await cacheClient.initialize();
    });

    it("should store and retrieve values", async () => {
      const key = "test-key";
      const value = { message: "Hello, World!", number: 42 };

      await cacheClient.set(key, value);
      const retrieved = await cacheClient.get(key);

      expect(retrieved).toEqual(value);
    });

    it("should return null for non-existent keys", async () => {
      const retrieved = await cacheClient.get("non-existent-key");
      expect(retrieved).toBeNull();
    });

    it("should delete values", async () => {
      const key = "test-key";
      const value = { data: "test" };

      await cacheClient.set(key, value);
      expect(await cacheClient.exists(key)).toBe(true);

      const deleted = await cacheClient.delete(key);
      expect(deleted).toBe(true);
      expect(await cacheClient.exists(key)).toBe(false);
    });

    it("should check key existence", async () => {
      const key = "test-key";
      const value = { data: "test" };

      expect(await cacheClient.exists(key)).toBe(false);

      await cacheClient.set(key, value);
      expect(await cacheClient.exists(key)).toBe(true);
    });

    it("should get keys by pattern", async () => {
      await cacheClient.set("user:1", { name: "Alice" });
      await cacheClient.set("user:2", { name: "Bob" });
      await cacheClient.set("session:1", { token: "abc123" });

      const userKeys = await cacheClient.keys("user:*");
      expect(userKeys).toHaveLength(2);
      expect(userKeys).toContain("user:1");
      expect(userKeys).toContain("user:2");
    });
  });

  describe("tenant contribution tracking", () => {
    beforeEach(async () => {
      await cacheClient.initialize();
    });

    it("should track tenant contributions", async () => {
      const tenantId = "tenant-1";
      const topicKey = "topic-1";
      const insightsCount = 5;

      await cacheClient.trackTenantContribution(
        tenantId,
        topicKey,
        insightsCount
      );

      const contributions = await cacheClient.getTenantContributions(tenantId);
      expect(contributions).toHaveLength(1);
      expect(contributions[0]).toMatchObject({
        tenantId,
        topicKey,
        insightsCount,
        contributionCount: 1,
      });
    });

    it("should increment contribution count for existing contributions", async () => {
      const tenantId = "tenant-1";
      const topicKey = "topic-1";

      await cacheClient.trackTenantContribution(tenantId, topicKey, 3);
      await cacheClient.trackTenantContribution(tenantId, topicKey, 2);

      const contributions = await cacheClient.getTenantContributions(tenantId);
      expect(contributions).toHaveLength(1);
      expect(contributions[0]).toMatchObject({
        tenantId,
        topicKey,
        insightsCount: 5, // 3 + 2
        contributionCount: 2,
      });
    });

    it("should get source tenants for a topic", async () => {
      await cacheClient.trackTenantContribution("tenant-1", "topic-1", 3);
      await cacheClient.trackTenantContribution("tenant-2", "topic-1", 2);
      await cacheClient.trackTenantContribution("tenant-1", "topic-2", 1);

      const sourceTenants = await cacheClient.getSourceTenants("topic-1");
      expect(sourceTenants).toHaveLength(2);
      expect(sourceTenants).toContain("tenant-1");
      expect(sourceTenants).toContain("tenant-2");
    });
  });

  describe("health check", () => {
    beforeEach(async () => {
      await cacheClient.initialize();
    });

    it("should return healthy status", async () => {
      const health = await cacheClient.healthCheck();
      expect(health.healthy).toBe(true);
    });
  });

  describe("error handling", () => {
    it("should handle operations when disabled", async () => {
      const disabledConfig = { ...config, enabled: false };
      const disabledClient = new DistributedCacheClient(disabledConfig, logger);
      await disabledClient.initialize();

      await expect(disabledClient.set("key", "value")).resolves.not.toThrow();
      expect(await disabledClient.get("key")).toBeNull();
      expect(await disabledClient.exists("key")).toBe(false);
      expect(await disabledClient.delete("key")).toBe(false);

      await disabledClient.shutdown();
    });
  });
});
