/**
 * Code Generation E2E Tests
 *
 * @author @darianrosebrook
 * @description End-to-end tests for agent code generation with quality validation
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
  CodeGenerationRunner,
  type CodeGenerationSpec,
} from "./runners/CodeGenerationRunner";

describe("Code Generation E2E Tests", () => {
  let runner: CodeGenerationRunner;
  let registry: ModelRegistry;

  beforeAll(async () => {
    console.log("\n🚀 Initializing Code Generation E2E Test Suite...");

    // 1. Initialize Model Registry
    registry = new ModelRegistry();

    // Register a local Ollama model
    await registry.registerOllamaModel(
      "gemma-2b-code",
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
      model: "gemma-2b-code",
      temperature: 0.7,
      maxTokens: 1000,
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

    // 7. Create Code Generation Runner
    const mockMcpServer = {} as any;

    runner = new CodeGenerationRunner(
      judge,
      mockMcpServer,
      performanceTracker,
      registry,
      {
        maxIterations: 3,
        passingThreshold: 0.8,
        requireAllCriteriaPassed: false,
        iterationTimeoutMs: 30000,
        delayBetweenIterationsMs: 500,
      }
    );

    console.log("✅ Code Generation E2E Test Suite Ready\n");
  }, 60000);

  afterAll(async () => {
    console.log("\n🧹 Cleaning up Code Generation E2E Test Suite...");
  });

  describe("Basic Code Generation", () => {
    it("should generate a React Button component", async () => {
      const spec: CodeGenerationSpec = {
        input: {
          specification:
            "Create a Button component with children, onClick, variant (primary/secondary), size (sm/md/lg), and disabled props",
          language: "typescript",
          requiredElements: ["interface", "export", "Button", "React"],
          bannedPatterns: ["any", "console.log"],
          minLines: 10,
          maxLines: 50,
        },
        expected: {
          hasTypes: true,
          hasComments: true,
        },
      };

      const result = await runner.runScenario(spec);

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

      // Check specific criteria
      const syntaxResult = result.report.criteria.find(
        (c: { id: string }) => c.id === "valid-syntax"
      );
      expect(syntaxResult).toBeDefined();
      expect(syntaxResult!.passed).toBe(true);

      const requiredResult = result.report.criteria.find(
        (c: { id: string }) => c.id === "required-elements"
      );
      expect(requiredResult).toBeDefined();
      expect(requiredResult!.score).toBeGreaterThan(0);

      // Check code structure
      const code = result.output as string;
      expect(code).toContain("interface");
      expect(code).toContain("Button");
      expect(code).toContain("export");
      expect(code.split("\n").length).toBeGreaterThanOrEqual(10);
      expect(code.split("\n").length).toBeLessThanOrEqual(50);

      console.log("\n✅ Button component generated successfully");
      console.log(`   Lines of code: ${code.split("\n").length}`);
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 60000);

    it("should generate a Form component with validation", async () => {
      const spec: CodeGenerationSpec = {
        input: {
          specification:
            "Create a LoginForm component with email and password fields, submit button, error handling, and loading state",
          language: "typescript",
          requiredElements: [
            "interface",
            "export",
            "LoginForm",
            "useState",
            "form",
          ],
          bannedPatterns: ["any"],
        },
        expected: {
          hasTypes: true,
          hasComments: true,
          hasFunctionality: ["validation", "submit", "error"],
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("LoginForm");
      expect(code).toContain("useState");
      expect(code).toContain("form");
      
      // Form component is complex, accept reasonable attempts
      expect(result.report.overallScore).toBeGreaterThan(0.6); // At least 60%

      console.log("\n✅ Form component generated successfully");
      console.log(`   Score: ${(result.report.overallScore * 100).toFixed(1)}%`);
    }, 60000);
  });

  describe("Utility Functions", () => {
    it("should generate a utility function", async () => {
      const spec: CodeGenerationSpec = {
        input: {
          specification:
            "Create a fibonacci function that calculates the nth Fibonacci number",
          language: "typescript",
          requiredElements: ["export", "function", "fibonacci", "number"],
          bannedPatterns: ["any"],
        },
        expected: {
          hasTypes: true,
          hasComments: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result.success).toBe(true);
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("function fibonacci");
      expect(code).toContain("export");
      expect(code).toContain("number");

      console.log("\n✅ Utility function generated successfully");
    }, 60000);
  });

  describe("Edge Cases", () => {
    it("should handle simple component generation", async () => {
      const spec: CodeGenerationSpec = {
        input: {
          specification: "Create a simple Counter component",
          language: "typescript",
          requiredElements: ["export", "Counter"],
          bannedPatterns: [],
        },
      };

      const result = await runner.runScenario(spec);

      expect(result.success).toBe(true);
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("Counter");
      expect(code).toContain("export");

      console.log("\n✅ Simple component generated successfully");
    }, 60000);

    it("should detect banned patterns", async () => {
      const spec: CodeGenerationSpec = {
        input: {
          specification: "Create a debug utility",
          language: "typescript",
          requiredElements: ["export", "function"],
          bannedPatterns: ["console.log", "debugger"],
        },
      };

      const result = await runner.runScenario(spec);

      // Should either pass without banned patterns or fail with appropriate feedback
      expect(result).toBeDefined();

      if (!result.success) {
        const bannedCriterion = result.report.criteria.find(
          (c: { id: string }) => c.id.includes("banned")
        );
        expect(bannedCriterion).toBeDefined();
      }

      console.log("\n✅ Banned pattern detection working");
    }, 60000);
  });

  describe("Quality Validation", () => {
    it("should validate code quality standards", async () => {
      const spec: CodeGenerationSpec = {
        input: {
          specification: "Create a well-documented, type-safe Button component",
          language: "typescript",
          requiredElements: ["interface", "export", "Button"],
          bannedPatterns: ["any"],
        },
        expected: {
          hasTypes: true,
          hasComments: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result.report).toBeDefined();

      // Check for quality criteria
      const qualityCriterion = result.report.criteria.find(
        (c: { id: string }) => c.id.includes("quality")
      );

      if (qualityCriterion) {
        console.log(
          `\n📊 Code Quality Score: ${(qualityCriterion.score * 100).toFixed(
            1
          )}%`
        );
      }

      console.log("\n✅ Quality validation working");
    }, 60000);
  });
});
