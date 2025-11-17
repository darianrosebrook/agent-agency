/**
 * Projects API Client
 *
 * Provides functions for fetching and managing projects from the v3 API.
 *
 * @author @darianrosebrook
 */

import type { TaskWithOptionalDescription } from "../types/task";
import { apiDelete, apiGet, apiPatch, apiPost } from "../utils/api";

/**
 * Project API response
 */
export interface ProjectApiResponse {
  project_id: string;
  id?: string; // Alias for project_id for compatibility
  name: string;
  overview?: string | null;
  summary?: string | null; // Alias for overview
  description?: string | null; // May come from overview or metadata
  state?: string | null;
  working_spec_id?: string | null;
  milestones?: Array<{
    id?: string;
    milestone_id?: string;
    title: string;
    description?: string | null;
    completed?: boolean;
    state?: string;
    due_date?: string | null;
    created_at?: string;
    updated_at?: string;
  }>;
  tasks?: Array<{
    id?: string;
    task_id?: string;
    title: string;
    description?: string | null;
    status: string;
    priority?: number | null;
    assignee?: string | null;
    assigned_worker_id?: string | null;
    created_at: string;
    updated_at?: string;
    completed_at?: string | null;
  }>;
  dependency_graph?: unknown;
  change_budget?: unknown;
  quality_gates?: unknown;
  evidence_requirements?: unknown;
  active_waivers?: unknown;
  metadata?: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
  last_accessed?: string; // May not exist, use updated_at as fallback
  approved_at?: string | null;
  completed_at?: string | null;
}

/**
 * Project milestone
 */
export interface ProjectMilestone {
  id?: string; // API returns 'id' not 'milestone_id' in some cases
  milestone_id?: string;
  project_id?: string;
  plan_id?: string; // API returns 'plan_id' in some cases
  title?: string; // API may return 'objective' instead
  objective?: string; // API returns 'objective' for milestone title
  description?: string | null;
  due_date?: string | null;
  completed?: boolean;
  state?: string; // API returns 'state' (pending, in_progress, completed)
  priority?: string;
  risk_tier?: number;
  is_blocking?: boolean;
  estimated_effort?: number;
  started_at?: string | null;
  completed_at?: string | null;
  created_at: string;
  updated_at: string;
}

/**
 * Project member
 */
export interface ProjectMember {
  member_id: string;
  project_id: string;
  user_id: string;
  role: string;
  joined_at: string;
}

/**
 * Project settings
 */
export interface ProjectSettings {
  project_id: string;
  settings: Record<string, unknown>;
  [key: string]: unknown;
}

/**
 * Task-specific settings for projects
 */
export interface ProjectTaskSettings {
  project_id: string;
  default_status: string;
  auto_archive: boolean;
  auto_archive_days: number;
  max_concurrent_tasks?: number;
  priority_inheritance?: boolean;
  assignee_required?: boolean;
  enable_dependencies?: boolean;
  require_description?: boolean;
}

const API_BASE = "/api/proxy/api/v1";

/**
 * Project list item from API (snake_case)
 */
export interface ProjectListItem {
  project_id: string;
  name: string;
  overview?: string | null;
  state?: string | null;
  created_at: string;
  updated_at: string;
  completed_at?: string | null;
}

/**
 * Projects list response
 */
export interface ProjectsListResponse {
  projects: ProjectListItem[];
}

/**
 * Project overview version
 */
export interface ProjectOverviewVersion {
  version_id: string;
  project_id: string;
  overview: string;
  created_at: string;
  created_by?: string | null;
  change_summary?: string | null;
}

/**
 * Project overview versions response
 */
export interface ProjectOverviewVersionsResponse {
  versions: ProjectOverviewVersion[];
}

/**
 * List all projects
 */
export async function listProjects(): Promise<ProjectsListResponse> {
  return apiGet<ProjectsListResponse>(`${API_BASE}/projects`);
}

/**
 * Create a new project
 */
export async function createProject(request: {
  name: string;
  summary?: string;
  description?: string;
}): Promise<ProjectApiResponse> {
  return apiPost<ProjectApiResponse>(`${API_BASE}/projects`, request);
}

/**
 * Get project details
 */
export async function getProjectHandler(
  projectId: string
): Promise<ProjectApiResponse> {
  return apiGet<ProjectApiResponse>(`${API_BASE}/projects/${projectId}`);
}

/**
 * Delete project
 */
export async function deleteProject(projectId: string): Promise<void> {
  return apiDelete<void>(`${API_BASE}/projects/${projectId}`);
}

/**
 * Update project details
 */
export async function updateProjectHandler(
  projectId: string,
  updates: {
    name?: string;
    description?: string;
    summary?: string;
  }
): Promise<ProjectApiResponse> {
  return apiPatch<ProjectApiResponse>(
    `${API_BASE}/projects/${projectId}`,
    updates
  );
}

/**
 * Get project members
 */
export async function getProjectMembers(
  projectId: string
): Promise<ProjectMember[]> {
  try {
    const response = await apiGet<{ members: ProjectMember[] }>(
      `${API_BASE}/projects/${projectId}/members`
    );
    return response.members || [];
  } catch (err) {
    console.error("Failed to get project members:", err);
    return [];
  }
}

/**
 * Project milestones response
 */
export interface ProjectMilestonesResponse {
  milestones: ProjectMilestone[];
}

/**
 * Get project milestones
 */
export async function getProjectMilestones(
  projectId: string
): Promise<ProjectMilestone[]> {
  // API returns { milestones: [...] }
  const response = await apiGet<ProjectMilestonesResponse>(
    `${API_BASE}/projects/${projectId}/milestones`
  );
  // Extract milestones array from response
  return Array.isArray(response) ? response : response.milestones || [];
}

/**
 * Create project milestone
 */
export async function createProjectMilestone(
  projectId: string,
  milestone: {
    title: string;
    description?: string;
    due_date?: string;
  }
): Promise<ProjectMilestone> {
  return apiPost<ProjectMilestone>(
    `${API_BASE}/projects/${projectId}/milestones`,
    milestone
  );
}

/**
 * Update project milestone
 */
export async function updateProjectMilestone(
  projectId: string,
  milestoneId: string,
  updates: Partial<
    Pick<ProjectMilestone, "title" | "description" | "due_date" | "completed">
  >
): Promise<ProjectMilestone> {
  return apiPatch<ProjectMilestone>(
    `${API_BASE}/projects/${projectId}/milestones/${milestoneId}`,
    updates
  );
}

/**
 * Project task from API
 *
 * Extends canonical Task interface with backward compatibility fields
 * Matches backend Task model from iterations/v3/data-infrastructure/src/models.rs
 */
export interface ProjectTask extends TaskWithOptionalDescription {
  // Backward compatibility: some APIs may return task_id instead of id
  task_id?: string; // Keep for backward compatibility, prefer id
}

/**
 * Project tasks response
 */
export interface ProjectTasksResponse {
  tasks: ProjectTask[];
}

/**
 * Get project tasks
 *
 * Validates task responses using runtime schema validation.
 * Maps API response to ensure id field is present (handles both id and task_id from API)
 */
export async function getProjectTasks(
  projectId: string
): Promise<ProjectTasksResponse> {
  const response = await apiGet<ProjectTasksResponse>(
    `${API_BASE}/projects/${projectId}/tasks`
  );

  // Validate and normalize tasks
  if (response.tasks && Array.isArray(response.tasks)) {
    const { safeValidateTaskArray } = await import("../utils/taskValidation");
    const { normalizeTaskId } = await import("../utils/taskTransform");

    // Validate each task
    response.tasks = safeValidateTaskArray(response.tasks) as unknown as ProjectTask[];

    // Normalize task IDs (some APIs return task_id instead of id)
    response.tasks = response.tasks.map((task) => ({
      ...task,
      id: normalizeTaskId(task),
    }));
  }

  return response;
}

/**
 * Create project task
 */
export async function createProjectTask(
  projectId: string,
  task: {
    title: string;
    description?: string;
    status?: string;
    priority?: number;
    risk_tier?: string;
  }
): Promise<ProjectTask> {
  return apiPost<ProjectTask>(`${API_BASE}/projects/${projectId}/tasks`, {
    ...task,
    metadata: {
      project_id: projectId,
    },
  });
}

/**
 * Update project task
 *
 * Supports updating all 14 backend task fields.
 */
export async function updateProjectTask(
  projectId: string,
  taskId: string,
  updates: Partial<{
    title?: string;
    description?: string;
    risk_tier?: string;
    scope?: Record<string, unknown>;
    acceptance_criteria?: unknown[];
    context?: Record<string, unknown>;
    caws_spec?: Record<string, unknown> | null;
    status?:
      | "pending"
      | "in_progress"
      | "paused"
      | "completed"
      | "cancelled"
      | "failed";
    assigned_worker_id?: string | null;
    project_id?: string | null;
    priority?: number | null;
    deadline?: string | null;
    metadata?: Record<string, unknown> | null;
    completed_at?: string | null;
  }>
): Promise<ProjectTask> {
  return apiPatch<ProjectTask>(
    `${API_BASE}/projects/${projectId}/tasks/${taskId}`,
    updates
  );
}

/**
 * Delete project task
 */
export async function deleteProjectTask(
  projectId: string,
  taskId: string
): Promise<void> {
  return apiDelete<void>(`${API_BASE}/projects/${projectId}/tasks/${taskId}`);
}

/**
 * Update project overview/description
 */
export async function updateProjectOverview(
  projectId: string,
  overview: string
): Promise<ProjectApiResponse> {
  return apiPatch<ProjectApiResponse>(`${API_BASE}/projects/${projectId}`, {
    overview,
  });
}

/**
 * Get project overview versions/history
 */
export async function getProjectOverviewVersions(
  projectId: string,
  limit?: number
): Promise<ProjectOverviewVersionsResponse> {
  const queryParams = new URLSearchParams();
  if (limit) queryParams.append("limit", limit.toString());

  const queryString = queryParams.toString();
  const url = `${API_BASE}/projects/${projectId}/overview-versions${
    queryString ? `?${queryString}` : ""
  }`;
  return apiGet<ProjectOverviewVersionsResponse>(url);
}

/**
 * Restore project overview from version
 */
export async function restoreProjectOverviewVersion(
  projectId: string,
  versionId: string
): Promise<ProjectApiResponse> {
  return apiPost<ProjectApiResponse>(
    `${API_BASE}/projects/${projectId}/overview-versions/${versionId}/restore`,
    {}
  );
}

/**
 * Get project task statistics
 */
export interface ProjectTaskStats {
  total: number;
  completed: number;
  in_progress: number;
  pending: number;
  cancelled: number;
  failed: number;
}

export async function getProjectTaskStats(
  projectId: string
): Promise<ProjectTaskStats> {
  return apiGet<ProjectTaskStats>(
    `${API_BASE}/projects/${projectId}/tasks/stats`
  );
}

/**
 * Work history entry
 */
export interface WorkHistoryEntry {
  entry_id: string;
  project_id: string;
  task_id?: string | null;
  action: string;
  description?: string | null;
  created_at: string;
  created_by?: string | null;
}

/**
 * Work history response
 */
export interface WorkHistoryResponse {
  entries: WorkHistoryEntry[];
  total: number;
}

/**
 * Get project work history
 */
export async function getProjectWorkHistory(
  projectId: string,
  params?: {
    limit?: number;
    offset?: number;
  }
): Promise<WorkHistoryEntry[]> {
  try {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.set("limit", params.limit.toString());
    if (params?.offset) queryParams.set("offset", params.offset.toString());

    const url = `${API_BASE}/projects/${projectId}/work-history${
      queryParams.toString() ? `?${queryParams.toString()}` : ""
    }`;
    const response = await apiGet<WorkHistoryResponse>(url);
    return response.entries || [];
  } catch (err) {
    console.error("Failed to get project work history:", err);
    return [];
  }
}

/**
 * Get project settings
 */
export async function getProjectSettings(
  projectId: string
): Promise<ProjectSettings> {
  return apiGet<ProjectSettings>(`${API_BASE}/projects/${projectId}/settings`);
}

/**
 * Update project settings
 */
export async function updateProjectSettings(
  projectId: string,
  settings: Partial<ProjectSettings>
): Promise<ProjectSettings> {
  return apiPatch<ProjectSettings>(
    `${API_BASE}/projects/${projectId}/settings`,
    settings
  );
}

/**
 * Get project task settings
 */
export async function getProjectTaskSettings(
  projectId: string
): Promise<ProjectTaskSettings> {
  return apiGet<ProjectTaskSettings>(
    `${API_BASE}/projects/${projectId}/task-settings`
  );
}

/**
 * Update project task settings
 */
export async function updateProjectTaskSettings(
  projectId: string,
  settings: Partial<ProjectTaskSettings>
): Promise<ProjectTaskSettings> {
  return apiPatch<ProjectTaskSettings>(
    `${API_BASE}/projects/${projectId}/task-settings`,
    settings
  );
}
