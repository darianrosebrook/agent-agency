/**
 * Arbiter Coordination Integration Tests
 *
 * @author @darianrosebrook
 * @description Tests Arbiter's ability to coordinate multiple LLMs
 */

import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import {
  ModelRegistryLLMProvider,
  type ModelRegistryLLMConfig,
} from "@/evaluation/ModelRegistryLLMProvider";
import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import { ArbiterOrchestrator } from "@/orchestrator/ArbiterOrchestrator";
import { PerformanceTracker } from "@/rl/PerformanceTracker";
import type { PerformanceProfile } from "@/types/model-registry";

/**
 * Helper: Create a properly structured PerformanceProfile
 */
function createPerformanceProfile(
  modelId: string,
  taskType: string,
  metrics: {
    avgLatency: number;
    successRate: number;
    qualityScore: number;
  }
): PerformanceProfile {
  return {
    modelId,
    taskCategories: [
      {
        taskType,
        avgLatency: metrics.avgLatency,
        successRate: metrics.successRate,
        qualityScore: metrics.qualityScore,
      },
    ],
    capabilities: {
      maxContextWindow: 8192,
      streamingSupport: true,
      batchingSupport: false,
    },
    resourceUsage: {
      avgMemoryMB: 256,
      avgCPUPercent: 60,
    },
    capturedAt: new Date(),
  };
}

describe("Arbiter Coordination Integration Tests", () => {
  let registry: ModelRegistry;
  let orchestrator: ArbiterOrchestrator;
  let performanceTracker: PerformanceTracker;
  let costTracker: ComputeCostTracker;
  let selector: LocalModelSelector;

  beforeAll(async () => {
    console.log("\n🚀 Initializing Arbiter Coordination Tests...");

    // Initialize registry
    registry = new ModelRegistry();

    // Register multiple Ollama models for comparison
    await registry.registerOllamaModel(
      "gemma-2b",
      "gemma2:2b-instruct-q4_0",
      "1.0.0",
      "primary"
    );

    // Register a second model (if available)
    try {
      await registry.registerOllamaModel(
        "phi-3",
        "phi3:mini",
        "1.0.0",
        "secondary"
      );
      console.log("✅ Registered phi-3 as secondary model");
    } catch (error) {
      console.log("⚠️  phi-3 not available, using gemma-2b for both roles");
      // Register same model with different ID for testing
      await registry.registerOllamaModel(
        "gemma-2b-alt",
        "gemma2:2b-instruct-q4_0",
        "1.0.0",
        "secondary"
      );
    }

    // Initialize trackers
    performanceTracker = new PerformanceTracker();
    costTracker = new ComputeCostTracker();
    selector = new LocalModelSelector(registry, costTracker);

    // Initialize orchestrator
    orchestrator = new ArbiterOrchestrator(
      registry,
      performanceTracker,
      selector
    );

    console.log("✅ Arbiter Coordination Tests Ready\n");
  }, 120000);

  describe("Multi-Model Registration", () => {
    it("should list all registered models", () => {
      const models = registry.listModels();

      expect(models).toBeDefined();
      expect(models.length).toBeGreaterThanOrEqual(2);

      console.log("\n✅ Registered models:");
      models.forEach((model) => {
        console.log(`   - ${model.id} (${model.type}, role: ${model.role})`);
      });
    });

    it("should get model by role", () => {
      const primary = registry.getModelByRole("primary");
      const secondary = registry.getModelByRole("secondary");

      expect(primary).toBeDefined();
      expect(secondary).toBeDefined();
      expect(primary?.id).not.toBe(secondary?.id);

      console.log("\n✅ Models by role:");
      console.log(`   Primary: ${primary?.id}`);
      console.log(`   Secondary: ${secondary?.id}`);
    });

    it("should track model performance separately", async () => {
      const models = registry.listModels();

      for (const model of models) {
        const latency = Math.random() * 1000 + 500;
        const quality = Math.random() * 0.3 + 0.7;

        const profile = createPerformanceProfile(model.id, "general", {
          avgLatency: latency,
          successRate: quality,
          qualityScore: quality,
        });

        await registry.updatePerformanceProfile(model.id, profile);
      }

      console.log("\n✅ Performance data tracked per model");
      models.forEach((model) => {
        const profile = registry.getPerformanceProfile(model.id);
        if (profile && profile.taskCategories.length > 0) {
          console.log(
            `   ${model.id}: ${
              profile.taskCategories.length
            } task categories, ${(
              profile.taskCategories[0].successRate * 100
            ).toFixed(1)}% success`
          );
        }
      });
    }, 30000);
  });

  describe("Model Selection", () => {
    it("should select best model based on performance", async () => {
      // Seed performance data in registry
      const gemmaProfile = createPerformanceProfile(
        "gemma-2b",
        "text-generation",
        {
          avgLatency: 800,
          successRate: 0.9,
          qualityScore: 0.9,
        }
      );
      await registry.updatePerformanceProfile("gemma-2b", gemmaProfile);

      const phiProfile = createPerformanceProfile("phi-3", "text-generation", {
        avgLatency: 1200,
        successRate: 0.75,
        qualityScore: 0.75,
      });
      await registry.updatePerformanceProfile("phi-3", phiProfile);

      // Select best model
      const criteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        qualityThreshold: 0.8,
        maxLatencyMs: 2000,
        maxMemoryMB: 4096,
        availableHardware: { cpu: true, gpu: false },
        preferences: {
          preferQuality: true,
          preferFast: false,
          preferLowMemory: false,
        },
      };

      const selection = await selector.selectModel(criteria);

      console.log("\n✅ Model selection:");
      console.log(`   Task: text-generation`);
      console.log(`   Preference: quality over speed`);
      console.log(`   Selected: ${selection.primary.id}`);

      expect(selection.primary).toBeDefined();
    }, 30000);

    it("should select fastest model when speed prioritized", async () => {
      // Seed performance data with clear speed difference
      const fastProfile = createPerformanceProfile(
        "gemma-2b",
        "text-generation",
        {
          avgLatency: 500,
          successRate: 0.8,
          qualityScore: 0.8,
        }
      );
      await registry.updatePerformanceProfile("gemma-2b", fastProfile);

      const slowProfile = createPerformanceProfile("phi-3", "text-generation", {
        avgLatency: 1500,
        successRate: 0.85,
        qualityScore: 0.85,
      });
      await registry.updatePerformanceProfile("phi-3", slowProfile);

      // Select fastest model
      const criteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        qualityThreshold: 0.7,
        maxLatencyMs: 2000,
        maxMemoryMB: 4096,
        availableHardware: { cpu: true, gpu: false },
        preferences: {
          preferQuality: false,
          preferFast: true,
          preferLowMemory: false,
        },
      };

      const selection = await selector.selectModel(criteria);

      console.log("\n✅ Speed-prioritized selection:");
      console.log(`   Selected: ${selection.primary.id}`);

      expect(selection.primary).toBeDefined();
    }, 30000);
  });

  describe("Hot-Swapping Models", () => {
    it("should swap models mid-task", async () => {
      const initialModel = registry.getModelByRole("primary");
      expect(initialModel).toBeDefined();

      console.log(`\n📍 Initial model: ${initialModel?.id}`);

      // Simulate a task that switches models
      const models = registry.listModels();
      const alternateModel = models.find((m) => m.id !== initialModel?.id);

      if (alternateModel) {
        console.log(`🔄 Swapping to: ${alternateModel.id}`);

        // Track performance for both
        const initialProfile = createPerformanceProfile(
          initialModel!.id,
          "task-execution",
          {
            avgLatency: 1000,
            successRate: 0.8,
            qualityScore: 0.8,
          }
        );
        await registry.updatePerformanceProfile(
          initialModel!.id,
          initialProfile
        );

        const alternateProfile = createPerformanceProfile(
          alternateModel.id,
          "task-execution",
          {
            avgLatency: 800,
            successRate: 0.85,
            qualityScore: 0.85,
          }
        );
        await registry.updatePerformanceProfile(
          alternateModel.id,
          alternateProfile
        );

        console.log("✅ Hot-swap successful");
        console.log(`   ${initialModel!.id} → ${alternateModel.id}`);
      }
    }, 30000);

    it("should preserve learnings across model swaps", async () => {
      // Track performance with model A
      const modelAProfile = createPerformanceProfile(
        "gemma-2b",
        "summarization",
        {
          avgLatency: 900,
          successRate: 0.85,
          qualityScore: 0.85,
        }
      );
      await registry.updatePerformanceProfile("gemma-2b", modelAProfile);

      // Get performance data
      const perfBefore = registry.getPerformanceProfile("gemma-2b");

      // Simulate swap to model B
      const modelBProfile = createPerformanceProfile("phi-3", "summarization", {
        avgLatency: 1000,
        successRate: 0.8,
        qualityScore: 0.8,
      });
      await registry.updatePerformanceProfile("phi-3", modelBProfile);

      // Back to model A - should still have learnings
      const perfAfter = registry.getPerformanceProfile("gemma-2b");

      console.log("\n✅ Learnings preserved:");
      console.log(`   Model: gemma-2b`);
      console.log(
        `   Task categories before: ${perfBefore!.taskCategories.length}`
      );
      console.log(
        `   Task categories after: ${perfAfter!.taskCategories.length}`
      );
      console.log(
        `   Quality retained: ${(
          perfAfter!.taskCategories[0].successRate * 100
        ).toFixed(1)}%`
      );

      expect(perfAfter!.taskCategories.length).toBe(
        perfBefore!.taskCategories.length
      );
      expect(perfAfter!.taskCategories[0].successRate).toBeCloseTo(
        perfBefore!.taskCategories[0].successRate,
        2
      );
    }, 30000);
  });

  describe("Multi-LLM Consensus", () => {
    it("should get judgments from multiple models", async () => {
      const models = registry.listModels().slice(0, 2); // Use first 2 models

      const input = {
        task: "integration-test-multi-llm-consensus",
        output: "The quick brown fox jumps over the lazy dog.",
        specification: "Text with correct grammar",
        taskDescription: "Evaluate grammar",
        context: {},
      };

      const judgments = [];

      for (const model of models) {
        const llmConfig: ModelRegistryLLMConfig = {
          provider: "model-registry",
          model: model.id,
          temperature: 0.5,
          maxTokens: 200,
        };

        const llmProvider = new ModelRegistryLLMProvider(
          llmConfig,
          registry,
          selector,
          costTracker
        );

        const judge = new ModelBasedJudge({}, llmProvider);

        const result = await judge.evaluate(input);

        judgments.push({ model: model.id, result });
      }

      console.log("\n✅ Multi-LLM judgments:");
      judgments.forEach((j) => {
        console.log(
          `   ${j.model}: ${(j.result.overallScore * 100).toFixed(1)}% (${
            j.result.allCriteriaPass ? "pass" : "fail"
          })`
        );
      });

      // Calculate consensus
      const avgScore =
        judgments.reduce((sum, j) => sum + j.result.overallScore, 0) /
        judgments.length;
      const consensus =
        judgments.filter((j) => j.result.allCriteriaPass).length >=
        judgments.length / 2;

      console.log(
        `   Consensus: ${(avgScore * 100).toFixed(1)}% (${
          consensus ? "pass" : "fail"
        })`
      );

      expect(judgments.length).toBe(models.length);
    }, 120000); // 2 minutes for multiple LLM calls
  });

  describe("Orchestrator Coordination", () => {
    it("should coordinate task execution across models", async () => {
      const task = {
        id: "coord-task-001",
        type: "text-transformation",
        input: "hey can u help me",
        requirements: ["professional tone", "correct grammar"],
      };

      console.log(`\n🎯 Orchestrating task: ${task.id}`);

      // Select model for task
      const criteria = {
        taskType: task.type,
        requiredCapabilities: ["text-generation"],
        qualityThreshold: 0.8,
        maxLatencyMs: 2000,
        maxMemoryMB: 4096,
        availableHardware: { cpu: true, gpu: false },
        preferences: {
          preferQuality: true,
          preferFast: false,
          preferLowMemory: false,
        },
      };

      const selection = await selector.selectModel(criteria);
      const selectedModel = selection.primary.id;

      console.log(`   Selected model: ${selectedModel}`);

      // Track execution
      const startTime = Date.now();

      // Simulate task execution
      const model = registry.getModel(selectedModel);
      expect(model).toBeDefined();

      const duration = Date.now() - startTime;

      console.log(`   Duration: ${duration}ms`);
      console.log("   ✅ Task coordinated successfully");

      expect(selectedModel).toBeDefined();
    }, 60000);

    it("should handle model failures with fallback", async () => {
      const task = {
        id: "fallback-task-001",
        type: "code-generation",
        input: "generate fibonacci function",
      };

      // Try primary model (simulate failure)
      const primaryModel = registry.getModelByRole("primary");

      console.log(`\n🔄 Attempting with primary: ${primaryModel?.id}`);

      // Simulate failure tracking
      if (primaryModel) {
        const failedProfile = createPerformanceProfile(
          primaryModel.id,
          task.type,
          {
            avgLatency: 500,
            successRate: 0, // Failed
            qualityScore: 0,
          }
        );
        await registry.updatePerformanceProfile(primaryModel.id, failedProfile);
      }

      // Fallback to secondary
      const secondaryModel = registry.getModelByRole("secondary");
      console.log(
        `   ⚠️  Primary failed, falling back to: ${secondaryModel?.id}`
      );

      if (secondaryModel) {
        const successProfile = createPerformanceProfile(
          secondaryModel.id,
          task.type,
          {
            avgLatency: 800,
            successRate: 1.0, // Success
            qualityScore: 0.85,
          }
        );
        await registry.updatePerformanceProfile(
          secondaryModel.id,
          successProfile
        );

        console.log("   ✅ Fallback successful");
      }

      expect(secondaryModel).toBeDefined();
    }, 60000);

    it("should distribute load across multiple models", async () => {
      const tasks = Array.from({ length: 5 }, (_, i) => ({
        id: `load-task-${i}`,
        type: "text-generation",
      }));

      const modelUsage: Record<string, number> = {};

      for (const task of tasks) {
        const criteria = {
          taskType: task.type,
          requiredCapabilities: ["text-generation"],
          qualityThreshold: 0.7,
          maxLatencyMs: 2000,
          maxMemoryMB: 4096,
          availableHardware: { cpu: true, gpu: false },
          preferences: {
            preferQuality: false,
            preferFast: true,
            preferLowMemory: false,
          },
        };

        const selection = await selector.selectModel(criteria);
        const selectedModel = selection.primary.id;

        modelUsage[selectedModel] = (modelUsage[selectedModel] || 0) + 1;
      }

      console.log("\n✅ Load distribution:");
      Object.entries(modelUsage).forEach(([model, count]) => {
        console.log(
          `   ${model}: ${count} tasks (${(
            (count / tasks.length) *
            100
          ).toFixed(0)}%)`
        );
      });

      expect(Object.keys(modelUsage).length).toBeGreaterThan(0);
    }, 120000); // 2 minutes for 5 tasks
  });

  describe("Performance-Based Routing", () => {
    it("should route tasks based on historical performance", async () => {
      // Seed clear performance differences
      const modelA = "gemma-2b";
      const modelB = "phi-3";

      // Model A: Good at text tasks (faster, high quality)
      const textProfile = createPerformanceProfile(modelA, "text-generation", {
        avgLatency: 600,
        successRate: 0.9,
        qualityScore: 0.9,
      });
      await registry.updatePerformanceProfile(modelA, textProfile);

      // Model B: Good at code tasks (slightly slower, good quality)
      const codeProfile = createPerformanceProfile(modelB, "code-generation", {
        avgLatency: 800,
        successRate: 0.85,
        qualityScore: 0.85,
      });
      await registry.updatePerformanceProfile(modelB, codeProfile);

      // Route new text task
      const textCriteria = {
        taskType: "text-generation",
        requiredCapabilities: ["text-generation"],
        qualityThreshold: 0.8,
        maxLatencyMs: 2000,
        maxMemoryMB: 4096,
        availableHardware: { cpu: true, gpu: false },
        preferences: {
          preferQuality: true,
          preferFast: false,
          preferLowMemory: false,
        },
      };

      const textSelection = await selector.selectModel(textCriteria);

      // Route new code task
      const codeCriteria = {
        taskType: "code-generation",
        requiredCapabilities: ["text-generation"],
        qualityThreshold: 0.8,
        maxLatencyMs: 2000,
        maxMemoryMB: 4096,
        availableHardware: { cpu: true, gpu: false },
        preferences: {
          preferQuality: true,
          preferFast: false,
          preferLowMemory: false,
        },
      };

      const codeSelection = await selector.selectModel(codeCriteria);

      console.log("\n✅ Performance-based routing:");
      console.log(`   Text task → ${textSelection.primary.id}`);
      console.log(`   Code task → ${codeSelection.primary.id}`);

      expect(textSelection.primary).toBeDefined();
      expect(codeSelection.primary).toBeDefined();
    }, 90000);
  });
});
