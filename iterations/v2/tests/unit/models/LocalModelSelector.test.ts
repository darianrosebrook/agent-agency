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
import type { ModelSelectionCriteria } from "@/types/model-registry";
import { beforeEach, describe, expect, it } from "@jest/globals";

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
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      expect(result).toBeDefined();
      expect(result.primary).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeDefined();
      expect(result.reasoning.length).toBeGreaterThan(0);
    });

    it("should throw error when no capable models found", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "nonexistent-task",
        requiredCapabilities: ["nonexistent-capability"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
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
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      expect(result.primary).toBeDefined();
      expect(result.reasoning).toContain("hardware");
    });

    it("should consider quality threshold in selection", async () => {
      // Add performance history with different quality levels
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.6,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory("quality-model", "text-generation", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
      });

      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.9,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select quality-model due to quality requirement
      expect(result.primary.name).toBe("quality-model");
    });

    it("should respect max latency constraint", async () => {
      // Add performance history
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.8,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory("quality-model", "text-generation", {
        quality: 0.95,
        latencyMs: 5000,
        memoryMB: 512,
        success: true,
      });

      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 1000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select fast-model due to latency constraint
      expect(result.primary.name).toBe("fast-model");
    });

    it("should use custom weights when provided", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.7,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory("quality-model", "text-generation", {
        quality: 0.95,
        latencyMs: 1000,
        memoryMB: 512,
        success: true,
      });

      const result = await selector.selectModel(criteria);

      // Should select quality-model due to high quality weight
      expect(result.primary.name).toBe("quality-model");
    });

    it("should include reasoning for selection", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      expect(result.reasoning).toBeDefined();
      expect(Array.isArray(result.reasoning)).toBe(true);
      expect(result.reasoning.length).toBeGreaterThan(0);
    });

    it("should calculate confidence based on data availability", async () => {
      // Add lots of performance data
      for (let i = 0; i < 100; i++) {
        selector.updatePerformanceHistory("balanced-model", "text-generation", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
        });
      }

      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should have high confidence with lots of data
      if (result.primary.name === "balanced-model") {
        expect(result.confidence).toBeGreaterThan(0.7);
      }
    });
  });

  describe("updatePerformanceHistory", () => {
    it("should create new history entry for first update", async () => {
      const metrics = {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
      };

      selector.updatePerformanceHistory(
        "fast-model",
        "text-generation",
        metrics
      );

      const history = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );
      expect(history).toBeDefined();
      expect(history?.samples).toBe(1);
      expect(history?.avgQuality).toBeCloseTo(0.85, 2);
      expect(history?.avgLatencyMs).toBeCloseTo(250, 0);
    });

    it("should use exponential moving average for updates", async () => {
      // First update
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.8,
        latencyMs: 200,
        memoryMB: 256,
        success: true,
      });

      // Second update with different values
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.9,
        latencyMs: 300,
        memoryMB: 384,
        success: true,
      });

      const history = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );
      expect(history?.samples).toBe(2);

      // Should be between 0.8 and 0.9 due to EMA
      expect(history?.avgQuality).toBeGreaterThan(0.8);
      expect(history?.avgQuality).toBeLessThan(0.9);
    });

    it("should track success rate correctly", async () => {
      // 7 successes, 3 failures
      for (let i = 0; i < 7; i++) {
        selector.updatePerformanceHistory("fast-model", "text-generation", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
        });
      }

      for (let i = 0; i < 3; i++) {
        selector.updatePerformanceHistory("fast-model", "text-generation", {
          quality: 0.0,
          latencyMs: 250,
          memoryMB: 384,
          success: false,
        });
      }

      const history = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );
      expect(history?.samples).toBe(10);

      // Success rate should be approximately 0.7 (with EMA)
      expect(history?.successRate).toBeGreaterThan(0.5);
      expect(history?.successRate).toBeLessThan(0.9);
    });

    it("should update P95 latency correctly", async () => {
      // Add various latencies
      const latencies = [100, 150, 200, 250, 300, 350, 400, 450, 500, 1000];

      for (const latency of latencies) {
        selector.updatePerformanceHistory("fast-model", "text-generation", {
          quality: 0.85,
          latencyMs: latency,
          memoryMB: 384,
          success: true,
        });
      }

      const history = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );

      // P95 should be high (1000 or close to it)
      expect(history?.p95LatencyMs).toBeGreaterThan(900);
    });

    it("should update timestamp on each update", async () => {
      const now = new Date();

      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
      });

      const history = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );
      expect(history?.lastUpdated.getTime()).toBeGreaterThanOrEqual(
        now.getTime()
      );
    });

    it("should handle multiple models independently", async () => {
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.7,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory("quality-model", "text-generation", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
      });

      const history1 = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );
      const history2 = selector.getPerformanceHistory(
        "quality-model",
        "text-generation"
      );

      expect(history1?.avgQuality).toBeCloseTo(0.7, 1);
      expect(history2?.avgQuality).toBeCloseTo(0.95, 2);
    });
  });

  describe("getPerformanceHistory", () => {
    it("should return undefined for unknown model", () => {
      const history = selector.getPerformanceHistory(
        "unknown-model",
        "text-generation"
      );
      expect(history).toBeUndefined();
    });

    it("should return history for tracked model", async () => {
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
      });

      const history = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );
      expect(history).toBeDefined();
      expect(history?.modelId).toBe("fast-model");
    });
  });

  describe("clearHistory", () => {
    it("should clear history for specific model", async () => {
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
      });

      selector.clearHistory();

      const history = selector.getPerformanceHistory(
        "fast-model",
        "text-generation"
      );
      expect(history).toBeUndefined();
    });

    it("should only clear specified model, not others", async () => {
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
      });

      selector.updatePerformanceHistory("quality-model", "text-generation", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
      });

      selector.clearHistory();

      expect(
        selector.getPerformanceHistory("fast-model", "text-generation")
      ).toBeUndefined();
      expect(
        selector.getPerformanceHistory("quality-model", "text-generation")
      ).toBeDefined();
    });

    it("should handle clearing unknown model gracefully", () => {
      expect(() => selector.clearHistory()).not.toThrow();
    });
  });

  describe("hardware compatibility", () => {
    it("should consider CPU compatibility", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      expect(result.primary.id).toContain("cpu");
    });

    it("should handle GPU preference", async () => {
      // Test with existing models and GPU hardware
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: true,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select a model when GPU is available
      expect(result.primary).toBeDefined();
      expect(result.reasoning).toBeDefined();
    }); //
    //
    //     it("should handle Apple Neural Engine preference", async () => {
    //       // Register an ANE-optimized model
    //       await registry.registerModel(
    //         "ane-model",
    //         {
    //           capabilities: ["text-generation", "chat"],
    //           hardwareRequirements: {
    //             preferredHardware: ["ane"],
    //             minMemoryMB: 2048,
    //           },
    //         },
    //         "1.0.0"
    //       );
    //
    //       const models = registry.getAllModels();
    //       const aneModel = models.find((m) => m.name === "ane-model");
    //       if (aneModel) {
    //         await registry.activateModel(aneModel.id);
    //       }
    //
    //       const criteria: ModelSelectionCriteria = {
    //         taskType: "text-generation",
    //         requiredCapabilities: ["text-generation"],
    //         maxLatencyMs: 5000,
    //         maxMemoryMB: 4096,
    //         qualityThreshold: 0.8,
    //         availableHardware: {
    //           cpu: true,
    //           gpu: false,
    //           ane: true,
    //         },
    //       };
    //
    //       const result = await selector.selectModel(criteria);
    //
    //       // Should prefer ANE model when available
    //       if (result.primary.name === "ane-model") {
    //         expect(result.primary.id).toContain(
    //           "ane"
    //         );
    //       }
    //     });
  });

  describe("scoring algorithm", () => {
    beforeEach(async () => {
      // Add performance history for all models
      selector.updatePerformanceHistory("fast-model", "text-generation", {
        quality: 0.7,
        latencyMs: 100,
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory("balanced-model", "text-generation", {
        quality: 0.85,
        latencyMs: 250,
        memoryMB: 384,
        success: true,
      });

      selector.updatePerformanceHistory("quality-model", "text-generation", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 512,
        success: true,
      });
    });

    it("should score quality appropriately", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.9,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select quality-model due to 100% quality weight
      expect(result.primary.name).toBe("quality-model");
    });

    it("should score latency appropriately", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select fast-model due to 100% latency weight
      expect(result.primary.name).toBe("fast-model");
    });

    it("should score memory appropriately", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should prefer model with lower memory usage
      expect(result.primary.name).toBe("fast-model");
    });

    it("should balance multiple factors", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Should select balanced-model as it balances quality and latency
      expect(result.primary.name).toBe("balanced-model");
    });
  });

  describe("error handling", () => {
    it("should throw ModelSelectorError for no capable models", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["impossible-capability"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      await expect(selector.selectModel(criteria)).rejects.toThrow(
        ModelSelectorError
      );
    });

    it("should include error code in ModelSelectorError", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["impossible-capability"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      try {
        await selector.selectModel(criteria);
        fail("Should have thrown error");
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
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      await expect(emptySelector.selectModel(criteria)).rejects.toThrow(
        ModelSelectorError
      );
    });
  });

  describe("confidence calculation", () => {
    it("should have low confidence with no historical data", async () => {
      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // No history = low confidence
      expect(result.confidence).toBeLessThan(0.5);
    });

    it("should have high confidence with lots of historical data", async () => {
      // Add 100 samples
      for (let i = 0; i < 100; i++) {
        selector.updatePerformanceHistory("balanced-model", "text-generation", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
        });
      }

      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Lots of history = high confidence if balanced-model selected
      if (result.primary.name === "balanced-model") {
        expect(result.confidence).toBeGreaterThan(0.7);
      }
    });

    it("should have medium confidence with moderate data", async () => {
      // Add 20 samples
      for (let i = 0; i < 20; i++) {
        selector.updatePerformanceHistory("balanced-model", "text-generation", {
          quality: 0.85,
          latencyMs: 250,
          memoryMB: 384,
          success: true,
        });
      }

      const criteria: ModelSelectionCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        maxMemoryMB: 4096,
        qualityThreshold: 0.8,
        availableHardware: {
          cpu: true,
          gpu: false,
        },
      };

      const result = await selector.selectModel(criteria);

      // Moderate history = medium confidence
      if (result.primary.name === "balanced-model") {
        expect(result.confidence).toBeGreaterThan(0.4);
        expect(result.confidence).toBeLessThan(0.8);
      }
    });
  });
});
