/**
 * @fileoverview
 * Unit tests for ModelRegistry - local-first model management.
 * Tests registration, versioning, querying, and performance tracking.
 *
 * @author @darianrosebrook
 */

import { ModelRegistry, ModelRegistryError } from "@/models/ModelRegistry";
import type {
  ModelRegistrationRequest,
  OllamaModelConfig,
  PerformanceProfile,
} from "@/types/model-registry";
import { beforeEach, describe, expect, it } from "vitest";

describe("ModelRegistry", () => {
  let registry: ModelRegistry;

  beforeEach(() => {
    registry = new ModelRegistry();
  });

  describe("Model Registration", () => {
    it("should register a new Ollama model", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "gemma-test",
          type: "ollama",
          ollamaName: "gemma3:1b",
          version: "1.0.0",
          category: "fast",
          capabilities: ["text-generation", "chat"],
          contextWindow: 8192,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: true,
        profile: false,
      };

      const model = await registry.registerModel(request);

      expect(model).toBeDefined();
      expect(model.name).toBe("gemma-test");
      expect(model.type).toBe("ollama");
      expect(model.status).toBe("testing");
      expect(model.id).toContain("gemma-test");
      expect(model.createdAt).toBeInstanceOf(Date);
      expect(model.updatedAt).toBeInstanceOf(Date);
    });

    it("should generate unique IDs for same model name and version", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "test-model",
          type: "ollama",
          ollamaName: "gemma3:1b",
          version: "1.0.0",
          category: "primary",
          capabilities: ["text-generation"],
          contextWindow: 8192,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: false,
      };

      const model1 = await registry.registerModel(request);

      // Wait 1ms to ensure different timestamps
      await new Promise((resolve) => setTimeout(resolve, 1));

      const model2 = await registry.registerModel(request);

      expect(model1.id).not.toBe(model2.id);
      expect(model1.name).toBe(model2.name);
      expect(model1.version).toBe(model2.version);
    });

    it("should reject registration with missing name", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "",
          type: "ollama",
          ollamaName: "gemma3:1b",
          version: "1.0.0",
          category: "primary",
          capabilities: ["text-generation"],
          contextWindow: 8192,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: true,
      };

      await expect(registry.registerModel(request)).rejects.toThrow(
        ModelRegistryError
      );
      await expect(registry.registerModel(request)).rejects.toThrow(
        "Model name is required"
      );
    });

    it("should reject registration with missing version", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "test-model",
          type: "ollama",
          ollamaName: "gemma3:1b",
          version: "",
          category: "primary",
          capabilities: ["text-generation"],
          contextWindow: 8192,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: true,
      };

      await expect(registry.registerModel(request)).rejects.toThrow(
        "Model version is required"
      );
    });

    it("should reject registration with no capabilities", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "test-model",
          type: "ollama",
          ollamaName: "gemma3:1b",
          version: "1.0.0",
          category: "primary",
          capabilities: [],
          contextWindow: 8192,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: true,
      };

      await expect(registry.registerModel(request)).rejects.toThrow(
        "At least one capability is required"
      );
    });

    it("should reject registration with invalid context window", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "test-model",
          type: "ollama",
          ollamaName: "gemma3:1b",
          version: "1.0.0",
          category: "primary",
          capabilities: ["text-generation"],
          contextWindow: 0,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: true,
      };

      await expect(registry.registerModel(request)).rejects.toThrow(
        "Context window must be positive"
      );
    });

    it("should reject Ollama model without colon in name", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "test-model",
          type: "ollama",
          ollamaName: "gemma3-invalid",
          version: "1.0.0",
          category: "primary",
          capabilities: ["text-generation"],
          contextWindow: 8192,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: true,
      };

      await expect(registry.registerModel(request)).rejects.toThrow(
        "Ollama model name must include tag"
      );
    });

    it("should skip validation when validate=false", async () => {
      const request: ModelRegistrationRequest = {
        config: {
          name: "",
          type: "ollama",
          ollamaName: "invalid",
          version: "",
          category: "primary",
          capabilities: [],
          contextWindow: 0,
          supportsStreaming: true,
          supportsBatching: false,
          quantization: "4bit",
        },
        validate: false,
      };

      const model = await registry.registerModel(request);
      expect(model).toBeDefined();
    });
  });

  describe("Ollama Convenience Method", () => {
    it("should register Ollama model with convenience method", async () => {
      const model = await registry.registerOllamaModel(
        "gemma-fast",
        "gemma3:1b",
        "1.0.0",
        "fast"
      );

      expect(model).toBeDefined();
      expect(model.name).toBe("gemma-fast");
      expect(model.type).toBe("ollama");
      expect((model as OllamaModelConfig).ollamaName).toBe("gemma3:1b");
      expect(model.category).toBe("fast");
      expect(model.capabilities).toContain("text-generation");
      expect(model.contextWindow).toBe(8192);
    });

    it("should default to primary category", async () => {
      const model = await registry.registerOllamaModel(
        "gemma-default",
        "gemma3:4b",
        "1.0.0"
      );

      expect(model.category).toBe("primary");
    });
  });

  describe("Model Retrieval", () => {
    it("should get model by ID", async () => {
      const registered = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "1.0.0"
      );

      const retrieved = registry.getModel(registered.id);

      expect(retrieved).toBeDefined();
      expect(retrieved?.id).toBe(registered.id);
      expect(retrieved?.name).toBe("test-model");
    });

    it("should return undefined for non-existent model", () => {
      const retrieved = registry.getModel("non-existent-id");
      expect(retrieved).toBeUndefined();
    });

    it("should get all versions of a model", async () => {
      await registry.registerOllamaModel("test-model", "gemma3:1b", "1.0.0");
      await new Promise((resolve) => setTimeout(resolve, 1));
      await registry.registerOllamaModel("test-model", "gemma3:1b", "1.1.0");
      await new Promise((resolve) => setTimeout(resolve, 1));
      await registry.registerOllamaModel("test-model", "gemma3:1b", "2.0.0");

      const versions = registry.getModelVersions("test-model");

      expect(versions).toHaveLength(3);
      expect(versions[0].version).toBe("2.0.0"); // Latest first
      expect(versions[1].version).toBe("1.1.0");
      expect(versions[2].version).toBe("1.0.0");
    });

    it("should return empty array for non-existent model name", () => {
      const versions = registry.getModelVersions("non-existent");
      expect(versions).toEqual([]);
    });

    it("should get latest version of a model", async () => {
      await registry.registerOllamaModel("test-model", "gemma3:1b", "1.0.0");
      await new Promise((resolve) => setTimeout(resolve, 1));
      await registry.registerOllamaModel("test-model", "gemma3:1b", "2.0.0");

      const latest = registry.getLatestVersion("test-model");

      expect(latest).toBeDefined();
      expect(latest?.version).toBe("2.0.0");
    });

    it("should return undefined for non-existent model name", () => {
      const latest = registry.getLatestVersion("non-existent");
      expect(latest).toBeUndefined();
    });
  });

  describe("Model Querying", () => {
    beforeEach(async () => {
      // Register test models
      await registry.registerOllamaModel(
        "fast-model",
        "gemma3:1b",
        "1.0.0",
        "fast"
      );
      await registry.registerOllamaModel(
        "primary-model",
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

      // Activate some models
      const models = registry.getAllModels();
      await registry.activateModel(models[0].id);
      await registry.activateModel(models[1].id);
    });

    it("should query all models", () => {
      const results = registry.queryModels();
      expect(results).toHaveLength(3);
    });

    it("should filter by status", () => {
      const active = registry.queryModels({ status: "active" });
      const testing = registry.queryModels({ status: "testing" });

      expect(active).toHaveLength(2);
      expect(testing).toHaveLength(1);
    });

    it("should filter by type", () => {
      const ollama = registry.queryModels({ type: "ollama" });
      expect(ollama).toHaveLength(3);
    });

    it("should filter by capabilities", () => {
      const capable = registry.queryModels({
        capabilities: ["text-generation"],
      });

      expect(capable).toHaveLength(3);
    });

    it("should filter by category", () => {
      const fast = registry.queryModels({ category: "fast" });
      const primary = registry.queryModels({ category: "primary" });

      expect(fast).toHaveLength(1);
      expect(primary).toHaveLength(1);
    });

    it("should find models by capabilities", () => {
      const capable = registry.findByCapabilities(["text-generation", "chat"]);

      // Should only return active models
      expect(capable.length).toBeGreaterThan(0);
      capable.forEach((model) => {
        expect(model.status).toBe("active");
        expect(model.capabilities).toContain("text-generation");
        expect(model.capabilities).toContain("chat");
      });
    });

    it("should paginate results", () => {
      const page1 = registry.queryModels({ limit: 2, offset: 0 });
      const page2 = registry.queryModels({ limit: 2, offset: 2 });

      expect(page1).toHaveLength(2);
      expect(page2).toHaveLength(1);
    });

    it("should sort by name ascending", () => {
      const results = registry.queryModels({
        sortBy: "name",
        sortOrder: "asc",
      });

      expect(results[0].name).toBe("fast-model");
      expect(results[1].name).toBe("primary-model");
      expect(results[2].name).toBe("quality-model");
    });

    it("should sort by createdAt descending", () => {
      const results = registry.queryModels({
        sortBy: "createdAt",
        sortOrder: "desc",
      });

      // Most recent first
      expect(results[0].name).toBe("quality-model");
      expect(results[2].name).toBe("fast-model");
    });
  });

  describe("Model Updates", () => {
    it("should update model metadata", async () => {
      const model = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "1.0.0"
      );

      const updated = await registry.updateModel({
        modelId: model.id,
        updates: {
          description: "Updated description",
          tags: ["test", "updated"],
        },
      });

      expect(updated.description).toBe("Updated description");
      expect(updated.tags).toEqual(["test", "updated"]);
      expect(updated.updatedAt.getTime()).toBeGreaterThan(
        model.updatedAt.getTime()
      );
    });

    it("should throw error for non-existent model", async () => {
      await expect(
        registry.updateModel({
          modelId: "non-existent",
          updates: { description: "test" },
        })
      ).rejects.toThrow("Model not found");
    });
  });

  describe("Model Status Management", () => {
    it("should activate a model", async () => {
      const model = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "1.0.0"
      );

      expect(model.status).toBe("testing");

      const activated = await registry.activateModel(model.id);

      expect(activated.status).toBe("active");
    });

    it("should deprecate a model", async () => {
      const model = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "1.0.0"
      );

      const deprecated = await registry.deprecateModel(model.id);

      expect(deprecated.status).toBe("deprecated");
      expect(deprecated.deprecatedAt).toBeInstanceOf(Date);
    });

    it("should throw error when activating non-existent model", async () => {
      await expect(registry.activateModel("non-existent")).rejects.toThrow(
        "Model not found"
      );
    });

    it("should throw error when deprecating non-existent model", async () => {
      await expect(registry.deprecateModel("non-existent")).rejects.toThrow(
        "Model not found"
      );
    });
  });

  describe("Performance Profile Management", () => {
    it("should update performance profile", async () => {
      const model = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "1.0.0"
      );

      const profile: PerformanceProfile = {
        modelId: model.id,
        taskCategories: [
          {
            taskType: "code-generation",
            successRate: 0.95,
            avgLatency: 500,
            qualityScore: 0.9,
          },
        ],
        capabilities: {
          maxContextWindow: 8192,
          streamingSupport: true,
          batchingSupport: false,
        },
        resourceUsage: {
          avgMemoryMB: 1024,
          avgCPUPercent: 50,
        },
        capturedAt: new Date(),
      };

      await registry.updatePerformanceProfile(model.id, profile);

      const retrieved = registry.getPerformanceProfile(model.id);

      expect(retrieved).toBeDefined();
      expect(retrieved?.modelId).toBe(model.id);
      expect(retrieved?.taskCategories).toHaveLength(1);
      expect(retrieved?.taskCategories[0].qualityScore).toBe(0.9);
    });

    it("should return undefined for non-existent profile", () => {
      const profile = registry.getPerformanceProfile("non-existent");
      expect(profile).toBeUndefined();
    });

    it("should throw error when updating profile for non-existent model", async () => {
      const profile: PerformanceProfile = {
        modelId: "non-existent",
        taskCategories: [],
        capabilities: {
          maxContextWindow: 8192,
          streamingSupport: true,
          batchingSupport: false,
        },
        resourceUsage: {
          avgMemoryMB: 1024,
          avgCPUPercent: 50,
        },
        capturedAt: new Date(),
      };

      await expect(
        registry.updatePerformanceProfile("non-existent", profile)
      ).rejects.toThrow("Model not found");
    });
  });

  describe("Utility Methods", () => {
    it("should get all models", async () => {
      await registry.registerOllamaModel("model1", "gemma3:1b", "1.0.0");
      await registry.registerOllamaModel("model2", "gemma3:4b", "1.0.0");

      const all = registry.getAllModels();

      expect(all).toHaveLength(2);
    });

    it("should get active models only", async () => {
      const model1 = await registry.registerOllamaModel(
        "model1",
        "gemma3:1b",
        "1.0.0"
      );
      const model2 = await registry.registerOllamaModel(
        "model2",
        "gemma3:4b",
        "1.0.0"
      );

      await registry.activateModel(model1.id);

      const active = registry.getActiveModels();

      expect(active).toHaveLength(1);
      expect(active[0].id).toBe(model1.id);
    });

    it("should check if model exists", async () => {
      const model = await registry.registerOllamaModel(
        "test-model",
        "gemma3:1b",
        "1.0.0"
      );

      expect(registry.hasModel(model.id)).toBe(true);
      expect(registry.hasModel("non-existent")).toBe(false);
    });

    it("should get model count", async () => {
      expect(registry.getModelCount()).toBe(0);

      await registry.registerOllamaModel("model1", "gemma3:1b", "1.0.0");
      expect(registry.getModelCount()).toBe(1);

      await registry.registerOllamaModel("model2", "gemma3:4b", "1.0.0");
      expect(registry.getModelCount()).toBe(2);
    });

    it("should clear all models", async () => {
      await registry.registerOllamaModel("model1", "gemma3:1b", "1.0.0");
      await registry.registerOllamaModel("model2", "gemma3:4b", "1.0.0");

      expect(registry.getModelCount()).toBe(2);

      registry.clear();

      expect(registry.getModelCount()).toBe(0);
      expect(registry.getAllModels()).toEqual([]);
    });
  });
});
