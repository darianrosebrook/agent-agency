// API Client for V3 Backend Communication
// Handles authentication, retries, error handling, and structured responses

export interface ApiConfig {
  baseUrl: string
  timeout: number
  maxRetries: number
  retryDelay: number
  authToken?: string
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public response?: unknown,
    public url?: string
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export class ApiClient {
  private config: ApiConfig

  constructor(config: Partial<ApiConfig> = {}) {
    this.config = {
      baseUrl: config.baseUrl ?? (typeof window !== 'undefined' ? '/api/proxy' : 'http://localhost:8080'),
      timeout: config.timeout ?? 30000,
      maxRetries: config.maxRetries ?? 3,
      retryDelay: config.retryDelay ?? 1000,
      authToken: config.authToken,
    }
  }

  // Update configuration
  updateConfig(updates: Partial<ApiConfig>) {
    this.config = { ...this.config, ...updates }
  }

  // Generic request method with retry logic
  private async request<T>(
    path: string,
    options: RequestInit = {},
    retryCount = 0
  ): Promise<T> {
    const url = `${this.config.baseUrl}${path}`

    // Prepare headers
    const headers = new Headers(options.headers)

    // Set default headers
    headers.set('Accept', 'application/json')
    headers.set('Content-Type', 'application/json')

    // Add authorization if available
    if (this.config.authToken) {
      headers.set('Authorization', `Bearer ${this.config.authToken}`)
    }

    // Add user agent
    headers.set('User-Agent', 'web-dashboard/0.1.0')

    // Create AbortController for timeout
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout)

    try {
      const response = await fetch(url, {
        ...options,
        headers,
        signal: controller.signal,
      })

      clearTimeout(timeoutId)

      // Handle non-2xx responses
      if (!response.ok) {
        const errorText = await response.text()
        let errorMessage = `HTTP ${response.status}: ${response.statusText}`

        try {
          const errorData = JSON.parse(errorText)
          errorMessage = errorData.error || errorData.message || errorMessage
        } catch {
          // Use error text if not JSON
          if (errorText) {
            errorMessage = errorText
          }
        }

        // Check if we should retry
        const shouldRetry = this.shouldRetry(response.status, retryCount)

        if (shouldRetry) {
          console.warn(`Request failed (${response.status}), retrying in ${this.config.retryDelay}ms... (${retryCount + 1}/${this.config.maxRetries})`)
          await this.delay(this.config.retryDelay * Math.pow(2, retryCount)) // Exponential backoff
          return this.request<T>(path, options, retryCount + 1)
        }

        throw new ApiError(response.status, errorMessage, errorText, url)
      }

      // Handle empty responses
      const text = await response.text()
      if (!text) {
        return {} as T
      }

      // Parse JSON response
      try {
        return JSON.parse(text)
      } catch (parseError) {
        throw new ApiError(
          response.status,
          `Invalid JSON response: ${text.substring(0, 100)}...`,
          text,
          url
        )
      }

    } catch (error) {
      clearTimeout(timeoutId)

      // Handle timeout
      if (error instanceof Error && error.name === 'AbortError') {
        const timeoutError = new ApiError(
          408,
          `Request timeout after ${this.config.timeout}ms`,
          undefined,
          url
        )

        if (retryCount < this.config.maxRetries) {
          console.warn(`Request timeout, retrying in ${this.config.retryDelay}ms... (${retryCount + 1}/${this.config.maxRetries})`)
          await this.delay(this.config.retryDelay * Math.pow(2, retryCount))
          return this.request<T>(path, options, retryCount + 1)
        }

        throw timeoutError
      }

      // Handle network errors
      if (error instanceof Error && error.name === 'TypeError') {
        const networkError = new ApiError(
          0,
          `Network error: ${error.message}`,
          undefined,
          url
        )

        if (retryCount < this.config.maxRetries) {
          console.warn(`Network error, retrying in ${this.config.retryDelay}ms... (${retryCount + 1}/${this.config.maxRetries})`)
          await this.delay(this.config.retryDelay * Math.pow(2, retryCount))
          return this.request<T>(path, options, retryCount + 1)
        }

        throw networkError
      }

      // Re-throw ApiError instances
      if (error instanceof ApiError) {
        throw error
      }

      // Wrap other errors
      throw new ApiError(
        0,
        error instanceof Error ? error.message : 'Unknown error occurred',
        error,
        url
      )
    }
  }

  // Determine if a request should be retried based on status code
  private shouldRetry(status: number, retryCount: number): boolean {
    if (retryCount >= this.config.maxRetries) {
      return false
    }

    // Retry on server errors, timeouts, and network issues
    const retryableStatuses = [408, 429, 500, 502, 503, 504]
    return retryableStatuses.includes(status)
  }

  // Utility delay function
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms))
  }

  // Health check
  async health(): Promise<{
    status: string
    timestamp: string
    dashboard: any
    backend: any
  }> {
    // Use direct API call for health check (not proxied)
    const response = await fetch('/api/health', {
      headers: { 'Accept': 'application/json' },
      signal: AbortSignal.timeout(5000)
    })

    if (!response.ok) {
      throw new ApiError(response.status, `Health check failed: ${response.statusText}`)
    }

    return response.json()
  }

  // Placeholder methods for future V3 API endpoints
  // These will be implemented as the backend endpoints become available

  async getTasks(filters?: Record<string, unknown>): Promise<unknown> {
    return this.request('/tasks', {
      method: 'GET',
      body: filters ? JSON.stringify(filters) : undefined
    })
  }

  async getTask(taskId: string): Promise<unknown> {
    return this.request(`/tasks/${encodeURIComponent(taskId)}`)
  }

  async createChatSession(): Promise<unknown> {
    return this.request('/chat/session', { method: 'POST' })
  }

  async sendChatMessage(sessionId: string, message: string): Promise<unknown> {
    return this.request(`/chat/message`, {
      method: 'POST',
      body: JSON.stringify({ sessionId, message })
    })
  }

  async getDatabaseTables(): Promise<unknown> {
    return this.request('/database/tables')
  }

  async queryDatabase(table: string, filters: Record<string, unknown>): Promise<unknown> {
    return this.request(`/database/tables/${encodeURIComponent(table)}/query`, {
      method: 'POST',
      body: JSON.stringify(filters)
    })
  }

  async getMetrics(): Promise<unknown> {
    return this.request('/metrics')
  }
}

// Default client instance
export const apiClient = new ApiClient()

// Types are exported inline with their declarations
