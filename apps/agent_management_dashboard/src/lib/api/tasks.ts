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
 * Task list item
 */
export interface Task {
  id: string;
  title: string;
  description?: string;
  priority?: string;
  type?: string;
  status: string;
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string | null;
  worker_id?: string | null;
  metadata?: Record<string, unknown>;
}

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
 * List all tasks
 */
export async function listTasks(): Promise<TasksListResponse> {
  return apiGet<TasksListResponse>(`${API_BASE}/tasks`);
}

