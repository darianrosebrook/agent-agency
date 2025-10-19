/**
 * Observer API Client
 *
 * Client for connecting to the Arbiter Observer HTTP API endpoints.
 * Provides methods for all observer operations with proper error handling.
 */

import type {
  ArbiterControlResult,
  ChainOfThoughtFilters,
  ChainOfThoughtListResult,
  CommandResult,
  EventFilters,
  EventListResult,
  ObservationResult,
  ObserverMetricsSnapshot,
  ObserverProgressSummary,
  ObserverStatusSummary,
  SubmitTaskPayload,
  SubmitTaskResult,
  Task,
} from "@/types/api";

export class ObserverApiError extends Error {
  constructor(
    message: string,
    public status?: number,
    public response?: unknown
  ) {
    super(message);
    this.name = "ObserverApiError";
  }
}

export class ObserverApiClient {
  private baseUrl: string;
  private authToken?: string;

  constructor(baseUrl = "http://127.0.0.1:4389", authToken?: string) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.authToken = authToken;
  }

  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const headers = new Headers(options.headers);

    headers.set("Accept", "application/json");

    if (!headers.has("Content-Type") && options.body) {
      headers.set("Content-Type", "application/json");
    }

    if (this.authToken) {
      headers.set("Authorization", `Bearer ${this.authToken}`);
    }

    const response = await fetch(url, { ...options, headers });

    if (!response.ok) {
      const errorText = await response.text();
      let errorMessage = `HTTP ${response.status}: ${response.statusText}`;

      try {
        const errorData = JSON.parse(errorText);
        errorMessage = errorData.error || errorMessage;
      } catch {
        // Use error text if not JSON
        if (errorText) {
          errorMessage = errorText;
        }
      }

      throw new ObserverApiError(errorMessage, response.status, errorText);
    }

    const text = await response.text();
    if (!text) {
      return {} as T;
    }

    try {
      return JSON.parse(text);
    } catch (error) {
      throw new ObserverApiError(`Invalid JSON response: ${text}`);
    }
  }

  private buildQueryString(params: Record<string, unknown>): string {
    const filtered = Object.entries(params).filter(
      ([, value]) => value !== undefined && value !== null
    );

    if (filtered.length === 0) {
      return "";
    }

    const searchParams = new URLSearchParams();
    filtered.forEach(([key, value]) => {
      searchParams.set(key, String(value));
    });

    return `?${searchParams.toString()}`;
  }

  // Status and Metrics
  async getStatus(): Promise<ObserverStatusSummary> {
    return this.request<ObserverStatusSummary>("/observer/status");
  }

  async getMetrics(): Promise<ObserverMetricsSnapshot> {
    return this.request<ObserverMetricsSnapshot>("/observer/metrics");
  }

  async getProgress(): Promise<ObserverProgressSummary> {
    return this.request<ObserverProgressSummary>("/observer/progress");
  }

  async getDiagnostics(): Promise<any> {
    return this.request("/observer/diagnostics");
  }

  async getAgents(): Promise<any> {
    return this.request("/observer/agents");
  }

  // Events
  async getEvents(filters: EventFilters = {}): Promise<EventListResult> {
    const query = this.buildQueryString({
      cursor: filters.cursor,
      limit: filters.limit,
      severity: filters.severity,
      type: filters.type,
      taskId: filters.taskId,
      sinceTs: filters.sinceTs,
      untilTs: filters.untilTs,
    });

    return this.request<EventListResult>(`/observer/logs${query}`);
  }

  // Chain of Thought
  async getChainOfThought(
    filters: ChainOfThoughtFilters = {}
  ): Promise<ChainOfThoughtListResult> {
    const query = this.buildQueryString({
      cursor: filters.cursor,
      limit: filters.limit,
      taskId: filters.taskId,
      since: filters.since,
    });

    return this.request<ChainOfThoughtListResult>(`/observer/cot${query}`);
  }

  async getTaskChainOfThought(
    taskId: string,
    filters: Omit<ChainOfThoughtFilters, "taskId"> = {}
  ): Promise<ChainOfThoughtListResult> {
    const query = this.buildQueryString({
      cursor: filters.cursor,
      limit: filters.limit,
      since: filters.since,
    });

    return this.request<ChainOfThoughtListResult>(
      `/observer/tasks/${encodeURIComponent(taskId)}/cot${query}`
    );
  }

  // Tasks
  async getTask(taskId: string): Promise<Task | null> {
    try {
      return await this.request<Task>(
        `/observer/tasks/${encodeURIComponent(taskId)}`
      );
    } catch (error) {
      if (error instanceof ObserverApiError && error.status === 404) {
        return null;
      }
      throw error;
    }
  }

  async submitTask(payload: SubmitTaskPayload): Promise<SubmitTaskResult> {
    return this.request<SubmitTaskResult>("/observer/tasks", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  }

  // Arbiter Control
  async startArbiter(): Promise<ArbiterControlResult> {
    return this.request<ArbiterControlResult>("/observer/arbiter/start", {
      method: "POST",
    });
  }

  async stopArbiter(): Promise<ArbiterControlResult> {
    return this.request<ArbiterControlResult>("/observer/arbiter/stop", {
      method: "POST",
    });
  }

  async executeCommand(command: string): Promise<CommandResult> {
    return this.request<CommandResult>("/observer/commands", {
      method: "POST",
      body: JSON.stringify({ command: command.trim() }),
    });
  }

  // Observations
  async addObservation(
    message: string,
    taskId?: string,
    author?: string
  ): Promise<ObservationResult> {
    const payload: Record<string, string> = { message };
    if (taskId) payload.taskId = taskId;
    if (author) payload.author = author;

    return this.request<ObservationResult>("/observer/observations", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  }

  // Server-Sent Events connection (for real-time streaming)
  connectToEventStream(
    filters: {
      taskId?: string;
      type?: string;
      severity?: "debug" | "info" | "warn" | "error";
      verbose?: boolean;
    } = {}
  ): EventSource {
    const query = this.buildQueryString({
      taskId: filters.taskId,
      type: filters.type,
      severity: filters.severity,
      verbose: filters.verbose ? "1" : undefined,
    });

    const url = `${this.baseUrl}/observer/events/stream${query}`;

    const eventSource = new EventSource(url);

    // Add authorization header if token is available
    if (this.authToken) {
      // Note: EventSource doesn't support custom headers directly
      // We'll need to handle auth differently for SSE, or use a proxy
      console.warn(
        "EventSource does not support Authorization headers. SSE auth may not work."
      );
    }

    return eventSource;
  }
}
