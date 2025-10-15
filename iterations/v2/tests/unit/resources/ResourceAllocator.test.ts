/**
 * @fileoverview Unit tests for ResourceAllocator
 *
 * @author @darianrosebrook
 */

import { LoadBalancer } from "@/resources/LoadBalancer";
import { ResourceAllocator } from "@/resources/ResourceAllocator";
import { ResourceMonitor } from "@/resources/ResourceMonitor";
import {
  LoadBalancingStrategy,
  ResourceType,
  TaskPriority,
  type ResourceAllocationRequest,
} from "@/types/resource-types";
import { beforeEach, describe, expect, it, jest } from "@jest/globals";

describe("ResourceAllocator", () => {
  let monitor: ResourceMonitor;
  let loadBalancer: LoadBalancer;
  let allocator: ResourceAllocator;

  beforeEach(async () => {
    monitor = new ResourceMonitor();
    loadBalancer = new LoadBalancer(
      monitor,
      LoadBalancingStrategy.LEAST_LOADED
    );
    allocator = new ResourceAllocator(loadBalancer);

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

  describe("Resource Allocation", () => {
    it("should successfully allocate resources", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const result = await allocator.allocate(request);

      expect(result.success).toBe(true);
      expect(result.assignedAgentId).toBeTruthy();
      expect(result.requestId).toBe("req-1");
      expect(result.waitTimeMs).toBeGreaterThanOrEqual(0);
    });

    it("should handle rate limit exceeded", async () => {
      // Mock rate limit check to return false
      jest.spyOn(allocator as any, "checkRateLimit").mockReturnValue(false);

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const result = await allocator.allocate(request);

      expect(result.success).toBe(false);
      expect(result.failureReason).toBe("Rate limit exceeded");
      expect(result.requestId).toBe("req-1");
    });

    it("should handle request timeout", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(Date.now() - 10000), // 10 seconds ago
        timeoutMs: 1000, // 1 second timeout (already expired)
      };

      const result = await allocator.allocate(request);

      expect(result.success).toBe(false);
      expect(result.failureReason).toBe("Request timeout");
      expect(result.requestId).toBe("req-1");
    });

    it("should handle no available agents", async () => {
      // Mock getAvailableAgents to return empty array
      jest.spyOn(allocator as any, "getAvailableAgents").mockResolvedValue([]);

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const result = await allocator.allocate(request);

      expect(result.success).toBe(false);
      expect(result.failureReason).toBe("No available agents");
      expect(result.requestId).toBe("req-1");
    });

    it("should handle load balancer selection failure", async () => {
      // Mock load balancer to throw error
      jest
        .spyOn(loadBalancer, "selectAgent")
        .mockRejectedValue(new Error("Selection failed"));

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const result = await allocator.allocate(request);

      expect(result.success).toBe(false);
      expect(result.failureReason).toBe("Selection failed");
      expect(result.requestId).toBe("req-1");
    });
  });

  describe("Resource Release", () => {
    it("should successfully release resources", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      // First allocate
      const allocationResult = await allocator.allocate(request);
      expect(allocationResult.success).toBe(true);

      // Then release
      await allocator.release("req-1");
      // Release method returns void, so we just verify it doesn't throw
    });

    it("should handle release of unknown allocation", async () => {
      // Release method returns void, so we just verify it doesn't throw
      await expect(allocator.release("unknown-req")).resolves.not.toThrow();
    });
  });

  describe("Statistics", () => {
    it("should track allocation statistics", async () => {
      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      await allocator.allocate(request);

      const stats = allocator.getAllocationStats();
      expect(stats.totalRequests).toBe(1);
      expect(stats.successfulAllocations).toBe(1);
      expect(stats.failedAllocations).toBe(0);
    });

    it("should track failed allocation statistics", async () => {
      // Create allocator with very low rate limit to force failure
      const lowRateAllocator = new ResourceAllocator(loadBalancer, {
        maxRequests: 0, // No requests allowed
        windowMs: 1000,
      });

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      await lowRateAllocator.allocate(request);

      const stats = lowRateAllocator.getAllocationStats();
      expect(stats.totalRequests).toBe(1);
      expect(stats.successfulAllocations).toBe(0);
      expect(stats.failedAllocations).toBe(1);
    });
  });

  describe("Configuration", () => {
    it("should initialize with default configuration", () => {
      const stats = allocator.getAllocationStats();
      expect(stats).toBeDefined();
      expect(stats.totalRequests).toBe(0);
      expect(stats.successfulAllocations).toBe(0);
      expect(stats.failedAllocations).toBe(0);
    });
  });

  describe("Rate Limiting", () => {
    it("should respect rate limits", async () => {
      // Create allocator with low rate limit
      const lowRateAllocator = new ResourceAllocator(loadBalancer, {
        maxRequests: 1,
        windowMs: 1000,
      });

      const request1: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const request2: ResourceAllocationRequest = {
        requestId: "req-2",
        taskId: "task-2",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      // First request should succeed
      const result1 = await lowRateAllocator.allocate(request1);
      expect(result1.success).toBe(true);

      // Second request should fail due to rate limit
      const result2 = await lowRateAllocator.allocate(request2);
      expect(result2.success).toBe(false);
      expect(result2.failureReason).toBe("Rate limit exceeded");
    });
  });
});
