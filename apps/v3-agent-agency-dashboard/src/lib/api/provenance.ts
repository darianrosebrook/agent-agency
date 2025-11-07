// Provenance API endpoints
import { apiClient } from './client';
import type { ProvenanceEntry } from '@/types';

export const provenanceApi = {
  async listProvenance(): Promise<ProvenanceEntry[]> {
    return apiClient.get<ProvenanceEntry[]>('/api/v1/provenance');
  },

  async getTaskProvenance(taskId: string): Promise<ProvenanceEntry[]> {
    return apiClient.get<ProvenanceEntry[]>(`/api/v1/tasks/${taskId}/provenance`);
  },

  async verifyProvenance(commitHash: string): Promise<{ verified: boolean }> {
    return apiClient.get<{ verified: boolean }>(`/api/v1/provenance/verify/${commitHash}`);
  },

  async getProvenanceByCommit(commitHash: string): Promise<ProvenanceEntry[]> {
    return apiClient.get<ProvenanceEntry[]>(`/api/v1/provenance/commit/${commitHash}`);
  },

  async linkProvenance(data: {
    task_id: string;
    commit_hash: string;
  }): Promise<ProvenanceEntry> {
    return apiClient.post<ProvenanceEntry>('/api/v1/provenance/link', data);
  },
};

