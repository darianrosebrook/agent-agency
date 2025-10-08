/**
 * @fileoverview Base DAO Implementation
 * @author @darianrosebrook
 *
 * Provides common database operations and patterns for all DAOs.
 * Implements connection management, error handling, and caching integration.
 */

import { Logger } from "../../utils/Logger";
import { DataLayer } from "../DataLayer";
import {
  DataLayerError,
  BaseDAO as IBaseDAO,
  NotFoundError,
  QueryOptions,
  QueryResult,
  ValidationError,
} from "../types";

export abstract class BaseDAO<
  T extends { id: string; tenantId: string; createdAt: Date; updatedAt: Date }
> implements IBaseDAO<T>
{
  protected dataLayer: DataLayer;
  protected logger: Logger;
  protected tableName: string;
  protected entityName: string;

  constructor(
    dataLayer: DataLayer,
    tableName: string,
    entityName: string,
    logger?: Logger
  ) {
    this.dataLayer = dataLayer;
    this.tableName = tableName;
    this.entityName = entityName;
    this.logger = logger || new Logger(`${entityName}DAO`);
  }

  /**
   * Create a new entity
   */
  async create(
    entity: Omit<T, "id" | "createdAt" | "updatedAt">,
    options: QueryOptions = {}
  ): Promise<QueryResult<T>> {
    try {
      this.validateEntity(entity);

      const now = new Date();
      const id = this.generateId();

      const columns = [
        "id",
        "tenant_id",
        ...this.getColumns(),
        "created_at",
        "updated_at",
      ];
      const values = [id, entity.tenantId, ...this.getValues(entity), now, now];
      const placeholders = values.map((_, i) => `$${i + 1}`);

      const query = `
        INSERT INTO ${this.tableName} (${columns.join(", ")})
        VALUES (${placeholders.join(", ")})
        RETURNING *
      `;

      const result = await this.dataLayer.query<T>(query, values, {
        cache: false, // Don't cache inserts
        timeout: options.timeout,
      });

      if (!result.success || !result.data || result.data.length === 0) {
        throw new DataLayerError(
          `Failed to create ${this.entityName}`,
          "CREATE_ERROR",
          "create",
          this.entityName
        );
      }

      const createdEntity = this.mapRowToEntity(result.data[0]);

      this.logger.debug(`${this.entityName} created`, {
        id,
        tenantId: entity.tenantId,
      });

      return {
        success: true,
        data: createdEntity,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error(`Failed to create ${this.entityName}`, error);
      throw error;
    }
  }

  /**
   * Find entity by ID and tenant
   */
  async findById(
    id: string,
    tenantId: string,
    options: QueryOptions = {}
  ): Promise<QueryResult<T>> {
    try {
      const cacheKey =
        options.cache !== false
          ? `${this.tableName}:${tenantId}:${id}`
          : undefined;

      const result = await this.dataLayer.query<T>(
        `SELECT * FROM ${this.tableName} WHERE id = $1 AND tenant_id = $2`,
        [id, tenantId],
        {
          cache: options.cache,
          cacheKey,
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new DataLayerError(
          `Failed to find ${this.entityName}`,
          "FIND_ERROR",
          "findById",
          this.entityName
        );
      }

      if (!result.data || result.data.length === 0) {
        throw new NotFoundError(
          `${this.entityName} not found: ${id}`,
          "findById",
          this.entityName
        );
      }

      const entity = this.mapRowToEntity(result.data[0]);

      return {
        success: true,
        data: entity,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error(`Failed to find ${this.entityName} by ID`, {
        id,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Find multiple entities with filtering
   */
  async findMany(
    filter: Partial<T> = {},
    options: QueryOptions = {}
  ): Promise<QueryResult<T[]>> {
    try {
      let query = `SELECT * FROM ${this.tableName} WHERE 1=1`;
      const params: any[] = [];
      let paramIndex = 1;

      // Build WHERE clause from filter
      for (const [key, value] of Object.entries(filter)) {
        if (value !== undefined) {
          const columnName = this.mapFieldToColumn(key);
          query += ` AND ${columnName} = $${paramIndex}`;
          params.push(value);
          paramIndex++;
        }
      }

      // Add ordering (optional - can be made configurable)
      query += ` ORDER BY created_at DESC`;

      const result = await this.dataLayer.query<T[]>(query, params, {
        cache: options.cache,
        timeout: options.timeout,
      });

      if (!result.success) {
        throw new DataLayerError(
          `Failed to find ${this.entityName} entities`,
          "FIND_MANY_ERROR",
          "findMany",
          this.entityName
        );
      }

      const entities = (result.data || []).map((row) =>
        this.mapRowToEntity(row)
      );

      return {
        success: true,
        data: entities,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error(`Failed to find ${this.entityName} entities`, {
        filter,
        error,
      });
      throw error;
    }
  }

  /**
   * Update an entity
   */
  async update(
    id: string,
    tenantId: string,
    updates: Partial<Omit<T, "id" | "tenantId" | "createdAt">>,
    options: QueryOptions = {}
  ): Promise<QueryResult<T>> {
    try {
      this.validateUpdates(updates);

      const now = new Date();
      const setParts: string[] = [];
      const params: any[] = [];
      let paramIndex = 1;

      // Build SET clause
      for (const [key, value] of Object.entries(updates)) {
        if (key !== "createdAt" && value !== undefined) {
          const columnName = this.mapFieldToColumn(key);
          setParts.push(`${columnName} = $${paramIndex}`);
          params.push(value);
          paramIndex++;
        }
      }

      // Always update updated_at
      setParts.push(`updated_at = $${paramIndex}`);
      params.push(now);
      paramIndex++;

      // Add WHERE conditions
      params.push(id, tenantId);

      const query = `
        UPDATE ${this.tableName}
        SET ${setParts.join(", ")}
        WHERE id = $${paramIndex - 1} AND tenant_id = $${paramIndex}
        RETURNING *
      `;

      const result = await this.dataLayer.query<T>(query, params, {
        cache: false, // Invalidate cache for updates
        timeout: options.timeout,
      });

      if (!result.success || !result.data || result.data.length === 0) {
        throw new NotFoundError(
          `${this.entityName} not found for update: ${id}`,
          "update",
          this.entityName
        );
      }

      const updatedEntity = this.mapRowToEntity(result.data[0]);

      // Invalidate cache
      if (this.dataLayer.getCache()) {
        await this.dataLayer
          .getCache()!
          .delete(`${this.tableName}:${tenantId}:${id}`);
      }

      this.logger.debug(`${this.entityName} updated`, { id, tenantId });

      return {
        success: true,
        data: updatedEntity,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error(`Failed to update ${this.entityName}`, {
        id,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Delete an entity
   */
  async delete(
    id: string,
    tenantId: string,
    options: QueryOptions = {}
  ): Promise<QueryResult<boolean>> {
    try {
      const result = await this.dataLayer.query(
        `DELETE FROM ${this.tableName} WHERE id = $1 AND tenant_id = $2`,
        [id, tenantId],
        {
          cache: false,
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new DataLayerError(
          `Failed to delete ${this.entityName}`,
          "DELETE_ERROR",
          "delete",
          this.entityName
        );
      }

      const deleted = (result.data as any)?.rowCount > 0;

      if (!deleted) {
        throw new NotFoundError(
          `${this.entityName} not found for deletion: ${id}`,
          "delete",
          this.entityName
        );
      }

      // Invalidate cache
      if (this.dataLayer.getCache()) {
        await this.dataLayer
          .getCache()!
          .delete(`${this.tableName}:${tenantId}:${id}`);
      }

      this.logger.debug(`${this.entityName} deleted`, { id, tenantId });

      return {
        success: true,
        data: true,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error(`Failed to delete ${this.entityName}`, {
        id,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Check if entity exists
   */
  async exists(
    id: string,
    tenantId: string,
    options: QueryOptions = {}
  ): Promise<QueryResult<boolean>> {
    try {
      const result = await this.dataLayer.query<{ exists: boolean }>(
        `SELECT EXISTS(SELECT 1 FROM ${this.tableName} WHERE id = $1 AND tenant_id = $2) as exists`,
        [id, tenantId],
        {
          cache: options.cache,
          cacheKey: options.cache
            ? `${this.tableName}:${tenantId}:${id}:exists`
            : undefined,
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new DataLayerError(
          `Failed to check ${this.entityName} existence`,
          "EXISTS_ERROR",
          "exists",
          this.entityName
        );
      }

      return {
        success: true,
        data: result.data?.[0]?.exists || false,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error(`Failed to check ${this.entityName} existence`, {
        id,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Count entities matching filter
   */
  async count(
    filter: Partial<T> = {},
    options: QueryOptions = {}
  ): Promise<QueryResult<number>> {
    try {
      let query = `SELECT COUNT(*) as count FROM ${this.tableName} WHERE 1=1`;
      const params: any[] = [];
      let paramIndex = 1;

      // Build WHERE clause from filter
      for (const [key, value] of Object.entries(filter)) {
        if (value !== undefined) {
          const columnName = this.mapFieldToColumn(key);
          query += ` AND ${columnName} = $${paramIndex}`;
          params.push(value);
          paramIndex++;
        }
      }

      const result = await this.dataLayer.query<{ count: number }>(
        query,
        params,
        {
          cache: options.cache,
          timeout: options.timeout,
        }
      );

      if (!result.success || !result.data || result.data.length === 0) {
        throw new DataLayerError(
          `Failed to count ${this.entityName} entities`,
          "COUNT_ERROR",
          "count",
          this.entityName
        );
      }

      return {
        success: true,
        data: parseInt(result.data[0].count.toString()),
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error(`Failed to count ${this.entityName} entities`, {
        filter,
        error,
      });
      throw error;
    }
  }

  // Abstract methods to be implemented by subclasses

  /**
   * Get database column names for the entity (excluding id, tenant_id, created_at, updated_at)
   */
  protected abstract getColumns(): string[];

  /**
   * Get values for database insertion (in same order as getColumns)
   */
  protected abstract getValues(
    entity: Omit<T, "id" | "createdAt" | "updatedAt">
  ): any[];

  /**
   * Map database row to entity object
   */
  protected abstract mapRowToEntity(row: any): T;

  /**
   * Map entity field name to database column name
   */
  protected abstract mapFieldToColumn(field: string): string;

  // Utility methods

  /**
   * Validate entity before creation/update
   */
  protected validateEntity(entity: Partial<T>): void {
    if (!entity.tenantId) {
      throw new ValidationError(
        "tenantId is required",
        "validateEntity",
        this.entityName
      );
    }
  }

  /**
   * Validate updates before applying
   */
  protected validateUpdates(updates: Partial<T>): void {
    // Default implementation - can be overridden
  }

  /**
   * Generate a unique ID for new entities
   */
  protected generateId(): string {
    return `id_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}
