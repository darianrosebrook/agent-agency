/**
 * V2 Arbiter Application Entry Point
 *
 * @fileoverview Main entry point for the V2 Arbiter orchestration system.
 * Initializes core services, database connections, and starts the application.
 *
 * @author @darianrosebrook
 */

// Load environment variables from .env.local
import * as dotenv from "dotenv";
dotenv.config({ path: ".env.local" });

import { ConnectionPoolManager } from "@/database/ConnectionPoolManager";
import { PerformanceTrackerDatabaseClient } from "@/database/PerformanceTrackerDatabaseClient";
import { ArbiterMCPServer } from "@/mcp-server/ArbiterMCPServer";
import { SystemHealthMonitor } from "@/monitoring/SystemHealthMonitor";
import { Logger } from "@/observability/Logger";
import { contractValidator, CommonSchemas } from "./api/ContractValidator";
import { HealthMonitor } from "./monitoring/HealthMonitor";
import {
  ObserverBridge,
  ObserverHttpServer,
  ObserverStoreImpl,
  loadObserverConfig,
  setObserverBridge,
} from "@/observer";
import { ArbiterController } from "@/orchestrator/ArbiterController";
import {
  ArbiterOrchestrator,
  defaultArbiterOrchestratorConfig,
} from "@/orchestrator/ArbiterOrchestrator";
import { events } from "@/orchestrator/EventEmitter";
import { LearningIntegration } from "@/orchestrator/LearningIntegration";
import { TerminalSessionManager } from "@/orchestrator/TerminalSessionManager";
import { ArbiterRuntime } from "@/orchestrator/runtime/ArbiterRuntime";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { spawn } from "child_process";
import * as path from "path";

// Initialize logger
const logger = new Logger("V2-Arbiter");
let arbiterRuntime: ArbiterRuntime | null = null;
let observerBridge: ObserverBridge | null = null;
let learningIntegration: LearningIntegration | null = null;
let httpServer: ObserverHttpServer | null = null;
let mcpServer: ArbiterMCPServer | null = null;
let orchestrator: ArbiterOrchestrator | null = null;
let arbiterController: ArbiterController | null = null;
let webServerProcess: import("child_process").ChildProcess | null = null;
let healthMonitor: HealthMonitor | null = null;

/**
 * Initialize application services
 */
async function initialize(): Promise<void> {
  logger.info("Initializing V2 Arbiter...");

  // Register API contracts
  contractValidator.registerContract("observer", {
    version: "2.0.0",
    endpoints: {
      submitTask: {
        method: "POST",
        path: "/observer/tasks",
        requestSchema: CommonSchemas.TaskSubmission,
        responseSchema: CommonSchemas.TaskResponse,
        errorSchemas: {
          400: CommonSchemas.ErrorResponse,
          500: CommonSchemas.ErrorResponse,
        },
        description: "Submit a new task to the arbiter",
      },
      getStatus: {
        method: "GET",
        path: "/observer/status",
        responseSchema: CommonSchemas.StatusResponse,
        description: "Get arbiter status information",
      },
    },
  });

      // Initialize health monitoring
      // Temporarily commented out for debugging
      // healthMonitor = new HealthMonitor({
      //   checkIntervalMs: 60000, // 60 seconds (reduced frequency)
      //   metricsIntervalMs: 30000, // 30 seconds (reduced frequency)
      //   alertThresholds: {
      //     memoryUsagePercent: 90, // Higher threshold
      //     cpuUsagePercent: 80,
      //     errorRatePercent: 5,
      //     responseTimeMs: 5000,
      //   },
      // });

      // healthMonitor.on("alert-created", (alert) => {
      //   logger.warn("Health alert created", {
      //     component: alert.component,
      //     severity: alert.severity,
      //     message: alert.message,
      //   });
      // });

      // healthMonitor.on("health-checks-completed", (summary) => {
      //   const overallStatus = healthMonitor?.getOverallStatus();
      //   if (overallStatus !== "healthy") {
      //     logger.warn("System health degraded", { status: overallStatus, summary });
      //   }
      // });

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

  // Initialize ArbiterController with real services
  try {
    arbiterController = new ArbiterController({
      orchestrator: defaultArbiterOrchestratorConfig,
      runtime: {
        outputDir: path.resolve(process.cwd(), "iterations/v2/runtime-output"),
        maxConcurrentTasks: 10,
        taskTimeoutMs: 300000,
      },
      mcpServer: {
        enabled: true,
        port: 3001,
        host: "localhost",
      },
      healthMonitor: {
        enabled: true,
        checkIntervalMs: 30000,
      },
      workspace: {
        enabled: true,
        workspaceRoot: process.cwd(),
        watchPaths: ["src", "tests"],
      },
      audit: {
        enabled: true,
        logLevel: "info",
      },
    });

    await arbiterController.initialize();
    logger.info("✅ ArbiterController initialized successfully");
  } catch (error) {
    logger.error("Failed to initialize ArbiterController", { error });
    throw error;
  }

  // Initialize observer bridge with shared registry
  try {
    // Use the ArbiterRuntime from ArbiterController (which has TaskOrchestrator)
    const controllerRuntime = arbiterController.getRuntime();
    if (!controllerRuntime) {
      throw new Error("ArbiterRuntime not available from ArbiterController");
    }
    arbiterRuntime = controllerRuntime;

    observerBridge = new ObserverBridge(
      arbiterRuntime,
      undefined,
      // healthMonitor // Temporarily commented out for debugging
    );
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

  // Stop health monitoring first
  if (healthMonitor) {
    healthMonitor.stop();
    logger.info("Health monitoring stopped");
  }

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

    // Shutdown web interface
    if (webServerProcess) {
      webServerProcess.kill("SIGTERM");
      // Give it a moment to shutdown gracefully
      await new Promise((resolve) => {
        const timeout = setTimeout(() => {
          webServerProcess?.kill("SIGKILL");
          resolve(void 0);
        }, 5000);

        webServerProcess?.on("exit", () => {
          clearTimeout(timeout);
          resolve(void 0);
        });
      });
      webServerProcess = null;
      logger.info("Web interface stopped");
    }

    // Shutdown orchestrator
    if (arbiterController) {
      await arbiterController.requestArbiterStop();
      arbiterController = null;
      logger.info("ArbiterController stopped");
    }

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

    // Use the ObserverHttpServer that's already created in ObserverBridge
    if (!observerBridge) {
      throw new Error("ObserverBridge not initialized");
    }

    // Get the server instance from ObserverBridge
    httpServer = observerBridge.server;

    // The server is already started by ObserverBridge.start()
    logger.info("HTTP server started successfully (via ObserverBridge)");
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
    const _terminalManager = new TerminalSessionManager();

    // Initialize database client for metrics storage
    const databaseClient = new PerformanceTrackerDatabaseClient({
      enableQueryLogging: false,
      enableRetries: true,
      maxRetries: 3,
      retryDelayMs: 1000,
      batchSize: 100,
    });

    // Initialize database client
    await databaseClient.initialize();

    // Create system health monitor with database client
    const systemHealthMonitor = new SystemHealthMonitor({}, databaseClient);

    // Create orchestrator instance with default config and system health monitor
    orchestrator = new ArbiterOrchestrator(
      defaultArbiterOrchestratorConfig,
      undefined,
      systemHealthMonitor
    );

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
 * Start Next.js web interface for human users
 */
async function startWebInterface(): Promise<void> {
  try {
    logger.info("Starting Next.js web interface...");

    const webObserverPath = path.resolve(process.cwd(), "apps/web-observer");

    // Check if the web-observer directory exists
    const fs = await import("fs");
    if (!fs.existsSync(webObserverPath)) {
      logger.warn(
        "Web observer directory not found, skipping web interface startup"
      );
      return;
    }

    // Check if package.json exists
    const packageJsonPath = path.join(webObserverPath, "package.json");
    if (!fs.existsSync(packageJsonPath)) {
      logger.warn(
        "Web observer package.json not found, skipping web interface startup"
      );
      return;
    }

    // Start the Next.js development server with configurable port and fallback
    const webObserverPort = process.env.WEB_OBSERVER_PORT || "3000";

    // Try to find an available port with multiple fallbacks
    let actualPort = webObserverPort;
    if (webObserverPort === "3000") {
      const fallbackPorts = ["3000", "3001", "3002", "3003", "3004"];
      let portFound = false;

      for (const port of fallbackPorts) {
        try {
          const net = await import("net");
          const testServer = net.createServer();

          await new Promise<void>((resolve, reject) => {
            const timeout = setTimeout(() => {
              testServer.close();
              reject(new Error("Port check timeout"));
            }, 1000);

            testServer.listen(parseInt(port), () => {
              clearTimeout(timeout);
              testServer.close(() => {
                actualPort = port;
                portFound = true;
                resolve();
              });
            });

            testServer.on("error", (err: any) => {
              clearTimeout(timeout);
              if (err.code === "EADDRINUSE") {
                // Port is in use, try next one
                resolve();
              } else {
                reject(err);
              }
            });
          });

          if (portFound) {
            logger.info(`Found available port: ${actualPort}`);
            break;
          }
        } catch (error) {
          logger.warn(`Port ${port} check failed, trying next port`, { error });
          continue;
        }
      }

      if (!portFound) {
        logger.error(
          "No available ports found in range 3000-3004, using default"
        );
        actualPort = "3000";
      }
    }

    webServerProcess = spawn("npm", ["run", "dev"], {
      cwd: webObserverPath,
      stdio: "inherit",
      detached: false,
      env: {
        ...process.env,
        WEB_OBSERVER_PORT: actualPort,
      },
    });

    // Handle process events
    webServerProcess.on("error", (error) => {
      logger.error("Failed to start web interface process", { error });
    });

    webServerProcess.on("exit", (code, signal) => {
      logger.info("Web interface process exited", { code, signal });
    });

    // Give the server a moment to start
    await new Promise((resolve) => setTimeout(resolve, 2000));

    logger.info(
      `Next.js web interface started successfully on http://localhost:${actualPort}`
    );
  } catch (error) {
    logger.error("Failed to start web interface", { error });
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
    await startHttpServer(); // Enable HTTP server for observer functionality
    await startMcpServer();
    await startTaskProcessing();
    await startWebInterface();

    // Start health monitoring
    // if (healthMonitor) {
    //   healthMonitor.start();
    //   logger.info("Health monitoring started");
    // }

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
