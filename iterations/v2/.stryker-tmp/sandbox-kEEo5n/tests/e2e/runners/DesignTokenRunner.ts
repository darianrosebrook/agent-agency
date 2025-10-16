/**
 * Design Token E2E Test Runner
 *
 * @author @darianrosebrook
 * @description Tests agent's ability to apply design systems using semantic tokens
 */
// @ts-nocheck


import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import type { ArbiterMCPServer } from "@/mcp-server/ArbiterMCPServer";
import type { ModelRegistry } from "@/models/ModelRegistry";
import type { PerformanceTracker } from "@/rl/PerformanceTracker";
import type {
  CriterionResult,
  EvaluationReport,
  GenerationContext,
  IterativeConfig,
  TestResult,
} from "../types/evaluation";
import {
  combineCriteria,
  createProgrammaticCriterion,
  createRegexCriterion,
} from "../utils/evaluation-helpers";
import { V2EvaluationRunner } from "./V2EvaluationRunner";

/**
 * Design token categories
 */
export interface DesignTokens {
  colors?: Record<string, string>;
  space?: Record<string, string>;
  radius?: Record<string, string>;
  typography?: Record<string, string>;
  shadows?: Record<string, string>;
}

/**
 * Design token specification
 */
export interface DesignTokenSpec {
  input: {
    componentSpec: string; // What to build
    tokens: DesignTokens; // Available tokens
    componentType: string; // Card, Button, Input, etc.
  };
  evaluation: {
    noHardcodedColors: boolean;
    noHardcodedSpacing: boolean;
    usesTokens: boolean;
    semanticNaming: boolean;
  };
}

/**
 * Design Token E2E Test Runner
 *
 * Tests the agent's ability to:
 * - Use semantic design tokens instead of hardcoded values
 * - Apply consistent token patterns across components
 * - Avoid hardcoded colors, spacing, radius, etc.
 * - Build components that follow design system principles
 */
export class DesignTokenRunner extends V2EvaluationRunner<
  DesignTokenSpec,
  string
> {
  constructor(
    judge: ModelBasedJudge,
    mcpServer: ArbiterMCPServer,
    performanceTracker: PerformanceTracker,
    registry: ModelRegistry,
    iterativeConfig?: Partial<IterativeConfig>
  ) {
    super(judge, mcpServer, performanceTracker, registry, iterativeConfig);
  }

  /**
   * Run a design token scenario
   */
  async runScenario(spec: DesignTokenSpec): Promise<TestResult> {
    console.log("\n🎨 Design Token E2E Test");
    console.log("================================");
    console.log(`Component: ${spec.input.componentType}`);
    console.log(`Spec: "${spec.input.componentSpec.substring(0, 80)}..."`);
    console.log(
      `Tokens available: ${Object.keys(spec.input.tokens).join(", ")}`
    );
    console.log("================================");

    return this.iterativeLoop(
      // Generate function
      async (context: GenerationContext) => {
        return this.generateComponent(spec, context);
      },

      // Evaluate function
      async (output: string) => {
        return this.evaluateComponent(output, spec);
      }
    );
  }

  /**
   * Generate component with design tokens
   */
  private async generateComponent(
    spec: DesignTokenSpec,
    context: GenerationContext
  ): Promise<string> {
    console.log(
      `\n🎨 Generating ${spec.input.componentType} with design tokens (iteration ${context.iteration})...`
    );

    // Build prompt
    const prompt = this.buildDesignTokenPrompt(spec, context);

    // For E2E testing, use mock generation
    // In production, this would call the actual LLM
    const component = this.mockComponentGeneration(spec);

    console.log(`✅ Generated ${component.split("\n").length} lines of code`);

    return component;
  }

  /**
   * Mock component generation for testing
   */
  private mockComponentGeneration(spec: DesignTokenSpec): string {
    const { componentType } = spec.input;

    switch (componentType.toLowerCase()) {
      case "card":
        return this.generateCardComponent(spec.input.tokens);
      case "button":
        return this.generateButtonComponent(spec.input.tokens);
      case "input":
        return this.generateInputComponent(spec.input.tokens);
      case "badge":
        return this.generateBadgeComponent(spec.input.tokens);
      default:
        return this.generateGenericComponent(componentType, spec.input.tokens);
    }
  }

  /**
   * Generate Card component
   */
  private generateCardComponent(tokens: DesignTokens): string {
    return `/**
 * Card component with design tokens
 */
import styled from "styled-components";

interface CardProps {
  variant?: "default" | "secondary";
  children: React.ReactNode;
}

const StyledCard = styled.div<{ variant: string }>\`
  background-color: \${(props) =>
    props.variant === "secondary"
      ? props.theme.tokens.colors["bg.secondary"]
      : props.theme.tokens.colors["bg.default"]};
  color: \${(props) => props.theme.tokens.colors["text.primary"]};
  padding: \${(props) => props.theme.tokens.space.md};
  border-radius: \${(props) => props.theme.tokens.radius.md};
  border: 1px solid \${(props) => props.theme.tokens.colors["border.light"]};
  box-shadow: \${(props) => props.theme.tokens.shadows?.sm || "none"};
\`;

export const Card: React.FC<CardProps> = ({
  variant = "default",
  children,
}) => {
  return <StyledCard variant={variant}>{children}</StyledCard>;
};

export default Card;
`;
  }

  /**
   * Generate Button component
   */
  private generateButtonComponent(tokens: DesignTokens): string {
    return `/**
 * Button component with design tokens
 */
import styled from "styled-components";

interface ButtonProps {
  variant?: "primary" | "secondary";
  size?: "sm" | "md" | "lg";
  children: React.ReactNode;
  onClick?: () => void;
}

const StyledButton = styled.button<{ variant: string; size: string }>\`
  background-color: \${(props) =>
    props.variant === "primary"
      ? props.theme.tokens.colors["bg.accent"]
      : props.theme.tokens.colors["bg.secondary"]};
  color: \${(props) =>
    props.variant === "primary"
      ? props.theme.tokens.colors["text.inverse"]
      : props.theme.tokens.colors["text.primary"]};
  padding: \${(props) => {
    if (props.size === "sm") return props.theme.tokens.space.sm;
    if (props.size === "lg") return props.theme.tokens.space.lg;
    return props.theme.tokens.space.md;
  }};
  border-radius: \${(props) => props.theme.tokens.radius.md};
  border: none;
  cursor: pointer;
  font-size: \${(props) => props.theme.tokens.typography?.["font-size.md"] || "1rem"};
  
  &:hover {
    opacity: 0.9;
  }
\`;

export const Button: React.FC<ButtonProps> = ({
  variant = "primary",
  size = "md",
  children,
  onClick,
}) => {
  return (
    <StyledButton variant={variant} size={size} onClick={onClick}>
      {children}
    </StyledButton>
  );
};

export default Button;
`;
  }

  /**
   * Generate Input component
   */
  private generateInputComponent(tokens: DesignTokens): string {
    return `/**
 * Input component with design tokens
 */
import styled from "styled-components";

interface InputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  error?: string;
}

const InputWrapper = styled.div\`
  display: flex;
  flex-direction: column;
  gap: \${(props) => props.theme.tokens.space.sm};
\`;

const Label = styled.label\`
  color: \${(props) => props.theme.tokens.colors["text.primary"]};
  font-size: \${(props) => props.theme.tokens.typography?.["font-size.sm"] || "0.875rem"};
\`;

const StyledInput = styled.input<{ hasError: boolean }>\`
  padding: \${(props) => props.theme.tokens.space.md};
  border: 1px solid \${(props) =>
    props.hasError
      ? props.theme.tokens.colors.error
      : props.theme.tokens.colors["border.light"]};
  border-radius: \${(props) => props.theme.tokens.radius.md};
  background-color: \${(props) => props.theme.tokens.colors["bg.default"]};
  color: \${(props) => props.theme.tokens.colors["text.primary"]};
  font-size: \${(props) => props.theme.tokens.typography?.["font-size.md"] || "1rem"};
  
  &:focus {
    outline: none;
    border-color: \${(props) => props.theme.tokens.colors["bg.accent"]};
  }
\`;

const ErrorText = styled.span\`
  color: \${(props) => props.theme.tokens.colors.error};
  font-size: \${(props) => props.theme.tokens.typography?.["font-size.sm"] || "0.875rem"};
\`;

export const Input: React.FC<InputProps> = ({
  label,
  value,
  onChange,
  error,
}) => {
  return (
    <InputWrapper>
      <Label>{label}</Label>
      <StyledInput
        hasError={!!error}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
      {error && <ErrorText>{error}</ErrorText>}
    </InputWrapper>
  );
};

export default Input;
`;
  }

  /**
   * Generate Badge component
   */
  private generateBadgeComponent(tokens: DesignTokens): string {
    return `/**
 * Badge component with design tokens
 */
import styled from "styled-components";

interface BadgeProps {
  variant?: "success" | "warning" | "error";
  children: React.ReactNode;
}

const StyledBadge = styled.span<{ variant: string }>\`
  display: inline-block;
  padding: \${(props) => \`\${props.theme.tokens.space.xs} \${props.theme.tokens.space.sm}\`};
  border-radius: \${(props) => props.theme.tokens.radius.full};
  background-color: \${(props) => props.theme.tokens.colors[props.variant]};
  color: \${(props) => props.theme.tokens.colors["text.inverse"]};
  font-size: \${(props) => props.theme.tokens.typography?.["font-size.sm"] || "0.875rem"};
  font-weight: 600;
\`;

export const Badge: React.FC<BadgeProps> = ({
  variant = "success",
  children,
}) => {
  return <StyledBadge variant={variant}>{children}</StyledBadge>;
};

export default Badge;
`;
  }

  /**
   * Generate generic component
   */
  private generateGenericComponent(name: string, tokens: DesignTokens): string {
    return `/**
 * ${name} component with design tokens
 */
import styled from "styled-components";

const Styled${name} = styled.div\`
  background-color: \${(props) => props.theme.tokens.colors["bg.default"]};
  color: \${(props) => props.theme.tokens.colors["text.primary"]};
  padding: \${(props) => props.theme.tokens.space.md};
  border-radius: \${(props) => props.theme.tokens.radius.md};
\`;

export const ${name}: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  return <Styled${name}>{children}</Styled${name}>;
};

export default ${name};
`;
  }

  /**
   * Build design token prompt
   */
  private buildDesignTokenPrompt(
    spec: DesignTokenSpec,
    context: GenerationContext
  ): string {
    let prompt = `Create a ${spec.input.componentType} component:\n\n`;
    prompt += `${spec.input.componentSpec}\n\n`;
    prompt += `Available Design Tokens:\n`;

    if (spec.input.tokens.colors) {
      prompt += `Colors: ${Object.keys(spec.input.tokens.colors).join(", ")}\n`;
    }
    if (spec.input.tokens.space) {
      prompt += `Spacing: ${Object.keys(spec.input.tokens.space).join(", ")}\n`;
    }
    if (spec.input.tokens.radius) {
      prompt += `Radius: ${Object.keys(spec.input.tokens.radius).join(", ")}\n`;
    }

    prompt += `\nIMPORTANT: Use semantic token references (tokens.colors.bg.default) instead of hardcoded values.\n`;

    if (context.previousOutput) {
      prompt += `\nPrevious attempt:\n${context.previousOutput}\n\n`;
      if (context.feedbackHistory.length > 0) {
        prompt += `Feedback:\n${
          context.feedbackHistory[context.feedbackHistory.length - 1]
        }\n`;
      }
    }

    return prompt;
  }

  /**
   * Evaluate component
   */
  private async evaluateComponent(
    component: string,
    spec: DesignTokenSpec
  ): Promise<EvaluationReport> {
    const startTime = Date.now();
    console.log("\n📊 Evaluating design token usage...");

    const criteria: CriterionResult[] = [];

    // 1. No hardcoded colors
    if (spec.evaluation.noHardcodedColors) {
      const result = await this.checkNoHardcodedColors(component);
      criteria.push(result);
      console.log(
        `   ${result.passed ? "✅" : "❌"} No hardcoded colors: ${(
          result.score * 100
        ).toFixed(0)}%`
      );
    }

    // 2. No hardcoded spacing
    if (spec.evaluation.noHardcodedSpacing) {
      const result = await this.checkNoHardcodedSpacing(component);
      criteria.push(result);
      console.log(
        `   ${result.passed ? "✅" : "❌"} No hardcoded spacing: ${(
          result.score * 100
        ).toFixed(0)}%`
      );
    }

    // 3. Uses design tokens
    if (spec.evaluation.usesTokens) {
      const result = await this.checkUsesTokens(component, spec.input.tokens);
      criteria.push(result);
      console.log(
        `   ${result.passed ? "✅" : "❌"} Uses tokens: ${(
          result.score * 100
        ).toFixed(0)}%`
      );
    }

    // 4. Semantic naming
    if (spec.evaluation.semanticNaming) {
      const result = await this.checkSemanticNaming(component);
      criteria.push(result);
      console.log(
        `   ${result.passed ? "✅" : "❌"} Semantic naming: ${(
          result.score * 100
        ).toFixed(0)}%`
      );
    }

    const overallScore =
      criteria.reduce((sum, c) => sum + c.score, 0) / criteria.length;
    const overallPassed = criteria.every((c) => c.passed);

    console.log(
      `\n   Overall: ${(overallScore * 100).toFixed(1)}% (${
        criteria.filter((c) => c.passed).length
      }/${criteria.length} passed)`
    );

    const executionTime = Date.now() - startTime;

    return {
      criteria,
      overallScore,
      overallPassed,
      executionTime,
      metadata: {
        componentType: spec.input.componentType,
        tokensAvailable: Object.keys(spec.input.tokens),
      },
    };
  }

  /**
   * Check for hardcoded colors
   */
  private async checkNoHardcodedColors(
    component: string
  ): Promise<CriterionResult> {
    const criterion = createRegexCriterion(
      "no-hardcoded-colors",
      "No Hardcoded Colors",
      /#[0-9a-fA-F]{3,6}|rgb\(|rgba\(/,
      false // Should NOT match (false = no hardcoded colors)
    );

    return criterion.evaluate(component, {});
  }

  /**
   * Check for hardcoded spacing
   */
  private async checkNoHardcodedSpacing(
    component: string
  ): Promise<CriterionResult> {
    const criterion = createRegexCriterion(
      "no-hardcoded-spacing",
      "No Hardcoded Spacing",
      /\d+\.?\d*(?:px|rem|em)/,
      false // Should NOT match (false = no hardcoded spacing)
    );

    return criterion.evaluate(component, {});
  }

  /**
   * Check uses design tokens
   */
  private async checkUsesTokens(
    component: string,
    tokens: DesignTokens
  ): Promise<CriterionResult> {
    const criterion = createProgrammaticCriterion(
      "uses-design-tokens",
      "Uses Design Tokens",
      "Component uses semantic token references",
      (output: unknown) => {
        if (typeof output !== "string") return false;

        // Check for token references
        const hasTokenReferences = /tokens\.\w+\.\w+/i.test(output);
        const hasThemeTokens = /theme\.tokens/i.test(output);

        return hasTokenReferences || hasThemeTokens;
      },
      0.8
    );

    return criterion.evaluate(component, { tokens });
  }

  /**
   * Check semantic naming
   */
  private async checkSemanticNaming(
    component: string
  ): Promise<CriterionResult> {
    const semanticPatterns = [
      /bg\.default|bg\.secondary|bg\.accent/i,
      /text\.primary|text\.secondary|text\.inverse/i,
      /space\.sm|space\.md|space\.lg/i,
      /radius\.sm|radius\.md|radius\.lg/i,
    ];

    const criteria = semanticPatterns.map((pattern, index) =>
      createRegexCriterion(
        `semantic-${index}`,
        `Semantic Pattern ${index + 1}`,
        pattern,
        true // Should match
      )
    );

    const combined = combineCriteria(
      "semantic-naming",
      "Semantic Naming",
      criteria
    );

    return combined.evaluate(component, {});
  }

  /**
   * Generate domain-specific feedback
   */
  protected generateFeedback(
    report: import("../types/evaluation").EvaluationReport,
    output: string
  ): string {
    const failedCriteria = report.criteria.filter((c) => !c.passed);

    const feedback = failedCriteria
      .map((criterion, index) => {
        const num = index + 1;
        const name = criterion.name;
        const score = (criterion.score * 100).toFixed(1);
        const threshold = (criterion.threshold * 100).toFixed(0);

        let suggestion = "";

        if (criterion.id.includes("hardcoded-colors")) {
          suggestion =
            "Replace hardcoded colors (#fff, rgb()) with semantic token references (tokens.colors.bg.default)";
        } else if (criterion.id.includes("hardcoded-spacing")) {
          suggestion =
            "Replace hardcoded spacing (16px, 1rem) with semantic token references (tokens.space.md)";
        } else if (criterion.id.includes("uses-design-tokens")) {
          suggestion =
            "Add more design token references throughout the component";
        } else if (criterion.id.includes("semantic")) {
          suggestion =
            "Use semantic token names (bg.default, text.primary) instead of generic ones";
        }

        return `${num}. ${name} (Score: ${score}%, Required: ${threshold}%)\n   Suggestion: ${suggestion}`;
      })
      .join("\n\n");

    return `The component needs improvement in ${failedCriteria.length} area${
      failedCriteria.length > 1 ? "s" : ""
    }:\n${feedback}\n\nApply design system principles consistently.`;
  }
}
