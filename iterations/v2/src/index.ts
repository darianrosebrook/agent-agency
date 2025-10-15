/**
 * V2 Arbiter Application Entry Point
 *
 * @fileoverview Main entry point for the V2 Arbiter orchestration system.
 * Initializes core services, database connections, and starts the application.
 *
 * @author @darianrosebrook
 */

import * as path from "path";
import { ConnectionPoolManager } from "@/database/ConnectionPoolManager";
import { Logger } from "@/observability/Logger";
import { ObserverBridge, setObserverBridge } from "@/observer";
import { ArbiterRuntime } from "@/orchestrator/runtime/ArbiterRuntime";

// Initialize logger
const logger = new Logger("V2-Arbiter");
let arbiterRuntime: ArbiterRuntime | null = null;
let observerBridge: ObserverBridge | null = null;

/**
 * Initialize application services
 */
async function initialize(): Promise<void> {
  logger.info("Initializing V2 Arbiter...");

  // Initialize database connection pool
  try {
    logger.info("Initializing database connection pool...");
    ConnectionPoolManager.getInstance().initializeFromEnv();

    // Verify database health
    const isHealthy = await ConnectionPoolManager.getInstance().healthCheck();
    if (!isHealthy) {
      throw new Error("Database health check failed");
    }

    const stats = ConnectionPoolManager.getInstance().getStats();
    logger.info("Database connection pool initialized", {
      totalConnections: stats.totalCount,
      idleConnections: stats.idleCount,
      healthStatus: stats.healthCheckStatus,
    });
  } catch (error) {
    logger.error("Failed to initialize database connection pool", { error });
    throw error;
  }

  // TODO: Initialize other services
  // - MCP Server
  // - Agent Registry
  // - Task Orchestrator
  // - Health Monitor
  // - Performance Tracking

  // Initialize observer bridge
  try {
    arbiterRuntime = new ArbiterRuntime({
      outputDir: path.resolve(process.cwd(), "iterations/v2/runtime-output"),
    });

    observerBridge = new ObserverBridge(arbiterRuntime);
    setObserverBridge(observerBridge);
    await observerBridge.start();
    logger.info("Observer bridge started");
  } catch (error) {
    logger.error("Failed to start observer bridge", { error });
  }

  logger.info("V2 Arbiter initialized successfully");
}

/**
 * Graceful shutdown handler
 */
async function shutdown(signal: string): Promise<void> {
  logger.info(`Received ${signal}, shutting down gracefully...`);

  try {
    // Shutdown database connection pool
    await ConnectionPoolManager.getInstance().shutdown();
    logger.info("Database connection pool closed");

    if (observerBridge) {
      await observerBridge.stop();
      setObserverBridge(null);
      observerBridge = null;
      logger.info("Observer bridge stopped");
    }

    if (arbiterRuntime) {
      await arbiterRuntime.stop();
      arbiterRuntime = null;
    }

    // TODO: Shutdown other services
    // - MCP Server
    // - Agent Registry
    // - Task Orchestrator
    // - Health Monitor

    logger.info("Shutdown complete");
    process.exit(0);
  } catch (error) {
    logger.error("Error during shutdown", { error });
    process.exit(1);
  }
}

/**
 * Main application entry point
 */
async function main(): Promise<void> {
  try {
    // Setup shutdown handlers
    process.on("SIGTERM", () => shutdown("SIGTERM"));
    process.on("SIGINT", () => shutdown("SIGINT"));

    // Initialize application
    await initialize();

    // TODO: Start services
    // - HTTP server (health endpoints, metrics)
    // - MCP server
    // - Task processing
    // - Performance monitoring

    logger.info("V2 Arbiter running");

    // Keep process alive
    // In a real implementation, this would be replaced by actual service loops
    // await startHttpServer();
    // await startMcpServer();
    // await startTaskProcessing();
  } catch (error) {
    logger.error("Fatal error during startup", { error });
    process.exit(1);
  }
}

// Run application if this is the main module
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error("Unhandled error:", error);
    process.exit(1);
  });
}

// Export for testing
export { initialize, shutdown };
