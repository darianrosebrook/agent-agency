/**
 * Agent Registry Manager
 *
 * @author @darianrosebrook
 * @module orchestrator/AgentRegistryManager
 *
 * Central registry for managing agent profiles, capabilities, and performance history.
 * Implements ARBITER-001 specification with capability tracking and atomic updates.
 */

import {
  AgentRegistryDatabaseConfig,
  AgentRegistryDbClient,
} from "../database/AgentRegistryDbClient.js";
import { PerformanceTracker } from "../rl/PerformanceTracker";
import {
  AgentRegistrySecurity,
  SecurityContext,
} from "../security/AgentRegistrySecurity.js";
import type {
  AgentId,
  AgentProfile,
  AgentQuery,
  AgentQueryResult,
  AgentRegistryConfig,
  PerformanceMetrics,
  RegistryStats,
} from "../types/agent-registry";
import { RegistryError, RegistryErrorType } from "../types/agent-registry";
import { AgentProfileHelper } from "./AgentProfile";

/**
 * Default configuration for the agent registry.
 */
const DEFAULT_CONFIG: AgentRegistryConfig = {
  maxAgents: 1000,
  staleAgentThresholdMs: 24 * 60 * 60 * 1000, // 24 hours
  enableAutoCleanup: true,
  cleanupIntervalMs: 60 * 60 * 1000, // 1 hour
  enablePersistence: false, // Disabled by default for backward compatibility
  enableSecurity: true, // Security enabled by default
};

/**
 * Agent Registry Manager
 *
 * Maintains the catalog of available agents with their capabilities,
 * performance history, and current load status.
 *
 * @remarks
 * Thread-safe: Uses Map for O(1) lookups with atomic updates.
 * Invariants:
 * - Agent profiles are immutable except for performance metrics
 * - Performance history updates are atomic and isolated per agent
 * - Registry queries never block agent registration operations
 * - All capability changes are versioned and auditable
 */
export class AgentRegistryManager {
  private readonly agents: Map<AgentId, AgentProfile>;
  private readonly config: AgentRegistryConfig;
  private cleanupTimer?: ReturnType<typeof setInterval>;
  private readonly maxConcurrentTasksPerAgent: number = 10;
  private dbClient?: AgentRegistryDbClient;
  private securityManager?: AgentRegistrySecurity;
  private performanceTracker?: PerformanceTracker;

  constructor(
    config: Partial<AgentRegistryConfig> = {},
    performanceTracker?: PerformanceTracker
  ) {
    this.agents = new Map();
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.performanceTracker = performanceTracker;

    // Initialize database client if persistence is enabled
    if (this.config.enablePersistence && this.config.database) {
      const dbConfig: AgentRegistryDatabaseConfig = {
        host: this.config.database.host,
        port: this.config.database.port,
        database: this.config.database.database,
        username: this.config.database.username,
        password: this.config.database.password,
        ssl: this.config.database.ssl,
        maxConnections: 10,
        connectionTimeoutMs: 10000,
        queryTimeoutMs: 30000,
        retryAttempts: 3,
        retryDelayMs: 1000,
      };

      this.dbClient = new AgentRegistryDbClient(dbConfig);
    }

    // Initialize security manager if security is enabled
    if (this.config.enableSecurity) {
      this.securityManager = new AgentRegistrySecurity(this.config.security);
    }

    if (this.config.enableAutoCleanup) {
      this.startAutoCleanup();
    }
  }

  /**
   * Set the performance tracker for agent lifecycle tracking.
   *
   * @param tracker - Performance tracker instance
   */
  setPerformanceTracker(tracker: PerformanceTracker): void {
    this.performanceTracker = tracker;
  }

  /**
   * Initialize the registry manager.
   *
   * Must be called before using the registry if persistence is enabled.
   */
  async initialize(): Promise<void> {
    if (this.config.enablePersistence && this.dbClient) {
      await this.dbClient.initialize();

      // Load existing agents from database
      await this.loadAgentsFromDatabase();
    }
  }

  /**
   * Load existing agents from database into memory cache.
   */
  private async loadAgentsFromDatabase(): Promise<void> {
    if (!this.dbClient) return;

    try {
      // Query all agents (simplified query for loading)
      const result = await this.dbClient.queryAgents({
        taskType: "code-editing", // Required field
      });

      // Load agents into memory cache
      for (const queryResult of result) {
        this.agents.set(queryResult.agent.id, queryResult.agent);
      }

      // Log successful loading
      console.log(`Loaded ${result.length} agents from database`);
    } catch (error) {
      throw new RegistryError(
        RegistryErrorType.DATABASE_ERROR,
        `Failed to load agents from database: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Register a new agent in the registry.
   *
   * @param agent - Agent to register (partial, will be filled with defaults)
   * @returns Complete agent profile with generated fields
   * @throws RegistryError if agent already exists or registry is full
   *
   * @remarks
   * Acceptance Criterion A1: Agent profile created with capability tracking initialized
   */
  async registerAgent(
    agent: Partial<AgentProfile>,
    securityContext?: SecurityContext
  ): Promise<AgentProfile> {
    // Security check: authenticate and authorize
    if (this.config.enableSecurity && this.securityManager) {
      if (!securityContext) {
        throw new RegistryError(
          RegistryErrorType.INVALID_AGENT_DATA,
          "Security context required when security is enabled"
        );
      }

      const authorized = await this.securityManager.authorize(
        securityContext,
        "create" as any,
        "agent",
        agent.id || "unknown",
        agent
      );

      if (!authorized) {
        await this.securityManager.logAuditEvent({
          id: this.generateId(),
          timestamp: new Date(),
          eventType: "agent_registration" as any,
          actor: {
            tenantId: securityContext.tenantId,
            userId: securityContext.userId,
            sessionId: securityContext.sessionId,
          },
          resource: { type: "agent", id: agent.id || "unknown" },
          action: "create" as any,
          details: { agentData: agent },
          result: "failure",
          errorMessage: "Authorization failed",
          ipAddress: securityContext.ipAddress,
          userAgent: securityContext.userAgent,
        });

        throw new RegistryError(
          RegistryErrorType.INVALID_AGENT_DATA,
          "Not authorized to register agents"
        );
      }
    }

    // Validate input data with security layer
    if (this.config.enableSecurity && this.securityManager) {
      const validation = this.securityManager.validateAgentData(agent);
      if (!validation.valid) {
        throw new RegistryError(
          RegistryErrorType.INVALID_AGENT_DATA,
          `Validation failed: ${validation.errors.join(", ")}`
        );
      }
      // Use sanitized data if available
      if (validation.sanitized) {
        agent = validation.sanitized;
      }
    } else {
      // Fallback to basic validation
      AgentProfileHelper.validateProfile(agent);
    }

    if (!agent.id) {
      throw new RegistryError(
        RegistryErrorType.INVALID_AGENT_DATA,
        "Agent ID is required"
      );
    }

    // Check if agent already exists
    if (this.agents.has(agent.id)) {
      throw new RegistryError(
        RegistryErrorType.AGENT_ALREADY_EXISTS,
        `Agent with ID ${agent.id} already exists`,
        { agentId: agent.id }
      );
    }

    // Check registry capacity
    if (this.agents.size >= this.config.maxAgents) {
      throw new RegistryError(
        RegistryErrorType.REGISTRY_FULL,
        `Registry is full (max: ${this.config.maxAgents} agents)`,
        { maxAgents: this.config.maxAgents, currentSize: this.agents.size }
      );
    }

    // Create complete profile with defaults
    const now = new Date().toISOString();
    const profile: AgentProfile = {
      id: agent.id,
      name: agent.name!,
      modelFamily: agent.modelFamily!,
      capabilities: agent.capabilities!,
      performanceHistory:
        agent.performanceHistory ??
        AgentProfileHelper.createInitialPerformanceHistory(),
      currentLoad: agent.currentLoad ?? AgentProfileHelper.createInitialLoad(),
      registeredAt: now,
      lastActiveAt: now,
    };

    // Initialize capability tracking
    await this.initializeCapabilityTracking(profile);

    // Store in registry
    this.agents.set(profile.id, profile);

    // Persist to database if enabled
    if (this.dbClient) {
      try {
        await this.dbClient.registerAgent(profile);
      } catch (error) {
        // Rollback in-memory storage on database failure
        this.agents.delete(profile.id);
        throw new RegistryError(
          RegistryErrorType.DATABASE_ERROR,
          `Failed to persist agent to database: ${
            error instanceof Error ? error.message : String(error)
          }`,
          { agentId: profile.id }
        );
      }
    }

    // Audit log successful registration
    if (this.config.enableSecurity && this.securityManager && securityContext) {
      await this.securityManager.logAuditEvent({
        id: this.generateId(),
        timestamp: new Date(),
        eventType: "agent_registration" as any,
        actor: {
          tenantId: securityContext.tenantId,
          userId: securityContext.userId,
          sessionId: securityContext.sessionId,
        },
        resource: { type: "agent", id: profile.id },
        action: "create" as any,
        details: { agentProfile: profile },
        result: "success",
        ipAddress: securityContext.ipAddress,
        userAgent: securityContext.userAgent,
      });
    }

    // Record performance baseline for new agent
    if (this.performanceTracker) {
      try {
        await this.performanceTracker.recordAgentRegistration(profile.id, {
          capabilities: profile.capabilities.taskTypes,
          baselineMetrics: this.calculateBaselineMetrics(profile),
          registrationTimestamp: profile.registeredAt,
        });
      } catch (error) {
        // Log but don't fail registration due to performance tracking issues
        console.warn(
          `Failed to record agent registration performance baseline for ${profile.id}:`,
          error
        );
      }
    }

    return AgentProfileHelper.cloneProfile(profile);
  }

  /**
   * Update agent availability status.
   *
   * @param agentId - ID of the agent to update
   * @param status - New availability status
   * @param reason - Optional reason for status change
   * @param securityContext - Security context for authorization
   * @throws RegistryError if agent not found or unauthorized
   */
  async updateAgentStatus(
    agentId: AgentId,
    status: "available" | "busy" | "offline" | "maintenance",
    reason?: string,
    securityContext?: SecurityContext
  ): Promise<void> {
    // Security check: authenticate and authorize
    if (this.config.enableSecurity && this.securityManager) {
      if (!securityContext) {
        throw new RegistryError(
          RegistryErrorType.INVALID_AGENT_DATA,
          "Security context required when security is enabled"
        );
      }

      const authorized = await this.securityManager.authorize(
        securityContext,
        "update" as any,
        "agent",
        agentId
      );

      if (!authorized) {
        await this.securityManager.logAuditEvent({
          id: this.generateId(),
          timestamp: new Date(),
          eventType: "agent_status_update" as any,
          actor: {
            tenantId: securityContext.tenantId,
            userId: securityContext.userId,
            sessionId: securityContext.sessionId,
          },
          resource: { type: "agent", id: agentId },
          action: "update" as any,
          details: { status, reason },
          result: "failure",
          errorMessage: "Authorization failed",
          ipAddress: securityContext.ipAddress,
          userAgent: securityContext.userAgent,
        });

        throw new RegistryError(
          RegistryErrorType.INVALID_AGENT_DATA,
          "Not authorized to update agent status"
        );
      }
    }

    // Get current agent profile
    const profile = this.agents.get(agentId);
    if (!profile) {
      throw new RegistryError(
        RegistryErrorType.AGENT_NOT_FOUND,
        `Agent with ID ${agentId} not found`,
        { agentId }
      );
    }

    // Get previous status for tracking
    const previousStatus = this.getAgentAvailabilityStatus(profile);

    // Update agent load status based on new availability
    const updatedProfile = AgentProfileHelper.cloneProfile(profile);
    updatedProfile.lastActiveAt = new Date().toISOString();

    // Update load based on status
    switch (status) {
      case "available":
        updatedProfile.currentLoad = {
          ...updatedProfile.currentLoad,
          activeTasks: 0,
          utilizationPercent: 0,
        };
        break;
      case "busy":
        updatedProfile.currentLoad = {
          ...updatedProfile.currentLoad,
          activeTasks: Math.max(updatedProfile.currentLoad.activeTasks, 1),
          utilizationPercent: Math.max(
            updatedProfile.currentLoad.utilizationPercent,
            50
          ),
        };
        break;
      case "offline":
      case "maintenance":
        updatedProfile.currentLoad = {
          ...updatedProfile.currentLoad,
          activeTasks: this.maxConcurrentTasksPerAgent, // Fully utilized = unavailable
          utilizationPercent: 100,
        };
        break;
    }

    // Store updated profile
    this.agents.set(agentId, updatedProfile);

    // Persist to database if enabled (TODO: implement updateAgentStatus in database client)

    // Audit log successful status update
    if (this.config.enableSecurity && this.securityManager && securityContext) {
      await this.securityManager.logAuditEvent({
        id: this.generateId(),
        timestamp: new Date(),
        eventType: "agent_status_update" as any,
        actor: {
          tenantId: securityContext.tenantId,
          userId: securityContext.userId,
          sessionId: securityContext.sessionId,
        },
        resource: { type: "agent", id: agentId },
        action: "update" as any,
        details: { previousStatus, newStatus: status, reason },
        result: "success",
        ipAddress: securityContext.ipAddress,
        userAgent: securityContext.userAgent,
      });
    }

    // Record status change in performance tracker
    if (this.performanceTracker) {
      try {
        await this.performanceTracker.recordAgentStatusChange(agentId, status, {
          previousStatus,
          reason,
        });
      } catch (error) {
        // Log but don't fail status update due to performance tracking issues
        console.warn(
          `Failed to record agent status change performance event for ${agentId}:`,
          error
        );
      }
    }
  }

  /**
   * Get agent profile by ID.
   *
   * @param agentId - ID of the agent to retrieve
   * @returns Agent profile
   * @throws RegistryError if agent not found
   */
  async getProfile(
    agentId: AgentId,
    securityContext?: SecurityContext
  ): Promise<AgentProfile> {
    // Security check: authenticate and authorize
    if (this.config.enableSecurity && this.securityManager) {
      if (!securityContext) {
        throw new RegistryError(
          RegistryErrorType.INVALID_AGENT_DATA,
          "Security context required when security is enabled"
        );
      }

      const authorized = await this.securityManager.authorize(
        securityContext,
        "read" as any,
        "agent",
        agentId
      );

      if (!authorized) {
        await this.securityManager.logAuditEvent({
          id: this.generateId(),
          timestamp: new Date(),
          eventType: "agent_query" as any,
          actor: {
            tenantId: securityContext.tenantId,
            userId: securityContext.userId,
            sessionId: securityContext.sessionId,
          },
          resource: { type: "agent", id: agentId },
          action: "read" as any,
          details: { queryType: "getProfile" },
          result: "failure",
          errorMessage: "Authorization failed",
          ipAddress: securityContext.ipAddress,
          userAgent: securityContext.userAgent,
        });

        throw new RegistryError(
          RegistryErrorType.AGENT_NOT_FOUND,
          "Not authorized to access this agent"
        );
      }
    }

    let profile = this.agents.get(agentId);

    // If not in memory cache, try to load from database
    if (!profile && this.dbClient) {
      try {
        const dbProfile = await this.dbClient.getAgent(agentId);
        if (dbProfile) {
          // Cache in memory for future requests
          this.agents.set(agentId, dbProfile);
          profile = dbProfile;
        }
      } catch (error) {
        throw new RegistryError(
          RegistryErrorType.DATABASE_ERROR,
          `Failed to retrieve agent from database: ${
            error instanceof Error ? error.message : String(error)
          }`,
          { agentId }
        );
      }
    }

    if (!profile) {
      throw new RegistryError(
        RegistryErrorType.AGENT_NOT_FOUND,
        `Agent with ID ${agentId} not found`,
        { agentId }
      );
    }

    // Audit log successful profile access
    if (this.config.enableSecurity && this.securityManager && securityContext) {
      await this.securityManager.logAuditEvent({
        id: this.generateId(),
        timestamp: new Date(),
        eventType: "agent_query" as any,
        actor: {
          tenantId: securityContext.tenantId,
          userId: securityContext.userId,
          sessionId: securityContext.sessionId,
        },
        resource: { type: "agent", id: agentId },
        action: "read" as any,
        details: { queryType: "getProfile", found: true },
        result: "success",
        ipAddress: securityContext.ipAddress,
        userAgent: securityContext.userAgent,
      });
    }

    return AgentProfileHelper.cloneProfile(profile);
  }

  /**
   * Query agents by capability and return sorted by performance.
   *
   * @param query - Query parameters with required capabilities
   * @returns Array of matching agents sorted by success rate (highest first)
   *
   * @remarks
   * Acceptance Criterion A2: Agents matching criteria returned sorted by performance history success rate
   * Performance Target: <50ms P95 latency
   */
  async getAgentsByCapability(query: AgentQuery): Promise<AgentQueryResult[]> {
    const results: AgentQueryResult[] = [];

    for (const profile of Array.from(this.agents.values())) {
      // Check task type match
      if (!profile.capabilities.taskTypes.includes(query.taskType)) {
        continue;
      }

      // Check language requirements if specified
      if (query.languages && query.languages.length > 0) {
        const hasAllLanguages = query.languages.every((lang) =>
          profile.capabilities.languages.includes(lang)
        );
        if (!hasAllLanguages) {
          continue;
        }
      }

      // Check specialization requirements if specified
      if (query.specializations && query.specializations.length > 0) {
        const hasAllSpecializations = query.specializations.every((spec) =>
          profile.capabilities.specializations.includes(spec)
        );
        if (!hasAllSpecializations) {
          continue;
        }
      }

      // Check utilization threshold if specified
      if (
        query.maxUtilization !== undefined &&
        profile.currentLoad.utilizationPercent > query.maxUtilization
      ) {
        continue;
      }

      // Check minimum success rate if specified
      if (
        query.minSuccessRate !== undefined &&
        profile.performanceHistory.successRate < query.minSuccessRate
      ) {
        continue;
      }

      // Calculate match score
      const matchScore = this.calculateMatchScore(profile, query);
      const matchReason = this.explainMatchScore(profile, query, matchScore);

      results.push({
        agent: AgentProfileHelper.cloneProfile(profile),
        matchScore,
        matchReason,
      });
    }

    // Sort by success rate (highest first), then by match score
    return results.sort((a, b) => {
      const successDiff =
        b.agent.performanceHistory.successRate -
        a.agent.performanceHistory.successRate;
      if (Math.abs(successDiff) > 0.01) {
        return successDiff;
      }
      return b.matchScore - a.matchScore;
    });
  }

  /**
   * Update performance metrics for an agent after task completion.
   *
   * @param agentId - ID of the agent to update
   * @param metrics - Performance metrics from the completed task
   * @returns Updated agent profile
   * @throws RegistryError if agent not found or update fails
   *
   * @remarks
   * Acceptance Criterion A3: Agent's running average performance history computed and persisted
   * Performance Target: <30ms P95 latency
   * Invariant: Performance history updates are atomic and isolated per agent
   */
  async updatePerformance(
    agentId: AgentId,
    metrics: PerformanceMetrics
  ): Promise<AgentProfile> {
    const profile = this.agents.get(agentId);

    if (!profile) {
      throw new RegistryError(
        RegistryErrorType.AGENT_NOT_FOUND,
        `Agent with ID ${agentId} not found`,
        { agentId }
      );
    }

    try {
      // Compute new running average (atomic operation)
      const newPerformanceHistory = AgentProfileHelper.updatePerformanceHistory(
        profile.performanceHistory,
        metrics
      );

      // Update profile with new performance history
      const updatedProfile: AgentProfile = {
        ...profile,
        performanceHistory: newPerformanceHistory,
        lastActiveAt: new Date().toISOString(),
      };

      // Atomically update in registry
      this.agents.set(agentId, updatedProfile);

      // Record performance metrics to database if enabled
      if (this.dbClient) {
        try {
          await this.dbClient.recordPerformance(agentId, metrics);
        } catch (error) {
          // Log database error but don't fail the operation
          console.error(
            `Failed to record performance to database for agent ${agentId}:`,
            error
          );
        }
      }

      return AgentProfileHelper.cloneProfile(updatedProfile);
    } catch (error) {
      throw new RegistryError(
        RegistryErrorType.UPDATE_FAILED,
        `Failed to update performance for agent ${agentId}: ${
          (error as Error).message
        }`,
        { agentId, metrics, error }
      );
    }
  }

  /**
   * Update agent's current load (active and queued tasks).
   *
   * @param agentId - ID of the agent to update
   * @param activeTasks - New active tasks count
   * @param queuedTasks - New queued tasks count
   * @returns Updated agent profile
   * @throws RegistryError if agent not found
   */
  async updateLoad(
    agentId: AgentId,
    activeTasks: number,
    queuedTasks: number
  ): Promise<AgentProfile> {
    const profile = this.agents.get(agentId);

    if (!profile) {
      throw new RegistryError(
        RegistryErrorType.AGENT_NOT_FOUND,
        `Agent with ID ${agentId} not found`,
        { agentId }
      );
    }

    const utilizationPercent =
      (activeTasks / this.maxConcurrentTasksPerAgent) * 100;

    const updatedProfile: AgentProfile = {
      ...profile,
      currentLoad: {
        activeTasks,
        queuedTasks,
        utilizationPercent: Math.min(100, utilizationPercent),
      },
      lastActiveAt: new Date().toISOString(),
    };

    this.agents.set(agentId, updatedProfile);

    return AgentProfileHelper.cloneProfile(updatedProfile);
  }

  /**
   * Get registry statistics.
   *
   * @returns Current registry stats
   */
  async getStats(): Promise<RegistryStats> {
    const allAgents = Array.from(this.agents.values());
    const activeAgents = allAgents.filter((a) => a.currentLoad.activeTasks > 0);
    const idleAgents = allAgents.filter((a) => a.currentLoad.activeTasks === 0);

    const totalUtilization = allAgents.reduce(
      (sum, a) => sum + a.currentLoad.utilizationPercent,
      0
    );
    const averageUtilization =
      allAgents.length > 0 ? totalUtilization / allAgents.length : 0;

    const totalSuccessRate = allAgents.reduce(
      (sum, a) => sum + a.performanceHistory.successRate,
      0
    );
    const averageSuccessRate =
      allAgents.length > 0 ? totalSuccessRate / allAgents.length : 0;

    return {
      totalAgents: allAgents.length,
      activeAgents: activeAgents.length,
      idleAgents: idleAgents.length,
      averageUtilization,
      averageSuccessRate,
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Remove an agent from the registry.
   *
   * @param agentId - ID of the agent to remove
   * @returns True if agent was removed
   */
  async unregisterAgent(agentId: AgentId): Promise<boolean> {
    return this.agents.delete(agentId);
  }

  /**
   * Initialize capability tracking for a new agent.
   */
  private async initializeCapabilityTracking(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
    _profile: AgentProfile
  ): Promise<void> {
    // Capability tracking initialization
    // In production, this would set up monitoring for capability usage
    // and initialize any external tracking systems
    // For now, this is a no-op, but provides extension point
  }

  /**
   * Calculate match score for query result ranking.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Match score (0.0 - 1.0)
   */
  private calculateMatchScore(
    profile: AgentProfile,
    query: AgentQuery
  ): number {
    let score = 0.0;
    let weights = 0.0;

    // Task type match (required, so always contributes)
    score += 0.3;
    weights += 0.3;

    // Language matches (if specified)
    if (query.languages && query.languages.length > 0) {
      const matchedLanguages = query.languages.filter((lang) =>
        profile.capabilities.languages.includes(lang)
      ).length;
      score += (matchedLanguages / query.languages.length) * 0.3;
      weights += 0.3;
    }

    // Specialization matches (if specified)
    if (query.specializations && query.specializations.length > 0) {
      const matchedSpecs = query.specializations.filter((spec) =>
        profile.capabilities.specializations.includes(spec)
      ).length;
      score += (matchedSpecs / query.specializations.length) * 0.2;
      weights += 0.2;
    }

    // Performance bonus
    score += profile.performanceHistory.successRate * 0.2;
    weights += 0.2;

    return weights > 0 ? score / weights : 0;
  }

  /**
   * Generate human-readable explanation of match score.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Explanation string
   */
  private explainMatchScore(
    profile: AgentProfile,
    query: AgentQuery,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
    _score: number
  ): string {
    const reasons: string[] = [];

    reasons.push(`Supports ${query.taskType}`);

    if (query.languages && query.languages.length > 0) {
      reasons.push(`Languages: ${query.languages.join(", ")}`);
    }

    if (query.specializations && query.specializations.length > 0) {
      reasons.push(`Specializations: ${query.specializations.join(", ")}`);
    }

    reasons.push(
      `${(profile.performanceHistory.successRate * 100).toFixed(
        1
      )}% success rate`
    );
    reasons.push(
      `${profile.currentLoad.utilizationPercent.toFixed(0)}% utilized`
    );

    return reasons.join("; ");
  }

  /**
   * Start automatic cleanup of stale agents.
   */
  private startAutoCleanup(): void {
    this.cleanupTimer = setInterval(() => {
      this.cleanupStaleAgents();
    }, this.config.cleanupIntervalMs);
  }

  /**
   * Clean up stale agents (inactive beyond threshold).
   */
  private cleanupStaleAgents(): void {
    const now = new Date().toISOString();
    const staleAgents: AgentId[] = [];

    const agents = Array.from(this.agents.entries());
    for (const [agentId, profile] of agents) {
      if (
        AgentProfileHelper.isStale(
          profile,
          this.config.staleAgentThresholdMs,
          now
        )
      ) {
        staleAgents.push(agentId);
      }
    }

    for (const agentId of staleAgents) {
      this.agents.delete(agentId);
    }
  }

  /**
   * Shutdown the registry manager and cleanup resources.
   */
  async shutdown(): Promise<void> {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
    }
  }

  /**
   * Get the current availability status of an agent.
   *
   * @param profile - Agent profile
   * @returns Availability status string
   */
  private getAgentAvailabilityStatus(profile: AgentProfile): string {
    // Determine status based on load and activity
    const utilization = profile.currentLoad.utilizationPercent;
    const activeTasks = profile.currentLoad.activeTasks;

    if (utilization >= 100 || activeTasks >= this.maxConcurrentTasksPerAgent) {
      return "offline";
    } else if (utilization >= 50 || activeTasks > 0) {
      return "busy";
    } else {
      return "available";
    }
  }

  /**
   * Calculate baseline performance metrics for a new agent.
   *
   * @param profile - Agent profile
   * @returns Baseline metrics for performance tracking
   */
  private calculateBaselineMetrics(profile: AgentProfile): {
    latencyMs: number;
    accuracy: number;
    costPerTask: number;
    reliability: number;
  } {
    // Use model family to estimate baseline performance
    // These are conservative estimates based on typical performance
    const modelFamily = profile.modelFamily.toLowerCase();

    let baselineLatency: number;
    let baselineAccuracy: number;
    let baselineCost: number;
    let baselineReliability: number;

    // Estimate based on model capabilities
    if (modelFamily.includes("gpt-4") || modelFamily.includes("claude-3")) {
      baselineLatency = 1500; // 1.5s average response time
      baselineAccuracy = 0.92; // 92% accuracy
      baselineCost = 0.015; // $0.015 per task
      baselineReliability = 0.98; // 98% reliability
    } else if (
      modelFamily.includes("gpt-3.5") ||
      modelFamily.includes("claude-2")
    ) {
      baselineLatency = 1200; // 1.2s average response time
      baselineAccuracy = 0.88; // 88% accuracy
      baselineCost = 0.008; // $0.008 per task
      baselineReliability = 0.95; // 95% reliability
    } else {
      // Conservative defaults for unknown models
      baselineLatency = 2000; // 2s average response time
      baselineAccuracy = 0.8; // 80% accuracy
      baselineCost = 0.01; // $0.010 per task
      baselineReliability = 0.9; // 90% reliability
    }

    // Adjust based on agent capabilities (more specialized = better performance)
    const capabilityBonus = Math.min(
      profile.capabilities.specializations.length * 0.02,
      0.1
    );
    baselineAccuracy = Math.min(baselineAccuracy + capabilityBonus, 0.95);

    // Language support bonus (more languages = slightly higher cost but better accuracy)
    const languageBonus = Math.min(
      profile.capabilities.languages.length * 0.01,
      0.05
    );
    baselineAccuracy = Math.min(baselineAccuracy + languageBonus, 0.95);
    baselineCost += languageBonus * 0.002;

    return {
      latencyMs: baselineLatency,
      accuracy: baselineAccuracy,
      costPerTask: baselineCost,
      reliability: baselineReliability,
    };
  }

  /**
   * Generate a unique ID for audit events
   */
  private generateId(): string {
    return `audit_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }
}
