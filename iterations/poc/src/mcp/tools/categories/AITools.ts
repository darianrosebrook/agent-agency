/**
 * AI Tools for MCP
 *
 * @author @darianrosebrook
 * @description MCP tools for AI model interactions and self-prompting
 */

import { Tool } from "@modelcontextprotocol/sdk/types.js";
import { MCPToolContext, ToolExecutionResult } from "../ToolManager.js";

export class AITools {
  private readonly context: MCPToolContext;

  constructor(context: MCPToolContext) {
    this.context = context;
  }

  getTools(): Tool[] {
    return [
      {
        name: "ai_generate",
        description: "Generate text using the configured AI model",
        inputSchema: {
          type: "object",
          properties: {
            prompt: {
              type: "string",
              description: "The prompt to send to the AI model",
            },
            systemPrompt: {
              type: "string",
              description: "Optional system prompt to set context",
            },
            temperature: {
              type: "number",
              description: "Temperature for generation (0.0-1.0)",
              minimum: 0.0,
              maximum: 1.0,
              default: 0.7,
            },
            maxTokens: {
              type: "number",
              description: "Maximum tokens to generate",
              minimum: 1,
              maximum: 4096,
              default: 1024,
            },
          },
          required: ["prompt"],
        },
      },
      {
        name: "ai_available",
        description: "Check if the AI model is available and ready",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      {
        name: "ai_info",
        description: "Get information about the configured AI model",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
    ];
  }

  async executeTool(name: string, args: any): Promise<ToolExecutionResult> {
    try {
      switch (name) {
        case "ai_generate":
          return await this.generate(args);
        case "ai_available":
          return await this.checkAvailability();
        case "ai_info":
          return await this.getInfo();
        default:
          return {
            content: [{ type: "text", text: `Unknown AI tool: ${name}` }],
            isError: true,
          };
      }
    } catch (error) {
      this.context.logger.error(`AI tool ${name} execution failed`, error);
      return {
        content: [
          {
            type: "text",
            text: `AI tool execution failed: ${(error as Error).message}`,
          },
        ],
        isError: true,
      };
    }
  }

  private async generate(args: {
    prompt: string;
    systemPrompt?: string;
    temperature?: number;
    maxTokens?: number;
  }): Promise<ToolExecutionResult> {
    if (!this.context.aiClient) {
      return {
        content: [{ type: "text", text: "AI client not configured" }],
        isError: true,
      };
    }

    const response = await this.context.aiClient.generate({
      prompt: args.prompt,
      systemPrompt: args.systemPrompt,
      config: {
        temperature: args.temperature ?? 0.7,
        maxTokens: args.maxTokens ?? 1024,
      },
    });

    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(
            {
              text: response.text,
              usage: response.usage,
              finishReason: response.finishReason,
            },
            null,
            2
          ),
        },
      ],
    };
  }

  private async checkAvailability(): Promise<ToolExecutionResult> {
    if (!this.context.aiClient) {
      return {
        content: [{ type: "text", text: "false" }],
      };
    }

    const available = await this.context.aiClient.isAvailable();

    return {
      content: [{ type: "text", text: available.toString() }],
    };
  }

  private async getInfo(): Promise<ToolExecutionResult> {
    if (!this.context.aiClient) {
      return {
        content: [{ type: "text", text: "AI client not configured" }],
        isError: true,
      };
    }

    const info = {
      modelName: this.context.aiClient.getModelName(),
      supportsToolCalling: this.context.aiClient.supportsToolCalling(),
    };

    return {
      content: [{ type: "text", text: JSON.stringify(info, null, 2) }],
    };
  }
}
