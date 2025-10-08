/**
 * MCP Server Integration Tests
 *
 * @author @darianrosebrook
 * @description Basic tests for MCP server integration with Agent Agency
 */

import { describe, expect, it } from "@jest/globals";

describe("MCP Server Basic Integration", () => {
  it("should load MCP server module", async () => {
    // Basic smoke test - just verify we can import the modules
    try {
      const { AgentOrchestrator } = await import(
        "../../src/services/AgentOrchestrator.js"
      );
      const { Logger } = await import("../../src/utils/Logger.js");

      const orchestrator = new AgentOrchestrator();
      const logger = new Logger("TestLogger");

      expect(orchestrator).toBeDefined();
      expect(logger).toBeDefined();

      // Initialize orchestrator
      await orchestrator.initialize();

      // Test basic orchestrator functionality
      const metrics = await orchestrator.getSystemMetrics();
      expect(metrics).toHaveProperty("totalAgents");
      expect(metrics).toHaveProperty("totalTasks");
    } catch (error) {
      console.error("MCP server integration test failed:", error);
      throw error;
    }
  });

  it("should validate MCP server structure", async () => {
    try {
      const { AgentAgencyMCPServer } = await import("../../src/mcp/server.js");
      const { AgentOrchestrator } = await import(
        "../../src/services/AgentOrchestrator.js"
      );
      const { Logger } = await import("../../src/utils/Logger.js");

      const orchestrator = new AgentOrchestrator();
      const logger = new Logger("TestLogger");

      const mcpServer = new AgentAgencyMCPServer({
        orchestrator,
        evaluationConfig: {
          minScore: 0.85,
          mandatoryGates: ["tests-pass", "lint-clean"],
          iterationPolicy: {
            maxIterations: 3,
            minDeltaToContinue: 0.02,
            noChangeBudget: 1,
          },
        },
      });

      expect(mcpServer).toBeDefined();
      expect(typeof mcpServer.start).toBe("function");
      expect(typeof mcpServer.stop).toBe("function");

      // Test internal components
      expect((mcpServer as any).orchestrator).toBe(orchestrator);
      expect((mcpServer as any).evaluationOrchestrator).toBeDefined();
    } catch (error) {
      console.error("MCP server structure validation failed:", error);
      throw error;
    }
  });

  it("should validate tool manager structure", async () => {
    try {
      const { MCPToolManager } = await import(
        "../../src/mcp/tools/ToolManager.js"
      );
      const { AgentOrchestrator } = await import(
        "../../src/services/AgentOrchestrator.js"
      );
      const { Logger } = await import("../../src/utils/Logger.js");
      const { EvaluationOrchestrator } = await import(
        "../../src/mcp/evaluation/EvaluationOrchestrator.js"
      );

      const orchestrator = new AgentOrchestrator();
      const logger = new Logger("TestLogger");
      const evaluationOrchestrator = new EvaluationOrchestrator(
        {
          minScore: 0.85,
          mandatoryGates: ["tests-pass", "lint-clean"],
          iterationPolicy: {
            maxIterations: 3,
            minDeltaToContinue: 0.02,
            noChangeBudget: 1,
          },
        },
        logger
      );

      const toolManager = new MCPToolManager(
        orchestrator,
        logger,
        evaluationOrchestrator
      );

      expect(toolManager).toBeDefined();
      expect(typeof toolManager.listTools).toBe("function");
      expect(typeof toolManager.executeTool).toBe("function");
    } catch (error) {
      console.error("Tool manager validation failed:", error);
      throw error;
    }
  });

  it("should validate resource manager structure", async () => {
    try {
      const { MCPResourceManager } = await import(
        "../../src/mcp/resources/ResourceManager.js"
      );
      const { AgentOrchestrator } = await import(
        "../../src/services/AgentOrchestrator.js"
      );
      const { Logger } = await import("../../src/utils/Logger.js");

      const orchestrator = new AgentOrchestrator();
      const logger = new Logger("TestLogger");

      const resourceManager = new MCPResourceManager(orchestrator, logger);

      expect(resourceManager).toBeDefined();
      expect(typeof resourceManager.listResources).toBe("function");
      expect(typeof resourceManager.readResource).toBe("function");
    } catch (error) {
      console.error("Resource manager validation failed:", error);
      throw error;
    }
  });
});
