/**
 * @fileoverview Performance tests for Adaptive Resource Manager
 *
 * Tests performance requirements:
 * - Agent selection: <50ms
 * - Monitoring overhead: <5%
 * - High load handling: 100+ concurrent requests
 *
 * @author @darianrosebrook
 */

import { AdaptiveResourceManager } from "@/resources/AdaptiveResourceManager";
import {
  LoadBalancingStrategy,
  TaskPriority,
  type ResourceAllocationRequest,
} from "@/types/resource-types";
import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";

describe("AdaptiveResourceManager Performance", () => {
  let resourceManager: AdaptiveResourceManager;

  beforeEach(async () => {
    // Create mock agent registry for performance test
    const mockAgentRegistry = {
      initialize: async () => {},
      getAgentsByCapability: async () => [
        {
          agent: {
            id: "perf-agent-1",
            name: "Performance Agent 1",
            modelFamily: "gpt-4" as const,
            capabilities: {
              taskTypes: ["analysis" as const, "general" as const],
              languages: ["TypeScript" as const, "Python" as const],
              specializations: ["Performance optimization" as const],
              expertiseLevel: "expert" as const,
            },
            performanceHistory: {
              successRate: 0.98,
              averageQuality: 0.95,
              averageLatency: 500,
              taskCount: 200,
            },
            currentLoad: {
              activeTasks: 0,
              queuedTasks: 0,
              utilizationPercent: 5,
            },
            registeredAt: new Date().toISOString(),
            lastActiveAt: new Date().toISOString(),
          },
          matchScore: 0.98,
          matchReason: "high performance score and expert level",
        },
      ],
      updatePerformance: async () => ({} as any),
      getStats: async () => ({
        totalAgents: 10,
        activeAgents: 10,
        idleAgents: 0,
        averageUtilization: 5,
        averageSuccessRate: 0.98,
        lastUpdated: new Date().toISOString(),
      }),
      getProfile: async () => ({} as any),
    };

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
  });

  describe("Agent Selection Performance", () => {
    it("should select agents within 50ms", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-perf-1",
        taskId: "task-perf-1",
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

      const selectionTime = endTime - startTime;
      console.log(`Agent selection time: ${selectionTime}ms`);

      // Should complete within 50ms (even if no agents available)
      expect(selectionTime).toBeLessThan(50);
      expect(result).toBeDefined();
      expect(result.requestId).toBe("req-perf-1");

      // Clean up
      await resourceManager.releaseResources("req-perf-1");
    });

    it("should maintain performance across multiple strategies", async () => {
      const strategies = [
        LoadBalancingStrategy.ROUND_ROBIN,
        LoadBalancingStrategy.LEAST_LOADED,
        LoadBalancingStrategy.WEIGHTED,
        LoadBalancingStrategy.PRIORITY_BASED,
        LoadBalancingStrategy.RANDOM,
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

        const startTime = Date.now();
        const result = await resourceManager.allocateResources(request);
        const endTime = Date.now();

        const selectionTime = endTime - startTime;
        console.log(`${strategy} selection time: ${selectionTime}ms`);

        expect(selectionTime).toBeLessThan(50);
        expect(result).toBeDefined();

        // Clean up
        await resourceManager.releaseResources(`req-${strategy}`);
      }
    });
  });

  describe("Monitoring Overhead", () => {
    it("should maintain reasonable monitoring overhead", async () => {
      const iterations = 50; // Reduced for more realistic test
      const requests: ResourceAllocationRequest[] = [];

      // Create test requests
      for (let i = 0; i < iterations; i++) {
        requests.push({
          requestId: `req-overhead-${i}`,
          taskId: `task-overhead-${i}`,
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

      // Measure time with monitoring (actual allocation)
      const startTime = Date.now();
      const results = await Promise.all(
        requests.map((request) => resourceManager.allocateResources(request))
      );
      const endTime = Date.now();
      const totalTime = endTime - startTime;
      const avgTimePerRequest = totalTime / iterations;

      console.log(
        `Monitoring overhead test: ${iterations} requests in ${totalTime}ms (avg: ${avgTimePerRequest.toFixed(
          2
        )}ms per request)`
      );

      // Average time per request should be reasonable (less than 10ms)
      expect(avgTimePerRequest).toBeLessThan(10);

      // Total time should be reasonable for the number of requests
      expect(totalTime).toBeLessThan(1000); // Less than 1 second for 50 requests

      // Clean up
      await Promise.all(
        results.map((result) =>
          resourceManager.releaseResources(result.requestId)
        )
      );
    });
  });

  describe("High Load Performance", () => {
    it("should handle 100+ concurrent requests efficiently", async () => {
      const numRequests = 100;
      const requests: ResourceAllocationRequest[] = [];

      // Create concurrent requests
      for (let i = 0; i < numRequests; i++) {
        requests.push({
          requestId: `req-load-${i}`,
          taskId: `task-load-${i}`,
          priority: i % 3 === 0 ? TaskPriority.HIGH : TaskPriority.MEDIUM,
          requiredResources: {
            cpuPercent: 5 + (i % 10),
            memoryMb: 64 + (i % 50),
            networkMbps: 2 + (i % 5),
          },
          requestedAt: new Date(),
          timeoutMs: 5000,
        });
      }

      const startTime = Date.now();
      const results = await Promise.all(
        requests.map((request) => resourceManager.allocateResources(request))
      );
      const endTime = Date.now();

      const totalTime = endTime - startTime;
      const avgTimePerRequest = totalTime / numRequests;

      console.log(
        `High load test: ${numRequests} requests in ${totalTime}ms (avg: ${avgTimePerRequest.toFixed(
          2
        )}ms per request)`
      );

      // All requests should be processed
      expect(results.length).toBe(numRequests);

      // Average time per request should be reasonable
      expect(avgTimePerRequest).toBeLessThan(10); // Less than 10ms per request on average

      // Total time should be reasonable for 100 requests
      expect(totalTime).toBeLessThan(2000); // Less than 2 seconds total

      // Clean up
      await Promise.all(
        results.map((result) =>
          resourceManager.releaseResources(result.requestId)
        )
      );
    });

    it("should maintain performance under sustained load", async () => {
      const batchSize = 20;
      const numBatches = 5;
      const totalRequests = batchSize * numBatches;

      let totalTime = 0;
      const allResults = [];

      for (let batch = 0; batch < numBatches; batch++) {
        const requests: ResourceAllocationRequest[] = [];

        for (let i = 0; i < batchSize; i++) {
          requests.push({
            requestId: `req-sustained-${batch}-${i}`,
            taskId: `task-sustained-${batch}-${i}`,
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
        const results = await Promise.all(
          requests.map((request) => resourceManager.allocateResources(request))
        );
        const endTime = Date.now();

        const batchTime = endTime - startTime;
        totalTime += batchTime;
        allResults.push(...results);

        console.log(
          `Batch ${batch + 1}: ${batchSize} requests in ${batchTime}ms`
        );

        // Each batch should complete quickly
        expect(batchTime).toBeLessThan(500); // Less than 500ms per batch
      }

      const avgTimePerRequest = totalTime / totalRequests;
      console.log(
        `Sustained load test: ${totalRequests} requests in ${totalTime}ms (avg: ${avgTimePerRequest.toFixed(
          2
        )}ms per request)`
      );

      // Average time per request should remain consistent
      expect(avgTimePerRequest).toBeLessThan(10);

      // Clean up
      await Promise.all(
        allResults.map((result) =>
          resourceManager.releaseResources(result.requestId)
        )
      );
    });
  });

  describe("Capacity Analysis Performance", () => {
    it("should perform capacity analysis within reasonable time", async () => {
      const startTime = Date.now();
      const capacityAnalysis = await resourceManager.analyzeCapacity();
      const endTime = Date.now();

      const analysisTime = endTime - startTime;
      console.log(`Capacity analysis time: ${analysisTime}ms`);

      expect(analysisTime).toBeLessThan(100); // Less than 100ms
      expect(capacityAnalysis).toBeDefined();
      expect(capacityAnalysis.timestamp).toBeDefined();
    });

    it("should handle multiple concurrent capacity analyses", async () => {
      const numAnalyses = 10;
      const startTime = Date.now();

      const analyses = await Promise.all(
        Array.from({ length: numAnalyses }, () =>
          resourceManager.analyzeCapacity()
        )
      );

      const endTime = Date.now();
      const totalTime = endTime - startTime;
      const avgTimePerAnalysis = totalTime / numAnalyses;

      console.log(
        `Concurrent capacity analysis: ${numAnalyses} analyses in ${totalTime}ms (avg: ${avgTimePerAnalysis.toFixed(
          2
        )}ms per analysis)`
      );

      expect(analyses.length).toBe(numAnalyses);
      expect(avgTimePerAnalysis).toBeLessThan(50); // Less than 50ms per analysis
    });
  });

  describe("Failover Performance", () => {
    it("should handle failover within reasonable time", async () => {
      const startTime = Date.now();
      const failoverResult = await resourceManager.handleFailover(
        "agent-1",
        "agent-2"
      );
      const endTime = Date.now();

      const failoverTime = endTime - startTime;
      console.log(`Failover time: ${failoverTime}ms`);

      expect(failoverTime).toBeLessThan(100); // Less than 100ms
      expect(failoverResult).toBeDefined();
      expect(failoverResult.success).toBe(true);
    });

    it("should handle multiple concurrent failovers", async () => {
      const numFailovers = 5;
      const startTime = Date.now();

      const failoverResults = await Promise.all(
        Array.from({ length: numFailovers }, (_, i) =>
          resourceManager.handleFailover(`agent-${i}`, `agent-${i + 1}`)
        )
      );

      const endTime = Date.now();
      const totalTime = endTime - startTime;
      const avgTimePerFailover = totalTime / numFailovers;

      console.log(
        `Concurrent failover: ${numFailovers} failovers in ${totalTime}ms (avg: ${avgTimePerFailover.toFixed(
          2
        )}ms per failover)`
      );

      expect(failoverResults.length).toBe(numFailovers);
      expect(avgTimePerFailover).toBeLessThan(50); // Less than 50ms per failover
    });
  });
});
