/**
 * @file ModelRegistryIntegration.test.ts
 * @description Integration tests for Model Registry, Selector, and Cost Tracker
 * @author @darianrosebrook
 */

import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import { OllamaProvider } from "@/models/providers/OllamaProvider";
import type {
  GenerationRequest,
  ModelSelectionCriteria,
  OllamaModelConfig,
} from "@/types/model-registry";
import { beforeEach, describe, expect, it, vi } from "vitest";

describe("Model Registry Integration", () => {
  let registry: ModelRegistry;
  let costTracker: ComputeCostTracker;
  let selector: LocalModelSelector;

  beforeEach(async () => {
    registry = new ModelRegistry();
    costTracker = new ComputeCostTracker();
    selector = new LocalModelSelector(registry, costTracker);
  });

  describe("End-to-End Model Management Flow", () => {
    it("should complete full lifecycle: register → activate → select → track", async () => {
      // Step 1: Register models
      const model1 = await registry.registerOllamaModel(
        "fast-model",
        "gemma3:1b",
        "1.0.0",
        "fast"
      );

      const model2 = await registry.registerOllamaModel(
        "balanced-model",
        "gemma3n:e2b",
        "1.0.0",
        "primary"
      );

      expect(model1.status).toBe("registered");
      expect(model2.status).toBe("registered");

      // Step 2: Activate models
      await registry.activateModel(model1.id);
      await registry.activateModel(model2.id);

      const activeModels = registry.getActiveModels();
      expect(activeModels).toHaveLength(2);

      // Step 3: Record performance for selection
      await selector.updatePerformanceHistory(model1.name, {
        quality: 0.75,
        latencyMs: 150,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory(model2.name, {
        quality: 0.85,
        latencyMs: 300,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      // Step 4: Select model based on criteria
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const selected = await selector.selectModel(criteria);

      expect(selected.model).toBeDefined();
      expect(selected.confidence).toBeGreaterThan(0);

      // Step 5: Track compute costs
      costTracker.recordOperation({
        modelId: selected.model.id,
        operationId: "test-op-1",
        timestamp: new Date(),
        wallClockMs: 250,
        cpuTimeMs: 200,
        peakMemoryMB: 400,
        avgMemoryMB: 300,
        cpuUtilization: 80,
        tokensPerSecond: 60,
      });

      const costProfile = costTracker.getCostProfile(selected.model.id);
      expect(costProfile).toBeDefined();
      expect(costProfile?.avgWallClockMs).toBe(250);
    });

    it("should handle model versioning across all components", async () => {
      // Register v1.0.0
      const v1 = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "1.0.0"
      );

      await registry.activateModel(v1.id);

      // Track performance for v1
      await selector.updatePerformanceHistory("test-model", {
        quality: 0.75,
        latencyMs: 200,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      costTracker.recordOperation({
        modelId: v1.id,
        operationId: "v1-op-1",
        timestamp: new Date(),
        wallClockMs: 200,
        cpuTimeMs: 150,
        peakMemoryMB: 300,
        avgMemoryMB: 250,
        cpuUtilization: 75,
        tokensPerSecond: 50,
      });

      // Register v2.0.0
      const v2 = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "2.0.0"
      );

      await registry.activateModel(v2.id);

      // Track performance for v2
      await selector.updatePerformanceHistory("test-model", {
        quality: 0.85,
        latencyMs: 150,
        memoryMB: 256,
        success: true,
        timestamp: new Date(),
      });

      costTracker.recordOperation({
        modelId: v2.id,
        operationId: "v2-op-1",
        timestamp: new Date(),
        wallClockMs: 150,
        cpuTimeMs: 120,
        peakMemoryMB: 280,
        avgMemoryMB: 240,
        cpuUtilization: 70,
        tokensPerSecond: 65,
      });

      // Compare versions
      const comparison = costTracker.compareCosts(v1.id, v2.id);
      expect(comparison).toBeDefined();
      expect(comparison?.winner).toBe(v2.id); // v2 is faster
    });

    it("should maintain separation between model versions in selection", async () => {
      // Register two versions
      const v1 = await registry.registerOllamaModel(
        "model",
        "gemma3:1b",
        "1.0.0"
      );
      const v2 = await registry.registerOllamaModel(
        "model",
        "gemma3:1b",
        "2.0.0"
      );

      await registry.activateModel(v1.id);
      await registry.activateModel(v2.id);

      // Different performance for each version
      await selector.updatePerformanceHistory("model", {
        quality: 0.7,
        latencyMs: 300,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      // Selection should consider version-specific performance
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.6,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const selected = await selector.selectModel(criteria);
      expect(selected.model.name).toBe("model");
    });
  });

  describe("Performance Tracking Integration", () => {
    beforeEach(async () => {
      // Register and activate models
      const model1 = await registry.registerOllamaModel(
        "fast-model",
        "gemma3:1b",
        "1.0.0",
        "fast"
      );
      const model2 = await registry.registerOllamaModel(
        "quality-model",
        "gemma3n:e4b",
        "1.0.0",
        "quality"
      );

      await registry.activateModel(model1.id);
      await registry.activateModel(model2.id);
    });

    it("should use cost tracking data in model selection", async () => {
      // Record costs for both models
      const models = registry.getAllModels();

      // Fast model has low costs
      for (let i = 0; i < 20; i++) {
        costTracker.recordOperation({
          modelId: models[0].id,
          operationId: `fast-op-${i}`,
          timestamp: new Date(),
          wallClockMs: 100,
          cpuTimeMs: 80,
          peakMemoryMB: 256,
          avgMemoryMB: 200,
          cpuUtilization: 70,
          tokensPerSecond: 80,
        });

        await selector.updatePerformanceHistory(models[0].name, {
          quality: 0.75,
          latencyMs: 100,
          memoryMB: 200,
          success: true,
          timestamp: new Date(),
        });
      }

      // Quality model has higher costs
      for (let i = 0; i < 20; i++) {
        costTracker.recordOperation({
          modelId: models[1].id,
          operationId: `quality-op-${i}`,
          timestamp: new Date(),
          wallClockMs: 500,
          cpuTimeMs: 400,
          peakMemoryMB: 512,
          avgMemoryMB: 450,
          cpuUtilization: 90,
          tokensPerSecond: 40,
        });

        await selector.updatePerformanceHistory(models[1].name, {
          quality: 0.95,
          latencyMs: 500,
          memoryMB: 450,
          success: true,
          timestamp: new Date(),
        });
      }

      // Select with latency priority
      const latencyCriteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.2,
          latency: 0.6,
          memory: 0.1,
          reliability: 0.05,
          recency: 0.05,
        },
      };

      const latencySelected = await selector.selectModel(latencyCriteria);
      expect(latencySelected.model.name).toBe("fast-model");

      // Select with quality priority
      const qualityCriteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.9,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.8,
          latency: 0.1,
          memory: 0.05,
          reliability: 0.025,
          recency: 0.025,
        },
      };

      const qualitySelected = await selector.selectModel(qualityCriteria);
      expect(qualitySelected.model.name).toBe("quality-model");
    });

    it("should provide optimization recommendations based on tracked costs", async () => {
      const model = await registry.registerOllamaModel(
        "underutilized-model",
        "gemma3:1b",
        "1.0.0"
      );

      await registry.activateModel(model.id);

      // Record operations with low utilization
      for (let i = 0; i < 30; i++) {
        costTracker.recordOperation({
          modelId: model.id,
          operationId: `op-${i}`,
          timestamp: new Date(),
          wallClockMs: 1000,
          cpuTimeMs: 200, // Low CPU time
          peakMemoryMB: 512,
          avgMemoryMB: 256,
          cpuUtilization: 20, // Very low utilization
          tokensPerSecond: 30,
        });
      }

      const recommendations = costTracker.getOptimizationRecommendations(
        model.id
      );

      expect(recommendations.length).toBeGreaterThan(0);
      expect(recommendations.some((r) => r.includes("CPU"))).toBe(true);
    });
  });

  describe("OllamaProvider Integration", () => {
    it("should integrate provider with registry", async () => {
      // Register Ollama model
      const model = await registry.registerOllamaModel(
        "test-ollama",
        "gemma3:1b",
        "1.0.0"
      );

      expect(model.modelType).toBe("ollama");
      expect((model.config as OllamaModelConfig).ollamaModelName).toBe(
        "gemma3:1b"
      );

      await registry.activateModel(model.id);

      const activeModel = registry.getModel(model.id);
      expect(activeModel?.status).toBe("active");
    });

    it("should track provider costs through compute tracker", async () => {
      const config: OllamaModelConfig = {
        capabilities: ["text-generation", "chat"],
        ollamaModelName: "gemma3:1b",
        ollamaEndpoint: "http://localhost:11434",
        hardwareRequirements: {
          preferredHardware: ["cpu"],
          minMemoryMB: 2048,
        },
      };

      const provider = new OllamaProvider(config);

      // Mock fetch for Ollama API
      global.fetch = vi.fn();

      // Mock successful generation
      (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          model: "gemma3:1b",
          created_at: new Date().toISOString(),
          response: "Test response",
          done: true,
          context: [],
          total_duration: 250000000, // 250ms in nanoseconds
          load_duration: 50000000,
          prompt_eval_count: 10,
          prompt_eval_duration: 100000000,
          eval_count: 20,
          eval_duration: 100000000,
        }),
      });

      const request: GenerationRequest = {
        prompt: "Test prompt",
        maxTokens: 100,
        temperature: 0.7,
      };

      const response = await provider.generate(request);

      expect(response).toBeDefined();
      expect(response.cost).toBeDefined();
      expect(response.cost.wallClockMs).toBeGreaterThan(0);

      // Record in cost tracker
      costTracker.recordOperation(response.cost);

      const profile = costTracker.getCostProfile(response.cost.modelId);
      expect(profile).toBeDefined();
    });
  });

  describe("Model Selection with Real-World Scenarios", () => {
    beforeEach(async () => {
      // Set up realistic model fleet
      await registry.registerOllamaModel(
        "gemma-2b",
        "gemma3:1b",
        "1.0.0",
        "fast"
      );
      await registry.registerOllamaModel(
        "gemma-7b",
        "gemma3n:e2b",
        "1.0.0",
        "primary"
      );
      await registry.registerOllamaModel(
        "gemma-14b",
        "gemma3n:e4b",
        "1.0.0",
        "quality"
      );

      const models = registry.getAllModels();
      for (const model of models) {
        await registry.activateModel(model.id);
      }

      // Record realistic performance data
      await selector.updatePerformanceHistory("gemma-2b", {
        quality: 0.7,
        latencyMs: 120,
        memoryMB: 2048,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("gemma-7b", {
        quality: 0.85,
        latencyMs: 300,
        memoryMB: 8192,
        success: true,
        timestamp: new Date(),
      });

      await selector.updatePerformanceHistory("gemma-14b", {
        quality: 0.95,
        latencyMs: 700,
        memoryMB: 16384,
        success: true,
        timestamp: new Date(),
      });
    });

    it("should select small model for fast, low-quality tasks", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 200,
        minQuality: 0.6,
        qualityThreshold: 0.7,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.2,
          latency: 0.6,
          memory: 0.1,
          reliability: 0.05,
          recency: 0.05,
        },
      };

      const selected = await selector.selectModel(criteria);
      expect(selected.model.name).toBe("gemma-2b");
    });

    it("should select large model for high-quality tasks", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.9,
        qualityThreshold: 0.95,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.8,
          latency: 0.1,
          memory: 0.05,
          reliability: 0.025,
          recency: 0.025,
        },
      };

      const selected = await selector.selectModel(criteria);
      expect(selected.model.name).toBe("gemma-14b");
    });

    it("should select medium model for balanced tasks", async () => {
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 1000,
        minQuality: 0.8,
        qualityThreshold: 0.85,
        preferredHardware: ["cpu"],
        weights: {
          quality: 0.5,
          latency: 0.3,
          memory: 0.1,
          reliability: 0.05,
          recency: 0.05,
        },
      };

      const selected = await selector.selectModel(criteria);
      expect(selected.model.name).toBe("gemma-7b");
    });
  });

  describe("Concurrent Model Usage", () => {
    it("should handle concurrent model operations", async () => {
      const model1 = await registry.registerOllamaModel(
        "concurrent-1",
        "gemma3:1b",
        "1.0.0"
      );
      const model2 = await registry.registerOllamaModel(
        "concurrent-2",
        "gemma3:1b",
        "1.0.0"
      );

      await registry.activateModel(model1.id);
      await registry.activateModel(model2.id);

      // Simulate concurrent operations
      const operations = [];

      for (let i = 0; i < 10; i++) {
        operations.push(
          selector.updatePerformanceHistory("concurrent-1", {
            quality: 0.8,
            latencyMs: 200,
            memoryMB: 256,
            success: true,
            timestamp: new Date(),
          })
        );

        operations.push(
          selector.updatePerformanceHistory("concurrent-2", {
            quality: 0.85,
            latencyMs: 250,
            memoryMB: 384,
            success: true,
            timestamp: new Date(),
          })
        );
      }

      await Promise.all(operations);

      const history1 = selector.getPerformanceHistory("concurrent-1");
      const history2 = selector.getPerformanceHistory("concurrent-2");

      expect(history1?.samples).toBe(10);
      expect(history2?.samples).toBe(10);
    });
  });

  describe("Model Lifecycle Management", () => {
    it("should handle model deprecation and replacement", async () => {
      // Register old model
      const oldModel = await registry.registerOllamaModel(
        "old-model",
        "gemma3:1b",
        "1.0.0"
      );

      await registry.activateModel(oldModel.id);

      // Track performance
      await selector.updatePerformanceHistory("old-model", {
        quality: 0.7,
        latencyMs: 300,
        memoryMB: 384,
        success: true,
        timestamp: new Date(),
      });

      // Deprecate old model
      await registry.deprecateModel(oldModel.id, "Replaced by new-model");

      // Register new model
      const newModel = await registry.registerOllamaModel(
        "new-model",
        "gemma3n:e2b",
        "1.0.0"
      );

      await registry.activateModel(newModel.id);

      // Only active models should be selectable
      const criteria: ModelSelectionCriteria = {
        requiredCapabilities: ["text-generation"],
        maxLatencyMs: 5000,
        minQuality: 0.7,
        qualityThreshold: 0.8,
        preferredHardware: ["cpu"],
      };

      const selected = await selector.selectModel(criteria);

      // Should select new model, not deprecated one
      expect(selected.model.name).toBe("new-model");
    });
  });
});
