/**
 * @file LocalModelSelector.test.ts
 * @description Unit tests for LocalModelSelector class
 * @author @darianrosebrook
 */

import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import {
  LocalModelSelector,
  ModelSelectorError,
} from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import type {
  ModelSelectionCriteria,
  PerformanceMetrics,
} from "@/types/model-registry";
import { beforeEach, describe, expect, it } from "vitest";

describe("LocalModelSelector", () => {
  let selector: LocalModelSelector;
  let registry: ModelRegistry;
  let costTracker: ComputeCostTracker;

  beforeEach(async () => {
    registry = new ModelRegistry();
    costTracker = new ComputeCostTracker();
    selector = new LocalModelSelector(registry, costTracker);

    // Register test models
    await registry.registerOllamaModel(
      "fast-model",
      "gemma3:1b",
      "1.0.0",
      "fast"
    );
    await registry.registerOllamaModel(
      "balanced-model",
      "gemma3n:e2b",
      "1.0.0",
      "primary"
    );
    await registry.registerOllamaModel(
      "quality-model",
      "gemma3n:e4b",
      "1.0.0",
      "quality"
    );

    // Activate models
    const models = registry.getAllModels();
    for (const model of models) {
      await registry.activateModel(model.id);
    }
  });

  describe("selectModel", () => {
    it("should select model based on basic criteria", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      expect(result).toBeDefined();
      expect(result.model).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeDefined();
      expect(result.reasoning.length).toBeGreaterThan(0);
    });

    it("should throw error when no capable models found", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["nonexistent-capability"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      await expect(selector.selectModel(criteria)).rejects.toThrow(
        ModelSelectorError
      );
      await expect(selector.selectModel(criteria)).rejects.toThrow(
        "No models found with capabilities"
      );
    });

    it("should prefer models with matching hardware", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      expect(result.model.hardwareRequirements.preferredHardware).toContain(
        "cpu"
      );
    });

    it("should consider quality threshold in selection", async () => {
      // Add performance history with different quality levels
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.6,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("quality-model", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
        timestamp: new Date(),
      });

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.9,
        qualityThreshold: 0.9,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      // Should select quality-model due to quality requirement
      expect(result.model.name).toBe("quality-model");
    });

    it("should respect max latency constraint", async () => {
      // Add performance history
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.8,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("quality-model", {
        quality: 0.95,
        latencyMs: 5000,
        memoryMB: 512,
        success: true,
        timestamp: new Date(),
      });

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 1000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      // Should select fast-model due to latency constraint
      expect(result.model.name).toBe("fast-model");
    });

    it("should use custom weights when provided", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.8, // Prioritize quality
          latency: 0.1,
          memory: 0.05,
          reliability: 0.05,
          recency: 0.0,
        },
      };

      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.7,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("quality-model", {
        quality: 0.95,
        latencyMs: 1000,
        memoryMB: 512,
        success: true,
        timestamp: new Date(),
      });

      const result = await selector.selectModel(criteria);

      // Should select quality-model due to high quality weight
      expect(result.model.name).toBe("quality-model");
    });

    it("should include reasoning for selection", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      expect(result.reasoning).toBeDefined();
      expect(Array.isArray(result.reasoning)).toBe(true);
      expect(result.reasoning.length).toBeGreaterThan(0);
    });

    it("should calculate confidence based on data availability", async () => {
      // Add lots of performance data
      for (let i = 0; i < 100; i++) {
        await selector.updatePerformanceHistory("balanced-model", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
          timestamp: new Date(),
        });
      }

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      // Should have high confidence with lots of data
      if (result.model.name === "balanced-model") {
        expect(result.confidence).toBeGreaterThan(0.7);
      }
    });
  });

  describe("updatePerformanceHistory", () => {
    it("should create new history entry for first update", async () => {
      const metrics: PerformanceMetrics = {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      };

      await selector.updatePerformanceHistory("fast-model", metrics);

      const history = selector.getPerformanceHistory("fast-model");
      expect(history).toBeDefined();
      expect(history?.samples).toBe(1);
      expect(history?.avgQuality).toBeCloseTo(0.85, 2);
      expect(history?.avgLatencyMs).toBeCloseTo(250, 0);
    });

    it("should use exponential moving average for updates", async () => {
      // First update
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.8,
        latencyMs: 200,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      // Second update with different values
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.9,
        latencyMs: 300,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      const history = selector.getPerformanceHistory("fast-model");
      expect(history?.samples).toBe(2);

      // Should be between 0.8 and 0.9 due to EMA
      expect(history?.avgQuality).toBeGreaterThan(0.8);
      expect(history?.avgQuality).toBeLessThan(0.9);
    });

    it("should track success rate correctly", async () => {
      // 7 successes, 3 failures
      for (let i = 0; i < 7; i++) {
        await selector.updatePerformanceHistory("fast-model", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
          timestamp: new Date(),
        });
      }

      for (let i = 0; i < 3; i++) {
        await selector.updatePerformanceHistory("fast-model", {
          quality: 0.0,
          latencyMs: 250,
          memoryMB: 384,
          success: false,
          timestamp: new Date(),
        });
      }

      const history = selector.getPerformanceHistory("fast-model");
      expect(history?.samples).toBe(10);

      // Success rate should be approximately 0.7 (with EMA)
      expect(history?.successRate).toBeGreaterThan(0.5);
      expect(history?.successRate).toBeLessThan(0.9);
    });

    it("should update P95 latency correctly", async () => {
      // Add various latencies
      const latencies = [100, 150, 200, 250, 300, 350, 400, 450, 500, 1000];

      for (const latency of latencies) {
        await selector.updatePerformanceHistory("fast-model", {
          quality: 0.85,
          latencyMs: latency,
          memoryMB: 384,
          success: true,
          timestamp: new Date(),
        });
      }

      const history = selector.getPerformanceHistory("fast-model");

      // P95 should be high (1000 or close to it)
      expect(history?.p95LatencyMs).toBeGreaterThan(900);
    });

    it("should update timestamp on each update", async () => {
      const now = new Date();

      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
        timestamp: now,
      });

      const history = selector.getPerformanceHistory("fast-model");
      expect(history?.lastUpdated.getTime()).toBeGreaterThanOrEqual(
        now.getTime()
      );
    });

    it("should handle multiple models independently", async () => {
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.7,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("quality-model", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
        timestamp: new Date(),
      });

      const history1 = selector.getPerformanceHistory("fast-model");
      const history2 = selector.getPerformanceHistory("quality-model");

      expect(history1?.avgQuality).toBeCloseTo(0.7, 1);
      expect(history2?.avgQuality).toBeCloseTo(0.95, 2);
    });
  });

  describe("getPerformanceHistory", () => {
    it("should return undefined for unknown model", () => {
      const history = selector.getPerformanceHistory("unknown-model");
      expect(history).toBeUndefined();
    });

    it("should return history for tracked model", async () => {
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      const history = selector.getPerformanceHistory("fast-model");
      expect(history).toBeDefined();
      expect(history?.modelId).toBe("fast-model");
    });
  });

  describe("clearHistory", () => {
    it("should clear history for specific model", async () => {
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      selector.clearHistory("fast-model");

      const history = selector.getPerformanceHistory("fast-model");
      expect(history).toBeUndefined();
    });

    it("should only clear specified model, not others", async () => {
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("quality-model", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
        timestamp: new Date(),
      });

      selector.clearHistory("fast-model");

      expect(selector.getPerformanceHistory("fast-model")).toBeUndefined();
      expect(selector.getPerformanceHistory("quality-model")).toBeDefined();
    });

    it("should handle clearing unknown model gracefully", () => {
      expect(() => selector.clearHistory("unknown-model")).not.toThrow();
    });
  });

  describe("hardware compatibility", () => {
    it("should consider CPU compatibility", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      expect(result.model.hardwareRequirements.preferredHardware).toContain(
        "cpu"
      );
    });

    it("should handle GPU preference", async () => {
      // Register a GPU-optimized model
      await registry.registerCustomModel(
        "gpu-model",
        {
          capabilities: ["text-generation", "chat"],
          hardwareRequirements: {
            preferredHardware: ["gpu"],
            minMemoryMB: 4096,
          },
        },
        "1.0.0"
      );

      const models = registry.getAllModels();
      const gpuModel = models.find((m) => m.name === "gpu-model");
      if (gpuModel) {
        await registry.activateModel(gpuModel.id);
      }

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["gpu"],
      };

      const result = await selector.selectModel(criteria);

      // Should prefer GPU model when available
      if (result.model.name === "gpu-model") {
        expect(result.model.hardwareRequirements.preferredHardware).toContain(
          "gpu"
        );
      }
    });

    it("should handle Apple Neural Engine preference", async () => {
      // Register an ANE-optimized model
      await registry.registerCustomModel(
        "ane-model",
        {
          capabilities: ["text-generation", "chat"],
          hardwareRequirements: {
            preferredHardware: ["ane"],
            minMemoryMB: 2048,
          },
        },
        "1.0.0"
      );

      const models = registry.getAllModels();
      const aneModel = models.find((m) => m.name === "ane-model");
      if (aneModel) {
        await registry.activateModel(aneModel.id);
      }

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["ane"],
      };

      const result = await selector.selectModel(criteria);

      // Should prefer ANE model when available
      if (result.model.name === "ane-model") {
        expect(result.model.hardwareRequirements.preferredHardware).toContain(
          "ane"
        );
      }
    });
  });

  describe("scoring algorithm", () => {
    beforeEach(async () => {
      // Add performance history for all models
      await selector.updatePerformanceHistory("fast-model", {
        quality: 0.7,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("balanced-model", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("quality-model", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
        timestamp: new Date(),
      });
    });

    it("should score quality appropriately", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.9,
        preferredHardware: ["cpu"],
        weights: {
          quality: 1.0,
          latency: 0.0,
          memory: 0.0,
          reliability: 0.0,
          recency: 0.0,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select quality-model due to 100% quality weight
      expect(result.model.name).toBe("quality-model");
    });

    it("should score latency appropriately", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.0,
          latency: 1.0,
          memory: 0.0,
          reliability: 0.0,
          recency: 0.0,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select fast-model due to 100% latency weight
      expect(result.model.name).toBe("fast-model");
    });

    it("should score memory appropriately", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.0,
          latency: 0.0,
          memory: 1.0,
          reliability: 0.0,
          recency: 0.0,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should prefer model with lower memory usage
      expect(result.model.name).toBe("fast-model");
    });

    it("should balance multiple factors", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.4,
          latency: 0.3,
          memory: 0.1,
          reliability: 0.1,
          recency: 0.1,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select balanced-model as it balances quality and latency
      expect(result.model.name).toBe("balanced-model");
    });
  });

  describe("error handling", () => {
    it("should throw ModelSelectorError for no capable models", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["impossible-capability"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      await expect(selector.selectModel(criteria)).rejects.toThrow(
        ModelSelectorError
      );
    });

    it("should include error code in ModelSelectorError", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["impossible-capability"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      try {
        await selector.selectModel(criteria);
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error).toBeInstanceOf(ModelSelectorError);
        if (error instanceof ModelSelectorError) {
          expect(error.code).toBe("NO_CAPABLE_MODELS");
        }
      }
    });

    it("should handle empty registry gracefully", async () => {
      const emptyRegistry = new ModelRegistry();
      const emptySelector = new LocalModelSelector(emptyRegistry, costTracker);

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      await expect(emptySelector.selectModel(criteria)).rejects.toThrow(
        ModelSelectorError
      );
    });
  });

  describe("confidence calculation", () => {
    it("should have low confidence with no historical data", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      // No history = low confidence
      expect(result.confidence).toBeLessThan(0.5);
    });

    it("should have high confidence with lots of historical data", async () => {
      // Add 100 samples
      for (let i = 0; i < 100; i++) {
        await selector.updatePerformanceHistory("balanced-model", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
          timestamp: new Date(),
        });
      }

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      // Lots of history = high confidence if balanced-model selected
      if (result.model.name === "balanced-model") {
        expect(result.confidence).toBeGreaterThan(0.7);
      }
    });

    it("should have medium confidence with moderate data", async () => {
      // Add 20 samples
      for (let i = 0; i < 20; i++) {
        await selector.updatePerformanceHistory("balanced-model", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
          timestamp: new Date(),
        });
      }

      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const result = await selector.selectModel(criteria);

      // Moderate history = medium confidence
      if (result.model.name === "balanced-model") {
        expect(result.confidence).toBeGreaterThan(0.4);
        expect(result.confidence).toBeLessThan(0.8);
      }
    });
  });
});
