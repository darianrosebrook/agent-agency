/**
 * Canonical Task Type Definitions
 * 
 * Single source of truth for Task interface matching backend Task model
 * from iterations/v3/data-infrastructure/src/models.rs
 * 
 * @author @darianrosebrook
 */

import type { BackendTaskStatus } from '../utils/taskStatus';

/**
 * Canonical Task interface matching backend exactly
 * 
 * All fields match backend Task struct:
 * - id: UUID (string in frontend)
 * - title: String (required)
 * - description: String (required in backend, optional in frontend for backward compatibility)
 * - risk_tier: String
 * - scope: JSONB (Record<string, unknown>)
 * - acceptance_criteria: JSONB (unknown[])
 * - context: JSONB (Record<string, unknown>)
 * - caws_spec: Optional JSONB
 * - status: BackendTaskStatus enum
 * - assigned_worker_id: Optional UUID
 * - project_id: Optional UUID
 * - priority: Optional i32 (0-10)
 * - deadline: Optional DateTime<Utc> (RFC3339 string)
 * - created_at: DateTime<Utc> (required, RFC3339 string)
 * - updated_at: DateTime<Utc> (required, RFC3339 string)
 * - completed_at: Optional DateTime<Utc> (RFC3339 string)
 * - metadata: Optional JSONB
 */
export interface Task {
  id: string;
  title: string;
  description: string; // Required in backend, but optional in frontend for backward compatibility
  risk_tier: string;
  scope: Record<string, unknown>;
  acceptance_criteria: unknown[];
  context: Record<string, unknown>;
  caws_spec?: Record<string, unknown> | null;
  status: BackendTaskStatus;
  assigned_worker_id?: string | null;
  project_id?: string | null;
  priority?: number | null; // 0-10
  deadline?: string | null; // RFC3339
  created_at: string; // RFC3339, required
  updated_at: string; // RFC3339, required
  completed_at?: string | null; // RFC3339
  metadata?: Record<string, unknown> | null;
}

/**
 * Task with optional description for backward compatibility
 * 
 * Use this when description might be missing from API responses
 * that haven't been updated yet. The backend requires description,
 * but some frontend code may still handle it as optional.
 */
export interface TaskWithOptionalDescription extends Omit<Task, 'description'> {
  description?: string | null; // Optional for backward compatibility, but backend requires it
}

/**
 * Task update request payload
 * 
 * All fields are optional except those required by backend
 */
export interface UpdateTaskRequest {
  title?: string;
  description?: string;
  risk_tier?: string;
  scope?: Record<string, unknown>;
  acceptance_criteria?: unknown[];
  context?: Record<string, unknown>;
  caws_spec?: Record<string, unknown> | null;
  status?: BackendTaskStatus;
  assigned_worker_id?: string | null;
  project_id?: string | null;
  priority?: number | null; // 0-10
  deadline?: string | null; // RFC3339
  metadata?: Record<string, unknown> | null;
  completed_at?: string | null; // RFC3339
}

/**
 * Task creation request payload
 * 
 * Required fields match backend CreateTask struct
 */
export interface CreateTaskRequest {
  title: string;
  description: string; // Required
  risk_tier: string;
  scope: Record<string, unknown>;
  acceptance_criteria: unknown[];
  context: Record<string, unknown>;
  caws_spec?: Record<string, unknown> | null;
  status?: BackendTaskStatus; // Defaults to "pending" if not provided
  assigned_worker_id?: string | null;
  project_id?: string | null;
  priority?: number | null; // 0-10
  deadline?: string | null; // RFC3339
  metadata?: Record<string, unknown> | null;
}

