/**
 * Database API Client
 *
 * Provides functions for database inspection endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost } from './base';
import { z } from 'zod';

/**
 * Database table schema
 */
export const DatabaseTableSchema = z.object({
  name: z.string(),
  schema: z.string().optional(),
  row_count: z.number().optional(),
});

export type DatabaseTable = z.infer<typeof DatabaseTableSchema>;

/**
 * Table schema response
 */
export const TableSchemaResponseSchema = z.object({
  name: z.string(),
  columns: z.array(z.object({
    name: z.string(),
    type: z.string(),
    nullable: z.boolean(),
    default: z.string().optional(),
  })),
  indexes: z.array(z.object({
    name: z.string(),
    columns: z.array(z.string()),
    unique: z.boolean(),
  })).optional(),
  foreign_keys: z.array(z.object({
    name: z.string(),
    columns: z.array(z.string()),
    referenced_table: z.string(),
    referenced_columns: z.array(z.string()),
  })).optional(),
});

export type TableSchemaResponse = z.infer<typeof TableSchemaResponseSchema>;

/**
 * Execute query request schema
 */
export const ExecuteQueryRequestSchema = z.object({
  query: z.string().min(1),
  parameters: z.record(z.string(), z.unknown()).optional(),
});

export type ExecuteQueryRequest = z.infer<typeof ExecuteQueryRequestSchema>;

/**
 * Query result schema
 */
export const QueryResultSchema = z.object({
  rows: z.array(z.record(z.string(), z.unknown())),
  row_count: z.number(),
  execution_time_ms: z.number().optional(),
});

export type QueryResult = z.infer<typeof QueryResultSchema>;

/**
 * List database tables
 */
export async function listDatabaseTables(): Promise<DatabaseTable[]> {
  return apiGet<DatabaseTable[]>('/api/v1/database/tables', {
    responseSchema: z.array(DatabaseTableSchema),
  });
}

/**
 * Get table schema
 */
export async function getTableSchema(tableName: string): Promise<TableSchemaResponse> {
  return apiGet<TableSchemaResponse>(`/api/v1/database/tables/${tableName}`, {
    responseSchema: TableSchemaResponseSchema,
  });
}

/**
 * Execute database query
 */
export async function executeQuery(request: ExecuteQueryRequest): Promise<QueryResult> {
  return apiPost<ExecuteQueryRequest, QueryResult>('/api/v1/database/query', request, {
    requestSchema: ExecuteQueryRequestSchema,
    responseSchema: QueryResultSchema,
  });
}

