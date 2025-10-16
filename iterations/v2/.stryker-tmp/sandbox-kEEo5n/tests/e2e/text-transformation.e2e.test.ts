/**
 * Text Transformation E2E Tests
 *
 * @author @darianrosebrook
 * @description End-to-end tests for agent text transformation with iterative feedback
 */
// @ts-nocheck


import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import {
  ModelRegistryLLMProvider,
  type ModelRegistryLLMConfig,
} from "@/evaluation/ModelRegistryLLMProvider";
import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelRegistry } from "@/models/ModelRegistry";
import { PerformanceTracker } from "@/rl/PerformanceTracker";
import {
  TextTransformationRunner,
  type TextTransformationSpec,
} from "./runners/TextTransformationRunner";

describe("Text Transformation E2E Tests", () => {
  let runner: TextTransformationRunner;
  let registry: ModelRegistry;

  beforeAll(async () => {
    console.log("\n🚀 Initializing Text Transformation E2E Test Suite...");

    // 1. Initialize Model Registry
    registry = new ModelRegistry();

    // Register a local Ollama model (gemma3n:e2b is fast and good for testing)
    await registry.registerOllamaModel(
      "gemma-2b-test",
      "gemma3n:e2b",
      "1.0.0",
      "primary"
    );

    // 2. Initialize Compute Cost Tracker
    const costTracker = new ComputeCostTracker();

    // 3. Initialize Local Model Selector
    const selector = new LocalModelSelector(registry, costTracker);

    // 4. Initialize ModelRegistryLLMProvider
    const llmConfig: ModelRegistryLLMConfig = {
      provider: "model-registry",
      model: "gemma-2b-test",
      temperature: 0.7,
      maxTokens: 500,
      taskType: "text-generation",
      qualityThreshold: 0.7,
      maxLatencyMs: 10000,
      maxMemoryMB: 4096,
    };

    const llmProvider = new ModelRegistryLLMProvider(
      llmConfig,
      registry,
      selector,
      costTracker
    );

    // 5. Initialize ModelBasedJudge
    const judge = new ModelBasedJudge({}, llmProvider);

    // 6. Initialize Performance Tracker
    const performanceTracker = new PerformanceTracker();

    // 7. Create Text Transformation Runner
    // Note: MCP server parameter is required by base class but not used in this test
    const mockMcpServer = {} as any; // Mock object to satisfy type requirements

    runner = new TextTransformationRunner(
      judge,
      mockMcpServer,
      performanceTracker,
      registry,
      {
        maxIterations: 3,
        passingThreshold: 0.8, // 80% overall score
        requireAllCriteriaPassed: false, // Allow overall score to pass even if one criterion fails
        iterationTimeoutMs: 30000, // 30 seconds for testing
        delayBetweenIterationsMs: 500, // Shorter delay for tests
      }
    );

    console.log("✅ Text Transformation E2E Test Suite Ready\n");
  }, 60000); // 1 minute timeout for setup

  afterAll(async () => {
    console.log("\n🧹 Cleaning up Text Transformation E2E Test Suite...");
    // Cleanup if needed
  });

  describe("Basic Text Transformation", () => {
    it("should transform casual message to professional language", async () => {
      const spec: TextTransformationSpec = {
        input: {
          text: "Hey team, this is a really casual message that needs to be made more professional. It's got some informal language and could use better structure. Let's make it work better for our stakeholders.",
          bannedPhrases: ["hey team", "really casual", "let's make it work"],
          requiredElements: ["professional", "stakeholders"],
          minLength: 50,
          maxLength: 500,
        },
        expected: {
          tone: "professional",
          style: ["clear", "concise"],
        },
      };

      const result = await runner.runScenario(spec);

      // Log summary
      runner.logTestSummary(result);

      // Assertions
      expect(result).toBeDefined();
      expect(result.success).toBe(true);
      expect(result.iterations).toBeGreaterThan(0);
      expect(result.iterations).toBeLessThanOrEqual(3);
      expect(result.output).toBeDefined();
      expect(typeof result.output).toBe("string");

      // Check evaluation report
      expect(result.report).toBeDefined();
      expect(result.report.overallScore).toBeGreaterThanOrEqual(0.8);
      // Note: overallPassed may be false even though we passed, because we allow
      // passing on overall score alone (requireAllCriteriaPassed: false)

      // Check specific criteria
      const bannedPhrasesResult = result.report.criteria.find(
        (c) => c.id === "no-banned-phrases"
      );
      expect(bannedPhrasesResult).toBeDefined();
      expect(bannedPhrasesResult?.passed).toBe(true);

      const requiredElementsResult = result.report.criteria.find(
        (c) => c.id === "required-elements"
      );
      expect(requiredElementsResult).toBeDefined();
      expect(requiredElementsResult?.passed).toBe(true);

      // Check agent interactions
      expect(result.agentInteractions).toBeDefined();
      expect(result.agentInteractions.length).toBeGreaterThan(0);

      const generations = result.agentInteractions.filter(
        (i) => i.type === "generation"
      );
      expect(generations.length).toBe(result.iterations);

      const evaluations = result.agentInteractions.filter(
        (i) => i.type === "evaluation"
      );
      expect(evaluations.length).toBe(result.iterations);

      // Check statistics
      const stats = runner.calculateStatistics(result);
      expect(stats.totalIterations).toBe(result.iterations);
      expect(stats.totalGenerationTimeMs).toBeGreaterThanOrEqual(0); // Mock might be very fast
      expect(stats.totalEvaluationTimeMs).toBeGreaterThanOrEqual(0); // Mock might be very fast
      expect(stats.averageScore).toBeGreaterThan(0);

      console.log("\n📊 Test Statistics:");
      console.log(`   Iterations: ${stats.totalIterations}`);
      console.log(
        `   Generation Time: ${(stats.totalGenerationTimeMs / 1000).toFixed(
          2
        )}s`
      );
      console.log(
        `   Evaluation Time: ${(stats.totalEvaluationTimeMs / 1000).toFixed(
          2
        )}s`
      );
      console.log(
        `   Average Score: ${(stats.averageScore * 100).toFixed(1)}%`
      );
      console.log(
        `   Score Improvement: ${(stats.scoreImprovement * 100).toFixed(1)}%`
      );
    }, 120000); // 2 minute timeout

    it("should handle short transformations", async () => {
      const spec: TextTransformationSpec = {
        input: {
          text: "Hey, can you help me with this?",
          bannedPhrases: ["hey"],
          requiredElements: ["assistance"],
          minLength: 20,
          maxLength: 100,
        },
        expected: {
          tone: "professional",
        },
      };

      const result = await runner.runScenario(spec);

      expect(result.success).toBe(true);
      expect(result.output).toBeDefined();

      const output = result.output as string;
      expect(output.length).toBeGreaterThanOrEqual(20);
      expect(output.length).toBeLessThanOrEqual(100);
      expect(output.toLowerCase()).toContain("assistance");
      expect(output.toLowerCase()).not.toContain("hey");
    }, 120000);
  });

  describe("Edge Cases", () => {
    it("should handle text with no banned phrases", async () => {
      const spec: TextTransformationSpec = {
        input: {
          text: "This message is already professional and appropriate for business communication.",
          bannedPhrases: ["casual", "hey"],
          requiredElements: ["professional"],
        },
        expected: {
          tone: "professional",
        },
      };

      const result = await runner.runScenario(spec);

      expect(result.success).toBe(true);
      expect(result.iterations).toBeLessThanOrEqual(2); // Should pass quickly
    }, 120000);

    it("should handle text requiring multiple iterations", async () => {
      const spec: TextTransformationSpec = {
        input: {
          text: "Hey dude, like, this is super casual and stuff. You know what I mean? It's totally not professional at all, lol.",
          bannedPhrases: ["hey dude", "like", "super casual", "lol", "stuff"],
          requiredElements: ["professional", "formal", "appropriate"],
          minLength: 50,
        },
        expected: {
          tone: "formal",
        },
      };

      const result = await runner.runScenario(spec);

      // This may or may not succeed depending on the model
      // But we can check that it tried multiple iterations
      expect(result.iterations).toBeGreaterThan(1);
      expect(result.feedbackHistory.length).toBeGreaterThan(0);

      if (result.success) {
        const output = result.output as string;
        expect(output.toLowerCase()).not.toContain("hey dude");
        expect(output.toLowerCase()).not.toContain("lol");
      } else {
        console.log(
          "⚠️  Test reached max iterations without passing (expected for difficult cases)"
        );
        expect(result.iterations).toBe(3);
      }
    }, 180000); // 3 minute timeout for difficult case
  });

  describe("Feedback Iteration", () => {
    it("should improve output with feedback", async () => {
      const spec: TextTransformationSpec = {
        input: {
          text: "Hey team, let's circle back on this and touch base later.",
          bannedPhrases: ["hey team", "circle back", "touch base"],
          requiredElements: ["schedule", "meeting"],
        },
        expected: {
          tone: "professional",
        },
      };

      const result = await runner.runScenario(spec);

      // Check that feedback was generated if needed
      if (result.iterations > 1) {
        expect(result.feedbackHistory.length).toBeGreaterThan(0);

        // Verify feedback contains actionable information
        result.feedbackHistory.forEach((feedback) => {
          expect(feedback.length).toBeGreaterThan(10);
          expect(typeof feedback).toBe("string");
        });
      }

      // Check score improvement over iterations
      const evaluations = result.agentInteractions.filter(
        (i) => i.type === "evaluation"
      );

      if (evaluations.length > 1) {
        const firstScore = evaluations[0].details.overallScore as number;
        const lastScore = evaluations[evaluations.length - 1].details
          .overallScore as number;

        // Score should improve or stay the same (not regress)
        expect(lastScore).toBeGreaterThanOrEqual(firstScore);
      }
    }, 120000);
  });
});
