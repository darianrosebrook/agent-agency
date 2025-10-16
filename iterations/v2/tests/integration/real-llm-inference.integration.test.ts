/**
 * Real LLM Inference Integration Tests
 *
 * @author @darianrosebrook
 * @description Tests using actual Ollama inference (no mocks)
 */

import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import {
  ModelRegistryLLMProvider,
  type ModelRegistryLLMConfig,
} from "@/evaluation/ModelRegistryLLMProvider";
import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import { OllamaProvider } from "@/models/providers/OllamaProvider";
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

describe("Real LLM Inference Integration Tests", () => {
  let registry: ModelRegistry;
  let ollamaProvider: OllamaProvider;
  let _performanceTracker: PerformanceTracker;
  let costTracker: ComputeCostTracker;
  let registeredModelId: string;

  beforeAll(async () => {
    console.log("\n🚀 Initializing Real LLM Integration Tests...");

    // Initialize registry
    registry = new ModelRegistry();

    // Register Ollama model (using available gemma3n:e2b)
    const model = await registry.registerOllamaModel(
      "gemma3n-e2b",
      "gemma3n:e2b",
      "1.0.0",
      "primary"
    );

    // Verify model was registered
    if (!model || model.type !== "ollama") {
      throw new Error("Failed to register Ollama model");
    }

    // Store model ID for use in tests
    registeredModelId = model.id;
    console.log(`✅ Registered model: ${registeredModelId}`);

    // Create provider from model config
    ollamaProvider = new OllamaProvider({
      id: model.id,
      name: model.name,
      type: "ollama",
      ollamaName: model.ollamaName,
      version: model.version,
      category: model.category,
      capabilities: model.capabilities,
      contextWindow: model.contextWindow,
      supportsStreaming: model.supportsStreaming,
      supportsBatching: model.supportsBatching,
      quantization: model.quantization,
      endpoint: model.endpoint || "http://localhost:11434",
      createdAt: model.createdAt,
      updatedAt: model.updatedAt,
      status: model.status,
    });

    // Initialize trackers
    performanceTracker = new PerformanceTracker();
    costTracker = new ComputeCostTracker();

    console.log("✅ Real LLM Integration Tests Ready\n");
  }, 60000);

  describe("Basic Ollama Inference", () => {
    it("should generate text with Ollama", async () => {
      const prompt = "Say hello in one sentence.";

      const startTime = Date.now();
      const response = await ollamaProvider.generate({
        prompt,
        temperature: 0.7,
        maxTokens: 50,
      });
      const duration = Date.now() - startTime;

      expect(response).toBeDefined();
      expect(response.text).toBeDefined();
      expect(response.text.length).toBeGreaterThan(0);

      console.log(`\n✅ Generated response in ${duration}ms`);
      console.log(`   Prompt: "${prompt}"`);
      console.log(`   Response: "${response.text.substring(0, 100)}..."`);
    }, 30000);

    it("should handle different temperatures", async () => {
      const prompt = "Complete this: The sky is";

      // Low temperature (more deterministic)
      const response1 = await ollamaProvider.generate({
        prompt,
        temperature: 0.1,
        maxTokens: 20,
      });

      // High temperature (more creative)
      const response2 = await ollamaProvider.generate({
        prompt,
        temperature: 0.9,
        maxTokens: 20,
      });

      expect(response1).toBeDefined();
      expect(response2).toBeDefined();

      console.log("\n✅ Temperature control working");
      console.log(`   Low temp (0.1): "${response1.text}"`);
      console.log(`   High temp (0.9): "${response2.text}"`);
    }, 60000);

    it("should track compute costs", async () => {
      const prompt = "Count to 5.";

      const startTime = Date.now();
      const response = await ollamaProvider.generate({
        prompt,
        temperature: 0.7,
        maxTokens: 30,
      });
      const duration = Date.now() - startTime;

      // Record compute cost
      costTracker.recordOperation({
        modelId: registeredModelId,
        operationId: `task-${Date.now()}`,
        timestamp: new Date(),
        wallClockMs: duration,
        cpuTimeMs: duration * 0.8,
        peakMemoryMB: 256,
        avgMemoryMB: 200,
        cpuUtilization: 60,
        inputTokens: response.inputTokens,
        outputTokens: response.outputTokens,
        tokensPerSecond: response.outputTokens / (duration / 1000),
      });

      // Get cost profile
      const profile = costTracker.getCostProfile(registeredModelId);

      expect(profile).toBeDefined();
      expect(profile!.avgWallClockMs).toBeGreaterThan(0);

      console.log("\n✅ Compute cost tracked");
      console.log(`   Duration: ${profile!.avgWallClockMs}ms`);
      console.log(`   Operations: ${profile!.totalOperations}`);
    }, 30000);
  });

  describe("Real LLM with Evaluation", () => {
    it("should use real LLM via OllamaProvider", async () => {
      const prompt = `Transform this casual text to professional:
"hey whats up, can u send me that file plz?"

Professional version:`;

      const startTime = Date.now();
      const response = await ollamaProvider.generate({
        prompt,
        temperature: 0.7,
        maxTokens: 200,
      });
      const duration = Date.now() - startTime;

      expect(response).toBeDefined();
      expect(response.text.length).toBeGreaterThan(0);

      console.log("\n✅ Text transformation with real LLM");
      console.log(`   Duration: ${duration}ms`);
      console.log(`   Input: "hey whats up, can u send me that file plz?"`);
      console.log(`   Output: "${response.text.substring(0, 100)}..."`);
    }, 60000);

    it("should use real LLM for code generation", async () => {
      const prompt = `Generate a TypeScript function to calculate the sum of an array of numbers.

Requirements:
- Function name: calculateSum
- Parameter: numbers (array of numbers)
- Return: sum (number)
- Include TypeScript types
- Add JSDoc comment

Code:`;

      const startTime = Date.now();
      const response = await ollamaProvider.generate({
        prompt,
        temperature: 0.5,
        maxTokens: 500,
      });
      const duration = Date.now() - startTime;

      expect(response).toBeDefined();
      expect(response.text.length).toBeGreaterThan(0);

      // Check for key elements
      const hasFunction =
        response.text.includes("function") || response.text.includes("=>");
      const hasTypes = response.text.includes("number");

      console.log("\n✅ Code generation with real LLM");
      console.log(`   Duration: ${duration}ms`);
      console.log(`   Has function: ${hasFunction}`);
      console.log(`   Has types: ${hasTypes}`);
      console.log(`   Output:\n${response.text.substring(0, 200)}...`);
    }, 60000);

    it("should use ModelBasedJudge with real LLM", async () => {
      const selector = new LocalModelSelector(registry, costTracker);

      const llmConfig: ModelRegistryLLMConfig = {
        provider: "model-registry",
        model: registeredModelId,
        temperature: 0.3,
        maxTokens: 200,
      };

      const llmProvider = new ModelRegistryLLMProvider(
        llmConfig,
        registry,
        selector,
        costTracker
      );

      const judge = new ModelBasedJudge({}, llmProvider);

      const input = {
        task: "integration-test-judgment",
        output: "Hello, I hope this message finds you well.",
        specification: "Professional email greeting",
        taskDescription: "Evaluate professional tone",
        context: {},
      };

      const startTime = Date.now();
      const result = await judge.evaluate(input);
      const duration = Date.now() - startTime;

      expect(result).toBeDefined();
      expect(result.allCriteriaPass).toBeDefined();
      expect(result.overallScore).toBeGreaterThanOrEqual(0);
      expect(result.overallScore).toBeLessThanOrEqual(1);

      console.log("\n✅ ModelBasedJudge with real LLM");
      console.log(`   Duration: ${duration}ms`);
      console.log(`   Output: "${input.output}"`);
      console.log(
        `   Overall Score: ${(result.overallScore * 100).toFixed(1)}%`
      );
      console.log(`   All Criteria Pass: ${result.allCriteriaPass}`);
      console.log(`   Criteria: ${result.assessments.length} evaluated`);
    }, 60000);
  });

  describe("Performance Tracking with Real LLM", () => {
    it("should track LLM performance via ModelRegistry", async () => {
      const modelId = registeredModelId;

      const startTime = Date.now();

      // Perform LLM call
      const prompt = "Rewrite this professionally: 'sup dude'";
      const _response = await ollamaProvider.generate({
        prompt,
        temperature: 0.7,
        maxTokens: 50,
      });

      const duration = Date.now() - startTime;

      // Update performance profile in registry
      const profile = createPerformanceProfile(modelId, "text-generation", {
        avgLatency: duration,
        successRate: 1.0,
        qualityScore: 0.85,
      });

      await registry.updatePerformanceProfile(modelId, profile);

      // Get performance data
      const retrievedProfile = registry.getPerformanceProfile(modelId);

      expect(retrievedProfile).toBeDefined();
      expect(retrievedProfile!.taskCategories.length).toBeGreaterThan(0);
      expect(retrievedProfile!.taskCategories[0].avgLatency).toBeGreaterThan(0);

      console.log("\n✅ Performance tracking via ModelRegistry");
      console.log(`   Model: ${modelId}`);
      console.log(
        `   Task categories: ${retrievedProfile!.taskCategories.length}`
      );
      console.log(
        `   Success rate: ${(
          retrievedProfile!.taskCategories[0].successRate * 100
        ).toFixed(1)}%`
      );
      console.log(
        `   Avg latency: ${retrievedProfile!.taskCategories[0].avgLatency.toFixed(
          0
        )}ms`
      );
    }, 30000);

    it("should track multiple inference calls", async () => {
      const modelId = registeredModelId;
      const prompts = ["Say hello", "Count to 3", "Name a color"];

      const results = [];
      let totalDuration = 0;

      for (const prompt of prompts) {
        const startTime = Date.now();

        const response = await ollamaProvider.generate({
          prompt,
          temperature: 0.7,
          maxTokens: 30,
        });

        const duration = Date.now() - startTime;
        totalDuration += duration;

        results.push({ prompt, response: response.text, duration });
      }

      // Update performance profile with aggregated data
      const avgLatency = totalDuration / prompts.length;
      const profile = createPerformanceProfile(modelId, "text-generation", {
        avgLatency,
        successRate: 1.0,
        qualityScore: 0.8,
      });

      await registry.updatePerformanceProfile(modelId, profile);

      const retrievedProfile = registry.getPerformanceProfile(modelId);

      console.log("\n✅ Multiple inference tracking");
      console.log(`   Total prompts: ${prompts.length}`);
      console.log(
        `   Avg latency: ${retrievedProfile!.taskCategories[0].avgLatency.toFixed(
          0
        )}ms`
      );

      results.forEach((r, i) => {
        console.log(`   ${i + 1}. "${r.prompt}" → ${r.duration}ms`);
      });

      expect(retrievedProfile!.taskCategories[0].avgLatency).toBeGreaterThan(0);
    }, 120000); // 2 minutes for prompts
  });

  describe("Iterative Refinement with Real LLM", () => {
    it("should refine output based on feedback", async () => {
      // Initial attempt
      let prompt = `Write a professional email greeting in one sentence.`;
      const response1 = await ollamaProvider.generate({
        prompt,
        temperature: 0.7,
        maxTokens: 200,
      });

      // Refine with feedback
      prompt = `Your previous response: "${response1.text}"

Feedback: Make it more concise and formal.

Improved version:`;
      const response2 = await ollamaProvider.generate({
        prompt,
        temperature: 0.7,
        maxTokens: 200,
      });

      console.log("\n✅ Iterative refinement");
      console.log(`   Iteration 1: "${response1.text.substring(0, 80)}..."`);
      console.log(`   Iteration 2: "${response2.text.substring(0, 80)}..."`);

      expect(response1).toBeDefined();
      expect(response2).toBeDefined();
      expect(response1.text.length).toBeGreaterThan(0);
      expect(response2.text.length).toBeGreaterThan(0);
    }, 90000);
  });
});
