/**
 * SLOs API Client
 *
 * Provides functions for service level objective management endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet } from './base';
import { z } from 'zod';

/**
 * SLO response schema
 */
export const SLOResponseSchema = z.object({
  name: z.string(),
  description: z.string().optional(),
  target: z.number(),
  window_days: z.number(),
  current_value: z.number().optional(),
  status: z.string().optional(),
});

export type SLOResponse = z.infer<typeof SLOResponseSchema>;

/**
 * SLO status response schema
 */
export const SLOStatusResponseSchema = z.object({
  name: z.string(),
  status: z.string(),
  current_value: z.number(),
  target: z.number(),
  compliance_percent: z.number(),
  window_start: z.string(),
  window_end: z.string(),
});

export type SLOStatusResponse = z.infer<typeof SLOStatusResponseSchema>;

/**
 * SLO measurement schema
 */
export const SLOMeasurementSchema = z.object({
  timestamp: z.string(),
  value: z.number(),
  target: z.number(),
  compliant: z.boolean(),
});

export type SLOMeasurement = z.infer<typeof SLOMeasurementSchema>;

/**
 * SLO alert schema
 */
export const SLOAlertSchema = z.object({
  id: z.string(),
  slo_name: z.string(),
  severity: z.string(),
  message: z.string(),
  created_at: z.string(),
  resolved: z.boolean(),
});

export type SLOAlert = z.infer<typeof SLOAlertSchema>;

/**
 * List SLOs
 */
export async function listSLOs(): Promise<SLOResponse[]> {
  return apiGet<SLOResponse[]>('/api/v1/slos', {
    responseSchema: z.array(SLOResponseSchema),
  });
}

/**
 * Get SLO status
 */
export async function getSLOStatus(sloName: string): Promise<SLOStatusResponse> {
  return apiGet<SLOStatusResponse>(`/api/v1/slos/${sloName}/status`, {
    responseSchema: SLOStatusResponseSchema,
  });
}

/**
 * Get SLO measurements
 */
export async function getSLOMeasurements(
  sloName: string,
  options?: { days?: number }
): Promise<SLOMeasurement[]> {
  const params = new URLSearchParams();
  if (options?.days) params.set('days', options.days.toString());
  
  const query = params.toString() ? `?${params.toString()}` : '';
  
  return apiGet<SLOMeasurement[]>(`/api/v1/slos/${sloName}/measurements${query}`, {
    responseSchema: z.array(SLOMeasurementSchema),
  });
}

/**
 * List SLO alerts
 */
export async function listSLOAlerts(options?: {
  severity?: string;
  limit?: number;
}): Promise<SLOAlert[]> {
  const params = new URLSearchParams();
  if (options?.severity) params.set('severity', options.severity);
  if (options?.limit) params.set('limit', options.limit.toString());
  
  const query = params.toString() ? `?${params.toString()}` : '';
  
  return apiGet<SLOAlert[]>(`/api/v1/slo-alerts${query}`, {
    responseSchema: z.array(SLOAlertSchema),
  });
}

