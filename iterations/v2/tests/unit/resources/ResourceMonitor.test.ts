/**
 * @fileoverview Unit tests for ResourceMonitor
 *
 * @author @darianrosebrook
 */

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

  describe("Lifecycle Management", () => {
    it("should start monitoring when not already running", async () => {
      // Test that start() doesn't throw and can be called
      await expect(monitor.start()).resolves.not.toThrow();

      // Test that we can call stop() after start()
      await expect(monitor.stop()).resolves.not.toThrow();
    });

    it("should warn and return early if already running", async () => {
      // Mock logger.warn to capture the warning
      const loggerSpy = jest
        .spyOn(monitor["logger"], "warn")
        .mockImplementation();

      // Start monitoring first time
      await monitor.start();

      // Start again - should warn and return early
      await monitor.start();

      expect(loggerSpy).toHaveBeenCalledWith(
        "Resource monitor already running"
      );

      loggerSpy.mockRestore();
    });

    it("should handle timer management correctly", async () => {
      // Mock setInterval and clearInterval
      const mockTimer = 123 as any;
      const setIntervalSpy = jest
        .spyOn(global, "setInterval")
        .mockReturnValue(mockTimer);
      const clearIntervalSpy = jest.spyOn(global, "clearInterval");

      // Start monitoring - should set up timer
      await monitor.start();

      expect(setIntervalSpy).toHaveBeenCalled();
      expect(clearIntervalSpy).not.toHaveBeenCalled(); // No existing timer to clear

      setIntervalSpy.mockRestore();
      clearIntervalSpy.mockRestore();
    });

    it("should handle timer cleanup when stopping", async () => {
      // Mock setInterval to return a mock timer
      const mockTimer = 123 as any;
      const setIntervalSpy = jest
        .spyOn(global, "setInterval")
        .mockReturnValue(mockTimer);
      const clearIntervalSpy = jest.spyOn(global, "clearInterval");

      await monitor.start();
      await monitor.stop();

      expect(clearIntervalSpy).toHaveBeenCalledWith(mockTimer);

      setIntervalSpy.mockRestore();
      clearIntervalSpy.mockRestore();
    });

    it("should stop monitoring and clear timer", async () => {
      // Mock setInterval to return a mock timer
      const mockTimer = 123 as any;
      const setIntervalSpy = jest
        .spyOn(global, "setInterval")
        .mockReturnValue(mockTimer);
      const clearIntervalSpy = jest.spyOn(global, "clearInterval");

      await monitor.start();
      await monitor.stop();

      expect(clearIntervalSpy).toHaveBeenCalledWith(mockTimer);

      setIntervalSpy.mockRestore();
      clearIntervalSpy.mockRestore();
    });

    it("should handle stop when not running", async () => {
      // Should not throw when stopping a non-running monitor
      await expect(monitor.stop()).resolves.not.toThrow();
    });

    it("should handle multiple start/stop cycles", async () => {
      // Test multiple start/stop cycles don't cause issues
      await monitor.start();
      await monitor.stop();
      await monitor.start();
      await monitor.stop();

      // Should not throw
      expect(true).toBe(true);
    });
  });
});
