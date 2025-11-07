// Projects API endpoints
import { serverApi } from './server';
import type { Project, Task } from '@/types';

export const projectsApi = {
  async listProjects(): Promise<Project[]> {
    return apiClient.get<Project[]>('/api/v1/projects');
  },

  async getProject(id: string): Promise<Project> {
    return apiClient.get<Project>(`/api/v1/projects/${id}`);
  },

  async getProjectTasks(id: string): Promise<Task[]> {
    return apiClient.get<Task[]>(`/api/v1/projects/${id}/tasks`);
  },

  async createProject(project: Partial<Project>): Promise<Project> {
    return apiClient.post<Project>('/api/v1/projects', project);
  },
};

