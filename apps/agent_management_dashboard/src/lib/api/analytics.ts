/**
 * Analytics API Client
 * 
 * Provides functions for fetching analytics data from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet } from '../utils/api';

const API_BASE = '/api/proxy/api/v1';

/**
 * Task analytics response
 */
export interface TaskAnalytics {
  total_tasks: number;
  completed: number;
  failed: number;
  in_progress: number;
  paused: number;
  success_rate: string; // Formatted as "XX.XX%"
  average_chain_of_thought_entries: number;
  average_council_decisions: number;
  average_worker_actions: number;
}

/**
 * Performance analytics response
 */
export interface PerformanceAnalytics {
  average_task_duration_ms: number;
  completed_tasks_count: number;
  total_tasks: number;
  performance_score: number; // 0-100 scale
}

/**
 * Success rates response
 */
export interface SuccessRates {
  success_rate: string; // Formatted as "XX.XX%"
  total_tasks: number;
  completed: number;
  failed: number;
}

/**
 * Get task analytics
 */
export async function getTaskAnalytics(): Promise<TaskAnalytics> {
  return apiGet<TaskAnalytics>(`${API_BASE}/analytics/tasks`);
}

/**
 * Get performance analytics
 */
export async function getPerformanceAnalytics(): Promise<PerformanceAnalytics> {
  return apiGet<PerformanceAnalytics>(`${API_BASE}/analytics/performance`);
}

/**
 * Get success rates
 */
export async function getSuccessRates(): Promise<SuccessRates> {
  return apiGet<SuccessRates>(`${API_BASE}/analytics/success-rates`);
}

