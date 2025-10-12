/**
 * @fileoverview Research Provenance Tracker for ARBITER-006 Phase 4
 *
 * Tracks research provenance for audit trails and quality analysis.
 * Records when research was performed, what queries were used, and results.
 *
 * @author @darianrosebrook
 */

import { IDatabaseClient } from "../DatabaseClient";
import { ResearchContext } from "./TaskResearchAugmenter";

/**
 * Research provenance record
 */
export interface ResearchProvenanceRecord {
  /** Task ID that research was performed for */
  taskId: string;

  /** Queries executed */
  queries: string[];

  /** Number of findings */
  findingsCount: number;

  /** Overall confidence */
  confidence: number;

  /** When research was performed */
  performedAt: Date;

  /** How long research took (ms) */
  durationMs?: number;

  /** Whether research was successful */
  successful: boolean;

  /** Error message if failed */
  error?: string;
}

/**
 * Research statistics
 */
export interface ResearchStatistics {
  /** Total research operations */
  totalResearch: number;

  /** Successful research operations */
  successfulResearch: number;

  /** Failed research operations */
  failedResearch: number;

  /** Average confidence */
  averageConfidence: number;

  /** Average duration (ms) */
  averageDurationMs: number;

  /** Most common query types */
  topQueryTypes: Array<{ type: string; count: number }>;
}

/**
 * Research Provenance Tracker
 *
 * Maintains audit trail of all research performed for tasks.
 * Provides analytics and statistics on research effectiveness.
 */
export class ResearchProvenance {
  private dbClient: IDatabaseClient | undefined;

  constructor(dbClient?: IDatabaseClient) {
    this.dbClient = dbClient;
  }

  /**
   * Record research provenance
   */
  async recordResearch(
    taskId: string,
    researchContext: ResearchContext,
    durationMs?: number
  ): Promise<void> {
    if (!this.dbClient || !this.dbClient.isConnected()) {
      console.warn("Database not available, research provenance not recorded");
      return;
    }

    const record: ResearchProvenanceRecord = {
      taskId,
      queries: researchContext.queries,
      findingsCount: researchContext.findings.length,
      confidence: researchContext.confidence,
      performedAt: researchContext.augmentedAt,
      durationMs,
      successful: true,
    };

    try {
      await this.dbClient.query(
        `
          INSERT INTO task_research_provenance (
            task_id,
            queries,
            findings_count,
            confidence,
            performed_at,
            duration_ms,
            successful
          ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        `,
        [
          record.taskId,
          JSON.stringify(record.queries),
          record.findingsCount,
          record.confidence,
          record.performedAt.toISOString(),
          record.durationMs || null,
          record.successful,
        ]
      );

      console.log(
        `Research provenance recorded for task ${taskId}: ${record.findingsCount} findings`
      );
    } catch (error) {
      console.error("Failed to record research provenance:", error);
    }
  }

  /**
   * Record failed research attempt
   */
  async recordFailure(
    taskId: string,
    queries: string[],
    error: Error,
    durationMs?: number
  ): Promise<void> {
    if (!this.dbClient || !this.dbClient.isConnected()) {
      return;
    }

    try {
      await this.dbClient.query(
        `
          INSERT INTO task_research_provenance (
            task_id,
            queries,
            findings_count,
            confidence,
            performed_at,
            duration_ms,
            successful,
            error
          ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        `,
        [
          taskId,
          JSON.stringify(queries),
          0,
          0,
          new Date().toISOString(),
          durationMs || null,
          false,
          error.message,
        ]
      );
    } catch (err) {
      console.error("Failed to record research failure:", err);
    }
  }

  /**
   * Get research history for a task
   */
  async getTaskResearch(taskId: string): Promise<ResearchProvenanceRecord[]> {
    if (!this.dbClient || !this.dbClient.isConnected()) {
      return [];
    }

    try {
      const result = await this.dbClient.query(
        `
          SELECT 
            task_id,
            queries,
            findings_count,
            confidence,
            performed_at,
            duration_ms,
            successful,
            error
          FROM task_research_provenance
          WHERE task_id = $1
          ORDER BY performed_at DESC
        `,
        [taskId]
      );

      return result.rows.map((row: any) => ({
        taskId: row.task_id,
        queries: JSON.parse(row.queries),
        findingsCount: row.findings_count,
        confidence: row.confidence,
        performedAt: new Date(row.performed_at),
        durationMs: row.duration_ms,
        successful: row.successful,
        error: row.error,
      }));
    } catch (error) {
      console.error("Failed to get task research:", error);
      return [];
    }
  }

  /**
   * Get research statistics
   */
  async getStatistics(
    startDate?: Date,
    endDate?: Date
  ): Promise<ResearchStatistics> {
    if (!this.dbClient || !this.dbClient.isConnected()) {
      return {
        totalResearch: 0,
        successfulResearch: 0,
        failedResearch: 0,
        averageConfidence: 0,
        averageDurationMs: 0,
        topQueryTypes: [],
      };
    }

    try {
      // Build date filter
      let dateFilter = "";
      const params: any[] = [];
      if (startDate) {
        dateFilter += " AND performed_at >= $1";
        params.push(startDate.toISOString());
      }
      if (endDate) {
        dateFilter += ` AND performed_at <= $${params.length + 1}`;
        params.push(endDate.toISOString());
      }

      // Get aggregate statistics
      const statsResult = await this.dbClient.query(
        `
          SELECT 
            COUNT(*) as total,
            COUNT(CASE WHEN successful THEN 1 END) as successful,
            COUNT(CASE WHEN NOT successful THEN 1 END) as failed,
            AVG(CASE WHEN successful THEN confidence END) as avg_confidence,
            AVG(CASE WHEN successful THEN duration_ms END) as avg_duration
          FROM task_research_provenance
          WHERE 1=1 ${dateFilter}
        `,
        params
      );

      const stats = statsResult.rows[0] || {};

      return {
        totalResearch: parseInt(stats.total) || 0,
        successfulResearch: parseInt(stats.successful) || 0,
        failedResearch: parseInt(stats.failed) || 0,
        averageConfidence: parseFloat(stats.avg_confidence) || 0,
        averageDurationMs: parseFloat(stats.avg_duration) || 0,
        topQueryTypes: [], // TODO: Extract query types from queries JSON
      };
    } catch (error) {
      console.error("Failed to get research statistics:", error);
      return {
        totalResearch: 0,
        successfulResearch: 0,
        failedResearch: 0,
        averageConfidence: 0,
        averageDurationMs: 0,
        topQueryTypes: [],
      };
    }
  }

  /**
   * Clean up old provenance records
   */
  async cleanupOldRecords(olderThanDays: number = 90): Promise<number> {
    if (!this.dbClient || !this.dbClient.isConnected()) {
      return 0;
    }

    try {
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

      const result = await this.dbClient.query(
        `
          DELETE FROM task_research_provenance
          WHERE performed_at < $1
        `,
        [cutoffDate.toISOString()]
      );

      const deletedCount = result.rowCount || 0;
      console.log(
        `Cleaned up ${deletedCount} research provenance records older than ${olderThanDays} days`
      );
      return deletedCount;
    } catch (error) {
      console.error("Failed to cleanup old research records:", error);
      return 0;
    }
  }
}
