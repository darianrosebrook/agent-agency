// Projects API endpoints
import { serverApi } from "./server";
import type { Project, Task } from "@/types";

export const projectsApi = {
  async listProjects(): Promise<Project[]> {
    return serverApi.get<Project[]>("/api/v1/projects");
  },

  async getProject(id: string): Promise<Project> {
    return serverApi.get<Project>(`/api/v1/projects/${id}`);
  },

  async getProjectTasks(id: string): Promise<Task[]> {
    return serverApi.get<Task[]>(`/api/v1/projects/${id}/tasks`);
  },

  async createProject(project: Partial<Project>): Promise<Project> {
    return serverApi.post<Project>("/api/v1/projects", project);
  },
};
