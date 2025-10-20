// Task System Types and Interfaces
// Defines the data structures for autonomous task execution and monitoring

export interface Task {
  id: string;
  working_spec_id: string;
  status: TaskStatus;
  phase: TaskPhase;
  priority: TaskPriority;
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
  title: string;
  description?: string;
  context?: TaskContext;
  progress: TaskProgress;
  artifacts: TaskArtifact[];
  quality_report?: QualityReport;
  error_message?: string;
  retry_count: number;
  max_retries: number;
}

export type TaskStatus =
  | "pending"
  | "running"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export type TaskPhase =
  | "planning"
  | "analysis"
  | "execution"
  | "validation"
  | "refinement"
  | "qa"
  | "finalization";

export type TaskPriority = "low" | "medium" | "high" | "critical";

export interface TaskContext {
  goals: string[];
  constraints: string[];
  resources: TaskResource[];
  working_spec: WorkingSpec;
  repository_context?: RepositoryContext;
  agent_assignments: AgentAssignment[];
}

export interface TaskResource {
  type: "file" | "url" | "data" | "tool";
  id: string;
  name: string;
  path?: string;
  url?: string;
  size_bytes?: number;
  content_type?: string;
  checksum?: string;
}

export interface WorkingSpec {
  id: string;
  title: string;
  description: string;
  acceptance_criteria: string[];
  deliverables: string[];
  risk_tier: 1 | 2 | 3;
  mode: "feature" | "fix" | "refactor" | "chore";
}

export interface RepositoryContext {
  branch: string;
  commit_hash?: string;
  files_modified: string[];
  working_directory: string;
}

export interface AgentAssignment {
  agent_id: string;
  role: "primary" | "secondary" | "reviewer" | "validator";
  capabilities: string[];
  assigned_at: string;
  status: "assigned" | "working" | "completed" | "failed";
}

export interface TaskProgress {
  percentage: number; // 0-100
  current_step?: string;
  total_steps?: number;
  current_step_index?: number;
  estimated_completion?: string;
  time_elapsed_ms: number;
  time_remaining_ms?: number;
}

export interface TaskArtifact {
  id: string;
  type: ArtifactType;
  name: string;
  description?: string;
  created_at: string;
  size_bytes?: number;
  content_type?: string;
  url?: string;
  data?: any;
  metadata: Record<string, any>;
}

export type ArtifactType =
  | "code"
  | "test"
  | "documentation"
  | "design"
  | "data"
  | "log"
  | "report"
  | "binary";

export interface QualityReport {
  id: string;
  task_id: string;
  generated_at: string;
  overall_score: number; // 0-100
  criteria: QualityCriterion[];
  recommendations: string[];
  passed: boolean;
  review_required: boolean;
}

export interface QualityCriterion {
  name: string;
  description: string;
  score: number; // 0-100
  weight: number; // 0-1
  status: "passed" | "failed" | "warning" | "pending";
  details?: string;
  evidence?: any;
}

// SSE Event Types for Task Monitoring
export interface TaskEvent {
  type:
    | "task_created"
    | "task_updated"
    | "task_completed"
    | "task_failed"
    | "phase_changed"
    | "artifact_added"
    | "quality_checked";
  task_id: string;
  data: any;
  timestamp: string;
  event_id: string;
  sequence_number: number;
}

export interface TaskCreatedEvent {
  task: Task;
}

export interface TaskUpdatedEvent {
  task_id: string;
  changes: Partial<Task>;
  previous_values: Partial<Task>;
}

export interface TaskPhaseChangedEvent {
  task_id: string;
  previous_phase: TaskPhase;
  new_phase: TaskPhase;
  reason?: string;
}

export interface TaskArtifactAddedEvent {
  task_id: string;
  artifact: TaskArtifact;
}

export interface TaskQualityCheckedEvent {
  task_id: string;
  quality_report: QualityReport;
}

// Component Props Types
export interface TaskListProps {
  tasks?: Task[];
  isLoading?: boolean;
  onTaskSelect?: (task: Task) => void;
  onTaskFilter?: (filters: TaskFilters) => void;
  selectedTaskId?: string;
}

export interface TaskCardProps {
  task: Task;
  isSelected?: boolean;
  showDetails?: boolean;
  onClick?: () => void;
  onPause?: () => void;
  onResume?: () => void;
  onCancel?: () => void;
}

export interface TaskTimelineProps {
  task: Task;
  showArtifacts?: boolean;
  onArtifactClick?: (artifact: TaskArtifact) => void;
  onPhaseClick?: (phase: TaskPhase) => void;
}

export interface ExecutionPhaseViewerProps {
  task: Task;
  currentPhase?: TaskPhase;
  onPhaseSelect?: (phase: TaskPhase) => void;
}

export interface WorkingSpecViewerProps {
  spec: WorkingSpec;
  task?: Task;
  onEdit?: () => void;
}

export interface TaskFilters {
  status?: TaskStatus[];
  phase?: TaskPhase[];
  priority?: TaskPriority[];
  agent_id?: string;
  working_spec_id?: string;
  date_range?: {
    start: string;
    end: string;
  };
}

// API Response Types
export interface GetTasksResponse {
  tasks: Task[];
  total_count: number;
  filtered_count: number;
  page: number;
  page_size: number;
  filters_applied: TaskFilters;
}

export interface GetTaskResponse {
  task: Task;
  events?: TaskEvent[];
  artifacts?: TaskArtifact[];
  quality_report?: QualityReport;
}

export interface TaskActionRequest {
  action: "pause" | "resume" | "cancel" | "restart";
  reason?: string;
}

export interface TaskActionResponse {
  task: Task;
  action_performed: string;
  timestamp: string;
}

// Error Types
export interface TaskError {
  code:
    | "task_not_found"
    | "invalid_action"
    | "permission_denied"
    | "server_error";
  message: string;
  task_id?: string;
  retryable: boolean;
}
