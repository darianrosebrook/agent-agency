/**
 * Tasks API Client
 * 
 * Provides functions for fetching task data and statistics from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet } from '../utils/api';
import type { Task } from '../types/task';

/**
 * Task statistics response
 */
export interface TasksStats {
  total: number;
  completed: number;
  in_progress: number;
  pending: number;
  failed: number;
  cancelled: number;
  completion_rate: number;
  average_completion_time_seconds?: number;
  success_rate?: number;
}

const API_BASE = '/api/proxy/api/v1';

/**
 * Task interface - uses canonical Task from lib/types/task.ts
 * 
 * Re-exported for convenience and backward compatibility
 */
export type { Task };

/**
 * Tasks list response
 */
export interface TasksListResponse {
  tasks: Task[];
  total: number;
  status_counts: {
    pending: number;
    running: number;
    completed: number;
    failed: number;
    cancelled: number;
  };
  status: string;
}

/**
 * Get overall task statistics
 */
export async function getTasksStats(): Promise<TasksStats> {
  return apiGet<TasksStats>(`${API_BASE}/tasks/stats`);
}

/**
 * Task stats history data point
 */
export interface TaskStatsHistoryPoint {
  date: string;
  total: number;
  completed: number;
  in_progress: number;
  pending: number;
  failed: number;
  cancelled: number;
  completion_rate: number;
}

/**
 * Task stats history response
 */
export interface TaskStatsHistoryResponse {
  period: string;
  period_days: number;
  history: TaskStatsHistoryPoint[];
}

/**
 * Get task completion history for trend analysis
 */
export async function getTasksStatsHistory(params?: {
  period?: string; // e.g., "30d", "7d", "90d"
}): Promise<TaskStatsHistoryResponse> {
  const queryParams = new URLSearchParams();
  if (params?.period) queryParams.append("period", params.period);

  const queryString = queryParams.toString();
  const url = `${API_BASE}/tasks/stats/history${
    queryString ? `?${queryString}` : ""
  }`;
  return apiGet<TaskStatsHistoryResponse>(url);
}

/**
 * List all tasks
 * 
 * Validates task responses using runtime schema validation
 */
export async function listTasks(): Promise<TasksListResponse> {
  const response = await apiGet<TasksListResponse>(`${API_BASE}/tasks`);
  
  // Validate each task in the response
  if (response.tasks && Array.isArray(response.tasks)) {
    // Import validation utilities (dynamic import to avoid circular dependencies)
    const { safeValidateTaskArray } = await import('../utils/taskValidation');
    response.tasks = safeValidateTaskArray(response.tasks) as Task[];
  }
  
  return response;
}

