#!/usr/bin/env node

/**
 * MCP Server CLI Entry Point
 *
 * @author @darianrosebrook
 * @description Command-line interface for running the Agent Agency MCP server
 */

// Main execution with CommonJS requires
async function main() {
  try {
    // CommonJS requires for compiled CommonJS modules
    const aiModule = require("../dist/ai/index.js");
    console.log("AI Module exports:", Object.keys(aiModule));
    console.log(
      "MultiModelOrchestrator type:",
      typeof aiModule.MultiModelOrchestrator
    );
    const OllamaClient = aiModule.OllamaClient;
    const OpenAIClient = aiModule.OpenAIClient;
    const MultiModelOrchestrator = aiModule.MultiModelOrchestrator;
    const {
      AgentAgencyMCPServer,
    } = require("../dist/mcp/agent-agency-server.js");
    const {
      AgentOrchestrator,
    } = require("../dist/services/AgentOrchestrator.js");
    const {
      MultiTenantMemoryManager,
    } = require("../dist/memory/MultiTenantMemoryManager.js");

    // Parse command line arguments
    const args = process.argv.slice(2);
    const command = args[0];

    switch (command) {
      case "start":
        await startServer(
          OllamaClient,
          OpenAIClient,
          MultiModelOrchestrator,
          AgentAgencyMCPServer,
          AgentOrchestrator,
          MultiTenantMemoryManager
        );
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

async function startServer(
  OllamaClient,
  OpenAIClient,
  MultiModelOrchestrator,
  AgentAgencyMCPServer,
  AgentOrchestrator,
  MultiTenantMemoryManager
) {
  console.log("🚀 Starting Agent Agency MCP Server...");

  // Initialize the orchestrator
  const orchestrator = new AgentOrchestrator();

  // Initialize the multi-model AI orchestrator
  let aiClient;
  try {
    console.log("🤖 Initializing multi-model AI orchestrator...");

    // Create AI orchestrator (different from agent orchestrator)
    const aiOrchestrator = new MultiModelOrchestrator({
      defaultModel: "ollama-gemma",
      fallbackModels: ["openai-gpt"],
      enableCostOptimization: true,
      enableQualityRouting: true,
      maxRetries: 2,
      timeout: 30000,
    });

    // Register Ollama model
    let ollamaAvailable = false;
    try {
      const ollamaClient = new OllamaClient({
        model: process.env.OLLAMA_MODEL || "gemma3n:e2b",
        host: process.env.OLLAMA_HOST || "http://localhost:11434",
      });

      if (await ollamaClient.isAvailable()) {
        aiOrchestrator.registerModel({
          name: "ollama-gemma",
          client: ollamaClient,
          strengths: ["code_generation", "analysis", "general"],
          costPerToken: 0.0, // Free local model
          maxTokens: 2048, // Reduced for faster inference
          contextWindow: 4096, // Adjusted for gemma3n:e2b
          supportsToolCalling: false,
          priority: 10,
        });
        ollamaAvailable = true;
        console.log("✅ Ollama model registered");
      }
    } catch (error) {
      console.log("⚠️  Ollama model not available:", error.message);
    }

    // Register OpenAI model if API key is available
    if (process.env.OPENAI_API_KEY) {
      try {
        const openaiClient = new OpenAIClient({
          apiKey: process.env.OPENAI_API_KEY,
          model: process.env.OPENAI_MODEL || "gpt-4",
          organization: process.env.OPENAI_ORG,
          timeout: 30000,
          maxRetries: 2,
        });

        if (await openaiClient.isAvailable()) {
          aiOrchestrator.registerModel({
            name: "openai-gpt",
            client: openaiClient,
            strengths: [
              "code_generation",
              "analysis",
              "creative",
              "tool_calling",
            ],
            costPerToken: 0.002, // Approximate cost per 1K tokens
            maxTokens: 4096,
            contextWindow: 128000,
            supportsToolCalling: true,
            priority: ollamaAvailable ? 5 : 15, // Higher if Ollama not available
          });
          console.log("✅ OpenAI model registered");
        }
      } catch (error) {
        console.log("⚠️  OpenAI model registration failed:", error.message);
      }
    } else {
      console.log(
        "ℹ️  OpenAI API key not provided, skipping OpenAI integration"
      );
    }

    // Check if aiOrchestrator has any models
    if (await aiOrchestrator.isAvailable()) {
      aiClient = aiOrchestrator;
      console.log("✅ Multi-model orchestrator initialized successfully");
      console.log(
        `   Registered models: ${aiOrchestrator
          .getRegisteredModels()
          .join(", ")}`
      );
    } else {
      console.log("⚠️  No AI models available");
      aiClient = null;
    }
  } catch (error) {
    console.log("⚠️  Failed to initialize AI orchestrator:", error.message);
    console.log("   AI tools will not be available");
    aiClient = null;
  }

  // Initialize the memory manager
  let memoryManager;
  try {
    console.log("🧠 Initializing memory system...");
    memoryManager = new MultiTenantMemoryManager({
      tenantIsolation: {
        enabled: true,
        defaultIsolationLevel: "federated",
        auditLogging: true,
        maxTenants: 100,
      },
      contextOffloading: {
        enabled: true,
        maxContextSize: 10000,
        compressionThreshold: 5000,
        relevanceThreshold: 0.7,
        embeddingDimensions: 768,
      },
      federatedLearning: {
        enabled: true,
        privacyLevel: "differential",
        aggregationFrequency: 3600000, // 1 hour
        minParticipants: 3,
      },
      performance: {
        cacheEnabled: true,
        cacheSize: 1000,
        batchProcessing: true,
        asyncOperations: true,
      },
    });

    // Memory system ready (no async initialization needed)
    console.log("✅ Memory system ready");
  } catch (error) {
    console.log("⚠️  Failed to initialize memory system:", error.message);
    memoryManager = null;
  }

  // Initialize the MCP server
  const mcpServer = new AgentAgencyMCPServer(
    orchestrator,
    memoryManager,
    aiClient
  );

  // Initialize MCP imports and server
  await mcpServer.initialize();

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
