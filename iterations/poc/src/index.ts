/**
 * Agent Agency - Main Entry Point
 *
 * @author @darianrosebrook
 * @description Main application entry point for the agent agency system
 */

import { AgentOrchestrator } from "./services/AgentOrchestrator";
import { Logger } from "./utils/Logger";

// Multi-Tenant Memory System Exports
export { ContextOffloader } from "./memory/ContextOffloader";
export { FederatedLearningEngine } from "./memory/FederatedLearningEngine";
export { MultiTenantMemoryManager } from "./memory/MultiTenantMemoryManager";
export { TenantIsolator } from "./memory/TenantIsolator";

// Agentic RL and Thinking Budgets (Enhanced POC Features)
export { AgenticRLTrainer } from "./rl/AgenticRLTrainer";
export { ThinkingBudgetManager } from "./thinking/ThinkingBudgetManager";

// Type exports
export type * from "./types/index";

const logger = new Logger("main");

/**
 * Initialize and start the agent agency system
 */
async function main(): Promise<void> {
  try {
    logger.info("Starting Agent Agency system...");

    const orchestrator = new AgentOrchestrator();
    await orchestrator.initialize();

    logger.info("Agent Agency system started successfully");
  } catch (error) {
    logger.error("Failed to start Agent Agency system:", error);
    process.exit(1);
  }
}

// Handle graceful shutdown
process.on("SIGINT", () => {
  logger.info("Received SIGINT, shutting down gracefully...");
  process.exit(0);
});

process.on("SIGTERM", () => {
  logger.info("Received SIGTERM, shutting down gracefully...");
  process.exit(0);
});

// Start the application
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    logger.error("Unhandled error in main:", error);
    process.exit(1);
  });
}
