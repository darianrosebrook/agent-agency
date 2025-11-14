/**
 * Sessions API Client
 *
 * Provides functions for session control endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost } from './base';
import { z } from 'zod';

/**
 * Session status response schema
 */
export const SessionStatusResponseSchema = z.object({
  session_id: z.string(),
  status: z.string(),
  task_id: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
});

export type SessionStatusResponse = z.infer<typeof SessionStatusResponseSchema>;

/**
 * Get session status
 */
export async function getSessionStatus(sessionId: string): Promise<SessionStatusResponse> {
  return apiGet<SessionStatusResponse>(`/api/v1/sessions/${sessionId}`, {
    responseSchema: SessionStatusResponseSchema,
  });
}

/**
 * Pause session
 */
export async function pauseSession(sessionId: string): Promise<void> {
  await apiPost(`/api/v1/sessions/${sessionId}/pause`, undefined);
}

/**
 * Resume session
 */
export async function resumeSession(sessionId: string): Promise<void> {
  await apiPost(`/api/v1/sessions/${sessionId}/resume`, undefined);
}

/**
 * Cancel session
 */
export async function cancelSession(sessionId: string): Promise<void> {
  await apiPost(`/api/v1/sessions/${sessionId}/cancel`, undefined);
}

