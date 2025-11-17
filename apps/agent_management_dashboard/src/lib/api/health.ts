/**
 * Health API Client
 *
 * Provides functions for health check endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet } from './base';
import { z } from 'zod';

/**
 * Health check response schema
 */
export const HealthResponseSchema = z.object({
  status: z.string(),
  timestamp: z.string().optional(),
});

export type HealthResponse = z.infer<typeof HealthResponseSchema>;

/**
 * System health response schema
 */
export const SystemHealthResponseSchema = z.object({
  status: z.string(),
  components: z.record(z.string(), z.object({
    status: z.string(),
    message: z.string().optional(),
  })).optional(),
  timestamp: z.string().optional(),
});

export type SystemHealthResponse = z.infer<typeof SystemHealthResponseSchema>;

/**
 * Get basic health check
 */
export async function getHealth(): Promise<HealthResponse> {
  return apiGet<HealthResponse>('/health', {
    responseSchema: HealthResponseSchema,
  });
}

/**
 * Get detailed system health check
 */
export async function getSystemHealth(): Promise<SystemHealthResponse> {
  return apiGet<SystemHealthResponse>('/api/v1/health', {
    responseSchema: SystemHealthResponseSchema,
  });
}

