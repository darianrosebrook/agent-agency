#!/usr/bin/env node

/**
 * MCP Server CLI Entry Point
 *
 * @author @darianrosebrook
 * @description Command-line interface for running the Agent Agency MCP server
 */

import { AgentOrchestrator } from "../dist/src/services/AgentOrchestrator.js";
import { AgentAgencyMCPServer } from "../dist/src/mcp/server.js";

// Parse command line arguments
const args = process.argv.slice(2);
const command = args[0];

async function main() {
  try {
    switch (command) {
      case "start":
        await startServer();
        break;
      case "help":
      default:
        showHelp();
        break;
    }
  } catch (error) {
    console.error("Error:", error.message);
    process.exit(1);
  }
}

async function startServer() {
  console.log("🚀 Starting Agent Agency MCP Server...");

  // Initialize the orchestrator
  const orchestrator = new AgentOrchestrator();

  // Initialize the MCP server
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

  // Handle graceful shutdown
  process.on("SIGINT", async () => {
    console.log("\n🛑 Received SIGINT, shutting down gracefully...");
    await mcpServer.stop();
    process.exit(0);
  });

  process.on("SIGTERM", async () => {
    console.log("\n🛑 Received SIGTERM, shutting down gracefully...");
    await mcpServer.stop();
    process.exit(0);
  });

  // Start the server
  await mcpServer.start();
}

function showHelp() {
  console.log(`
Agent Agency MCP Server

Usage:
  mcp-server <command>

Commands:
  start    Start the MCP server
  help     Show this help message

Examples:
  mcp-server start
  mcp-server help

The MCP server provides resources and tools for autonomous agent operation
with built-in reasoning and validation capabilities.

For more information, see the documentation at docs/MCP/README.md
`);
}

main().catch((error) => {
  console.error("Unhandled error:", error);
  process.exit(1);
});

