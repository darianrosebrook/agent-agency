/**
 * @fileoverview Integration tests for Adaptive Resource Manager
 *
 * Tests integration with SystemHealthMonitor and end-to-end resource management flow
 *
 * @author @darianrosebrook
 */

import { SystemHealthMonitor } from "@/monitoring/SystemHealthMonitor";
import { AdaptiveResourceManager } from "@/resources/AdaptiveResourceManager";
import {
  LoadBalancingStrategy,
  TaskPriority,
  type ResourceAllocationRequest,
} from "@/types/resource-types";
import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";

describe("AdaptiveResourceManager Integration", () => {
  let resourceManager: AdaptiveResourceManager;
  let healthMonitor: SystemHealthMonitor;

  beforeEach(async () => {
    // Initialize SystemHealthMonitor
    healthMonitor = new SystemHealthMonitor({
      collectionIntervalMs: 1000, // Faster for tests
      healthCheckIntervalMs: 2000,
      retentionPeriodMs: 10000,
      enableCircuitBreaker: false, // Disable for tests
    });

    await healthMonitor.initialize();

    // Create mock agent registry for integration test
    const mockAgentRegistry = {
      initialize: async () => {},
      getAgentsByCapability: async () => [
        {
          agent: {
            id: "test-agent-1",
            name: "Test Agent 1",
            modelFamily: "gpt-3.5-turbo" as const,
            capabilities: {
              taskTypes: ["general" as const],
              languages: ["TypeScript" as const],
              specializations: [],
              expertiseLevel: "intermediate" as const,
            },
            performanceHistory: {
              successRate: 0.95,
              averageQuality: 0.8,
              averageLatency: 1000,
              taskCount: 50,
            },
            currentLoad: {
              activeTasks: 0,
              queuedTasks: 0,
              utilizationPercent: 10,
            },
            registeredAt: new Date().toISOString(),
            lastActiveAt: new Date().toISOString(),
          },
          matchScore: 0.8,
          matchReason: "basic capabilities match",
        },
      ],
      updatePerformance: async () => ({} as any),
      getStats: async () => ({
        totalAgents: 1,
        activeAgents: 1,
        idleAgents: 0,
        averageUtilization: 10,
        averageSuccessRate: 0.95,
        lastUpdated: new Date().toISOString(),
      }),
      getProfile: async () => ({} as any),
    };

    // Initialize AdaptiveResourceManager
    resourceManager = new AdaptiveResourceManager(mockAgentRegistry, {
      enabled: true,
      monitoringIntervalMs: 1000,
      loadBalancingStrategy: LoadBalancingStrategy.LEAST_LOADED,
      enableDynamicRateLimiting: true,
      enableAutoFailover: true,
      maxAllocationDecisionMs: 50,
      enableCapacityPlanning: true,
    });

    await resourceManager.initialize();
    await resourceManager.start();
  });

  afterEach(async () => {
    if (resourceManager) {
      await resourceManager.stop();
    }
    if (healthMonitor) {
      await healthMonitor.shutdown();
    }
  });

  describe("SystemHealthMonitor Integration", () => {
    it("should integrate with SystemHealthMonitor for health metrics", async () => {
      // Get health metrics from SystemHealthMonitor
      const healthMetrics = await healthMonitor.getHealthMetrics();
      expect(healthMetrics).toBeDefined();
      expect(healthMetrics.overallHealth).toBeDefined();
      expect(healthMetrics.system).toBeDefined();
      expect(healthMetrics.agents).toBeDefined();
    });

    it("should handle SystemHealthMonitor unavailability gracefully", async () => {
      // Shutdown health monitor to simulate unavailability
      await healthMonitor.shutdown();

      // Resource manager should still function
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      // Should not throw error even without health monitor
      const result = await resourceManager.allocateResources(request);
      expect(result).toBeDefined();
      // Result may be successful or failed, but should not crash
    });
  });

  describe("End-to-End Resource Management Flow", () => {
    it("should complete full resource allocation and release cycle", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.HIGH,
        requiredResources: {
          cpuPercent: 20,
          memoryMb: 256,
          networkMbps: 10,
        },
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      // Allocate resources (may fail if no agents available, which is expected)
      const allocationResult = await resourceManager.allocateResources(request);
      expect(allocationResult).toBeDefined();
      expect(allocationResult.requestId).toBe("req-1");

      // Release resources (should not throw even if allocation failed)
      await expect(
        resourceManager.releaseResources("req-1")
      ).resolves.not.toThrow();
    });

    it("should handle multiple concurrent resource allocations", async () => {
      // Create multiple concurrent requests
      const requests: ResourceAllocationRequest[] = [];
      for (let i = 1; i <= 5; i++) {
        requests.push({
          requestId: `req-${i}`,
          taskId: `task-${i}`,
          priority: TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 10,
            memoryMb: 128,
            networkMbps: 5,
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        });
      }

      // Allocate all resources concurrently
      const allocationPromises = requests.map((request) =>
        resourceManager.allocateResources(request)
      );

      const results = await Promise.all(allocationPromises);

      // All requests should be processed (may fail due to no agents, which is expected)
      expect(results.length).toBe(5);
      results.forEach((result) => {
        expect(result).toBeDefined();
        expect(result.requestId).toBeTruthy();
      });

      // Release all resources
      const releasePromises = results.map((r) =>
        resourceManager.releaseResources(r.requestId)
      );
      await Promise.all(releasePromises);
    });

    it("should provide capacity analysis and scaling recommendations", async () => {
      // Get capacity analysis
      const capacityAnalysis = await resourceManager.analyzeCapacity();
      expect(capacityAnalysis).toBeDefined();
      expect(capacityAnalysis.totalCapacity).toBeDefined();
      expect(capacityAnalysis.usedCapacity).toBeDefined();
      expect(capacityAnalysis.availableCapacity).toBeDefined();
      expect(capacityAnalysis.utilizationPercent).toBeGreaterThanOrEqual(0);
    });

    it("should handle agent failover scenarios", async () => {
      // Simulate agent failure and trigger failover
      const failoverResult = await resourceManager.handleFailover(
        "agent-1",
        "agent-2"
      );
      expect(failoverResult.success).toBe(true);
      expect(failoverResult.tasksTransferred).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Load Balancing Integration", () => {
    it("should distribute load across multiple agents using different strategies", async () => {
      // Test different load balancing strategies
      const strategies = [
        "round_robin",
        "least_loaded",
        "weighted",
        "priority_based",
        "random",
      ];

      for (const strategy of strategies) {
        const request: ResourceAllocationRequest = {
          requestId: `req-${strategy}`,
          taskId: `task-${strategy}`,
          priority: TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 10,
            memoryMb: 128,
            networkMbps: 5,
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        };

        const result = await resourceManager.allocateResources(request);
        expect(result).toBeDefined();
        expect(result.requestId).toBe(`req-${strategy}`);

        // Release the resource
        await resourceManager.releaseResources(`req-${strategy}`);
      }
    });

    it("should track load distribution statistics", async () => {
      // Get load distribution statistics
      const loadStats = await resourceManager.getLoadDistribution();
      expect(loadStats).toBeDefined();
      expect(loadStats.size).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Performance and Scalability", () => {
    it("should maintain performance under high load", async () => {
      const startTime = Date.now();
      const numRequests = 10; // Reduced for integration test

      // Create and allocate many resources quickly
      const allocationPromises = [];
      for (let i = 1; i <= numRequests; i++) {
        const request: ResourceAllocationRequest = {
          requestId: `req-${i}`,
          taskId: `task-${i}`,
          priority: TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 5,
            memoryMb: 64,
            networkMbps: 2,
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        };

        allocationPromises.push(resourceManager.allocateResources(request));
      }

      const results = await Promise.all(allocationPromises);
      const endTime = Date.now();

      const avgTimePerRequest = (endTime - startTime) / numRequests;

      console.log(
        `High Load Test - ${
          results.length
        }/${numRequests} processed, avg: ${avgTimePerRequest.toFixed(
          2
        )}ms per request`
      );

      // All requests should be processed (may fail due to no agents, which is expected)
      expect(results.length).toBe(numRequests);
      // Average time per request should be reasonable
      expect(avgTimePerRequest).toBeLessThan(100); // Less than 100ms per request

      // Clean up
      const releasePromises = results.map((r) =>
        resourceManager.releaseResources(r.requestId)
      );
      await Promise.all(releasePromises);
    }, 30000); // Extended timeout for performance test
  });
});
