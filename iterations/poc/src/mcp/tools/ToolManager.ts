/**
 * MCP Tool Manager for Agent Agency
 *
 * @author @darianrosebrook
 * @description Manages MCP tool registration and execution for agent operations
 */

import { Tool } from "@modelcontextprotocol/sdk/types.js";
import { AIModelClient } from "../../ai/index.js";
import { AgentOrchestrator } from "../../services/AgentOrchestrator.js";
import { Logger } from "../../utils/Logger.js";
import { EvaluationOrchestrator } from "../evaluation/EvaluationOrchestrator.js";
import { AITools } from "./categories/AITools.js";
import { AgentManagementTools } from "./categories/AgentManagementTools.js";
import { EvaluationTools } from "./categories/EvaluationTools.js";
import { SystemTools } from "./categories/SystemTools.js";
import { TaskManagementTools } from "./categories/TaskManagementTools.js";

export interface ToolExecutionResult {
  content: Array<{
    type: "text";
    text: string;
  }>;
  isError?: boolean;
}

export interface MCPToolContext {
  orchestrator: AgentOrchestrator;
  logger: Logger;
  evaluationOrchestrator: EvaluationOrchestrator;
  aiClient?: AIModelClient;
}

export class MCPToolManager {
  private readonly logger: Logger;
  private readonly orchestrator: AgentOrchestrator;
  private readonly evaluationOrchestrator: EvaluationOrchestrator;
  private readonly aiClient?: AIModelClient;
  private readonly agentTools: AgentManagementTools;
  private readonly taskTools: TaskManagementTools;
  private readonly evaluationTools: EvaluationTools;
  private readonly systemTools: SystemTools;
  private readonly aiTools: AITools;

  constructor(
    orchestrator: AgentOrchestrator,
    logger: Logger,
    evaluationOrchestrator: EvaluationOrchestrator,
    aiClient?: AIModelClient
  ) {
    this.orchestrator = orchestrator;
    this.logger = logger;
    this.evaluationOrchestrator = evaluationOrchestrator;
    this.aiClient = aiClient;

    const context: MCPToolContext = {
      orchestrator: this.orchestrator,
      logger: this.logger,
      evaluationOrchestrator: this.evaluationOrchestrator,
      aiClient: this.aiClient,
    };

    this.agentTools = new AgentManagementTools(context);
    this.taskTools = new TaskManagementTools(context);
    this.evaluationTools = new EvaluationTools(context);
    this.systemTools = new SystemTools(context);
    this.aiTools = new AITools(context);
  }

  /**
   * List all available tools
   */
  async listTools(): Promise<{ tools: Tool[] }> {
    try {
      const tools: Tool[] = [];

      // Agent management tools
      tools.push(...(await this.agentTools.getTools()));

      // Task management tools
      tools.push(...(await this.taskTools.getTools()));

      // Evaluation tools
      tools.push(...(await this.evaluationTools.getTools()));

      // System tools
      tools.push(...(await this.systemTools.getTools()));

      // AI tools (only if AI client is available)
      if (this.aiClient) {
        tools.push(...this.aiTools.getTools());
      }

      this.logger.debug(`Listed ${tools.length} MCP tools`);
      return { tools };
    } catch (error) {
      this.logger.error("Failed to list tools:", error);
      throw error;
    }
  }

  /**
   * Execute a tool by name
   */
  async executeTool(name: string, args: any): Promise<ToolExecutionResult> {
    try {
      this.logger.debug(`Executing tool: ${name}`, args);

      let result: any;

      // Route to appropriate tool category
      if (await this.agentTools.hasTool(name)) {
        result = await this.agentTools.executeTool(name, args);
      } else if (await this.taskTools.hasTool(name)) {
        result = await this.taskTools.executeTool(name, args);
      } else if (await this.evaluationTools.hasTool(name)) {
        result = await this.evaluationTools.executeTool(name, args);
      } else if (await this.systemTools.hasTool(name)) {
        result = await this.systemTools.executeTool(name, args);
      } else if (this.aiClient && name.startsWith("ai_")) {
        result = await this.aiTools.executeTool(name, args);
      } else {
        throw new Error(`Unknown tool: ${name}`);
      }

      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    } catch (error) {
      this.logger.error(`Tool execution failed: ${name}`, error);

      const errorMessage =
        error instanceof Error ? error.message : "Unknown error";
      return {
        content: [
          {
            type: "text",
            text: `Tool execution failed: ${errorMessage}`,
          },
        ],
        isError: true,
      };
    }
  }
}
