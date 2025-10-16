/**
 * Learning Database Client
 *
 * Repository pattern for learning coordination data persistence
 * Handles learning sessions, iterations, error patterns, and context snapshots
 *
 * @author @darianrosebrook
 */

import type { Pool, PoolClient } from "pg";
import type {
  ContextSnapshot,
  ErrorPattern,
  LearningIteration,
  LearningSession,
  LearningSessionConfig,
  LearningSummary,
} from "../types/learning-coordination.js";
import {
  ErrorCategory,
  LearningSessionStatus,
} from "../types/learning-coordination.js";

/**
 * Learning Database Client
 *
 * Provides CRUD operations for learning coordination data
 */
export class LearningDatabaseClient {
  constructor(private readonly _pool: Pool) {}

  /**
   * Get connection pool
   */
  getPool(): Pool {
    return this.pool;
  }

  /**
   * Create a new learning session
   */
  async createSession(session: LearningSession): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query(
        `INSERT INTO learning_sessions (
          session_id, task_id, agent_id, status,
          max_iterations, progress_timeout, no_progress_limit,
          resource_budget_mb, compression_ratio, quality_threshold,
          enable_adaptive_prompting, enable_error_recognition,
          start_time, iteration_count, quality_score,
          improvement_trajectory, error_patterns
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)`,
        [
          session.sessionId,
          session.taskId,
          session.agentId,
          session.status,
          session.config.maxIterations,
          session.config.progressTimeout,
          session.config.noProgressLimit,
          session.config.resourceBudgetMB,
          session.config.compressionRatio,
          session.config.qualityThreshold,
          session.config.enableAdaptivePrompting,
          session.config.enableErrorRecognition,
          session.startTime,
          session.iterationCount,
          session.qualityScore,
          JSON.stringify(session.improvementTrajectory),
          JSON.stringify(session.errorPatterns),
        ]
      );
    } finally {
      client.release();
    }
  }

  /**
   * Update learning session
   */
  async updateSession(
    sessionId: string,
    updates: Partial<LearningSession>
  ): Promise<void> {
    const client = await this.pool.connect();

    try {
      const setClauses: string[] = [];
      const values: unknown[] = [];
      let paramIndex = 1;

      if (updates.status !== undefined) {
        setClauses.push(`status = $${paramIndex++}`);
        values.push(updates.status);
      }

      if (updates.iterationCount !== undefined) {
        setClauses.push(`iteration_count = $${paramIndex++}`);
        values.push(updates.iterationCount);
      }

      if (updates.qualityScore !== undefined) {
        setClauses.push(`quality_score = $${paramIndex++}`);
        values.push(updates.qualityScore);
      }

      if (updates.improvementTrajectory !== undefined) {
        setClauses.push(`improvement_trajectory = $${paramIndex++}`);
        values.push(JSON.stringify(updates.improvementTrajectory));
      }

      if (updates.errorPatterns !== undefined) {
        setClauses.push(`error_patterns = $${paramIndex++}`);
        values.push(JSON.stringify(updates.errorPatterns));
      }

      if (updates.endTime !== undefined) {
        setClauses.push(`end_time = $${paramIndex++}`);
        values.push(updates.endTime);
      }

      if (updates.finalResult !== undefined) {
        setClauses.push(`final_result = $${paramIndex++}`);
        values.push(JSON.stringify(updates.finalResult));
      }

      if (updates.learningSummary !== undefined) {
        setClauses.push(`learning_summary = $${paramIndex++}`);
        values.push(JSON.stringify(updates.learningSummary));
      }

      setClauses.push(`updated_at = NOW()`);

      if (setClauses.length === 1) {
        return;
      }

      values.push(sessionId);

      await client.query(
        `UPDATE learning_sessions SET ${setClauses.join(
          ", "
        )} WHERE session_id = $${paramIndex}`,
        values
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get learning session by ID
   */
  async getSession(sessionId: string): Promise<LearningSession | null> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(
        `SELECT * FROM learning_sessions WHERE session_id = $1`,
        [sessionId]
      );

      if (result.rows.length === 0) {
        return null;
      }

      return this.mapRowToSession(result.rows[0]);
    } finally {
      client.release();
    }
  }

  /**
   * Get sessions by task ID
   */
  async getSessionsByTask(taskId: string): Promise<LearningSession[]> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(
        `SELECT * FROM learning_sessions WHERE task_id = $1 ORDER BY start_time DESC`,
        [taskId]
      );

      return result.rows.map((row) => this.mapRowToSession(row));
    } finally {
      client.release();
    }
  }

  /**
   * Create learning iteration
   */
  async createIteration(iteration: LearningIteration): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query(
        `INSERT INTO learning_iterations (
          iteration_id, session_id, iteration_number,
          start_time, end_time, duration_ms,
          context_snapshot_id, error_detected, error_category,
          quality_score, improvement_delta, resource_usage_mb,
          prompt_modifications, feedback
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)`,
        [
          iteration.iterationId,
          iteration.sessionId,
          iteration.iterationNumber,
          iteration.startTime,
          iteration.endTime,
          iteration.durationMs,
          iteration.contextSnapshotId,
          iteration.errorDetected,
          iteration.errorCategory,
          iteration.qualityScore,
          iteration.improvementDelta,
          iteration.resourceUsageMB,
          JSON.stringify(iteration.promptModifications),
          iteration.feedback ? JSON.stringify(iteration.feedback) : null,
        ]
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get iterations for a session
   */
  async getIterations(sessionId: string): Promise<LearningIteration[]> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(
        `SELECT * FROM learning_iterations WHERE session_id = $1 ORDER BY iteration_number`,
        [sessionId]
      );

      return result.rows.map((row) => this.mapRowToIteration(row));
    } finally {
      client.release();
    }
  }

  /**
   * Create or update error pattern
   */
  async upsertErrorPattern(pattern: ErrorPattern): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query(
        `INSERT INTO error_patterns (
          pattern_id, category, pattern, frequency, confidence,
          detected_at, remediation_strategy, success_rate, examples
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (pattern_id) DO UPDATE SET
          frequency = error_patterns.frequency + 1,
          confidence = $5,
          success_rate = $8,
          updated_at = NOW()`,
        [
          pattern.patternId,
          pattern.category,
          pattern.pattern,
          pattern.frequency,
          pattern.confidence,
          pattern.detectedAt,
          pattern.remediationStrategy,
          pattern.successRate,
          JSON.stringify(pattern.examples),
        ]
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get error patterns by category
   */
  async getErrorPatterns(category?: string): Promise<ErrorPattern[]> {
    const client = await this.pool.connect();

    try {
      const query = category
        ? `SELECT * FROM error_patterns WHERE category = $1 ORDER BY confidence DESC, frequency DESC`
        : `SELECT * FROM error_patterns ORDER BY confidence DESC, frequency DESC`;

      const params = category ? [category] : [];
      const result = await client.query(query, params);

      return result.rows.map((row) => this.mapRowToErrorPattern(row));
    } finally {
      client.release();
    }
  }

  /**
   * Save context snapshot
   */
  async saveSnapshot(snapshot: ContextSnapshot): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query(
        `INSERT INTO context_snapshots (
          snapshot_id, session_id, iteration_number, timestamp,
          full_context, compressed_context, compression_ratio,
          checksum_md5, size_bytes, is_diff, based_on_snapshot_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)`,
        [
          snapshot.snapshotId,
          snapshot.sessionId,
          snapshot.iterationNumber,
          snapshot.timestamp,
          snapshot.fullContext ? JSON.stringify(snapshot.fullContext) : null,
          snapshot.compressedContext,
          snapshot.compressionRatio,
          snapshot.checksumMD5,
          snapshot.sizeBytes,
          snapshot.isDiff,
          snapshot.basedOnSnapshotId,
        ]
      );
    } finally {
      client.release();
    }
  }

  /**
   * Get context snapshot by ID
   */
  async getSnapshot(snapshotId: string): Promise<ContextSnapshot | null> {
    const client = await this.pool.connect();

    try {
      const result = await client.query(
        `SELECT * FROM context_snapshots WHERE snapshot_id = $1`,
        [snapshotId]
      );

      if (result.rows.length === 0) {
        return null;
      }

      return this.mapRowToSnapshot(result.rows[0]);
    } finally {
      client.release();
    }
  }

  /**
   * Execute operations within a transaction
   */
  async transaction<T>(
    callback: (_client: PoolClient) => Promise<T>
  ): Promise<T> {
    const client = await this.pool.connect();

    try {
      await client.query("BEGIN");
      const result = await callback(client);
      await client.query("COMMIT");
      return result;
    } catch (error) {
      await client.query("ROLLBACK");
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Map database row to LearningSession
   */
  private mapRowToSession(row: Record<string, unknown>): LearningSession {
    const config: LearningSessionConfig = {
      maxIterations: row.max_iterations as number,
      progressTimeout: row.progress_timeout as number,
      noProgressLimit: row.no_progress_limit as number,
      resourceBudgetMB: row.resource_budget_mb as number,
      compressionRatio: row.compression_ratio as number,
      qualityThreshold: row.quality_threshold as number,
      enableAdaptivePrompting: row.enable_adaptive_prompting as boolean,
      enableErrorRecognition: row.enable_error_recognition as boolean,
    };

    return {
      sessionId: row.session_id as string,
      taskId: row.task_id as string,
      agentId: row.agent_id as string,
      status: row.status as LearningSessionStatus,
      config,
      startTime: row.start_time as Date,
      endTime: row.end_time ? (row.end_time as Date) : undefined,
      iterationCount: row.iteration_count as number,
      qualityScore: row.quality_score as number,
      improvementTrajectory: row.improvement_trajectory
        ? JSON.parse(row.improvement_trajectory as string)
        : [],
      errorPatterns: row.error_patterns
        ? JSON.parse(row.error_patterns as string)
        : [],
      finalResult: row.final_result
        ? JSON.parse(row.final_result as string)
        : undefined,
      learningSummary: row.learning_summary
        ? (JSON.parse(row.learning_summary as string) as LearningSummary)
        : undefined,
    };
  }

  /**
   * Map database row to LearningIteration
   */
  private mapRowToIteration(row: Record<string, unknown>): LearningIteration {
    return {
      iterationId: row.iteration_id as string,
      sessionId: row.session_id as string,
      iterationNumber: row.iteration_number as number,
      startTime: row.start_time as Date,
      endTime: row.end_time as Date | undefined,
      durationMs: row.duration_ms as number,
      contextSnapshotId: row.context_snapshot_id as string,
      errorDetected: row.error_detected as boolean,
      errorCategory: row.error_category as ErrorCategory | undefined,
      qualityScore: row.quality_score as number,
      improvementDelta: row.improvement_delta as number,
      resourceUsageMB: row.resource_usage_mb as number,
      promptModifications: row.prompt_modifications
        ? JSON.parse(row.prompt_modifications as string)
        : [],
      feedback: row.feedback ? JSON.parse(row.feedback as string) : undefined,
    };
  }

  /**
   * Map database row to ErrorPattern
   */
  private mapRowToErrorPattern(row: Record<string, unknown>): ErrorPattern {
    return {
      patternId: row.pattern_id as string,
      category: row.category as ErrorCategory,
      pattern: row.pattern as string,
      frequency: row.frequency as number,
      confidence: row.confidence as number,
      detectedAt: row.detected_at as Date,
      remediationStrategy: row.remediation_strategy as string,
      successRate: row.success_rate as number,
      examples: row.examples ? JSON.parse(row.examples as string) : [],
    };
  }

  /**
   * Map database row to ContextSnapshot
   */
  private mapRowToSnapshot(row: Record<string, unknown>): ContextSnapshot {
    return {
      snapshotId: row.snapshot_id as string,
      sessionId: row.session_id as string,
      iterationNumber: row.iteration_number as number,
      timestamp: row.timestamp as Date,
      fullContext: row.full_context
        ? JSON.parse(row.full_context as string)
        : undefined,
      compressedContext: row.compressed_context as string,
      compressionRatio: row.compression_ratio as number,
      checksumMD5: row.checksum_md5 as string,
      sizeBytes: row.size_bytes as number,
      isDiff: row.is_diff as boolean,
      basedOnSnapshotId: row.based_on_snapshot_id as string | undefined,
    };
  }
}
