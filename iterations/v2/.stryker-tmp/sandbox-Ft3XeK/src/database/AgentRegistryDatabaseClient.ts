/**
 * @fileoverview PostgreSQL Database Client for Agent Registry (ARBITER-001)
 *
 * Provides persistent storage for agent profiles, capabilities, and performance history.
 * Implements ACID transactions and connection pooling for production reliability.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { Pool, PoolClient } from "pg";
import {
  AgentId,
  AgentProfile,
  AgentQuery,
  PerformanceMetrics,
  RegistryStats,
} from "../types/agent-registry";

/**
 * Database Configuration
 */
export interface DatabaseConfig {
  /** PostgreSQL connection string or config */
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;

  /** Connection pool settings */
  poolMin: number;
  poolMax: number;
  poolIdleTimeoutMs: number;
  poolConnectionTimeoutMs: number;

  /** Query timeouts */
  queryTimeoutMs: number;

  /** Enable query logging */
  enableQueryLogging: boolean;

  /** Enable connection retries */
  enableRetries: boolean;
  maxRetries: number;
  retryDelayMs: number;
}

/**
 * Database Client for Agent Registry
 *
 * Provides ACID-compliant persistent storage for agent registry data.
 */
export class AgentRegistryDatabaseClient {
  private pool: Pool;
  private config: DatabaseConfig;

  constructor(config: Partial<DatabaseConfig> = {}) {
    this.config = {
      host: process.env.DB_HOST || "localhost",
      port: parseInt(process.env.DB_PORT || "5432"),
      database: process.env.DB_NAME || "agent_agency_v2",
      user: process.env.DB_USER || "postgres",
      password: process.env.DB_PASSWORD || "",
      poolMin: 2,
      poolMax: 10,
      poolIdleTimeoutMs: 30000,
      poolConnectionTimeoutMs: 10000,
      queryTimeoutMs: 5000,
      enableQueryLogging: false,
      enableRetries: true,
      maxRetries: 3,
      retryDelayMs: 1000,
      ...config,
    };

    this.pool = new Pool({
      host: this.config.host,
      port: this.config.port,
      database: this.config.database,
      user: this.config.user,
      password: this.config.password,
      min: this.config.poolMin,
      max: this.config.poolMax,
      idleTimeoutMillis: this.config.poolIdleTimeoutMs,
      connectionTimeoutMillis: this.config.poolConnectionTimeoutMs,
      statement_timeout: this.config.queryTimeoutMs,
    });

    this.pool.on("error", (err) => {
      console.error("Unexpected database pool error:", err);
    });
  }

  /**
   * Initialize database connection and verify schema
   */
  async initialize(): Promise<void> {
    try {
      const client = await this.pool.connect();
      try {
        // Verify connection
        await client.query("SELECT 1");

        // Verify schema exists
        const schemaCheck = await client.query(`
          SELECT table_name 
          FROM information_schema.tables 
          WHERE table_schema = 'public' 
          AND table_name IN ('agent_profiles', 'agent_capabilities', 'performance_history')
        `);

        if (schemaCheck.rows.length < 3) {
          throw new Error(
            "Database schema not initialized. Run migrations first: psql < migrations/001_create_agent_registry_tables.sql"
          );
        }
      } finally {
        client.release();
      }
    } catch (error) {
      throw new Error(
        `Database initialization failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Register a new agent (INSERT)
   */
  async registerAgent(agent: AgentProfile): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");

      // Insert agent profile
      await client.query(
        `
        INSERT INTO agent_profiles (
          id, name, model_family, registered_at, last_active_at
        ) VALUES ($1, $2, $3, $4, $5)
        `,
        [
          agent.id,
          agent.name,
          agent.modelFamily,
          agent.registeredAt,
          agent.lastActiveAt,
        ]
      );

      // Insert capabilities
      for (const taskType of agent.capabilities.taskTypes) {
        await client.query(
          `
          INSERT INTO agent_capabilities (agent_id, capability_type, capability_value)
          VALUES ($1, 'task_type', $2)
          `,
          [agent.id, taskType]
        );
      }

      for (const language of agent.capabilities.languages) {
        await client.query(
          `
          INSERT INTO agent_capabilities (agent_id, capability_type, capability_value)
          VALUES ($1, 'language', $2)
          `,
          [agent.id, language]
        );
      }

      for (const specialization of agent.capabilities.specializations) {
        await client.query(
          `
          INSERT INTO agent_capabilities (agent_id, capability_type, capability_value)
          VALUES ($1, 'specialization', $2)
          `,
          [agent.id, specialization]
        );
      }

      // Insert performance history
      await client.query(
        `
        INSERT INTO performance_history (
          agent_id, success_rate, average_quality, average_latency, task_count
        ) VALUES ($1, $2, $3, $4, $5)
        `,
        [
          agent.id,
          agent.performanceHistory.successRate,
          agent.performanceHistory.averageQuality,
          agent.performanceHistory.averageLatency,
          agent.performanceHistory.taskCount,
        ]
      );

      // Insert current load
      await client.query(
        `
        INSERT INTO current_load (
          agent_id, active_tasks, queued_tasks, utilization_percent
        ) VALUES ($1, $2, $3, $4)
        `,
        [
          agent.id,
          agent.currentLoad.activeTasks,
          agent.currentLoad.queuedTasks,
          agent.currentLoad.utilizationPercent,
        ]
      );

      await client.query("COMMIT");
    } catch (error) {
      await client.query("ROLLBACK");
      throw new Error(
        `Failed to register agent: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get agent profile by ID (SELECT)
   */
  async getAgent(agentId: AgentId): Promise<AgentProfile | null> {
    const client = await this.pool.connect();

    try {
      // Use the view that joins all data
      const result = await client.query(
        `
        SELECT * FROM agent_profiles_with_capabilities
        WHERE id = $1
        `,
        [agentId]
      );

      if (result.rows.length === 0) {
        return null;
      }

      return this.mapRowToProfile(result.rows[0]);
    } finally {
      client.release();
    }
  }

  /**
   * Get all agents (SELECT)
   */
  async getAllAgents(): Promise<AgentProfile[]> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(`
        SELECT * FROM agent_profiles_with_capabilities
        ORDER BY last_active_at DESC
      `);

      return result.rows.map((row) => this.mapRowToProfile(row));
    } finally {
      client.release();
    }
  }

  /**
   * Query agents by capability
   */
  async queryAgentsByCapability(query: AgentQuery): Promise<AgentProfile[]> {
    const client = await this.pool.connect();

    try {
      let sql = `
        SELECT DISTINCT ap.* 
        FROM agent_profiles_with_capabilities ap
        WHERE 1=1
      `;

      const params: any[] = [];
      let paramIndex = 1;

      // Filter by task type
      if (query.taskType) {
        sql += ` AND $${paramIndex} = ANY(ap.task_types)`;
        params.push(query.taskType);
        paramIndex++;
      }

      // Filter by languages
      if (query.languages && query.languages.length > 0) {
        sql += ` AND ap.languages && $${paramIndex}::text[]`;
        params.push(query.languages);
        paramIndex++;
      }

      // Filter by utilization
      if (query.maxUtilization !== undefined) {
        sql += ` AND ap.utilization_percent <= $${paramIndex}`;
        params.push(query.maxUtilization);
        paramIndex++;
      }

      // Filter by success rate
      if (query.minSuccessRate !== undefined) {
        sql += ` AND ap.success_rate >= $${paramIndex}`;
        params.push(query.minSuccessRate);
        paramIndex++;
      }

      // Order by success rate
      sql += ` ORDER BY ap.success_rate DESC`;

      const result = await client.query(sql, params);
      return result.rows.map((row) => this.mapRowToProfile(row));
    } finally {
      client.release();
    }
  }

  /**
   * Update performance history (UPDATE)
   */
  async updatePerformance(
    agentId: AgentId,
    metrics: PerformanceMetrics
  ): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");

      // Get current performance history
      const currentResult = await client.query(
        `SELECT * FROM performance_history WHERE agent_id = $1`,
        [agentId]
      );

      if (currentResult.rows.length === 0) {
        throw new Error(`Agent ${agentId} not found`);
      }

      const current = currentResult.rows[0];

      // Calculate new running averages
      const taskCount = current.task_count + 1;
      const successRate =
        (current.success_rate * current.task_count +
          (metrics.success ? 1 : 0)) /
        taskCount;
      const averageQuality =
        (current.average_quality * current.task_count + metrics.qualityScore) /
        taskCount;
      const averageLatency =
        (current.average_latency * current.task_count + metrics.latencyMs) /
        taskCount;

      // Update performance history
      await client.query(
        `
        UPDATE performance_history
        SET success_rate = $1,
            average_quality = $2,
            average_latency = $3,
            task_count = $4,
            updated_at = CURRENT_TIMESTAMP
        WHERE agent_id = $5
        `,
        [successRate, averageQuality, averageLatency, taskCount, agentId]
      );

      // Insert performance event for audit trail
      await client.query(
        `
        INSERT INTO agent_performance_events (
          agent_id, success, quality_score, latency_ms, tokens_used, task_type
        ) VALUES ($1, $2, $3, $4, $5, $6)
        `,
        [
          agentId,
          metrics.success,
          metrics.qualityScore,
          metrics.latencyMs,
          metrics.tokensUsed,
          metrics.taskType,
        ]
      );

      // Update last active timestamp
      await client.query(
        `UPDATE agent_profiles SET last_active_at = CURRENT_TIMESTAMP WHERE id = $1`,
        [agentId]
      );

      await client.query("COMMIT");
    } catch (error) {
      await client.query("ROLLBACK");
      throw new Error(
        `Failed to update performance: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    } finally {
      client.release();
    }
  }

  /**
   * Update agent load (UPDATE)
   */
  async updateLoad(
    agentId: AgentId,
    activeTasksDelta: number,
    queuedTasksDelta: number
  ): Promise<void> {
    const client = await this.pool.connect();

    try {
      // Update with atomic increment/decrement
      await client.query(
        `
        UPDATE current_load
        SET active_tasks = GREATEST(0, active_tasks + $1),
            queued_tasks = GREATEST(0, queued_tasks + $2),
            utilization_percent = LEAST(100, (GREATEST(0, active_tasks + $1)::float / NULLIF(10, 0)) * 100),
            updated_at = CURRENT_TIMESTAMP
        WHERE agent_id = $3
        `,
        [activeTasksDelta, queuedTasksDelta, agentId]
      );
    } finally {
      client.release();
    }
  }

  /**
   * Unregister agent (DELETE)
   */
  async unregisterAgent(agentId: AgentId): Promise<boolean> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");

      // Delete cascades to all related tables (configured in migration)
      const result = await client.query(
        `DELETE FROM agent_profiles WHERE id = $1`,
        [agentId]
      );

      await client.query("COMMIT");

      return result.rowCount !== null && result.rowCount > 0;
    } catch (error) {
      await client.query("ROLLBACK");
      throw new Error(
        `Failed to unregister agent: ${
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
  async getStats(): Promise<RegistryStats> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(`
        SELECT 
          COUNT(*) as total_agents,
          COUNT(*) FILTER (WHERE active_tasks > 0) as active_agents,
          COUNT(*) FILTER (WHERE active_tasks = 0) as idle_agents,
          AVG(utilization_percent) as avg_utilization,
          AVG(success_rate) as avg_success_rate,
          AVG(average_quality) as avg_quality,
          MAX(last_active_at) as last_updated
        FROM agent_profiles_with_capabilities
      `);

      const stats = result.rows[0];

      return {
        totalAgents: parseInt(stats.total_agents),
        activeAgents: parseInt(stats.active_agents),
        idleAgents: parseInt(stats.idle_agents),
        averageUtilization: parseFloat(stats.avg_utilization) || 0,
        averageSuccessRate: parseFloat(stats.avg_success_rate) || 0,
        lastUpdated: stats.last_updated || new Date().toISOString(),
      };
    } finally {
      client.release();
    }
  }

  /**
   * Clean up stale agents
   */
  async cleanupStaleAgents(staleThresholdMs: number): Promise<number> {
    const client = await this.pool.connect();

    try {
      const staleTimestamp = new Date(
        Date.now() - staleThresholdMs
      ).toISOString();

      const result = await client.query(
        `
        DELETE FROM agent_profiles 
        WHERE last_active_at < $1
        RETURNING id
        `,
        [staleTimestamp]
      );

      return result.rowCount || 0;
    } finally {
      client.release();
    }
  }

  /**
   * Health check
   */
  async healthCheck(): Promise<{
    healthy: boolean;
    latencyMs: number;
    poolStats: {
      total: number;
      idle: number;
      waiting: number;
    };
  }> {
    const startTime = Date.now();

    try {
      const client = await this.pool.connect();
      try {
        await client.query("SELECT 1");
        const latencyMs = Date.now() - startTime;

        return {
          healthy: true,
          latencyMs,
          poolStats: {
            total: this.pool.totalCount,
            idle: this.pool.idleCount,
            waiting: this.pool.waitingCount,
          },
        };
      } finally {
        client.release();
      }
    } catch (error) {
      return {
        healthy: false,
        latencyMs: Date.now() - startTime,
        poolStats: {
          total: this.pool.totalCount,
          idle: this.pool.idleCount,
          waiting: this.pool.waitingCount,
        },
      };
    }
  }

  /**
   * Close database connection pool
   */
  async close(): Promise<void> {
    await this.pool.end();
  }

  /**
   * Map database row to AgentProfile
   */
  private mapRowToProfile(row: any): AgentProfile {
    return {
      id: row.id,
      name: row.name,
      modelFamily: row.model_family,
      capabilities: {
        taskTypes: row.task_types || [],
        languages: row.languages || [],
        specializations: row.specializations || [],
      },
      performanceHistory: {
        successRate: parseFloat(row.success_rate) || 0,
        averageQuality: parseFloat(row.average_quality) || 0,
        averageLatency: parseFloat(row.average_latency) || 0,
        taskCount: parseInt(row.task_count) || 0,
      },
      currentLoad: {
        activeTasks: parseInt(row.active_tasks) || 0,
        queuedTasks: parseInt(row.queued_tasks) || 0,
        utilizationPercent: parseFloat(row.utilization_percent) || 0,
      },
      registeredAt: row.registered_at,
      lastActiveAt: row.last_active_at,
    };
  }

  /**
   * Execute query with retry logic
   */
  private async executeWithRetry<T>(
    operation: (client: PoolClient) => Promise<T>
  ): Promise<T> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt < this.config.maxRetries; attempt++) {
      try {
        const client = await this.pool.connect();
        try {
          return await operation(client);
        } finally {
          client.release();
        }
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        if (this.config.enableQueryLogging) {
          console.log(
            `Query attempt ${attempt + 1}/${this.config.maxRetries} failed:`,
            lastError.message
          );
        }

        if (attempt < this.config.maxRetries - 1) {
          await new Promise((resolve) =>
            setTimeout(resolve, this.config.retryDelayMs)
          );
        }
      }
    }

    throw lastError || new Error("Query failed after retries");
  }
}
