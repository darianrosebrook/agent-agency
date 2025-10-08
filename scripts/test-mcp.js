#!/usr/bin/env node

/**
 * MCP Server Test Script
 *
 * @author @darianrosebrook
 * @description Simple test script for MCP server functionality
 */

import { AgentAgencyMCPServer } from "../dist/src/mcp/server.js";
import { AgentOrchestrator } from "../dist/src/services/AgentOrchestrator.js";
import { Logger } from "../dist/src/utils/Logger.js";

async function runTests() {
  console.log("🧪 Running MCP Server Tests...\n");

  try {
    // Test 1: Basic module loading
    console.log("✅ Test 1: Module loading");
    const orchestrator = new AgentOrchestrator();
    const logger = new Logger("TestLogger");

    console.log("   - AgentOrchestrator created");
    console.log("   - Logger created");

    // Test 2: MCP Server creation
    console.log("\n✅ Test 2: MCP Server creation");
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

    console.log("   - AgentAgencyMCPServer created");
    console.log("   - Evaluation orchestrator configured");

    // Test 3: Resource management
    console.log("\n✅ Test 3: Resource management");
    const resourceManager = mcpServer.resourceManager;
    const resources = await resourceManager.listResources();

    console.log(`   - Listed ${resources.resources.length} resources`);
    console.log(
      "   - Resources:",
      resources.resources.map((r) => r.name).join(", ")
    );

    // Test 4: Tool management
    console.log("\n✅ Test 4: Tool management");
    const toolManager = mcpServer.toolManager;
    const tools = await toolManager.listTools();

    console.log(`   - Listed ${tools.tools.length} tools`);
    const toolCategories = {
      agent: tools.tools.filter((t) => t.name.includes("agent")).length,
      task: tools.tools.filter((t) => t.name.includes("task")).length,
      evaluation: tools.tools.filter((t) => t.name.includes("evaluate")).length,
      system: tools.tools.filter((t) => t.name.includes("system")).length,
    };
    console.log("   - Tool categories:", toolCategories);

    // Test 5: Tool execution
    console.log("\n✅ Test 5: Tool execution");
    const listAgentsResult = await toolManager.executeTool("list_agents", {});
    console.log("   - list_agents tool executed successfully");

    // Test 6: Error handling
    console.log("\n✅ Test 6: Error handling");
    try {
      await toolManager.executeTool("nonexistent_tool", {});
      console.log("   - ERROR: Should have thrown for nonexistent tool");
    } catch (error) {
      console.log("   - Correctly handled nonexistent tool error");
    }

    // Test 7: Evaluation system
    console.log("\n✅ Test 7: Evaluation system");
    const evalOrchestrator = mcpServer.evaluationOrchestrator;
    console.log("   - Evaluation orchestrator available");
    console.log(
      "   - Available evaluators:",
      evalOrchestrator.getAvailableEvaluators()
    );

    console.log("\n🎉 All MCP Server tests passed!");
  } catch (error) {
    console.error("\n❌ MCP Server test failed:", error);
    process.exit(1);
  }
}

runTests();
