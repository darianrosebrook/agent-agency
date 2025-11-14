/**
 * System API Client
 *
 * Provides functions for system monitoring endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet } from './base';
import { z } from 'zod';

/**
 * System health response schema
 */
export const SystemHealthResponseSchema = z.object({
  status: z.string(),
  components: z.record(z.object({
    status: z.string(),
    message: z.string().optional(),
  })).optional(),
  timestamp: z.string().optional(),
});

export type SystemHealthResponse = z.infer<typeof SystemHealthResponseSchema>;

/**
 * System resources response schema
 */
export const SystemResourcesResponseSchema = z.object({
  cpu: z.object({
    usage_percent: z.number(),
    cores: z.number().optional(),
  }).optional(),
  memory: z.object({
    used_mb: z.number(),
    total_mb: z.number(),
    usage_percent: z.number(),
  }).optional(),
  disk: z.object({
    used_gb: z.number(),
    total_gb: z.number(),
    usage_percent: z.number(),
  }).optional(),
  network: z.object({
    bytes_sent: z.number(),
    bytes_received: z.number(),
  }).optional(),
});

export type SystemResourcesResponse = z.infer<typeof SystemResourcesResponseSchema>;

/**
 * System metrics response schema
 */
export const SystemMetricsResponseSchema = z.object({
  requests_per_second: z.number().optional(),
  response_time_ms: z.number().optional(),
  error_rate: z.number().optional(),
  active_connections: z.number().optional(),
});

export type SystemMetricsResponse = z.infer<typeof SystemMetricsResponseSchema>;

/**
 * Get system health
 */
export async function getSystemHealth(): Promise<SystemHealthResponse> {
  return apiGet<SystemHealthResponse>('/api/v1/system/health', {
    responseSchema: SystemHealthResponseSchema,
  });
}

/**
 * Get system resources
 */
export async function getSystemResources(): Promise<SystemResourcesResponse> {
  return apiGet<SystemResourcesResponse>('/api/v1/system/resources', {
    responseSchema: SystemResourcesResponseSchema,
  });
}

/**
 * Get system metrics
 */
export async function getSystemMetrics(): Promise<SystemMetricsResponse> {
  return apiGet<SystemMetricsResponse>('/api/v1/system/metrics', {
    responseSchema: SystemMetricsResponseSchema,
  });
}

