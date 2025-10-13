/**
 * @file ComputeCostTracker.test.ts
 * @description Unit tests for ComputeCostTracker class
 * @author @darianrosebrook
 */

import type { LocalComputeCost } from "@/types/model-registry";
import { beforeEach, describe, expect, it } from "@jest/globals";
import { ComputeCostTracker } from "../../../src/models/ComputeCostTracker";

describe("ComputeCostTracker", () => {
  let tracker: ComputeCostTracker;

  beforeEach(() => {
    tracker = new ComputeCostTracker();
  });

  describe("recordOperation", () => {
    it("should record a single operation cost", () => {
      const cost: LocalComputeCost = {
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        tokensPerSecond: 50,
      };

      tracker.recordOperation(cost);

      const modelCosts = tracker.getModelCosts("model-1");
      expect(modelCosts).toHaveLength(1);
      expect(modelCosts[0]).toEqual(cost);
    });

    it("should record multiple operations for same model", () => {
      const cost1: LocalComputeCost = {
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        tokensPerSecond: 50,
      };

      const cost2: LocalComputeCost = {
        modelId: "model-1",
        operationId: "op-2",
        timestamp: new Date(),
        wallClockMs: 1200,
        cpuTimeMs: 900,
        peakMemoryMB: 600,
        avgMemoryMB: 300,
        cpuUtilization: 80,
        tokensPerSecond: 45,
      };

      tracker.recordOperation(cost1);
      tracker.recordOperation(cost2);

      const modelCosts = tracker.getModelCosts("model-1");
      expect(modelCosts).toHaveLength(2);
    });

    it("should track operations for multiple models separately", () => {
      const cost1: LocalComputeCost = {
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        tokensPerSecond: 50,
      };

      const cost2: LocalComputeCost = {
        modelId: "model-2",
        operationId: "op-2",
        timestamp: new Date(),
        wallClockMs: 1200,
        cpuTimeMs: 900,
        peakMemoryMB: 600,
        avgMemoryMB: 300,
        cpuUtilization: 80,
        tokensPerSecond: 45,
      };

      tracker.recordOperation(cost1);
      tracker.recordOperation(cost2);

      expect(tracker.getModelCosts("model-1")).toHaveLength(1);
      expect(tracker.getModelCosts("model-2")).toHaveLength(1);
    });

    it("should enforce max costs per model (FIFO)", () => {
      // Default max is 1000, let's create a custom tracker with smaller limit
      const smallTracker = new ComputeCostTracker({ maxCostsPerModel: 5 });

      // Record 10 operations
      for (let i = 0; i < 10; i++) {
        smallTracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 75,
          tokensPerSecond: 50,
        });
      }

      // Should only keep the last 5
      const costs = smallTracker.getModelCosts("model-1");
      expect(costs).toHaveLength(5);
      expect(costs[0].operationId).toBe("op-5"); // First one kept (FIFO)
      expect(costs[4].operationId).toBe("op-9"); // Last one
    });

    it("should handle GPU utilization when present", () => {
      const cost: LocalComputeCost = {
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        gpuTimeMs: 500,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        gpuUtilization: 90,
        tokensPerSecond: 50,
      };

      tracker.recordOperation(cost);

      const profile = tracker.getCostProfile("model-1");
      expect(profile?.avgGpuTimeMs).toBe(500);
    });

    it("should handle Apple Neural Engine utilization", () => {
      const cost: LocalComputeCost = {
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        aneUtilization: 85,
        tokensPerSecond: 50,
      };

      tracker.recordOperation(cost);

      const costs = tracker.getModelCosts("model-1");
      expect(costs[0].aneUtilization).toBe(85);
    });

    it("should handle energy tracking when available", () => {
      const cost: LocalComputeCost = {
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        tokensPerSecond: 50,
        estimatedEnergyMWh: 0.05,
      };

      tracker.recordOperation(cost);

      const profile = tracker.getCostProfile("model-1");
      expect(profile?.avgEnergyMWh).toBe(0.05);
    });
  });

  describe("getCostProfile", () => {
    beforeEach(() => {
      // Record multiple operations for averaging
      for (let i = 0; i < 10; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000 + i * 100, // Increasing times
          cpuTimeMs: 800 + i * 80,
          peakMemoryMB: 512 + i * 50,
          avgMemoryMB: 256 + i * 25,
          cpuUtilization: 75 + i,
          tokensPerSecond: 50 - i,
          estimatedEnergyMWh: 0.05 + i * 0.01,
        });
      }
    });

    it("should return undefined for unknown model", () => {
      const profile = tracker.getCostProfile("unknown-model");
      expect(profile).toBeUndefined();
    });

    it("should return undefined when no costs recorded", () => {
      const emptyTracker = new ComputeCostTracker();
      const profile = emptyTracker.getCostProfile("model-1");
      expect(profile).toBeUndefined();
    });

    it("should calculate average metrics correctly", () => {
      const profile = tracker.getCostProfile("model-1");

      expect(profile).toBeDefined();
      expect(profile?.samples).toBe(10);
      expect(profile?.avgWallClockMs).toBeCloseTo(1450, 0); // Mean of 1000..1900
      expect(profile?.avgCpuTimeMs).toBeCloseTo(1160, 0); // Mean of 800..1520
      expect(profile?.avgMemoryMB).toBeCloseTo(368.5, 0); // Mean of 256..481
      expect(profile?.avgCpuUtilization).toBeCloseTo(79.5, 0); // Mean of 75..84
    });

    it("should calculate P95 latency correctly", () => {
      const profile = tracker.getCostProfile("model-1");

      expect(profile).toBeDefined();
      // P95 of [1000, 1100, ..., 1900] should be around 1850
      expect(profile?.p95LatencyMs).toBeGreaterThan(1800);
      expect(profile?.p95LatencyMs).toBeLessThanOrEqual(1900);
    });

    it("should calculate throughput correctly", () => {
      const profile = tracker.getCostProfile("model-1");

      expect(profile).toBeDefined();
      expect(profile?.avgTokensPerSecond).toBeCloseTo(45.5, 0); // Mean of 50..41
    });

    it("should respect sample size limit", () => {
      const profile = tracker.getCostProfile("model-1", 5);

      expect(profile).toBeDefined();
      expect(profile?.samples).toBe(5);
      // Should use last 5 samples: ops 5-9
      // Wall clock: 1500, 1600, 1700, 1800, 1900
      expect(profile?.avgWallClockMs).toBeCloseTo(1700, 0);
    });

    it("should handle GPU metrics when present", () => {
      tracker.recordOperation({
        modelId: "model-2",
        operationId: "op-gpu",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        gpuTimeMs: 500,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        gpuUtilization: 90,
        tokensPerSecond: 50,
      });

      const profile = tracker.getCostProfile("model-2");
      expect(profile?.avgGpuTimeMs).toBe(500);
      expect(profile?.avgGpuUtilization).toBe(90);
    });

    it("should calculate energy averages correctly", () => {
      const profile = tracker.getCostProfile("model-1");

      expect(profile).toBeDefined();
      // Energy: 0.05, 0.06, ..., 0.14
      expect(profile?.avgEnergyMWh).toBeCloseTo(0.095, 2);
    });
  });

  describe("getModelCosts", () => {
    beforeEach(() => {
      for (let i = 0; i < 20; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 75,
          tokensPerSecond: 50,
        });
      }
    });

    it("should return all costs when no limit specified", () => {
      const costs = tracker.getModelCosts("model-1");
      expect(costs).toHaveLength(20);
    });

    it("should return limited costs when limit specified", () => {
      const costs = tracker.getModelCosts("model-1", 10);
      expect(costs).toHaveLength(10);
      // Should return last 10
      expect(costs[0].operationId).toBe("op-10");
      expect(costs[9].operationId).toBe("op-19");
    });

    it("should return empty array for unknown model", () => {
      const costs = tracker.getModelCosts("unknown-model");
      expect(costs).toEqual([]);
    });

    it("should not modify internal state (returns copy)", () => {
      const costs1 = tracker.getModelCosts("model-1");
      costs1.pop(); // Modify returned array

      const costs2 = tracker.getModelCosts("model-1");
      expect(costs2).toHaveLength(20); // Still has all items
    });
  });

  describe("compareCosts", () => {
    beforeEach(() => {
      // Model 1: Fast, low energy
      for (let i = 0; i < 10; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-1-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 75,
          tokensPerSecond: 50,
          estimatedEnergyMWh: 0.05,
        });
      }

      // Model 2: Slower, higher energy
      for (let i = 0; i < 10; i++) {
        tracker.recordOperation({
          modelId: "model-2",
          operationId: `op-2-${i}`,
          timestamp: new Date(),
          wallClockMs: 2000,
          cpuTimeMs: 1600,
          peakMemoryMB: 1024,
          avgMemoryMB: 512,
          cpuUtilization: 85,
          tokensPerSecond: 25,
          estimatedEnergyMWh: 0.1,
        });
      }
    });

    it("should return undefined for unknown models", () => {
      const result = tracker.compareCosts("unknown-1", "unknown-2");
      expect(result).toBeUndefined();
    });

    it("should return undefined when one model unknown", () => {
      const result = tracker.compareCosts("model-1", "unknown-model");
      expect(result).toBeUndefined();
    });

    it("should calculate latency difference correctly", () => {
      const result = tracker.compareCosts("model-1", "model-2");

      expect(result).toBeDefined();
      // Model 2 is 100% slower (1000ms -> 2000ms)
      expect(result?.latencyDiff).toBeCloseTo(100, 0);
    });

    it("should calculate energy difference correctly", () => {
      const result = tracker.compareCosts("model-1", "model-2");

      expect(result).toBeDefined();
      // Model 2 uses 100% more energy (0.05 -> 0.1)
      expect(result?.energyDiff).toBeCloseTo(100, 0);
    });

    it("should calculate throughput difference correctly", () => {
      const result = tracker.compareCosts("model-1", "model-2");

      expect(result).toBeDefined();
      // Model 2 has -50% throughput (50 -> 25 tokens/sec)
      expect(result?.throughputDiff).toBeCloseTo(-50, 0);
    });

    it("should identify winner correctly (lower latency)", () => {
      const result = tracker.compareCosts("model-1", "model-2");

      expect(result).toBeDefined();
      expect(result?.winner).toBe("model-1"); // Faster model wins
    });

    it("should handle equal performance", () => {
      // Add model-3 with same performance as model-1
      for (let i = 0; i < 10; i++) {
        tracker.recordOperation({
          modelId: "model-3",
          operationId: `op-3-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 75,
          tokensPerSecond: 50,
          estimatedEnergyMWh: 0.05,
        });
      }

      const result = tracker.compareCosts("model-1", "model-3");

      expect(result).toBeDefined();
      expect(result?.latencyDiff).toBeCloseTo(0, 0);
      expect(result?.energyDiff).toBeCloseTo(0, 0);
      expect(result?.throughputDiff).toBeCloseTo(0, 0);
    });
  });

  describe("getOptimizationRecommendations", () => {
    it("should return empty array for unknown model", () => {
      const recommendations =
        tracker.getOptimizationRecommendations("unknown-model");
      expect(recommendations).toEqual([]);
    });

    it("should recommend low CPU utilization optimization", () => {
      // Record operations with low CPU utilization
      for (let i = 0; i < 20; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 25, // Low CPU
          tokensPerSecond: 50,
        });
      }

      const recommendations = tracker.getOptimizationRecommendations("model-1");

      expect(recommendations.length).toBeGreaterThan(0);
      expect(recommendations.some((r) => r.includes("CPU utilization"))).toBe(
        true
      );
    });

    it("should recommend low GPU utilization optimization", () => {
      // Record operations with low GPU utilization
      for (let i = 0; i < 20; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          gpuTimeMs: 500,
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 75,
          gpuUtilization: 20, // Low GPU
          tokensPerSecond: 50,
        });
      }

      const recommendations = tracker.getOptimizationRecommendations("model-1");

      expect(recommendations.length).toBeGreaterThan(0);
      expect(recommendations.some((r) => r.includes("GPU utilization"))).toBe(
        true
      );
    });

    it("should recommend memory optimization for high peak usage", () => {
      // Record operations with high memory spikes
      for (let i = 0; i < 20; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 2048, // High peak
          avgMemoryMB: 512, // Low average
          cpuUtilization: 75,
          tokensPerSecond: 50,
        });
      }

      const recommendations = tracker.getOptimizationRecommendations("model-1");

      expect(recommendations.length).toBeGreaterThan(0);
      expect(recommendations.some((r) => r.includes("Memory spikes"))).toBe(
        true
      );
    });

    it("should return insufficient data message for small sample", () => {
      // Record only 5 operations (< 10 threshold)
      for (let i = 0; i < 5; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 75,
          tokensPerSecond: 50,
        });
      }

      const recommendations = tracker.getOptimizationRecommendations("model-1");

      expect(recommendations).toContain(
        "Insufficient data for optimization recommendations"
      );
    });

    it("should handle well-optimized model (no recommendations)", () => {
      // Record operations with good utilization
      for (let i = 0; i < 20; i++) {
        tracker.recordOperation({
          modelId: "model-1",
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 800,
          peakMemoryMB: 512,
          avgMemoryMB: 480, // Consistent memory
          cpuUtilization: 80, // Good CPU
          gpuUtilization: 85, // Good GPU
          tokensPerSecond: 50,
        });
      }

      const recommendations = tracker.getOptimizationRecommendations("model-1");

      // Should have minimal or no recommendations
      expect(recommendations.length).toBeLessThanOrEqual(1);
    });
  });

  describe("getAllModelIds", () => {
    it("should return empty array when no models tracked", () => {
      const ids = tracker.getAllModelIds();
      expect(ids).toEqual([]);
    });

    it("should return all tracked model IDs", () => {
      tracker.recordOperation({
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        tokensPerSecond: 50,
      });

      tracker.recordOperation({
        modelId: "model-2",
        operationId: "op-2",
        timestamp: new Date(),
        wallClockMs: 1000,
        cpuTimeMs: 800,
        peakMemoryMB: 512,
        avgMemoryMB: 256,
        cpuUtilization: 75,
        tokensPerSecond: 50,
      });

      const ids = tracker.getAllModelIds();
      expect(ids).toHaveLength(2);
      expect(ids).toContain("model-1");
      expect(ids).toContain("model-2");
    });
  });

  describe("edge cases", () => {
    it("should handle zero values gracefully", () => {
      tracker.recordOperation({
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 0,
        cpuTimeMs: 0,
        peakMemoryMB: 0,
        avgMemoryMB: 0,
        cpuUtilization: 0,
        tokensPerSecond: 0,
      });

      const profile = tracker.getCostProfile("model-1");
      expect(profile).toBeDefined();
      expect(profile?.avgWallClockMs).toBe(0);
    });

    it("should handle very large numbers", () => {
      tracker.recordOperation({
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1e9,
        cpuTimeMs: 1e9,
        peakMemoryMB: 1e6,
        avgMemoryMB: 1e6,
        cpuUtilization: 100,
        tokensPerSecond: 1e6,
      });

      const profile = tracker.getCostProfile("model-1");
      expect(profile).toBeDefined();
      expect(profile?.avgWallClockMs).toBe(1e9);
    });

    it("should handle fractional values", () => {
      tracker.recordOperation({
        modelId: "model-1",
        operationId: "op-1",
        timestamp: new Date(),
        wallClockMs: 1.5,
        cpuTimeMs: 0.8,
        peakMemoryMB: 512.5,
        avgMemoryMB: 256.25,
        cpuUtilization: 75.5,
        tokensPerSecond: 50.75,
      });

      const profile = tracker.getCostProfile("model-1");
      expect(profile).toBeDefined();
      expect(profile?.avgWallClockMs).toBeCloseTo(1.5, 1);
    });
  });
});
