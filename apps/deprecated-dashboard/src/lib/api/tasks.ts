// Tasks API endpoints
import { serverApi } from "./server";
import type {
  Task,
  TaskExecution,
  ChainOfThought,
  CouncilDecisions,
  WorkerActions,
  TaskLogs,
  TaskProgress,
} from "@/types";

export const tasksApi = {
  async listTasks(): Promise<Task[]> {
    return serverApi.get<Task[]>("/api/v1/tasks");
  },

  async getTask(id: string): Promise<Task> {
    return serverApi.get<Task>(`/api/v1/tasks/${id}`);
  },

  async getTaskStatus(id: string): Promise<{ status: string }> {
    return serverApi.get<{ status: string }>(`/api/v1/tasks/${id}/status`);
  },

  async getChainOfThought(id: string): Promise<ChainOfThought> {
    return serverApi.get<ChainOfThought>(
      `/api/v1/tasks/${id}/chain-of-thought`
    );
  },

  async getCouncilDecisions(id: string): Promise<CouncilDecisions> {
    return serverApi.get<CouncilDecisions>(
      `/api/v1/tasks/${id}/council-decisions`
    );
  },

  async getWorkerActions(id: string): Promise<WorkerActions> {
    return serverApi.get<WorkerActions>(`/api/v1/tasks/${id}/worker-actions`);
  },

  async getTaskLogs(id: string): Promise<TaskLogs> {
    return serverApi.get<TaskLogs>(`/api/v1/tasks/${id}/logs`);
  },

  async getTaskProgress(id: string): Promise<TaskProgress> {
    return serverApi.get<TaskProgress>(`/api/v1/tasks/${id}/progress`);
  },

  async getTaskEvents(id: string): Promise<unknown[]> {
    return serverApi.get<unknown[]>(`/api/v1/tasks/${id}/events`);
  },

  async getTaskResult(id: string): Promise<TaskExecution> {
    return serverApi.get<TaskExecution>(`/api/v1/tasks/${id}/result`);
  },

  async requestPause(id: string): Promise<void> {
    await serverApi.post(`/api/v1/tasks/${id}/pause`);
  },

  async requestResume(id: string): Promise<void> {
    await serverApi.post(`/api/v1/tasks/${id}/resume`);
  },

  async requestCancel(id: string): Promise<void> {
    await serverApi.post(`/api/v1/tasks/${id}/cancel`);
  },

  async submitTask(task: Partial<Task>): Promise<Task> {
    return serverApi.post<Task>("/api/v1/tasks", task);
  },
};
