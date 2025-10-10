/**
 * @fileoverview Task Queue Database Persistence Layer
 *
 * Provides database operations for the TaskQueue, ensuring durability
 * and crash recovery for queued tasks.
 *
 * @author @darianrosebrook
 */

import { Task, TaskStatus } from "../types/arbiter-orchestration";

export interface DatabaseConnection {
  query(sql: string, params?: any[]): Promise<any[]>;
  transaction<T>(callback: (tx: DatabaseConnection) => Promise<T>): Promise<T>;
}

/**
 * Task Queue Database Operations
 */
export class TaskQueuePersistence {
  constructor(private db: DatabaseConnection) {}

  /**
   * Initialize database tables if they don't exist
   */
  async initialize(): Promise<void> {
    // Tables are created by migration, but we could add checks here
    await this.ensureConfiguration();
  }

  /**
   * Persist a task to the database
   */
  async persistTask(task: Task): Promise<void> {
    const sql = `
      INSERT INTO task_queue (
        task_id, task_type, description, priority, timeout_ms,
        attempts, max_attempts, budget_max_files, budget_max_loc,
        required_capabilities, task_metadata, status
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
      ON CONFLICT (task_id) DO UPDATE SET
        priority = EXCLUDED.priority,
        timeout_ms = EXCLUDED.timeout_ms,
        attempts = EXCLUDED.attempts,
        max_attempts = EXCLUDED.max_attempts,
        budget_max_files = EXCLUDED.budget_max_files,
        budget_max_loc = EXCLUDED.budget_max_loc,
        required_capabilities = EXCLUDED.required_capabilities,
        task_metadata = EXCLUDED.task_metadata,
        updated_at = NOW()
    `;

    await this.db.query(sql, [
      task.id,
      task.type,
      task.description,
      task.priority,
      task.timeoutMs,
      task.attempts,
      task.maxAttempts,
      task.budget.maxFiles,
      task.budget.maxLoc,
      JSON.stringify(task.requiredCapabilities || {}),
      JSON.stringify(task.metadata || {}),
      "queued",
    ]);
  }

  /**
   * Load all queued tasks from database
   */
  async loadQueuedTasks(): Promise<Task[]> {
    const sql = `
      SELECT
        task_id as id,
        task_type as type,
        description,
        priority,
        timeout_ms as "timeoutMs",
        attempts,
        max_attempts as "maxAttempts",
        budget_max_files as "budgetMaxFiles",
        budget_max_loc as "budgetMaxLoc",
        required_capabilities,
        task_metadata as metadata,
        created_at as "createdAt"
      FROM task_queue
      WHERE status = 'queued'
      ORDER BY priority DESC, created_at ASC
    `;

    const rows = await this.db.query(sql);

    return rows.map((row) => ({
      id: row.id,
      type: row.type,
      description: row.description,
      priority: row.priority,
      timeoutMs: row.timeoutMs,
      attempts: row.attempts,
      maxAttempts: row.maxAttempts,
      budget: {
        maxFiles: row.budgetMaxFiles,
        maxLoc: row.budgetMaxLoc,
      },
      requiredCapabilities: row.required_capabilities || {},
      metadata: row.metadata || {},
      createdAt: new Date(row.createdAt),
    }));
  }

  /**
   * Mark task as dequeued
   */
  async markDequeued(taskId: string): Promise<void> {
    const sql = `
      UPDATE task_queue
      SET status = 'dequeued', dequeued_at = NOW(), updated_at = NOW()
      WHERE task_id = $1 AND status = 'queued'
    `;

    await this.db.query(sql, [taskId]);
  }

  /**
   * Mark task as completed
   */
  async markCompleted(taskId: string): Promise<void> {
    const sql = `
      UPDATE task_queue
      SET status = 'completed', completed_at = NOW(), updated_at = NOW()
      WHERE task_id = $1
    `;

    await this.db.query(sql, [taskId]);
  }

  /**
   * Mark task as failed
   */
  async markFailed(taskId: string, error?: string): Promise<void> {
    const sql = `
      UPDATE task_queue
      SET status = 'failed', completed_at = NOW(), updated_at = NOW(),
          task_metadata = jsonb_set(task_metadata, '{lastError}', $2::jsonb, true)
      WHERE task_id = $1
    `;

    await this.db.query(sql, [
      taskId,
      JSON.stringify(error || "Unknown error"),
    ]);
  }

  /**
   * Increment task attempts
   */
  async incrementAttempts(taskId: string): Promise<number> {
    const sql = `
      UPDATE task_queue
      SET attempts = attempts + 1, updated_at = NOW()
      WHERE task_id = $1
      RETURNING attempts
    `;

    const result = await this.db.query(sql, [taskId]);
    return result[0]?.attempts || 0;
  }

  /**
   * Clear all queued tasks (for shutdown/reset)
   */
  async clearAllQueued(): Promise<void> {
    const sql = `
      UPDATE task_queue
      SET status = 'cancelled', completed_at = NOW(), updated_at = NOW()
      WHERE status = 'queued'
    `;

    await this.db.query(sql);
  }

  /**
   * Get queue statistics from database
   */
  async getQueueStats(): Promise<{
    depth: number;
    totalEnqueued: number;
    totalDequeued: number;
    totalCompleted: number;
    totalFailed: number;
    averageWaitTimeMs: number;
    priorityDistribution: Record<number, number>;
    statusDistribution: Record<TaskStatus, number>;
  }> {
    // Get basic counts
    const countsSql = `
      SELECT
        COUNT(*) FILTER (WHERE status = 'queued') as depth,
        COUNT(*) FILTER (WHERE status IN ('dequeued', 'completed', 'failed')) as total_processed,
        COUNT(*) FILTER (WHERE status = 'completed') as total_completed,
        COUNT(*) FILTER (WHERE status = 'failed') as total_failed,
        COUNT(*) FILTER (WHERE status = 'dequeued') as total_dequeued
      FROM task_queue
    `;

    const countsResult = await this.db.query(countsSql);
    const counts = countsResult[0];

    // Get priority distribution
    const prioritySql = `
      SELECT priority, COUNT(*) as count
      FROM task_queue
      WHERE status = 'queued'
      GROUP BY priority
    `;

    const priorityRows = await this.db.query(prioritySql);
    const priorityDistribution: Record<number, number> = {};
    priorityRows.forEach((row: any) => {
      priorityDistribution[row.priority] = parseInt(row.count);
    });

    // Get status distribution
    const statusSql = `
      SELECT status, COUNT(*) as count
      FROM task_queue
      GROUP BY status
    `;

    const statusRows = await this.db.query(statusSql);
    const statusDistribution: Record<TaskStatus, number> = {
      [TaskStatus.QUEUED]: 0,
      [TaskStatus.ROUTING]: 0,
      [TaskStatus.ASSIGNED]: 0,
      [TaskStatus.EXECUTING]: 0,
      [TaskStatus.VALIDATING]: 0,
      [TaskStatus.COMPLETED]: 0,
      [TaskStatus.FAILED]: 0,
      [TaskStatus.TIMEOUT]: 0,
      [TaskStatus.CANCELED]: 0,
    };

    statusRows.forEach((row: any) => {
      const status = row.status as TaskStatus;
      if (status in statusDistribution) {
        statusDistribution[status] = parseInt(row.count);
      }
    });

    // Calculate average wait time (simplified)
    const waitTimeSql = `
      SELECT
        AVG(EXTRACT(EPOCH FROM (COALESCE(completed_at, NOW()) - created_at))) * 1000 as avg_wait_ms
      FROM task_queue
      WHERE status IN ('completed', 'failed')
      AND completed_at IS NOT NULL
    `;

    const waitTimeResult = await this.db.query(waitTimeSql);
    const averageWaitTimeMs = waitTimeResult[0]?.avg_wait_ms || 0;

    return {
      depth: parseInt(counts.depth) || 0,
      totalEnqueued:
        parseInt(counts.total_processed) + parseInt(counts.depth) || 0,
      totalDequeued: parseInt(counts.total_dequeued) || 0,
      totalCompleted: parseInt(counts.total_completed) || 0,
      totalFailed: parseInt(counts.total_failed) || 0,
      averageWaitTimeMs: parseFloat(averageWaitTimeMs) || 0,
      priorityDistribution,
      statusDistribution,
    };
  }

  /**
   * Store queue statistics snapshot
   */
  async storeQueueStats(stats: {
    depth: number;
    totalEnqueued: number;
    totalDequeued: number;
    totalCompleted: number;
    totalFailed: number;
    averageWaitTimeMs: number;
    priorityDistribution: Record<number, number>;
    statusDistribution: Record<TaskStatus, number>;
  }): Promise<void> {
    const sql = `
      INSERT INTO queue_statistics (
        queue_depth, total_enqueued, total_dequeued, total_completed, total_failed,
        average_wait_time_ms, priority_distribution, status_distribution
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    `;

    await this.db.query(sql, [
      stats.depth,
      stats.totalEnqueued,
      stats.totalDequeued,
      stats.totalCompleted,
      stats.totalFailed,
      stats.averageWaitTimeMs,
      JSON.stringify(stats.priorityDistribution),
      JSON.stringify(stats.statusDistribution),
    ]);
  }

  /**
   * Load configuration from database
   */
  async loadConfiguration(): Promise<Record<string, any>> {
    const sql = `SELECT config_key, config_value FROM queue_configuration`;
    const rows = await this.db.query(sql);

    const config: Record<string, any> = {};
    rows.forEach((row: any) => {
      config[row.config_key] =
        typeof row.config_value === "string"
          ? JSON.parse(row.config_value)
          : row.config_value;
    });

    return config;
  }

  /**
   * Save configuration to database
   */
  async saveConfiguration(
    key: string,
    value: any,
    description?: string
  ): Promise<void> {
    const sql = `
      INSERT INTO queue_configuration (config_key, config_value, description, updated_at)
      VALUES ($1, $2, $3, NOW())
      ON CONFLICT (config_key) DO UPDATE SET
        config_value = EXCLUDED.config_value,
        description = EXCLUDED.description,
        updated_at = NOW()
    `;

    await this.db.query(sql, [key, JSON.stringify(value), description]);
  }

  /**
   * Ensure default configuration exists
   */
  private async ensureConfiguration(): Promise<void> {
    const defaults = {
      maxCapacity: 1000,
      defaultTimeoutMs: 30000,
      maxRetries: 3,
      priorityMode: "priority",
      persistenceEnabled: true,
    };

    for (const [key, value] of Object.entries(defaults)) {
      await this.saveConfiguration(key, value, `Default ${key} configuration`);
    }
  }

  /**
   * Clean up old completed/failed tasks (data retention)
   */
  async cleanupOldTasks(olderThanDays: number = 30): Promise<number> {
    const sql = `
      DELETE FROM task_queue
      WHERE status IN ('completed', 'failed', 'cancelled')
      AND completed_at < NOW() - INTERVAL '${olderThanDays} days'
    `;

    const result = await this.db.query(sql);
    return result.rowCount || 0;
  }

  /**
   * Get tasks that have timed out
   */
  async getTimedOutTasks(): Promise<Task[]> {
    const sql = `
      SELECT
        task_id as id,
        task_type as type,
        description,
        priority,
        timeout_ms as "timeoutMs",
        attempts,
        max_attempts as "maxAttempts",
        budget_max_files as "budgetMaxFiles",
        budget_max_loc as "budgetMaxLoc",
        required_capabilities,
        task_metadata as metadata,
        created_at as "createdAt"
      FROM task_queue
      WHERE status = 'queued'
      AND (created_at + (timeout_ms || ' milliseconds')::interval) < NOW()
    `;

    const rows = await this.db.query(sql);

    return rows.map((row) => ({
      id: row.id,
      type: row.type,
      description: row.description,
      priority: row.priority,
      timeoutMs: row.timeoutMs,
      attempts: row.attempts,
      maxAttempts: row.maxAttempts,
      budget: {
        maxFiles: row.budgetMaxFiles,
        maxLoc: row.budgetMaxLoc,
      },
      requiredCapabilities: row.required_capabilities || {},
      metadata: row.metadata || {},
      createdAt: new Date(row.createdAt),
    }));
  }
}
