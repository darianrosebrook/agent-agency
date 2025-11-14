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
 */
export async function getSystemMetrics(): Promise<SystemMetrics> {
  return apiGet<SystemMetrics>(`${API_BASE}/observability/system-metrics`);
}

/**
 * Get active alerts
 */
export async function getAlerts(params?: {
  severity?: 'critical' | 'warning' | 'info';
  acknowledged?: boolean;
  resolved?: boolean;
}): Promise<Alert[]> {
  const queryParams = new URLSearchParams();
  if (params?.severity) queryParams.append('severity', params.severity);
  if (params?.acknowledged !== undefined) queryParams.append('acknowledged', params.acknowledged.toString());
  if (params?.resolved !== undefined) queryParams.append('resolved', params.resolved.toString());
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/observability/alerts${queryString ? `?${queryString}` : ''}`;
  return apiGet<Alert[]>(url);
}











