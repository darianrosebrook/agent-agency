// API response types
import type { CouncilVerdict } from "./models";

export interface ApiResponse<T> {
  data?: T;
  error?: ApiError;
}

export interface ApiError {
  message: string;
  code?: string;
  details?: Record<string, unknown>;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface ChainOfThought {
  task_id: string;
  entries: ChainOfThoughtEntry[];
}

export interface ChainOfThoughtEntry {
  timestamp: string;
  actor: string;
  thought: string;
  context?: Record<string, unknown>;
}

export interface CouncilDecisions {
  task_id: string;
  verdicts: CouncilVerdict[];
  final_decision?: string;
}

export interface WorkerActions {
  task_id: string;
  actions: WorkerAction[];
}

export interface WorkerAction {
  id: string;
  worker_id: string;
  action_type: string;
  timestamp: string;
  details: Record<string, unknown>;
}

export interface TaskLogs {
  task_id: string;
  logs: LogEntry[];
}

export interface LogEntry {
  timestamp: string;
  level: "debug" | "info" | "warn" | "error";
  message: string;
  context?: Record<string, unknown>;
}

export interface TaskProgress {
  task_id: string;
  progress_percent: number;
  current_step?: string;
  estimated_completion?: string;
  metrics?: Record<string, unknown>;
}

export interface TaskAnalytics {
  total_tasks: number;
  completed: number;
  failed: number;
  in_progress?: number;
  paused?: number;
  success_rate: number | string;
  average_execution_time_ms?: number;
  average_chain_of_thought_entries?: number;
  average_council_decisions?: number;
  average_worker_actions?: number;
  tasks_by_status?: Record<string, number>;
  tasks_by_worker?: Record<string, number>;
}

export interface PerformanceAnalytics {
  average_response_time_ms: number;
  p95_response_time_ms: number;
  p99_response_time_ms: number;
  requests_per_second: number;
  error_rate: number;
  time_series?: Array<{
    timestamp: string;
    metrics: Record<string, number>;
  }>;
}

export interface SuccessRates {
  total_tasks?: number;
  completed?: number;
  failed?: number;
  overall_success_rate: number | string;
  success_rate?: string;
  success_rate_by_worker?: Record<string, number | string>;
  success_rate_by_task_type?: Record<string, number>;
  trend?: Array<{
    date: string;
    success_rate: number;
  }>;
}
