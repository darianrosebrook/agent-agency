/**
 * Tasks API Client
 * 
 * Provides functions for fetching task data and statistics from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet } from '../utils/api';

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
 * Get overall task statistics
 */
export async function getTasksStats(): Promise<TasksStats> {
  return apiGet<TasksStats>(`${API_BASE}/tasks/stats`);
}

