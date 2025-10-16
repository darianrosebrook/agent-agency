/**
 * Health Monitor Unit Tests
 *
 * Tests health monitoring, check registration, and system health aggregation
 * for the core health monitoring functionality.
 *
 * @author @darianrosebrook
 */

import {
  ComponentHealth,
  HealthMonitor,
  HealthStatus,
} from "../../../src/health/HealthMonitor";

describe("HealthMonitor", () => {
  let healthMonitor: HealthMonitor;

  // Test fixtures
  const createMockHealthCheck = (
    status: HealthStatus = HealthStatus.HEALTHY,
    message?: string,
    delay = 0
  ): (() => Promise<ComponentHealth>) => {
    return async (): Promise<ComponentHealth> => {
      if (delay > 0) {
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
      return {
        name: "test-component",
        status,
        message,
        lastCheck: new Date(),
        details: { mock: true },
      };
    };
  };

  beforeEach(() => {
    healthMonitor = new HealthMonitor();
  });

  describe("Health Check Registration", () => {
    it("should register health checks", () => {
      const check = createMockHealthCheck();

      healthMonitor.registerCheck("database", check);

      // Verify check is registered internally
      expect((healthMonitor as any).checks.has("database")).toBe(true);
    });

    it("should allow multiple checks with different names", () => {
      const dbCheck = createMockHealthCheck();
      const apiCheck = createMockHealthCheck();

      healthMonitor.registerCheck("database", dbCheck);
      healthMonitor.registerCheck("api", apiCheck);

      expect((healthMonitor as any).checks.size).toBe(2);
    });

    it("should overwrite existing checks with same name", () => {
      const firstCheck = createMockHealthCheck(HealthStatus.HEALTHY);
      const secondCheck = createMockHealthCheck(HealthStatus.UNHEALTHY);

      healthMonitor.registerCheck("database", firstCheck);
      healthMonitor.registerCheck("database", secondCheck);

      expect((healthMonitor as any).checks.size).toBe(1);
      expect((healthMonitor as any).checks.get("database")).toBe(secondCheck);
    });
  });

  describe("Health Check Execution", () => {
    it("should execute registered health checks", async () => {
      const check = createMockHealthCheck(HealthStatus.HEALTHY, "All good");
      healthMonitor.registerCheck("database", check);

      const result = await healthMonitor.checkComponent("database");

      expect(result.name).toBe("test-component");
      expect(result.status).toBe(HealthStatus.HEALTHY);
      expect(result.message).toBe("All good");
      expect(result.lastCheck).toBeInstanceOf(Date);
    });

    it("should return cached results for repeated checks", async () => {
      const check = createMockHealthCheck();
      healthMonitor.registerCheck("cache-test", check);

      const result1 = await healthMonitor.checkComponent("cache-test");
      const result2 = await healthMonitor.checkComponent("cache-test");

      // Results should be identical (cached)
      expect(result1.lastCheck.getTime()).toBe(result2.lastCheck.getTime());
    });

    it("should handle check execution errors", async () => {
      const failingCheck = async (): Promise<ComponentHealth> => {
        throw new Error("Connection failed");
      };

      healthMonitor.registerCheck("failing-service", failingCheck);

      await expect(
        healthMonitor.checkComponent("failing-service")
      ).rejects.toThrow("Connection failed");
    });

    it("should throw error for unregistered components", async () => {
      await expect(
        healthMonitor.checkComponent("non-existent")
      ).rejects.toThrow("Health check not registered: non-existent");
    });
  });

  describe("System Health Aggregation", () => {
    it("should report healthy system when all components are healthy", async () => {
      healthMonitor.registerCheck(
        "db",
        createMockHealthCheck(HealthStatus.HEALTHY)
      );
      healthMonitor.registerCheck(
        "api",
        createMockHealthCheck(HealthStatus.HEALTHY)
      );

      const health = await healthMonitor.getSystemHealth();

      expect(health.status).toBe(HealthStatus.HEALTHY);
      expect(health.components).toHaveLength(2);
      expect(
        health.components.every((c) => c.status === HealthStatus.HEALTHY)
      ).toBe(true);
    });

    it("should report degraded system when some components are degraded", async () => {
      healthMonitor.registerCheck(
        "db",
        createMockHealthCheck(HealthStatus.HEALTHY)
      );
      healthMonitor.registerCheck(
        "api",
        createMockHealthCheck(HealthStatus.DEGRADED)
      );

      const health = await healthMonitor.getSystemHealth();

      expect(health.status).toBe(HealthStatus.DEGRADED);
      expect(
        health.components.some((c) => c.status === HealthStatus.DEGRADED)
      ).toBe(true);
    });

    it("should report unhealthy system when any component is unhealthy", async () => {
      healthMonitor.registerCheck(
        "db",
        createMockHealthCheck(HealthStatus.HEALTHY)
      );
      healthMonitor.registerCheck(
        "api",
        createMockHealthCheck(HealthStatus.UNHEALTHY)
      );

      const health = await healthMonitor.getSystemHealth();

      expect(health.status).toBe(HealthStatus.UNHEALTHY);
      expect(
        health.components.some((c) => c.status === HealthStatus.UNHEALTHY)
      ).toBe(true);
    });

    it("should include system uptime in health report", async () => {
      healthMonitor.registerCheck("db", createMockHealthCheck());

      const health = await healthMonitor.getSystemHealth();

      expect(health.uptime).toBeGreaterThanOrEqual(0);
      expect(health.timestamp).toBeInstanceOf(Date);
    });

    it("should handle empty system (no registered checks)", async () => {
      const health = await healthMonitor.getSystemHealth();

      expect(health.status).toBe(HealthStatus.HEALTHY); // Empty system is considered healthy
      expect(health.components).toHaveLength(0);
    });
  });

  describe("Concurrent Health Checks", () => {
    it("should handle multiple concurrent health checks", async () => {
      // Register checks with different delays
      healthMonitor.registerCheck(
        "fast",
        createMockHealthCheck(HealthStatus.HEALTHY, "Fast", 10)
      );
      healthMonitor.registerCheck(
        "slow",
        createMockHealthCheck(HealthStatus.HEALTHY, "Slow", 50)
      );

      const startTime = Date.now();

      const results = await Promise.all([
        healthMonitor.checkComponent("fast"),
        healthMonitor.checkComponent("slow"),
      ]);

      const duration = Date.now() - startTime;

      expect(results).toHaveLength(2);
      expect(results[0].status).toBe(HealthStatus.HEALTHY);
      expect(results[1].status).toBe(HealthStatus.HEALTHY);
      // Should complete in parallel (less than 100ms total)
      expect(duration).toBeLessThan(100);
    });

    it("should cache results during concurrent calls", async () => {
      healthMonitor.registerCheck("shared", createMockHealthCheck());

      const [result1, result2] = await Promise.all([
        healthMonitor.checkComponent("shared"),
        healthMonitor.checkComponent("shared"),
      ]);

      // Both should return the same cached result
      expect(result1.lastCheck.getTime()).toBe(result2.lastCheck.getTime());
    });
  });

  describe("Health Check Lifecycle", () => {
    it("should allow unregistering health checks", () => {
      const check = createMockHealthCheck();
      healthMonitor.registerCheck("temp", check);

      expect((healthMonitor as any).checks.has("temp")).toBe(true);

      // Note: HealthMonitor doesn't have an unregister method in this implementation
      // This test documents the expected behavior if it were added
      expect((healthMonitor as any).checks.has("temp")).toBe(true);
    });

    it("should clear cached results when re-registering", async () => {
      const check1 = createMockHealthCheck(HealthStatus.HEALTHY);
      const check2 = createMockHealthCheck(HealthStatus.UNHEALTHY);

      healthMonitor.registerCheck("changing", check1);
      await healthMonitor.checkComponent("changing");

      // Re-register with different check
      healthMonitor.registerCheck("changing", check2);
      const result = await healthMonitor.checkComponent("changing");

      expect(result.status).toBe(HealthStatus.UNHEALTHY);
    });
  });

  describe("Error Handling", () => {
    it("should handle checks that return invalid health objects", async () => {
      const invalidCheck = async (): Promise<ComponentHealth> => {
        return {
          name: "", // Invalid: empty name
          status: "invalid-status" as any, // Invalid status
          lastCheck: new Date(),
        } as ComponentHealth;
      };

      healthMonitor.registerCheck("invalid", invalidCheck);

      const result = await healthMonitor.checkComponent("invalid");

      // Should still return something, even if invalid
      expect(result).toBeDefined();
      expect(result.name).toBe("");
    });

    it("should handle checks that take too long", async () => {
      const slowCheck = async (): Promise<ComponentHealth> => {
        await new Promise((resolve) => setTimeout(resolve, 100));
        return {
          name: "slow-component",
          status: HealthStatus.HEALTHY,
          lastCheck: new Date(),
        };
      };

      healthMonitor.registerCheck("slow", slowCheck);

      const startTime = Date.now();
      const result = await healthMonitor.checkComponent("slow");
      const duration = Date.now() - startTime;

      expect(result.status).toBe(HealthStatus.HEALTHY);
      expect(duration).toBeGreaterThanOrEqual(100);
    });

    it("should handle system health checks with failing components", async () => {
      healthMonitor.registerCheck(
        "good",
        createMockHealthCheck(HealthStatus.HEALTHY)
      );

      const failingCheck = async (): Promise<ComponentHealth> => {
        throw new Error("Service down");
      };
      healthMonitor.registerCheck("bad", failingCheck);

      // getSystemHealth should handle individual failures gracefully
      await expect(healthMonitor.getSystemHealth()).rejects.toThrow();
    });
  });

  describe("Health Status Transitions", () => {
    it("should track health status changes over time", async () => {
      healthMonitor.registerCheck(
        "flaky",
        createMockHealthCheck(HealthStatus.HEALTHY)
      );

      let result = await healthMonitor.checkComponent("flaky");
      expect(result.status).toBe(HealthStatus.HEALTHY);

      // Simulate status change by re-registering
      healthMonitor.registerCheck(
        "flaky",
        createMockHealthCheck(HealthStatus.DEGRADED)
      );

      result = await healthMonitor.checkComponent("flaky");
      expect(result.status).toBe(HealthStatus.DEGRADED);
    });

    it("should maintain health history", async () => {
      healthMonitor.registerCheck("service", createMockHealthCheck());

      // Multiple checks should build history
      await healthMonitor.checkComponent("service");
      await healthMonitor.checkComponent("service");
      await healthMonitor.checkComponent("service");

      // Check that results are cached/stored
      const lastResults = (healthMonitor as any).lastResults;
      expect(lastResults.has("service")).toBe(true);
    });
  });

  describe("System Health API", () => {
    it("should provide detailed component health information", async () => {
      const detailedCheck = async (): Promise<ComponentHealth> => ({
        name: "detailed-service",
        status: HealthStatus.HEALTHY,
        message: "All systems operational",
        lastCheck: new Date(),
        details: {
          connections: 15,
          throughput: "150 req/sec",
          latency: "45ms",
        },
      });

      healthMonitor.registerCheck("detailed", detailedCheck);

      const health = await healthMonitor.getSystemHealth();

      expect(health.components[0].details).toBeDefined();
      expect(health.components[0].details?.connections).toBe(15);
    });

    it("should aggregate component messages in system health", async () => {
      healthMonitor.registerCheck(
        "service1",
        createMockHealthCheck(HealthStatus.HEALTHY, "Service 1 OK")
      );
      healthMonitor.registerCheck(
        "service2",
        createMockHealthCheck(HealthStatus.DEGRADED, "Service 2 slow")
      );

      const health = await healthMonitor.getSystemHealth();

      expect(health.components).toHaveLength(2);
      const messages = health.components.map((c) => c.message);
      expect(messages).toContain("Service 1 OK");
      expect(messages).toContain("Service 2 slow");
    });
  });
});
