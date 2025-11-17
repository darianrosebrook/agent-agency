/**
 * Judges API Client
 *
 * Provides functions for judge management endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost, apiPatch, apiDelete } from '../utils/api';
import { z } from 'zod';

/**
 * Judge response schema
 */
export const JudgeResponseSchema = z.object({
  id: z.string(),
  name: z.string(),
  status: z.string(),
  metadata: z.record(z.string(), z.unknown()).optional(),
});

export type JudgeResponse = z.infer<typeof JudgeResponseSchema>;

/**
 * Judge statistics schema
 */
export const JudgeStatsSchema = z.object({
  total_evaluations: z.number(),
  success_rate: z.number(),
  average_score: z.number().optional(),
});

export type JudgeStats = z.infer<typeof JudgeStatsSchema>;

/**
 * Judge evaluation schema
 */
export const JudgeEvaluationSchema = z.object({
  id: z.string(),
  task_id: z.string(),
  score: z.number(),
  verdict: z.string(),
  created_at: z.string(),
});

export type JudgeEvaluation = z.infer<typeof JudgeEvaluationSchema>;

const API_BASE = '/api/proxy/api/v1';

/**
 * List all judges
 */
export async function listJudges(): Promise<JudgeResponse[]> {
  const response = await apiGet<{ judges?: JudgeResponse[] } | JudgeResponse[]>(`${API_BASE}/judges`);
  return Array.isArray(response) ? response : (response.judges || []);
}

/**
 * Get judge details
 */
export async function getJudge(id: string): Promise<JudgeResponse> {
  return apiGet<JudgeResponse>(`${API_BASE}/judges/${id}`);
}

/**
 * Get judge statistics
 */
export async function getJudgesStats(): Promise<JudgeStats> {
  return apiGet<JudgeStats>(`${API_BASE}/judges/stats`);
}

/**
 * Get judge-specific statistics
 */
export async function getJudgeStats(id: string): Promise<JudgeStats> {
  return apiGet<JudgeStats>(`${API_BASE}/judges/${id}/stats`);
}

/**
 * Get judge evaluations
 */
export async function getJudgeEvaluations(
  id: string,
  options?: { limit?: number; offset?: number }
): Promise<JudgeEvaluation[]> {
  const params = new URLSearchParams();
  if (options?.limit) params.set('limit', options.limit.toString());
  if (options?.offset) params.set('offset', options.offset.toString());
  
  const query = params.toString() ? `?${params.toString()}` : '';
  
  const response = await apiGet<{ evaluations?: JudgeEvaluation[] } | JudgeEvaluation[]>(
    `${API_BASE}/judges/${id}/evaluations${query}`
  );
  return Array.isArray(response) ? response : (response.evaluations || []);
}

