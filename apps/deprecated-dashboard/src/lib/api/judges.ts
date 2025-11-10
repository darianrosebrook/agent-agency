// Judges API endpoints
import { serverApi } from "./server";

export interface Judge {
  id: string;
  name: string;
  judge_type: string;
  description?: string;
  configuration: Record<string, unknown>;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface JudgeStats {
  total_judges: number;
  active_judges: number;
  inactive_judges: number;
  judges_by_type: Record<string, number>;
  total_evaluations: number;
  avg_confidence: number;
}

export interface JudgeEvaluation {
  id: string;
  judge_id: string;
  task_id: string;
  verdict: string;
  confidence: number;
  reasoning: string;
  timestamp: string;
}

export interface CreateJudgeRequest {
  name: string;
  judge_type: string;
  description?: string;
  configuration?: Record<string, unknown>;
}

export interface UpdateJudgeRequest {
  name?: string;
  description?: string;
  configuration?: Record<string, unknown>;
  is_active?: boolean;
}

export const judgesApi = {
  async listJudges(): Promise<Judge[]> {
    return serverApi.get<Judge[]>("/api/v1/judges");
  },

  async getJudge(id: string): Promise<Judge> {
    return serverApi.get<Judge>(`/api/v1/judges/${id}`);
  },

  async getJudgesStats(): Promise<JudgeStats> {
    return serverApi.get<JudgeStats>("/api/v1/judges/stats");
  },

  async getJudgeStats(id: string): Promise<JudgeStats> {
    return serverApi.get<JudgeStats>(`/api/v1/judges/${id}/stats`);
  },

  async getJudgeEvaluations(id: string): Promise<JudgeEvaluation[]> {
    return serverApi.get<JudgeEvaluation[]>(`/api/v1/judges/${id}/evaluations`);
  },

  async createJudge(judge: CreateJudgeRequest): Promise<Judge> {
    return serverApi.post<Judge>("/api/v1/judges", judge);
  },

  async updateJudge(id: string, updates: UpdateJudgeRequest): Promise<Judge> {
    return serverApi.patch<Judge>(`/api/v1/judges/${id}`, updates);
  },

  async deleteJudge(id: string): Promise<void> {
    return serverApi.delete(`/api/v1/judges/${id}`);
  },
};


