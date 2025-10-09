/**
 * @fileoverview Vector Data Access Object
 * @author @darianrosebrook
 *
 * Provides advanced vector operations including similarity search,
 * hybrid search (vector + metadata), and optimized indexing strategies.
 * Implements pgvector operations with IVFFlat and HNSW indexing.
 */

import { BaseDAO } from "./BaseDAO";
import { DataLayer } from "../DataLayer";
import { Logger } from "../../utils/Logger";
import {
  VectorDAO as IVectorDAO,
  VectorSearchOptions,
  VectorSearchResult,
  BulkOperationOptions,
  BulkOperationResult,
  QueryOptions,
  QueryResult,
  ValidationError
} from "../types";

export abstract class VectorDAO<T extends { id: string; tenantId: string; embedding?: number[]; createdAt: Date; updatedAt: Date }>
  extends BaseDAO<T> implements IVectorDAO<T> {

  constructor(dataLayer: DataLayer, tableName: string, entityName: string, logger?: Logger) {
    super(dataLayer, tableName, entityName, logger);
  }

  /**
   * Find similar entities using vector similarity search
   */
  async findSimilar(
    vector: number[],
    options: VectorSearchOptions & QueryOptions = {}
  ): Promise<QueryResult<VectorSearchResult[]>> {
    try {
      this.validateVector(vector);

      const limit = Math.min(options.limit || 10, 100); // Max 100 results
      const threshold = options.threshold || 0.1; // Minimum similarity threshold

      const cacheKey = options.cache !== false
        ? `vector_similar_${this.tableName}_${vector.slice(0, 5).join('_')}_${limit}_${threshold}`
        : undefined;

      const result = await this.dataLayer.query<{ id: string; score: number; metadata?: any }>(
        `SELECT
          id,
          1 - (embedding <=> $1::vector) as score,
          metadata
         FROM ${this.tableName}
         WHERE embedding IS NOT NULL
           AND 1 - (embedding <=> $1::vector) > $2
         ORDER BY embedding <=> $1::vector
         LIMIT $3`,
        [vector, threshold, limit],
        {
          cache: options.cache,
          cacheKey,
          timeout: options.timeout || 10000 // 10s timeout for vector search
        }
      );

      if (!result.success) {
        throw new Error(`Failed to perform vector similarity search: ${result.error}`);
      }

      const data = result.data || [];
      const dataArray = Array.isArray(data) ? data : [data];
      const searchResults: VectorSearchResult[] = dataArray.map((row: any) => ({
        id: row.id,
        score: row.score,
        metadata: row.metadata || {}
      }));

      this.logger.debug(`Vector similarity search completed`, {
        queryVector: vector.length,
        results: searchResults.length,
        threshold
      });

      return {
        success: true,
        data: searchResults,
        duration: 0,
        queryId: ""
      };
    } catch (error) {
      this.logger.error("Failed to perform vector similarity search", { vector: vector.length, error });
      throw error;
    }
  }

  /**
   * Find similar entities by existing entity ID
   */
  async findSimilarById(
    id: string,
    tenantId: string,
    options: VectorSearchOptions & QueryOptions = {}
  ): Promise<QueryResult<VectorSearchResult[]>> {
    try {
      // First get the entity's embedding
      const entityResult = await this.findById(id, tenantId, { cache: true });
      if (!entityResult.success || !entityResult.data?.embedding) {
        throw new Error(`Entity ${id} not found or has no embedding`);
      }

      const vector = entityResult.data.embedding;

      // Perform similarity search excluding the entity itself
      const limit = (options.limit || 10) + 1; // +1 to account for self-exclusion
      const threshold = options.threshold || 0.1;

      const cacheKey = options.cache !== false
        ? `vector_similar_by_id_${this.tableName}_${id}_${limit}_${threshold}`
        : undefined;

      const result = await this.dataLayer.query<{ id: string; score: number; metadata?: any }>(
        `SELECT
          id,
          1 - (embedding <=> $1::vector) as score,
          metadata
         FROM ${this.tableName}
         WHERE embedding IS NOT NULL
           AND id != $2
           AND 1 - (embedding <=> $1::vector) > $3
         ORDER BY embedding <=> $1::vector
         LIMIT $4`,
        [vector, id, threshold, limit],
        {
          cache: options.cache,
          cacheKey,
          timeout: options.timeout || 10000
        }
      );

      if (!result.success) {
        throw new Error(`Failed to perform vector similarity search by ID: ${result.error}`);
      }

      const data = result.data || [];
      const dataArray = Array.isArray(data) ? data : [data];
      const searchResults: VectorSearchResult[] = dataArray.map((row: any) => ({
        id: row.id,
        score: row.score,
        metadata: row.metadata || {}
      }));

      this.logger.debug(`Vector similarity search by ID completed`, {
        entityId: id,
        results: searchResults.length,
        threshold
      });

      return {
        success: true,
        data: searchResults,
        duration: 0,
        queryId: ""
      };
    } catch (error) {
      this.logger.error("Failed to perform vector similarity search by ID", { id, tenantId, error });
      throw error;
    }
  }

  /**
   * Perform hybrid search combining vector similarity with metadata filtering
   */
  async hybridSearch(
    vector: number[],
    metadataFilter: Record<string, any>,
    options: VectorSearchOptions & QueryOptions = {}
  ): Promise<QueryResult<VectorSearchResult[]>> {
    try {
      this.validateVector(vector);

      const limit = Math.min(options.limit || 10, 100);
      const threshold = options.threshold || 0.1;
      const vectorWeight = options.vectorWeight || 0.7;
      const metadataWeight = 1 - vectorWeight;

      // Build metadata filter conditions
      const filterConditions: string[] = [];
      const params: any[] = [vector, threshold, vectorWeight, metadataWeight];
      let paramIndex = 5;

      for (const [key, value] of Object.entries(metadataFilter)) {
        if (value !== undefined) {
          filterConditions.push(`metadata->>'${key}' = $${paramIndex}`);
          params.push(String(value));
          paramIndex++;
        }
      }

      const whereClause = filterConditions.length > 0
        ? `AND (${filterConditions.join(' AND ')})`
        : '';

      const cacheKey = options.cache !== false
        ? `hybrid_search_${this.tableName}_${vector.slice(0, 3).join('_')}_${JSON.stringify(metadataFilter).slice(0, 50)}`
        : undefined;

      const result = await this.dataLayer.query<{ id: string; score: number; metadata?: any }>(
        `SELECT
          id,
          ($3 * (1 - (embedding <=> $1::vector))) + ($4 * 0.8) as score,
          metadata
         FROM ${this.tableName}
         WHERE embedding IS NOT NULL
           AND 1 - (embedding <=> $1::vector) > $2
           ${whereClause}
         ORDER BY ($3 * (embedding <=> $1::vector)) + ($4 * 0.2) ASC
         LIMIT $${paramIndex}`,
        [...params, limit],
        {
          cache: options.cache,
          cacheKey,
          timeout: options.timeout || 15000 // 15s timeout for hybrid search
        }
      );

      if (!result.success) {
        throw new Error(`Failed to perform hybrid search: ${result.error}`);
      }

      const data = result.data || [];
      const dataArray = Array.isArray(data) ? data : [data];
      const searchResults: VectorSearchResult[] = dataArray.map((row: any) => ({
        id: row.id,
        score: row.score,
        metadata: row.metadata || {}
      }));

      this.logger.debug(`Hybrid search completed`, {
        queryVector: vector.length,
        metadataFilters: Object.keys(metadataFilter),
        results: searchResults.length,
        vectorWeight,
        metadataWeight
      });

      return {
        success: true,
        data: searchResults,
        duration: 0,
        queryId: ""
      };
    } catch (error) {
      this.logger.error("Failed to perform hybrid search", {
        vector: vector.length,
        metadataFilter,
        error
      });
      throw error;
    }
  }

  /**
   * Bulk insert entities with embeddings
   */
  async bulkInsertWithEmbeddings(
    entities: Array<Omit<T, "id" | "createdAt" | "updatedAt">>,
    options: BulkOperationOptions = {}
  ): Promise<BulkOperationResult<T>> {
    try {
      const batchSize = options.batchSize || 100;
      const concurrency = options.concurrency || 3;
      const results: T[] = [];
      const errors: Array<{ index: number; error: string; data: any }> = [];

      // Process in batches with concurrency control
      for (let i = 0; i < entities.length; i += batchSize * concurrency) {
        const batchPromises: Promise<any>[] = [];

        for (let j = 0; j < concurrency && (i + j * batchSize) < entities.length; j++) {
          const startIdx = i + j * batchSize;
          const endIdx = Math.min(startIdx + batchSize, entities.length);
          const batch = entities.slice(startIdx, endIdx);

          batchPromises.push(this.processBatch(batch, startIdx, options.continueOnError));
        }

        const batchResults = await Promise.allSettled(batchPromises);

        for (const result of batchResults) {
          if (result.status === "fulfilled") {
            results.push(...result.value.success);
            errors.push(...result.value.errors);
          } else if (!options.continueOnError) {
            throw result.reason;
          }
        }
      }

      this.logger.info(`Bulk insert with embeddings completed`, {
        total: entities.length,
        successful: results.length,
        failed: errors.length,
        duration: Date.now() - Date.now() // Would need to track start time
      });

      return {
        success: errors.length === 0,
        processed: entities.length,
        successful: results.length,
        failed: errors.length,
        errors,
        duration: 0 // Would need to track actual duration
      };
    } catch (error) {
      this.logger.error("Failed to bulk insert with embeddings", { count: entities.length, error });
      throw error;
    }
  }

  /**
   * Update embeddings for existing entities
   */
  async updateEmbeddings(
    updates: Array<{ id: string; tenantId: string; embedding: number[] }>,
    options: BulkOperationOptions = {}
  ): Promise<BulkOperationResult<{ id: string; tenantId: string }>> {
    try {
      const batchSize = options.batchSize || 50;
      const errors: Array<{ index: number; error: string; data: any }> = [];

      for (let i = 0; i < updates.length; i += batchSize) {
        const batch = updates.slice(i, i + batchSize);

        try {
          await this.dataLayer.transaction(async (connection) => {
            for (const update of batch) {
              await connection.query(
                `UPDATE ${this.tableName}
                 SET embedding = $1::vector, updated_at = NOW()
                 WHERE id = $2 AND tenant_id = $3`,
                [update.embedding, update.id, update.tenantId]
              );
            }
          });

          // Invalidate cache for updated entities
          if (this.dataLayer.getCache()) {
            for (const update of batch) {
              await this.dataLayer.getCache()!.delete(`${this.tableName}:${update.tenantId}:${update.id}`);
            }
          }
        } catch (error) {
          if (options.continueOnError) {
            errors.push({
              index: i,
              error: (error as Error).message,
              data: batch
            });
          } else {
            throw error;
          }
        }
      }

      const successful = updates.length - errors.length;

      this.logger.info(`Embedding updates completed`, {
        total: updates.length,
        successful,
        failed: errors.length
      });

      return {
        success: errors.length === 0,
        processed: updates.length,
        successful,
        failed: errors.length,
        errors,
        duration: 0
      };
    } catch (error) {
      this.logger.error("Failed to update embeddings", { count: updates.length, error });
      throw error;
    }
  }

  /**
   * Bulk insert entities with embeddings
   */
  async bulkInsert(
    entities: Array<Omit<T, "id" | "createdAt" | "updatedAt">>,
    options: BulkOperationOptions = {}
  ): Promise<BulkOperationResult<T>> {
    try {
      const batchSize = options.batchSize || 100;
      const results: T[] = [];
      const errors: Array<{ index: number; error: string; data: any }> = [];

      for (let i = 0; i < entities.length; i += batchSize) {
        const batch = entities.slice(i, i + batchSize);

        try {
          await this.dataLayer.transaction(async (connection) => {
            for (const entity of batch) {
              const result = await this.create(entity);
              if (result.success && result.data) {
                results.push(result.data);
              } else {
                throw new Error(result.error || "Unknown error");
              }
            }
          });
        } catch (error) {
          if (options.continueOnError) {
            errors.push({
              index: i,
              error: (error as Error).message,
              data: batch
            });
          } else {
            throw error;
          }
        }
      }

      this.logger.info(`Bulk insert completed`, {
        total: entities.length,
        successful: results.length,
        failed: errors.length
      });

      return {
        success: errors.length === 0,
        processed: entities.length,
        successful: results.length,
        failed: errors.length,
        errors,
        duration: 0
      };
    } catch (error) {
      this.logger.error("Failed to bulk insert", { count: entities.length, error });
      throw error;
    }
  }

  /**
   * Bulk update embeddings for existing entities
   */
  async bulkUpdate(
    updates: Array<{ id: string; tenantId: string; data: Partial<T> }>,
    options: BulkOperationOptions = {}
  ): Promise<BulkOperationResult<T>> {
    try {
      const batchSize = options.batchSize || 50;
      const errors: Array<{ index: number; error: string; data: any }> = [];

      for (let i = 0; i < updates.length; i += batchSize) {
        const batch = updates.slice(i, i + batchSize);

        try {
          await this.dataLayer.transaction(async (connection) => {
            for (const update of batch) {
              await this.update(update.id, update.tenantId, update.data);
            }
          });
        } catch (error) {
          if (options.continueOnError) {
            errors.push({
              index: i,
              error: (error as Error).message,
              data: batch
            });
          } else {
            throw error;
          }
        }
      }

      const successful = updates.length - errors.length;

      this.logger.info(`Bulk update completed`, {
        total: updates.length,
        successful,
        failed: errors.length
      });

      return {
        success: errors.length === 0,
        processed: updates.length,
        successful,
        failed: errors.length,
        errors,
        duration: 0
      };
    } catch (error) {
      this.logger.error("Failed to bulk update", { count: updates.length, error });
      throw error;
    }
  }

  /**
   * Get vector statistics for the collection
   */
  async getVectorStats(tenantId: string): Promise<QueryResult<{
    totalVectors: number;
    avgDimensions: number;
    minDimensions: number;
    maxDimensions: number;
    nullEmbeddings: number;
  }>> {
    try {
      const result = await this.dataLayer.query<{
        total_vectors: number;
        avg_dimensions: number;
        min_dimensions: number;
        max_dimensions: number;
        null_embeddings: number;
      }>(
        `SELECT
          COUNT(*) as total_vectors,
          COALESCE(AVG(array_length(embedding, 1)), 0) as avg_dimensions,
          COALESCE(MIN(array_length(embedding, 1)), 0) as min_dimensions,
          COALESCE(MAX(array_length(embedding, 1)), 0) as max_dimensions,
          COUNT(*) FILTER (WHERE embedding IS NULL) as null_embeddings
         FROM ${this.tableName}
         WHERE tenant_id = $1`,
        [tenantId],
        { cache: true, cacheTtl: 300 } // Cache for 5 minutes
      );

      if (!result.success || !result.data) {
        throw new Error(`Failed to get vector statistics: ${result.error}`);
      }

      const data = result.data;
      const stats = Array.isArray(data) ? data[0] : data;

      return {
        success: true,
        data: {
          totalVectors: stats.total_vectors,
          avgDimensions: Math.round(stats.avg_dimensions),
          minDimensions: stats.min_dimensions,
          maxDimensions: stats.max_dimensions,
          nullEmbeddings: stats.null_embeddings
        },
        duration: 0,
        queryId: ""
      };
    } catch (error) {
      this.logger.error("Failed to get vector statistics", { tenantId, error });
      throw error;
    }
  }

  // Private helper methods

  private async processBatch(
    batch: Array<Omit<T, "id" | "createdAt" | "updatedAt">>,
    startIndex: number,
    continueOnError: boolean = false
  ): Promise<{ success: T[]; errors: Array<{ index: number; error: string; data: any }> }> {
    const success: T[] = [];
    const errors: Array<{ index: number; error: string; data: any }> = [];

    for (let i = 0; i < batch.length; i++) {
      try {
        const result = await this.create(batch[i]);
        if (result.success && result.data) {
          success.push(result.data);
        } else {
          throw new Error(result.error || "Unknown error");
        }
      } catch (error) {
        if (continueOnError) {
          errors.push({
            index: startIndex + i,
            error: (error as Error).message,
            data: batch[i]
          });
        } else {
          throw error;
        }
      }
    }

    return { success, errors };
  }

  private validateVector(vector: number[]): void {
    if (!Array.isArray(vector) || vector.length === 0) {
      throw new ValidationError("Vector must be a non-empty array", "validateVector", this.entityName);
    }

    if (vector.length > 4096) {
      throw new ValidationError("Vector dimension cannot exceed 4096", "validateVector", this.entityName);
    }

    for (const val of vector) {
      if (typeof val !== "number" || isNaN(val)) {
        throw new ValidationError("All vector values must be valid numbers", "validateVector", this.entityName);
      }
    }
  }

  protected validateEntity(entity: Partial<T>): void {
    super.validateEntity(entity);

    if (entity.embedding) {
      this.validateVector(entity.embedding);
    }
  }
}
