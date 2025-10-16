/**
 * Performance Tracker Database Client
 *
 * PostgreSQL client for the Performance Tracker (ARBITER-004).
 * Provides ACID-compliant persistence for performance events, agent profiles, and system health metrics.
 *
 * @author @darianrosebrook
 */

import { PerformanceStats } from "../rl/PerformanceTracker.js";
import { PerformanceEvent } from "../types/agentic-rl.js";
import { Logger } from "../utils/Logger.js";
import { ConnectionPoolManager } from "./ConnectionPoolManager.js";

export interface PerformanceTrackerDatabaseConfig {
  /** Enable query logging */
  enableQueryLogging: boolean;
  /** Enable connection retries */
  enableRetries: boolean;
  /** Maximum number of retry attempts */
  maxRetries: number;
  /** Delay between retries in milliseconds */
  retryDelayMs: number;
  /** Batch size for bulk operations */
  batchSize: number;
}

export class PerformanceTrackerDatabaseClient {
  private poolManager: ConnectionPoolManager;
  private config: PerformanceTrackerDatabaseConfig;
  private logger: Logger;

  constructor(config: Partial<PerformanceTrackerDatabaseConfig> = {}) {
    this.config = {
      enableQueryLogging: false,
      enableRetries: true,
      maxRetries: 3,
      retryDelayMs: 1000,
      batchSize: 100,
      ...config,
    };

    this.logger = new Logger("PerformanceTrackerDatabaseClient");
    this.poolManager = ConnectionPoolManager.getInstance();
  }

  /**
   * Initialize database connection and verify schema
   */
  async initialize(): Promise<void> {
    try {
      this.logger.info("Initializing Performance Tracker database client...");

      // Test connection
      const pool = this.poolManager.getPool();
      const client = await pool.connect();
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
        "Performance Tracker database client initialized successfully"
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
      this.logger.info("Shutting down Performance Tracker database client...");
      // Pool manager handles shutdown centrally
      this.logger.info("Performance Tracker database client shutdown complete");
    } catch (error) {
      this.logger.error("Error during shutdown:", error);
    }
  }

  /**
   * Verify that required tables exist
   */
  private async verifySchema(): Promise<void> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      // Check for performance tracking tables
      const requiredTables = [
        "performance_events",
        "agent_performance_profiles",
        "benchmark_datasets",
        "system_health_metrics",
      ];

      for (const tableName of requiredTables) {
        const result = await client.query(
          "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)",
          [tableName]
        );

        if (!result.rows[0].exists) {
          throw new Error(`Required table '${tableName}' does not exist`);
        }
      }

      this.logger.info("All required performance tracking tables verified");
    } finally {
      client.release();
    }
  }

  /**
   * Store a performance event in the database
   */
  async storePerformanceEvent(event: PerformanceEvent): Promise<void> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      const eventId = `${event.type}-${Date.now()}-${Math.random()
        .toString(36)
        .substr(2, 9)}`;
      const agentId = (event.data?.agentId as string) || "";
      const taskId = (event.data?.taskId as string) || "";
      const routingDecision = event.data?.routingDecision || {};
      const outcome = event.data?.outcome || {};
      const context = event.data || {};

      await client.query(
        `INSERT INTO performance_events (
          event_id, event_type, agent_id, task_id, timestamp,
          routing_decision, task_outcome, context_data, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())`,
        [
          eventId,
          event.type,
          agentId,
          taskId,
          event.timestamp,
          JSON.stringify(routingDecision),
          JSON.stringify(outcome),
          JSON.stringify(context),
        ]
      );

      if (this.config.enableQueryLogging) {
        this.logger.debug(`Stored performance event: ${eventId}`);
      }
    } catch (error) {
      this.logger.error(`Failed to store performance event:`, error);
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Store multiple performance events in batch
   */
  async storePerformanceEventsBatch(events: PerformanceEvent[]): Promise<void> {
    if (events.length === 0) return;

    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      await client.query("BEGIN");

      for (const event of events) {
        const eventId = `${event.type}-${Date.now()}-${Math.random()
          .toString(36)
          .substr(2, 9)}`;
        const agentId = (event.data?.agentId as string) || "";
        const taskId = (event.data?.taskId as string) || "";
        const routingDecision = event.data?.routingDecision || {};
        const outcome = event.data?.outcome || {};
        const context = event.data || {};

        await client.query(
          `INSERT INTO performance_events (
            event_id, event_type, agent_id, task_id, timestamp,
            routing_decision, task_outcome, context_data, created_at
          ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
          ON CONFLICT (event_id) DO UPDATE SET
            event_type = EXCLUDED.event_type,
            agent_id = EXCLUDED.agent_id,
            task_id = EXCLUDED.task_id,
            timestamp = EXCLUDED.timestamp,
            routing_decision = EXCLUDED.routing_decision,
            task_outcome = EXCLUDED.task_outcome,
            context_data = EXCLUDED.context_data,
            updated_at = NOW()`,
          [
            eventId,
            event.type,
            agentId,
            taskId,
            event.timestamp,
            JSON.stringify(routingDecision),
            JSON.stringify(outcome),
            JSON.stringify(context),
          ]
        );
      }

      await client.query("COMMIT");

      if (this.config.enableQueryLogging) {
        this.logger.debug(
          `Stored ${events.length} performance events in batch`
        );
      }
    } catch (error) {
      await client.query("ROLLBACK");
      this.logger.error(`Failed to store performance events batch:`, error);
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Store agent performance profile
   */
  async storeAgentPerformanceProfile(
    agentId: string,
    profile: {
      capabilities: string[];
      baselineMetrics: {
        latencyMs: number;
        accuracy: number;
        costPerTask: number;
        reliability: number;
      };
      registrationTimestamp: string;
    }
  ): Promise<void> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      await client.query(
        `INSERT INTO agent_performance_profiles (
          agent_id, capabilities, baseline_latency_ms, baseline_accuracy,
          baseline_cost_per_task, baseline_reliability, registration_timestamp, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
        ON CONFLICT (agent_id) DO UPDATE SET
          capabilities = EXCLUDED.capabilities,
          baseline_latency_ms = EXCLUDED.baseline_latency_ms,
          baseline_accuracy = EXCLUDED.baseline_accuracy,
          baseline_cost_per_task = EXCLUDED.baseline_cost_per_task,
          baseline_reliability = EXCLUDED.baseline_reliability,
          registration_timestamp = EXCLUDED.registration_timestamp,
          updated_at = NOW()`,
        [
          agentId,
          JSON.stringify(profile.capabilities),
          profile.baselineMetrics.latencyMs,
          profile.baselineMetrics.accuracy,
          profile.baselineMetrics.costPerTask,
          profile.baselineMetrics.reliability,
          profile.registrationTimestamp,
        ]
      );

      if (this.config.enableQueryLogging) {
        this.logger.debug(`Stored agent performance profile: ${agentId}`);
      }
    } catch (error) {
      this.logger.error(`Failed to store agent profile ${agentId}:`, error);
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Store benchmark dataset
   */
  async storeBenchmarkDataset(
    datasetId: string,
    dataset: {
      name: string;
      description: string;
      taskTypes: string[];
      metrics: Record<string, number>;
      createdAt: string;
    }
  ): Promise<void> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      await client.query(
        `INSERT INTO benchmark_datasets (
          dataset_id, name, description, task_types, metrics, created_at, db_created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
        ON CONFLICT (dataset_id) DO UPDATE SET
          name = EXCLUDED.name,
          description = EXCLUDED.description,
          task_types = EXCLUDED.task_types,
          metrics = EXCLUDED.metrics,
          created_at = EXCLUDED.created_at,
          updated_at = NOW()`,
        [
          datasetId,
          dataset.name,
          dataset.description,
          JSON.stringify(dataset.taskTypes),
          JSON.stringify(dataset.metrics),
          dataset.createdAt,
        ]
      );

      if (this.config.enableQueryLogging) {
        this.logger.debug(`Stored benchmark dataset: ${datasetId}`);
      }
    } catch (error) {
      this.logger.error(
        `Failed to store benchmark dataset ${datasetId}:`,
        error
      );
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Store system health metrics
   */
  async storeSystemHealthMetrics(metrics: {
    timestamp: string;
    cpuUsage: number;
    memoryUsage: number;
    activeConnections: number;
    queueDepth: number;
    errorRate: number;
    responseTimeMs: number;
  }): Promise<void> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      await client.query(
        `INSERT INTO system_health_metrics (
          timestamp, cpu_usage, memory_usage, active_connections,
          queue_depth, error_rate, response_time_ms, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())`,
        [
          metrics.timestamp,
          metrics.cpuUsage,
          metrics.memoryUsage,
          metrics.activeConnections,
          metrics.queueDepth,
          metrics.errorRate,
          metrics.responseTimeMs,
        ]
      );

      if (this.config.enableQueryLogging) {
        this.logger.debug(`Stored system health metrics: ${metrics.timestamp}`);
      }
    } catch (error) {
      this.logger.error(`Failed to store system health metrics:`, error);
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Retrieve performance statistics
   */
  async getPerformanceStats(
    startTime?: string,
    endTime?: string
  ): Promise<PerformanceStats> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      let whereClause = "";
      const params: any[] = [];

      if (startTime && endTime) {
        whereClause = "WHERE timestamp BETWEEN $1 AND $2";
        params.push(startTime, endTime);
      } else if (startTime) {
        whereClause = "WHERE timestamp >= $1";
        params.push(startTime);
      } else if (endTime) {
        whereClause = "WHERE timestamp <= $1";
        params.push(endTime);
      }

      const result = await client.query(
        `SELECT 
          COUNT(*) as total_events,
          COUNT(CASE WHEN event_type = 'routing_decision' THEN 1 END) as routing_decisions,
          COUNT(CASE WHEN event_type = 'task_execution' THEN 1 END) as task_executions,
          COUNT(CASE WHEN event_type = 'evaluation_outcome' THEN 1 END) as evaluation_outcomes,
          AVG(CASE WHEN task_outcome->>'completionTimeMs' IS NOT NULL 
              THEN (task_outcome->>'completionTimeMs')::numeric 
              ELSE NULL END) as avg_completion_time,
          AVG(CASE WHEN task_outcome->>'success' = 'true' THEN 1.0 ELSE 0.0 END) as success_rate,
          MIN(timestamp) as collection_started_at,
          MAX(timestamp) as last_collection_at
        FROM performance_events ${whereClause}`,
        params
      );

      const stats = result.rows[0];

      return {
        totalRoutingDecisions: parseInt(stats.routing_decisions) || 0,
        totalTaskExecutions: parseInt(stats.task_executions) || 0,
        totalEvaluationOutcomes: parseInt(stats.evaluation_outcomes) || 0,
        averageCompletionTimeMs: parseFloat(stats.avg_completion_time) || 0,
        overallSuccessRate: parseFloat(stats.success_rate) || 0,
        collectionStartedAt:
          stats.collection_started_at || new Date().toISOString(),
        lastUpdatedAt: stats.last_collection_at || new Date().toISOString(),
      };
    } catch (error) {
      this.logger.error("Failed to retrieve performance stats:", error);
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Retrieve performance events by time range
   */
  async getPerformanceEvents(
    startTime?: string,
    endTime?: string,
    limit: number = 1000
  ): Promise<PerformanceEvent[]> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      let whereClause = "";
      const params: any[] = [];
      let paramIndex = 1;

      if (startTime && endTime) {
        whereClause = "WHERE timestamp BETWEEN $1 AND $2";
        params.push(startTime, endTime);
        paramIndex = 3;
      } else if (startTime) {
        whereClause = "WHERE timestamp >= $1";
        params.push(startTime);
        paramIndex = 2;
      } else if (endTime) {
        whereClause = "WHERE timestamp <= $1";
        params.push(endTime);
        paramIndex = 2;
      }

      params.push(limit);

      const result = await client.query(
        `SELECT 
          event_id, event_type, agent_id, task_id, timestamp,
          routing_decision, task_outcome, context_data
        FROM performance_events ${whereClause}
        ORDER BY timestamp DESC
        LIMIT $${paramIndex}`,
        params
      );

      return result.rows.map((row) => ({
        type: row.event_type,
        timestamp: row.timestamp,
        data: {
          agentId: row.agent_id,
          taskId: row.task_id,
          routingDecision: JSON.parse(row.routing_decision || "{}"),
          outcome: JSON.parse(row.task_outcome || "{}"),
          ...JSON.parse(row.context_data || "{}"),
        },
      }));
    } catch (error) {
      this.logger.error("Failed to retrieve performance events:", error);
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Clean up old performance data
   */
  async cleanupOldData(retentionDays: number = 30): Promise<number> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      const result = await client.query(
        `DELETE FROM performance_events 
        WHERE created_at < NOW() - INTERVAL '${retentionDays} days'`
      );

      const deletedCount = result.rowCount || 0;
      this.logger.info(`Cleaned up ${deletedCount} old performance events`);

      return deletedCount;
    } catch (error) {
      this.logger.error("Failed to cleanup old performance data:", error);
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Get database health status
   */
  async getHealthStatus(): Promise<{
    status: "healthy" | "degraded" | "unhealthy";
    details: Record<string, any>;
  }> {
    const pool = this.poolManager.getPool();
    const client = await pool.connect();

    try {
      // Check connection health
      await client.query("SELECT 1");

      // Check table sizes
      const tableSizes = await client.query(`
        SELECT 
          schemaname,
          tablename,
          pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
        FROM pg_tables 
        WHERE schemaname = 'public' 
        AND tablename IN ('performance_events', 'agent_performance_profiles', 'benchmark_datasets', 'system_health_metrics')
        ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
      `);

      // Check recent activity
      const recentActivity = await client.query(`
        SELECT COUNT(*) as event_count
        FROM performance_events 
        WHERE created_at > NOW() - INTERVAL '1 hour'
      `);

      return {
        status: "healthy",
        details: {
          tableSizes: tableSizes.rows,
          recentActivity: recentActivity.rows[0].event_count,
          timestamp: new Date().toISOString(),
        },
      };
    } catch (error) {
      this.logger.error("Database health check failed:", error);
      return {
        status: "unhealthy",
        details: {
          error: error instanceof Error ? error.message : String(error),
          timestamp: new Date().toISOString(),
        },
      };
    } finally {
      client.release();
    }
  }
}
