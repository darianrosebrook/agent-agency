/**
 * @fileoverview Unit tests for AdaptiveResourceManager
 *
 * @author @darianrosebrook
 */

import { AdaptiveResourceManager } from "@/resources/AdaptiveResourceManager";
import type { AgentRegistry } from "@/types/agent-registry";
import {
  TaskPriority,
  type ResourceAllocationRequest,
} from "@/types/resource-types";
import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";

describe("AdaptiveResourceManager", () => {
  let manager: AdaptiveResourceManager;
  let mockAgentRegistry: jest.Mocked<AgentRegistry>;

  beforeEach(() => {
    mockAgentRegistry = {
      initialize: jest.fn().mockResolvedValue(undefined),
      getAgentsByCapability: jest.fn().mockResolvedValue([]),
      updatePerformance: jest.fn().mockResolvedValue({} as any),
      getStats: jest.fn().mockResolvedValue({
        totalAgents: 0,
        activeAgents: 0,
        averagePerformanceScore: 0,
        specializationDistribution: {},
      }),
      getProfile: jest.fn().mockResolvedValue({} as any),
    };

    manager = new AdaptiveResourceManager(mockAgentRegistry);
  });

  afterEach(async () => {
    await manager.stop();
  });

  describe("Initialization", () => {
    it("should initialize manager", async () => {
      await manager.initialize();

      const status = manager.getHealthStatus();
      expect(status).toBeDefined();
    });

    it("should start and stop manager", async () => {
      await manager.initialize();
      await manager.start();

      let status = manager.getHealthStatus();
      expect(status.isRunning).toBe(true);

      await manager.stop();

      status = manager.getHealthStatus();
      expect(status.isRunning).toBe(false);
    });
  });

  describe("Resource Allocation", () => {
    it("should allocate resources", async () => {
      await manager.initialize();

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {
          cpuPercent: 20,
          memoryMb: 256,
        },
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      const result = await manager.allocateResources(request);

      expect(result.requestId).toBe("req-1");
      expect(result.success).toBeDefined();
    });

    it("should release resources", async () => {
      await manager.initialize();

      const request: ResourceAllocationRequest = {
        requestId: "req-1",
        taskId: "task-1",
        priority: TaskPriority.MEDIUM,
        requiredResources: {},
        requestedAt: new Date(),
        timeoutMs: 5000,
      };

      await manager.allocateResources(request);
      await manager.releaseResources("req-1");

      // Should not throw
    });
  });

  describe("Capacity Analysis", () => {
    it("should analyze capacity", async () => {
      await manager.initialize();

      const analysis = await manager.analyzeCapacity();

      expect(analysis.timestamp).toBeInstanceOf(Date);
      expect(analysis.utilizationPercent).toBeGreaterThanOrEqual(0);
      expect(analysis.scalingRecommendation).toBeDefined();
    });
  });

  describe("Statistics", () => {
    it("should get pool statistics", async () => {
      await manager.initialize();

      const stats = await manager.getPoolStatistics();

      expect(stats.totalAgents).toBeGreaterThanOrEqual(0);
    });

    it("should get allocation statistics", async () => {
      await manager.initialize();

      const stats = manager.getAllocationStatistics();

      expect(stats.totalRequests).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Configuration", () => {
    it("should get configuration", () => {
      const config = manager.getConfig();

      expect(config.enabled).toBeDefined();
      expect(config.loadBalancingStrategy).toBeDefined();
    });

    it("should update configuration", () => {
      manager.updateConfig({
        monitoringIntervalMs: 10000,
      });

      const config = manager.getConfig();
      expect(config.monitoringIntervalMs).toBe(10000);
    });
  });

  describe("Failover", () => {
    it("should handle failover", async () => {
      await manager.initialize();

      const event = await manager.handleFailover("agent-1", "agent-2");

      expect(event.eventId).toBeTruthy();
      expect(event.failedAgentId).toBe("agent-1");
      expect(event.backupAgentId).toBe("agent-2");
    });

    it("should track failover history", async () => {
      await manager.initialize();

      await manager.handleFailover("agent-1", "agent-2");

      const history = manager.getFailoverHistory(10);
      expect(history.length).toBeGreaterThanOrEqual(1);
    });
  });
});
