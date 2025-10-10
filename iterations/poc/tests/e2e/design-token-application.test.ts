/**
 * Design Token Application E2E Test
 *
 * @author @darianrosebrook
 * @description End-to-end test for design token application with semantic validation
 */

import { describe, expect, it } from "@jest/globals";
import { DESIGN_TOKEN_CRITERIA } from "./evaluation-framework";
import { E2EEvaluationRunner, E2ETestResult } from "./evaluation-runner";

describe("Design Token Application E2E", () => {
  let runner: E2EEvaluationRunner;

  beforeEach(async () => {
    runner = new E2EEvaluationRunner(false); // Live mode with real MCP server
    await runner.initialize();
  }, 240000); // 4 minute timeout for setup

  afterEach(async () => {
    await runner?.shutdown();
  }, 60000);

  it("should apply design tokens to component styling", async () => {
    jest.setTimeout(180000); // 3 minutes for AI generation
    // Define the test scenario
    const scenario = {
      id: "design-token-e2e",
      name: "Design Token Application E2E",
      description:
        "Generate a styled component using design tokens instead of hardcoded values",
      input: {
        componentSpec: `Create a Card component with:
- Background color and text color
- Padding using spacing tokens
- Border radius using radius tokens
- No hardcoded colors, spacing, or radius values
- Use semantic token references like tokens.colors.bg.default`,
        tokens: {
          colors: {
            "bg.default": "#ffffff",
            "bg.secondary": "#f8f9fa",
            "text.primary": "#212529",
            "text.secondary": "#6c757d",
            "border.light": "#dee2e6",
          },
          space: {
            sm: "0.5rem",
            md: "1rem",
            lg: "1.5rem",
          },
          radius: {
            sm: "0.25rem",
            md: "0.375rem",
            lg: "0.5rem",
          },
        },
        availableTokens: {
          colors: {
            "bg.default": "#ffffff",
            "bg.secondary": "#f8f9fa",
            "text.primary": "#212529",
            "text.secondary": "#6c757d",
            "border.light": "#dee2e6",
          },
          space: {
            sm: "0.5rem",
            md: "1rem",
            lg: "1.5rem",
          },
          radius: {
            sm: "0.25rem",
            md: "0.375rem",
            lg: "0.5rem",
          },
        },
      },
      expectedCriteria: DESIGN_TOKEN_CRITERIA,
      timeout: 150000, // 2.5 minutes for AI processing
    };

    // Run the scenario
    const result: E2ETestResult = await runner.runScenario(scenario);

    // Verify the result structure
    expect(result).toBeDefined();
    expect(result.scenario.id).toBe(scenario.id);
    expect(result.output).toBeDefined();
    expect(typeof result.output).toBe("string");
    expect(result.output.length).toBeGreaterThan(0);
    expect(result.executionTime).toBeGreaterThan(0);

    // Check that agent made tool calls
    const toolCalls = result.agentInteractions.filter(
      (i) => i.type === "tool_call"
    );
    expect(toolCalls.length).toBeGreaterThan(0);

    // Verify evaluation was performed
    expect(result.report).toBeDefined();
    expect(result.report.criteria).toBeDefined();
    expect(result.report.criteria.length).toBe(
      scenario.expectedCriteria.length
    );

    // Log detailed results
    console.log("\n📊 Design Token Application Results:");
    console.log(`✅ Success: ${result.success}`);
    console.log(`📈 Score: ${(result.report.overallScore * 100).toFixed(1)}%`);
    console.log(`⏱️  Duration: ${result.executionTime}ms`);

    console.log("\n🔍 Detailed Criteria:");
    result.report.criteria.forEach((criterion) => {
      const status = criterion.result.passed ? "✅" : "❌";
      console.log(
        `${status} ${criterion.criteria.name}: ${(
          criterion.result.score * 100
        ).toFixed(1)}%`
      );
      if (!criterion.result.passed && criterion.result.suggestions) {
        console.log(`   💡 ${criterion.result.suggestions.join(", ")}`);
      }
    });

    // Additional validation for design tokens
    if (result.output) {
      validateDesignTokenUsage(result.output, scenario.input.tokens);
    }
  }, 60000); // 1 minute timeout

  it("should evaluate design token criteria correctly", async () => {
    const _runner = new E2EEvaluationRunner();

    // Test hardcoded color detection
    const badCode = `
const Card = styled.div\`
  background-color: #ffffff;
  color: #212529;
  padding: 1rem;
  border-radius: 0.375rem;
\`;
`;

    const noHardcodedColorsCriteria = DESIGN_TOKEN_CRITERIA.find(
      (c) => c.id === "no-hardcoded-colors"
    )!;
    const result1 = await noHardcodedColorsCriteria.evaluate(badCode);

    expect(result1.passed).toBe(false);

    // Test token usage detection
    const goodCode = `
const Card = styled.div\`
  background-color: \${tokens.colors.bg.default};
  color: \${tokens.text.primary};
  padding: \${tokens.space.md};
  border-radius: \${tokens.radius.md};
\`;
`;

    const usesDesignTokensCriteria = DESIGN_TOKEN_CRITERIA.find(
      (c) => c.id === "uses-design-tokens"
    )!;
    const result2 = await usesDesignTokensCriteria.evaluate(goodCode, {
      availableTokens: {
        colors: { "bg.default": "#fff", "text.primary": "#000" },
        space: { md: "1rem" },
        radius: { md: "0.375rem" },
      },
    });

    expect(result2.passed).toBe(true);

    console.log("✅ Design token evaluation criteria working correctly");
  });

  it("should handle semantic token application", async () => {
    jest.setTimeout(180000); // 3 minutes for AI generation
    // Test semantic token usage in different contexts
    const scenarios = [
      {
        name: "Background colors",
        spec: "Create a component with primary and secondary background colors",
        expectedTokens: ["bg.default", "bg.secondary"],
      },
      {
        name: "Text colors",
        spec: "Create a component with primary and secondary text colors",
        expectedTokens: ["text.primary", "text.secondary"],
      },
      {
        name: "Spacing tokens",
        spec: "Create a component with small, medium, and large spacing",
        expectedTokens: ["space.sm", "space.md", "space.lg"],
      },
    ];

    for (const testCase of scenarios) {
      const scenario = {
        id: `design-semantic-${testCase.name
          .toLowerCase()
          .replace(/\s+/g, "-")}`,
        name: `Semantic ${testCase.name}`,
        description: `Test semantic token usage for ${testCase.name.toLowerCase()}`,
        input: {
          componentSpec: testCase.spec,
          tokens: {
            colors: {
              "bg.default": "#ffffff",
              "bg.secondary": "#f8f9fa",
              "text.primary": "#212529",
              "text.secondary": "#6c757d",
            },
            space: {
              sm: "0.5rem",
              md: "1rem",
              lg: "1.5rem",
            },
          },
        },
        expectedCriteria: [
          DESIGN_TOKEN_CRITERIA.find((c) => c.id === "semantic-token-usage")!,
        ],
        timeout: 120000, // 2 minutes for AI processing
      };

      const result = await runner.runScenario(scenario);

      expect(result.output).toBeDefined();
      expect(result.report.criteria.length).toBeGreaterThan(0);

      console.log(
        `✅ ${testCase.name} semantic tokens: ${(
          result.report.overallScore * 100
        ).toFixed(1)}%`
      );
    }
  });

  it("should validate token consistency across components", async () => {
    jest.setTimeout(240000); // 4 minutes for multiple AI generations
    // Test that similar components use consistent token patterns
    const componentTypes = ["Card", "Button", "Input"];

    const results = [];

    for (const componentType of componentTypes) {
      const scenario = {
        id: `design-consistency-${componentType.toLowerCase()}`,
        name: `${componentType} Consistency`,
        description: `Test token consistency for ${componentType} component`,
        input: {
          componentSpec: `Create a ${componentType} component with appropriate styling using design tokens`,
          tokens: {
            colors: {
              "bg.default": "#ffffff",
              "text.primary": "#212529",
              "border.light": "#dee2e6",
            },
            space: {
              sm: "0.5rem",
              md: "1rem",
            },
            radius: {
              sm: "0.25rem",
              md: "0.375rem",
            },
          },
        },
        expectedCriteria: [
          DESIGN_TOKEN_CRITERIA.find((c) => c.id === "uses-design-tokens")!,
          DESIGN_TOKEN_CRITERIA.find((c) => c.id === "no-hardcoded-colors")!,
          DESIGN_TOKEN_CRITERIA.find((c) => c.id === "no-hardcoded-spacing")!,
        ],
        timeout: 120000, // 2 minutes for AI processing
      };

      const result = await runner.runScenario(scenario);
      results.push({ component: componentType, result });
    }

    // Check for consistency across components
    const scores = results.map((r) => r.result.report.overallScore);
    const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
    const consistency = 1 - (Math.max(...scores) - Math.min(...scores)); // Higher = more consistent

    console.log(`\n🔍 Token Consistency Analysis:`);
    console.log(`   Average Score: ${(avgScore * 100).toFixed(1)}%`);
    console.log(`   Consistency: ${(consistency * 100).toFixed(1)}%`);

    results.forEach(({ component, result }) => {
      console.log(
        `   ${component}: ${(result.report.overallScore * 100).toFixed(1)}%`
      );
    });

    // All components should pass basic token usage
    results.forEach(({ component, result }) => {
      expect(result.success).toBe(true);
      console.log(`✅ ${component} uses design tokens correctly`);
    });
  });

  it("should handle complex design systems", async () => {
    jest.setTimeout(240000); // 4 minutes for complex AI generation
    // Test with a comprehensive design token system
    const comprehensiveTokens = {
      colors: {
        "bg.default": "#ffffff",
        "bg.secondary": "#f8f9fa",
        "bg.accent": "#007bff",
        "text.primary": "#212529",
        "text.secondary": "#6c757d",
        "text.inverse": "#ffffff",
        "border.light": "#dee2e6",
        "border.medium": "#adb5bd",
        success: "#28a745",
        warning: "#ffc107",
        error: "#dc3545",
      },
      space: {
        xs: "0.25rem",
        sm: "0.5rem",
        md: "1rem",
        lg: "1.5rem",
        xl: "3rem",
        xxl: "4rem",
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
        "font-size.xl": "1.25rem",
        "line-height.tight": "1.25",
        "line-height.normal": "1.5",
        "line-height.loose": "2",
      },
      shadows: {
        sm: "0 1px 2px 0 rgba(0, 0, 0, 0.05)",
        md: "0 4px 6px -1px rgba(0, 0, 0, 0.1)",
        lg: "0 10px 15px -3px rgba(0, 0, 0, 0.1)",
      },
    };

    const scenario = {
      id: "design-comprehensive-design-system",
      name: "Comprehensive Design System",
      description:
        "Test design token application with comprehensive token library",
      input: {
        componentSpec: `Create a complex DashboardCard component with:
- Multiple background and text color variants
- Comprehensive spacing (padding, margins, gaps)
- Typography hierarchy (headings, body text)
- Border radius variations
- Shadow effects
- Success/warning/error state styling
- Responsive spacing`,
        tokens: comprehensiveTokens,
        availableTokens: comprehensiveTokens,
      },
      expectedCriteria: DESIGN_TOKEN_CRITERIA,
      timeout: 180000, // 3 minutes for complex AI processing
    };

    const result = await runner.runScenario(scenario);

    expect(result.output).toBeDefined();
    expect(result.output.length).toBeGreaterThan(200); // Should be substantial

    // Count different types of tokens used
    const output = result.output;
    const tokenUsage = {
      colors: (output.match(/tokens\.colors\./g) || []).length,
      space: (output.match(/tokens\.space\./g) || []).length,
      radius: (output.match(/tokens\.radius\./g) || []).length,
      typography: (output.match(/tokens\.typography\./g) || []).length,
      shadows: (output.match(/tokens\.shadows\./g) || []).length,
    };

    const totalTokensUsed = Object.values(tokenUsage).reduce(
      (a, b) => a + b,
      0
    );

    console.log(`\n🔍 Comprehensive Design System Analysis:`);
    console.log(`   Colors: ${tokenUsage.colors}`);
    console.log(`   Space: ${tokenUsage.space}`);
    console.log(`   Radius: ${tokenUsage.radius}`);
    console.log(`   Typography: ${tokenUsage.typography}`);
    console.log(`   Shadows: ${tokenUsage.shadows}`);
    console.log(`   Total tokens used: ${totalTokensUsed}`);

    // Should use a variety of token types
    expect(totalTokensUsed).toBeGreaterThan(5);
    expect(tokenUsage.colors).toBeGreaterThan(0);
    expect(tokenUsage.space).toBeGreaterThan(0);

    console.log("✅ Comprehensive design system token usage working");
  });

  /**
   * Helper function to validate design token usage
   */
  function validateDesignTokenUsage(output: string, _tokens: any): void {
    const issues: string[] = [];

    // Check for hardcoded values that should use tokens
    const hardcodedColors = output.match(/#[0-9a-fA-F]{3,6}/g) || [];
    const hardcodedSpacing = output.match(/\d+\.?\d*(?:px|rem|em)/g) || [];

    if (hardcodedColors.length > 0) {
      issues.push(
        `${hardcodedColors.length} hardcoded colors found: ${hardcodedColors
          .slice(0, 3)
          .join(", ")}`
      );
    }

    if (hardcodedSpacing.length > 0) {
      issues.push(
        `${
          hardcodedSpacing.length
        } hardcoded spacing values found: ${hardcodedSpacing
          .slice(0, 3)
          .join(", ")}`
      );
    }

    // Check for token references
    const tokenReferences = output.match(/tokens\.\w+\.\w+/g) || [];
    if (tokenReferences.length === 0) {
      issues.push("No design token references found");
    }

    if (issues.length > 0) {
      console.log("⚠️ Design token validation issues:");
      issues.forEach((issue) => console.log(`   - ${issue}`));
    } else {
      console.log("✅ No design token validation issues found");
    }
  }
});
