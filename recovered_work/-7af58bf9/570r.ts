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
    dashboard: {
      status: string;
      version: string;
      uptime: number;
      node_version: string;
    };
    backend: {
      status: string;
      url: string;
      response_time_ms: number;
      error?: string;
    };
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

  async getTask(taskId: string): Promise<{
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
    try {
      const response = await this.request<{
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
      }>(`/api/v1/tasks/${taskId}`);

      return response;
    } catch (error) {
      console.error("Failed to get task:", error);
      throw error;
    }
  }

  async createChatSession(): Promise<{ sessionId: string; createdAt: string }> {
    try {
      const response = await this.request<{ sessionId: string; createdAt: string }>(
        "/api/v1/chat/session",
        {
          method: "POST",
          body: JSON.stringify({}),
        }
      );
      return response;
    } catch (error) {
      console.error("Failed to create chat session:", error);
      throw error;
    }
  }

  async sendChatMessage(
    sessionId: string,
    message: string
  ): Promise<{ messageId: string; response: string; timestamp: string }> {
    try {
      // For now, use HTTP POST instead of WebSocket for simplicity
      // TODO: Upgrade to WebSocket when real-time messaging is needed
      const response = await this.request<{ messageId: string; response: string; timestamp: string }>(
        `/api/v1/chat/ws/${sessionId}`,
        {
          method: "POST",
          body: JSON.stringify({ message }),
        }
      );
      return response;
    } catch (error) {
      console.error("Failed to send chat message:", error);
      throw error;
    }
  }

  async getDatabaseTables(): Promise<{
    tables: Array<{
      name: string;
      schema: string;
      rowCount: number;
      columns: Array<{
        name: string;
        type: string;
        nullable: boolean;
        primaryKey: boolean;
      }>;
    }>;
  }> {
    try {
      const response = await this.request<{
        tables: Array<{
          name: string;
          schema: string;
          rowCount: number;
          columns: Array<{
            name: string;
            type: string;
            nullable: boolean;
            primaryKey: boolean;
          }>;
        }>;
      }>("/api/v1/database/tables");
      return response;
    } catch (error) {
      console.error("Failed to get database tables:", error);
      throw error;
    }
  }

  async queryDatabase(
    table: string,
    filters: Record<string, unknown>
  ): Promise<{
    data: Array<Record<string, unknown>>;
    total: number;
    columns: Array<{
      name: string;
      type: string;
    }>;
  }> {
    try {
      const response = await this.request<{
        data: Array<Record<string, unknown>>;
        total: number;
        columns: Array<{
          name: string;
          type: string;
        }>;
      }>(`/api/v1/database/tables/${table}/query`, {
        method: "POST",
        body: JSON.stringify({ filters }),
      });
      return response;
    } catch (error) {
      console.error("Failed to query database:", error);
      throw error;
    }
  }
  /* eslint-enable no-unused-vars */

  async getMetrics(): Promise<{
    agents: Array<{
      id: string;
      status: string;
      activeTasks: number;
      completedTasks: number;
      errorRate: number;
    }>;
    coordination: {
      totalTasks: number;
      activeTasks: number;
      completedTasks: number;
      failedTasks: number;
    };
    business: {
      totalUsers: number;
      activeUsers: number;
      systemHealth: number;
      responseTime: number;
    };
  }> {
    try {
      const response = await this.request<{
        agents: Array<{
          id: string;
          status: string;
          activeTasks: number;
          completedTasks: number;
          errorRate: number;
        }>;
        coordination: {
          totalTasks: number;
          activeTasks: number;
          completedTasks: number;
          failedTasks: number;
        };
        business: {
          totalUsers: number;
          activeUsers: number;
          systemHealth: number;
          responseTime: number;
        };
      }>("/metrics/stream");
      return response;
    } catch (error) {
      console.error("Failed to get metrics:", error);
      throw error;
    }
  }
}

// Default client instance
export const apiClient = new ApiClient();

// Types are exported inline with their declarations
