/**
 * @fileoverview Unit tests for LoadBalancer
 *
 * @author @darianrosebrook
 */

import { LoadBalancer } from "@/resources/LoadBalancer";
import { ResourceMonitor } from "@/resources/ResourceMonitor";
import {
  LoadBalancingStrategy,
  ResourceType,
  TaskPriority,
  type ResourceAllocationRequest,
} from "@/types/resource-types";
import { beforeEach, describe, expect, it } from "@jest/globals";

describe("LoadBalancer", () => {
  let monitor: ResourceMonitor;
  let balancer: LoadBalancer;

  beforeEach(async () => {
    monitor = new ResourceMonitor();
    balancer = new LoadBalancer(monitor, LoadBalancingStrategy.LEAST_LOADED);

    // Register test agents with the monitor
    const cpu1Usage = {
      type: ResourceType.CPU,
      current: 20,
      maximum: 100,
      usagePercent: 20,
      unit: "%",
      timestamp: new Date(),
      source: "test",
    };

    const cpu2Usage = {
      type: ResourceType.CPU,
      current: 40,
      maximum: 100,
      usagePercent: 40,
      unit: "%",
      timestamp: new Date(),
      source: "test",
    };

    await monitor.recordUsage("agent-1", cpu1Usage);
    await monitor.recordUsage("agent-2", cpu2Usage);

    // Set task counts so agents are considered available
    await monitor.updateTaskCount("agent-1", 2);
    await monitor.updateTaskCount("agent-2", 5);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Agent Selection", () => {
    it("should select agent using least loaded strategy", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const decision = await balancer.selectAgent(request, [
        "agent-1",
        "agent-2",
      ]);

      expect(decision.selectedAgentId).toBeTruthy();
      expect(decision.strategy).toBe(LoadBalancingStrategy.LEAST_LOADED);
      expect(decision.decisionDurationMs).toBeGreaterThanOrEqual(0); // Can be 0 for very fast decisions
    });

    it("should select agent using round robin strategy", async () => {
      balancer.setStrategy(LoadBalancingStrategy.ROUND_ROBIN);

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const decision = await balancer.selectAgent(request, [
        "agent-1",
        "agent-2",
      ]);

      expect(decision.selectedAgentId).toBeTruthy();
      expect(decision.strategy).toBe(LoadBalancingStrategy.ROUND_ROBIN);
      expect(decision.decisionDurationMs).toBeGreaterThanOrEqual(0);
    });

    it("should select agent using weighted strategy", async () => {
      balancer.setStrategy(LoadBalancingStrategy.WEIGHTED);

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const decision = await balancer.selectAgent(request, [
        "agent-1",
        "agent-2",
      ]);

      expect(decision.selectedAgentId).toBeTruthy();
      expect(decision.strategy).toBe(LoadBalancingStrategy.WEIGHTED);
      expect(decision.decisionDurationMs).toBeGreaterThanOrEqual(0);
    });

    it("should select agent using priority based strategy", async () => {
      balancer.setStrategy(LoadBalancingStrategy.PRIORITY_BASED);

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.HIGH,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const decision = await balancer.selectAgent(request, [
        "agent-1",
        "agent-2",
      ]);

      expect(decision.selectedAgentId).toBeTruthy();
      expect(decision.strategy).toBe(LoadBalancingStrategy.PRIORITY_BASED);
      expect(decision.decisionDurationMs).toBeGreaterThanOrEqual(0);
    });

    it("should select agent using random strategy", async () => {
      balancer.setStrategy(LoadBalancingStrategy.RANDOM);

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const decision = await balancer.selectAgent(request, [
        "agent-1",
        "agent-2",
      ]);

      expect(decision.selectedAgentId).toBeTruthy();
      expect(decision.strategy).toBe(LoadBalancingStrategy.RANDOM);
      expect(decision.decisionDurationMs).toBeGreaterThanOrEqual(0);
    });

    it("should throw error for unknown strategy", async () => {
      // @ts-expect-error Testing invalid strategy
      balancer.setStrategy("INVALID_STRATEGY");

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      await expect(balancer.selectAgent(request, ["agent-1"])).rejects.toThrow(
        "Unknown load balancing strategy"
      );
    });

    it("should throw error for empty agent list", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      await expect(balancer.selectAgent(request, [])).rejects.toThrow();
    });
  });

  describe("Strategy Management", () => {
    it("should get current strategy", () => {
      const strategy = balancer.getStrategy();
      expect(strategy).toBe(LoadBalancingStrategy.LEAST_LOADED);
    });

    it("should update strategy", () => {
      balancer.setStrategy(LoadBalancingStrategy.ROUND_ROBIN);
      expect(balancer.getStrategy()).toBe(LoadBalancingStrategy.ROUND_ROBIN);
    });
  });

  describe("Load Distribution", () => {
    it("should track load distribution", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      await balancer.selectAgent(request, ["agent-1", "agent-2"]);

      const distribution = await balancer.getLoadDistribution();
      expect(distribution.size).toBeGreaterThan(0);
    });

    it("should reset load distribution", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      await balancer.selectAgent(request, ["agent-1"]);
      balancer.resetLoadDistribution();

      const distribution = await balancer.getLoadDistribution();
      expect(distribution.size).toBe(0);
    });
  });
});
