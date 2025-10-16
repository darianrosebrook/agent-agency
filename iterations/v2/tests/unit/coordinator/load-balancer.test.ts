/**
 * Load Balancer Unit Tests
 *
 * Tests load distribution, component selection, and performance tracking
 * for the system coordinator's load balancing functionality.
 *
 * @author @darianrosebrook
 */

import { LoadBalancer } from "../../../src/coordinator/LoadBalancer";
import { SystemCoordinator } from "../../../src/coordinator/SystemCoordinator";
import {
  ComponentCapabilities,
  ComponentHealth,
  ComponentRegistration,
  ComponentType,
  HealthStatus,
  RoutingPreferences,
} from "../../../src/types/coordinator";

describe("LoadBalancer", () => {
  let loadBalancer: LoadBalancer;
  let mockCoordinator: jest.Mocked<SystemCoordinator>;

  // Test fixtures
  const createComponent = (
    id: string,
    supportedTaskTypes: string[] = [],
    maxConcurrentTasks = 5
  ): ComponentRegistration => ({
    id,
    name: `Component ${id}`,
    type: ComponentType.TASK_ROUTER,
    endpoint: `http://localhost:3000/${id}`,
    healthCheck: {
      endpoint: "/health",
      method: "GET",
      timeout: 5000,
      interval: 30000,
      retries: 3,
    },
    capabilities: {
      maxConcurrentTasks,
      supportedTaskTypes,
      performanceMetrics: true,
      healthMonitoring: true,
      constitutionalCompliance: true,
    } as ComponentCapabilities,
    dependencies: [],
    metadata: {},
  });

  const createComponentHealth = (
    id: string,
    status = HealthStatus.HEALTHY,
    responseTime = 100
  ): ComponentHealth => ({
    id,
    status,
    lastCheck: new Date(),
    responseTime,
    errorCount: 0,
  });

  const mockPayload = { task: "test-task", priority: 1 };

  beforeEach(() => {
    mockCoordinator = {
      getComponentHealth: jest.fn(),
      getAllComponents: jest.fn(),
      registerComponent: jest.fn(),
      unregisterComponent: jest.fn(),
    } as any;

    loadBalancer = new LoadBalancer(mockCoordinator);
  });

  describe("Component Selection", () => {
    it("should return single candidate immediately", async () => {
      const component = createComponent("comp-1");
      const candidates = [component];

      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload
      );

      expect(result).toBe(component);
    });

    it("should select component with lowest load when no preferences", async () => {
      const lowLoadComp = createComponent("low-load", [], 0.2);
      const highLoadComp = createComponent("high-load", [], 0.8);
      const candidates = [highLoadComp, lowLoadComp];

      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload
      );

      expect(result.id).toBe("low-load");
    });

    it("should filter by capabilities preference", async () => {
      const codeComp = createComponent("code-agent", [
        "typescript",
        "javascript",
      ]);
      const generalComp = createComponent("general-agent", ["general"]);
      const candidates = [codeComp, generalComp];

      const preferences: RoutingPreferences = {
        capabilities: ["typescript"],
      };

      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload,
        preferences
      );

      expect(result.id).toBe("code-agent");
    });

    it("should exclude unhealthy components", async () => {
      const healthyComp = createComponent("healthy");
      const unhealthyComp = createComponent("unhealthy");

      // Mock health checks
      mockCoordinator.getComponentHealth.mockImplementation((id) => {
        if (id === "healthy") {
          return createComponentHealth("healthy", HealthStatus.HEALTHY, 100);
        } else {
          return createComponentHealth(
            "unhealthy",
            HealthStatus.UNHEALTHY,
            100
          );
        }
      });

      const candidates = [healthyComp, unhealthyComp];

      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload
      );

      expect(result.id).toBe("healthy");
    });

    it("should prefer components with better health", async () => {
      const fastComp = createComponent("fast");
      const slowComp = createComponent("slow");

      // Mock different response times
      mockCoordinator.getComponentHealth.mockImplementation((id) => {
        if (id === "fast") {
          return createComponentHealth("fast", HealthStatus.HEALTHY, 50);
        } else {
          return createComponentHealth("slow", HealthStatus.HEALTHY, 200);
        }
      });

      const candidates = [fastComp, slowComp];

      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload
      );

      expect(result.id).toBe("fast");
    });

    it("should balance load when components have equal scores", async () => {
      const comp1 = createComponent("comp-1", [], 0.5);
      const comp2 = createComponent("comp-2", [], 0.5);
      const comp3 = createComponent("comp-3", [], 0.5);

      const candidates = [comp1, comp2, comp3];

      // Run multiple selections to test load balancing
      const results = [];
      for (let i = 0; i < 10; i++) {
        const result = await loadBalancer.selectComponent(
          candidates,
          mockPayload
        );
        results.push(result.id);
      }

      // Should distribute across all components
      const uniqueSelections = new Set(results);
      expect(uniqueSelections.size).toBeGreaterThan(1);
    });
  });

  describe("Load Tracking", () => {
    it("should track request assignments", async () => {
      const component = createComponent("comp-1");
      const candidates = [component];

      await loadBalancer.selectComponent(candidates, mockPayload);

      // Verify request was tracked - check load distribution
      const loadDistribution = loadBalancer.getLoadDistribution();
      expect(loadDistribution.length).toBeGreaterThan(0);
      expect(loadDistribution.some((ld) => ld.componentId === "comp-1")).toBe(
        true
      );
    });

    it("should provide load statistics", () => {
      const stats = loadBalancer.getLoadStats();
      expect(stats).toBeDefined();
      expect(typeof stats.totalRequests).toBe("number");
      expect(typeof stats.averageResponseTime).toBe("number");
      expect(stats.requestsPerComponent).toBeDefined();
    });

    it("should calculate load distribution", async () => {
      const comp1 = createComponent("comp-1");
      const comp2 = createComponent("comp-2");
      const candidates = [comp1, comp2];

      // Make multiple requests
      for (let i = 0; i < 5; i++) {
        await loadBalancer.selectComponent(candidates, mockPayload);
      }

      const distribution = loadBalancer.getLoadDistribution();
      expect(distribution.length).toBeGreaterThan(0);
      expect(
        distribution.every((ld) => typeof ld.loadPercentage === "number")
      ).toBe(true);
    });
  });

  describe("Health Integration", () => {
    it("should query coordinator for component health", async () => {
      const component = createComponent("comp-1");
      const candidates = [component];

      mockCoordinator.getComponentHealth.mockReturnValue(
        createComponentHealth("comp-1", HealthStatus.HEALTHY, 100)
      );

      await loadBalancer.selectComponent(candidates, mockPayload);

      expect(mockCoordinator.getComponentHealth).toHaveBeenCalledWith("comp-1");
    });

    it("should handle health check failures gracefully", async () => {
      const component = createComponent("comp-1");
      const candidates = [component];

      mockCoordinator.getComponentHealth.mockImplementation(() => {
        throw new Error("Health check failed");
      });

      // Should still work with cached health data
      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload
      );
      expect(result.id).toBe("comp-1");
    });
  });

  describe("Performance Optimization", () => {
    it("should handle component capabilities limits", async () => {
      const limitedComp = createComponent("limited", [], 2); // maxConcurrentTasks = 2
      const unlimitedComp = createComponent("unlimited", [], 10);

      const candidates = [limitedComp, unlimitedComp];

      // Should prefer component with higher capacity
      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload
      );
      expect(result.id).toBe("unlimited");
    });

    it("should redistribute load when requested", async () => {
      // Add some components and make requests
      const comp1 = createComponent("comp-1");
      const comp2 = createComponent("comp-2");

      await loadBalancer.selectComponent([comp1], mockPayload);
      await loadBalancer.selectComponent([comp2], mockPayload);

      // Redistribute load
      await loadBalancer.redistributeLoad();

      // Should still have valid load distribution
      const distribution = loadBalancer.getLoadDistribution();
      expect(distribution.length).toBeGreaterThan(0);
    });

    it("should update component health status", async () => {
      const component = createComponent("comp-1");
      const health = createComponentHealth(
        "comp-1",
        HealthStatus.DEGRADED,
        500
      );

      await loadBalancer.updateComponentHealth(component.id, health);

      // Health should be updated internally
      const distribution = loadBalancer.getLoadDistribution();
      // Component should still be tracked
      expect(distribution.some((ld) => ld.componentId === "comp-1")).toBe(true);
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty candidate list", async () => {
      await expect(
        loadBalancer.selectComponent([], mockPayload)
      ).rejects.toThrow("No healthy components available");
    });

    it("should handle null/undefined preferences", async () => {
      const component = createComponent("comp-1");
      const candidates = [component];

      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload,
        undefined
      );
      expect(result.id).toBe("comp-1");
    });

    it("should handle very large candidate lists efficiently", async () => {
      const candidates = Array.from({ length: 100 }, (_, i) =>
        createComponent(`comp-${i}`)
      );

      const startTime = Date.now();
      const result = await loadBalancer.selectComponent(
        candidates,
        mockPayload
      );
      const duration = Date.now() - startTime;

      expect(result).toBeDefined();
      expect(duration).toBeLessThan(100); // Should complete quickly
    });

    it("should maintain state across multiple operations", async () => {
      const comp1 = createComponent("comp-1");
      const comp2 = createComponent("comp-2");

      // First selection
      await loadBalancer.selectComponent([comp1, comp2], mockPayload);

      // Second selection should still work
      const result2 = await loadBalancer.selectComponent(
        [comp1, comp2],
        mockPayload
      );

      // Should still make a valid selection
      expect(["comp-1", "comp-2"]).toContain(result2.id);
    });
  });

  describe("Load Distribution API", () => {
    it("should expose current load distribution", () => {
      const distribution = loadBalancer.getLoadDistribution();
      expect(Array.isArray(distribution)).toBe(true);
    });

    it("should provide component load statistics", () => {
      const stats = loadBalancer.getLoadStats();
      expect(stats).toBeDefined();
      expect(typeof stats.totalRequests).toBe("number");
      expect(typeof stats.averageResponseTime).toBe("number");
    });

    it("should handle component removal gracefully", async () => {
      const component = createComponent("comp-1");
      await loadBalancer.selectComponent([component], mockPayload);

      // Simulate component removal
      await loadBalancer.handleComponentRemoval(component);

      // Should still be able to get stats without crashing
      const stats = loadBalancer.getLoadStats();
      expect(stats).toBeDefined();
    });
  });
});
