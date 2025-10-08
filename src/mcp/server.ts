/**
 * MCP Server for Agent Agency
 *
 * @author @darianrosebrook
 * @description Model Context Protocol server providing resources and tools
 * for autonomous agent operation with built-in reasoning and validation
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListResourcesRequestSchema,
  ListToolsRequestSchema,
  ReadResourceRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { AgentOrchestrator } from "../services/AgentOrchestrator.js";
import { Logger } from "../utils/Logger.js";
import { EvaluationOrchestrator } from "./evaluation/EvaluationOrchestrator.js";
import { MCPResourceManager } from "./resources/ResourceManager.js";
import { MCPToolManager } from "./tools/ToolManager.js";

export interface MCPServerConfig {
  orchestrator: AgentOrchestrator;
  evaluationConfig?: {
    minScore: number;
    mandatoryGates: string[];
    iterationPolicy: {
      maxIterations: number;
      minDeltaToContinue: number;
      noChangeBudget: number;
    };
  };
}

export class AgentAgencyMCPServer {
  private readonly server: Server;
  private readonly logger: Logger;
  private readonly resourceManager: MCPResourceManager;
  private readonly toolManager: MCPToolManager;
  private readonly evaluationOrchestrator: EvaluationOrchestrator;
  private readonly orchestrator: AgentOrchestrator;

  constructor(config: MCPServerConfig) {
    this.logger = new Logger("AgentAgencyMCPServer");
    this.orchestrator = config.orchestrator;

    // Initialize MCP server
    this.server = new Server(
      {
        name: "agent-agency-mcp",
        version: "1.0.0",
      },
      {
        capabilities: {
          resources: {},
          tools: {},
        },
      }
    );

    // Initialize evaluation orchestrator
    this.evaluationOrchestrator = new EvaluationOrchestrator(
      config.evaluationConfig ?? {
        minScore: 0.85,
        mandatoryGates: ["tests-pass", "lint-clean"],
        iterationPolicy: {
          maxIterations: 3,
          minDeltaToContinue: 0.02,
          noChangeBudget: 1,
        },
      },
      this.logger
    );

    // Initialize managers
    this.resourceManager = new MCPResourceManager(
      this.orchestrator,
      this.logger
    );
    this.toolManager = new MCPToolManager(
      this.orchestrator,
      this.logger,
      this.evaluationOrchestrator
    );

    this.setupHandlers();
  }

  private setupHandlers(): void {
    // List available resources
    this.server.setRequestHandler(ListResourcesRequestSchema, async () => {
      const result = await this.resourceManager.listResources();
      return result as any; // Type assertion for MCP compatibility
    });

    // Read specific resource
    this.server.setRequestHandler(
      ReadResourceRequestSchema,
      async (request) => {
        const result = await this.resourceManager.readResource(
          request.params.uri
        );
        return result as any; // Type assertion for MCP compatibility
      }
    );

    // List available tools
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      const result = await this.toolManager.listTools();
      return result as any; // Type assertion for MCP compatibility
    });

    // Execute tool
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const result = await this.toolManager.executeTool(
        request.params.name,
        request.params.arguments || {}
      );
      return result as any; // Type assertion for MCP compatibility
    });
  }

  /**
   * Start the MCP server
   */
  async start(): Promise<void> {
    try {
      const transport = new StdioServerTransport();
      await this.server.connect(transport);
      this.logger.info("MCP Server started successfully");
    } catch (error) {
      this.logger.error("Failed to start MCP server:", error);
      throw error;
    }
  }

  /**
   * Stop the MCP server
   */
  async stop(): Promise<void> {
    try {
      await this.server.close();
      this.logger.info("MCP Server stopped");
    } catch (error) {
      this.logger.error("Error stopping MCP server:", error);
      throw error;
    }
  }
}
