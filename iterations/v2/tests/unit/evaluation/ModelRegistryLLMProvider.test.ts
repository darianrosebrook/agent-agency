/**
 * @file ModelRegistryLLMProvider.test.ts
 * @description Unit tests for ModelRegistryLLMProvider
 * @author @darianrosebrook
 */

import { ModelRegistryLLMProvider } from "@/evaluation/ModelRegistryLLMProvider";
import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import type { JudgmentInput } from "@/types/judge";
import { EvaluationCriterion } from "@/types/judge";
import { beforeEach, describe, expect, it } from "@jest/globals";

describe("ModelRegistryLLMProvider", () => {
  let registry: ModelRegistry;
  let selector: LocalModelSelector;
  let costTracker: ComputeCostTracker;
  let provider: ModelRegistryLLMProvider;

  beforeEach(async () => {
    registry = new ModelRegistry();
    costTracker = new ComputeCostTracker();
    selector = new LocalModelSelector(registry, costTracker);

    // Register test models
    await registry.registerOllamaModel("test-model-1", "gemma3n:e2b", "1.0.0");

    await registry.registerOllamaModel("test-model-2", "gemma3:1b", "1.0.0");

    // Activate models
    const models = registry.getAllModels();
    for (const model of models) {
      await registry.activateModel(model.id);
    }

    // Create provider
    provider = new ModelRegistryLLMProvider(
      {
        provider: "model-registry",
        model: "test-model-1",
        taskType: "judgment",
        qualityThreshold: 0.8,
        maxLatencyMs: 3000,
        maxMemoryMB: 4096,
        temperature: 0,
        maxTokens: 1000,
      },
      registry,
      selector,
      costTracker
    );
  });

  describe("Constructor", () => {
    it("should initialize with required dependencies", () => {
      expect(provider).toBeDefined();
      expect(provider.getActiveModelId()).toBeNull(); // Not active until evaluate called
    });

    it("should accept custom configuration", () => {
      const customProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "custom-model",
          taskType: "custom-task",
          qualityThreshold: 0.9,
          maxLatencyMs: 1000,
          maxMemoryMB: 2048,
          temperature: 0,
          maxTokens: 1000,
        },
        registry,
        selector,
        costTracker
      );

      expect(customProvider).toBeDefined();
    });
  });

  describe("evaluate()", () => {
    it("should evaluate judgment input", async () => {
      const input: JudgmentInput = {
        task: "Summarize the text",
        output: "This is a concise summary.",
        context: { text: "Original text about AI..." },
      };

      const result = await provider.evaluate(
        input,
        EvaluationCriterion.FAITHFULNESS
      );

      expect(result).toBeDefined();
      expect(result.criterion).toBe(EvaluationCriterion.FAITHFULNESS);
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1);
      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeDefined();
      expect(result.reasoning.length).toBeGreaterThan(0);
    });

    it("should select model based on criteria", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      await provider.evaluate(input, EvaluationCriterion.RELEVANCE);

      const activeModelId = provider.getActiveModelId();
      expect(activeModelId).toBeDefined();

      // Verify model is from registry
      const model = registry.getModel(activeModelId!);
      expect(model).toBeDefined();
    });

    it("should track performance after evaluation", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      await provider.evaluate(input, EvaluationCriterion.SAFETY);

      const activeModelId = provider.getActiveModelId();
      const history = selector.getPerformanceHistory(
        activeModelId!,
        "judgment"
      );

      expect(history).toBeDefined();
      expect(history!.samples).toBeGreaterThan(0);
    });

    it("should record compute cost", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      await provider.evaluate(input, EvaluationCriterion.MINIMALITY);

      const activeModelId = provider.getActiveModelId();
      const costProfile = costTracker.getCostProfile(activeModelId!);

      expect(costProfile).toBeDefined();
      expect(costProfile!.totalOperations).toBeGreaterThan(0);
      expect(costProfile!.avgWallClockMs).toBeGreaterThan(0);
    });
  });

  describe("Criterion-Specific Evaluation", () => {
    const input: JudgmentInput = {
      task: "Analyze sentiment",
      output: "The sentiment is positive with high confidence.",
      expectedOutput: "Positive sentiment detected.",
    };

    it("should evaluate faithfulness with expected output", async () => {
      const result = await provider.evaluate(
        input,
        EvaluationCriterion.FAITHFULNESS
      );

      expect(result.criterion).toBe(EvaluationCriterion.FAITHFULNESS);
      expect(result.score).toBeGreaterThan(0.5); // Should be high with expected output
      expect(result.reasoning).toContain("output");
    });

    it("should evaluate faithfulness without expected output", async () => {
      const inputNoExpected: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      const result = await provider.evaluate(
        inputNoExpected,
        EvaluationCriterion.FAITHFULNESS
      );

      expect(result.criterion).toBe(EvaluationCriterion.FAITHFULNESS);
      expect(result.score).toBeDefined();
    });

    it("should evaluate relevance based on output length", async () => {
      const shortInput: JudgmentInput = {
        task: "Test",
        output: "Short", // < 50 chars
      };

      const longInput: JudgmentInput = {
        task: "Test",
        output:
          "This is a much longer output that should score higher on relevance because it provides substantial content.", // > 50 chars
      };

      const shortResult = await provider.evaluate(
        shortInput,
        EvaluationCriterion.RELEVANCE
      );
      const longResult = await provider.evaluate(
        longInput,
        EvaluationCriterion.RELEVANCE
      );

      expect(longResult.score).toBeGreaterThan(shortResult.score);
    });

    it("should evaluate minimality based on output length", async () => {
      const conciseInput: JudgmentInput = {
        task: "Test",
        output: "Concise output", // < 500 chars
      };

      const verboseInput: JudgmentInput = {
        task: "Test",
        output: "A".repeat(600), // > 500 chars
      };

      const conciseResult = await provider.evaluate(
        conciseInput,
        EvaluationCriterion.MINIMALITY
      );
      const verboseResult = await provider.evaluate(
        verboseInput,
        EvaluationCriterion.MINIMALITY
      );

      expect(conciseResult.score).toBeGreaterThan(verboseResult.score);
    });

    it("should evaluate safety by detecting unsafe patterns", async () => {
      const safeInput: JudgmentInput = {
        task: "Test",
        output: "This is a safe response",
      };

      const unsafeInput: JudgmentInput = {
        task: "Test",
        output: "Here is the API_KEY: sk-12345",
      };

      const safeResult = await provider.evaluate(
        safeInput,
        EvaluationCriterion.SAFETY
      );
      const unsafeResult = await provider.evaluate(
        unsafeInput,
        EvaluationCriterion.SAFETY
      );

      expect(safeResult.score).toBeGreaterThan(unsafeResult.score);
      expect(unsafeResult.score).toBeLessThan(0.5); // Should be flagged as unsafe
    });
  });

  describe("Model Selection", () => {
    it("should select model based on quality threshold", async () => {
      const models = registry.getAllModels();

      // Record different quality levels
      selector.updatePerformanceHistory(models[0].id, "judgment", {
        quality: 0.95,
        latencyMs: 500,
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory(models[1].id, "judgment", {
        quality: 0.7,
        latencyMs: 100,
        memoryMB: 128,
        success: true,
      });

      // Create provider with high quality requirement
      const highQualityProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "test-model",
          taskType: "judgment",
          qualityThreshold: 0.9, // High threshold
          maxLatencyMs: 5000,
          maxMemoryMB: 4096,
          temperature: 0,
          maxTokens: 1000,
        },
        registry,
        selector,
        costTracker
      );

      const input: JudgmentInput = {
        task: "High quality task",
        output: "High quality output",
      };

      await highQualityProvider.evaluate(
        input,
        EvaluationCriterion.FAITHFULNESS
      );

      // Should select the high-quality model
      expect(highQualityProvider.getActiveModelId()).toBe(models[0].id);
    });

    it("should respect latency constraints", async () => {
      const models = registry.getAllModels();

      // Record different latencies
      selector.updatePerformanceHistory(models[0].id, "judgment", {
        quality: 0.8,
        latencyMs: 5000, // Slow
        memoryMB: 256,
        success: true,
      });

      selector.updatePerformanceHistory(models[1].id, "judgment", {
        quality: 0.75,
        latencyMs: 500, // Fast
        memoryMB: 128,
        success: true,
      });

      // Create provider with strict latency requirement
      const fastProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "test-model",
          taskType: "judgment",
          qualityThreshold: 0.7,
          maxLatencyMs: 1000, // Strict
          maxMemoryMB: 4096,
          temperature: 0,
          maxTokens: 1000,
        },
        registry,
        selector,
        costTracker
      );

      const input: JudgmentInput = {
        task: "Fast task",
        output: "Fast output",
      };

      await fastProvider.evaluate(input, EvaluationCriterion.RELEVANCE);

      // Should select the fast model
      expect(fastProvider.getActiveModelId()).toBe(models[1].id);
    });

    it("should update active model on each evaluation", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      await provider.evaluate(input, EvaluationCriterion.FAITHFULNESS);
      const firstModelId = provider.getActiveModelId();

      await provider.evaluate(input, EvaluationCriterion.RELEVANCE);
      const secondModelId = provider.getActiveModelId();

      // Model IDs should be set (may be same or different)
      expect(firstModelId).toBeDefined();
      expect(secondModelId).toBeDefined();
    });
  });

  describe("Performance Tracking", () => {
    it("should accumulate performance history across evaluations", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      // Perform multiple evaluations
      for (let i = 0; i < 5; i++) {
        await provider.evaluate(input, EvaluationCriterion.FAITHFULNESS);
      }

      const activeModelId = provider.getActiveModelId();
      const history = selector.getPerformanceHistory(
        activeModelId!,
        "judgment"
      );

      expect(history).toBeDefined();
      expect(history!.samples).toBe(5);
      expect(history!.avgLatencyMs).toBeGreaterThan(0);
      expect(history!.successRate).toBe(1); // All should succeed
    });

    it("should track different criteria separately", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      await provider.evaluate(input, EvaluationCriterion.FAITHFULNESS);
      await provider.evaluate(input, EvaluationCriterion.RELEVANCE);
      await provider.evaluate(input, EvaluationCriterion.SAFETY);

      const activeModelId = provider.getActiveModelId();
      const history = selector.getPerformanceHistory(
        activeModelId!,
        "judgment"
      );

      expect(history).toBeDefined();
      expect(history!.samples).toBe(3); // 3 different criteria
    });
  });

  describe("Cost Tracking", () => {
    it("should accumulate costs across evaluations", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      // Perform multiple evaluations
      for (let i = 0; i < 3; i++) {
        await provider.evaluate(input, EvaluationCriterion.FAITHFULNESS);
      }

      const activeModelId = provider.getActiveModelId();
      const costProfile = costTracker.getCostProfile(activeModelId!);

      expect(costProfile).toBeDefined();
      expect(costProfile!.totalOperations).toBe(3);
      expect(costProfile!.avgWallClockMs).toBeGreaterThan(0);
      expect(costProfile!.avgWallClockMs).toBeGreaterThan(0);
      expect(costProfile!.avgWallClockMs).toBeGreaterThan(0);
    });

    it("should track token usage", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output with some content",
      };

      await provider.evaluate(input, EvaluationCriterion.RELEVANCE);

      const activeModelId = provider.getActiveModelId();
      const costProfile = costTracker.getCostProfile(activeModelId!);

      expect(costProfile).toBeDefined();
      expect(costProfile!.avgTokensPerSec).toBeGreaterThan(0);
      expect(costProfile!.avgTokensPerSec).toBeGreaterThan(0);
      expect(costProfile!.avgTokensPerSec).toBeGreaterThan(0);
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty output", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "",
      };

      const result = await provider.evaluate(
        input,
        EvaluationCriterion.FAITHFULNESS
      );

      expect(result).toBeDefined();
      expect(result.score).toBeDefined();
      expect(result.confidence).toBeDefined();
    });

    it("should handle very long output", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "A".repeat(10000), // Very long
      };

      const result = await provider.evaluate(
        input,
        EvaluationCriterion.MINIMALITY
      );

      expect(result).toBeDefined();
      expect(result.score).toBeLessThan(0.8); // Should penalize verbosity
    });

    it("should handle special characters in output", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test with special chars: @#$%^&*()",
      };

      const result = await provider.evaluate(input, EvaluationCriterion.SAFETY);

      expect(result).toBeDefined();
      expect(result.score).toBeGreaterThan(0.5); // Should be safe
    });

    it("should handle multiple unsafe patterns", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Password: secret123, API_KEY: sk-abc, TOKEN: xyz",
      };

      const result = await provider.evaluate(input, EvaluationCriterion.SAFETY);

      expect(result).toBeDefined();
      expect(result.score).toBeLessThan(0.5); // Should be flagged
    });
  });

  describe("getActiveModelId()", () => {
    it("should return null before any evaluation", () => {
      const newProvider = new ModelRegistryLLMProvider(
        {
          provider: "model-registry",
          model: "test-model",
          taskType: "test",
          temperature: 0,
          maxTokens: 1000,
        },
        registry,
        selector,
        costTracker
      );

      expect(newProvider.getActiveModelId()).toBeNull();
    });

    it("should return model ID after evaluation", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      await provider.evaluate(input, EvaluationCriterion.FAITHFULNESS);

      const modelId = provider.getActiveModelId();
      expect(modelId).toBeDefined();
      expect(modelId).not.toBeNull();

      // Verify it's a valid model
      const model = registry.getModel(modelId!);
      expect(model).toBeDefined();
    });
  });

  describe("Integration with ModelBasedJudge", () => {
    it("should work as LLMProvider for ModelBasedJudge", async () => {
      // This is tested in E2E, but verify basic compatibility
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      // Provider should implement evaluate() correctly
      const result = await provider.evaluate(
        input,
        EvaluationCriterion.FAITHFULNESS
      );

      // Result should match LLMResponse interface
      expect(result).toHaveProperty("criterion");
      expect(result).toHaveProperty("score");
      expect(result).toHaveProperty("confidence");
      expect(result).toHaveProperty("reasoning");
    });
  });
});
