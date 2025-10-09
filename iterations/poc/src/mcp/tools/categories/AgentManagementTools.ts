/**
 * Agent Management Tools for MCP
 *
 * @author @darianrosebrook
 * @description Tools for managing agent lifecycle and operations
 */

import { Tool } from "@modelcontextprotocol/sdk/types.js";
import { AgentType } from "../../../types/index.js";
import { MCPToolContext } from "../ToolManager.js";

export class AgentManagementTools {
  constructor(private context: MCPToolContext) {}

  async getTools(): Promise<Tool[]> {
    return [
      {
        name: "register_agent",
        description: "Register a new agent with the orchestrator",
        inputSchema: {
          type: "object",
          properties: {
            name: {
              type: "string",
              description: "Human-readable agent name",
            },
            type: {
              type: "string",
              enum: ["orchestrator", "worker", "monitor", "coordinator"],
              description: "Agent type",
            },
            capabilities: {
              type: "array",
              items: { type: "string" },
              description: "List of agent capabilities",
            },
            metadata: {
              type: "object",
              additionalProperties: true,
              description: "Additional agent metadata",
            },
          },
          required: ["name", "type", "capabilities"],
        },
      },
      {
        name: "update_agent",
        description: "Update an existing agent's information",
        inputSchema: {
          type: "object",
          properties: {
            agentId: {
              type: "string",
              description: "ID of the agent to update",
            },
            updates: {
              type: "object",
              properties: {
                name: { type: "string" },
                status: {
                  type: "string",
                  enum: ["idle", "active", "busy", "error", "offline"],
                },
                capabilities: {
                  type: "array",
                  items: { type: "string" },
                },
                metadata: { type: "object" },
              },
              additionalProperties: false,
            },
          },
          required: ["agentId", "updates"],
        },
      },
      {
        name: "get_agent",
        description: "Retrieve detailed information about a specific agent",
        inputSchema: {
          type: "object",
          properties: {
            agentId: {
              type: "string",
              description: "ID of the agent to retrieve",
            },
          },
          required: ["agentId"],
        },
      },
      {
        name: "list_agents",
        description: "List all registered agents with optional filtering",
        inputSchema: {
          type: "object",
          properties: {
            type: {
              type: "string",
              enum: ["orchestrator", "worker", "monitor", "coordinator"],
              description: "Filter by agent type",
            },
            status: {
              type: "string",
              enum: ["idle", "active", "busy", "error", "offline"],
              description: "Filter by agent status",
            },
            capability: {
              type: "string",
              description: "Filter by specific capability",
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
      case "register_agent":
        return await this.registerAgent(args);
      case "update_agent":
        return await this.updateAgent(args);
      case "get_agent":
        return await this.getAgent(args);
      case "list_agents":
        return await this.listAgents(args);
      default:
        throw new Error(`Unknown agent management tool: ${name}`);
    }
  }

  private async registerAgent(args: {
    name: string;
    type: AgentType;
    capabilities: string[];
    metadata?: Record<string, unknown>;
  }): Promise<{ agentId: string; agent: any }> {
    try {
      const agentId = await this.context.orchestrator.registerAgent({
        name: args.name,
        type: args.type,
        status: "idle",
        capabilities: args.capabilities,
        metadata: args.metadata || {},
      });

      const agent = this.context.orchestrator.getAgent(agentId);

      if (!agent) {
        throw new Error("Failed to retrieve registered agent");
      }

      this.context.logger.info(
        `Agent registered via MCP: ${agentId} (${args.name})`
      );

      return { agentId, agent };
    } catch (error) {
      this.context.logger.error("Failed to register agent:", error);
      throw error;
    }
  }

  private async updateAgent(args: {
    agentId: string;
    updates: Partial<{
      name: string;
      status: string;
      capabilities: string[];
      metadata: Record<string, unknown>;
    }>;
  }): Promise<{ agent: any }> {
    try {
      const existingAgent = this.context.orchestrator.getAgent(args.agentId);

      if (!existingAgent) {
        throw new Error(`Agent not found: ${args.agentId}`);
      }

      // Create updated agent (simplified - in real implementation would update in orchestrator)
      const updatedAgent = {
        ...existingAgent,
        ...args.updates,
        updatedAt: new Date(),
      };

      // In a real implementation, you would call orchestrator.updateAgent()
      // For now, we'll just return the updated data

      this.context.logger.info(`Agent updated via MCP: ${args.agentId}`);

      return { agent: updatedAgent };
    } catch (error: any) {
      this.context.logger.error("Failed to update agent:", error);
      throw error;
    }
  }

  private async getAgent(args: { agentId: string }): Promise<any> {
    try {
      const agent = this.context.orchestrator.getAgent(args.agentId);

      if (!agent) {
        throw new Error(`Agent not found: ${args.agentId}`);
      }

      return agent;
    } catch (error) {
      this.context.logger.error("Failed to get agent:", error);
      throw error;
    }
  }

  private async listAgents(args: {
    type?: AgentType;
    status?: string;
    capability?: string;
  }): Promise<{ agents: any[]; total: number; filters: any }> {
    try {
      // Simplified implementation - in a real system you'd have proper filtering
      // For now, we'll return a mock list
      const agents = [
        {
          id: "agent_001",
          name: "Data Processor",
          type: "worker",
          status: "active",
          capabilities: ["process", "analyze"],
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      ];

      let filteredAgents = agents;

      // Apply filters (simplified)
      if (args.type) {
        filteredAgents = filteredAgents.filter((a) => a.type === args.type);
      }

      if (args.status) {
        filteredAgents = filteredAgents.filter((a) => a.status === args.status);
      }

      if (args.capability) {
        filteredAgents = filteredAgents.filter((a) =>
          a.capabilities.includes(args.capability!)
        );
      }

      return {
        agents: filteredAgents,
        total: filteredAgents.length,
        filters: args,
      };
    } catch (error) {
      this.context.logger.error("Failed to list agents:", error);
      throw error;
    }
  }
}
