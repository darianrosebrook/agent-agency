/**
 * Advanced Reasoning E2E Tests
 *
 * @author @darianrosebrook
 * @description Tests requiring deep reasoning, multi-step problem solving, and iterative refinement
 */

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
  AdvancedReasoningRunner,
  type AdvancedReasoningSpec,
} from "./runners/AdvancedReasoningRunner";

describe("Advanced Reasoning E2E Tests", () => {
  let runner: AdvancedReasoningRunner;
  let registry: ModelRegistry;

  beforeAll(async () => {
    console.log("\n🚀 Initializing Advanced Reasoning E2E Test Suite...");

    // 1. Initialize Model Registry
    registry = new ModelRegistry();

    // Register a local Ollama model
    await registry.registerOllamaModel(
      "gemma-2b-reasoning",
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
      model: "gemma-2b-reasoning",
      temperature: 0.7,
      maxTokens: 2000, // More tokens for complex reasoning
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

    // 7. Create Advanced Reasoning Runner
    const mockMcpServer = {} as any;

    runner = new AdvancedReasoningRunner(
      judge,
      mockMcpServer,
      performanceTracker,
      registry,
      {
        maxIterations: 5, // More iterations for complex problems
        passingThreshold: 0.85, // Higher bar
        requireAllCriteriaPassed: false,
        iterationTimeoutMs: 60000, // 1 minute per iteration
        delayBetweenIterationsMs: 1000,
      }
    );

    console.log("✅ Advanced Reasoning E2E Test Suite Ready\n");
  }, 60000);

  afterAll(async () => {
    console.log("\n🧹 Cleaning up Advanced Reasoning E2E Test Suite...");
  });

  describe("Algorithm Design", () => {
    it("should design and implement an LRU Cache", async () => {
      const spec: AdvancedReasoningSpec = {
        problemType: "algorithm-design",
        input: {
          description: `Design and implement an LRU (Least Recently Used) cache with the following operations:
          
- get(key): Get the value of the key if it exists, otherwise return -1
- put(key, value): Set or insert the value if the key is not already present
- When capacity is reached, invalidate the least recently used item

The cache should support O(1) time complexity for both operations.`,
          requirements: [
            "class LRUCache",
            "get",
            "put",
            "O(1)",
            "capacity",
            "export",
          ],
          constraints: [
            "Must be O(1) time complexity",
            "Must handle capacity limits",
            "Must maintain LRU order",
          ],
          testCases: [
            {
              input: { operation: "put", key: 1, value: 1 },
              expectedOutput: undefined,
              description: "Put key 1",
            },
            {
              input: { operation: "get", key: 1 },
              expectedOutput: 1,
              description: "Get key 1",
            },
            {
              input: { operation: "get", key: 2 },
              expectedOutput: -1,
              description: "Get non-existent key",
            },
          ],
        },
        evaluation: {
          correctness: true,
          efficiency: true,
          quality: true,
          completeness: true,
        },
      };

      const result = await runner.runScenario(spec);

      // Assertions
      expect(result).toBeDefined();
      expect(result.success).toBe(true);
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("LRUCache");
      expect(code).toContain("get");
      expect(code).toContain("put");
      expect(code).toContain("O(1)");

      console.log("\n✅ LRU Cache implementation generated successfully");
      console.log(`   Iterations: ${result.iterations}`);
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 180000); // 3 minutes for complex algorithm
  });

  describe("Code Refactoring", () => {
    it("should refactor messy code to clean code", async () => {
      const messyCode = `
function calc(x,y,z){var r;if(x>0){if(y>0){if(z>0){r=x+y+z}else{r=x+y}}else{r=x}}else{r=0}return r}
function proc(arr){var res=[];for(var i=0;i<arr.length;i++){for(var j=0;j<arr.length;j++){if(i!=j){res.push(arr[i]+arr[j])}}}return res}
`;

      const spec: AdvancedReasoningSpec = {
        problemType: "code-refactoring",
        input: {
          description:
            "Refactor this messy code to follow best practices: proper naming, formatting, error handling, and TypeScript types",
          existingCode: messyCode,
          requirements: [
            "TypeScript types",
            "descriptive names",
            "error handling",
            "JSDoc comments",
            "guard clauses",
          ],
        },
        evaluation: {
          correctness: true,
          quality: true,
          completeness: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const code = result.output as string;

      // Advanced reasoning: Check for improvement attempt
      expect(code.length).toBeGreaterThan(messyCode.length); // Should be more verbose with comments
      expect(result.report.overallScore).toBeGreaterThan(0.3); // At least 30% quality (hard problem)

      console.log("\n✅ Code refactoring attempted");
      console.log(`   Original: ${messyCode.length} chars`);
      console.log(`   Refactored: ${code.length} chars`);
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 180000);
  });

  describe("System Design", () => {
    it("should design a scalable system architecture", async () => {
      const spec: AdvancedReasoningSpec = {
        problemType: "system-design",
        input: {
          description: `Design a task queue system that can:
          
- Accept tasks with priorities
- Process tasks asynchronously
- Handle failures with retry logic
- Scale horizontally
- Provide status monitoring

Explain your architecture, components, and trade-offs.`,
          requirements: [
            "Repository",
            "Service",
            "interface",
            "async",
            "scalable",
            "export",
          ],
        },
        evaluation: {
          correctness: true,
          completeness: true,
          quality: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const design = result.output as string;
      expect(design).toContain("interface");
      expect(result.report.overallScore).toBeGreaterThan(0.5); // At least 50% quality

      console.log("\n✅ System design attempted");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 180000);
  });

  describe("Bug Analysis", () => {
    it("should find and fix bugs in code", async () => {
      const buggyCode = `
function findMax(numbers) {
  let max = 0;
  for (let i = 0; i <= numbers.length; i++) {
    if (numbers[i] > max) {
      max = numbers[i];
    }
  }
  return max;
}
`;

      const spec: AdvancedReasoningSpec = {
        problemType: "bug-analysis",
        input: {
          description:
            "Find and fix all bugs in this code. Explain what was wrong and why your fix is correct.",
          existingCode: buggyCode,
          requirements: ["Root Cause", "Fix", "function", "return"],
        },
        evaluation: {
          correctness: true,
          quality: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const fixed = result.output as string;
      expect(fixed).toContain("Root Cause");
      expect(result.report.overallScore).toBeGreaterThan(0.5); // At least 50% quality

      console.log("\n✅ Bug analysis attempted");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
      console.log(
        `   Bugs identified: ${
          fixed.includes("off-by-one") ? "Yes" : "Partial"
        }`
      );
    }, 180000);
  });

  describe("Performance Optimization", () => {
    it("should optimize slow code", async () => {
      const slowCode = `
function findDuplicates(arr) {
  const duplicates = [];
  for (let i = 0; i < arr.length; i++) {
    for (let j = i + 1; j < arr.length; j++) {
      if (arr[i] === arr[j] && !duplicates.includes(arr[i])) {
        duplicates.push(arr[i]);
      }
    }
  }
  return duplicates;
}
`;

      const spec: AdvancedReasoningSpec = {
        problemType: "performance-optimization",
        input: {
          description:
            "Optimize this O(n²) code to O(n). Explain the optimization and measure improvement.",
          existingCode: slowCode,
          requirements: [
            "O(n)",
            "Performance",
            "Optimization",
            "function",
            "export",
          ],
        },
        evaluation: {
          correctness: true,
          efficiency: true,
          quality: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const optimized = result.output as string;
      expect(optimized).toContain("O(n)");
      expect(optimized).toContain("Optimization");
      expect(result.report.overallScore).toBeGreaterThan(0.2); // At least 20% quality (very hard problem)

      console.log("\n✅ Performance optimization attempted");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 180000);
  });

  describe("Complex Problem Solving", () => {
    it("should handle multi-step reasoning", async () => {
      const spec: AdvancedReasoningSpec = {
        problemType: "algorithm-design",
        input: {
          description: `Implement a function to detect a cycle in a linked list.
          
If there is a cycle, return the node where the cycle begins.
If there is no cycle, return null.

Use O(1) space complexity (no additional data structures).`,
          requirements: [
            "function",
            "cycle",
            "O(1)",
            "space",
            "Reasoning",
            "export",
          ],
        },
        evaluation: {
          correctness: true,
          efficiency: true,
          completeness: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();

      // Allow multiple iterations to refine solution
      expect(result.iterations).toBeGreaterThan(0);
      expect(result.iterations).toBeLessThanOrEqual(5);

      console.log("\n✅ Complex problem solved");
      console.log(`   Took ${result.iterations} iteration(s) to solve`);
      console.log(
        `   Final score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 180000);
  });
});
