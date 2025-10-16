/**
 * @fileoverview Unit tests for ResourceMonitor
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { ResourceMonitor } from "@/resources/ResourceMonitor";
import { ResourceType, type ResourceUsage } from "@/types/resource-types";
import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";

describe("ResourceMonitor", () => {
  let monitor: ResourceMonitor;

  beforeEach(() => {
    monitor = new ResourceMonitor();
  });

  afterEach(async () => {
    await monitor.stop();
  });

  describe("Resource Tracking", () => {
    it("should record resource usage", async () => {
      const usage: ResourceUsage = {
        type: ResourceType.CPU,
        current: 50,
        maximum: 100,
        usagePercent: 50,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      };

      await monitor.recordUsage("agent-1", usage);

      const profile = await monitor.getAgentResources("agent-1");
      expect(profile).not.toBeNull();
      expect(profile?.cpuUsage.usagePercent).toBe(50);
    });

    it("should track multiple resource types", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 50,
        maximum: 100,
        usagePercent: 50,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      await monitor.recordUsage("agent-1", {
        type: ResourceType.MEMORY,
        current: 512,
        maximum: 1024,
        usagePercent: 50,
        unit: "MB",
        timestamp: new Date(),
        source: "agent-1",
      });

      const profile = await monitor.getAgentResources("agent-1");
      expect(profile?.cpuUsage.usagePercent).toBe(50);
      expect(profile?.memoryUsage.usagePercent).toBe(50);
    });

    it("should update task count", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 50,
        maximum: 100,
        usagePercent: 50,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      await monitor.updateTaskCount("agent-1", 5);

      const profile = await monitor.getAgentResources("agent-1");
      expect(profile?.currentTaskCount).toBe(5);
    });
  });

  describe("Health Status", () => {
    it("should compute healthy status", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 50,
        maximum: 100,
        usagePercent: 50,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      const profile = await monitor.getAgentResources("agent-1");
      expect(profile?.healthStatus).toBe("healthy");
    });

    it("should detect degraded status", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 75,
        maximum: 100,
        usagePercent: 75,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      const profile = await monitor.getAgentResources("agent-1");
      expect(profile?.healthStatus).toBe("degraded");
    });

    it("should detect unhealthy status", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 90,
        maximum: 100,
        usagePercent: 90,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      const profile = await monitor.getAgentResources("agent-1");
      expect(profile?.healthStatus).toBe("unhealthy");
    });
  });

  describe("Pool Statistics", () => {
    it("should get pool statistics", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 50,
        maximum: 100,
        usagePercent: 50,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      await monitor.updateTaskCount("agent-1", 3);

      const stats = await monitor.getPoolStats();
      expect(stats.totalAgents).toBe(1);
      expect(stats.activeAgents).toBe(1);
      expect(stats.tasksInProgress).toBe(3);
    });

    it("should track idle agents", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 10,
        maximum: 100,
        usagePercent: 10,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      await monitor.updateTaskCount("agent-1", 0);

      const stats = await monitor.getPoolStats();
      expect(stats.idleAgents).toBe(1);
    });
  });

  describe("Agent Management", () => {
    it("should remove agent", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 50,
        maximum: 100,
        usagePercent: 50,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      await monitor.removeAgent("agent-1");

      const profile = await monitor.getAgentResources("agent-1");
      expect(profile).toBeNull();
    });

    it("should get all agent resources", async () => {
      await monitor.recordUsage("agent-1", {
        type: ResourceType.CPU,
        current: 50,
        maximum: 100,
        usagePercent: 50,
        unit: "%",
        timestamp: new Date(),
        source: "agent-1",
      });

      await monitor.recordUsage("agent-2", {
        type: ResourceType.CPU,
        current: 60,
        maximum: 100,
        usagePercent: 60,
        unit: "%",
        timestamp: new Date(),
        source: "agent-2",
      });

      const allProfiles = await monitor.getAllAgentResources();
      expect(allProfiles).toHaveLength(2);
    });
  });

  describe("Lifecycle", () => {
    it("should start and stop monitoring", async () => {
      await monitor.start();
      await monitor.stop();
      // Should not throw
    });
  });
});
