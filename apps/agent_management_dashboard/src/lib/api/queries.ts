/**
 * Queries API Client
 *
 * Provides functions for query management endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost, apiDelete } from './base';
import { z } from 'zod';

/**
 * Saved query response schema
 */
export const SavedQueryResponseSchema = z.object({
  id: z.string(),
  name: z.string(),
  query_text: z.string(),
  description: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
});

export type SavedQueryResponse = z.infer<typeof SavedQueryResponseSchema>;

/**
 * Save query request schema
 */
export const SaveQueryRequestSchema = z.object({
  name: z.string().min(1),
  query_text: z.string().min(1),
  description: z.string().optional(),
});

export type SaveQueryRequest = z.infer<typeof SaveQueryRequestSchema>;

/**
 * List saved queries
 */
export async function listQueries(): Promise<SavedQueryResponse[]> {
  return apiGet<SavedQueryResponse[]>('/api/v1/queries', {
    responseSchema: z.array(SavedQueryResponseSchema),
  });
}

/**
 * Save a query
 */
export async function saveQuery(request: SaveQueryRequest): Promise<SavedQueryResponse> {
  return apiPost<SaveQueryRequest, SavedQueryResponse>('/api/v1/queries', request, {
    requestSchema: SaveQueryRequestSchema,
    responseSchema: SavedQueryResponseSchema,
  });
}

/**
 * Delete a query
 */
export async function deleteQuery(queryId: string): Promise<void> {
  await apiDelete(`/api/v1/queries/${queryId}`);
}

