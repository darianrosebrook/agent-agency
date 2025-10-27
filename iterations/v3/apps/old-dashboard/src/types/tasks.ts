// Task-related type definitions for the web dashboard
// Synchronized with Rust backend API structures

export interface Task {
  task_id: string;
  status: string;
  progress_percentage: number;
  current_phase?: string;
  started_at?: string;
  updated_at?: string;
  quality_score?: number;
  description?: string;
  priority?: string;
  working_spec?: WorkingSpec;
  artifacts?: ExecutionArtifacts;
  quality_report?: QualityReport;
  completed_at?: string;
  error_message?: string;
}

// Working spec type from Rust backend
export interface WorkingSpec {
  id: string;
  title: string;
  description?: string;
  risk_tier?: string;
  mode?: string;
  change_budget?: ChangeBudget;
  blast_radius?: BlastRadius;
  operational_rollback_slo?: string;
  scope?: TaskScope;
  invariants?: string[];
  acceptance?: AcceptanceCriterion[];
  non_functional?: NonFunctionalRequirements;
}

export interface ChangeBudget {
  max_files: number;
  max_loc: number;
}

export interface BlastRadius {
  modules: string[];
  data_migration: boolean;
}

export interface TaskScope {
  in: string[];
  out: string[];
}

export interface AcceptanceCriterion {
  id: string;
  given: string;
  when: string;
  then: string;
}

export interface NonFunctionalRequirements {
  a11y?: string[];
  perf?: PerformanceRequirements;
  security?: string[];
}

export interface PerformanceRequirements {
  api_p95_ms?: number;
  lcp_ms?: number;
}

export interface ExecutionArtifacts {
  files_created?: string[];
  files_modified?: string[];
  files_deleted?: string[];
  total_changes?: number;
  git_commit_hash?: string;
}

export interface TaskContext {
  goals: string[];
  constraints: string[];
  requirements?: string[];
  environment?: string;
  dependencies?: string[];
}

export interface TaskProgress {
  percentage: number;
  current_step: string;
  steps_completed: number;
  total_steps: number;
  estimated_completion?: string;
}

export interface QualityReport {
  overall_score: number;
  passed: boolean;
  details?: Record<string, any>;
  checks_performed: string[];
  recommendations?: string[];
}

export interface TaskArtifact {
  id: string;
  name: string;
  type: string;
  description?: string;
  size: number;
  created_at: string;
  url?: string;
  metadata?: Record<string, any>;
}

export interface AuditLogEntry {
  id: string;
  action: string;
  actor?: string;
  timestamp: string;
  change_summary?: any;
  resource_id?: string;
  resource_type?: string;
}

// API Request/Response types matching Rust backend
export interface TaskSubmissionRequest {
  description: string;
  execution_mode?: string;
  risk_tier?: string;
  context?: string;
  priority?: string;
  deadline?: string;
}

export interface TaskSubmissionResponse {
  task_id: string;
  status: string;
  message: string;
  estimated_completion?: string;
}

export interface TaskListResponse {
  tasks: Task[];
  total?: number;
  has_more?: boolean;
}

export interface TaskListFilters {
  status?: string[];
  phase?: string[];
  priority?: string[];
  created_after?: string;
  created_before?: string;
  search?: string;
}

export interface TaskMetrics {
  total_tasks: number;
  active_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  average_completion_time: number;
  success_rate: number;
}

// API Error types
export interface ApiError {
  error: string;
  message: string;
  details?: any;
  timestamp: string;
}

// WebSocket message types for real-time updates
export interface TaskUpdateMessage {
  type: "task_update";
  task_id: string;
  status: string;
  progress_percentage: number;
  current_phase?: string;
  timestamp: string;
}

export interface TaskCompletionMessage {
  type: "task_completion";
  task_id: string;
  status: string;
  quality_report?: QualityReport;
  error_message?: string;
  timestamp: string;
}

export interface TaskProgressMessage {
  type: "task_progress";
  task_id: string;
  progress_percentage: number;
  current_phase?: string;
  timestamp: string;
}

// System health WebSocket messages
export interface SystemHealthMessage {
  type: "health_update" | "component_update" | "alert_created" | "alert_updated" | "metrics_update";
  data: any;
  timestamp: string;
}

// SSE message types
export interface SSEMessage {
  type: string;
  data: any;
  id?: string;
  timestamp: string;
}

export type WebSocketMessage =
  | TaskUpdateMessage
  | TaskCompletionMessage
  | TaskProgressMessage
  | SystemHealthMessage;

// Dashboard-specific types
export interface TaskDashboardStats {
  total_tasks: number;
  active_tasks: number;
  completed_today: number;
  failed_today: number;
  average_completion_time: number;
  success_rate: number;
  top_phases: Array<{
    phase: Task["phase"];
    count: number;
    percentage: number;
  }>;
  recent_activity: Array<{
    task_id: string;
    action: string;
    timestamp: string;
    actor?: string;
  }>;
}

export interface TaskTimelineEvent {
  id: string;
  type: "task_created" | "task_started" | "task_completed" | "task_failed" | "task_paused" | "task_cancelled";
  task_id: string;
  timestamp: string;
  actor?: string;
  details?: any;
}

// Form types for task creation and editing
export interface TaskFormData {
  title: string;
  description: string;
  working_spec_id: string;
  priority: Task["priority"];
  context: {
    goals: string[];
    constraints: string[];
    requirements: string[];
    environment: string;
    dependencies: string[];
  };
  max_retries: number;
}

// Filter and search types
export interface TaskSearchParams {
  query?: string;
  status?: Task["status"][];
  phase?: Task["phase"][];
  priority?: Task["priority"][];
  date_range?: {
    start: string;
    end: string;
  };
  sort_by?: "created_at" | "updated_at" | "priority" | "status";
  sort_order?: "asc" | "desc";
  page?: number;
  page_size?: number;
}

// Export all types for easy importing
// export type {
//   Task,
//   TaskContext,
//   TaskProgress,
//   QualityReport,
//   TaskArtifact,
//   AuditLogEntry,
//   TaskSubmissionRequest,
//   TaskSubmissionResponse,
//   TaskListResponse,
//   TaskListFilters,
//   TaskMetrics,
//   ApiError,
//   TaskUpdateMessage,
//   TaskCompletionMessage,
//   TaskProgressMessage,
//   WebSocketMessage,
//   TaskDashboardStats,
//   TaskTimelineEvent,
//   TaskFormData,
//   TaskSearchParams,
// };