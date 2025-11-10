/**
 * Agent API Client
 * 
 * Provides functions for fetching agent data and statistics from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet } from '../utils/api';

/**
 * Agent statistics response
 */
export interface AgentStats {
  total: number;
  active: number;
  inactive: number;
  by_type: Record<string, number>;
}

/**
 * Individual agent statistics
 */
export interface AgentDetailStats {
  agent_id: string;
  name: string;
  total_tasks: number;
  completed: number;
  failed: number;
  in_progress: number;
  success_rate: number;
  performance_history: unknown;
}

/**
 * Agent information
 */
export interface Agent {
  id: string;
  name: string;
  worker_type: string;
  specialty: string | null;
  model_name: string | null;
  endpoint: string | null;
  capabilities: string[] | null;
  performance_history: unknown;
  is_active: boolean;
}

/**
 * Agent activity data point
 */
export interface AgentActivityPoint {
  timestamp: string;
  agent_id: string;
  activity_type: string;
  count: number;
}

/**
 * Model contribution statistics
 */
export interface ModelContribution {
  model_name: string;
  task_count: number;
  success_rate: number;
  avg_completion_time: number;
}

/**
 * Contribution statistics
 */
export interface ContributionStats {
  agent_id: string;
  agent_name: string;
  lines_added: number;
  lines_modified: number;
  lines_deleted: number;
  files_changed: number;
  commits: number;
}

/**
 * Efficiency metrics
 */
export interface EfficiencyMetrics {
  agent_id: string;
  efficiency_score: number;
  resource_utilization: number;
  throughput: number;
}

const API_BASE = '/api/proxy/api/v1';

/**
 * Get overall agent statistics
 */
export async function getAgentsStats(): Promise<AgentStats> {
  return apiGet<AgentStats>(`${API_BASE}/agents/stats`);
}

/**
 * Get list of all agents
 */
export async function getAgents(): Promise<Agent[]> {
  return apiGet<Agent[]>(`${API_BASE}/agents`);
}

/**
 * Get statistics for a specific agent
 */
export async function getAgentStats(agentId: string): Promise<AgentDetailStats> {
  return apiGet<AgentDetailStats>(`${API_BASE}/agents/${agentId}/stats`);
}

/**
 * Get agent details
 */
export async function getAgent(agentId: string): Promise<Agent> {
  return apiGet<Agent>(`${API_BASE}/agents/${agentId}`);
}

/**
 * Get agent activity time-series data
 */
export async function getAgentActivity(params?: {
  agent_id?: string;
  start_date?: string;
  end_date?: string;
}): Promise<AgentActivityPoint[]> {
  const queryParams = new URLSearchParams();
  if (params?.agent_id) queryParams.append('agent_id', params.agent_id);
  if (params?.start_date) queryParams.append('start_date', params.start_date);
  if (params?.end_date) queryParams.append('end_date', params.end_date);
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/agent-activity${queryString ? `?${queryString}` : ''}`;
  return apiGet<AgentActivityPoint[]>(url);
}

/**
 * Get model contribution statistics
 */
export async function getModelContributions(): Promise<ModelContribution[]> {
  return apiGet<ModelContribution[]>(`${API_BASE}/telemetry/model-contributions`);
}

/**
 * Get code contribution statistics
 */
export async function getContributions(params?: {
  agent_id?: string;
  start_date?: string;
  end_date?: string;
}): Promise<ContributionStats[]> {
  const queryParams = new URLSearchParams();
  if (params?.agent_id) queryParams.append('agent_id', params.agent_id);
  if (params?.start_date) queryParams.append('start_date', params.start_date);
  if (params?.end_date) queryParams.append('end_date', params.end_date);
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/contributions${queryString ? `?${queryString}` : ''}`;
  return apiGet<ContributionStats[]>(url);
}

/**
 * Get efficiency metrics
 */
export async function getEfficiencyMetrics(agentId?: string): Promise<EfficiencyMetrics[]> {
  const url = agentId 
    ? `${API_BASE}/observability/efficiency?agent_id=${agentId}`
    : `${API_BASE}/observability/efficiency`;
  return apiGet<EfficiencyMetrics[]>(url);
}

