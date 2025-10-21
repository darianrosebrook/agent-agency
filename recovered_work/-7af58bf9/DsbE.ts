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
  async request<T>(
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
  /* eslint-disable no-unused-vars */
  async getTasks(filters?: Record<string, unknown>): Promise<{
    tasks: Array<{
      id: string;
      title: string;
      status: string;
      priority: string;
      createdAt: string;
      updatedAt: string;
    }>;
    total: number;
    page: number;
    limit: number;
  }> {
    try {
      const params = new URLSearchParams();
      if (filters) {
        Object.entries(filters).forEach(([key, value]) => {
          if (value !== undefined && value !== null) {
            params.append(key, String(value));
          }
        });
      }

      const response = await this.request<{
        tasks: Array<{
          id: string;
          title: string;
          status: string;
          priority: string;
          createdAt: string;
          updatedAt: string;
        }>;
        total: number;
        page: number;
        limit: number;
      }>(`/api/v1/tasks?${params}`);

      return response;
    } catch (error) {
      console.error("Failed to get tasks:", error);
      throw error;
    }
  }

  async getTask(_taskId: string): Promise<{
    id: string;
    title: string;
    description: string;
    status: string;
    priority: string;
    createdAt: string;
    updatedAt: string;
    workingSpec?: {
      id: string;
      title: string;
      riskTier: number;
      acceptance: Array<{
        id: string;
        given: string;
        when: string;
        then: string;
      }>;
    };
    artifacts?: Array<{
      id: string;
      type: string;
      size: number;
      createdAt: string;
    }>;
    qualityReport?: {
      coverage: number;
      mutationScore: number;
      lintErrors: number;
      typeErrors: number;
    };
  }> {
    console.warn(
      "getTask using mock implementation - V3 task detail API not available"
    );
    // TODO: Milestone 2 - Task Detail API Implementation
    // - [ ] Implement V3 GET /api/v1/tasks/:id endpoint
    // - [ ] Include working spec, artifacts, quality report
    // - [ ] Test with various task states

    // Mock implementation for development
    return {
      id: _taskId,
      title: "Implement user authentication flow",
      description:
        "Create a secure user authentication system with JWT tokens, password hashing, and session management.",
      status: "in_progress",
      priority: "high",
      createdAt: "2025-01-15T10:00:00Z",
      updatedAt: "2025-01-22T11:45:00Z",
      workingSpec: {
        id: "AUTH-001",
        title: "User Authentication Implementation",
        riskTier: 1,
        acceptance: [
          {
            id: "A1",
            given: "User is not logged in",
            when: "User submits valid credentials",
            then: "User is authenticated and redirected to dashboard",
          },
        ],
      },
      artifacts: [
        {
          id: "artifact_001",
          type: "unit_tests",
          size: 245760,
          createdAt: "2025-01-20T14:30:00Z",
        },
        {
          id: "artifact_002",
          type: "linting",
          size: 15360,
          createdAt: "2025-01-20T14:35:00Z",
        },
      ],
      qualityReport: {
        coverage: 85.5,
        mutationScore: 72.3,
        lintErrors: 0,
        typeErrors: 0,
      },
    };
  }

  async createChatSession(): Promise<{ sessionId: string; createdAt: string }> {
    console.warn(
      "createChatSession using mock implementation - V3 backend not available"
    );
    // TODO: Milestone 1 - Chat Session Management
    // - [ ] Implement V3 POST /api/v1/chat/session endpoint
    // - [ ] Add session persistence and cleanup
    // - [ ] Test session creation and retrieval

    // Mock implementation for development
    return {
      sessionId: `session_${Date.now()}_${Math.random()
        .toString(36)
        .substr(2, 9)}`,
      createdAt: new Date().toISOString(),
    };
  }

  async sendChatMessage(
    _sessionId: string,
    _message: string
  ): Promise<{ messageId: string; response: string; timestamp: string }> {
    console.warn(
      "sendChatMessage using mock implementation - V3 WebSocket API not available"
    );
    // TODO: Milestone 1 - Chat Message Handling
    // - [ ] Implement V3 WebSocket /api/v1/chat/ws/:session_id
    // - [ ] Add message routing and intent parsing
    // - [ ] Integrate with planning agent and orchestrator
    // - [ ] Test real-time message exchange

    // Mock implementation for development
    // Simulate processing delay
    await new Promise((resolve) =>
      setTimeout(resolve, 1000 + Math.random() * 2000)
    );

    const responses = [
      "I understand your request. Let me help you with that.",
      "That's an interesting point. Here's what I can tell you...",
      "Based on the current system state, I recommend the following approach:",
      "I've analyzed your query and here's the most relevant information:",
      "Let me break this down for you step by step:",
    ];

    return {
      messageId: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      response: responses[Math.floor(Math.random() * responses.length)],
      timestamp: new Date().toISOString(),
    };
  }

  async getDatabaseTables(): Promise<unknown> {
    console.warn(
      "getDatabaseTables not implemented - requires V3 database API"
    );
    // TODO: Milestone 4 - Database API Implementation
    // - [ ] Implement V3 GET /api/v1/database/tables endpoint
    // - [ ] Add table schema and metadata endpoints
    // - [ ] Test with all target tables
    throw new Error("Database tables API not yet implemented");
  }

  async queryDatabase(
    _table: string,
    _filters: Record<string, unknown>
  ): Promise<unknown> {
    console.warn(
      "queryDatabase not implemented - requires V3 database query service"
    );
    // TODO: Milestone 4 - Database Query Service
    // - [ ] Implement V3 query_service.rs with read-only queries
    // - [ ] Add POST /api/v1/database/tables/:table/query endpoint
    // - [ ] Enforce safety constraints and rate limiting
    // - [ ] Test with various query patterns
    throw new Error("Database query API not yet implemented");
  }
  /* eslint-enable no-unused-vars */

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
