/**
 * @fileoverview Experience Data Access Object
 * @author @darianrosebrook
 *
 * Specialized DAO for agent experiences with vector similarity search,
 * relevance scoring, and temporal decay calculations.
 * Extends VectorDAO with experience-specific operations.
 */

import { Logger } from "../../utils/Logger";
import { DataLayer } from "../DataLayer";
import {
  ExperienceEntity,
  QueryOptions,
  QueryResult,
  VectorSearchOptions,
} from "../types";
import { VectorDAO } from "./VectorDAO";

export class ExperienceDAO extends VectorDAO<ExperienceEntity> {
  constructor(dataLayer: DataLayer, logger?: Logger) {
    super(dataLayer, "experiences", "Experience", logger);
  }

  /**
   * Find experiences by agent with relevance decay
   */
  async findByAgentWithDecay(
    agentId: string,
    tenantId: string,
    maxAgeHours: number = 168, // 7 days
    options: QueryOptions = {}
  ): Promise<QueryResult<ExperienceEntity[]>> {
    try {
      const result = await this.dataLayer.query<ExperienceEntity[]>(
        `SELECT *,
          relevance_score * POWER(0.99, EXTRACT(EPOCH FROM (NOW() - created_at)) / 3600) as decayed_relevance
         FROM experiences
         WHERE tenant_id = $1
           AND agent_id = $2
           AND created_at > NOW() - INTERVAL '${maxAgeHours} hours'
         ORDER BY decayed_relevance DESC, created_at DESC`,
        [tenantId, agentId],
        {
          cache: options.cache,
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new Error(`Failed to find experiences by agent: ${result.error}`);
      }

      const experiences = (result.data || []).map((row: any) =>
        this.mapRowToEntity(row)
      );

      this.logger.debug(`Found experiences by agent with decay`, {
        agentId,
        tenantId,
        count: experiences.length,
        maxAgeHours,
      });

      return {
        success: true,
        data: experiences,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error("Failed to find experiences by agent", {
        agentId,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Find similar experiences by task type and outcome
   */
  async findSimilarByTaskType(
    taskType: string,
    outcome: "success" | "failure" | "partial",
    tenantId: string,
    embedding?: number[],
    options: VectorSearchOptions & QueryOptions = {}
  ): Promise<QueryResult<ExperienceEntity[]>> {
    try {
      let query = `
        SELECT * FROM experiences
        WHERE tenant_id = $1
          AND type = $2
          AND outcome = $3
      `;
      const params: any[] = [tenantId, taskType, outcome];
      let paramIndex = 4;

      // Add vector similarity if embedding provided
      if (embedding) {
        query += ` AND embedding IS NOT NULL
                   ORDER BY embedding <=> $${paramIndex}::vector`;
        params.push(embedding);
        paramIndex++;
      } else {
        query += ` ORDER BY relevance_score DESC, created_at DESC`;
      }

      query += ` LIMIT $${paramIndex}`;
      params.push(options.limit || 20);

      const result = await this.dataLayer.query<ExperienceEntity[]>(
        query,
        params,
        {
          cache: options.cache,
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new Error(
          `Failed to find similar experiences by task type: ${result.error}`
        );
      }

      const experiences = (result.data || []).map((row: any) =>
        this.mapRowToEntity(row)
      );

      this.logger.debug(`Found similar experiences by task type`, {
        taskType,
        outcome,
        tenantId,
        count: experiences.length,
        hasEmbedding: !!embedding,
      });

      return {
        success: true,
        data: experiences,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error("Failed to find similar experiences by task type", {
        taskType,
        outcome,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Get experience insights aggregated by agent and task type
   */
  async getInsightsByAgent(
    tenantId: string,
    options: { minSamples?: number; maxAgeHours?: number } & QueryOptions = {}
  ): Promise<
    QueryResult<
      Array<{
        agentId: string;
        taskType: string;
        totalExperiences: number;
        successRate: number;
        avgRelevance: number;
        avgDuration: number;
      }>
    >
  > {
    try {
      const minSamples = options.minSamples || 5;
      const maxAgeHours = options.maxAgeHours || 168; // 7 days

      const result = await this.dataLayer.query<
        {
          agent_id: string;
          type: string;
          total_experiences: number;
          success_rate: number;
          avg_relevance: number;
          avg_duration: number;
        }[]
      >(
        `SELECT
          agent_id,
          type,
          COUNT(*) as total_experiences,
          AVG(CASE WHEN outcome = 'success' THEN 1 ELSE 0 END) as success_rate,
          AVG(relevance_score) as avg_relevance,
          AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_duration
         FROM experiences
         WHERE tenant_id = $1
           AND created_at > NOW() - INTERVAL '${maxAgeHours} hours'
         GROUP BY agent_id, type
         HAVING COUNT(*) >= $2
         ORDER BY total_experiences DESC, success_rate DESC`,
        [tenantId, minSamples],
        {
          cache: options.cache,
          cacheTtl: 600, // Cache for 10 minutes
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new Error(`Failed to get experience insights: ${result.error}`);
      }

      const insights = (result.data || []).map((row: any) => ({
        agentId: row.agent_id,
        taskType: row.type,
        totalExperiences: row.total_experiences,
        successRate: row.success_rate,
        avgRelevance: row.avg_relevance,
        avgDuration: row.avg_duration,
      }));

      this.logger.debug(`Generated experience insights`, {
        tenantId,
        insights: insights.length,
        minSamples,
        maxAgeHours,
      });

      return {
        success: true,
        data: insights,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error("Failed to get experience insights", {
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Clean up old experiences based on retention policy
   */
  async cleanupOldExperiences(
    tenantId: string,
    maxAgeHours: number = 720, // 30 days
    minRelevance: number = 0.3
  ): Promise<QueryResult<{ deleted: number }>> {
    try {
      const result = await this.dataLayer.query<{ id: string }[]>(
        `DELETE FROM experiences
         WHERE tenant_id = $1
           AND (created_at < NOW() - INTERVAL '${maxAgeHours} hours'
                OR relevance_score < $2)
         RETURNING id`,
        [tenantId, minRelevance]
      );

      if (!result.success) {
        throw new Error(`Failed to cleanup old experiences: ${result.error}`);
      }

      const deleted = result.data?.length || 0;

      this.logger.info(`Cleaned up old experiences`, {
        tenantId,
        deleted,
        maxAgeHours,
        minRelevance,
      });

      return {
        success: true,
        data: { deleted },
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error("Failed to cleanup old experiences", {
        tenantId,
        error,
      });
      throw error;
    }
  }

  // Implementation of abstract methods from VectorDAO/BaseDAO

  protected getColumns(): string[] {
    return [
      "agent_id",
      "task_id",
      "type",
      "content",
      "outcome",
      "relevance_score",
      "context_match",
      "reasoning_path",
      "temporal_relevance",
      "weight",
      "embedding",
      "metadata",
    ];
  }

  protected getValues(
    entity: Omit<ExperienceEntity, "id" | "createdAt" | "updatedAt">
  ): any[] {
    return [
      entity.agentId,
      entity.taskId,
      entity.type,
      JSON.stringify(entity.content),
      entity.outcome,
      entity.relevanceScore,
      JSON.stringify(entity.contextMatch),
      entity.reasoningPath ? JSON.stringify(entity.reasoningPath) : null,
      JSON.stringify(entity.temporalRelevance),
      entity.weight,
      entity.embedding,
      JSON.stringify(entity.metadata),
    ];
  }

  protected mapRowToEntity(row: any): ExperienceEntity {
    return {
      id: row.id,
      tenantId: row.tenant_id,
      agentId: row.agent_id,
      taskId: row.task_id,
      type: row.type,
      content:
        typeof row.content === "string" ? JSON.parse(row.content) : row.content,
      outcome: row.outcome,
      relevanceScore: row.relevance_score,
      contextMatch:
        typeof row.context_match === "string"
          ? JSON.parse(row.context_match)
          : row.context_match,
      reasoningPath: row.reasoning_path
        ? typeof row.reasoning_path === "string"
          ? JSON.parse(row.reasoning_path)
          : row.reasoning_path
        : undefined,
      temporalRelevance:
        typeof row.temporal_relevance === "string"
          ? JSON.parse(row.temporal_relevance)
          : row.temporal_relevance,
      weight: row.weight,
      embedding: row.embedding,
      metadata:
        typeof row.metadata === "string"
          ? JSON.parse(row.metadata)
          : row.metadata,
      createdAt: new Date(row.created_at),
      updatedAt: new Date(row.updated_at),
    };
  }

  protected mapFieldToColumn(field: string): string {
    const fieldMap: Record<string, string> = {
      tenantId: "tenant_id",
      agentId: "agent_id",
      taskId: "task_id",
      createdAt: "created_at",
      updatedAt: "updated_at",
      relevanceScore: "relevance_score",
      contextMatch: "context_match",
      reasoningPath: "reasoning_path",
      temporalRelevance: "temporal_relevance",
    };

    return fieldMap[field] || field;
  }
}
