// API Client for V3 Backend Communication
// Handles authentication, retries, error handling, and structured responses

export interface ApiConfig {
  baseUrl: string;
  timeout: number;
  maxRetries: number;
  retryDelay: number;
  authToken?: string;
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public response?: unknown,
    public url?: string
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export class ApiClient {
  private config: ApiConfig;

  constructor(config: Partial<ApiConfig> = {}) {
    this.config = {
      baseUrl:
        config.baseUrl ??
        (typeof window !== "undefined"
          ? "/api/proxy"
          : "http://localhost:8080"),
      timeout: config.timeout ?? 30000,
      maxRetries: config.maxRetries ?? 3,
      retryDelay: config.retryDelay ?? 1000,
      authToken: config.authToken,
    };
  }

  // Update configuration
  updateConfig(updates: Partial<ApiConfig>) {
    this.config = { ...this.config, ...updates };
  }

  // Generic request method with retry logic
  private async request<T>(
    path: string,
    options: RequestInit = {},
    retryCount = 0
  ): Promise<T> {
    const url = `${this.config.baseUrl}${path}`;

    // Prepare headers
    const headers = new Headers(options.headers);

    // Set default headers
    headers.set("Accept", "application/json");
    headers.set("Content-Type", "application/json");

    // Add authorization if available
    if (this.config.authToken) {
      headers.set("Authorization", `Bearer ${this.config.authToken}`);
    }

    // Add user agent
    headers.set("User-Agent", "web-dashboard/0.1.0");

    // Create AbortController for timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

    try {
      const response = await fetch(url, {
        ...options,
        headers,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      // Handle non-2xx responses
      if (!response.ok) {
        const errorText = await response.text();
        let errorMessage = `HTTP ${response.status}: ${response.statusText}`;

        try {
          const errorData = JSON.parse(errorText);
          errorMessage = errorData.error || errorData.message || errorMessage;
        } catch {
          // Use error text if not JSON
          if (errorText) {
            errorMessage = errorText;
          }
        }

        // Check if we should retry
        const shouldRetry = this.shouldRetry(response.status, retryCount);

        if (shouldRetry) {
          console.warn(
            `Request failed (${response.status}), retrying in ${
              this.config.retryDelay
            }ms... (${retryCount + 1}/${this.config.maxRetries})`
          );
          await this.delay(this.config.retryDelay * Math.pow(2, retryCount)); // Exponential backoff
          return this.request<T>(path, options, retryCount + 1);
        }

        throw new ApiError(response.status, errorMessage, errorText, url);
      }

      // Handle empty responses
      const text = await response.text();
      if (!text) {
        return {} as T;
      }

      // Parse JSON response
      try {
        return JSON.parse(text);
      } catch (parseError) {
        throw new ApiError(
          response.status,
          `Invalid JSON response: ${text.substring(0, 100)}...`,
          text,
          url
        );
      }
    } catch (error) {
      clearTimeout(timeoutId);

      // Handle timeout
      if (error instanceof Error && error.name === "AbortError") {
        const timeoutError = new ApiError(
          408,
          `Request timeout after ${this.config.timeout}ms`,
          undefined,
          url
        );

        if (retryCount < this.config.maxRetries) {
          console.warn(
            `Request timeout, retrying in ${this.config.retryDelay}ms... (${
              retryCount + 1
            }/${this.config.maxRetries})`
          );
          await this.delay(this.config.retryDelay * Math.pow(2, retryCount));
          return this.request<T>(path, options, retryCount + 1);
        }

        throw timeoutError;
      }

      // Handle network errors
      if (error instanceof Error && error.name === "TypeError") {
        const networkError = new ApiError(
          0,
          `Network error: ${error.message}`,
          undefined,
          url
        );

        if (retryCount < this.config.maxRetries) {
          console.warn(
            `Network error, retrying in ${this.config.retryDelay}ms... (${
              retryCount + 1
            }/${this.config.maxRetries})`
          );
          await this.delay(this.config.retryDelay * Math.pow(2, retryCount));
          return this.request<T>(path, options, retryCount + 1);
        }

        throw networkError;
      }

      // Re-throw ApiError instances
      if (error instanceof ApiError) {
        throw error;
      }

      // Wrap other errors
      throw new ApiError(
        0,
        error instanceof Error ? error.message : "Unknown error occurred",
        error,
        url
      );
    }
  }

  // Determine if a request should be retried based on status code
  private shouldRetry(status: number, retryCount: number): boolean {
    if (retryCount >= this.config.maxRetries) {
      return false;
    }

    // Retry on server errors, timeouts, and network issues
    const retryableStatuses = [408, 429, 500, 502, 503, 504];
    return retryableStatuses.includes(status);
  }

  // Utility delay function
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  // Health check
  async health(): Promise<{
    status: string;
    timestamp: string;
    dashboard: any;
    backend: any;
  }> {
    // Use direct API call for health check (not proxied)
    const response = await fetch("/api/health", {
      headers: { Accept: "application/json" },
      signal: AbortSignal.timeout(5000),
    });

    if (!response.ok) {
      throw new ApiError(
        response.status,
        `Health check failed: ${response.statusText}`
      );
    }

    return response.json();
  }

  // Placeholder methods for future V3 API endpoints
  // These will be implemented as the backend endpoints become available

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async getTasks(_filters?: Record<string, unknown>): Promise<unknown> {
    console.warn("getTasks not implemented - requires V3 task API endpoints");
    // TODO: Milestone 2 - Task API Implementation
    // - [ ] Implement V3 GET /api/v1/tasks endpoint with filtering
    // - [ ] Add pagination and sorting support
    // - [ ] Test with various filter combinations
    throw new Error("Task API not yet implemented");
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async getTask(_taskId: string): Promise<unknown> {
    console.warn("getTask not implemented - requires V3 task detail API");
    // TODO: Milestone 2 - Task Detail API Implementation
    // - [ ] Implement V3 GET /api/v1/tasks/:id endpoint
    // - [ ] Include working spec, artifacts, quality report
    // - [ ] Test with various task states
    throw new Error("Task detail API not yet implemented");
  }

  async createChatSession(): Promise<unknown> {
    console.warn("createChatSession not implemented - requires V3 chat API");
    // TODO: Milestone 1 - Chat Session Management
    // - [ ] Implement V3 POST /api/v1/chat/session endpoint
    // - [ ] Add session persistence and cleanup
    // - [ ] Test session creation and retrieval
    throw new Error("Chat session API not yet implemented");
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async sendChatMessage(_sessionId: string, _message: string): Promise<unknown> {
    console.warn("sendChatMessage not implemented - requires V3 chat WebSocket API");
    // TODO: Milestone 1 - Chat Message Handling
    // - [ ] Implement V3 WebSocket /api/v1/chat/ws/:session_id
    // - [ ] Add message routing and intent parsing
    // - [ ] Integrate with planning agent and orchestrator
    // - [ ] Test real-time message exchange
    throw new Error("Chat message API not yet implemented");
  }

  async getDatabaseTables(): Promise<unknown> {
    console.warn("getDatabaseTables not implemented - requires V3 database API");
    // TODO: Milestone 4 - Database API Implementation
    // - [ ] Implement V3 GET /api/v1/database/tables endpoint
    // - [ ] Add table schema and metadata endpoints
    // - [ ] Test with all target tables
    throw new Error("Database tables API not yet implemented");
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async queryDatabase(
    _table: string,
    _filters: Record<string, unknown>
  ): Promise<unknown> {
    console.warn("queryDatabase not implemented - requires V3 database query service");
    // TODO: Milestone 4 - Database Query Service
    // - [ ] Implement V3 query_service.rs with read-only queries
    // - [ ] Add POST /api/v1/database/tables/:table/query endpoint
    // - [ ] Enforce safety constraints and rate limiting
    // - [ ] Test with various query patterns
    throw new Error("Database query API not yet implemented");
  }

  async getMetrics(): Promise<unknown> {
    console.warn("getMetrics not implemented - requires V3 metrics streaming");
    // TODO: Milestone 3 - Metrics Streaming Implementation
    // - [ ] Implement V3 GET /metrics/stream SSE endpoint
    // - [ ] Add REST aggregation endpoints for agents, coordination, business
    // - [ ] Test real-time metrics updates
    throw new Error("Metrics API not yet implemented");
  }
}

// Default client instance
export const apiClient = new ApiClient();

// Types are exported inline with their declarations
