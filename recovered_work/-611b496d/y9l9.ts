// Task API client for the web dashboard
import { ApiClient } from "./api-client";
import type {
  Task,
  TaskSubmissionRequest,
  TaskSubmissionResponse,
  TaskListResponse,
  TaskListFilters,
  TaskMetrics,
  AuditLogEntry,
  ApiError,
} from "@/types/tasks";

export class TaskApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl?: string) {
    this.apiClient = new ApiClient({
      baseUrl: baseUrl ?? "/api/v1/tasks",
    });
  }

  /**
   * Submit a new task for execution
   */
  async submitTask(taskData: TaskSubmissionRequest): Promise<TaskSubmissionResponse> {
    try {
      const response = await this.apiClient.request<TaskSubmissionResponse>("/", {
        method: "POST",
        body: JSON.stringify(taskData),
      });
      return response;
    } catch (error) {
      console.error("Failed to submit task:", error);
      throw error;
    }
  }

  /**
   * Get a list of tasks with optional filtering
   */
  async getTasks(filters?: TaskListFilters): Promise<TaskListResponse> {
    try {
      const queryParams = new URLSearchParams();
      
      if (filters) {
        if (filters.status) {
          filters.status.forEach(status => queryParams.append("status", status));
        }
        if (filters.phase) {
          filters.phase.forEach(phase => queryParams.append("phase", phase));
        }
        if (filters.priority) {
          filters.priority.forEach(priority => queryParams.append("priority", priority));
        }
        if (filters.created_after) {
          queryParams.append("created_after", filters.created_after);
        }
        if (filters.created_before) {
          queryParams.append("created_before", filters.created_before);
        }
        if (filters.search) {
          queryParams.append("search", filters.search);
        }
      }

      const url = queryParams.toString() ? `/?${queryParams.toString()}` : "/";
      const response = await this.apiClient.request<TaskListResponse>(url, {
        method: "GET",
      });
      return response;
    } catch (error) {
      console.error("Failed to get tasks:", error);
      throw error;
    }
  }

  /**
   * Get a specific task by ID
   */
  async getTask(taskId: string): Promise<Task> {
    try {
      const response = await this.apiClient.request<Task>(`/${taskId}`, {
        method: "GET",
      });
      return response;
    } catch (error) {
      console.error(`Failed to get task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Get task metrics and statistics
   */
  async getTaskMetrics(): Promise<TaskMetrics> {
    try {
      const response = await this.apiClient.request<TaskMetrics>("/metrics", {
        method: "GET",
      });
      return response;
    } catch (error) {
      console.error("Failed to get task metrics:", error);
      throw error;
    }
  }

  /**
   * Get audit trail for a specific task
   */
  async getTaskAuditTrail(taskId: string): Promise<AuditLogEntry[]> {
    try {
      const response = await this.apiClient.request<AuditLogEntry[]>(`/${taskId}/audit`, {
        method: "GET",
      });
      return response;
    } catch (error) {
      console.error(`Failed to get audit trail for task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Pause a running task
   */
  async pauseTask(taskId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.apiClient.request<{ success: boolean; message: string }>(`/${taskId}/pause`, {
        method: "POST",
      });
      return response;
    } catch (error) {
      console.error(`Failed to pause task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Resume a paused task
   */
  async resumeTask(taskId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.apiClient.request<{ success: boolean; message: string }>(`/${taskId}/resume`, {
        method: "POST",
      });
      return response;
    } catch (error) {
      console.error(`Failed to resume task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Cancel a task
   */
  async cancelTask(taskId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.apiClient.request<{ success: boolean; message: string }>(`/${taskId}/cancel`, {
        method: "POST",
      });
      return response;
    } catch (error) {
      console.error(`Failed to cancel task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Retry a failed task
   */
  async retryTask(taskId: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await this.apiClient.request<{ success: boolean; message: string }>(`/${taskId}/retry`, {
        method: "POST",
      });
      return response;
    } catch (error) {
      console.error(`Failed to retry task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Get task artifacts
   */
  async getTaskArtifacts(taskId: string): Promise<any[]> {
    try {
      const response = await this.apiClient.request<any[]>(`/${taskId}/artifacts`, {
        method: "GET",
      });
      return response;
    } catch (error) {
      console.error(`Failed to get artifacts for task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Download a task artifact
   */
  async downloadArtifact(taskId: string, artifactId: string): Promise<Blob> {
    try {
      const response = await fetch(`/api/v1/tasks/${taskId}/artifacts/${artifactId}/download`);
      if (!response.ok) {
        throw new Error(`Failed to download artifact: ${response.statusText}`);
      }
      return await response.blob();
    } catch (error) {
      console.error(`Failed to download artifact ${artifactId} for task ${taskId}:`, error);
      throw error;
    }
  }

  /**
   * Get real-time task updates via WebSocket
   */
  createTaskUpdateWebSocket(taskId: string): WebSocket {
    const wsUrl = new URL(`/api/v1/tasks/${taskId}/ws`, window.location.origin);
    wsUrl.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    
    const ws = new WebSocket(wsUrl.toString());
    
    ws.onopen = () => {
      console.log(`WebSocket connected for task ${taskId}`);
    };
    
    ws.onerror = (error) => {
      console.error(`WebSocket error for task ${taskId}:`, error);
    };
    
    ws.onclose = () => {
      console.log(`WebSocket disconnected for task ${taskId}`);
    };
    
    return ws;
  }

  /**
   * Get all tasks with real-time updates via WebSocket
   */
  createTaskListWebSocket(): WebSocket {
    const wsUrl = new URL("/api/v1/tasks/ws", window.location.origin);
    wsUrl.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    
    const ws = new WebSocket(wsUrl.toString());
    
    ws.onopen = () => {
      console.log("WebSocket connected for task list updates");
    };
    
    ws.onerror = (error) => {
      console.error("WebSocket error for task list:", error);
    };
    
    ws.onclose = () => {
      console.log("WebSocket disconnected for task list");
    };
    
    return ws;
  }
}