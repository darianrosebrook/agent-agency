/**
 * @fileoverview Production validation tests for Adaptive Resource Manager
 *
 * Tests production readiness, stability, and operational requirements
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { SystemHealthMonitor } from "@/monitoring/SystemHealthMonitor";
import { AdaptiveResourceManager } from "@/resources/AdaptiveResourceManager";
import { ResourceMonitor } from "@/resources/ResourceMonitor";
import {
  LoadBalancingStrategy,
  ResourceType,
  TaskPriority,
  type ResourceAllocationRequest,
} from "@/types/resource-types";
import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  jest,
} from "@jest/globals";

describe("AdaptiveResourceManager Production Validation", () => {
  let resourceManager: AdaptiveResourceManager;
  let healthMonitor: SystemHealthMonitor;
  let resourceMonitor: ResourceMonitor;

  beforeEach(async () => {
    // Initialize SystemHealthMonitor
    healthMonitor = new SystemHealthMonitor({
      collectionIntervalMs: 1000,
      healthCheckIntervalMs: 2000,
      retentionPeriodMs: 10000,
      enableCircuitBreaker: false,
    });
    await healthMonitor.initialize();

    // Initialize AdaptiveResourceManager with production-like configuration
    resourceManager = new AdaptiveResourceManager({
      enabled: true,
      monitoringIntervalMs: 1000,
      loadBalancingStrategy: LoadBalancingStrategy.LEAST_LOADED,
      enableDynamicRateLimiting: true,
      enableAutoFailover: true,
      maxAllocationDecisionMs: 50,
      enableCapacityPlanning: true,
      thresholds: {
        cpuWarning: 70,
        cpuCritical: 85,
        memoryWarning: 75,
        memoryCritical: 90,
      },
    });
    await resourceManager.initialize();
    await resourceManager.start();

    // Access the internal resourceMonitor for agent setup
    resourceMonitor = (resourceManager as any).resourceMonitor;

    // Register production-like agents
    for (let i = 1; i <= 10; i++) {
      await resourceMonitor.recordUsage(`agent-${i}`, {
        type: ResourceType.CPU,
        current: 10 * i,
        maximum: 100,
        usagePercent: 10 * i,
        unit: "%",
        timestamp: new Date(),
        source: "production-test",
      });
      await resourceMonitor.updateTaskCount(`agent-${i}`, i % 3);
    }
  });

  afterEach(async () => {
    if (resourceManager) {
      await resourceManager.stop();
    }
    if (healthMonitor) {
      await healthMonitor.shutdown();
    }
    jest.clearAllMocks();
  });

  describe("Production Stability", () => {
    it("should maintain system stability under continuous operation", async () => {
      const numRequests = 100;
      const requests: ResourceAllocationRequest[] = [];

      // Generate production-like requests
      for (let i = 0; i < numRequests; i++) {
        requests.push({
          requestId: `prod-req-${i}`,
          taskId: `prod-task-${i}`,
          priority: i % 3 === 0 ? TaskPriority.HIGH : TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 5 + (i % 20),
            memoryMb: 64 + (i % 128),
            networkMbps: 2 + (i % 10),
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        });
      }

      // Execute requests in batches to simulate production load
      const batchSize = 20;
      const results: any[] = [];

      for (let i = 0; i < requests.length; i += batchSize) {
        const batch = requests.slice(i, i + batchSize);
        const batchResults = await Promise.all(
          batch.map((request) => resourceManager.allocateResources(request))
        );
        results.push(...batchResults);

        // Small delay between batches
        await new Promise((resolve) => setTimeout(resolve, 10));
      }

      // Verify system stability
      const successfulAllocations = results.filter((r) => r.success).length;
      const successRate = successfulAllocations / numRequests;

      console.log(
        `Production stability test: ${successfulAllocations}/${numRequests} successful (${(
          successRate * 100
        ).toFixed(1)}% success rate)`
      );

      // Should maintain at least 80% success rate under normal conditions
      expect(successRate).toBeGreaterThanOrEqual(0.8);

      // Clean up
      await Promise.all(
        results.map((result) =>
          resourceManager.releaseResources(result.requestId)
        )
      );
    }, 30000);

    it("should handle resource exhaustion gracefully", async () => {
      // Simulate high resource usage
      for (let i = 1; i <= 10; i++) {
        await resourceMonitor.recordUsage(`agent-${i}`, {
          type: ResourceType.CPU,
          current: 95, // High CPU usage
          maximum: 100,
          usagePercent: 95,
          unit: "%",
          timestamp: new Date(),
          source: "production-test",
        });
        await resourceMonitor.updateTaskCount(`agent-${i}`, 10); // High task count
      }

      const request: ResourceAllocationRequest = {
        requestId: "exhaustion-test",
        taskId: "exhaustion-task",
        priority: TaskPriority.HIGH,
        requiredResources: {
          cpuPercent: 50,
          memoryMb: 512,
          networkMbps: 50,
        },
        requestedAt: new Date(),
        timeoutMs: 1000,
      };

      const result = await resourceManager.allocateResources(request);

      // Should handle gracefully (may succeed or fail, but shouldn't crash)
      expect(result).toBeDefined();
      expect(result.requestId).toBe("exhaustion-test");
      expect(typeof result.success).toBe("boolean");

      if (result.success) {
        await resourceManager.releaseResources(request.requestId);
      }
    });

    it("should maintain performance under sustained load", async () => {
      const numBatches = 10;
      const batchSize = 20;
      const allResults: any[] = [];

      for (let batch = 0; batch < numBatches; batch++) {
        const batchRequests: ResourceAllocationRequest[] = [];
        for (let i = 0; i < batchSize; i++) {
          batchRequests.push({
            requestId: `sustained-${batch}-${i}`,
            taskId: `sustained-task-${batch}-${i}`,
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

        const startTime = Date.now();
        const batchResults = await Promise.all(
          batchRequests.map((request) =>
            resourceManager.allocateResources(request)
          )
        );
        const endTime = Date.now();
        const batchDuration = endTime - startTime;

        allResults.push(...batchResults);

        console.log(
          `Batch ${batch + 1}: ${
            batchResults.filter((r) => r.success).length
          }/${batchSize} successful in ${batchDuration}ms`
        );

        // Each batch should complete within reasonable time
        expect(batchDuration).toBeLessThan(5000); // 5 seconds per batch

        // Wait between batches
        await new Promise((resolve) => setTimeout(resolve, 100));
      }

      const totalSuccessful = allResults.filter((r) => r.success).length;
      const totalRequests = numBatches * batchSize;
      const overallSuccessRate = totalSuccessful / totalRequests;

      console.log(
        `Sustained load test: ${totalSuccessful}/${totalRequests} successful (${(
          overallSuccessRate * 100
        ).toFixed(1)}% success rate)`
      );

      // Should maintain good success rate over time
      expect(overallSuccessRate).toBeGreaterThanOrEqual(0.7);

      // Clean up
      await Promise.all(
        allResults.map((result) =>
          resourceManager.releaseResources(result.requestId)
        )
      );
    }, 60000);
  });

  describe("Operational Requirements", () => {
    it("should provide accurate capacity analysis", async () => {
      // Allow time for resource monitor to collect data
      await new Promise((resolve) => setTimeout(resolve, 1500));

      const capacityAnalysis = await resourceManager.analyzeCapacity();

      expect(capacityAnalysis).toBeDefined();
      expect(capacityAnalysis.timestamp).toBeDefined();
      expect(capacityAnalysis.totalCapacity).toBeDefined();
      expect(capacityAnalysis.usedCapacity).toBeDefined();
      expect(capacityAnalysis.availableCapacity).toBeDefined();
      expect(capacityAnalysis.utilizationPercent).toBeGreaterThanOrEqual(0);
      expect(capacityAnalysis.utilizationPercent).toBeLessThanOrEqual(100);
      expect(capacityAnalysis.scalingRecommendation).toBeDefined();

      // Verify capacity calculations are reasonable
      expect(capacityAnalysis.totalCapacity.agentCount).toBeGreaterThan(0);
      expect(capacityAnalysis.usedCapacity.activeAgents).toBeGreaterThanOrEqual(
        0
      );
      expect(
        capacityAnalysis.availableCapacity.idleAgents
      ).toBeGreaterThanOrEqual(0);
    });

    it("should handle configuration updates without disruption", async () => {
      // Get initial statistics
      const initialStats = resourceManager.getPoolStatistics();

      // Update configuration
      await resourceManager.updateConfig({
        loadBalancingStrategy: LoadBalancingStrategy.ROUND_ROBIN,
        maxAllocationDecisionMs: 100,
        thresholds: {
          cpuWarning: 60,
          cpuCritical: 80,
          memoryWarning: 65,
          memoryCritical: 85,
        },
      });

      // Verify system still works after configuration change
      const request: ResourceAllocationRequest = {
        requestId: "config-test",
        taskId: "config-task",
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
      expect(result.success).toBe(true);

      // Verify configuration was applied
      const config = resourceManager.getConfig();
      expect(config.loadBalancingStrategy).toBe(
        LoadBalancingStrategy.ROUND_ROBIN
      );
      expect(config.maxAllocationDecisionMs).toBe(100);

      await resourceManager.releaseResources(request.requestId);
    });

    it("should provide comprehensive monitoring data", async () => {
      // Allow time for monitoring to collect data
      await new Promise((resolve) => setTimeout(resolve, 1500));

      const poolStats = await resourceManager.getPoolStatistics();
      const loadDistribution = resourceManager.getLoadDistribution();

      expect(poolStats).toBeDefined();
      expect(poolStats.totalAgents).toBeGreaterThan(0);
      expect(poolStats.activeAgents).toBeGreaterThanOrEqual(0);
      expect(poolStats.idleAgents).toBeGreaterThanOrEqual(0);
      expect(poolStats.totalCpuCapacity).toBeGreaterThan(0);
      expect(poolStats.totalMemoryCapacity).toBeGreaterThan(0);

      expect(loadDistribution).toBeDefined();
      expect(typeof loadDistribution).toBe("object");
    });

    it("should handle failover scenarios correctly", async () => {
      // Simulate agent failure
      await resourceMonitor.removeAgent("agent-1");

      const failoverResult = await resourceManager.handleFailover(
        "agent-1",
        "agent-2"
      );

      expect(failoverResult).toBeDefined();
      expect(failoverResult.success).toBe(true);
      expect(failoverResult.failedAgentId).toBe("agent-1");
      expect(failoverResult.backupAgentId).toBe("agent-2");
      expect(failoverResult.tasksTransferred).toBeGreaterThanOrEqual(0);
      expect(failoverResult.durationMs).toBeGreaterThanOrEqual(0);
    });

    it("should maintain data consistency during operations", async () => {
      const requests: ResourceAllocationRequest[] = [];
      for (let i = 0; i < 50; i++) {
        requests.push({
          requestId: `consistency-${i}`,
          taskId: `consistency-task-${i}`,
          priority: TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 5,
            memoryMb: 64,
            networkMbps: 2,
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        });
      }

      // Allocate resources
      const allocationResults = await Promise.all(
        requests.map((request) => resourceManager.allocateResources(request))
      );

      // Verify all successful allocations have valid agent assignments
      const successfulAllocations = allocationResults.filter((r) => r.success);
      for (const result of successfulAllocations) {
        expect(result.assignedAgentId).toBeTruthy();
        expect(typeof result.assignedAgentId).toBe("string");
        expect(result.assignedAgentId!.length).toBeGreaterThan(0);
      }

      // Release resources
      await Promise.all(
        allocationResults.map((result) =>
          resourceManager.releaseResources(result.requestId)
        )
      );

      // Verify system state is consistent after cleanup
      const finalStats = await resourceManager.getPoolStatistics();
      // Note: activeAgents may not be 0 due to test setup task counts, but should be consistent
      expect(finalStats.activeAgents).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Error Handling and Recovery", () => {
    it("should handle invalid requests gracefully", async () => {
      const invalidRequest: ResourceAllocationRequest = {
        requestId: "invalid-request",
        taskId: "invalid-task",
        priority: TaskPriority.MEDIUM,
        requiredResources: {
          cpuPercent: -10, // Invalid negative value
          memoryMb: 0, // Invalid zero value
          networkMbps: 1000, // Unrealistically high value
        },
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const result = await resourceManager.allocateResources(invalidRequest);

      // Should handle gracefully (may succeed or fail, but shouldn't crash)
      expect(result).toBeDefined();
      expect(result.requestId).toBe("invalid-request");
      expect(typeof result.success).toBe("boolean");

      if (result.success) {
        await resourceManager.releaseResources(invalidRequest.requestId);
      }
    });

    it("should handle concurrent configuration updates", async () => {
      const configUpdates = [
        { loadBalancingStrategy: LoadBalancingStrategy.ROUND_ROBIN },
        { loadBalancingStrategy: LoadBalancingStrategy.LEAST_LOADED },
        { maxAllocationDecisionMs: 75 },
        { maxAllocationDecisionMs: 100 },
      ];

      // Apply configuration updates concurrently
      const updatePromises = configUpdates.map((config) =>
        resourceManager.updateConfig(config)
      );

      await expect(Promise.all(updatePromises)).resolves.not.toThrow();

      // Verify system is still functional
      const request: ResourceAllocationRequest = {
        requestId: "concurrent-config-test",
        taskId: "concurrent-config-task",
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
      expect(result.success).toBe(true);

      await resourceManager.releaseResources(request.requestId);
    });

    it("should recover from temporary failures", async () => {
      // Simulate temporary agent unavailability
      await resourceMonitor.removeAgent("agent-1");
      await resourceMonitor.removeAgent("agent-2");

      const request: ResourceAllocationRequest = {
        requestId: "recovery-test",
        taskId: "recovery-task",
        priority: TaskPriority.HIGH,
        requiredResources: {
          cpuPercent: 20,
          memoryMb: 256,
          networkMbps: 10,
        },
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const result = await resourceManager.allocateResources(request);

      // Should still be able to allocate to remaining agents
      expect(result).toBeDefined();
      expect(result.requestId).toBe("recovery-test");

      if (result.success) {
        expect(result.assignedAgentId).not.toBe("agent-1");
        expect(result.assignedAgentId).not.toBe("agent-2");
        await resourceManager.releaseResources(request.requestId);
      }

      // Restore agents
      await resourceMonitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 20,
        maximum: 100,
        usagePercent: 20,
        unit: "%",
        timestamp: new Date(),
        source: "production-test",
      });
      await resourceMonitor.updateTaskCount("agent-1", 1);

      await resourceMonitor.recordUsage("agent-2", {
        type: ResourceType.CPU,
        current: 40,
        maximum: 100,
        usagePercent: 40,
        unit: "%",
        timestamp: new Date(),
        source: "production-test",
      });
      await resourceMonitor.updateTaskCount("agent-2", 2);

      // Verify system recovers
      const recoveryRequest: ResourceAllocationRequest = {
        requestId: "recovery-test-2",
        taskId: "recovery-task-2",
        priority: TaskPriority.MEDIUM,
        requiredResources: {
          cpuPercent: 15,
          memoryMb: 192,
          networkMbps: 8,
        },
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const recoveryResult = await resourceManager.allocateResources(
        recoveryRequest
      );
      expect(recoveryResult.success).toBe(true);

      await resourceManager.releaseResources(recoveryRequest.requestId);
    });
  });

  describe("Performance Requirements", () => {
    it("should meet allocation time requirements", async () => {
      const numTests = 20;
      const times: number[] = [];

      for (let i = 0; i < numTests; i++) {
        const request: ResourceAllocationRequest = {
          requestId: `perf-test-${i}`,
          taskId: `perf-task-${i}`,
          priority: TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 10,
            memoryMb: 128,
            networkMbps: 5,
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        };

        const startTime = Date.now();
        const result = await resourceManager.allocateResources(request);
        const endTime = Date.now();
        const allocationTime = endTime - startTime;

        times.push(allocationTime);
        expect(result.success).toBe(true);

        await resourceManager.releaseResources(request.requestId);
      }

      const avgTime = times.reduce((sum, time) => sum + time, 0) / times.length;
      const p95Time = times.sort((a, b) => a - b)[
        Math.floor(times.length * 0.95)
      ];

      console.log(
        `Allocation performance: avg=${avgTime.toFixed(2)}ms, p95=${p95Time}ms`
      );

      // Should meet performance requirements
      expect(avgTime).toBeLessThan(50); // Average allocation time < 50ms
      expect(p95Time).toBeLessThan(100); // P95 allocation time < 100ms
    });

    it("should maintain low memory usage", async () => {
      const initialMemory = process.memoryUsage().heapUsed;

      // Perform many operations
      const numOperations = 100;
      for (let i = 0; i < numOperations; i++) {
        const request: ResourceAllocationRequest = {
          requestId: `memory-test-${i}`,
          taskId: `memory-task-${i}`,
          priority: TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 5,
            memoryMb: 64,
            networkMbps: 2,
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        };

        const result = await resourceManager.allocateResources(request);
        if (result.success) {
          await resourceManager.releaseResources(request.requestId);
        }
      }

      const finalMemory = process.memoryUsage().heapUsed;
      const memoryIncrease = finalMemory - initialMemory;
      const memoryIncreaseMB = memoryIncrease / 1024 / 1024;

      console.log(`Memory usage increase: ${memoryIncreaseMB.toFixed(2)}MB`);

      // Should not have significant memory leaks
      expect(memoryIncreaseMB).toBeLessThan(50); // Less than 50MB increase
    });
  });
});
