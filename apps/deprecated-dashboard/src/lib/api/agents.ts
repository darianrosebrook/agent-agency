// Agents API endpoints
import { serverApi } from "./server";

export interface Agent {
  id: string;
  name: string;
  worker_type: string;
  specialty?: string;
  model_name: string;
  endpoint: string;
  capabilities: Record<string, unknown>;
  performance_history: Record<string, unknown>;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface AgentStats {
  total_agents: number;
  active_agents: number;
  inactive_agents: number;
  agents_by_type: Record<string, number>;
  agents_by_specialty: Record<string, number>;
}

export interface AgentHealth {
  agent_id: string;
  status: string;
  last_heartbeat?: string;
  metrics?: Record<string, unknown>;
  alerts?: Array<{
    severity: string;
    message: string;
    timestamp: string;
  }>;
}

export interface AgentMetrics {
  agent_id: string;
  metrics: Record<string, unknown>;
  timestamp: string;
}

export interface AgentLog {
  id: string;
  entity_type: string;
  entity_id: string;
  action: string;
  details: Record<string, unknown>;
  user_id?: string;
  ip_address?: string;
  created_at: string;
}

export interface AgentLogsResponse {
  agent_id: string;
  logs: AgentLog[];
  total: number;
  status: string;
  message?: string;
}

export const agentsApi = {
  async listAgents(): Promise<Agent[]> {
    return serverApi.get<Agent[]>("/api/v1/agents");
  },

  async getAgent(id: string): Promise<Agent> {
    return serverApi.get<Agent>(`/api/v1/agents/${id}`);
  },

  async getAgentsStats(): Promise<AgentStats> {
    return serverApi.get<AgentStats>("/api/v1/agents/stats");
  },

  async getAgentStats(id: string): Promise<AgentStats> {
    return serverApi.get<AgentStats>(`/api/v1/agents/${id}/stats`);
  },

  async getAgentHealth(id: string): Promise<AgentHealth> {
    return serverApi.get<AgentHealth>(`/api/v1/agents/${id}/health`);
  },

  async getAgentMetrics(id: string): Promise<AgentMetrics> {
    return serverApi.get<AgentMetrics>(`/api/v1/agents/${id}/metrics`);
  },

  async getAgentLogs(id: string): Promise<AgentLogsResponse> {
    return serverApi.get<AgentLogsResponse>(`/api/v1/agents/${id}/logs`);
  },

  async updateAgent(
    id: string,
    updates: Partial<Agent>
  ): Promise<Agent> {
    return serverApi.patch<Agent>(`/api/v1/agents/${id}`, updates);
  },

  async deleteAgent(id: string): Promise<void> {
    return serverApi.delete(`/api/v1/agents/${id}`);
  },

  async restartAgent(id: string): Promise<{ status: string; agent_id: string; message?: string }> {
    return serverApi.post<{ status: string; agent_id: string; message?: string }>(
      `/api/v1/agents/${id}/restart`,
      {}
    );
  },

  async stopAgent(id: string): Promise<{ status: string; agent_id: string; message?: string }> {
    return serverApi.post<{ status: string; agent_id: string; message?: string }>(
      `/api/v1/agents/${id}/stop`,
      {}
    );
  },
};


