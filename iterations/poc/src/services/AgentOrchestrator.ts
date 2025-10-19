/**
 * Agent Orchestrator Service
 *
 * @author @darianrosebrook
 * @description Core service for managing and coordinating agent activities
 */

import { MultiTenantMemoryManager } from "../memory/MultiTenantMemoryManager";
import {
  Agent,
  AgentOrchestratorConfig,
  ContextualMemory,
  MultiTenantMemoryConfig,
  SystemMetrics,
  Task,
  TaskContext,
} from "../types/index";
import { Logger } from "../utils/Logger";
import { AdvancedTaskRouter, RoutingConfig } from "./AdvancedTaskRouter.js";
import { CawsConstitutionalEnforcer } from "./CawsConstitutionalEnforcer.js";
import { ErrorPatternAnalyzer } from "./ErrorPatternAnalyzer.js";

export interface MemoryAwareAgentOrchestratorConfig
  extends AgentOrchestratorConfig {
  memoryEnabled: boolean;
  memoryConfig?: Partial<MultiTenantMemoryConfig>;
  defaultTenantId?: string;
  experienceLearningEnabled: boolean;
  memoryBasedRoutingEnabled: boolean;
  advancedRoutingEnabled: boolean;
  routingConfig?: Partial<RoutingConfig>;
  errorAnalysisEnabled: boolean;
  cawsEnforcementEnabled: boolean;
  defaultTier: number;
}

export class AgentOrchestrator {
  private readonly logger: Logger;
  private readonly config: MemoryAwareAgentOrchestratorConfig;
  private agents: Map<string, Agent> = new Map();
  private tasks: Map<string, Task> = new Map();
  private isInitialized = false;
  private memoryManager?: MultiTenantMemoryManager;
  private taskRouter?: AdvancedTaskRouter;
  private errorAnalyzer?: ErrorPatternAnalyzer;
  private cawsEnforcer?: CawsConstitutionalEnforcer;

  constructor(config?: Partial<MemoryAwareAgentOrchestratorConfig>) {
    this.logger = new Logger("AgentOrchestrator");
    this.config = {
      maxConcurrentTasks: 10,
      taskTimeoutMs: 30000,
      retryAttempts: 3,
      healthCheckIntervalMs: 5000,
      memoryEnabled: true,
      experienceLearningEnabled: true,
      memoryBasedRoutingEnabled: true,
      advancedRoutingEnabled: true,
      errorAnalysisEnabled: true,
      cawsEnforcementEnabled: true,
      defaultTier: 2,
      defaultTenantId: "default-tenant",
      ...config,
    };
  }

  /**
   * Initialize the orchestrator system
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      this.logger.warn("Orchestrator already initialized");
      return;
    }

    this.logger.info("Initializing Agent Orchestrator...");

    // Initialize memory system if enabled
    if (this.config.memoryEnabled) {
      await this.initializeMemorySystem();
    }

    // Initialize advanced task router if enabled
    if (this.config.advancedRoutingEnabled) {
      await this.initializeTaskRouter();
    }

    // Initialize error pattern analyzer if enabled
    if (this.config.errorAnalysisEnabled) {
      await this.initializeErrorAnalyzer();
    }

    // Initialize CAWS constitutional enforcer if enabled
    if (this.config.cawsEnforcementEnabled) {
      await this.initializeCawsEnforcer();
    }

    // TODO: Initialize agent registry
    // TODO: Set up task queue
    // TODO: Start health monitoring

    this.isInitialized = true;
    this.logger.info("Agent Orchestrator initialized successfully");
  }

  /**
   * Initialize the advanced task router
   */
  private async initializeTaskRouter(): Promise<void> {
    try {
      this.logger.info("Initializing advanced task router...");

      const routingConfig: RoutingConfig = {
        enabled: this.config.advancedRoutingEnabled,
        priorityQueuing: true,
        predictiveRouting: true,
        loadBalancing: true,
        memoryAwareRouting: this.config.memoryBasedRoutingEnabled,
        maxConcurrentTasksPerAgent: this.config.maxConcurrentTasks,
        routingHistoryWindow: 7, // 7 days
        performancePredictionEnabled: true,
        queueTimeoutMs: 300000, // 5 minutes
        ...this.config.routingConfig,
      };

      this.taskRouter = new AdvancedTaskRouter(
        routingConfig,
        this.memoryManager
      );

      this.logger.info("Advanced task router initialized successfully");
    } catch (error) {
      this.logger.error("Failed to initialize task router:", error);
      throw error;
    }
  }

  /**
   * Initialize the error pattern analyzer
   */
  private async initializeErrorAnalyzer(): Promise<void> {
    try {
      this.logger.info("Initializing error pattern analyzer...");

      this.errorAnalyzer = new ErrorPatternAnalyzer(this.memoryManager);

      this.logger.info("Error pattern analyzer initialized successfully");
    } catch (error) {
      this.logger.error("Failed to initialize error analyzer:", error);
      throw error;
    }
  }

  /**
   * Initialize the CAWS constitutional enforcer
   */
  private async initializeCawsEnforcer(): Promise<void> {
    try {
      this.logger.info("Initializing CAWS constitutional enforcer...");

      this.cawsEnforcer = new CawsConstitutionalEnforcer(this.memoryManager);

      this.logger.info("CAWS constitutional enforcer initialized successfully");
    } catch (error) {
      this.logger.error("Failed to initialize CAWS enforcer:", error);
      throw error;
    }
  }

  /**
   * Initialize the memory management system
   */
  private async initializeMemorySystem(): Promise<void> {
    try {
      this.logger.info("Initializing memory management system...");

      // Default memory configuration
      const defaultMemoryConfig: MultiTenantMemoryConfig = {
        tenantIsolation: {
          enabled: true,
          defaultIsolationLevel: "shared",
          auditLogging: true,
          maxTenants: 50,
        },
        contextOffloading: {
          enabled: true,
          maxContextSize: 10000,
          compressionThreshold: 0.8,
          relevanceThreshold: 0.7,
          embeddingDimensions: 384,
        },
        federatedLearning: {
          enabled: false, // Disabled by default for orchestrator
          privacyLevel: "basic",
          aggregationFrequency: 3600000, // 1 hour
          minParticipants: 3,
          maxParticipants: 10,
          privacyBudget: 1.0,
          aggregationMethod: "weighted",
          learningRate: 0.1,
          convergenceThreshold: 0.01,
        },
        performance: {
          cacheEnabled: true,
          cacheSize: 1000,
          batchProcessing: true,
          asyncOperations: true,
        },
        ...this.config.memoryConfig,
      };

      this.memoryManager = new MultiTenantMemoryManager(
        defaultMemoryConfig,
        this.logger
      );

      // Register default tenant if specified
      if (this.config.defaultTenantId) {
        const tenantConfig = {
          tenantId: this.config.defaultTenantId,
          projectId: "agent-orchestrator",
          name: "Agent Orchestrator Default Tenant",
          isolationLevel: "shared" as const,
          accessPolicies: [],
          sharingRules: [],
          dataRetention: {
            defaultRetentionDays: 30,
            archivalPolicy: "delete" as const,
            complianceRequirements: [],
            backupFrequency: "weekly" as const,
          },
          encryptionEnabled: false,
          auditLogging: true,
        };

        await this.memoryManager.registerTenant(tenantConfig);
        this.logger.info(
          `Registered default tenant: ${this.config.defaultTenantId}`
        );
      }

      this.logger.info("Memory management system initialized");
    } catch (error) {
      this.logger.error("Failed to initialize memory system:", error);
      throw error;
    }
  }

  /**
   * Register a new agent with the orchestrator
   */
  async registerAgent(
    agent: Omit<Agent, "id" | "createdAt" | "updatedAt">
  ): Promise<string> {
    const agentId = this.generateId();
    const now = new Date();

    const newAgent: Agent = {
      ...agent,
      id: agentId,
      createdAt: now,
      updatedAt: now,
    };

    this.agents.set(agentId, newAgent);
    this.logger.info(`Registered agent: ${agentId} (${agent.name})`);

    return agentId;
  }

  /**
   * Submit a task for execution with memory-aware routing
   */
  async submitTask(
    task: Omit<Task, "id" | "status" | "createdAt" | "updatedAt">,
    options?: {
      tenantId?: string;
      useMemoryRouting?: boolean;
      context?: TaskContext;
      tier?: number;
      skipConstitutionCheck?: boolean;
    }
  ): Promise<string> {
    const taskId = this.generateId();
    const now = new Date();
    const tenantId =
      options?.tenantId || this.config.defaultTenantId || "default-tenant";
    const tier = options?.tier || this.config.defaultTier || 2;

    // Enforce CAWS constitution before task creation
    if (
      this.config.cawsEnforcementEnabled &&
      this.cawsEnforcer &&
      !options?.skipConstitutionCheck
    ) {
      const enforcement = await this.cawsEnforcer.enforceConstitution(
        taskId,
        tenantId,
        tier,
        {
          taskType: task.type,
          description: task.description,
          priority: task.priority,
          ...options?.context,
        }
      );

      if (!enforcement.allowed) {
        const errorMsg = `CAWS Constitutional violation: ${enforcement.violations.join(
          ", "
        )}`;
        this.logger.warn(
          `Task ${taskId} blocked by CAWS constitution:`,
          enforcement.violations
        );
        throw new Error(errorMsg);
      }

      this.logger.info(`Task ${taskId} passed CAWS constitutional check`);
    }

    const newTask: Task = {
      ...task,
      id: taskId,
      status: "pending",
      createdAt: now,
      updatedAt: now,
    };

    this.tasks.set(taskId, newTask);
    this.logger.info(`Submitted task: ${taskId} for agent: ${task.agentId}`);

    // Start budget tracking for approved tasks
    if (this.cawsEnforcer) {
      this.cawsEnforcer.startBudgetTracking(taskId, tenantId, tier);
    }

    // Use advanced routing if available and enabled
    if (this.config.advancedRoutingEnabled && this.taskRouter) {
      const tenantId =
        options?.tenantId || this.config.defaultTenantId || "default-tenant";
      const context = options?.context || this.createTaskContext(newTask);

      try {
        const routingDecision = await this.taskRouter.submitTask(
          newTask,
          tenantId,
          context
        );

        // Update task with routing decision
        if (routingDecision.selectedAgentId !== task.agentId) {
          newTask.agentId = routingDecision.selectedAgentId;
          newTask.metadata = {
            ...newTask.metadata,
            routed: true,
            originalAgentId: task.agentId,
            routingDecision,
            routingStrategy: routingDecision.routingStrategy,
            routingConfidence: routingDecision.confidence,
            estimatedLatency: routingDecision.estimatedLatency,
            expectedQuality: routingDecision.expectedQuality,
          };
          this.tasks.set(taskId, newTask);
          this.logger.info(
            `Advanced routing: Task ${taskId} assigned to ${
              routingDecision.selectedAgentId
            } (${routingDecision.routingStrategy}, ${(
              routingDecision.confidence * 100
            ).toFixed(1)}% confidence)`
          );
        }
      } catch (error) {
        this.logger.warn(
          `Advanced routing failed for task ${taskId}, falling back to basic routing:`,
          error
        );
        // Fall back to basic memory routing
        await this.fallbackMemoryRouting(newTask, taskId, options);
      }
    } else if (
      this.config.memoryBasedRoutingEnabled &&
      options?.useMemoryRouting !== false &&
      this.memoryManager
    ) {
      // Fallback to basic memory routing
      await this.fallbackMemoryRouting(newTask, taskId, options);
    }

    // TODO: Queue task for execution

    return taskId;
  }

  /**
   * Complete a task and learn from the outcome
   */
  async completeTask(
    taskId: string,
    result: any,
    outcome: "success" | "failure" | "partial",
    metadata?: Record<string, any>
  ): Promise<void> {
    const task = this.tasks.get(taskId);
    if (!task) {
      throw new Error(`Task ${taskId} not found`);
    }

    const completedAt = new Date();
    task.status = outcome === "failure" ? "failed" : "completed";
    task.updatedAt = completedAt;
    task.metadata = {
      ...task.metadata,
      completedAt,
      outcome,
      result,
      ...metadata,
    };

    this.logger.info(`Completed task: ${taskId} with outcome: ${outcome}`);

    // Notify task router of completion for load tracking
    if (this.taskRouter) {
      this.taskRouter.taskCompleted(task.agentId);
    }

    // Analyze failures for pattern recognition if error analysis is enabled
    if (outcome === "failure" && this.errorAnalyzer && this.memoryManager) {
      const tenantId =
        (task.metadata as any)?.tenantId ||
        this.config.defaultTenantId ||
        "default-tenant";

      try {
        const error =
          typeof result === "string"
            ? result
            : result?.error || result?.message || "Unknown error";

        const analysis = await this.errorAnalyzer.analyzeFailure(
          taskId,
          task.type,
          error,
          {
            agentId: task.agentId,
            taskMetadata: task.metadata,
            result,
            ...metadata,
          },
          tenantId
        );

        // Store error analysis in task metadata
        task.metadata = {
          ...task.metadata,
          errorAnalysis: analysis,
        };

        this.logger.info(
          `Error analysis completed for failed task ${taskId}: ${analysis.patterns.length} patterns identified`
        );
      } catch (analysisError) {
        this.logger.warn(
          `Error analysis failed for task ${taskId}:`,
          analysisError
        );
        // Continue with normal completion even if analysis fails
      }
    }

    // Stop budget tracking and update final usage
    if (this.cawsEnforcer) {
      // Estimate final usage from task metadata if available
      const finalFiles = (task.metadata as any)?.finalFiles || 0;
      const finalLoc = (task.metadata as any)?.finalLoc || 0;

      this.cawsEnforcer.updateBudgetUsage(
        taskId,
        finalFiles,
        finalLoc,
        `Task ${outcome}`
      );
      this.cawsEnforcer.stopBudgetTracking(taskId);
    }

    // Learn from task outcome if memory system is enabled
    if (this.config.experienceLearningEnabled && this.memoryManager) {
      await this.learnFromTaskOutcome(task, result, outcome);
    }
  }

  /**
   * Fallback memory-based routing (original logic)
   */
  private async fallbackMemoryRouting(
    task: Task,
    taskId: string,
    options?: {
      tenantId?: string;
      useMemoryRouting?: boolean;
      context?: TaskContext;
    }
  ): Promise<void> {
    if (!this.memoryManager) return;

    const tenantId =
      options?.tenantId || this.config.defaultTenantId || "default-tenant";
    const context = options?.context || this.createTaskContext(task);

    try {
      const optimalAgentId = await this.findOptimalAgentWithMemory(
        task,
        tenantId,
        context
      );
      if (optimalAgentId && optimalAgentId !== task.agentId) {
        // Update task with memory-recommended agent
        task.agentId = optimalAgentId;
        task.metadata = {
          ...task.metadata,
          memoryRouted: true,
          originalAgentId: task.agentId,
          routingReason: "memory_based_optimization",
        };
        this.tasks.set(taskId, task);
        this.logger.info(
          `Memory routing: Task ${taskId} rerouted to agent ${optimalAgentId}`
        );
      }
    } catch (error) {
      this.logger.warn(
        `Memory-based routing failed for task ${taskId}:`,
        error
      );
      // Continue with original agent assignment
    }
  }

  /**
   * Find the optimal agent for a task using memory analysis
   */
  private async findOptimalAgentWithMemory(
    task: Task,
    tenantId: string,
    context: TaskContext
  ): Promise<string | null> {
    if (!this.memoryManager) return null;

    try {
      // Query memory for similar tasks and their outcomes
      const memoryQuery: TaskContext = {
        ...context,
        type: "agent_selection",
        description: `Find optimal agent for task: ${task.description}`,
        requirements: ["agent_performance", "task_similarity"],
        constraints: {
          taskType: task.type,
          priority: task.priority,
        },
      };

      const memories = await this.memoryManager.getContextualMemories(
        tenantId,
        memoryQuery,
        {
          limit: 10,
          minRelevance: 0.6,
        }
      );

      if (!memories.success || !memories.data || memories.data.length === 0) {
        return null;
      }

      // Analyze agent performance from memories
      const agentPerformance = new Map<
        string,
        { success: number; total: number; avgRelevance: number }
      >();

      for (const memory of memories.data) {
        if (memory.content.agentId) {
          const agentId = memory.content.agentId;
          const existing = agentPerformance.get(agentId) || {
            success: 0,
            total: 0,
            avgRelevance: 0,
          };

          existing.total++;
          if (memory.content.outcome === "success") {
            existing.success++;
          }
          existing.avgRelevance =
            (existing.avgRelevance + memory.relevanceScore) / 2;

          agentPerformance.set(agentId, existing);
        }
      }

      // Find the best performing agent for this task type
      let bestAgent: string | null = null;
      let bestScore = 0;

      for (const [agentId, stats] of agentPerformance.entries()) {
        if (stats.total >= 3) {
          // Require minimum sample size
          const successRate = stats.success / stats.total;
          const relevanceBonus = stats.avgRelevance;
          const score = successRate * 0.7 + relevanceBonus * 0.3;

          if (score > bestScore && this.agents.has(agentId)) {
            bestScore = score;
            bestAgent = agentId;
          }
        }
      }

      if (bestAgent) {
        this.logger.debug(
          `Selected agent ${bestAgent} for task ${task.id} with score ${(
            bestScore * 100
          ).toFixed(1)}%`
        );
      }

      return bestAgent;
    } catch (error) {
      this.logger.warn(`Memory-based agent selection failed:`, error);
      return null;
    }
  }

  /**
   * Learn from task outcomes and store experiences
   */
  private async learnFromTaskOutcome(
    task: Task,
    result: any,
    outcome: "success" | "failure" | "partial"
  ): Promise<void> {
    if (!this.memoryManager) return;

    try {
      const tenantId = this.config.defaultTenantId || "default-tenant";

      // Create contextual memory from task experience
      const experience: ContextualMemory = {
        memoryId: `task_${task.id}_${Date.now()}`,
        relevanceScore:
          outcome === "success" ? 0.85 : outcome === "partial" ? 0.6 : 0.3,
        contextMatch: {
          similarityScore: 0.8,
          keywordMatches: task.description.toLowerCase().split(" "),
          semanticMatches: [task.type, outcome],
          temporalAlignment: 0.9,
        },
        content: {
          taskType: task.type,
          outcome,
          agentId: task.agentId,
          taskId: task.id,
          duration: task.updatedAt.getTime() - task.createdAt.getTime(),
          result:
            typeof result === "object"
              ? JSON.stringify(result)
              : String(result),
          lessons: this.extractLessonsFromOutcome(task, result, outcome),
        },
      };

      const storeResult = await this.memoryManager.storeExperience(
        tenantId,
        experience,
        {
          offloadContext: true,
          sharingLevel: outcome === "success" ? "shared" : "private",
        }
      );

      if (storeResult.success) {
        this.logger.info(
          `Learned from task ${task.id}: stored experience ${storeResult.data}`
        );
      } else {
        this.logger.warn(
          `Failed to store experience for task ${task.id}:`,
          storeResult.error
        );
      }
    } catch (error) {
      this.logger.error(`Learning from task ${task.id} failed:`, error);
    }
  }

  /**
   * Extract lessons from task outcomes
   */
  private extractLessonsFromOutcome(
    task: Task,
    result: any,
    outcome: "success" | "failure" | "partial"
  ): string[] {
    const lessons: string[] = [];

    if (outcome === "success") {
      lessons.push(
        `${task.type} tasks can be completed efficiently by ${task.agentId}`
      );
      if (task.priority === "high") {
        lessons.push("High priority tasks benefit from experienced agents");
      }
    } else if (outcome === "failure") {
      lessons.push(`${task.agentId} may not be optimal for ${task.type} tasks`);
      lessons.push("Consider agent reassignment for failed task types");
    }

    // Extract additional insights based on result structure
    if (typeof result === "object" && result) {
      if (result.performance) {
        lessons.push(
          `Task performance metrics: ${JSON.stringify(result.performance)}`
        );
      }
      if (result.errors && result.errors.length > 0) {
        lessons.push(`Common failure patterns: ${result.errors.join(", ")}`);
      }
    }

    return lessons;
  }

  /**
   * Create a task context for memory operations
   */
  private createTaskContext(task: Task): TaskContext {
    return {
      taskId: task.id,
      agentId: task.agentId,
      type: task.type,
      description: task.description,
      requirements: task.requirements || [],
      constraints: {
        priority: task.priority,
        maxRetries: task.maxRetries,
        timeout: task.timeout,
      },
      metadata: task.metadata || {},
    };
  }

  /**
   * Get enhanced system metrics including memory statistics
   */
  async getSystemMetrics(): Promise<SystemMetrics & { memoryStats?: any }> {
    const totalAgents = this.agents.size;
    const activeAgents = Array.from(this.agents.values()).filter(
      (a) => a.status === "active"
    ).length;
    const totalTasks = this.tasks.size;
    const completedTasks = Array.from(this.tasks.values()).filter(
      (t) => t.status === "completed"
    ).length;
    const failedTasks = Array.from(this.tasks.values()).filter(
      (t) => t.status === "failed"
    ).length;

    // Calculate average task duration from completed tasks
    const completedTaskDurations = Array.from(this.tasks.values())
      .filter((t) => t.status === "completed")
      .map((t) => t.updatedAt.getTime() - t.createdAt.getTime());

    const averageTaskDuration =
      completedTaskDurations.length > 0
        ? completedTaskDurations.reduce((sum, duration) => sum + duration, 0) /
          completedTaskDurations.length
        : 0;

    const baseMetrics: SystemMetrics = {
      totalAgents,
      activeAgents,
      totalTasks,
      completedTasks,
      failedTasks,
      averageTaskDuration,
      systemUptime: process.uptime(),
    };

    // Add memory system statistics if available
    if (this.memoryManager) {
      try {
        const memoryHealth = await this.memoryManager.getSystemHealth();
        return {
          ...baseMetrics,
          memoryStats: {
            tenants: memoryHealth.tenants,
            activeOperations: memoryHealth.activeOperations,
            cacheSize: memoryHealth.cacheSize,
            offloadedContexts: memoryHealth.offloadedContexts,
            federatedParticipants: memoryHealth.federatedParticipants,
            memoryEnabled: true,
          },
        };
      } catch (error) {
        this.logger.warn("Failed to get memory system health:", error);
      }
    }

    return baseMetrics;
  }

  /**
   * Get agent by ID
   */
  getAgent(agentId: string): Agent | undefined {
    return this.agents.get(agentId);
  }

  /**
   * Get task by ID
   */
  getTask(taskId: string): Task | undefined {
    return this.tasks.get(taskId);
  }

  /**
   * Generate a unique ID
   */
  private generateId(): string {
    return `id_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * Cleanup resources and stop background processes
   */
  cleanup(): void {
    if (this.taskRouter) {
      this.taskRouter.stopQueueProcessor();
    }
    this.logger.info("Agent Orchestrator cleanup completed");
  }
}
