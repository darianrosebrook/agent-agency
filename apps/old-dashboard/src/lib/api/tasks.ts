// Task API client for communicating with V3 backend task endpoints
// Provides typed interfaces for task operations and real-time updates

export interface Task {
  id: string;
  title: string;
  description: string;
  status: 'pending' | 'executing' | 'completed' | 'failed' | 'canceled';
  priority: 'low' | 'medium' | 'high' | 'urgent';
  createdAt: string;
  updatedAt: string;
  acceptanceCriteria?: string[];
  events?: TaskEvent[];
}

export interface TaskEvent {
  id: string;
  action: string;
  actor?: string;
  details?: Record<string, any>;
  timestamp: string;
}

export interface TaskSubmissionRequest {
  description: string;
  files?: string[];
  model?: string;
  max_iterations?: number;
  execution_mode?: string;
  quality_gates?: string[];
}

export interface TaskSubmissionResponse {
  task_id: string;
  status: string;
  message: string;
}

export interface TaskListResponse {
  tasks: Task[];
  total: number;
  page: number;
  limit: number;
}

export interface TaskFilters {
  status?: string[];
  priority?: string[];
  date_range?: {
    start: string;
    end: string;
  };
}

export class TaskApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = '/api') {
    this.baseUrl = baseUrl;
  }

  async submitTask(request: TaskSubmissionRequest): Promise<TaskSubmissionResponse> {
    const response = await fetch(`${this.baseUrl}/tasks`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Failed to submit task: ${response.statusText}`);
    }

    return response.json();
  }

  async getTask(taskId: string): Promise<Task> {
    const response = await fetch(`${this.baseUrl}/tasks/${taskId}`);

    if (!response.ok) {
      throw new Error(`Failed to get task: ${response.statusText}`);
    }

    return response.json();
  }

  async listTasks(filters?: TaskFilters): Promise<TaskListResponse> {
    const params = new URLSearchParams();

    if (filters?.status?.length) {
      params.append('status', filters.status.join(','));
    }

    if (filters?.priority?.length) {
      params.append('priority', filters.priority.join(','));
    }

    if (filters?.date_range) {
      params.append('start_date', filters.date_range.start);
      params.append('end_date', filters.date_range.end);
    }

    const queryString = params.toString();
    const url = `${this.baseUrl}/tasks${queryString ? `?${queryString}` : ''}`;

    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to list tasks: ${response.statusText}`);
    }

    return response.json();
  }

  async pauseTask(taskId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/tasks/${taskId}/pause`, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Failed to pause task: ${response.statusText}`);
    }
  }

  async resumeTask(taskId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/tasks/${taskId}/resume`, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Failed to resume task: ${response.statusText}`);
    }
  }

  async cancelTask(taskId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/tasks/${taskId}/cancel`, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Failed to cancel task: ${response.statusText}`);
    }
  }

  async getTaskEvents(taskId: string): Promise<TaskEvent[]> {
    const response = await fetch(`${this.baseUrl}/tasks/${taskId}/events`);

    if (!response.ok) {
      throw new Error(`Failed to get task events: ${response.statusText}`);
    }

    const data = await response.json();
    return data.events || [];
  }

  async getTaskProvenance(taskId: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/tasks/${taskId}/provenance`);

    if (!response.ok) {
      throw new Error(`Failed to get task provenance: ${response.statusText}`);
    }

    return response.json();
  }
}

// Arbiter verdict related interfaces
export interface ArbiterVerdict {
  verdict_id: string;
  task_id: string;
  decision: any;
  confidence: number;
  evidence_manifest?: any;
  waiver_reason?: string;
}

export interface ClaimVerificationData {
  task_id: string;
  claims: any[];
  verification_results: any[];
}

// Mock functions for arbiter operations
export async function getArbiterVerdict(taskId: string): Promise<ArbiterVerdict> {
  // This would proxy to the V3 backend
  return {
    verdict_id: `verdict_${taskId}`,
    task_id: taskId,
    decision: { approved: true },
    confidence: 0.85,
    evidence_manifest: {
      claims: [],
      verification_results: []
    }
  };
}

export async function getClaimVerificationData(taskId: string): Promise<ClaimVerificationData> {
  // This would proxy to the V3 backend
  return {
    task_id: taskId,
    claims: [],
    verification_results: []
  };
}

// Export singleton instance
export const taskApiClient = new TaskApiClient();
