/**
 * Agent API Client
 *
 * Provides functions for fetching agent data and statistics from the v3 API.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost, apiPatch } from "../utils/api";

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
  agent_name: string;
  worker_type: string;
  total_tasks: number;
  completed_tasks: number;
  tasks_per_hour: number;
  success_rate: number;
  avg_execution_time_ms: number | null;
  median_execution_time_ms: number | null;
  p95_execution_time_ms: number | null;
  efficiency_score: number;
  total_tokens_used: number | null;
  avg_tokens_per_task: number | null;
  period_hours: number;
}

/**
 * Task completion metrics per agent
 */
export interface TaskCompletionMetrics {
  agent_id: string;
  agent_name: string;
  worker_type: string;
  total_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  cancelled_tasks: number;
  running_tasks: number;
  completion_rate_percent: number;
  success_rate: number;
  avg_execution_time_ms: number | null;
  min_execution_time_ms: number | null;
  max_execution_time_ms: number | null;
  period_hours: number;
  period_start: string;
}

/**
 * Task completion response with summary
 */
export interface TaskCompletionResponse {
  agents: TaskCompletionMetrics[];
  summary: {
    total_agents: number;
    total_tasks: number;
    total_completed: number;
    total_failed: number;
    overall_completion_rate: number;
    overall_success_rate: number;
  };
  period_hours: number;
  period_start: string;
  period_end: string;
}

/**
 * Efficiency metrics response with summary
 */
export interface EfficiencyResponse {
  agents: EfficiencyMetrics[];
  summary: {
    total_agents: number;
    total_completed_tasks: number;
    overall_tasks_per_hour: number;
    avg_efficiency_score: number;
  };
  period_hours: number;
  period_start: string;
  period_end: string;
}

const API_BASE = "/api/proxy/api/v1";

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
  // API returns { "agents": [...] }, so extract the agents array
  const response = await apiGet<{ agents?: Agent[] } | Agent[]>(`${API_BASE}/agents`);
  return Array.isArray(response) ? response : (response.agents || []);
}

/**
 * Get statistics for a specific agent
 */
export async function getAgentStats(
  agentId: string
): Promise<AgentDetailStats> {
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
  if (params?.agent_id) queryParams.append("agent_id", params.agent_id);
  if (params?.start_date) queryParams.append("start_date", params.start_date);
  if (params?.end_date) queryParams.append("end_date", params.end_date);

  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/agent-activity${
    queryString ? `?${queryString}` : ""
  }`;
  return apiGet<AgentActivityPoint[]>(url);
}

/**
 * Monthly model contribution data point
 */
export interface MonthlyModelContribution {
  month: string;
  [model: string]: string | number;
}

/**
 * Model contributions response with monthly breakdown
 */
export interface ModelContributionsResponse {
  total_requests?: number;
  models?: ModelContribution[];
  monthly_contributions?: MonthlyModelContribution[];
  models_list?: string[];
}

/**
 * Get model contribution statistics
 * Use group_by=month to get monthly breakdown for charts
 */
export async function getModelContributions(params?: {
  group_by?: "month" | "source";
}): Promise<ModelContribution[] | ModelContributionsResponse> {
  const queryParams = new URLSearchParams();
  if (params?.group_by) queryParams.append("group_by", params.group_by);

  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/model-contributions${
    queryString ? `?${queryString}` : ""
  }`;

  // If group_by=month, return the full response with monthly breakdown
  if (params?.group_by === "month") {
    return apiGet<ModelContributionsResponse>(url);
  }

  // Otherwise return legacy format (for backwards compatibility)
  return apiGet<ModelContribution[]>(url);
}

/**
 * Daily contribution data point
 */
export interface DailyContribution {
  day: string;
  count: number;
  unique_contributors: number;
}

/**
 * Contributions response with daily breakdown
 */
export interface ContributionsResponse {
  period_days: number;
  total_contributions: number;
  unique_contributors: number;
  daily_contributions?: DailyContribution[];
}

/**
 * Get code contribution statistics
 * Use group_by=day to get daily breakdown for charts
 */
export async function getContributions(params?: {
  agent_id?: string;
  start_date?: string;
  end_date?: string;
  group_by?: "day" | "total";
}): Promise<ContributionStats[] | ContributionsResponse> {
  const queryParams = new URLSearchParams();
  if (params?.agent_id) queryParams.append("agent_id", params.agent_id);
  if (params?.start_date) queryParams.append("start_date", params.start_date);
  if (params?.end_date) queryParams.append("end_date", params.end_date);
  if (params?.group_by) queryParams.append("group_by", params.group_by);

  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/contributions${
    queryString ? `?${queryString}` : ""
  }`;

  // If group_by=day, return the full response with daily breakdown
  if (params?.group_by === "day") {
    return apiGet<ContributionsResponse>(url);
  }

  // Otherwise return legacy format (for backwards compatibility)
  return apiGet<ContributionStats[]>(url);
}

/**
 * Get efficiency metrics for all agents
 * Uses the new /agents/efficiency endpoint
 */
export async function getEfficiencyMetrics(params?: {
  hours?: number;
}): Promise<EfficiencyResponse> {
  const queryParams = new URLSearchParams();
  if (params?.hours) queryParams.append("hours", params.hours.toString());

  const queryString = queryParams.toString();
  const url = `${API_BASE}/agents/efficiency${
    queryString ? `?${queryString}` : ""
  }`;
  return apiGet<EfficiencyResponse>(url);
}

/**
 * Get task completion metrics for all agents
 * Uses the new /agents/tasks/completion endpoint
 */
export async function getAgentsTaskCompletion(params?: {
  hours?: number;
}): Promise<TaskCompletionResponse> {
  const queryParams = new URLSearchParams();
  if (params?.hours) queryParams.append("hours", params.hours.toString());

  const queryString = queryParams.toString();
  const url = `${API_BASE}/agents/tasks/completion${
    queryString ? `?${queryString}` : ""
  }`;
  return apiGet<TaskCompletionResponse>(url);
}

/**
 * Agent health status
 */
export interface AgentHealth {
  agent_id: string;
  status: "healthy" | "warning" | "critical" | "offline";
  uptime_seconds: number;
  last_seen: string;
  error_count: number;
  response_time_ms: number;
  health_score: number;
}

/**
 * Agent metrics
 */
export interface AgentMetrics {
  agent_id: string;
  cpu_usage_percent: number;
  memory_usage_mb: number;
  response_time_p50_ms: number;
  response_time_p95_ms: number;
  response_time_p99_ms: number;
  requests_per_second: number;
  error_rate: number;
  timestamp: string;
}

/**
 * Agent log entry
 */
export interface AgentLog {
  id: string;
  agent_id: string;
  level: "error" | "warn" | "info" | "debug";
  message: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

/**
 * Get agent health status
 */
export async function getAgentHealth(agentId: string): Promise<AgentHealth> {
  return apiGet<AgentHealth>(`${API_BASE}/agents/${agentId}/health`);
}

/**
 * Get agent metrics
 */
export async function getAgentMetrics(agentId: string): Promise<AgentMetrics> {
  return apiGet<AgentMetrics>(`${API_BASE}/agents/${agentId}/metrics`);
}

/**
 * Get agent logs
 */
export async function getAgentLogs(
  agentId: string,
  params?: {
    level?: "error" | "warn" | "info" | "debug";
    limit?: number;
    offset?: number;
  }
): Promise<AgentLog[]> {
  const queryParams = new URLSearchParams();
  if (params?.level) queryParams.append("level", params.level);
  if (params?.limit) queryParams.append("limit", params.limit.toString());
  if (params?.offset) queryParams.append("offset", params.offset.toString());

  const queryString = queryParams.toString();
  const url = `${API_BASE}/agents/${agentId}/logs${
    queryString ? `?${queryString}` : ""
  }`;
  return apiGet<AgentLog[]>(url);
}

/**
 * Restart agent
 */
export async function restartAgent(
  agentId: string
): Promise<{ success: boolean; message: string }> {
  return apiPost<{ success: boolean; message: string }>(
    `${API_BASE}/agents/${agentId}/restart`
  );
}

/**
 * Stop agent
 */
export async function stopAgent(
  agentId: string
): Promise<{ success: boolean; message: string }> {
  return apiPost<{ success: boolean; message: string }>(
    `${API_BASE}/agents/${agentId}/stop`
  );
}

/**
 * Update agent
 */
export async function updateAgent(
  agentId: string,
  updates: Partial<
    Pick<Agent, "name" | "is_active" | "specialty" | "capabilities">
  >
): Promise<Agent> {
  return apiPatch<Agent>(`${API_BASE}/agents/${agentId}`, updates);
}
