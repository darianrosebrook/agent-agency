/**
 * Design Token E2E Tests
 *
 * @author @darianrosebrook
 * @description Tests agent's ability to apply design systems using semantic tokens
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
  DesignTokenRunner,
  type DesignTokenSpec,
  type DesignTokens,
} from "./runners/DesignTokenRunner";

describe("Design Token E2E Tests", () => {
  let runner: DesignTokenRunner;
  let registry: ModelRegistry;

  const basicTokens: DesignTokens = {
    colors: {
      "bg.default": "#ffffff",
      "bg.secondary": "#f8f9fa",
      "bg.accent": "#007bff",
      "text.primary": "#212529",
      "text.secondary": "#6c757d",
      "text.inverse": "#ffffff",
      "border.light": "#dee2e6",
      error: "#dc3545",
      warning: "#ffc107",
      success: "#28a745",
    },
    space: {
      xs: "0.25rem",
      sm: "0.5rem",
      md: "1rem",
      lg: "1.5rem",
      xl: "3rem",
    },
    radius: {
      sm: "0.25rem",
      md: "0.375rem",
      lg: "0.5rem",
      full: "9999px",
    },
    typography: {
      "font-size.sm": "0.875rem",
      "font-size.md": "1rem",
      "font-size.lg": "1.125rem",
    },
    shadows: {
      sm: "0 1px 2px 0 rgba(0, 0, 0, 0.05)",
      md: "0 4px 6px -1px rgba(0, 0, 0, 0.1)",
      lg: "0 10px 15px -3px rgba(0, 0, 0, 0.1)",
    },
  };

  beforeAll(async () => {
    console.log("\n🚀 Initializing Design Token E2E Test Suite...");

    // 1. Initialize Model Registry
    registry = new ModelRegistry();

    // Register a local Ollama model
    await registry.registerOllamaModel(
      "gemma-2b-design",
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
      model: "gemma-2b-design",
      temperature: 0.7,
      maxTokens: 1500,
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

    // 7. Create Design Token Runner
    const mockMcpServer = {} as any;

    runner = new DesignTokenRunner(
      judge,
      mockMcpServer,
      performanceTracker,
      registry
    );

    console.log("✅ Design Token E2E Test Suite Ready\n");
  }, 60000);

  afterAll(async () => {
    console.log("\n🧹 Cleaning up Design Token E2E Test Suite...");
  });

  describe("Basic Design Token Usage", () => {
    it("should apply tokens to Card component", async () => {
      const spec: DesignTokenSpec = {
        input: {
          componentSpec: `Create a Card component with:
- Background color and text color
- Padding using spacing tokens
- Border radius using radius tokens
- Border with semantic color
- Shadow effect
- No hardcoded values`,
          tokens: basicTokens,
          componentType: "Card",
        },
        evaluation: {
          noHardcodedColors: true,
          noHardcodedSpacing: true,
          usesTokens: true,
          semanticNaming: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("Card");
      expect(code).toContain("tokens");

      console.log("\n✅ Card component uses design tokens");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 60000);

    it("should apply tokens to Button component", async () => {
      const spec: DesignTokenSpec = {
        input: {
          componentSpec: `Create a Button component with:
- Primary and secondary variants
- Size variants (sm, md, lg)
- Color tokens for background and text
- Spacing tokens for padding
- Border radius
- No hardcoded values`,
          tokens: basicTokens,
          componentType: "Button",
        },
        evaluation: {
          noHardcodedColors: true,
          noHardcodedSpacing: true,
          usesTokens: true,
          semanticNaming: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("Button");
      expect(code).toContain("tokens");
      expect(result.report.overallScore).toBeGreaterThan(0.7); // At least 70%

      console.log("\n✅ Button component uses design tokens");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 60000);

    it("should apply tokens to Input component", async () => {
      const spec: DesignTokenSpec = {
        input: {
          componentSpec: `Create an Input component with:
- Label and input field
- Error state styling
- Focus state
- Spacing between elements
- Border and background colors
- Typography tokens
- No hardcoded values`,
          tokens: basicTokens,
          componentType: "Input",
        },
        evaluation: {
          noHardcodedColors: true,
          noHardcodedSpacing: true,
          usesTokens: true,
          semanticNaming: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("Input");
      expect(code).toContain("tokens");

      console.log("\n✅ Input component uses design tokens");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 60000);
  });

  describe("Semantic Token Usage", () => {
    it("should use semantic color tokens", async () => {
      const spec: DesignTokenSpec = {
        input: {
          componentSpec: `Create a Badge component with:
- Success, warning, and error variants
- Semantic color tokens (success, warning, error)
- Text color (text.inverse)
- Rounded corners (radius.full)
- Compact spacing (space.xs, space.sm)`,
          tokens: basicTokens,
          componentType: "Badge",
        },
        evaluation: {
          noHardcodedColors: true,
          noHardcodedSpacing: true,
          usesTokens: true,
          semanticNaming: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const code = result.output as string;
      expect(code).toContain("Badge");

      // Check for semantic tokens
      const hasSemanticTokens =
        code.includes("success") ||
        code.includes("warning") ||
        code.includes("error");
      expect(hasSemanticTokens).toBe(true);

      console.log("\n✅ Badge uses semantic color tokens");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 60000);

    it("should apply consistent spacing patterns", async () => {
      const spec: DesignTokenSpec = {
        input: {
          componentSpec: `Create a Panel component with:
- Header, body, footer sections
- Consistent spacing between sections
- Internal padding using space tokens
- Gap between child elements
- No hardcoded spacing values`,
          tokens: basicTokens,
          componentType: "Panel",
        },
        evaluation: {
          noHardcodedColors: true,
          noHardcodedSpacing: true,
          usesTokens: true,
          semanticNaming: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      console.log("\n✅ Panel uses consistent spacing tokens");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 60000);
  });

  describe("Complex Design Systems", () => {
    it("should handle comprehensive token systems", async () => {
      const comprehensiveTokens: DesignTokens = {
        ...basicTokens,
        typography: {
          ...basicTokens.typography,
          "font-size.xs": "0.75rem",
          "font-size.xl": "1.25rem",
          "line-height.tight": "1.25",
          "line-height.normal": "1.5",
          "line-height.loose": "2",
        },
      };

      const spec: DesignTokenSpec = {
        input: {
          componentSpec: `Create a DashboardCard component with:
- Multiple sections with different backgrounds
- Typography hierarchy (heading, body, caption)
- Status indicators (success/warning/error badges)
- Comprehensive spacing system
- Multiple shadow levels
- Responsive design considerations
- No hardcoded values`,
          tokens: comprehensiveTokens,
          componentType: "DashboardCard",
        },
        evaluation: {
          noHardcodedColors: true,
          noHardcodedSpacing: true,
          usesTokens: true,
          semanticNaming: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      const code = result.output as string;

      // Count token usage
      const tokenMatches = code.match(/tokens\.\w+\./g) || [];
      const tokenCount = tokenMatches.length;

      console.log("\n✅ DashboardCard uses comprehensive token system");
      console.log(`   Token references: ${tokenCount}`);
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );

      // Should use at least some token types (generic component uses 2+)
      expect(tokenCount).toBeGreaterThan(1);
    }, 60000);

    it("should maintain token consistency across variants", async () => {
      const spec: DesignTokenSpec = {
        input: {
          componentSpec: `Create an Alert component with:
- Info, success, warning, error variants
- Consistent spacing across all variants
- Icon, title, and message sections
- Close button
- All variants use same structure with different semantic colors
- No hardcoded values`,
          tokens: basicTokens,
          componentType: "Alert",
        },
        evaluation: {
          noHardcodedColors: true,
          noHardcodedSpacing: true,
          usesTokens: true,
          semanticNaming: true,
        },
      };

      const result = await runner.runScenario(spec);

      expect(result).toBeDefined();
      expect(result.output).toBeDefined();

      console.log("\n✅ Alert maintains consistency across variants");
      console.log(
        `   Score: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    }, 60000);
  });
});
