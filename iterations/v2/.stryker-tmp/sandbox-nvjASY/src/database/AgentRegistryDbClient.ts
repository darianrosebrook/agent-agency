/**
 * Agent Registry Database Client
 *
 * PostgreSQL client for the Agent Registry Manager (ARBITER-001).
 * Provides ACID-compliant persistence for agent profiles, capabilities, and performance history.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { Pool, PoolClient } from "pg";
import {
  AgentCapabilities,
  AgentProfile,
  AgentQuery,
  AgentQueryResult,
  DatabaseConfig,
  PerformanceHistory,
  PerformanceMetrics,
  ProgrammingLanguage,
  Specialization,
  TaskType,
} from "../types/agent-registry.js";
import { Logger } from "../utils/Logger.js";

export interface AgentRegistryDatabaseConfig extends DatabaseConfig {
  maxConnections: number;
  connectionTimeoutMs: number;
  queryTimeoutMs: number;
  retryAttempts: number;
  retryDelayMs: number;
}

export class AgentRegistryDbClient {
  private pool: Pool;
  private logger: Logger;
  private config: AgentRegistryDatabaseConfig;

  constructor(
    config:
      | AgentRegistryDatabaseConfig
      | {
          host: string;
          port: number;
          database: string;
          user: string;
          password: string;
        }
  ) {
    // Handle legacy constructor for backward compatibility
    if ("host" in config && "user" in config && !("maxConnections" in config)) {
      this.config = {
        host: config.host,
        port: config.port,
        database: config.database,
        username: config.user,
        password: config.password,
        maxConnections: 10,
        connectionTimeoutMs: 10000,
        queryTimeoutMs: 30000,
        retryAttempts: 3,
        retryDelayMs: 1000,
      };
    } else {
      this.config = config as AgentRegistryDatabaseConfig;
    }

    this.logger = new Logger("AgentRegistryDbClient");

    this.pool = new Pool({
      host: this.config.host,
      port: this.config.port,
      database: this.config.database,
      user: this.config.username,
      password: this.config.password,
      max: this.config.maxConnections,
      connectionTimeoutMillis: this.config.connectionTimeoutMs,
      query_timeout: this.config.queryTimeoutMs,
      ssl: false, // Disable SSL for tests
    });

    this.setupPoolErrorHandling();
  }

  /**
   * Initialize database connection and verify schema
   */
  async initialize(): Promise<void> {
    try {
      this.logger.info("Initializing Agent Registry database client...");

      // Test connection
      const client = await this.pool.connect();
      try {
        await client.query("SELECT 1");
        this.logger.info("Database connection established");
      } finally {
        client.release();
      }

      // Verify schema exists
      await this.verifySchema();
      this.logger.info("Database schema verified");

      this.logger.info(
        "Agent Registry database client initialized successfully"
      );
    } catch (error) {
      this.logger.error("Failed to initialize database client:", error);
      throw new Error(
        `Database initialization failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Clean shutdown of database connections
   */
  async shutdown(): Promise<void> {
    try {
      this.logger.info("Shutting down Agent Registry database client...");
      await this.pool.end();
      this.logger.info("Database connections closed");
    } catch (error) {
      this.logger.error("Error during database shutdown:", error);
    }
  }

  /**
   * Register a new agent profile
   */
  async registerAgent(
    profile: Omit<
      AgentProfile,
      "id" | "registeredAt" | "lastActiveAt" | "createdAt" | "updatedAt"
    >
  ): Promise<string> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");

      // Insert agent profile
      const profileResult = await client.query(
        `
        INSERT INTO agent_profiles (
          name, model_family, active_tasks, queued_tasks, utilization_percent
        ) VALUES ($1, $2, $3, $4, $5)
        RETURNING id
      `,
        [
          profile.name,
          profile.modelFamily,
          profile.currentLoad?.activeTasks || 0,
          profile.currentLoad?.queuedTasks || 0,
          profile.currentLoad?.utilizationPercent || 0,
        ]
      );

      const agentId = profileResult.rows[0].id;

      // Insert capabilities if provided
      if (profile.capabilities) {
        // Insert task types
        for (const taskType of profile.capabilities.taskTypes) {
          await client.query(
            `
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `,
            [
              agentId,
              `task_${taskType}`,
              1.0, // Default score for task types
              JSON.stringify({ type: "task", value: taskType }),
            ]
          );
        }

        // Insert languages
        for (const language of profile.capabilities.languages) {
          await client.query(
            `
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `,
            [
              agentId,
              `lang_${language}`,
              1.0, // Default score for languages
              JSON.stringify({ type: "language", value: language }),
            ]
          );
        }

        // Insert specializations
        for (const specialization of profile.capabilities.specializations) {
          await client.query(
            `
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `,
            [
              agentId,
              `spec_${specialization}`,
              1.0, // Default score for specializations
              JSON.stringify({ type: "specialization", value: specialization }),
            ]
          );
        }
      }

      // Insert performance history if provided
      if (profile.performanceHistory) {
        await client.query(
          `
          INSERT INTO agent_performance_history (
            agent_id, task_type, success_rate, average_latency,
            total_tasks, quality_score, confidence_score, metadata
          ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        `,
          [
            agentId,
            "general", // Default task type for overall performance
            profile.performanceHistory.successRate,
            profile.performanceHistory.averageLatency,
            profile.performanceHistory.taskCount,
            profile.performanceHistory.averageQuality,
            1.0, // Default confidence score
            JSON.stringify({}),
          ]
        );
      }

      await client.query("COMMIT");

      this.logger.info(`Agent registered with ID: ${agentId}`);
      return agentId;
    } catch (error) {
      await client.query("ROLLBACK");
      this.logger.error("Failed to register agent:", error);
      throw new Error(
        `Agent registration failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get agent profile by ID
   */
  async getAgent(agentId: string): Promise<AgentProfile | null> {
    const client = await this.pool.connect();

    try {
      // Get profile
      const profileResult = await client.query(
        `
        SELECT * FROM agent_profiles WHERE id = $1
      `,
        [agentId]
      );

      if (profileResult.rows.length === 0) {
        return null;
      }

      const profileRow = profileResult.rows[0];

      // Get capabilities
      const capabilitiesResult = await client.query(
        `
        SELECT capability_name, score, metadata FROM agent_capabilities
        WHERE agent_id = $1 ORDER BY capability_name
      `,
        [agentId]
      );

      // Reconstruct capabilities from database records
      const taskTypes: TaskType[] = [];
      const languages: ProgrammingLanguage[] = [];
      const specializations: Specialization[] = [];

      capabilitiesResult.rows.forEach((row) => {
        const metadata = row.metadata || {};
        if (metadata.type === "task") {
          taskTypes.push(metadata.value);
        } else if (metadata.type === "language") {
          languages.push(metadata.value);
        } else if (metadata.type === "specialization") {
          specializations.push(metadata.value);
        }
      });

      const capabilities: AgentCapabilities = {
        taskTypes,
        languages,
        specializations,
      };

      // Get performance history (take the most recent record)
      const performanceResult = await client.query(
        `
        SELECT * FROM agent_performance_history
        WHERE agent_id = $1 ORDER BY recorded_at DESC LIMIT 1
      `,
        [agentId]
      );

      const performanceHistory: PerformanceHistory =
        performanceResult.rows.length > 0
          ? {
              successRate: performanceResult.rows[0].success_rate,
              averageQuality: performanceResult.rows[0].quality_score,
              averageLatency: performanceResult.rows[0].average_latency,
              taskCount: performanceResult.rows[0].total_tasks,
            }
          : {
              successRate: 0,
              averageQuality: 0,
              averageLatency: 0,
              taskCount: 0,
            };

      return {
        id: profileRow.id,
        name: profileRow.name,
        modelFamily: profileRow.model_family,
        capabilities,
        performanceHistory,
        currentLoad: {
          activeTasks: profileRow.active_tasks || 0,
          queuedTasks: profileRow.queued_tasks || 0,
          utilizationPercent: profileRow.utilization_percent || 0,
        },
        registeredAt: profileRow.registered_at,
        lastActiveAt: profileRow.last_active_at,
      };
    } catch (error) {
      this.logger.error("Failed to get agent:", error);
      throw new Error(
        `Failed to retrieve agent: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Update agent profile
   */
  async updateAgent(
    agentId: string,
    updates: Partial<AgentProfile>
  ): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");

      // Update profile
      const updateFields: string[] = [];
      const values: any[] = [];
      let paramIndex = 1;

      if (updates.name !== undefined) {
        updateFields.push(`name = $${paramIndex++}`);
        values.push(updates.name);
      }
      if (updates.lastActiveAt !== undefined) {
        updateFields.push(`last_active_at = $${paramIndex++}`);
        values.push(updates.lastActiveAt);
      }
      if (updates.currentLoad?.activeTasks !== undefined) {
        updateFields.push(`active_tasks = $${paramIndex++}`);
        values.push(updates.currentLoad.activeTasks);
      }
      if (updates.currentLoad?.queuedTasks !== undefined) {
        updateFields.push(`queued_tasks = $${paramIndex++}`);
        values.push(updates.currentLoad.queuedTasks);
      }
      if (updates.currentLoad?.utilizationPercent !== undefined) {
        updateFields.push(`utilization_percent = $${paramIndex++}`);
        values.push(updates.currentLoad.utilizationPercent);
      }

      if (updateFields.length > 0) {
        updateFields.push(`updated_at = NOW()`);
        values.push(agentId);

        await client.query(
          `
          UPDATE agent_profiles
          SET ${updateFields.join(", ")}
          WHERE id = $${paramIndex}
        `,
          values
        );
      }

      // Update capabilities if provided
      if (updates.capabilities) {
        // Delete existing capabilities
        await client.query(
          "DELETE FROM agent_capabilities WHERE agent_id = $1",
          [agentId]
        );

        // Insert new capabilities - task types
        for (const taskType of updates.capabilities.taskTypes) {
          await client.query(
            `
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `,
            [
              agentId,
              `task_${taskType}`,
              1.0,
              JSON.stringify({ type: "task", value: taskType }),
            ]
          );
        }

        // Insert new capabilities - languages
        for (const language of updates.capabilities.languages) {
          await client.query(
            `
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `,
            [
              agentId,
              `lang_${language}`,
              1.0,
              JSON.stringify({ type: "language", value: language }),
            ]
          );
        }

        // Insert new capabilities - specializations
        for (const specialization of updates.capabilities.specializations) {
          await client.query(
            `
            INSERT INTO agent_capabilities (agent_id, capability_name, score, metadata)
            VALUES ($1, $2, $3, $4)
          `,
            [
              agentId,
              `spec_${specialization}`,
              1.0,
              JSON.stringify({ type: "specialization", value: specialization }),
            ]
          );
        }
      }

      await client.query("COMMIT");

      this.logger.info(`Agent updated: ${agentId}`);
    } catch (error) {
      await client.query("ROLLBACK");
      this.logger.error("Failed to update agent:", error);
      throw new Error(
        `Agent update failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Delete agent profile
   */
  async deleteAgent(agentId: string): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");

      // Delete in reverse dependency order
      await client.query(
        "DELETE FROM agent_performance_history WHERE agent_id = $1",
        [agentId]
      );
      await client.query("DELETE FROM agent_capabilities WHERE agent_id = $1", [
        agentId,
      ]);
      await client.query("DELETE FROM agent_profiles WHERE id = $1", [agentId]);

      await client.query("COMMIT");

      this.logger.info(`Agent deleted: ${agentId}`);
    } catch (error) {
      await client.query("ROLLBACK");
      this.logger.error("Failed to delete agent:", error);
      throw new Error(
        `Agent deletion failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Query agents with advanced filtering
   */
  async queryAgents(query: AgentQuery): Promise<AgentQueryResult[]> {
    const client = await this.pool.connect();

    try {
      const whereConditions: string[] = [];
      const values: any[] = [];
      let paramIndex = 1;

      // Build WHERE conditions based on available AgentQuery fields
      if (query.maxUtilization !== undefined) {
        whereConditions.push(`p.utilization_percent <= $${paramIndex++}`);
        values.push(query.maxUtilization);
      }

      if (query.minSuccessRate !== undefined) {
        whereConditions.push(`
          EXISTS (
            SELECT 1 FROM agent_performance_history ph
            WHERE ph.agent_id = p.id AND ph.success_rate >= $${paramIndex}
          )
        `);
        values.push(query.minSuccessRate);
      }

      // Add capability filtering for languages
      if (query.languages && query.languages.length > 0) {
        const placeholders = query.languages
          .map(() => `$${paramIndex++}`)
          .join(", ");
        whereConditions.push(`
          EXISTS (
            SELECT 1 FROM agent_capabilities c
            WHERE c.agent_id = p.id AND c.metadata->>'type' = 'language'
            AND c.metadata->>'value' = ANY(ARRAY[${placeholders}])
          )
        `);
        values.push(...query.languages);
      }

      // Add capability filtering for specializations
      if (query.specializations && query.specializations.length > 0) {
        const placeholders = query.specializations
          .map(() => `$${paramIndex++}`)
          .join(", ");
        whereConditions.push(`
          EXISTS (
            SELECT 1 FROM agent_capabilities c
            WHERE c.agent_id = p.id AND c.metadata->>'type' = 'specialization'
            AND c.metadata->>'value' = ANY(ARRAY[${placeholders}])
          )
        `);
        values.push(...query.specializations);
      }

      const whereClause =
        whereConditions.length > 0
          ? `WHERE ${whereConditions.join(" AND ")}`
          : "";

      const result = await client.query(
        `
        SELECT p.* FROM agent_profiles p
        ${whereClause}
        ORDER BY p.last_active_at DESC
        LIMIT 50
      `,
        values
      );

      // Convert results to AgentQueryResult format
      const queryResults: AgentQueryResult[] = [];

      for (const row of result.rows) {
        // Reconstruct the full agent profile
        const agent = await this.getAgent(row.id);
        if (agent) {
          queryResults.push({
            agent,
            matchScore: 0.8, // Placeholder - would calculate based on query criteria
            matchReason: "Matches query criteria",
          });
        }
      }

      this.logger.debug(`Found ${queryResults.length} agents matching query`);
      return queryResults;
    } catch (error) {
      this.logger.error("Failed to query agents:", error);
      throw new Error(
        `Agent query failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Record performance metrics for an agent
   */
  async recordPerformance(
    agentId: string,
    performance: PerformanceMetrics
  ): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query(
        `
        INSERT INTO agent_performance_history (
          agent_id, task_type, success_rate, average_latency,
          total_tasks, quality_score, confidence_score, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      `,
        [
          agentId,
          performance.taskType || "general",
          performance.success ? 1.0 : 0.0,
          performance.latencyMs,
          1, // Single task
          performance.qualityScore,
          1.0, // Default confidence
          JSON.stringify({ tokensUsed: performance.tokensUsed }),
        ]
      );

      this.logger.debug(`Performance recorded for agent: ${agentId}`);
    } catch (error) {
      this.logger.error("Failed to record performance:", error);
      throw new Error(
        `Performance recording failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get performance statistics for an agent
   */
  async getAgentStats(agentId: string): Promise<{
    totalTasks: number;
    averageSuccessRate: number;
    averageLatency: number;
    averageQuality: number;
    taskTypeBreakdown: Record<
      string,
      { count: number; successRate: number; avgLatency: number }
    >;
  }> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(
        `
        SELECT
          task_type,
          COUNT(*) as task_count,
          AVG(success_rate) as avg_success_rate,
          AVG(average_latency) as avg_latency,
          AVG(quality_score) as avg_quality
        FROM agent_performance_history
        WHERE agent_id = $1
        GROUP BY task_type
      `,
        [agentId]
      );

      let totalTasks = 0;
      let totalSuccessRate = 0;
      let totalLatency = 0;
      let totalQuality = 0;
      const taskTypeBreakdown: Record<
        string,
        { count: number; successRate: number; avgLatency: number }
      > = {};

      for (const row of result.rows) {
        const count = parseInt(row.task_count);
        totalTasks += count;
        totalSuccessRate += row.avg_success_rate * count;
        totalLatency += row.avg_latency * count;
        totalQuality += (row.avg_quality || 0) * count;

        taskTypeBreakdown[row.task_type] = {
          count,
          successRate: row.avg_success_rate,
          avgLatency: row.avg_latency,
        };
      }

      return {
        totalTasks,
        averageSuccessRate: totalTasks > 0 ? totalSuccessRate / totalTasks : 0,
        averageLatency: totalTasks > 0 ? totalLatency / totalTasks : 0,
        averageQuality: totalTasks > 0 ? totalQuality / totalTasks : 0,
        taskTypeBreakdown,
      };
    } catch (error) {
      this.logger.error("Failed to get agent stats:", error);
      throw new Error(
        `Agent stats retrieval failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Health check for database connectivity
   */
  async healthCheck(): Promise<{
    healthy: boolean;
    latency: number;
    error?: string;
  }> {
    const startTime = Date.now();

    try {
      const client = await this.pool.connect();
      try {
        await client.query("SELECT 1");
        const latency = Date.now() - startTime;
        return { healthy: true, latency };
      } finally {
        client.release();
      }
    } catch (error) {
      const latency = Date.now() - startTime;
      return {
        healthy: false,
        latency,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Verify database schema exists and is correct
   */
  private async verifySchema(): Promise<void> {
    const client = await this.pool.connect();

    try {
      // Check if required tables exist
      const tables = [
        "agent_profiles",
        "agent_capabilities",
        "agent_performance_history",
      ];
      for (const table of tables) {
        const result = await client.query(
          `
          SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = $1
          )
        `,
          [table]
        );

        if (!result.rows[0].exists) {
          throw new Error(`Required table '${table}' does not exist`);
        }
      }

      this.logger.debug("All required tables verified");
    } catch (error) {
      throw new Error(
        `Schema verification failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Setup pool error handling
   */
  private setupPoolErrorHandling(): void {
    this.pool.on("error", (err) => {
      this.logger.error("Unexpected database pool error:", err);
    });

    this.pool.on("connect", () => {
      this.logger.debug("New database connection established");
    });

    this.pool.on("remove", () => {
      this.logger.debug("Database connection removed from pool");
    });
  }

  /**
   * Update agent performance metrics (legacy method for compatibility)
   */
  async updatePerformance(
    agentId: string,
    metrics: PerformanceMetrics
  ): Promise<void> {
    await this.recordPerformance(agentId, metrics);
  }

  /**
   * Update agent load (active and queued tasks)
   */
  async updateLoad(
    agentId: string,
    activeTasksDelta: number,
    queuedTasksDelta: number
  ): Promise<void> {
    const client = await this.pool.connect();

    try {
      // Update with atomic increment/decrement
      await client.query(
        `
        UPDATE agent_profiles
        SET
          active_tasks = GREATEST(0, active_tasks + $2),
          queued_tasks = GREATEST(0, queued_tasks + $3),
          utilization_percent = LEAST(100, GREATEST(0,
            CASE
              WHEN active_tasks + queued_tasks + $2 + $3 = 0 THEN 0
              ELSE ((active_tasks + $2) * 100.0) / (active_tasks + queued_tasks + $2 + $3)
            END
          )),
          updated_at = NOW()
        WHERE id = $1
      `,
        [agentId, activeTasksDelta, queuedTasksDelta]
      );

      this.logger.debug(
        `Updated load for agent: ${agentId} (+${activeTasksDelta} active, +${queuedTasksDelta} queued)`
      );
    } catch (error) {
      this.logger.error("Failed to update agent load:", error);
      throw new Error(
        `Load update failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Unregister an agent (delete from database)
   */
  async unregisterAgent(agentId: string): Promise<boolean> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");

      // Delete in reverse dependency order
      await client.query(
        "DELETE FROM agent_performance_history WHERE agent_id = $1",
        [agentId]
      );
      await client.query("DELETE FROM agent_capabilities WHERE agent_id = $1", [
        agentId,
      ]);
      const result = await client.query(
        "DELETE FROM agent_profiles WHERE id = $1",
        [agentId]
      );

      await client.query("COMMIT");

      const deleted = (result.rowCount ?? 0) > 0;
      if (deleted) {
        this.logger.info(`Agent unregistered: ${agentId}`);
      }

      return deleted;
    } catch (error) {
      await client.query("ROLLBACK");
      this.logger.error("Failed to unregister agent:", error);
      throw new Error(
        `Agent unregistration failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get registry statistics
   */
  async getStats(): Promise<{
    totalAgents: number;
    activeAgents: number;
    totalCapabilities: number;
    averagePerformance: number;
  }> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(`
        SELECT
          (SELECT COUNT(*) FROM agent_profiles) as total_agents,
          (SELECT COUNT(*) FROM agent_profiles WHERE last_active_at > NOW() - INTERVAL '1 hour') as active_agents,
          (SELECT COUNT(*) FROM agent_capabilities) as total_capabilities,
          (SELECT COALESCE(AVG(success_rate), 0) FROM agent_performance_history) as avg_performance
      `);

      const stats = result.rows[0];

      return {
        totalAgents: parseInt(stats.total_agents),
        activeAgents: parseInt(stats.active_agents),
        totalCapabilities: parseInt(stats.total_capabilities),
        averagePerformance: parseFloat(stats.avg_performance) || 0,
      };
    } catch (error) {
      this.logger.error("Failed to get registry stats:", error);
      throw new Error(
        `Stats retrieval failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Execute query with retry logic
   */
  private async executeWithRetry<T>(
    operation: (client: PoolClient) => Promise<T>, // eslint-disable-line no-unused-vars
    operationName: string
  ): Promise<T> {
    let lastError: Error = new Error("Unknown error");

    for (let attempt = 1; attempt <= this.config.retryAttempts; attempt++) {
      const client = await this.pool.connect();

      try {
        const result = await operation(client);
        return result;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        this.logger.warn(
          `${operationName} failed on attempt ${attempt}:`,
          lastError.message
        );

        if (attempt < this.config.retryAttempts) {
          await new Promise((resolve) =>
            setTimeout(resolve, this.config.retryDelayMs)
          );
        }
      } finally {
        client.release();
      }
    }

    throw new Error(
      `${operationName} failed after ${this.config.retryAttempts} attempts: ${lastError.message}`
    );
  }
}
