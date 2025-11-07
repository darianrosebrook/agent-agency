// Data models matching Rust API structures
// Based on iterations/v3/data-infrastructure/src/models.rs

export type TaskStatus =
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'paused'
  | 'cancelled';

export interface Task {
  id: string;
  title: string;
  description: string;
  risk_tier: string;
  scope: Record<string, unknown>;
  acceptance_criteria: Record<string, unknown>;
  context: Record<string, unknown>;
  caws_spec?: Record<string, unknown>;
  status: TaskStatus;
  assigned_worker_id?: string;
  created_at: string;
  updated_at: string;
  completed_at?: string;
  priority?: number;
  deadline?: string;
  metadata?: Record<string, unknown>;
}

export interface TaskExecution {
  id: string;
  task_id: string;
  worker_id: string;
  execution_started_at: string;
  execution_completed_at?: string;
  execution_time_ms?: number;
  status: string;
  worker_output: Record<string, unknown>;
  self_assessment: Record<string, unknown>;
  metadata: Record<string, unknown>;
  error_message?: string;
  tokens_used?: number;
  created_at: string;
  updated_at?: string;
  execution_metadata?: Record<string, unknown>;
  result_data?: Record<string, unknown>;
}

export interface Project {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
  metadata?: Record<string, unknown>;
}

export interface Worker {
  id: string;
  name: string;
  worker_type: string;
  specialty?: string;
  model_name: string;
  endpoint: string;
  capabilities: Record<string, unknown>;
  performance_history: Record<string, unknown>;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface Judge {
  id: string;
  name: string;
  model_name: string;
  endpoint: string;
  weight: number;
  timeout_ms: number;
  optimization_target: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CouncilVerdict {
  id: string;
  task_id: string;
  judge_id: string;
  verdict: string;
  reasoning: string;
  confidence: number;
  created_at: string;
}

export interface ProvenanceEntry {
  id: string;
  task_id: string;
  action: string;
  actor: string;
  resource_id?: string;
  resource_type?: string;
  change_summary: string;
  timestamp: string;
  created_at: string;
  metadata: Record<string, unknown>;
}

export interface Waiver {
  id: string;
  title: string;
  reason: string;
  description: string;
  gates: string[];
  approved_by: string;
  impact_level: string;
  mitigation_plan: string;
  expires_at: string;
  created_at: string;
  updated_at: string;
  status: string;
  metadata: Record<string, unknown>;
}

export interface ExecutionPlan {
  id: string;
  session_id: string;
  working_spec_id: string;
  title: string;
  overview?: string;
  state: string;
  milestones: Record<string, unknown>;
  dependency_graph: Record<string, unknown>;
  change_budget: Record<string, unknown>;
  quality_gates: Record<string, unknown>;
  evidence_requirements: Record<string, unknown>;
  active_waivers: Record<string, unknown>;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
  approved_at?: string;
  completed_at?: string;
}

export interface Milestone {
  id: string;
  plan_id: string;
  objective: string;
  scope: Record<string, unknown>;
  interfaces: Record<string, unknown>[];
  tests: Record<string, unknown>[];
  evidence_gate: Record<string, unknown>;
  rollback_plan?: string;
  dependencies: Record<string, unknown>[];
  state: string;
  assigned_worker_id?: string;
  estimated_effort?: number;
  priority?: string;
  risk_tier?: number;
  is_blocking?: boolean;
  blocking_reason?: string;
  metrics?: Record<string, unknown>;
  started_at?: string;
  completed_at?: string;
  created_at: string;
  updated_at: string;
}

export interface SystemHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  components: Record<string, ComponentHealth>;
  timestamp: string;
}

export interface ComponentHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  message?: string;
  last_check?: string;
}

export interface SystemMetrics {
  cpu_usage?: number;
  memory_usage?: number;
  disk_usage?: number;
  network_io?: {
    bytes_sent: number;
    bytes_received: number;
  };
  timestamp: string;
}

export interface DatabaseTable {
  name: string;
  row_count?: number;
  size_bytes?: number;
}

export interface DatabaseTableSchema {
  name: string;
  columns: DatabaseColumn[];
  constraints?: DatabaseConstraint[];
}

export interface DatabaseColumn {
  name: string;
  type: string;
  nullable: boolean;
  default?: string;
}

export interface DatabaseConstraint {
  name: string;
  type: string;
  columns: string[];
}

export interface DatabaseQueryResult {
  columns: string[];
  rows: unknown[][];
  row_count: number;
  execution_time_ms?: number;
}

