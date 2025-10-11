/**
 * Evaluation Tools for MCP
 *
 * @author @darianrosebrook
 * @description Tools for autonomous evaluation and self-improvement
 */

import { Tool } from "@modelcontextprotocol/sdk/types.js";
import { TaskType } from "../../../types/index.js";
import { MCPToolContext } from "../ToolManager.js";

export class EvaluationTools {
  constructor(private context: MCPToolContext) {}

  async getTools(): Promise<Tool[]> {
    return [
      {
        name: "evaluate_code",
        description: "Evaluate code quality with automated testing and linting",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the task being evaluated",
            },
            projectDir: {
              type: "string",
              description: "Project directory containing the code",
              default: ".",
            },
            scripts: {
              type: "object",
              properties: {
                test: { type: "string" },
                lint: { type: "string" },
                typecheck: { type: "string" },
                a11y: { type: "string" },
              },
              description: "Custom scripts to run for evaluation",
            },
            iteration: {
              type: "number",
              description: "Current iteration number for evaluation loop",
              default: 1,
            },
          },
          required: ["taskId"],
        },
      },
      {
        name: "evaluate_text",
        description: "Evaluate text quality and adherence to requirements",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the task being evaluated",
            },
            artifactPath: {
              type: "string",
              description: "Path to the text file to evaluate",
            },
            config: {
              type: "object",
              properties: {
                style: {
                  type: "string",
                  enum: ["concise", "formal", "neutral"],
                  description: "Expected text style",
                },
                maxChars: { type: "number" },
                minChars: { type: "number" },
                bannedPhrases: {
                  type: "array",
                  items: { type: "string" },
                },
                requiredPhrases: {
                  type: "array",
                  items: { type: "string" },
                },
                readingGradeMax: { type: "number" },
              },
              description: "Evaluation configuration",
            },
            iteration: {
              type: "number",
              description: "Current iteration number",
              default: 1,
            },
          },
          required: ["taskId", "artifactPath"],
        },
      },
      {
        name: "evaluate_design",
        description: "Evaluate design token compliance and consistency",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the task being evaluated",
            },
            artifactPath: {
              type: "string",
              description: "Path to the design file to evaluate",
            },
            tokenRegistry: {
              type: "string",
              description: "Path to design token registry file",
            },
            iteration: {
              type: "number",
              description: "Current iteration number",
              default: 1,
            },
          },
          required: ["taskId", "artifactPath"],
        },
      },
      {
        name: "run_evaluation_loop",
        description: "Execute a complete autonomous evaluation loop",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the task to evaluate",
            },
            taskType: {
              type: "string",
              enum: ["code", "text", "design"],
              description: "Type of task being evaluated",
            },
            artifactPath: {
              type: "string",
              description: "Path to the artifact to evaluate",
            },
            maxIterations: {
              type: "number",
              description: "Maximum number of evaluation iterations",
              default: 3,
            },
            evaluationConfig: {
              type: "object",
              description: "Task-specific evaluation configuration",
            },
          },
          required: ["taskId", "taskType", "artifactPath"],
        },
      },
      {
        name: "analyze_error_patterns",
        description: "Analyze task failures to identify patterns and generate adaptive improvements",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the failed task to analyze",
            },
            taskType: {
              type: "string",
              description: "Type of task that failed",
            },
            error: {
              type: "string",
              description: "Error message or description",
            },
            context: {
              type: "object",
              additionalProperties: true,
              description: "Additional context about the failure",
            },
            tenantId: {
              type: "string",
              description: "Tenant ID for contextual analysis",
              default: "default-tenant",
            },
          },
          required: ["taskId", "taskType", "error"],
        },
      },
      {
        name: "get_error_analytics",
        description: "Retrieve error pattern analytics and system-wide failure insights",
        inputSchema: {
          type: "object",
          properties: {
            includePatterns: {
              type: "boolean",
              description: "Include detailed error patterns",
              default: true,
            },
            timeRange: {
              type: "string",
              enum: ["1h", "24h", "7d", "30d"],
              description: "Time range for analytics",
              default: "24h",
            },
          },
        },
      },
    ];
  }

  async hasTool(name: string): Promise<boolean> {
    const tools = await this.getTools();
    return tools.some((tool) => tool.name === name);
  }

  async executeTool(name: string, args: any): Promise<any> {
    switch (name) {
      case "evaluate_code":
        return await this.evaluateCode(args);
      case "evaluate_text":
        return await this.evaluateText(args);
      case "evaluate_design":
        return await this.evaluateDesign(args);
      case "run_evaluation_loop":
        return await this.runEvaluationLoop(args);
      case "analyze_error_patterns":
        return await this.analyzeErrorPatterns(args);
      case "get_error_analytics":
        return await this.getErrorAnalytics(args);
      default:
        throw new Error(`Unknown evaluation tool: ${name}`);
    }
  }

  private async evaluateCode(args: {
    taskId: string;
    projectDir?: string;
    scripts?: {
      test?: string;
      lint?: string;
      typecheck?: string;
      a11y?: string;
    };
    iteration?: number;
  }): Promise<any> {
    try {
      // Placeholder implementation - would integrate with actual evaluation system
      this.context.logger.info(`Evaluating code for task: ${args.taskId}`);

      const evaluation = {
        taskId: args.taskId,
        type: "code",
        iteration: args.iteration || 1,
        status: "completed",
        score: 0.85,
        criteria: [
          {
            id: "tests-pass",
            description: "Unit tests pass",
            weight: 0.4,
            passed: true,
            score: 1.0,
          },
          {
            id: "lint-clean",
            description: "Linting passes",
            weight: 0.25,
            passed: true,
            score: 1.0,
          },
          {
            id: "types-ok",
            description: "Type checking passes",
            weight: 0.25,
            passed: false,
            score: 0.0,
            notes: "TypeScript compilation errors found",
          },
        ],
        timestamp: new Date().toISOString(),
        message: "Evaluation system not yet fully implemented",
      };

      return evaluation;
    } catch (error) {
      this.context.logger.error("Failed to evaluate code:", error);
      throw error;
    }
  }

  private async evaluateText(args: {
    taskId: string;
    artifactPath: string;
    config?: any;
    iteration?: number;
  }): Promise<any> {
    try {
      // Placeholder implementation
      this.context.logger.info(`Evaluating text for task: ${args.taskId}`);

      const evaluation = {
        taskId: args.taskId,
        type: "text",
        iteration: args.iteration || 1,
        status: "completed",
        score: 0.78,
        criteria: [
          {
            id: "length-band",
            description: "Text length within bounds",
            weight: 0.15,
            passed: true,
            score: 1.0,
          },
          {
            id: "style-heuristic",
            description: "Style conforms to requirements",
            weight: 0.1,
            passed: false,
            score: 0.5,
            notes: "Some informal language detected",
          },
        ],
        timestamp: new Date().toISOString(),
        message: "Evaluation system not yet fully implemented",
      };

      return evaluation;
    } catch (error) {
      this.context.logger.error("Failed to evaluate text:", error);
      throw error;
    }
  }

  private async evaluateDesign(args: {
    taskId: string;
    artifactPath: string;
    tokenRegistry?: string;
    iteration?: number;
  }): Promise<any> {
    try {
      // Placeholder implementation
      this.context.logger.info(`Evaluating design for task: ${args.taskId}`);

      const evaluation = {
        taskId: args.taskId,
        type: "design",
        iteration: args.iteration || 1,
        status: "completed",
        score: 0.92,
        criteria: [
          {
            id: "no-hardcoded-hex",
            description: "No raw hex color values",
            weight: 0.35,
            passed: true,
            score: 1.0,
          },
          {
            id: "token-coverage",
            description: "Design tokens properly used",
            weight: 0.15,
            passed: true,
            score: 1.0,
          },
        ],
        timestamp: new Date().toISOString(),
        message: "Evaluation system not yet fully implemented",
      };

      return evaluation;
    } catch (error) {
      this.context.logger.error("Failed to evaluate design:", error);
      throw error;
    }
  }

  private async runEvaluationLoop(args: {
    taskId: string;
    taskType: TaskType | "code" | "text" | "design";
    artifactPath: string;
    maxIterations?: number;
    evaluationConfig?: any;
  }): Promise<any> {
    try {
      const maxIterations = args.maxIterations || 3;
      const evaluations = [];

      this.context.logger.info(
        `Starting evaluation loop for task: ${args.taskId}`
      );

      for (let i = 1; i <= maxIterations; i++) {
        let evaluation;

        switch (args.taskType) {
          case "code":
            evaluation = await this.evaluateCode({
              taskId: args.taskId,
              iteration: i,
              ...args.evaluationConfig,
            });
            break;
          case "text":
            evaluation = await this.evaluateText({
              taskId: args.taskId,
              artifactPath: args.artifactPath,
              iteration: i,
              config: args.evaluationConfig,
            });
            break;
          case "design":
            evaluation = await this.evaluateDesign({
              taskId: args.taskId,
              artifactPath: args.artifactPath,
              iteration: i,
              ...args.evaluationConfig,
            });
            break;
          default:
            throw new Error(
              `Unsupported task type for evaluation: ${args.taskType}`
            );
        }

        evaluations.push(evaluation);

        // Check if we should stop (simplified satisficing logic)
        if (evaluation.score >= 0.85) {
          this.context.logger.info(
            `Satisficing threshold reached at iteration ${i}`
          );
          break;
        }

        // In a real implementation, you might modify the artifact here
        // and continue the loop
      }

      const finalEvaluation = evaluations[evaluations.length - 1];
      const successful = finalEvaluation.score >= 0.85;

      return {
        taskId: args.taskId,
        evaluations,
        finalEvaluation,
        iterationsCompleted: evaluations.length,
        successful,
        satisficed: successful,
        message: "Evaluation loop completed",
      };
    } catch (error) {
      this.context.logger.error("Failed to run evaluation loop:", error);
      throw error;
    }
  }

  private async analyzeErrorPatterns(args: {
    taskId: string;
    taskType: string;
    error: string;
    context?: Record<string, any>;
    tenantId?: string;
  }): Promise<any> {
    try {
      const orchestrator = this.context.orchestrator as any;

      if (!orchestrator.errorAnalyzer) {
        return {
          content: [
            {
              type: "text",
              text: "Error pattern analysis is not enabled or not available",
            },
          ],
        };
      }

      const tenantId = args.tenantId || "default-tenant";

      const analysis = await orchestrator.errorAnalyzer.analyzeFailure(
        args.taskId,
        args.taskType,
        args.error,
        args.context || {},
        tenantId
      );

      let response = `🔍 **Error Pattern Analysis for Task ${args.taskId}**

**Error:** ${args.error}
**Task Type:** ${args.taskType}
**Severity:** ${analysis.severity.toUpperCase()}
**Confidence:** ${(analysis.confidence * 100).toFixed(1)}%

**Identified Patterns:**`;

      if (analysis.patterns.length === 0) {
        response += "\n- No patterns identified (insufficient historical data)";
      } else {
        analysis.patterns.forEach((pattern, i) => {
          response += `\n${i + 1}. **${pattern.pattern}** (${pattern.category})
   - Frequency: ${pattern.frequency}
   - Confidence: ${(pattern.confidence * 100).toFixed(1)}%
   - Common Causes: ${pattern.commonCauses.slice(0, 2).join(", ")}`;
        });
      }

      response += `

**Recommendations:**
${analysis.recommendations.map(r => `- ${r}`).join("\n")}

**Adaptive Prompt Suggestion:**
${analysis.adaptivePrompt || "No specific prompt adaptations recommended"}`;

      this.context.logger.info(`Error analysis completed for task ${args.taskId}`);

      return {
        content: [
          {
            type: "text",
            text: response,
          },
        ],
        analysis, // Include full analysis for programmatic access
      };
    } catch (error) {
      this.context.logger.error("Failed to analyze error patterns:", error);
      return {
        content: [
          {
            type: "text",
            text: `Failed to analyze error patterns: ${
              error instanceof Error ? error.message : String(error)
            }`,
          },
        ],
      };
    }
  }

  private async getErrorAnalytics(args: {
    includePatterns?: boolean;
    timeRange?: string;
  }): Promise<any> {
    try {
      const orchestrator = this.context.orchestrator as any;

      if (!orchestrator.errorAnalyzer) {
        return {
          content: [
            {
              type: "text",
              text: "Error analytics are not enabled or not available",
            },
          ],
        };
      }

      const analytics = orchestrator.errorAnalyzer.getAnalytics();

      let response = `📊 **Error Pattern Analytics**

**Overview:**
- Total patterns identified: ${analytics.totalPatterns}
- Recent failures analyzed: ${analytics.recentFailures}
- Average confidence: ${(analytics.averageConfidence * 100).toFixed(1)}%

**Top Error Patterns:**
${analytics.topPatterns.slice(0, 5).map((p, i) =>
  `${i + 1}. ${p.pattern} (${p.category}) - ${p.frequency} occurrences`
).join("\n")}

**Category Breakdown:**
${Object.entries(analytics.categoryBreakdown).map(([category, count]) =>
  `- ${category}: ${count} patterns`
).join("\n")}

**Severity Distribution:**
${Object.entries(analytics.severityDistribution).map(([severity, count]) =>
  `- ${severity}: ${count} failures`
).join("\n")}`;

      if (args.includePatterns) {
        response += `

**Detailed Pattern Analysis:**
${analytics.topPatterns.map(p =>
  `\n**${p.pattern}** (${p.category})
- Frequency: ${p.frequency}
- Category: ${p.category}
- Affected tasks: ${p.frequency} (estimated)`
).join("")}`;
      }

      this.context.logger.info("Error analytics retrieved");

      return {
        content: [
          {
            type: "text",
            text: response,
          },
        ],
        analytics, // Include full analytics for programmatic access
      };
    } catch (error) {
      this.context.logger.error("Failed to get error analytics:", error);
      return {
        content: [
          {
            type: "text",
            text: `Failed to retrieve error analytics: ${
              error instanceof Error ? error.message : String(error)
            }`,
          },
        ],
      };
    }
  }
}
