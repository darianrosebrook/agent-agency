/**
 * Waivers API Client
 *
 * Provides functions for quality gate waiver management endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost } from './base';
import { z } from 'zod';

/**
 * Waiver response schema
 */
export const WaiverResponseSchema = z.object({
  id: z.string(),
  task_id: z.string(),
  title: z.string(),
  reason: z.string(),
  description: z.string(),
  gates: z.array(z.string()),
  approved_by: z.string(),
  impact_level: z.string(),
  mitigation_plan: z.string(),
  expires_at: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
  status: z.string(),
  metadata: z.record(z.unknown()).optional(),
});

export type WaiverResponse = z.infer<typeof WaiverResponseSchema>;

/**
 * Waiver request schema
 */
export const WaiverRequestSchema = z.object({
  task_id: z.string(),
  title: z.string().min(1),
  reason: z.string().min(1),
  description: z.string().min(1),
  gates: z.array(z.string()).min(1),
  approved_by: z.string().min(1),
  impact_level: z.enum(['low', 'medium', 'high', 'critical']),
  mitigation_plan: z.string().min(1),
  expires_at: z.string(),
  metadata: z.record(z.unknown()).optional(),
});

export type WaiverRequest = z.infer<typeof WaiverRequestSchema>;

/**
 * Waiver approval request schema
 */
export const WaiverApprovalRequestSchema = z.object({
  waiver_id: z.string(),
  approved_by: z.string().min(1),
  approval_notes: z.string().optional(),
});

export type WaiverApprovalRequest = z.infer<typeof WaiverApprovalRequestSchema>;

/**
 * List waivers
 */
export async function listWaivers(options?: {
  status?: string;
  task_id?: string;
}): Promise<WaiverResponse[]> {
  const params = new URLSearchParams();
  if (options?.status) params.set('status', options.status);
  if (options?.task_id) params.set('task_id', options.task_id);
  
  const query = params.toString() ? `?${params.toString()}` : '';
  
  return apiGet<WaiverResponse[]>(`/api/v1/waivers${query}`, {
    responseSchema: z.array(WaiverResponseSchema),
  });
}

/**
 * Create waiver
 */
export async function createWaiver(request: WaiverRequest): Promise<WaiverResponse> {
  return apiPost<WaiverRequest, WaiverResponse>('/api/v1/waivers', request, {
    requestSchema: WaiverRequestSchema,
    responseSchema: WaiverResponseSchema,
  });
}

/**
 * Approve waiver
 */
export async function approveWaiver(
  waiverId: string,
  request: WaiverApprovalRequest
): Promise<WaiverResponse> {
  return apiPost<WaiverApprovalRequest, WaiverResponse>(
    `/api/v1/waivers/${waiverId}/approve`,
    request,
    {
      requestSchema: WaiverApprovalRequestSchema,
      responseSchema: WaiverResponseSchema,
    }
  );
}

