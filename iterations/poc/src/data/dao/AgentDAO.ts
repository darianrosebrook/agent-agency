/**
 * @fileoverview Agent Data Access Object
 * @author @darianrosebrook
 *
 * Implements database operations for Agent entities with caching and validation.
 * Extends BaseDAO with Agent-specific functionality.
 */

import { Logger } from "../../utils/Logger";
import { DataLayer } from "../DataLayer";
import {
  AgentEntity,
  QueryOptions,
  QueryResult,
  ValidationError,
} from "../types";
import { BaseDAO } from "./BaseDAO";

export class AgentDAO extends BaseDAO<AgentEntity> {
  constructor(dataLayer: DataLayer, logger?: Logger) {
    super(dataLayer, "agents", "Agent", logger);
  }

  /**
   * Find agents by capabilities (partial match)
   */
  async findByCapabilities(
    capabilities: string[],
    tenantId: string,
    options: QueryOptions = {}
  ): Promise<QueryResult<AgentEntity[]>> {
    try {
      const placeholders = capabilities.map((_, i) => `$${i + 2}`).join(",");
      const params = [tenantId, ...capabilities];

      const result = await this.dataLayer.query<AgentEntity[]>(
        `SELECT * FROM agents WHERE tenant_id = $1 AND capabilities && ARRAY[${placeholders}] ORDER BY created_at DESC`,
        params,
        {
          cache: options.cache,
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new Error(
          `Failed to find agents by capabilities: ${result.error}`
        );
      }

      const agents = (result.data || []).map((row: any) =>
        this.mapRowToEntity(row)
      );

      return {
        success: true,
        data: agents,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error("Failed to find agents by capabilities", {
        capabilities,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Find agents by status
   */
  async findByStatus(
    status: AgentEntity["status"],
    tenantId: string,
    options: QueryOptions = {}
  ): Promise<QueryResult<AgentEntity[]>> {
    try {
      const result = await this.dataLayer.query<AgentEntity[]>(
        `SELECT * FROM agents WHERE tenant_id = $1 AND status = $2 ORDER BY created_at DESC`,
        [tenantId, status],
        {
          cache: options.cache,
          timeout: options.timeout,
        }
      );

      if (!result.success) {
        throw new Error(`Failed to find agents by status: ${result.error}`);
      }

      const agents = (result.data || []).map((row: any) =>
        this.mapRowToEntity(row)
      );

      return {
        success: true,
        data: agents,
        duration: result.duration || 0,
        queryId: result.queryId || "",
      };
    } catch (error) {
      this.logger.error("Failed to find agents by status", {
        status,
        tenantId,
        error,
      });
      throw error;
    }
  }

  /**
   * Update agent status
   */
  async updateStatus(
    id: string,
    tenantId: string,
    status: AgentEntity["status"],
    options: QueryOptions = {}
  ): Promise<QueryResult<AgentEntity>> {
    return this.update(id, tenantId, { status }, options);
  }

  // Implementation of abstract methods

  protected getColumns(): string[] {
    return ["name", "type", "capabilities", "status", "config", "metadata"];
  }

  protected getValues(
    entity: Omit<AgentEntity, "id" | "createdAt" | "updatedAt">
  ): any[] {
    return [
      entity.name,
      entity.type,
      entity.capabilities,
      entity.status,
      JSON.stringify(entity.config),
      JSON.stringify(entity.metadata),
    ];
  }

  protected mapRowToEntity(row: any): AgentEntity {
    return {
      id: row.id,
      tenantId: row.tenant_id,
      name: row.name,
      type: row.type,
      capabilities: row.capabilities || [],
      status: row.status,
      config:
        typeof row.config === "string" ? JSON.parse(row.config) : row.config,
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
      createdAt: "created_at",
      updatedAt: "updated_at",
    };

    return fieldMap[field] || field;
  }

  protected validateEntity(entity: Partial<AgentEntity>): void {
    super.validateEntity(entity);

    if (!entity.name || entity.name.trim().length === 0) {
      throw new ValidationError(
        "Agent name is required and cannot be empty",
        "validateEntity",
        "Agent"
      );
    }

    if (
      !entity.type ||
      !["worker", "coordinator", "specialist"].includes(entity.type)
    ) {
      throw new ValidationError(
        "Agent type must be one of: worker, coordinator, specialist",
        "validateEntity",
        "Agent"
      );
    }

    if (
      !entity.capabilities ||
      !Array.isArray(entity.capabilities) ||
      entity.capabilities.length === 0
    ) {
      throw new ValidationError(
        "Agent must have at least one capability",
        "validateEntity",
        "Agent"
      );
    }

    if (
      !entity.status ||
      !["active", "inactive", "maintenance"].includes(entity.status)
    ) {
      throw new ValidationError(
        "Agent status must be one of: active, inactive, maintenance",
        "validateEntity",
        "Agent"
      );
    }
  }

  protected validateUpdates(updates: Partial<AgentEntity>): void {
    if (
      updates.type &&
      !["worker", "coordinator", "specialist"].includes(updates.type)
    ) {
      throw new ValidationError(
        "Agent type must be one of: worker, coordinator, specialist",
        "validateUpdates",
        "Agent"
      );
    }

    if (
      updates.status &&
      !["active", "inactive", "maintenance"].includes(updates.status)
    ) {
      throw new ValidationError(
        "Agent status must be one of: active, inactive, maintenance",
        "validateUpdates",
        "Agent"
      );
    }

    if (
      updates.capabilities &&
      (!Array.isArray(updates.capabilities) ||
        updates.capabilities.length === 0)
    ) {
      throw new ValidationError(
        "Agent must have at least one capability",
        "validateUpdates",
        "Agent"
      );
    }
  }
}
