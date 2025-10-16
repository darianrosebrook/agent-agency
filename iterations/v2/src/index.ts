/**
 * V2 Arbiter Application Entry Point
 *
 * @fileoverview Main entry point for the V2 Arbiter orchestration system.
 * Initializes core services, database connections, and starts the application.
 *
 * @author @darianrosebrook
 */

import { ConnectionPoolManager } from "@/database/ConnectionPoolManager";
import { ArbiterMCPServer } from "@/mcp-server/ArbiterMCPServer";
import { Logger } from "@/observability/Logger";
import {
  ObserverBridge,
  ObserverHttpServer,
  ObserverStoreImpl,
  loadObserverConfig,
  setObserverBridge,
} from "@/observer";
import {
  ArbiterOrchestrator,
  defaultArbiterOrchestratorConfig,
} from "@/orchestrator/ArbiterOrchestrator";
import { events } from "@/orchestrator/EventEmitter";
import { LearningIntegration } from "@/orchestrator/LearningIntegration";
import { TerminalSessionManager } from "@/orchestrator/TerminalSessionManager";
import { ArbiterRuntime } from "@/orchestrator/runtime/ArbiterRuntime";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import * as path from "path";

// Initialize logger
const logger = new Logger("V2-Arbiter");
let arbiterRuntime: ArbiterRuntime | null = null;
let observerBridge: ObserverBridge | null = null;
let learningIntegration: LearningIntegration | null = null;
let httpServer: ObserverHttpServer | null = null;
let mcpServer: ArbiterMCPServer | null = null;
let orchestrator: ArbiterOrchestrator | null = null;

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

  // Initialize learning integration
  try {
    logger.info("Initializing learning integration...");
    const dbPool = ConnectionPoolManager.getInstance().getPool();
    learningIntegration = new LearningIntegration(dbPool, {
      enableAutoLearning: true,
      minErrorCount: 2,
      minQualityThreshold: 0.7,
    });
    await learningIntegration.initialize();
    logger.info("Learning integration initialized");
  } catch (error) {
    logger.error("Failed to initialize learning integration", { error });
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

    // Connect learning integration to runtime events
    if (arbiterRuntime && learningIntegration) {
      logger.info("Connecting learning integration to runtime events...");

      // Listen for task completion events from the runtime
      events.on("task.completed", async (event: any) => {
        if (!learningIntegration) return;
        try {
          await learningIntegration.handleTaskCompletion({
            taskId: event.taskId,
            agentId: event.agentId,
            success: event.success,
            duration: event.durationMs,
            errorMessage: event.success ? undefined : "Task failed",
            qualityScore: event.qualityScore,
            context: event.result || {},
          });
        } catch (error) {
          logger.error(
            "Failed to handle task completion in learning integration",
            { error, event }
          );
        }
      });

      // Listen for task failure events
      events.on("task.failed", async (event: any) => {
        if (!learningIntegration) return;
        try {
          await learningIntegration.handleTaskCompletion({
            taskId: event.taskId,
            agentId: event.agentId,
            success: false,
            duration: event.durationMs,
            errorMessage: event.error,
            qualityScore: 0, // Failed tasks get 0 quality
            context: {},
          });
        } catch (error) {
          logger.error(
            "Failed to handle task failure in learning integration",
            { error, event }
          );
        }
      });

      logger.info("Learning integration connected to runtime events");
    }
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

    if (learningIntegration) {
      await learningIntegration.shutdown();
      learningIntegration = null;
      logger.info("Learning integration shut down");
    }

    // Shutdown HTTP server
    if (httpServer) {
      // ObserverHttpServer may need a stop method - check if it exists
      if (typeof (httpServer as any).stop === "function") {
        await (httpServer as any).stop();
      }
      httpServer = null;
      logger.info("HTTP server stopped");
    }

    // Shutdown MCP server
    if (mcpServer) {
      await mcpServer.close();
      mcpServer = null;
      logger.info("MCP server stopped");
    }

    // Shutdown orchestrator
    if (orchestrator) {
      // Check if orchestrator has a shutdown method
      if (typeof (orchestrator as any).shutdown === "function") {
        await (orchestrator as any).shutdown();
      }
      orchestrator = null;
      logger.info("Orchestrator stopped");
    }

    logger.info("Shutdown complete");
    process.exit(0);
  } catch (error) {
    logger.error("Error during shutdown", { error });
    process.exit(1);
  }
}

/**
 * Start HTTP server for health endpoints and metrics
 */
async function startHttpServer(): Promise<void> {
  try {
    logger.info("Starting HTTP server...");

    // Load observer configuration
    const config = await loadObserverConfig();

    // Create observer store
    const store = new ObserverStoreImpl({
      dataDir: process.env.OBSERVER_DATA_DIR || "./observer-data",
      enablePersistence: true,
      enableRedaction: true,
      maxEventsInMemory: 1000,
      flushIntervalMs: 5000,
    });

    // Create a mock controller for now - TODO: implement proper ArbiterController
    const mockController = {
      ensureArbiterRunning: async () => ({ status: "running" as const }),
      requestArbiterStop: async () => ({ status: "stopped" as const }),
      submitTask: async () => ({
        taskId: "mock-task",
        status: "accepted" as const,
      }),
      executeCommand: async () => ({ acknowledged: true }),
    };

    // Create HTTP server
    httpServer = new ObserverHttpServer(config, store, mockController);

    // Start the server
    await httpServer.start();

    logger.info("HTTP server started successfully");
  } catch (error) {
    logger.error("Failed to start HTTP server", { error });
    throw error;
  }
}

/**
 * Start MCP server for external tool integration
 */
async function startMcpServer(): Promise<void> {
  try {
    logger.info("Starting MCP server...");

    // Create terminal session manager for MCP server
    const terminalManager = new TerminalSessionManager();

    // Create orchestrator instance with default config
    orchestrator = new ArbiterOrchestrator(defaultArbiterOrchestratorConfig);

    // Initialize orchestrator
    await orchestrator.initialize();

    // Create MCP server with project root and orchestrator
    mcpServer = new ArbiterMCPServer(process.cwd(), orchestrator);

    // Create stdio transport and connect
    const transport = new StdioServerTransport();
    await mcpServer.connect(transport);

    logger.info("MCP server started successfully");
  } catch (error) {
    logger.error("Failed to start MCP server", { error });
    throw error;
  }
}

/**
 * Start task processing and orchestration
 */
async function startTaskProcessing(): Promise<void> {
  try {
    logger.info("Starting task processing...");

    if (!orchestrator) {
      throw new Error("Orchestrator not initialized");
    }

    // The orchestrator is already initialized in startMcpServer
    // Additional task processing setup can go here if needed

    logger.info("Task processing started successfully");
  } catch (error) {
    logger.error("Failed to start task processing", { error });
    throw error;
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

    // Start services
    await startHttpServer();
    await startMcpServer();
    await startTaskProcessing();

    logger.info("V2 Arbiter running with all services started");
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
