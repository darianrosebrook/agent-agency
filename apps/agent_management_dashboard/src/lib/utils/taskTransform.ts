/**
 * Task Transformation Utilities
 * 
 * Utilities for transforming between API Task format and UI-specific formats.
 * 
 * @author @darianrosebrook
 */

import type { Task, TaskWithOptionalDescription } from '../types/task';
import type { ProjectTask } from '../api/projects';

/**
 * Convert API Task to UI Task format (with Date objects)
 * 
 * Used by ProjectContext and other UI components that need Date objects
 * instead of RFC3339 strings.
 */
export interface UITask {
  id: string;
  title: string;
  description?: string;
  status: Task['status'];
  priority?: number | null;
  assigned_worker_id?: string | null;
  createdAt: Date;
  // Include other fields as needed
  risk_tier?: string;
  scope?: Record<string, unknown>;
  acceptance_criteria?: unknown[];
  context?: Record<string, unknown>;
}

/**
 * Convert API Task to UI Task format
 * 
 * @param apiTask - Task from API (with RFC3339 strings)
 * @returns UI Task (with Date objects)
 */
export function apiTaskToUITask(apiTask: Task | TaskWithOptionalDescription | ProjectTask): UITask {
  return {
    id: apiTask.id || (apiTask as ProjectTask).task_id || '',
    title: apiTask.title,
    description: apiTask.description ?? undefined,
    status: apiTask.status,
    priority: apiTask.priority ?? null,
    assigned_worker_id: apiTask.assigned_worker_id ?? null,
    createdAt: new Date(apiTask.created_at),
    risk_tier: apiTask.risk_tier,
    scope: apiTask.scope,
    acceptance_criteria: apiTask.acceptance_criteria,
    context: apiTask.context,
  };
}

/**
 * Convert UI Task to API Task format
 * 
 * @param uiTask - UI Task (with Date objects)
 * @returns API Task (with RFC3339 strings)
 */
export function uiTaskToApiTask(uiTask: UITask, apiTask: Partial<Task> = {}): Partial<Task> {
  return {
    ...apiTask,
    id: uiTask.id,
    title: uiTask.title,
    description: uiTask.description ?? '',
    status: uiTask.status,
    priority: uiTask.priority ?? null,
    assigned_worker_id: uiTask.assigned_worker_id ?? null,
    created_at: uiTask.createdAt.toISOString(),
    risk_tier: uiTask.risk_tier ?? apiTask.risk_tier,
    scope: uiTask.scope ?? apiTask.scope,
    acceptance_criteria: uiTask.acceptance_criteria ?? apiTask.acceptance_criteria,
    context: uiTask.context ?? apiTask.context,
  };
}

/**
 * Normalize task ID from API response
 * 
 * Handles backward compatibility where some APIs return `task_id` instead of `id`
 * 
 * @param task - Task from API
 * @returns Normalized task ID
 */
export function normalizeTaskId(task: { id?: string; task_id?: string }): string {
  return task.id || task.task_id || '';
}

/**
 * Normalize task from API response
 * 
 * Ensures all required fields are present and normalized
 * 
 * @param task - Task from API (may have task_id instead of id)
 * @returns Normalized task with id field
 */
export function normalizeTask(
  task: TaskWithOptionalDescription | ProjectTask
): TaskWithOptionalDescription {
  return {
    ...task,
    id: normalizeTaskId(task),
    description: task.description ?? '', // Backend requires description, provide default
  };
}

/**
 * Validate task has required fields
 * 
 * @param task - Task to validate
 * @returns True if task has all required fields
 */
export function isValidTask(task: unknown): task is Task {
  if (!task || typeof task !== 'object') {
    return false;
  }

  const t = task as Partial<Task>;

  return (
    typeof t.id === 'string' &&
    typeof t.title === 'string' &&
    typeof t.status === 'string' &&
    typeof t.created_at === 'string' &&
    typeof t.updated_at === 'string'
  );
}

/**
 * Create a safe task object with defaults
 * 
 * Useful for handling incomplete task data from API
 * 
 * @param partial - Partial task data
 * @returns Task with defaults filled in
 */
export function createSafeTask(
  partial: Partial<TaskWithOptionalDescription>
): TaskWithOptionalDescription {
  const now = new Date().toISOString();

  return {
    id: partial.id || '',
    title: partial.title || 'Untitled Task',
    description: partial.description ?? '',
    status: partial.status || 'pending',
    risk_tier: partial.risk_tier || '',
    scope: partial.scope || {},
    acceptance_criteria: partial.acceptance_criteria || [],
    context: partial.context || {},
    created_at: partial.created_at || now,
    updated_at: partial.updated_at || now,
    ...partial,
  };
}


