import { apiClient } from "@/lib/api-client";
import {
  Task,
  GetTasksResponse,
  GetTaskResponse,
  TaskActionRequest,
  TaskActionResponse,
  TaskFilters,
  TaskEvent,
  TaskArtifact,
  QualityReport,
  TaskError,
} from "@/types/tasks";

// Task API Client
// Handles REST API calls for task management and monitoring

export class TaskApiError extends Error {
  constructor(
    public code: TaskError["code"],
    message: string,
    public retryable: boolean = false
  ) {
    super(message);
    this.name = "TaskApiError";
  }
}

export class TaskApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl ?? "/api/tasks";
  }

  // Get list of tasks with optional filtering
  async getTasks(
    filters?: TaskFilters,
    page: number = 1,
    pageSize: number = 20,
    sortBy: "created_at" | "updated_at" | "priority" = "updated_at",
    sortOrder: "asc" | "desc" = "desc"
  ): Promise<GetTasksResponse> {
    try {
      const params = new URLSearchParams();

      // Convert page/pageSize to offset/limit for backend compatibility
      const offset = (page - 1) * pageSize;
      params.append("limit", pageSize.toString());
      params.append("offset", offset.toString());
      params.append("sort_by", sortBy);
      params.append("sort_order", sortOrder);

      // Add filters to query params
      if (filters) {
        if (filters.status?.length) {
          filters.status.forEach((status) => params.append("status", status));
        }
        if (filters.phase?.length) {
          filters.phase.forEach((phase) => params.append("phase", phase));
        }
        if (filters.priority?.length) {
          filters.priority.forEach((priority) =>
            params.append("priority", priority)
          );
        }
        if (filters.working_spec_id) {
          params.append("working_spec_id", filters.working_spec_id);
        }
        if (filters.date_range) {
          if (filters.date_range.start) {
            params.append("start_date", filters.date_range.start);
          }
          if (filters.date_range.end) {
            params.append("end_date", filters.date_range.end);
          }
        }
      }

      const response = await apiClient.request<GetTasksResponse>(
        `${this.baseUrl}?${params}`
      );

      return response;
    } catch (error) {
      console.error("Failed to get tasks:", error);
      throw new TaskApiError("server_error", "Failed to retrieve tasks", true);
    }
  }

  // Get detailed information about a specific task
  async getTask(taskId: string): Promise<GetTaskResponse> {
    try {
      const response = await apiClient.request<GetTaskResponse>(
        `/tasks/${encodeURIComponent(taskId)}`
      );

      return response;
    } catch (error) {
      console.error("Failed to get task:", error);
      if (error instanceof Error && error.message.includes("404")) {
        throw new TaskApiError("task_not_found", "Task not found", false);
      }
      throw new TaskApiError(
        "server_error",
        "Failed to retrieve task details",
        true
      );
    }
  }

  // Get task events (for timeline/history)
  async getTaskEvents(
    taskId: string,
    limit: number = 100,
    before?: string
  ): Promise<TaskEvent[]> {
    try {
      const params = new URLSearchParams({
        limit: limit.toString(),
      });
      if (before) {
        params.append("before", before);
      }

      const response = await apiClient.request<{ events: TaskEvent[] }>(
        `/tasks/${encodeURIComponent(taskId)}/events?${params}`
      );

      return response.events;
    } catch (error) {
      console.error("Failed to get task events:", error);
      throw new TaskApiError(
        "server_error",
        "Failed to retrieve task events",
        true
      );
    }
  }

  // Get task artifacts
  async getTaskArtifacts(taskId: string): Promise<TaskArtifact[]> {
    try {
      const response = await apiClient.request<{ artifacts: TaskArtifact[] }>(
        `/tasks/${encodeURIComponent(taskId)}/artifacts`
      );

      return response.artifacts;
    } catch (error) {
      console.error("Failed to get task artifacts:", error);
      throw new TaskApiError(
        "server_error",
        "Failed to retrieve task artifacts",
        true
      );
    }
  }

  // Get task quality report
  async getTaskQualityReport(taskId: string): Promise<QualityReport> {
    try {
      const response = await apiClient.request<{
        quality_report: QualityReport;
      }>(`/tasks/${encodeURIComponent(taskId)}/quality`);

      return response.quality_report;
    } catch (error) {
      console.error("Failed to get task quality report:", error);
      throw new TaskApiError(
        "server_error",
        "Failed to retrieve quality report",
        true
      );
    }
  }

  // Perform action on task (pause, resume, cancel, restart)
  async performTaskAction(
    taskId: string,
    action: TaskActionRequest
  ): Promise<TaskActionResponse> {
    try {
      const response = await apiClient.request<TaskActionResponse>(
        `/tasks/${encodeURIComponent(taskId)}/action`,
        {
          method: "POST",
          body: JSON.stringify(action),
        }
      );

      return response;
    } catch (error) {
      console.error("Failed to perform task action:", error);
      throw new TaskApiError(
        "invalid_action",
        "Failed to perform task action",
        true
      );
    }
  }

  // Pause task
  async pauseTask(taskId: string, reason?: string): Promise<Task> {
    const response = await this.performTaskAction(taskId, {
      action: "pause",
      reason,
    });
    return response.task;
  }

  // Resume task
  async resumeTask(taskId: string, reason?: string): Promise<Task> {
    const response = await this.performTaskAction(taskId, {
      action: "resume",
      reason,
    });
    return response.task;
  }

  // Cancel task
  async cancelTask(taskId: string, reason?: string): Promise<Task> {
    const response = await this.performTaskAction(taskId, {
      action: "cancel",
      reason,
    });
    return response.task;
  }

  // Restart task
  async restartTask(taskId: string, reason?: string): Promise<Task> {
    const response = await this.performTaskAction(taskId, {
      action: "restart",
      reason,
    });
    return response.task;
  }

  // Create new task (for manual task initiation)
  async createTask(taskData: {
    working_spec_id: string;
    title: string;
    description?: string;
    priority?: Task["priority"];
    context?: Partial<Task["context"]>;
  }): Promise<Task> {
    try {
      const response = await apiClient.request<{ task: Task }>("/tasks", {
        method: "POST",
        body: JSON.stringify(taskData),
      });

      return response.task;
    } catch (error) {
      console.error("Failed to create task:", error);
      throw new TaskApiError("server_error", "Failed to create task", true);
    }
  }
}

// Default task API client instance
export const taskApiClient = new TaskApiClient();
