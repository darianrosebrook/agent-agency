/**
 * Tasks API Module
 * Handles all task-related API operations
 */

import { ApiClient } from '../../api-client';

export class TasksModule {
  constructor(private apiClient: ApiClient) {}

  async getTasks(filters?: {
    status?: string;
    priority?: string;
    limit?: number;
    offset?: number;
  }) {
    const params = new URLSearchParams();
    if (filters?.status) params.append('status', filters.status);
    if (filters?.priority) params.append('priority', filters.priority);
    if (filters?.limit) params.append('limit', filters.limit.toString());
    if (filters?.offset) params.append('offset', filters.offset.toString());
    
    const queryString = params.toString();
    return this.apiClient.request(`/api/tasks${queryString ? `?${queryString}` : ''}`);
  }

  async getTask(taskId: string) {
    return this.apiClient.request(`/api/tasks/${taskId}`);
  }

  async createTask(taskData: any) {
    return this.apiClient.request('/api/tasks', {
      method: 'POST',
      body: JSON.stringify(taskData),
    });
  }

  async updateTask(taskId: string, action: string, data?: any) {
    return this.apiClient.request(`/api/tasks/${taskId}/action`, {
      method: 'POST',
      body: JSON.stringify({ action, ...data }),
    });
  }

  async getTaskMetrics() {
    return this.apiClient.request('/api/tasks/metrics');
  }

  async getTaskEvents(taskId?: string) {
    const endpoint = taskId ? `/api/tasks/${taskId}/events` : '/api/tasks/events';
    return this.apiClient.request(endpoint);
  }
}

