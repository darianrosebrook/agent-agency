/**
 * Query Performance API Client
 *
 * Provides functions for database query performance monitoring endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet } from './base';
import { z } from 'zod';

/**
 * Query performance summary schema
 */
export const QueryPerformanceSummarySchema = z.object({
  total_queries: z.number(),
  average_duration_ms: z.number(),
  slow_queries_count: z.number(),
  p95_duration_ms: z.number().optional(),
  p99_duration_ms: z.number().optional(),
});

export type QueryPerformanceSummary = z.infer<typeof QueryPerformanceSummarySchema>;

/**
 * Query performance metric schema
 */
export const QueryPerformanceMetricSchema = z.object({
  query: z.string(),
  duration_ms: z.number(),
  executed_at: z.string(),
  parameters: z.record(z.string(), z.unknown()).optional(),
});

export type QueryPerformanceMetric = z.infer<typeof QueryPerformanceMetricSchema>;

/**
 * Get query performance summary
 */
export async function getQueryPerformanceSummary(): Promise<QueryPerformanceSummary> {
  return apiGet<QueryPerformanceSummary>('/api/v1/query-performance/summary', {
    responseSchema: QueryPerformanceSummarySchema,
  });
}

/**
 * Get query performance metrics
 */
export async function getQueryPerformanceMetrics(): Promise<QueryPerformanceMetric[]> {
  return apiGet<QueryPerformanceMetric[]>('/api/v1/query-performance/metrics', {
    responseSchema: z.array(QueryPerformanceMetricSchema),
  });
}

/**
 * Get slow queries
 */
export async function getSlowQueries(options?: { limit?: number }): Promise<QueryPerformanceMetric[]> {
  const params = new URLSearchParams();
  if (options?.limit) params.set('limit', options.limit.toString());
  
  const query = params.toString() ? `?${params.toString()}` : '';
  
  return apiGet<QueryPerformanceMetric[]>(`/api/v1/query-performance/slow${query}`, {
    responseSchema: z.array(QueryPerformanceMetricSchema),
  });
}

/**
 * Get top slow queries
 */
export async function getTopSlowQueries(options?: { limit?: number }): Promise<QueryPerformanceMetric[]> {
  const params = new URLSearchParams();
  if (options?.limit) params.set('limit', options.limit.toString());
  
  const query = params.toString() ? `?${params.toString()}` : '';
  
  return apiGet<QueryPerformanceMetric[]>(`/api/v1/query-performance/top-slow${query}`, {
    responseSchema: z.array(QueryPerformanceMetricSchema),
  });
}

