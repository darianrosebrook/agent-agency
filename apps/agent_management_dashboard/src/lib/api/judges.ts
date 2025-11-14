/**
 * Judges API Client
 *
 * Provides functions for judge management endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet } from './base';
import { z } from 'zod';

/**
 * Judge response schema
 */
export const JudgeResponseSchema = z.object({
  id: z.string(),
  name: z.string(),
  status: z.string(),
  metadata: z.record(z.unknown()).optional(),
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

/**
 * List all judges
 */
export async function listJudges(): Promise<JudgeResponse[]> {
  return apiGet<JudgeResponse[]>('/api/v1/judges', {
    responseSchema: z.array(JudgeResponseSchema),
  });
}

/**
 * Get judge details
 */
export async function getJudge(id: string): Promise<JudgeResponse> {
  return apiGet<JudgeResponse>(`/api/v1/judges/${id}`, {
    responseSchema: JudgeResponseSchema,
  });
}

/**
 * Get judge statistics
 */
export async function getJudgesStats(): Promise<JudgeStats> {
  return apiGet<JudgeStats>('/api/v1/judges/stats', {
    responseSchema: JudgeStatsSchema,
  });
}

/**
 * Get judge-specific statistics
 */
export async function getJudgeStats(id: string): Promise<JudgeStats> {
  return apiGet<JudgeStats>(`/api/v1/judges/${id}/stats`, {
    responseSchema: JudgeStatsSchema,
  });
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
  
  return apiGet<JudgeEvaluation[]>(`/api/v1/judges/${id}/evaluations${query}`, {
    responseSchema: z.array(JudgeEvaluationSchema),
  });
}

