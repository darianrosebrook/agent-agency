/**
 * System Coordinator Tests
 *
 * @author @darianrosebrook
 */

import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  jest,
} from "@jest/globals";
import {
  ComponentHealthMonitor,
  SystemCoordinator,
} from "../../../src/coordinator";
import { ComponentType, HealthStatus } from "../../../src/types/coordinator";

describe("SystemCoordinator", () => {
  let coordinator: SystemCoordinator;
  let healthMonitor: jest.Mocked<ComponentHealthMonitor>;

  beforeEach(() => {
    healthMonitor = {
      registerComponent: jest.fn(),
      unregisterComponent: jest.fn(),
      checkComponentHealth: jest.fn(),
      start: jest.fn(),
      stop: jest.fn(),
      on: jest.fn(),
    } as any;

    coordinator = new SystemCoordinator(
      {
        healthCheckInterval: 30000,
        failureThreshold: 3,
        recoveryTimeout: 300000,
        loadBalancingEnabled: true,
        autoScalingEnabled: false,
        maxComponentsPerType: 5,
      },
      healthMonitor
    );
  });

  afterEach(async () => {
    try {
      // Clean up coordinator resources
      if (coordinator && typeof (coordinator as any).shutdown === "function") {
        await (coordinator as any).shutdown();
      }
    } catch (error) {
      // Ignore cleanup errors in tests
    }
    // Clear all mocks
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should initialize with config", () => {
      expect(coordinator).toBeDefined();
    });

    it("should setup event handlers", () => {
      expect(healthMonitor.on).toHaveBeenCalled();
    });
  });

  describe("component registration", () => {
    const validRegistration = {
      id: "test-component",
      name: "Test Component",
      type: ComponentType.AGENT_REGISTRY,
      endpoint: "http://localhost:3001",
      healthCheck: {
        endpoint: "http://localhost:3001/health",
        method: "GET" as const,
        timeout: 5000,
        interval: 30000,
        retries: 3,
      },
      capabilities: {},
      dependencies: [],
      metadata: {},
    };

    it("should register component successfully", async () => {
      healthMonitor.registerComponent.mockResolvedValue(undefined);

      await coordinator.registerComponent(validRegistration);

      const component = coordinator.getComponent("test-component");
      expect(component).toEqual(validRegistration);
      expect(healthMonitor.registerComponent).toHaveBeenCalledWith(
        validRegistration
      );
    });

    it("should validate dependencies", async () => {
      const invalidRegistration = {
        ...validRegistration,
        dependencies: ["non-existent"],
      };

      await expect(
        coordinator.registerComponent(invalidRegistration)
      ).rejects.toThrow("Dependency non-existent not registered");
    });

    it("should unregister component", async () => {
      healthMonitor.registerComponent.mockResolvedValue(undefined);
      healthMonitor.unregisterComponent.mockResolvedValue(undefined);

      await coordinator.registerComponent(validRegistration);
      await coordinator.unregisterComponent("test-component");

      expect(coordinator.getComponent("test-component")).toBeUndefined();
      expect(healthMonitor.unregisterComponent).toHaveBeenCalledWith(
        "test-component"
      );
    });
  });

  describe("component queries", () => {
    beforeEach(async () => {
      healthMonitor.registerComponent.mockResolvedValue(undefined);

      await coordinator.registerComponent({
        id: "agent-reg-1",
        name: "Agent Registry 1",
        type: ComponentType.AGENT_REGISTRY,
        endpoint: "http://localhost:3001",
        healthCheck: {
          endpoint: "http://localhost:3001/health",
          method: "GET" as const,
          timeout: 5000,
          interval: 30000,
          retries: 3,
        },
        capabilities: {},
        dependencies: [],
        metadata: {},
      });

      await coordinator.registerComponent({
        id: "task-router-1",
        name: "Task Router 1",
        type: ComponentType.TASK_ROUTER,
        endpoint: "http://localhost:3002",
        healthCheck: {
          endpoint: "http://localhost:3002/health",
          method: "GET" as const,
          timeout: 5000,
          interval: 30000,
          retries: 3,
        },
        capabilities: {},
        dependencies: [],
        metadata: {},
      });
    });

    it("should get components by type", () => {
      const agentRegistries = coordinator.getComponentsByType(
        ComponentType.AGENT_REGISTRY
      );
      expect(agentRegistries).toHaveLength(1);
      expect(agentRegistries[0].id).toBe("agent-reg-1");
    });

    it("should get healthy components", () => {
      // Mock healthy status
      (coordinator as any).componentHealth.set("agent-reg-1", {
        id: "agent-reg-1",
        status: HealthStatus.HEALTHY,
        lastCheck: new Date(),
        responseTime: 100,
        errorCount: 0,
      });

      const healthy = coordinator.getHealthyComponents(
        ComponentType.AGENT_REGISTRY
      );
      expect(healthy).toHaveLength(1);
      expect(healthy[0].id).toBe("agent-reg-1");
    });
  });

  describe("system health", () => {
    it("should report system health", () => {
      const health = coordinator.getSystemHealth();

      expect(health).toHaveProperty("status");
      expect(health).toHaveProperty("components");
      expect(health.components.total).toBe(0);
      expect(health.components.healthy).toBe(0);
    });

    it("should get coordinator stats", () => {
      const stats = coordinator.getStats();

      expect(stats).toHaveProperty("components");
      expect(stats).toHaveProperty("health");
      expect(stats).toHaveProperty("load");
      expect(stats).toHaveProperty("failures");
    });
  });

  describe("request routing", () => {
    it("should throw error when no healthy components available", async () => {
      await expect(
        coordinator.routeRequest("task_routing", {})
      ).rejects.toThrow("No healthy components available");
    });
  });
});
