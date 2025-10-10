/**
 * E2E Evaluation Runner
 *
 * @author @darianrosebrook
 * @description Orchestrates E2E tests with agent evaluation and critique capabilities
 */

import {
  CODE_GENERATION_CRITERIA,
  DESIGN_TOKEN_CRITERIA,
  E2EEvaluator,
  EvaluationReport,
  TEXT_TRANSFORMATION_CRITERIA,
  TestScenario,
} from "./evaluation-framework";
import { createMCPClient } from "./mcp-client";

export interface E2ETestResult {
  scenario: TestScenario;
  report: EvaluationReport;
  output: any;
  executionTime: number;
  agentInteractions: AgentInteraction[];
  success: boolean;
  error?: string;
  iterations?: number;
  feedbackHistory?: string[];
}

export interface AgentInteraction {
  type: "tool_call" | "resource_read" | "evaluation";
  timestamp: string;
  details: any;
  result?: any;
  duration?: number;
}

export interface E2ETestSuite {
  name: string;
  description: string;
  scenarios: TestScenario[];
  setup?: () => Promise<void>;
  teardown?: () => Promise<void>;
}

export class E2EEvaluationRunner {
  private client?: any; // MCPClient with methods
  private evaluator: E2EEvaluator;
  private interactions: AgentInteraction[] = [];
  private mockMode: boolean = false;

  constructor(mockMode: boolean = false) {
    this.evaluator = new E2EEvaluator();
    this.mockMode = mockMode;

    // Register all evaluation criteria
    [
      ...TEXT_TRANSFORMATION_CRITERIA,
      ...CODE_GENERATION_CRITERIA,
      ...DESIGN_TOKEN_CRITERIA,
    ].forEach((criteria) => this.evaluator.registerCriteria(criteria));
  }

  /**
   * Initialize the E2E runner
   */
  async initialize(): Promise<void> {
    console.log("🚀 Initializing E2E Evaluation Runner...");

    if (this.mockMode) {
      console.log(
        "🔧 Running in mock mode - skipping MCP server initialization"
      );
      return;
    }

    // Create MCP client
    this.client = await createMCPClient();
    if (!this.client) {
      throw new Error("MCP client not initialized");
    }
    // Verify server capabilities
    const tools = await this.client.listTools();
    const resources = await this.client.listResources();

    console.log(
      `✅ Server ready: ${tools.length} tools, ${resources.length} resources`
    );
  }

  /**
   * Run a single test scenario
   */
  async runScenario(scenario: TestScenario): Promise<E2ETestResult> {
    const startTime = Date.now();
    this.interactions = [];
    let currentIteration = 0;

    try {
      console.log(`\n🧪 Running scenario: ${scenario.name}`);
      console.log(`📝 ${scenario.description}`);

      const maxIterations = scenario.maxIterations || 3;
      let lastOutput: string | null = null;
      let lastReport: any = null;
      const feedbackHistory: string[] = [];

      // Multi-turn feedback loop
      while (currentIteration < maxIterations) {
        currentIteration++;
        console.log(`\n🔄 Iteration ${currentIteration}/${maxIterations}`);

        try {
          // Execute the scenario with feedback context
          const output = await this.executeScenario(scenario, {
            iteration: currentIteration,
            previousOutput: lastOutput,
            feedbackHistory,
            mockErrors: scenario.mockErrors,
          });

          // Evaluate the output
          const report = await this.evaluateScenarioOutput(scenario, output);

          lastOutput = output;
          lastReport = report;

          console.log(
            `📊 Iteration ${currentIteration} Score: ${(
              report.overallScore * 100
            ).toFixed(1)}%`
          );

          // Check if we meet success criteria
          if (report.overallPassed) {
            console.log(`✅ Success on iteration ${currentIteration}!`);
            break;
          }

          // Generate feedback for next iteration
          const feedback = this.generateFeedback(scenario, report, output);
          feedbackHistory.push(feedback);
          console.log(
            `📝 Feedback for next iteration: ${feedback.substring(0, 100)}...`
          );

          // Small delay between iterations
          if (currentIteration < maxIterations) {
            await new Promise((resolve) => setTimeout(resolve, 1000));
          }
        } catch (iterationError) {
          console.log(
            `❌ Iteration ${currentIteration} failed: ${
              (iterationError as Error).message
            }`
          );
          const errorFeedback = `Error occurred: ${
            (iterationError as Error).message
          }. Please fix this issue.`;
          feedbackHistory.push(errorFeedback);

          if (currentIteration >= maxIterations) {
            throw iterationError;
          }
        }
      }

      const executionTime = Date.now() - startTime;

      const result: E2ETestResult = {
        scenario,
        report:
          lastReport ||
          this.createErrorReport(
            scenario,
            new Error("No successful iterations")
          ),
        output: lastOutput,
        executionTime,
        agentInteractions: [...this.interactions],
        success: lastReport?.overallPassed || false,
        iterations: currentIteration,
        feedbackHistory,
      };

      console.log(`\n📊 Final Result: ${scenario.name}`);
      console.log(`✅ Passed: ${result.success}`);
      console.log(
        `📈 Final Score: ${(lastReport?.overallScore * 100 || 0).toFixed(1)}%`
      );
      console.log(`🔄 Iterations: ${currentIteration}/${maxIterations}`);
      console.log(`⏱️  Total Duration: ${executionTime}ms`);
      console.log(`💬 Interactions: ${this.interactions.length}`);

      return result;
    } catch (error) {
      const executionTime = Date.now() - startTime;

      const result: E2ETestResult = {
        scenario,
        report: this.createErrorReport(scenario, error as Error),
        output: null,
        executionTime,
        agentInteractions: [...this.interactions],
        success: false,
        error: (error as Error).message,
        iterations: currentIteration || 0,
      };

      console.log(`\n❌ Scenario Failed: ${scenario.name}`);
      console.log(`💥 Error: ${(error as Error).message}`);

      return result;
    }
  }

  /**
   * Run multiple test scenarios
   */
  async runSuite(suite: E2ETestSuite): Promise<{
    suite: E2ETestSuite;
    results: E2ETestResult[];
    summary: {
      total: number;
      passed: number;
      failed: number;
      averageScore: number;
      totalDuration: number;
    };
  }> {
    console.log(`\n📋 Running Test Suite: ${suite.name}`);
    console.log(`📝 ${suite.description}`);

    // Setup
    if (suite.setup) {
      await suite.setup();
    }

    const results: E2ETestResult[] = [];

    try {
      for (const scenario of suite.scenarios) {
        const result = await this.runScenario(scenario);
        results.push(result);
      }
    } finally {
      // Teardown
      if (suite.teardown) {
        await suite.teardown();
      }
    }

    // Calculate summary
    const summary = {
      total: results.length,
      passed: results.filter((r) => r.success).length,
      failed: results.filter((r) => !r.success).length,
      averageScore:
        results.reduce((sum, r) => sum + r.report.overallScore, 0) /
        results.length,
      totalDuration: results.reduce((sum, r) => sum + r.executionTime, 0),
    };

    console.log(`\n📊 Suite Summary: ${suite.name}`);
    console.log(`📈 Passed: ${summary.passed}/${summary.total}`);
    console.log(
      `📊 Average Score: ${(summary.averageScore * 100).toFixed(1)}%`
    );
    console.log(`⏱️  Total Duration: ${summary.totalDuration}ms`);

    return {
      suite,
      results,
      summary,
    };
  }

  /**
   * Generate feedback for the AI based on evaluation results
   */
  private generateFeedback(
    scenario: TestScenario,
    report: any,
    _output: string
  ): string {
    const failedCriteria = report.criteria.filter((c: any) => !c.passed);
    const feedback: string[] = [];

    if (failedCriteria.length > 0) {
      feedback.push("The following issues were found:");

      for (const criterion of failedCriteria) {
        feedback.push(
          `- ${criterion.description}: ${
            criterion.notes || "Failed to meet requirements"
          }`
        );
      }

      feedback.push("\nPlease fix these issues and try again.");
    }

    // Add specific guidance based on scenario type
    const scenarioType = scenario.id.split("-")[0];
    switch (scenarioType) {
      case "text":
        feedback.push(
          "Remember to maintain formal language, avoid banned phrases, and ensure all required elements are present."
        );
        break;
      case "code":
        feedback.push(
          "Ensure the code is syntactically valid, follows TypeScript best practices, and meets all functional requirements."
        );
        break;
      case "design":
        feedback.push(
          "Use design tokens instead of hardcoded values, ensure semantic token usage, and maintain consistency across components."
        );
        break;
    }

    return feedback.join("\n");
  }

  /**
   * Execute a specific test scenario
   */
  private async executeScenario(
    scenario: TestScenario,
    _context?: {
      iteration: number;
      previousOutput: string | null;
      feedbackHistory: string[];
      mockErrors?: any[];
    }
  ): Promise<any> {
    // Extract the base scenario type from the ID
    const scenarioType = scenario.id.split("-")[0];
    console.log(
      "🔄 executeScenario called with ID:",
      scenario.id,
      "type:",
      scenarioType
    );

    switch (scenarioType) {
      case "text":
        return await this.executeTextTransformation(scenario);
      case "code":
        return await this.executeCodeGeneration(scenario);
      case "design":
        return await this.executeDesignTokenApplication(scenario);
      default:
        throw new Error(
          `Unknown scenario type: ${scenarioType} (from ${scenario.id})`
        );
    }
  }

  /**
   * Execute text transformation scenario
   */
  private async executeTextTransformation(
    scenario: TestScenario,
    context?: {
      iteration: number;
      previousOutput: string | null;
      feedbackHistory: string[];
      mockErrors?: any[];
    }
  ): Promise<string> {
    const inputText = scenario.input.text;
    console.log(
      "🔧 executeTextTransformation called with mockMode:",
      this.mockMode
    );

    if (this.mockMode) {
      // Check for mock errors first
      if (context?.mockErrors) {
        const mockError = context.mockErrors.find(
          (e: any) => e.iteration === context.iteration
        );
        if (mockError) {
          throw new Error(mockError.error);
        }
      }

      // Mock response for framework testing - improve based on feedback
      let mockResponse =
        "The team requires a more professional approach to stakeholder communications. The content contains informal language that should be revised for better clarity and structure. We need to ensure our messaging aligns with organizational standards.";

      // If this is an iteration with feedback, provide a better response
      if (context && context.feedbackHistory.length > 0) {
        mockResponse =
          "The organization requires a comprehensive approach to stakeholder engagement. Our communications must maintain professional standards, eliminate informal language, and ensure clarity in messaging. This strategic initiative demands careful attention to communication protocols and stakeholder expectations.";
      }

      console.log(
        "📝 Adding interaction to array, current length:",
        this.interactions.length
      );
      this.interactions.push({
        type: "tool_call",
        timestamp: new Date().toISOString(),
        details: {
          tool: "generate_text",
          input: inputText.substring(0, 100) + "...",
        },
        result: mockResponse,
        duration: 150,
      });
      console.log(
        "✅ Interaction added, new length:",
        this.interactions.length
      );

      return mockResponse;
    }

    if (!this.client) throw new Error("MCP client not initialized");

    // Use the generate_text tool to transform the text
    const startTime = Date.now();

    // Build prompt with feedback context if available
    let prompt = `Transform the following text to be more professional and formal:\n\n${inputText}`;
    let systemPrompt =
      "You are a professional editor. Transform casual text into formal, professional language while maintaining the core meaning.";

    if (context && context.feedbackHistory.length > 0) {
      prompt += `\n\nPrevious attempts and feedback:\n${context.feedbackHistory.join(
        "\n\n"
      )}\n\nPlease address the feedback and improve your response.`;
      systemPrompt +=
        " You are iterating on a previous response. Use the feedback provided to improve your work.";
    }

    // Check for mock errors on this iteration
    if (context?.mockErrors) {
      const mockError = context.mockErrors.find(
        (e: any) => e.iteration === context.iteration
      );
      if (mockError) {
        throw new Error(mockError.error);
      }
    }

    const result = await this.client.callTool("generate_text", {
      prompt,
      systemPrompt,
      maxTokens: 100, // Optimized for gemma3n:e2b speed
    });

    const duration = Date.now() - startTime;

    // Extract text from MCP tool response
    // MCP server returns: { content: [{ type: "text", text: "response" }] }
    let output = "";
    if (result && typeof result === "object") {
      if (
        result.content &&
        Array.isArray(result.content) &&
        result.content[0]?.text
      ) {
        output = result.content[0].text;
      } else if (result.text) {
        output = result.text;
      } else if (typeof result === "string") {
        output = result;
      }
    } else if (typeof result === "string") {
      output = result;
    }

    this.interactions.push({
      type: "tool_call",
      timestamp: new Date().toISOString(),
      details: {
        tool: "generate_text",
        input: inputText.substring(0, 100) + "...",
      },
      result: output,
      duration,
    });

    // Fallback: if AI didn't work, return a simple transformation
    if (!output || output.trim().length === 0) {
      console.log("⚠️ AI generation failed, using fallback transformation");
      return `Transformed: ${inputText}`;
    }

    return output;
  }

  /**
   * Execute code generation scenario
   */
  private async executeCodeGeneration(
    scenario: TestScenario,
    context?: {
      iteration: number;
      previousOutput: string | null;
      feedbackHistory: string[];
      mockErrors?: any[];
    }
  ): Promise<string> {
    const spec = scenario.input.specification;

    if (this.mockMode) {
      // Mock response for framework testing - improve based on feedback
      let mockResponse = `
interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  onClick,
  variant = 'primary',
  size = 'md',
  disabled = false
}) => {
  const handleClick = () => {
    if (!disabled && onClick) {
      onClick();
    }
  };

  return (
    <button
      onClick={handleClick}
      disabled={disabled}
      className={\`btn btn-\${variant} btn-\${size}\`}
    >
      {children}
    </button>
  );
};
`;

      // If this is an iteration with feedback, provide an improved response
      if (context && context.feedbackHistory.length > 0) {
        mockResponse = `
import React from 'react';

interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'small' | 'medium' | 'large';
  disabled?: boolean;
  loading?: boolean;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  onClick,
  variant = 'primary',
  size = 'medium',
  disabled = false,
  loading = false
}) => {
  const handleClick = () => {
    if (!disabled && !loading && onClick) {
      onClick();
    }
  };

  const baseClasses = 'inline-flex items-center justify-center font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';
  const variantClasses = {
    primary: 'bg-blue-600 hover:bg-blue-700 text-white focus:ring-blue-500',
    secondary: 'bg-gray-200 hover:bg-gray-300 text-gray-900 focus:ring-gray-500',
    danger: 'bg-red-600 hover:bg-red-700 text-white focus:ring-red-500'
  };
  const sizeClasses = {
    small: 'px-3 py-1.5 text-sm rounded',
    medium: 'px-4 py-2 text-base rounded-md',
    large: 'px-6 py-3 text-lg rounded-lg'
  };

  return (
    <button
      onClick={handleClick}
      disabled={disabled || loading}
      className={\`\${baseClasses} \${variantClasses[variant]} \${sizeClasses[size]}\`}
    >
      {loading && <span className="mr-2">⏳</span>}
      {children}
    </button>
  );
};
`;
      }

      this.interactions.push({
        type: "tool_call",
        timestamp: new Date().toISOString(),
        details: {
          tool: "generate_text",
          input: spec.substring(0, 100) + "...",
        },
        result: mockResponse,
        duration: 200,
      });

      return mockResponse;
    }

    if (!this.client) throw new Error("MCP client not initialized");

    // Use the generate_text tool to generate code
    const startTime = Date.now();

    // Build prompt with feedback context if available
    let prompt = `Generate a React TypeScript component based on this specification:\n\n${spec}\n\nProvide only the component code, no explanations.`;
    let systemPrompt =
      "You are an expert React TypeScript developer. Generate clean, well-typed, production-ready code.";

    if (context && context.feedbackHistory.length > 0) {
      prompt += `\n\nPrevious attempts and feedback:\n${context.feedbackHistory.join(
        "\n\n"
      )}\n\nPlease address the feedback and improve your response.`;
      systemPrompt +=
        " You are iterating on a previous response. Use the feedback provided to fix issues and improve the code.";
    }

    // Check for mock errors on this iteration
    if (context?.mockErrors) {
      const mockError = context.mockErrors.find(
        (e: any) => e.iteration === context.iteration
      );
      if (mockError) {
        throw new Error(mockError.error);
      }
    }

    const result = await this.client!.callTool("generate_text", {
      prompt,
      systemPrompt,
      maxTokens: 1000,
    });

    const duration = Date.now() - startTime;

    this.interactions.push({
      type: "tool_call",
      timestamp: new Date().toISOString(),
      details: {
        tool: "generate_text",
        input: spec.substring(0, 100) + "...",
      },
      result: result.content?.[0]?.text || "",
      duration,
    });

    return result.content?.[0]?.text || "";
  }

  /**
   * Execute design token application scenario
   */
  private async executeDesignTokenApplication(
    scenario: TestScenario,
    context?: {
      iteration: number;
      previousOutput: string | null;
      feedbackHistory: string[];
      mockErrors?: any[];
    }
  ): Promise<string> {
    const componentSpec = scenario.input.componentSpec;
    const tokens = scenario.input.tokens;

    if (this.mockMode) {
      // Mock response for framework testing - improve based on feedback
      let mockResponse = `
const Card = styled.div\`
  background-color: \${tokens.colors.bg.default};
  color: \${tokens.text.primary};
  padding: \${tokens.space.md};
  border-radius: \${tokens.radius.md};
  border: 1px solid \${tokens.colors.border.light};
\`;

const CardHeader = styled.div\`
  margin-bottom: \${tokens.space.md};
  font-size: \${tokens.typography.fontSize.lg};
  font-weight: 600;
\`;
`;

      // If this is an iteration with feedback, provide an improved response with more token usage
      if (context && context.feedbackHistory.length > 0) {
        mockResponse = `
import styled from 'styled-components';

interface CardProps {
  variant?: 'default' | 'elevated' | 'outlined';
  padding?: keyof typeof tokens.space;
}

export const Card = styled.div<CardProps>\`
  background-color: \${({ variant = 'default' }) =>
    variant === 'elevated' ? tokens.colors.bg.secondary : tokens.colors.bg.default};
  color: \${tokens.colors.text.primary};
  padding: \${({ padding = 'md' }) => tokens.space[padding]};
  border-radius: \${tokens.radius.md};
  border: \${({ variant }) =>
    variant === 'outlined' ? \`1px solid \${tokens.colors.border.light}\` : 'none'};
  box-shadow: \${({ variant }) =>
    variant === 'elevated' ? '0 2px 4px rgba(0, 0, 0, 0.1)' : 'none'};
  transition: box-shadow 0.2s ease-in-out;

  &:hover {
    box-shadow: \${({ variant }) =>
      variant === 'elevated' ? '0 4px 8px rgba(0, 0, 0, 0.15)' : 'none'};
  }
\`;

export const CardHeader = styled.div\`
  margin-bottom: \${tokens.space.md};
  font-size: \${tokens.typography.fontSize.lg};
  font-weight: \${tokens.typography.fontWeight.semibold};
  color: \${tokens.colors.text.primary};
  line-height: \${tokens.typography.lineHeight.tight};
\`;

export const CardContent = styled.div\`
  color: \${tokens.colors.text.secondary};
  line-height: \${tokens.typography.lineHeight.normal};
\`;
`;
      }

      this.interactions.push({
        type: "tool_call",
        timestamp: new Date().toISOString(),
        details: {
          tool: "generate_text",
          input: componentSpec.substring(0, 100) + "...",
        },
        result: mockResponse,
        duration: 180,
      });

      return mockResponse;
    }

    if (!this.client) throw new Error("MCP client not initialized");

    // Use the generate_text tool to generate styled component
    const startTime = Date.now();

    // Build prompt with feedback context if available
    let prompt = `Generate a styled React component using design tokens. Replace any hardcoded colors or spacing with token references.\n\nComponent spec: ${componentSpec}\n\nAvailable tokens: ${JSON.stringify(
      tokens,
      null,
      2
    )}\n\nUse tokens like tokens.colors.bg.default, tokens.space.md, etc.`;
    let systemPrompt =
      "You are a design system expert. Generate components that use semantic design tokens instead of hardcoded values.";

    if (context && context.feedbackHistory.length > 0) {
      prompt += `\n\nPrevious attempts and feedback:\n${context.feedbackHistory.join(
        "\n\n"
      )}\n\nPlease address the feedback and improve your response.`;
      systemPrompt +=
        " You are iterating on a previous response. Use the feedback provided to fix hardcoded values and ensure proper token usage.";
    }

    // Check for mock errors on this iteration
    if (context?.mockErrors) {
      const mockError = context.mockErrors.find(
        (e: any) => e.iteration === context.iteration
      );
      if (mockError) {
        throw new Error(mockError.error);
      }
    }

    const result = await this.client!.callTool("generate_text", {
      prompt,
      systemPrompt,
      maxTokens: 800,
    });

    const duration = Date.now() - startTime;

    this.interactions.push({
      type: "tool_call",
      timestamp: new Date().toISOString(),
      details: {
        tool: "generate_text",
        input: componentSpec.substring(0, 100) + "...",
      },
      result: result.content?.[0]?.text || "",
      duration,
    });

    return result.content?.[0]?.text || "";
  }

  /**
   * Evaluate scenario output
   */
  private async evaluateScenarioOutput(
    scenario: TestScenario,
    output: any
  ): Promise<EvaluationReport> {
    const startTime = Date.now();

    const report = await this.evaluator.evaluateOutput(
      scenario.id,
      output,
      scenario.expectedCriteria.map((c) => c.id),
      scenario.input
    );

    const duration = Date.now() - startTime;

    this.interactions.push({
      type: "evaluation",
      timestamp: new Date().toISOString(),
      details: {
        criteriaCount: scenario.expectedCriteria.length,
        overallScore: report.overallScore,
        overallPassed: report.overallPassed,
      },
      result: report.summary,
      duration,
    });

    return report;
  }

  /**
   * Create error report
   */
  private createErrorReport(
    scenario: TestScenario,
    error: Error
  ): EvaluationReport {
    return {
      taskId: scenario.id,
      criteria: [],
      overallScore: 0,
      overallPassed: false,
      summary: `Error: ${error.message}`,
      timestamp: new Date().toISOString(),
      duration: 0,
    };
  }

  /**
   * Clean shutdown
   */
  async shutdown(): Promise<void> {
    if (this.client) {
      await this.client!.shutdown();
      this.client = undefined;
    }
  }

  /**
   * Get available tools from server
   */
  async getAvailableTools(): Promise<any[]> {
    if (this.mockMode) {
      return [
        { name: "generate_text", description: "Generate text using AI" },
        { name: "register_agent", description: "Register a new agent" },
        { name: "submit_task", description: "Submit a task for execution" },
        { name: "store_memory", description: "Store information in memory" },
        {
          name: "retrieve_memory",
          description: "Retrieve information from memory",
        },
      ];
    }

    if (!this.client) throw new Error("MCP client not initialized");
    return await this.client!.listTools();
  }

  /**
   * Get available resources from server
   */
  async getAvailableResources(): Promise<any[]> {
    if (this.mockMode) {
      return [
        { uri: "agent://status", description: "Agent system status" },
        { uri: "agent://metrics", description: "System performance metrics" },
        { uri: "agent://memory/status", description: "Memory system status" },
        {
          uri: "agent://memory/insights",
          description: "Federated memory insights",
        },
      ];
    }

    if (!this.client) throw new Error("MCP client not initialized");
    return await this.client!.listResources();
  }
}

/**
 * Pre-configured test scenarios
 */
export const E2E_SCENARIOS: TestScenario[] = [
  {
    id: "text-transformation",
    name: "Text Transformation",
    description: "Transform casual text into professional language",
    input: {
      text: "Hey team, this is a really casual message that needs to be made more professional. It's got some informal language and could use better structure. Let's make it work better for our stakeholders.",
      bannedPhrases: ["hey team", "really casual", "let's make it work"],
      requiredElements: ["professional", "stakeholders"],
    },
    expectedCriteria: TEXT_TRANSFORMATION_CRITERIA,
    timeout: 30000,
  },
  {
    id: "code-generation",
    name: "Code Generation",
    description: "Generate a React component with proper TypeScript types",
    input: {
      specification:
        "Create a Button component that accepts children, onClick handler, variant (primary/secondary), and size (sm/md/lg) props. Use proper TypeScript types and follow React best practices.",
      expectedFunctionality: ["component", "props", "typescript", "render"],
    },
    expectedCriteria: CODE_GENERATION_CRITERIA,
    timeout: 45000,
  },
  {
    id: "design-token-application",
    name: "Design Token Application",
    description:
      "Generate a component using design tokens instead of hardcoded values",
    input: {
      componentSpec:
        "Create a Card component with background, padding, border radius, and text color. Use design tokens for all styling values.",
      tokens: {
        colors: {
          "bg.default": "#ffffff",
          "text.primary": "#212529",
          "border.light": "#dee2e6",
        },
        space: {
          sm: "0.5rem",
          md: "1rem",
          lg: "1.5rem",
        },
        radius: {
          md: "0.375rem",
        },
      },
    },
    expectedCriteria: DESIGN_TOKEN_CRITERIA,
    timeout: 40000,
  },
];

/**
 * Create and run a comprehensive E2E test suite
 */
export async function runComprehensiveE2ETests(): Promise<{
  results: E2ETestResult[];
  summary: any;
}> {
  const runner = new E2EEvaluationRunner();

  try {
    await runner.initialize();

    const suite: E2ETestSuite = {
      name: "Agent Agency E2E Test Suite",
      description:
        "Comprehensive end-to-end testing of agent evaluation capabilities",
      scenarios: E2E_SCENARIOS,
    };

    const { results, summary } = await runner.runSuite(suite);

    return { results, summary };
  } finally {
    await runner.shutdown();
  }
}
