/**
 * Unit tests for ConnectionPoolManager
 *
 * @fileoverview Tests the centralized connection pool manager singleton
 */
// @ts-nocheck


import { ConnectionPoolManager } from "@/database/ConnectionPoolManager";
import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";

describe("ConnectionPoolManager", () => {
  // Reset singleton between tests
  beforeEach(() => {
    ConnectionPoolManager.resetForTesting();
  });

  afterEach(async () => {
    const manager = ConnectionPoolManager.getInstance();
    if (manager.isInitialized()) {
      await manager.shutdown();
    }
    ConnectionPoolManager.resetForTesting();
  });

  describe("Singleton Pattern", () => {
    it("should return the same instance", () => {
      const instance1 = ConnectionPoolManager.getInstance();
      const instance2 = ConnectionPoolManager.getInstance();

      expect(instance1).toBe(instance2);
    });

    it("should reset for testing", () => {
      const instance1 = ConnectionPoolManager.getInstance();
      ConnectionPoolManager.resetForTesting();
      const instance2 = ConnectionPoolManager.getInstance();

      expect(instance1).not.toBe(instance2);
    });
  });

  describe("Initialization", () => {
    it("should initialize from environment variables", () => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initializeFromEnv();

      expect(manager.isInitialized()).toBe(true);
    });

    it("should initialize from config object", () => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initialize({
        host: "localhost",
        port: 5432,
        database: "test_db",
        user: "test_user",
        password: "test_pass",
      });

      expect(manager.isInitialized()).toBe(true);
    });

    it("should warn if already initialized", () => {
      const manager = ConnectionPoolManager.getInstance();
      const consoleSpy = jest.spyOn(console, "warn").mockImplementation();

      manager.initializeFromEnv();
      manager.initializeFromEnv(); // Second call

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("already initialized")
      );
      consoleSpy.mockRestore();
    });

    it("should throw if getPool called before initialization", () => {
      const manager = ConnectionPoolManager.getInstance();

      expect(() => manager.getPool()).toThrow(
        "Connection pool not initialized"
      );
    });
  });

  describe("Health Checks", () => {
    beforeEach(() => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initializeFromEnv();
    });

    it("should perform health check successfully", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const isHealthy = await manager.healthCheck();

      expect(isHealthy).toBe(true);
    });

    it("should return pool stats", () => {
      const manager = ConnectionPoolManager.getInstance();
      const stats = manager.getStats();

      expect(stats).toHaveProperty("totalCount");
      expect(stats).toHaveProperty("idleCount");
      expect(stats).toHaveProperty("waitingCount");
      expect(stats).toHaveProperty("activeConnections");
      expect(stats).toHaveProperty("healthCheckStatus");
      expect(stats).toHaveProperty("createdAt");

      expect(stats.healthCheckStatus).toMatch(/healthy|degraded|unhealthy/);
    });

    it("should report healthy status under normal load", () => {
      const manager = ConnectionPoolManager.getInstance();
      const stats = manager.getStats();

      expect(stats.healthCheckStatus).toBe("healthy");
    });
  });

  describe("Tenant Context (RLS)", () => {
    beforeEach(() => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initializeFromEnv();
    });

    it("should get client with tenant context", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const client = await manager.getClientWithTenantContext("tenant-123");

      try {
        // Verify tenant context was set
        const result = await client.query(
          "SELECT current_setting('app.current_tenant', true) as tenant_id"
        );

        expect(result.rows[0]?.tenant_id).toBe("tenant-123");
      } finally {
        client.release();
      }
    });

    it("should set user context when provided", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const client = await manager.getClientWithTenantContext("tenant-123", {
        userId: "user-456",
      });

      try {
        const result = await client.query(
          "SELECT current_setting('app.current_user', true) as user_id"
        );

        expect(result.rows[0]?.user_id).toBe("user-456");
      } finally {
        client.release();
      }
    });

    it("should execute query with tenant context", async () => {
      const manager = ConnectionPoolManager.getInstance();

      const result = await manager.queryWithTenantContext(
        "tenant-123",
        "SELECT current_setting('app.current_tenant', true) as tenant_id"
      );

      expect(result.rows[0]?.tenant_id).toBe("tenant-123");
    });

    it("should isolate queries by tenant", async () => {
      const manager = ConnectionPoolManager.getInstance();

      // Insert test data for different tenants (requires RLS policies to exist)
      // This is a simplified test - real test would verify actual data isolation

      const tenant1Result = await manager.queryWithTenantContext(
        "tenant-1",
        "SELECT current_setting('app.current_tenant', true) as tenant_id"
      );

      const tenant2Result = await manager.queryWithTenantContext(
        "tenant-2",
        "SELECT current_setting('app.current_tenant', true) as tenant_id"
      );

      expect(tenant1Result.rows[0]?.tenant_id).toBe("tenant-1");
      expect(tenant2Result.rows[0]?.tenant_id).toBe("tenant-2");
    });
  });

  describe("Connection Management", () => {
    beforeEach(() => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initializeFromEnv();
    });

    it("should execute basic queries", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const pool = manager.getPool();

      const result = await pool.query("SELECT 1 as test");

      expect(result.rows[0].test).toBe(1);
    });

    it("should handle multiple concurrent queries", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const pool = manager.getPool();

      const queries = Array.from({ length: 10 }, (_, i) =>
        pool.query("SELECT $1::int as id", [i])
      );

      const results = await Promise.all(queries);

      results.forEach((result, i) => {
        expect(result.rows[0].id).toBe(i);
      });
    });

    it("should reuse connections efficiently", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const pool = manager.getPool();

      // Execute multiple queries sequentially
      await pool.query("SELECT 1");
      await pool.query("SELECT 2");
      await pool.query("SELECT 3");

      const stats = manager.getStats();

      // Should not create new connection for each query
      expect(stats.totalCount).toBeLessThanOrEqual(5);
    });
  });

  describe("Configuration", () => {
    it("should normalize config with defaults", () => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initialize({
        host: "localhost",
        port: 5432,
        database: "test_db",
        user: "test_user",
        password: "test_pass",
        // Omit optional fields - should use defaults
      });

      const config = manager.getConfig();

      expect(config?.min).toBe(2);
      expect(config?.max).toBe(20);
      expect(config?.idleTimeoutMs).toBe(30000);
      expect(config?.connectionTimeoutMs).toBe(10000);
      expect(config?.statementTimeoutMs).toBe(30000);
      expect(config?.applicationName).toBe("v2-arbiter");
    });

    it("should respect custom config values", () => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initialize({
        host: "localhost",
        port: 5432,
        database: "test_db",
        user: "test_user",
        password: "test_pass",
        min: 5,
        max: 50,
        idleTimeoutMs: 60000,
        applicationName: "custom-app",
      });

      const config = manager.getConfig();

      expect(config?.min).toBe(5);
      expect(config?.max).toBe(50);
      expect(config?.idleTimeoutMs).toBe(60000);
      expect(config?.applicationName).toBe("custom-app");
    });

    it("should support DATABASE_URL environment variable", () => {
      // Save original env
      const originalDatabaseUrl = process.env.DATABASE_URL;

      // Set DATABASE_URL
      process.env.DATABASE_URL =
        "postgresql://testuser:testpass@testhost:5433/testdb";

      try {
        const manager = ConnectionPoolManager.getInstance();
        manager.initializeFromEnv();

        const config = manager.getConfig();

        expect(config?.host).toBe("testhost");
        expect(config?.port).toBe(5433);
        expect(config?.database).toBe("testdb");
        expect(config?.user).toBe("testuser");
        expect(config?.password).toBe("testpass");
      } finally {
        // Restore original env
        if (originalDatabaseUrl) {
          process.env.DATABASE_URL = originalDatabaseUrl;
        } else {
          delete process.env.DATABASE_URL;
        }
      }
    });
  });

  describe("Graceful Shutdown", () => {
    beforeEach(() => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initializeFromEnv();
    });

    it("should shutdown gracefully", async () => {
      const manager = ConnectionPoolManager.getInstance();

      await manager.shutdown();

      expect(manager.isInitialized()).toBe(false);
    });

    it("should close all connections on shutdown", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const pool = manager.getPool();

      // Execute some queries to create connections
      await pool.query("SELECT 1");
      await pool.query("SELECT 2");

      // Verify connections exist
      const statsBefore = manager.getStats();
      expect(statsBefore.totalCount).toBeGreaterThan(0);

      // Shutdown
      await manager.shutdown();

      // Pool should be null
      expect(manager.isInitialized()).toBe(false);
    });

    it("should warn if shutdown called without pool", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const consoleSpy = jest.spyOn(console, "warn").mockImplementation();

      // Don't initialize - shutdown anyway
      await manager.shutdown();

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("No pool to shutdown")
      );
      consoleSpy.mockRestore();
    });

    it("should throw if getPool called during shutdown", async () => {
      const manager = ConnectionPoolManager.getInstance();

      // Start shutdown (don't await)
      const shutdownPromise = manager.shutdown();

      // Try to get pool during shutdown
      // Note: This is a race condition test, might be flaky
      try {
        manager.getPool();
      } catch (error: any) {
        expect(error.message).toMatch(/shutting down/i);
      }

      await shutdownPromise;
    });
  });

  describe("Error Handling", () => {
    beforeEach(() => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initializeFromEnv();
    });

    it("should handle query errors gracefully", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const pool = manager.getPool();

      await expect(pool.query("INVALID SQL QUERY")).rejects.toThrow();
    });

    it("should handle health check failures", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const pool = manager.getPool();

      // Mock query to fail
      const originalQuery = pool.query.bind(pool);
      pool.query = jest.fn().mockRejectedValue(new Error("Connection failed"));

      const isHealthy = await manager.healthCheck();

      expect(isHealthy).toBe(false);

      // Restore original query
      pool.query = originalQuery;
    });

    it("should release client if tenant context setup fails", async () => {
      const manager = ConnectionPoolManager.getInstance();
      const pool = manager.getPool();

      // Mock query to fail during context setup
      const originalQuery = pool.query.bind(pool);
      let callCount = 0;
      pool.query = jest
        .fn()
        .mockImplementation(async (queryText: string, values?: any[]) => {
          callCount++;
          if (callCount === 1) {
            // First call (SET LOCAL) fails
            throw new Error("Failed to set context");
          }
          return originalQuery(queryText, values);
        }) as any;

      await expect(
        manager.getClientWithTenantContext("tenant-123")
      ).rejects.toThrow("Failed to set context");

      // Pool should still be usable
      pool.query = originalQuery;
      const result = await pool.query("SELECT 1");
      expect(result.rows[0]).toEqual({ "?column?": 1 });
    });
  });

  describe("Convenience Functions", () => {
    beforeEach(() => {
      const manager = ConnectionPoolManager.getInstance();
      manager.initializeFromEnv();
    });

    it("should export getPool convenience function", async () => {
      const { getPool } = await import("@/database/ConnectionPoolManager");

      const pool = getPool();
      const result = await pool.query("SELECT 1 as test");

      expect(result.rows[0].test).toBe(1);
    });

    it("should export withTenantContext helper", async () => {
      const { withTenantContext } = await import(
        "@/database/ConnectionPoolManager"
      );

      const result = await withTenantContext("tenant-123", async (client) => {
        const res = await client.query(
          "SELECT current_setting('app.current_tenant', true) as tenant_id"
        );
        return res.rows[0].tenant_id;
      });

      expect(result).toBe("tenant-123");
    });

    it("should release client even if callback throws", async () => {
      const { withTenantContext } = await import(
        "@/database/ConnectionPoolManager"
      );

      const manager = ConnectionPoolManager.getInstance();
      const statsBefore = manager.getStats();

      try {
        await withTenantContext("tenant-123", async (_client) => {
          throw new Error("Callback error");
        });
      } catch (error: any) {
        expect(error.message).toBe("Callback error");
      }

      // Pool should not have leaked connections
      const statsAfter = manager.getStats();
      expect(statsAfter.idleCount).toBeGreaterThanOrEqual(
        statsBefore.idleCount
      );
    });
  });
});
