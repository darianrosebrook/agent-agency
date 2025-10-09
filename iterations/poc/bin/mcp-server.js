#!/usr/bin/env node

/**
 * MCP Server CLI Entry Point
 *
 * @author @darianrosebrook
 * @description Command-line interface for running the Agent Agency MCP server
 */

import { OllamaClient } from "../dist/src/ai/index.js";
import { AgentAgencyMCPServer } from "../dist/src/mcp/server.js";
import { AgentOrchestrator } from "../dist/src/services/AgentOrchestrator.js";

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

  // Initialize the AI client (Ollama)
  let aiClient;
  try {
    console.log("🤖 Initializing Ollama client...");
    aiClient = new OllamaClient({
      model: process.env.OLLAMA_MODEL || "gemma:3n",
      host: process.env.OLLAMA_HOST || "http://localhost:11434",
    });

    // Check if Ollama is available
    const isAvailable = await aiClient.isAvailable();
    if (isAvailable) {
      console.log("✅ Ollama client initialized successfully");
    } else {
      console.log("⚠️  Ollama client created but service may not be available");
    }
  } catch (error) {
    console.log("⚠️  Failed to initialize Ollama client:", error.message);
    console.log("   AI tools will not be available");
  }

  // Initialize the MCP server
  const mcpServer = new AgentAgencyMCPServer({
    orchestrator,
    aiClient,
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

Environment Variables:
  OLLAMA_MODEL    AI model to use (default: gemma:3n)
  OLLAMA_HOST     Ollama server host (default: http://localhost:11434)

Examples:
  mcp-server start
  OLLAMA_MODEL=gemma:2b mcp-server start
  mcp-server help

The MCP server provides resources and tools for autonomous agent operation
with built-in reasoning and validation capabilities. When Ollama is available,
AI generation tools are automatically enabled.

For more information, see the documentation at docs/MCP/README.md
`);
}

main().catch((error) => {
  console.error("Unhandled error:", error);
  process.exit(1);
});
