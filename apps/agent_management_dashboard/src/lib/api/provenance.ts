/**
 * Provenance API Client
 *
 * Provides functions for code provenance and audit tracking endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost } from './base';
import { z } from 'zod';

/**
 * Provenance response schema
 */
export const ProvenanceResponseSchema = z.object({
  id: z.string(),
  verdict_id: z.string(),
  task_id: z.string(),
  decision: z.record(z.string(), z.unknown()),
  consensus_score: z.number(),
  caws_compliance: z.record(z.string(), z.unknown()),
  git_commit_hash: z.string().optional(),
  git_trailer: z.string(),
  signature: z.string(),
  timestamp: z.string(),
  metadata: z.record(z.string(), z.unknown()).optional(),
});

export type ProvenanceResponse = z.infer<typeof ProvenanceResponseSchema>;

/**
 * Link provenance request schema
 */
export const LinkProvenanceRequestSchema = z.object({
  task_id: z.string(),
  provenance_id: z.string(),
  relationship_type: z.string(),
  commit_hash: z.string(),
});

export type LinkProvenanceRequest = z.infer<typeof LinkProvenanceRequestSchema>;

/**
 * Provenance verification response schema
 */
export const ProvenanceVerificationResponseSchema = z.object({
  verified: z.boolean(),
  commit_hash: z.string(),
  signature: z.string(),
  timestamp: z.string(),
  message: z.string().optional(),
});

export type ProvenanceVerificationResponse = z.infer<typeof ProvenanceVerificationResponseSchema>;

/**
 * List provenance records
 */
export async function listProvenance(): Promise<ProvenanceResponse[]> {
  return apiGet<ProvenanceResponse[]>('/api/v1/provenance', {
    responseSchema: z.array(ProvenanceResponseSchema),
  });
}

/**
 * Link provenance to task
 */
export async function linkProvenance(request: LinkProvenanceRequest): Promise<void> {
  await apiPost<LinkProvenanceRequest>('/api/v1/provenance/link', request, {
    requestSchema: LinkProvenanceRequestSchema,
  });
}

/**
 * Verify provenance
 */
export async function verifyProvenance(commitHash: string): Promise<ProvenanceVerificationResponse> {
  return apiGet<ProvenanceVerificationResponse>(`/api/v1/provenance/verify/${commitHash}`, {
    responseSchema: ProvenanceVerificationResponseSchema,
  });
}

/**
 * Get provenance by commit hash
 */
export async function getProvenanceByCommit(commitHash: string): Promise<ProvenanceResponse> {
  return apiGet<ProvenanceResponse>(`/api/v1/provenance/commit/${commitHash}`, {
    responseSchema: ProvenanceResponseSchema,
  });
}

/**
 * Get task provenance
 */
export async function getTaskProvenance(taskId: string): Promise<ProvenanceResponse[]> {
  return apiGet<ProvenanceResponse[]>(`/api/v1/tasks/${taskId}/provenance`, {
    responseSchema: z.array(ProvenanceResponseSchema),
  });
}

