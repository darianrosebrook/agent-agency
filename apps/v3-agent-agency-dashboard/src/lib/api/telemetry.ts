// Telemetry API endpoints
import { serverApi } from "./server";

export interface Contribution {
  agent_id: string;
  agent_name: string;
  contribution_count: number;
  contribution_percentage: number;
  last_contribution?: string;
}

export interface ModelContribution {
  model_id: string;
  model_name: string;
  contribution_count: number;
  contribution_percentage: number;
  avg_confidence?: number;
  last_used?: string;
}

export interface AgentActivity {
  agent_id: string;
  agent_name: string;
  activity_type: string;
  timestamp: string;
  details?: Record<string, unknown>;
}

export interface ContributionsResponse {
  contributions: Contribution[];
  total_contributions: number;
  period_start: string;
  period_end: string;
}

export interface ModelContributionsResponse {
  contributions: ModelContribution[];
  total_contributions: number;
  period_start: string;
  period_end: string;
}

export interface AgentActivityResponse {
  activities: AgentActivity[];
  total_activities: number;
  period_start: string;
  period_end: string;
}

export const telemetryApi = {
  async getContributions(): Promise<ContributionsResponse> {
    return serverApi.get<ContributionsResponse>("/api/v1/telemetry/contributions");
  },

  async getModelContributions(): Promise<ModelContributionsResponse> {
    return serverApi.get<ModelContributionsResponse>(
      "/api/v1/telemetry/model-contributions"
    );
  },

  async getAgentActivity(): Promise<AgentActivityResponse> {
    return serverApi.get<AgentActivityResponse>("/api/v1/telemetry/agent-activity");
  },
};

