/**
 * Observability API Client
 * 
 * Provides functions for fetching observability and efficiency metrics from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet } from '../utils/api';
import type { EfficiencyResponse, EfficiencyMetrics as AgentEfficiencyMetrics } from './agents';

/**
 * Efficiency metrics for observability charts
 */
export interface EfficiencyMetrics {
  agent_id?: string;
  efficiency_score: number;
  resource_utilization: number;
  throughput: number;
  timestamp: string;
}

/**
 * System metrics
 */
export interface SystemMetrics {
  cpu_usage_percent: number;
  memory_usage_mb: number;
  disk_usage_percent: number;
  network_io_mbps: number;
  timestamp: string;
}

/**
 * Alert information
 */
export interface Alert {
  id: string;
  severity: 'critical' | 'warning' | 'info';
  title: string;
  message: string;
  source: string;
  timestamp: string;
  acknowledged: boolean;
  resolved: boolean;
}

const API_BASE = '/api/proxy/api/v1';

/**
 * Get efficiency metrics
 * Uses the agents/efficiency endpoint which returns the correct format
 */
export async function getEfficiencyMetrics(agentId?: string): Promise<EfficiencyMetrics[]> {
  const queryParams = new URLSearchParams();
  if (agentId) queryParams.append('agent_id', agentId);
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/agents/efficiency${queryString ? `?${queryString}` : ''}`;
  const response = await apiGet<EfficiencyResponse>(url);
  
  // Safely extract agents array
  const agentsArray = Array.isArray(response?.agents) ? response.agents : [];
  
  // Transform AgentEfficiencyMetrics[] to EfficiencyMetrics[]
  return agentsArray.map((agent: AgentEfficiencyMetrics) => ({
    agent_id: agent.agent_id,
    efficiency_score: agent.efficiency_score,
    resource_utilization: 0, // Not available in backend response
    throughput: agent.tasks_per_hour,
    timestamp: response?.period_end || new Date().toISOString(),
  }));
}

/**
 * Get system metrics
 * 
 * Note: Endpoint may return NOT_IMPLEMENTED (501) if not yet implemented.
 * Returns null if endpoint is unavailable.
 */
export async function getSystemMetrics(): Promise<SystemMetrics | null> {
  try {
    return await apiGet<SystemMetrics>(`${API_BASE}/observability/system-metrics`);
  } catch (error) {
    // Endpoint may not be implemented yet (501) or may not exist (404)
    console.warn('System metrics endpoint unavailable:', error);
    return null;
  }
}

/**
 * Get active alerts
 * 
 * Note: Endpoint may return NOT_IMPLEMENTED (501) if not yet implemented.
 * Returns empty array if endpoint is unavailable.
 */
export async function getAlerts(params?: {
  severity?: 'critical' | 'warning' | 'info';
  acknowledged?: boolean;
  resolved?: boolean;
}): Promise<Alert[]> {
  try {
    const queryParams = new URLSearchParams();
    if (params?.severity) queryParams.append('severity', params.severity);
    if (params?.acknowledged !== undefined) queryParams.append('acknowledged', params.acknowledged.toString());
    if (params?.resolved !== undefined) queryParams.append('resolved', params.resolved.toString());
    
    const queryString = queryParams.toString();
    const url = `${API_BASE}/observability/alerts${queryString ? `?${queryString}` : ''}`;
    return await apiGet<Alert[]>(url);
  } catch (error) {
    // Endpoint may not be implemented yet (501) or may not exist (404)
    console.warn('Alerts endpoint unavailable:', error);
    return [];
  }
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
 * Contribution statistics (legacy format)
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
 * Get code contribution statistics
 * Use group_by=day to get daily breakdown for charts
 */
export async function getContributions(params?: {
  agent_id?: string;
  start_date?: string;
  end_date?: string;
  days?: number;
  group_by?: 'day' | 'total';
}): Promise<ContributionStats[] | ContributionsResponse> {
  const queryParams = new URLSearchParams();
  if (params?.agent_id) queryParams.append('agent_id', params.agent_id);
  if (params?.start_date) queryParams.append('start_date', params.start_date);
  if (params?.end_date) queryParams.append('end_date', params.end_date);
  if (params?.days) queryParams.append('days', params.days.toString());
  if (params?.group_by) queryParams.append('group_by', params.group_by);

  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/contributions${queryString ? `?${queryString}` : ''}`;

  // If group_by=day, return the full response with daily breakdown
  if (params?.group_by === 'day') {
    return apiGet<ContributionsResponse>(url);
  }

  // Otherwise return legacy format (for backwards compatibility)
  return apiGet<ContributionStats[]>(url);
}

/**
 * Monthly model contribution data point
 */
export interface MonthlyModelContribution {
  month: string;
  [model: string]: string | number;
}

/**
 * Model contribution statistics (legacy format)
 */
export interface ModelContribution {
  model: string;
  request_count: number;
  last_used?: string;
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
  group_by?: 'month' | 'source';
}): Promise<ModelContribution[] | ModelContributionsResponse> {
  const queryParams = new URLSearchParams();
  if (params?.group_by) queryParams.append('group_by', params.group_by);

  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/model-contributions${queryString ? `?${queryString}` : ''}`;

  // If group_by=month, return the full response with monthly breakdown
  if (params?.group_by === 'month') {
    return apiGet<ModelContributionsResponse>(url);
  }

  // Otherwise return legacy format (for backwards compatibility)
  return apiGet<ModelContribution[]>(url);
}

/**
 * Agent activity data point
 */
export interface AgentActivityPoint {
  hour: string;
  source: string;
  activity_count: number;
}

/**
 * Agent activity response
 */
export interface AgentActivityResponse {
  period_hours: number;
  time_series: AgentActivityPoint[];
  by_source: Array<{
    source: string;
    total_activity: number;
  }>;
}

/**
 * Get agent activity time-series data
 */
export async function getAgentActivity(params?: {
  agent_id?: string;
  hours?: number;
}): Promise<AgentActivityResponse> {
  const queryParams = new URLSearchParams();
  if (params?.agent_id) queryParams.append('agent_id', params.agent_id);
  if (params?.hours) queryParams.append('hours', params.hours.toString());

  const queryString = queryParams.toString();
  const url = `${API_BASE}/telemetry/agent-activity${queryString ? `?${queryString}` : ''}`;
  return apiGet<AgentActivityResponse>(url);
}











