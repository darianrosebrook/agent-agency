/**
 * System API Client
 *
 * Provides functions for system monitoring endpoints.
 *
 * @author @darianrosebrook
 */

import { z } from "zod";
import { apiGet } from "./base";

/**
 * System health response schema
 */
export const SystemHealthResponseSchema = z.object({
  status: z.string(),
  components: z
    .record(
      z.string(),
      z.object({
        status: z.string(),
        message: z.string().optional(),
      })
    )
    .optional(),
  timestamp: z.string().optional(),
});

export type SystemHealthResponse = z.infer<typeof SystemHealthResponseSchema>;

/**
 * System resources response schema
 */
export const SystemResourcesResponseSchema = z.object({
  cpu: z
    .object({
      usage_percent: z.number(),
      cores: z.number().optional(),
    })
    .optional(),
  memory: z
    .object({
      used_mb: z.number(),
      total_mb: z.number(),
      usage_percent: z.number(),
    })
    .optional(),
  disk: z
    .object({
      used_gb: z.number(),
      total_gb: z.number(),
      usage_percent: z.number(),
    })
    .optional(),
  network: z
    .object({
      bytes_sent: z.number(),
      bytes_received: z.number(),
    })
    .optional(),
});

export type SystemResourcesResponse = z.infer<
  typeof SystemResourcesResponseSchema
>;

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
 *
 * Note: Endpoint may return NOT_IMPLEMENTED (501) if not yet implemented.
 * Returns null if endpoint is unavailable.
 */
export async function getSystemHealth(): Promise<SystemHealthResponse | null> {
  try {
    return await apiGet<SystemHealthResponse>("/system/health", {
      responseSchema: SystemHealthResponseSchema,
      showToast: false, // Suppress toast for expected 404/501
    });
  } catch (error: unknown) {
    console.warn("System health endpoint unavailable:", error);
    // Endpoint may not be implemented yet (501) or may not exist (404)
    // Silently return null - this is expected if endpoint doesn't exist
    return null;
  }
}

/**
 * Get system resources
 *
 * Note: Endpoint may return NOT_IMPLEMENTED (501) if not yet implemented.
 * Returns null if endpoint is unavailable.
 */
export async function getSystemResources(): Promise<SystemResourcesResponse | null> {
  try {
    return await apiGet<SystemResourcesResponse>("/system/resources", {
      responseSchema: SystemResourcesResponseSchema,
      showToast: false, // Suppress toast for expected 404/501
    });
  } catch (error: unknown) {
    console.warn("System resources endpoint unavailable:", error);
    // Endpoint may not be implemented yet (501) or may not exist (404)
    // Silently return null - this is expected if endpoint doesn't exist
    return null;
  }
}

/**
 * Get system metrics
 *
 * Note: Endpoint may return NOT_IMPLEMENTED (501) if not yet implemented.
 * Returns null if endpoint is unavailable.
 */
export async function getSystemMetrics(): Promise<SystemMetricsResponse | null> {
  try {
    return await apiGet<SystemMetricsResponse>("/system/metrics", {
      responseSchema: SystemMetricsResponseSchema,
      showToast: false, // Suppress toast for expected 404/501
    });
  } catch (error: unknown) {
    console.warn("System metrics endpoint unavailable:", error);
    // Endpoint may not be implemented yet (501) or may not exist (404)
    // Silently return null - this is expected if endpoint doesn't exist
    return null;
  }
}
