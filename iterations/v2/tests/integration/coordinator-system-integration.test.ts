/**
 * Coordinator System Integration Tests
 *
 * Tests the complete coordinator system integration including:
 * - Load balancing with health monitoring
 * - Component failure detection and recovery
 * - System-wide health aggregation
 * - Dynamic component registration/removal
 *
 * @author @darianrosebrook
 */

import { ComponentHealthMonitor } from "../../src/coordinator/ComponentHealthMonitor";
import { FailureManager } from "../../src/coordinator/FailureManager";
import { LoadBalancer } from "../../src/coordinator/LoadBalancer";
import { SystemCoordinator } from "../../src/coordinator/SystemCoordinator";
import {
  ComponentCapabilities,
  ComponentRegistration,
  ComponentType,
  HealthStatus,
  SystemCoordinatorConfig,
} from "../../src/types/coordinator";

describe("Coordinator System Integration", () => {
  let systemCoordinator: SystemCoordinator;
  let loadBalancer: LoadBalancer;
  let healthMonitor: ComponentHealthMonitor;
  let failureManager: FailureManager;

  // Test components
  const createTestComponent = (
    id: string,
    type: ComponentType = ComponentType.TASK_ROUTER,
    capabilities: Partial<ComponentCapabilities> = {}
  ): ComponentRegistration => ({
    id,
    name: `Test ${type} ${id}`,
    type,
    endpoint: `http://localhost:3000/${id}`,
    healthCheck: {
      endpoint: "/health",
      method: "GET",
      timeout: 5000,
      interval: 30000,
      retries: 3,
    },
    capabilities: {
      maxConcurrentTasks: 10,
      supportedTaskTypes: ["test"],
      performanceMetrics: true,
      healthMonitoring: true,
      constitutionalCompliance: true,
      ...capabilities,
    } as ComponentCapabilities,
    dependencies: [],
    metadata: {},
  });

  const createCoordinatorConfig = (): SystemCoordinatorConfig => ({
    healthCheckInterval: 30000,
    failureThreshold: 3,
    recoveryTimeout: 60000,
    loadBalancingEnabled: true,
    autoScalingEnabled: false,
    maxComponentsPerType: 10,
  });

  beforeEach(async () => {
    // Create dependencies in correct order
    healthMonitor = new ComponentHealthMonitor();
    const config = createCoordinatorConfig();

    systemCoordinator = new SystemCoordinator(config, healthMonitor);
    loadBalancer = new LoadBalancer(systemCoordinator);
    failureManager = new FailureManager(systemCoordinator, config);
  });

  afterEach(async () => {
    await systemCoordinator.stop();
  });

  describe("Component Lifecycle Integration", () => {
    it("should handle complete component registration and health monitoring", async () => {
      const component = createTestComponent("test-agent-1");

      // Register component
      await systemCoordinator.registerComponent(component);

      // Health monitor should detect the component
      const health = systemCoordinator.getComponentHealth(component.id);
      expect(health).toBeDefined();
      expect(health?.status).toBe(HealthStatus.HEALTHY);

      // Load balancer should be able to select the component
      const candidates = systemCoordinator.getComponentsByType(component.type);
      expect(candidates).toHaveLength(1);

      const selected = await loadBalancer.selectComponent(candidates, {
        type: "test",
      });
      expect(selected.id).toBe(component.id);
    });

    it("should handle component failure and recovery", async () => {
      const component = createTestComponent("test-agent-2");

      // Register component
      await systemCoordinator.registerComponent(component);

      // Simulate component failure by triggering failure handling
      await systemCoordinator.handleComponentFailure(
        component.id,
        new Error("Connection timeout")
      );

      // Failure manager should have recorded the failure
      const failureStats = failureManager.getFailureStats();
      expect(failureStats.totalFailures).toBeGreaterThan(0);

      // Healthy components should still be available for selection
      const healthyCandidates = systemCoordinator.getHealthyComponents(
        component.type
      );

      // The failed component should not be in healthy list
      const failedComponent = healthyCandidates.find(
        (c) => c.id === component.id
      );
      expect(failedComponent).toBeUndefined();

      // Note: Recovery would need to be handled by the health monitoring system
      // This test focuses on failure detection and isolation
    });

    it("should handle dynamic component removal", async () => {
      const component1 = createTestComponent("test-agent-3");
      const component2 = createTestComponent("test-agent-4");

      // Register both components
      await systemCoordinator.registerComponent(component1);
      await systemCoordinator.registerComponent(component2);

      // Both should be available
      let candidates = systemCoordinator.getComponentsByType(component1.type);
      expect(candidates).toHaveLength(2);

      // Remove one component
      await systemCoordinator.unregisterComponent(component1.id);

      // Only one should remain
      candidates = systemCoordinator.getComponentsByType(component1.type);
      expect(candidates).toHaveLength(1);
      expect(candidates[0].id).toBe(component2.id);

      // Load balancer should adapt
      const selected = await loadBalancer.selectComponent(candidates, {
        type: "test",
      });
      expect(selected.id).toBe(component2.id);
    });
  });

  describe("Load Balancing with Health Monitoring", () => {
    it("should distribute load based on component health and capabilities", async () => {
      // Create components with different capabilities
      const fastComponent = createTestComponent(
        "fast-agent",
        ComponentType.TASK_ROUTER,
        {
          maxConcurrentTasks: 20,
        }
      );
      const slowComponent = createTestComponent(
        "slow-agent",
        ComponentType.TASK_ROUTER,
        {
          maxConcurrentTasks: 5,
        }
      );

      await systemCoordinator.registerComponent(fastComponent);
      await systemCoordinator.registerComponent(slowComponent);

      const candidates = [fastComponent, slowComponent];

      // Load balancer should prefer the faster, more capable component
      const selections = [];
      for (let i = 0; i < 10; i++) {
        const selected = await loadBalancer.selectComponent(candidates, {
          type: "test",
        });
        selections.push(selected.id);
      }

      // Fast component should be selected more often
      const fastSelections = selections.filter(
        (id) => id === fastComponent.id
      ).length;
      const slowSelections = selections.filter(
        (id) => id === slowComponent.id
      ).length;

      expect(fastSelections).toBeGreaterThan(slowSelections);
    });

    it("should handle health degradation during load balancing", async () => {
      const component1 = createTestComponent("agent-5");
      const component2 = createTestComponent("agent-6");

      await systemCoordinator.registerComponent(component1);
      await systemCoordinator.registerComponent(component2);

      const candidates = [component1, component2];

      // Initial selections should distribute between both
      const initialSelections = [];
      for (let i = 0; i < 5; i++) {
        const selected = await loadBalancer.selectComponent(candidates, {
          type: "test",
        });
        initialSelections.push(selected.id);
      }

      const uniqueInitial = new Set(initialSelections);
      expect(uniqueInitial.size).toBeGreaterThan(1);

      // Simulate component degradation through failure
      await systemCoordinator.handleComponentFailure(
        component1.id,
        new Error("Performance degradation")
      );

      // Subsequent selections should avoid the degraded component
      const degradedSelections = [];
      for (let i = 0; i < 5; i++) {
        const selected = await loadBalancer.selectComponent(candidates, {
          type: "test",
        });
        degradedSelections.push(selected.id);
      }

      const component1Selections = degradedSelections.filter(
        (id) => id === component1.id
      ).length;
      const component2Selections = degradedSelections.filter(
        (id) => id === component2.id
      ).length;

      expect(component2Selections).toBeGreaterThan(component1Selections);
    });

    it("should recover from complete component failure", async () => {
      const component1 = createTestComponent("agent-7");
      const component2 = createTestComponent("agent-8");

      await systemCoordinator.registerComponent(component1);
      await systemCoordinator.registerComponent(component2);

      // Both healthy initially
      const candidates = [component1, component2];

      // Make some selections
      for (let i = 0; i < 3; i++) {
        await loadBalancer.selectComponent(candidates, { type: "test" });
      }

      // Fail component1 through multiple failures
      for (let i = 0; i < 5; i++) {
        await systemCoordinator.handleComponentFailure(
          component1.id,
          new Error(`Critical failure ${i}`)
        );
      }

      // System should continue working with component2 (which should still be healthy)
      const selected = await loadBalancer.selectComponent([component2], {
        type: "test",
      });
      expect(selected.id).toBe(component2.id);

      // Both should be available again
      const recoveredCandidates = await systemCoordinator.getComponentsByType(
        component1.type
      );
      expect(recoveredCandidates).toHaveLength(2);
    });
  });

  describe("System Health Aggregation", () => {
    it("should aggregate health across all components", async () => {
      const healthyComp = createTestComponent("healthy-comp");
      const degradedComp = createTestComponent("degraded-comp");
      const unhealthyComp = createTestComponent("unhealthy-comp");

      await systemCoordinator.registerComponent(healthyComp);
      await systemCoordinator.registerComponent(degradedComp);
      await systemCoordinator.registerComponent(unhealthyComp);

      // Trigger failures to simulate different health states
      await systemCoordinator.handleComponentFailure(
        degradedComp.id,
        new Error("High latency")
      );
      await systemCoordinator.handleComponentFailure(
        unhealthyComp.id,
        new Error("Connection failed")
      );
      await systemCoordinator.handleComponentFailure(
        unhealthyComp.id,
        new Error("Connection failed again")
      );

      // Get system health
      const systemHealth = systemCoordinator.getSystemHealth();

      expect(systemHealth.status).toBeDefined();
      expect(systemHealth.components.total).toBe(3);
      expect(systemHealth.components.healthy).toBeGreaterThanOrEqual(1);
      expect(systemHealth.lastUpdate).toBeInstanceOf(Date);
    });

    it("should handle component type-specific queries", async () => {
      const routerComp = createTestComponent(
        "router-1",
        ComponentType.TASK_ROUTER
      );
      const validatorComp = createTestComponent(
        "validator-1",
        ComponentType.CAWS_VALIDATOR
      );

      await systemCoordinator.registerComponent(routerComp);
      await systemCoordinator.registerComponent(validatorComp);

      const routers = systemCoordinator.getComponentsByType(
        ComponentType.TASK_ROUTER
      );
      const validators = systemCoordinator.getComponentsByType(
        ComponentType.CAWS_VALIDATOR
      );
      const healthyRouters = systemCoordinator.getHealthyComponents(
        ComponentType.TASK_ROUTER
      );

      expect(routers).toHaveLength(1);
      expect(validators).toHaveLength(1);
      expect(routers[0].type).toBe(ComponentType.TASK_ROUTER);
      expect(validators[0].type).toBe(ComponentType.CAWS_VALIDATOR);
      expect(healthyRouters).toHaveLength(1);
    });
  });

  describe("Failure Management Integration", () => {
    it("should track and resolve component failures", async () => {
      const component = createTestComponent("failing-agent");

      await systemCoordinator.registerComponent(component);

      // Simulate multiple failures
      for (let i = 0; i < 3; i++) {
        await systemCoordinator.handleComponentFailure(
          component.id,
          new Error(`Failure ${i + 1}`)
        );
      }

      // Failure manager should track the pattern
      const failureStats = failureManager.getFailureStats();
      expect(failureStats.totalFailures).toBeGreaterThanOrEqual(3);
      expect(failureStats.activeRecoveries).toBeGreaterThanOrEqual(1);

      // Healthy components should exclude the failed one
      const healthyComponents = systemCoordinator.getHealthyComponents(
        component.type
      );
      const failedComponent = healthyComponents.find(
        (c) => c.id === component.id
      );
      expect(failedComponent).toBeUndefined();
    });

    it("should trigger automatic recovery actions", async () => {
      const component = createTestComponent("auto-recover-agent");

      await systemCoordinator.registerComponent(component);

      // Fail the component
      await systemCoordinator.handleComponentFailure(
        component.id,
        new Error("Connection timeout")
      );

      // Check failure stats
      const failureStats = failureManager.getFailureStats();
      expect(failureStats.activeRecoveries).toBeGreaterThan(0);

      // Component should be isolated from healthy selections
      const healthyCandidates = systemCoordinator.getHealthyComponents(
        component.type
      );
      const failedComponent = healthyCandidates.find(
        (c) => c.id === component.id
      );
      expect(failedComponent).toBeUndefined();
    });
  });

  describe("Performance and Scalability", () => {
    it("should handle high-frequency failure events", async () => {
      const component = createTestComponent("perf-test-agent");
      await systemCoordinator.registerComponent(component);

      const startTime = Date.now();

      // Simulate rapid failure events
      for (let i = 0; i < 50; i++) {
        await systemCoordinator.handleComponentFailure(
          component.id,
          new Error(`Test failure ${i}`)
        );
      }

      const duration = Date.now() - startTime;

      // Should complete within reasonable time (under 3 seconds)
      expect(duration).toBeLessThan(3000);

      // System should still be functional
      const health = systemCoordinator.getSystemHealth();
      expect(health).toBeDefined();
    });

    it("should maintain performance with many components", async () => {
      const componentCount = 20;
      const components: ComponentRegistration[] = [];

      // Register many components
      for (let i = 0; i < componentCount; i++) {
        const component = createTestComponent(`bulk-agent-${i}`);
        components.push(component);
        await systemCoordinator.registerComponent(component);
      }

      const startTime = Date.now();

      // Perform load balancing operations
      for (let i = 0; i < 30; i++) {
        const randomComponent =
          components[Math.floor(Math.random() * components.length)];
        await loadBalancer.selectComponent([randomComponent], { type: "test" });
      }

      const duration = Date.now() - startTime;

      // Should handle bulk operations efficiently
      expect(duration).toBeLessThan(5000);

      // All components should still be registered
      const allComponents = systemCoordinator.getAllComponents();
      expect(allComponents.length).toBe(componentCount);
    });
  });

  describe("Edge Cases and Error Handling", () => {
    it("should handle component registration conflicts", async () => {
      const component1 = createTestComponent("conflict-agent");
      const component2 = createTestComponent("conflict-agent"); // Same ID

      await systemCoordinator.registerComponent(component1);

      // Second registration with same ID should either update or fail gracefully
      try {
        await systemCoordinator.registerComponent(component2);
      } catch (error) {
        // Expected - duplicate registration
        expect(error).toBeDefined();
      }

      // Only one component should exist
      const components = await systemCoordinator.getAllComponents();
      const matchingComponents = components.filter(
        (c) => c.id === "conflict-agent"
      );
      expect(matchingComponents.length).toBe(1);
    });

    it("should handle failure reports for non-existent components", async () => {
      // Should not throw error when reporting failures for non-existent components
      await expect(
        systemCoordinator.handleComponentFailure(
          "non-existent",
          new Error("Test")
        )
      ).resolves.not.toThrow();
    });

    it("should handle load balancing with no available components", async () => {
      // Try to select from empty candidate list
      await expect(
        loadBalancer.selectComponent([], { type: "test" })
      ).rejects.toThrow("No healthy components available");
    });

    it("should handle concurrent component operations", async () => {
      const componentBase = "concurrent-agent";
      const operations = [];

      // Create concurrent registration operations
      for (let i = 0; i < 5; i++) {
        const component = createTestComponent(`${componentBase}-${i}`);
        operations.push(systemCoordinator.registerComponent(component));
      }

      // Execute all concurrently
      await Promise.all(operations);

      // All components should be registered
      const allComponents = systemCoordinator.getAllComponents();
      const concurrentComponents = allComponents.filter((c) =>
        c.id.startsWith(componentBase)
      );
      expect(concurrentComponents.length).toBe(5);
    });
  });
});
