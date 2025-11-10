/**
 * Projects API Client
 * 
 * Provides functions for fetching and managing projects from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet, apiPost, apiPatch, apiDelete } from '../utils/api';

/**
 * Project member
 */
export interface ProjectMember {
  id: string;
  user_id: string;
  user_name: string;
  user_email: string;
  role: string;
  added_at: string;
}

/**
 * Project milestone
 */
export interface ProjectMilestone {
  id: string;
  project_id: string;
  title: string;
  description: string | null;
  due_date: string | null;
  completed: boolean;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

/**
 * Project task statistics
 */
export interface ProjectTaskStats {
  total: number;
  completed: number;
  in_progress: number;
  pending: number;
  failed: number;
  completion_rate: number;
}

/**
 * Project work history entry
 */
export interface WorkHistoryEntry {
  id: string;
  project_id: string;
  task_id: string | null;
  agent_id: string;
  action: string;
  description: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

/**
 * Project file
 */
export interface ProjectFile {
  id: string;
  project_id: string;
  filename: string;
  path: string;
  size: number;
  mime_type: string;
  uploaded_by: string;
  created_at: string;
  updated_at: string;
}

/**
 * Project timeline entry
 */
export interface TimelineEntry {
  id: string;
  project_id: string;
  type: 'task' | 'milestone' | 'file' | 'member';
  title: string;
  description: string | null;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

/**
 * Project settings
 */
export interface ProjectSettings {
  default_assignee_id?: string | null;
  auto_assign_tasks?: boolean;
  notification_preferences?: Record<string, unknown>;
  [key: string]: unknown;
}

const API_BASE = '/api/proxy/api/v1';

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
 * Project details from API (snake_case)
 */
export interface ProjectApiResponse {
  id: string;
  name: string;
  summary?: string | null;
  description?: string | null;
  created_at: string;
  last_accessed: string;
  milestones?: Array<{
    id: string;
    title: string;
    completed: boolean;
  }>;
  tasks?: Array<{
    id: string;
    title: string;
    description?: string | null;
    status: string;
    priority?: string | null;
    assignee?: string | null;
    created_at: string;
  }>;
}

/**
 * List all projects
 */
export async function listProjects(): Promise<ProjectsListResponse> {
  return apiGet<ProjectsListResponse>(`${API_BASE}/projects`);
}

/**
 * Get project details
 */
export async function getProjectHandler(projectId: string): Promise<ProjectApiResponse> {
  return apiGet<ProjectApiResponse>(`${API_BASE}/projects/${projectId}`);
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
  return apiPatch<ProjectApiResponse>(`${API_BASE}/projects/${projectId}`, updates);
}

/**
 * Get project members
 */
export async function getProjectMembers(projectId: string): Promise<ProjectMember[]> {
  return apiGet<ProjectMember[]>(`${API_BASE}/projects/${projectId}/members`);
}

/**
 * Add project member
 */
export async function addProjectMember(
  projectId: string,
  userId: string,
  role: string
): Promise<ProjectMember> {
  return apiPost<ProjectMember>(`${API_BASE}/projects/${projectId}/members`, {
    user_id: userId,
    role,
  });
}

/**
 * Remove project member
 */
export async function removeProjectMember(
  projectId: string,
  memberId: string
): Promise<void> {
  return apiDelete<void>(`${API_BASE}/projects/${projectId}/members/${memberId}`);
}

/**
 * Get project milestones
 */
export async function getProjectMilestones(projectId: string): Promise<ProjectMilestone[]> {
  return apiGet<ProjectMilestone[]>(`${API_BASE}/projects/${projectId}/milestones`);
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
  return apiPost<ProjectMilestone>(`${API_BASE}/projects/${projectId}/milestones`, milestone);
}

/**
 * Update project milestone
 */
export async function updateProjectMilestone(
  projectId: string,
  milestoneId: string,
  updates: Partial<Pick<ProjectMilestone, 'title' | 'description' | 'due_date' | 'completed'>>
): Promise<ProjectMilestone> {
  return apiPatch<ProjectMilestone>(`${API_BASE}/projects/${projectId}/milestones/${milestoneId}`, updates);
}

/**
 * Project task from API
 */
export interface ProjectTask {
  task_id: string;
  title: string;
  description?: string | null;
  status: string;
  risk_tier?: string | null;
  priority?: number | null;
  created_at: string;
  updated_at: string;
  completed_at?: string | null;
}

/**
 * Project tasks response
 */
export interface ProjectTasksResponse {
  tasks: ProjectTask[];
}

/**
 * Get project tasks
 */
export async function getProjectTasks(projectId: string): Promise<ProjectTasksResponse> {
  return apiGet<ProjectTasksResponse>(`${API_BASE}/projects/${projectId}/tasks`);
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
 * Get project task statistics
 */
export async function getProjectTaskStats(projectId: string): Promise<ProjectTaskStats> {
  return apiGet<ProjectTaskStats>(`${API_BASE}/projects/${projectId}/tasks/stats`);
}

/**
 * Get project work history
 */
export async function getProjectWorkHistory(
  projectId: string,
  params?: {
    limit?: number;
    offset?: number;
    start_date?: string;
    end_date?: string;
  }
): Promise<WorkHistoryEntry[]> {
  const queryParams = new URLSearchParams();
  if (params?.limit) queryParams.append('limit', params.limit.toString());
  if (params?.offset) queryParams.append('offset', params.offset.toString());
  if (params?.start_date) queryParams.append('start_date', params.start_date);
  if (params?.end_date) queryParams.append('end_date', params.end_date);
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/projects/${projectId}/work-history${queryString ? `?${queryString}` : ''}`;
  return apiGet<WorkHistoryEntry[]>(url);
}

/**
 * Get project files
 */
export async function getProjectFiles(projectId: string): Promise<ProjectFile[]> {
  return apiGet<ProjectFile[]>(`${API_BASE}/projects/${projectId}/files`);
}

/**
 * Upload project file
 */
export async function uploadProjectFile(
  projectId: string,
  file: File,
  path?: string
): Promise<ProjectFile> {
  const formData = new FormData();
  formData.append('file', file);
  if (path) formData.append('path', path);
  
  const response = await fetch(`${API_BASE}/projects/${projectId}/files/upload`, {
    method: 'POST',
    body: formData,
  });
  
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new Error(error.error || `Failed to upload file: ${response.statusText}`);
  }
  
  return response.json();
}

/**
 * Delete project file
 */
export async function deleteProjectFile(
  projectId: string,
  fileId: string
): Promise<void> {
  return apiDelete<void>(`${API_BASE}/projects/${projectId}/files/${fileId}`);
}

/**
 * Get project timeline
 */
export async function getProjectTimeline(
  projectId: string,
  params?: {
    limit?: number;
    offset?: number;
  }
): Promise<TimelineEntry[]> {
  const queryParams = new URLSearchParams();
  if (params?.limit) queryParams.append('limit', params.limit.toString());
  if (params?.offset) queryParams.append('offset', params.offset.toString());
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/projects/${projectId}/tasks/timeline${queryString ? `?${queryString}` : ''}`;
  return apiGet<TimelineEntry[]>(url);
}

/**
 * Get project settings
 */
export async function getProjectSettings(projectId: string): Promise<ProjectSettings> {
  return apiGet<ProjectSettings>(`${API_BASE}/projects/${projectId}/settings`);
}

/**
 * Update project settings
 */
export async function updateProjectSettings(
  projectId: string,
  settings: Partial<ProjectSettings>
): Promise<ProjectSettings> {
  return apiPatch<ProjectSettings>(`${API_BASE}/projects/${projectId}/settings`, settings);
}

/**
 * Get project task settings
 */
export async function getProjectTaskSettings(projectId: string): Promise<ProjectSettings> {
  return apiGet<ProjectSettings>(`${API_BASE}/projects/${projectId}/settings/tasks`);
}

/**
 * Update project task settings
 */
export async function updateProjectTaskSettings(
  projectId: string,
  settings: Partial<ProjectSettings>
): Promise<ProjectSettings> {
  return apiPatch<ProjectSettings>(`${API_BASE}/projects/${projectId}/settings/tasks`, settings);
}

