/**
 * @fileoverview ArbiterController - Real implementation for orchestrator control
 *
 * Provides real control interface for the Arbiter system, replacing mock implementations
 * with actual service integration and management capabilities.
 *
 * @author @darianrosebrook
 */

import * as path from "path";
import { ArbiterMCPServer } from "../mcp-server/ArbiterMCPServer.js";
import { SystemHealthMonitor } from "../monitoring/SystemHealthMonitor.js";
import {
  AuditEventType,
  AuditLogger,
  AuditSeverity,
} from "../observability/AuditLogger.js";
import { LogLevel } from "../observability/Logger.js";
import { PerformanceTracker } from "../rl/PerformanceTracker.js";
import { TaskPriority } from "../types/task-runner.js";
import { WorkspaceStateManager } from "../workspace/WorkspaceStateManager.js";
import { AgentRegistryManager } from "./AgentRegistryManager.js";
import { ArbiterOrchestrator } from "./ArbiterOrchestrator.js";
import { TaskOrchestrator } from "./TaskOrchestrator.js";
import { ArbiterRuntime } from "./runtime/ArbiterRuntime.js";

/**
 * ArbiterController configuration
 */
export interface ArbiterControllerConfig {
  orchestrator: any; // ArbiterOrchestratorConfig
  runtime: {
    outputDir: string;
    maxConcurrentTasks?: number;
    taskTimeoutMs?: number;
  };
  mcpServer: {
    enabled: boolean;
    port?: number;
    host?: string;
  };
  healthMonitor: {
    enabled: boolean;
    checkIntervalMs?: number;
  };
  workspace: {
    enabled: boolean;
    workspaceRoot: string;
    watchPaths?: string[];
  };
  audit: {
    enabled: boolean;
    logLevel?: string;
  };
}

/**
 * Arbiter system status
 */
export interface ArbiterStatus {
  status: "running" | "stopped" | "starting" | "stopping" | "error";
  components: {
    orchestrator: boolean;
    runtime: boolean;
    mcpServer: boolean;
    healthMonitor: boolean;
    workspace: boolean;
    audit: boolean;
  };
  metrics: {
    tasksProcessed: number;
    tasksActive: number;
    tasksFailed: number;
    uptimeMs: number;
    lastHealthCheck?: Date;
  };
  error?: string;
}

/**
 * Task submission result
 */
export interface TaskSubmissionResult {
  taskId: string;
  status: "accepted" | "rejected" | "queued" | "error";
  message?: string;
  estimatedCompletionTime?: Date;
}

/**
 * Command execution result
 */
export interface CommandResult {
  acknowledged: boolean;
  result?: any;
  error?: string;
}

/**
 * Real ArbiterController implementation
 */
export class ArbiterController {
  private config: ArbiterControllerConfig;
  private orchestrator?: ArbiterOrchestrator;
  private runtime?: ArbiterRuntime;
  private mcpServer?: ArbiterMCPServer;
  private healthMonitor?: SystemHealthMonitor;
  private workspaceManager?: WorkspaceStateManager;
  private agentRegistry?: AgentRegistryManager;
  private taskOrchestrator?: TaskOrchestrator;
  private performanceTracker?: PerformanceTracker;
  private auditLogger?: AuditLogger;

  private initialized = false;
  private startTime = Date.now();
  private taskCounter = 0;

  constructor(config: ArbiterControllerConfig) {
    this.config = config;
  }

  /**
   * Initialize all arbiter services
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      console.log("🚀 Initializing ArbiterController...");

      // Initialize audit logger first
      if (this.config.audit.enabled) {
        const logLevel =
          this.config.audit.logLevel === "debug"
            ? LogLevel.DEBUG
            : this.config.audit.logLevel === "warn"
            ? LogLevel.WARN
            : this.config.audit.logLevel === "error"
            ? LogLevel.ERROR
            : LogLevel.INFO;

        this.auditLogger = new AuditLogger("ArbiterController", logLevel);
        console.log("✅ Audit logger initialized");
      }

      // Initialize workspace manager
      if (this.config.workspace.enabled) {
        this.workspaceManager = new WorkspaceStateManager({
          workspaceRoot: this.config.workspace.workspaceRoot,
          watcher: {
            watchPaths: this.config.workspace.watchPaths || ["src", "tests"],
            ignorePatterns: ["**/node_modules/**", "**/dist/**"],
            debounceMs: 100,
            recursive: true,
            followSymlinks: false,
            maxFileSize: 10 * 1024 * 1024, // 10MB
            detectBinaryFiles: true,
          },
          defaultContextCriteria: {
            maxFiles: 10,
            maxSizeBytes: 1024 * 1024,
            priorityExtensions: [".ts", ".js", ".json"],
            excludeExtensions: [".log", ".tmp"],
            excludeDirectories: ["node_modules", "dist", ".git"],
            includeBinaryFiles: false,
            relevanceKeywords: [],
            recencyWeight: 0.3,
          },
          snapshotRetentionDays: 30,
          enablePersistence: true,
          compressionLevel: 6,
        });
        await this.workspaceManager.initialize();
        console.log("✅ Workspace manager initialized");
      }

      // Initialize health monitor
      if (this.config.healthMonitor.enabled) {
        this.healthMonitor = new SystemHealthMonitor({
          collectionIntervalMs:
            this.config.healthMonitor.checkIntervalMs || 30000,
          healthCheckIntervalMs: 60000,
          retentionPeriodMs: 3600000,
          thresholds: {
            cpuWarningThreshold: 70,
            cpuCriticalThreshold: 90,
            memoryWarningThreshold: 80,
            memoryCriticalThreshold: 95,
            diskWarningThreshold: 85,
            diskCriticalThreshold: 95,
            agentErrorRateThreshold: 5,
            agentResponseTimeThreshold: 5000,
            systemErrorRateThreshold: 10,
            queueDepthThreshold: 100,
          },
          enableCircuitBreaker: true,
          circuitBreakerFailureThreshold: 5,
          circuitBreakerRecoveryTimeoutMs: 300000,
        });
        await this.healthMonitor.initialize();
        console.log("✅ Health monitor initialized");
      }

      // Initialize performance tracker
      this.performanceTracker = new PerformanceTracker({
        enabled: true,
        maxEventsInMemory: 10000,
        retentionPeriodMs: 7 * 24 * 60 * 60 * 1000, // 7 days
        batchSize: 100,
        anonymizeData: true,
      });
      console.log("✅ Performance tracker initialized");

      // Initialize agent registry
      this.agentRegistry = new AgentRegistryManager(
        {
          maxAgents: 100,
          staleAgentThresholdMs: 24 * 60 * 60 * 1000, // 24 hours
          enableAutoCleanup: true,
          cleanupIntervalMs: 60 * 60 * 1000, // 1 hour
          enablePersistence: false,
          enableSecurity: true,
        },
        this.performanceTracker
      );
      await this.agentRegistry.initialize();
      console.log("✅ Agent registry initialized");

      // Initialize task orchestrator
      this.taskOrchestrator = new TaskOrchestrator(
        {
          workerPool: {
            minPoolSize: 2,
            maxPoolSize: 10,
            workerCapabilities: [
              "analysis",
              "research",
              "computation",
              "writing",
            ],
            workerTimeout: 300000, // 5 minutes
            artifactConfig: {
              rootPath: path.join(
                this.config.runtime.outputDir,
                "worker-artifacts"
              ),
              maxFileSizeBytes: 10 * 1024 * 1024,
              maxTotalFiles: 100,
              maxPathLength: 255,
            },
          },
          queue: {
            maxSize: 100,
            priorityLevels: [
              TaskPriority.LOW,
              TaskPriority.MEDIUM,
              TaskPriority.HIGH,
              TaskPriority.CRITICAL,
            ],
            persistenceEnabled: false,
          },
          retry: {
            maxAttempts: 3,
            initialDelay: 1000,
            maxDelay: 10000,
            backoffMultiplier: 2,
          },
          routing: {
            enabled: true,
            strategy: "load_balanced",
          },
          performance: {
            trackingEnabled: true,
            metricsInterval: 60000,
          },
          pleading: {
            enabled: false,
            requiredApprovals: 0,
            timeoutHours: 1,
          },
        },
        this.agentRegistry,
        this.performanceTracker
      );
      console.log("✅ Task orchestrator initialized");

      // Initialize arbiter runtime
      this.runtime = new ArbiterRuntime({
        outputDir: this.config.runtime.outputDir,
      });
      await this.runtime.start();
      console.log("✅ Arbiter runtime initialized");

      // Initialize orchestrator with real components
      this.orchestrator = new ArbiterOrchestrator(
        this.config.orchestrator,
        this.workspaceManager,
        this.healthMonitor
      );

      // Inject real components into orchestrator
      if (this.orchestrator) {
        (this.orchestrator as any).components.agentRegistry =
          this.agentRegistry;
        (this.orchestrator as any).components.taskOrchestrator =
          this.taskOrchestrator;
        (this.orchestrator as any).components.performanceTracker =
          this.performanceTracker;
      }

      await this.orchestrator.initialize();
      console.log("✅ Arbiter orchestrator initialized");

      // Initialize MCP server
      if (this.config.mcpServer.enabled) {
        this.mcpServer = new ArbiterMCPServer(process.cwd(), this.orchestrator);
        this.mcpServer.initialize();
        console.log("✅ MCP server initialized");
      }

      this.initialized = true;
      console.log("🎉 ArbiterController fully initialized");

      // Log initialization
      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.SYSTEM_STARTUP,
          AuditSeverity.LOW,
          "system",
          "arbiter-controller",
          "initialize",
          "success",
          {
            components: Object.keys(this.getComponentStatus()),
            uptime: Date.now() - this.startTime,
          }
        );
      }
    } catch (error) {
      console.error("❌ Failed to initialize ArbiterController:", error);

      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.SYSTEM_STARTUP,
          AuditSeverity.HIGH,
          "system",
          "arbiter-controller",
          "initialize",
          "failure",
          { error: error instanceof Error ? error.message : String(error) }
        );
      }

      throw error;
    }
  }

  /**
   * Ensure arbiter is running
   */
  async ensureArbiterRunning(): Promise<{ status: "running" }> {
    if (!this.initialized) {
      await this.initialize();
    }

    const status = await this.getStatus();
    if (status.status !== "running") {
      throw new Error(`Arbiter is not running: ${status.status}`);
    }

    return { status: "running" };
  }

  /**
   * Request arbiter stop
   */
  async requestArbiterStop(): Promise<{ status: "stopped" }> {
    try {
      console.log("🛑 Stopping ArbiterController...");

      if (this.mcpServer) {
        // ArbiterMCPServer doesn't have a stop method, just nullify the reference
      }

      if (this.runtime) {
        await this.runtime.stop();
      }

      if (this.orchestrator) {
        // Note: ArbiterOrchestrator doesn't have a stop method yet
        // This would be added for graceful shutdown
      }

      if (this.healthMonitor) {
        // Note: SystemHealthMonitor doesn't have a stop method yet
        // This would be added for graceful shutdown
      }

      if (this.workspaceManager) {
        // Note: WorkspaceStateManager doesn't have a stop method yet
        // This would be added for graceful shutdown
      }

      this.initialized = false;
      console.log("✅ ArbiterController stopped");

      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.SYSTEM_SHUTDOWN,
          AuditSeverity.LOW,
          "system",
          "arbiter-controller",
          "stop",
          "success",
          {
            uptime: Date.now() - this.startTime,
          }
        );
      }

      return { status: "stopped" };
    } catch (error) {
      console.error("❌ Error stopping ArbiterController:", error);

      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.SYSTEM_SHUTDOWN,
          AuditSeverity.HIGH,
          "system",
          "arbiter-controller",
          "stop",
          "failure",
          { error: error instanceof Error ? error.message : String(error) }
        );
      }

      throw error;
    }
  }

  /**
   * Submit a task for processing
   */
  async submitTask(task: any): Promise<TaskSubmissionResult> {
    if (!this.initialized) {
      await this.initialize();
    }

    try {
      const taskId = `task-${++this.taskCounter}-${Date.now()}`;

      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.TASK_SUBMISSION,
          AuditSeverity.LOW,
          "system",
          "task-queue",
          "submit",
          "success",
          {
            taskId,
            taskType: task.type,
            description: task.description?.substring(0, 100),
          }
        );
      }

      // Submit to runtime for processing
      if (this.runtime) {
        const result = await this.runtime.submitTask({
          ...task,
          id: taskId,
        });

        return {
          taskId,
          status: "accepted",
          message: "Task accepted for processing",
          estimatedCompletionTime: new Date(Date.now() + 300000), // 5 minutes estimate
        };
      }

      return {
        taskId,
        status: "error",
        message: "Runtime not available",
      };
    } catch (error) {
      console.error("❌ Error submitting task:", error);

      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.TASK_SUBMISSION,
          AuditSeverity.HIGH,
          "system",
          "task-queue",
          "submit",
          "failure",
          { error: error instanceof Error ? error.message : String(error) }
        );
      }

      return {
        taskId: `task-error-${Date.now()}`,
        status: "error",
        message: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Execute a command
   */
  async executeCommand(command: any): Promise<CommandResult> {
    if (!this.initialized) {
      await this.initialize();
    }

    try {
      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.SYSTEM_STARTUP, // Using system event for command execution
          AuditSeverity.LOW,
          "system",
          "command-executor",
          "execute",
          "success",
          { command: command.type || "unknown" }
        );
      }

      // Route command to appropriate handler
      switch (command.type) {
        case "status":
          return {
            acknowledged: true,
            result: await this.getStatus(),
          };

        case "health":
          if (this.healthMonitor) {
            const health = await this.healthMonitor.getHealthMetrics();
            return {
              acknowledged: true,
              result: health,
            };
          }
          break;

        case "agents":
          if (this.agentRegistry) {
            const agents = await this.agentRegistry.getAllAgents();
            return {
              acknowledged: true,
              result: agents,
            };
          }
          break;

        case "tasks":
          if (this.runtime) {
            const tasks = await this.runtime.getStatus();
            return {
              acknowledged: true,
              result: tasks,
            };
          }
          break;

        default:
          return {
            acknowledged: false,
            error: `Unknown command type: ${command.type}`,
          };
      }

      return { acknowledged: true };
    } catch (error) {
      console.error("❌ Error executing command:", error);

      if (this.auditLogger) {
        await this.auditLogger.logAuditEvent(
          AuditEventType.SYSTEM_STARTUP, // Using system event for command execution
          AuditSeverity.HIGH,
          "system",
          "command-executor",
          "execute",
          "failure",
          { error: error instanceof Error ? error.message : String(error) }
        );
      }

      return {
        acknowledged: false,
        error: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Get current system status
   */
  async getStatus(): Promise<ArbiterStatus> {
    const components = this.getComponentStatus();
    const metrics = await this.getMetrics();

    // Determine overall status
    let status: ArbiterStatus["status"] = "running";
    if (!this.initialized) {
      status = "stopped";
    }

    return {
      status,
      components,
      metrics,
    };
  }

  /**
   * Get component status
   */
  private getComponentStatus(): ArbiterStatus["components"] {
    return {
      orchestrator: !!this.orchestrator,
      runtime: !!this.runtime,
      mcpServer: !!this.mcpServer,
      healthMonitor: !!this.healthMonitor,
      workspace: !!this.workspaceManager,
      audit: !!this.auditLogger,
    };
  }

  /**
   * Get system metrics
   */
  private async getMetrics(): Promise<ArbiterStatus["metrics"]> {
    const uptimeMs = Date.now() - this.startTime;

    let tasksProcessed = 0;
    let tasksActive = 0;
    let tasksFailed = 0;
    let lastHealthCheck: Date | undefined;

    try {
      if (this.runtime) {
        const runtimeStatus = await this.runtime.getStatus();
        tasksProcessed = runtimeStatus.completedTasks || 0;
        tasksActive = runtimeStatus.activeTasks || 0;
        tasksFailed = runtimeStatus.failedTasks || 0;
      }

      if (this.healthMonitor) {
        const health = await this.healthMonitor.getHealthMetrics();
        lastHealthCheck = new Date(health.timestamp);
      }
    } catch (error) {
      console.warn("⚠️ Error getting metrics:", error);
    }

    return {
      tasksProcessed,
      tasksActive,
      tasksFailed,
      uptimeMs,
      lastHealthCheck,
    };
  }

  /**
   * Get component references for external access
   */
  getComponents() {
    return {
      orchestrator: this.orchestrator,
      runtime: this.runtime,
      mcpServer: this.mcpServer,
      healthMonitor: this.healthMonitor,
      workspaceManager: this.workspaceManager,
      agentRegistry: this.agentRegistry,
      taskOrchestrator: this.taskOrchestrator,
      performanceTracker: this.performanceTracker,
      auditLogger: this.auditLogger,
    };
  }
}
