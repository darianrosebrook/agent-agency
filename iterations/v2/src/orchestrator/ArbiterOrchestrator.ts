/**
 * @fileoverview Arbiter Orchestrator - Main Integration Component (ARBITER-005)
 *
 * Central orchestrator that integrates all arbiter components including
 * task management, agent registry, security, health monitoring, and
 * knowledge research capabilities.
 *
 * @author @darianrosebrook
 */

import {
  KnowledgeDatabaseClient,
  KnowledgeDatabaseConfig,
} from "../database/KnowledgeDatabaseClient";
import { AgentControlConfig } from "../types/agent-prompting";
import { TaskStatus } from "../types/arbiter-orchestration";
import { PromptingEngine } from "./prompting/PromptingEngine";

import { KnowledgeSeeker } from "../knowledge/KnowledgeSeeker";
import { AgentRegistryManager } from "./AgentRegistryManager";
import { DatabaseClientFactory } from "./DatabaseClient";
import { EventEmitter, events } from "./EventEmitter";
import { HealthMonitor } from "./HealthMonitor";
import { EventTypes } from "./OrchestratorEvents";
import { RecoveryManager } from "./RecoveryManager";
import { SecurityManager } from "./SecurityManager";
import { TaskAssignmentManager } from "./TaskAssignment";
import { TaskQueue } from "./TaskQueue";
import { ResearchDetector } from "./research/ResearchDetector";
import { ResearchProvenance } from "./research/ResearchProvenance";
import { TaskResearchAugmenter } from "./research/TaskResearchAugmenter";

/**
 * Arbiter Orchestrator Configuration
 */
export interface ArbiterOrchestratorConfig {
  /** Task queue configuration */
  taskQueue: any; // Using any for now, should be TaskQueueConfig

  /** Task assignment configuration */
  taskAssignment: any; // Using any for now, should be TaskAssignmentConfig

  /** Agent registry configuration */
  agentRegistry: any; // Using any for now, should be AgentRegistryConfig

  /** Security configuration */
  security: any; // Using any for now, should be SecurityConfig

  /** Health monitoring configuration */
  healthMonitor: any; // Using any for now, should be HealthMonitorConfig

  /** Recovery management configuration */
  recoveryManager: any; // Using any for now, should be RecoveryManagerConfig

  /** Knowledge seeker configuration */
  knowledgeSeeker: any; // Using any for now, should be KnowledgeSeekerConfig

  /** Database configuration (optional - graceful degradation if not provided) */
  database?: KnowledgeDatabaseConfig;

  /** GPT-5 prompting engine configuration */
  prompting: AgentControlConfig & {
    enabled: boolean;
  };

  /** Task research system configuration (ARBITER-006 Phase 4) */
  research?: {
    enabled: boolean;
    detector?: {
      minConfidence?: number;
      maxQueries?: number;
      enableQuestionDetection?: boolean;
      enableUncertaintyDetection?: boolean;
      enableTechnicalDetection?: boolean;
    };
    augmenter?: {
      maxResultsPerQuery?: number;
      relevanceThreshold?: number;
      timeoutMs?: number;
      maxQueries?: number;
      enableCaching?: boolean;
    };
    provenance?: {
      enabled: boolean;
    };
  };

  /** General orchestrator settings */
  orchestrator: {
    enableMetrics: boolean;
    enableTracing: boolean;
    maxConcurrentTasks: number;
    taskTimeoutMs: number;
  };
}

/**
 * Arbiter Orchestrator Status
 */
export interface ArbiterOrchestratorStatus {
  /** Overall system health */
  healthy: boolean;

  /** Component statuses */
  components: {
    taskQueue: boolean;
    taskAssignment: boolean;
    agentRegistry: boolean;
    security: boolean;
    healthMonitor: boolean;
    recoveryManager: boolean;
    knowledgeSeeker: boolean;
    promptingEngine: boolean;
  };

  /** System metrics */
  metrics: {
    activeTasks: number;
    queuedTasks: number;
    registeredAgents: number;
    completedTasks: number;
    failedTasks: number;
  };

  /** Knowledge capabilities */
  knowledgeCapabilities: {
    available: boolean;
    providers: string[];
    cacheSize: number;
  };
}

/**
 * Main Arbiter Orchestrator
 *
 * Central integration point for all arbiter components providing
 * unified task orchestration, agent management, security, and
 * knowledge research capabilities.
 */
export class ArbiterOrchestrator {
  private config: ArbiterOrchestratorConfig;
  private components: {
    taskQueue: TaskQueue;
    taskAssignment: TaskAssignmentManager;
    agentRegistry: AgentRegistryManager;
    security: SecurityManager;
    healthMonitor: HealthMonitor;
    recoveryManager: RecoveryManager;
    knowledgeSeeker: KnowledgeSeeker;
    promptingEngine: PromptingEngine;
    knowledgeDbClient?: KnowledgeDatabaseClient;
    researchDetector?: ResearchDetector;
    researchAugmenter?: TaskResearchAugmenter;
    researchProvenance?: ResearchProvenance;
  };
  private databaseClient?: any;
  private eventEmitter: EventEmitter;
  private initialized = false;

  constructor(config: ArbiterOrchestratorConfig) {
    this.config = config;
    this.eventEmitter = new EventEmitter();
    this.components = undefined;

    // Initialize event listeners
    this.setupEventListeners();
  }

  /**
   * Initialize the orchestrator and all components
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      // Initialize database client if configuration provided
      let knowledgeDbClient: KnowledgeDatabaseClient | undefined;
      if (this.config.database) {
        knowledgeDbClient = new KnowledgeDatabaseClient(this.config.database);
        try {
          await knowledgeDbClient.initialize();
          console.log("Knowledge database client initialized successfully");
        } catch (error) {
          console.warn(
            "Failed to initialize knowledge database, continuing without persistence:",
            error
          );
          knowledgeDbClient = undefined;
        }
      }

      // Initialize knowledge seeker first (needed by research system)
      const knowledgeSeeker = new KnowledgeSeeker(
        this.config.knowledgeSeeker,
        knowledgeDbClient
      );

      // Initialize research components if enabled (ARBITER-006 Phase 4)
      let researchDetector: ResearchDetector | undefined;
      let researchAugmenter: TaskResearchAugmenter | undefined;
      let researchProvenance: ResearchProvenance | undefined;

      if (this.config.research?.enabled) {
        console.log(
          "Initializing task research system (ARBITER-006 Phase 4)..."
        );

        // Initialize research detector
        researchDetector = new ResearchDetector(this.config.research.detector);

        // Initialize research augmenter
        researchAugmenter = new TaskResearchAugmenter(
          knowledgeSeeker,
          researchDetector,
          this.config.research.augmenter
        );

        // Initialize research provenance if enabled
        if (this.config.research.provenance?.enabled) {
          researchProvenance = new ResearchProvenance(this.databaseClient);
        }

        console.log("Task research system initialized successfully");
      }

      // Initialize components in dependency order
      this.components = {
        taskQueue: new TaskQueue(this.config.taskQueue),
        taskAssignment: new TaskAssignmentManager(this.config.taskAssignment),
        agentRegistry: new AgentRegistryManager(this.config.agentRegistry),
        security: new SecurityManager(this.config.security),
        healthMonitor: new HealthMonitor(this.config.healthMonitor),
        recoveryManager: new RecoveryManager(this.config.recoveryManager),
        knowledgeSeeker,
        knowledgeDbClient,
        promptingEngine: new PromptingEngine({
          enabled: this.config.prompting.enabled,
          reasoningEffort: this.config.prompting.reasoningEffort,
          eagerness: this.config.prompting.eagerness,
          toolBudget: this.config.prompting.toolBudget,
          contextGathering: {
            ...this.config.prompting.contextGathering,
            knowledgeSeeker,
          },
          selfReflection: this.config.prompting.selfReflection,
          monitoring: {
            enableMetrics: this.config.orchestrator.enableMetrics,
            enableTracing: this.config.orchestrator.enableTracing,
            metricsPrefix: "arbiter-orchestrator",
          },
        }),
        researchDetector,
        researchAugmenter,
        researchProvenance,
      };

      // Initialize all components
      await Promise.all([
        this.components.taskQueue.initialize(),
        this.components.taskAssignment.initialize(),
        // Agent registry doesn't need async init
        // Security manager doesn't need async init
        // Health monitor doesn't need async init
        // Recovery manager doesn't need async init
        // Knowledge seeker doesn't need async init
      ]);

      // Setup component integrations
      await this.setupComponentIntegrations();

      this.initialized = true;

      // Emit orchestrator started event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.ORCHESTRATOR_STARTED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "ArbiterOrchestrator",
        metadata: {
          version: "2.0.0",
          components: Object.keys(this.components),
        },
      });
    } catch (error) {
      console.error("Failed to initialize Arbiter Orchestrator:", error);
      throw error;
    }
  }

  /**
   * Shutdown the orchestrator and all components
   */
  async shutdown(): Promise<void> {
    try {
      // Shutdown components if initialized
      if (this.initialized && this.components) {
        // Shutdown components in reverse order
        await Promise.all([
          this.components.taskQueue.shutdown(),
          this.components.taskAssignment.shutdown(),
        ]);

        // Shutdown database client if it exists
        if (this.components.knowledgeDbClient) {
          await this.components.knowledgeDbClient.shutdown();
        }

        // Emit orchestrator shutdown event
        events.emit({
          id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          type: EventTypes.ORCHESTRATOR_SHUTDOWN,
          timestamp: new Date(),
          severity: "info" as any,
          source: "ArbiterOrchestrator",
          metadata: {
            uptimeMs: Date.now(), // Would need to track actual uptime
            cleanShutdown: true,
          },
        });
      }

      this.initialized = false;

      // Always shutdown local event emitter
      this.eventEmitter.shutdown();
    } catch (error) {
      console.error("Error during orchestrator shutdown:", error);
      throw error;
    }
  }

  /**
   * Get orchestrator status
   */
  async getStatus(): Promise<ArbiterOrchestratorStatus> {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    const componentStatuses = await Promise.all([
      this.checkComponentHealth("taskQueue"),
      this.checkComponentHealth("taskAssignment"),
      this.checkComponentHealth("agentRegistry"),
      this.checkComponentHealth("security"),
      this.checkComponentHealth("healthMonitor"),
      this.checkComponentHealth("recoveryManager"),
      this.checkComponentHealth("knowledgeSeeker"),
      this.checkComponentHealth("promptingEngine"),
    ]);

    const healthy = componentStatuses.every((status) => status);

    const metrics = await this.getSystemMetrics();
    const knowledgeCapabilities = await this.getKnowledgeCapabilities();

    return {
      healthy,
      components: {
        taskQueue: componentStatuses[0],
        taskAssignment: componentStatuses[1],
        agentRegistry: componentStatuses[2],
        security: componentStatuses[3],
        healthMonitor: componentStatuses[4],
        recoveryManager: componentStatuses[5],
        knowledgeSeeker: componentStatuses[6],
        promptingEngine: componentStatuses[7],
      },
      metrics,
      knowledgeCapabilities,
    };
  }

  // ========================================
  // Task Management API
  // ========================================

  /**
   * Submit a task for execution
   */
  async submitTask(
    task: Task,
    credentials?: any
  ): Promise<{ taskId: string; assignmentId?: string }> {
    if (!this.initialized || !this.components) {
      // Return error result without throwing for graceful degradation
      return { taskId: "error-orchestrator-not-initialized" };
    }

    // Authenticate if credentials provided
    if (credentials) {
      const context = this.components.security.authenticate(credentials);
      if (!context) {
        throw new Error("Authentication failed");
      }
    }

    // Augment task with research if enabled (ARBITER-006 Phase 4)
    let augmentedTask = task;
    let researchContext = null;
    if (this.config.research?.enabled && this.components.researchAugmenter) {
      const researchStartTime = Date.now();
      try {
        augmentedTask = await this.components.researchAugmenter.augmentTask(
          task
        );

        if (augmentedTask.researchProvided && augmentedTask.researchContext) {
          researchContext = augmentedTask.researchContext;
          const researchDuration = Date.now() - researchStartTime;

          // Log research augmentation
          console.log(`Research augmentation completed for task ${task.id}:`, {
            findingsCount: researchContext.findings.length,
            confidence: researchContext.confidence.toFixed(2),
            durationMs: researchDuration,
          });

          // Record provenance if enabled
          if (this.components.researchProvenance) {
            await this.components.researchProvenance.recordResearch(
              task.id,
              researchContext,
              researchDuration
            );
          }
        }
      } catch (error) {
        console.warn(
          `Research augmentation failed for task ${task.id}:`,
          error
        );

        // Record failure in provenance
        if (this.components.researchProvenance) {
          await this.components.researchProvenance.recordFailure(
            task.id,
            [],
            error instanceof Error ? error : new Error(String(error)),
            Date.now() - researchStartTime
          );
        }

        // Continue with original task if research fails
        augmentedTask = task;
      }
    }

    // Apply GPT-5 prompting optimizations if enabled
    let promptingResult = null;
    if (this.config.prompting.enabled && this.components) {
      try {
        promptingResult = await this.components.promptingEngine.processTask(
          task,
          {
            complexity: task.complexity,
            type: task.type,
            accuracyRequirement: "standard", // Default, could be made configurable
          }
        );

        // Log prompting insights
        console.log(`PromptingEngine: Task ${task.id} optimized`, {
          effort: promptingResult.reasoningEffort,
          eagerness: promptingResult.eagerness,
          budget: promptingResult.toolBudget.maxCalls,
          processingTime: promptingResult.metadata.processingTimeMs,
        });
      } catch (error) {
        console.warn(
          `PromptingEngine: Failed to optimize task ${task.id}:`,
          error
        );
        // Continue with default behavior if prompting fails
      }
    }

    // Enqueue the augmented task with prompting metadata and research context
    await this.components.taskQueue.enqueueWithCredentials(
      augmentedTask,
      credentials,
      {
        promptingResult,
        researchContext,
      }
    );

    // Attempt immediate assignment if agents are available
    const assignment = await this.attemptImmediateAssignment(
      augmentedTask,
      promptingResult
    );

    return {
      taskId: augmentedTask.id,
      assignmentId: assignment?.id,
    };
  }

  /**
   * Get task status
   */
  async getTaskStatus(taskId: string): Promise<TaskStatus | null> {
    if (!this.initialized || !this.components) {
      // Return null for uninitialized state
      return null;
    }

    const taskState = await this.components.taskQueue.getTaskState(taskId);
    return taskState ? taskState.status : null;
  }

  /**
   * Cancel a task
   */
  async cancelTask(taskId: string): Promise<boolean> {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    // Task cancellation not implemented in TaskQueue yet
    // Return false for now
    console.warn(`Task cancellation not implemented for task ${taskId}`);
    return false;
  }

  // ========================================
  // Agent Management API
  // ========================================

  /**
   * Register an agent
   */
  async registerAgent(agent: AgentProfile, credentials?: any): Promise<void> {
    if (!this.initialized || !this.components) {
      // Gracefully return without throwing for uninitialized state
      return;
    }

    // Authenticate if credentials provided
    if (credentials) {
      const context = this.components.security.authenticate(credentials);
      if (!context) {
        throw new Error("Authentication failed");
      }
    }

    await this.components.agentRegistry.registerAgent(agent);
  }

  /**
   * Get agent profile
   */
  async getAgentProfile(agentId: string): Promise<AgentProfile | null> {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    return await this.components.agentRegistry.getProfile(agentId);
  }

  /**
   * Update agent performance
   */
  async updateAgentPerformance(
    agentId: string,
    performance: any
  ): Promise<void> {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    await this.components.agentRegistry.updatePerformance(agentId, performance);
  }

  // ========================================
  // Knowledge Research API
  // ========================================

  /**
   * Process a knowledge query
   */
  async processKnowledgeQuery(
    query: KnowledgeQuery
  ): Promise<KnowledgeResponse> {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    return await this.components.knowledgeSeeker.processQuery(query);
  }

  /**
   * Get knowledge seeker status
   */
  async getKnowledgeStatus(): Promise<any> {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    return await this.components.knowledgeSeeker.getStatus();
  }

  /**
   * Clear knowledge caches
   */
  async clearKnowledgeCaches(): Promise<void> {
    if (!this.initialized || !this.components) {
      // Gracefully return without throwing for uninitialized state
      return;
    }

    await this.components.knowledgeSeeker.clearCaches();
  }

  // ========================================
  // Security API
  // ========================================

  /**
   * Authenticate agent
   */
  authenticate(credentials: any): any {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    return this.components.security.authenticate(credentials);
  }

  /**
   * Authorize action
   */
  authorize(context: any, permission: any, resource?: string): boolean {
    if (!this.initialized || !this.components) {
      throw new Error("Orchestrator not initialized");
    }

    return this.components.security.authorize(context, permission, resource);
  }

  // ========================================
  // Internal Methods
  // ========================================

  /**
   * Setup component integrations
   */
  private async setupComponentIntegrations(): Promise<void> {
    // Connect to database if configured
    if (
      this.components.databaseClient &&
      !this.components.databaseClient.isConnected()
    ) {
      try {
        await this.components.databaseClient.connect();
        console.log("✅ Database client connected");

        // Initialize database schema
        await this.initializeDatabaseSchema();
        console.log("✅ Database schema initialized");
      } catch (error) {
        console.error("❌ Failed to connect to database:", error);
        console.warn(
          "⚠️ Continuing without database persistence - using in-memory storage"
        );
      }
    }

    // Setup task queue with security
    // TODO: Implement SecureTaskQueue integration
    // const secureQueue = new (await import("./TaskQueue")).SecureTaskQueue(
    //   this.components.taskQueue,
    //   this.components.security
    // );

    // Setup event forwarding between components
    this.setupEventForwarding();
  }

  /**
   * Initialize database schema for Arbiter operations
   */
  private async initializeDatabaseSchema(): Promise<void> {
    if (!this.components.databaseClient) return;

    // Initialize schema using the database client
    await this.components.databaseClient.transaction(async (tx) => {
      // Task assignments table (already defined in DatabaseClient.ts)
      await tx.query(`
        CREATE TABLE IF NOT EXISTS task_assignments (
          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
          task_id VARCHAR(255) NOT NULL,
          agent_id VARCHAR(255) NOT NULL,
          agent_name VARCHAR(255),
          agent_model_family VARCHAR(100),
          assigned_at TIMESTAMP NOT NULL,
          deadline TIMESTAMP,
          assignment_timeout_ms INTEGER,
          routing_confidence DECIMAL(3,2),
          routing_strategy VARCHAR(50),
          routing_reason TEXT,
          status VARCHAR(50) NOT NULL DEFAULT 'pending',
          acknowledged_at TIMESTAMP,
          started_at TIMESTAMP,
          completed_at TIMESTAMP,
          progress DECIMAL(5,2) DEFAULT 0,
          last_progress_update TIMESTAMP,
          error_message TEXT,
          error_code VARCHAR(100),
          assignment_metadata JSONB,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
          updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_task_assignments_task_id ON task_assignments(task_id);
        CREATE INDEX IF NOT EXISTS idx_task_assignments_agent_id ON task_assignments(agent_id);
        CREATE INDEX IF NOT EXISTS idx_task_assignments_status ON task_assignments(status);
        CREATE INDEX IF NOT EXISTS idx_task_assignments_created_at ON task_assignments(created_at);
      `);

      // Agent performance history table
      await tx.query(`
        CREATE TABLE IF NOT EXISTS agent_performance_history (
          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
          agent_id VARCHAR(255) NOT NULL,
          task_id VARCHAR(255),
          timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
          success BOOLEAN NOT NULL,
          quality_score DECIMAL(3,2),
          latency_ms INTEGER,
          error_code VARCHAR(100),
          performance_metadata JSONB,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_agent_performance_agent_id ON agent_performance_history(agent_id);
        CREATE INDEX IF NOT EXISTS idx_agent_performance_timestamp ON agent_performance_history(timestamp);
      `);

      // Provenance tracking table
      await tx.query(`
        CREATE TABLE IF NOT EXISTS provenance_entries (
          id VARCHAR(255) PRIMARY KEY,
          project_id VARCHAR(255) NOT NULL,
          timestamp TIMESTAMP NOT NULL,
          actor VARCHAR(50) NOT NULL,
          action_type VARCHAR(100) NOT NULL,
          action_description TEXT NOT NULL,
          affected_files JSONB,
          ai_attribution JSONB,
          human_intervention JSONB,
          related_task_ids JSONB,
          metadata JSONB,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_provenance_project_id ON provenance_entries(project_id);
        CREATE INDEX IF NOT EXISTS idx_provenance_timestamp ON provenance_entries(timestamp);
        CREATE INDEX IF NOT EXISTS idx_provenance_actor ON provenance_entries(actor);
      `);
    });
  }

  /**
   * Setup event listeners and forwarding
   */
  private setupEventListeners(): void {
    // Listen for task completion events to trigger agent performance updates
    events.on("task.completed", async (event: any) => {
      if (
        event.agentId &&
        event.qualityScore !== undefined &&
        this.components
      ) {
        try {
          await this.components.agentRegistry.updatePerformance(event.agentId, {
            success: true,
            qualityScore: event.qualityScore,
            latencyMs: event.durationMs,
          });
        } catch (error) {
          console.error("Failed to update agent performance:", error);
        }
      }
    });

    // Listen for task failures to trigger recovery actions
    events.on("task.failed", async (event: any) => {
      if (!this.components) return;
      try {
        const error = new Error(event.error || "Task execution failed");
        await this.components.recoveryManager.handleFailure(
          "task_execution",
          error
        );
      } catch (error) {
        console.error("Failed to handle task failure recovery:", error);
      }
    });

    // Listen for health alerts to trigger recovery
    events.on("system.resource_alert", async (event: any) => {
      if (event.severity === "critical" && this.components) {
        try {
          const error = new Error(
            `Resource exhaustion: ${event.resource} in ${event.component}`
          );
          await this.components.recoveryManager.handleFailure(
            event.component,
            error
          );
        } catch (error) {
          console.error("Failed to handle resource alert recovery:", error);
        }
      }
    });
  }

  /**
   * Setup event forwarding between components
   */
  private setupEventForwarding(): void {
    // Forward component events to main orchestrator events
    // This could be expanded to create a unified event stream
  }

  /**
   * Attempt immediate task assignment
   */
  private async attemptImmediateAssignment(
    task: Task,
    _promptingResult?: any
  ): Promise<TaskAssignment | null> {
    try {
      // Simplified assignment - in production this would use proper agent selection
      // For now, just return null to indicate queuing only
      console.log(
        `Would assign task ${task.id} of type ${task.type} to available agent`
      );
      return null;
    } catch (error) {
      console.error("Failed to attempt immediate assignment:", error);
      return null;
    }
  }

  /**
   * Check component health
   */
  private async checkComponentHealth(componentName: string): Promise<boolean> {
    if (!this.components) {
      return false;
    }

    try {
      switch (componentName) {
        case "taskQueue": {
          const queueSize = await this.components.taskQueue.size();
          return queueSize >= 0; // Simple health check
        }

        case "taskAssignment": {
          // Would need a proper health check method
          return true;
        }

        case "agentRegistry": {
          const stats = await this.components.agentRegistry.getStats();
          return stats.totalAgents >= 0;
        }

        case "security": {
          // Security manager health check
          return this.components.security ? true : false;
        }

        case "healthMonitor": {
          // Health monitor should monitor itself
          return true;
        }

        case "recoveryManager": {
          // Recovery manager health check
          return true;
        }

        case "knowledgeSeeker": {
          const knowledgeStatus =
            await this.components.knowledgeSeeker.getStatus();
          return knowledgeStatus.enabled;
        }

        case "promptingEngine": {
          if (this.config.prompting.enabled) {
            return await this.components.promptingEngine.isHealthy();
          }
          return true; // Disabled is considered healthy
        }

        default: {
          return false;
        }
      }
    } catch (error) {
      console.error(`Health check failed for ${componentName}:`, error);
      return false;
    }
  }

  /**
   * Get system-wide metrics
   */
  private async getSystemMetrics(): Promise<any> {
    if (!this.components) {
      return {
        activeTasks: 0,
        queuedTasks: 0,
        registeredAgents: 0,
        completedTasks: 0,
        failedTasks: 0,
      };
    }

    try {
      const queueSize = await this.components.taskQueue.size();
      const registryStats = await this.components.agentRegistry.getStats();

      return {
        activeTasks: 0, // Would need to track active assignments
        queuedTasks: queueSize,
        registeredAgents: registryStats.totalAgents,
        completedTasks: 0, // TODO: Track completed tasks across agents
        failedTasks: 0, // Would need to track failures
      };
    } catch (error) {
      console.error("Failed to get system metrics:", error);
      return {
        activeTasks: 0,
        queuedTasks: 0,
        registeredAgents: 0,
        completedTasks: 0,
        failedTasks: 0,
      };
    }
  }

  /**
   * Get knowledge capabilities status
   */
  private async getKnowledgeCapabilities(): Promise<any> {
    if (!this.components) {
      return {
        available: false,
        providers: [],
        cacheSize: 0,
      };
    }

    try {
      const knowledgeStatus = await this.components.knowledgeSeeker.getStatus();

      return {
        available: knowledgeStatus.enabled,
        providers: knowledgeStatus.providers.map((p) => p.name),
        cacheSize:
          knowledgeStatus.cacheStats.queryCacheSize +
          knowledgeStatus.cacheStats.resultCacheSize,
      };
    } catch (error) {
      console.error("Failed to get knowledge capabilities:", error);
      return {
        available: false,
        providers: [],
        cacheSize: 0,
      };
    }
  }
}

/**
 * Default Arbiter Orchestrator Configuration
 */
export const defaultArbiterOrchestratorConfig: ArbiterOrchestratorConfig = {
  taskQueue: {
    maxCapacity: 1000,
    defaultTimeoutMs: 300000, // 5 minutes
    maxRetries: 3,
    priorityMode: "priority",
    persistenceEnabled: true,
    securityManager: null, // Will be set during initialization
  },

  taskAssignment: {
    acknowledgmentTimeoutMs: 10000,
    maxAssignmentDurationMs: 300000,
    autoReassignmentEnabled: true,
    maxReassignmentAttempts: 3,
    progressCheckIntervalMs: 30000,
    persistenceEnabled: true,
    databaseClient: this.config.database
      ? DatabaseClientFactory.createPostgresClient({
          host: this.config.database.host,
          port: this.config.database.port,
          database: this.config.database.database,
          user: this.config.database.user,
          password: this.config.database.password,
          maxConnections: this.config.database.maxConnections || 10,
          connectionTimeoutMs:
            this.config.database.connectionTimeoutMs || 10000,
          queryTimeoutMs: 30000,
        })
      : undefined,
  },

  agentRegistry: {
    maxAgents: 100,
    cleanupIntervalMs: 300000, // 5 minutes
    persistenceEnabled: true,
  },

  security: {
    enabled: true,
    trustedAgents: [],
    adminAgents: [],
    rateLimits: {
      submitTask: {
        requestsPerWindow: 10,
        windowMs: 60000,
        blockDurationMs: 300000,
      },
      queryTasks: {
        requestsPerWindow: 30,
        windowMs: 60000,
        blockDurationMs: 60000,
      },
      updateProgress: {
        requestsPerWindow: 60,
        windowMs: 60000,
        blockDurationMs: 30000,
      },
    },
    policies: {
      maxMetadataSize: 1024,
      requireAuthentication: true,
    },
    auditLogging: true,
  },

  healthMonitor: {
    checkIntervalMs: 30000,
    unhealthyThreshold: 3,
    recoveryTimeoutMs: 300000,
  },

  recoveryManager: {
    maxConcurrentRecoveries: 5,
    recoveryTimeoutMs: 300000,
  },

  knowledgeSeeker: {
    enabled: true,
    providers: [
      {
        name: "google",
        type: "web_search" as any,
        endpoint: "https://www.googleapis.com/customsearch/v1",
        apiKey: process.env.GOOGLE_SEARCH_API_KEY,
        searchEngineId: process.env.GOOGLE_SEARCH_ENGINE_ID,
        rateLimit: {
          requestsPerMinute: 100,
          requestsPerHour: 1000,
        },
        limits: {
          maxResultsPerQuery: 10,
          maxConcurrentQueries: 5,
        },
        options: {
          safeSearch: "moderate",
          language: "en",
        },
      },
      {
        name: "bing",
        type: "web_search" as any,
        endpoint: "https://api.bing.microsoft.com/v7.0/search",
        apiKey: process.env.BING_SEARCH_API_KEY,
        rateLimit: {
          requestsPerMinute: 50,
          requestsPerHour: 500,
        },
        limits: {
          maxResultsPerQuery: 10,
          maxConcurrentQueries: 3,
        },
        options: {
          safeSearch: "Moderate",
          market: "en-US",
        },
      },
      {
        name: "mock",
        type: "web_search" as any,
        endpoint: "mock://",
        rateLimit: {
          requestsPerMinute: 100,
          requestsPerHour: 1000,
        },
        limits: {
          maxResultsPerQuery: 10,
          maxConcurrentQueries: 5,
        },
        options: {},
      },
    ],
    processor: {
      minRelevanceScore: 0.5,
      minCredibilityScore: 0.5,
      maxResultsToProcess: 10,
      diversity: {
        minSources: 1,
        minSourceTypes: 1,
        maxResultsPerDomain: 3,
      },
      quality: {
        enableCredibilityScoring: true,
        enableRelevanceFiltering: true,
        enableDuplicateDetection: true,
      },
      caching: {
        enableResultCaching: false,
        cacheTtlMs: 3600000,
      },
    },
    queryProcessing: {
      maxConcurrentQueries: 5,
      defaultTimeoutMs: 30000,
      retryAttempts: 2,
    },
    caching: {
      enableQueryCaching: true,
      enableResultCaching: false,
      cacheTtlMs: 3600000,
    },
    observability: {
      enableMetrics: true,
      enableTracing: false,
      logLevel: "info",
    },
  },

  orchestrator: {
    enableMetrics: true,
    enableTracing: true,
    maxConcurrentTasks: 50,
    taskTimeoutMs: 300000,
  },
};
