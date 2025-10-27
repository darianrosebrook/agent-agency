/**
 * API Client for Rust Backend
 * Provides connection management, abort controllers, and proper error handling
 *
 * @author @darianrosebrook
 */

import { Task, TaskSubmissionRequest, TaskSubmissionResponse, TaskListResponse } from '@/types/tasks';

// Connection management types
export interface ConnectionConfig {
  baseUrl: string;
  timeout: number;
  retryAttempts: number;
  retryDelay: number;
}

export interface RequestOptions {
  signal?: AbortSignal;
  timeout?: number;
  retries?: number;
}

export interface ApiResponse<T> {
  data: T;
  status: number;
  headers: Record<string, string>;
  timestamp: string;
}

export interface ApiError {
  message: string;
  status?: number;
  code?: string;
  details?: any;
  timestamp: string;
}

// Connection pool management
class ConnectionPool {
  private activeConnections = new Map<string, AbortController>();
  private maxConnections = 10;
  private connectionTimeouts = new Map<string, NodeJS.Timeout>();

  createConnection(endpoint: string): AbortController {
    // Clean up expired connections
    this.cleanup();

    if (this.activeConnections.size >= this.maxConnections) {
      throw new Error('Maximum connections exceeded');
    }

    const controller = new AbortController();
    const connectionId = `${endpoint}-${Date.now()}-${Math.random()}`;

    this.activeConnections.set(connectionId, controller);

    // Auto-cleanup after timeout
    const timeout = setTimeout(() => {
      this.cleanupConnection(connectionId);
    }, 30000); // 30 second timeout

    this.connectionTimeouts.set(connectionId, timeout);

    return controller;
  }

  abortConnection(endpoint: string) {
    // Find and abort connections for this endpoint
    for (const [id, controller] of this.activeConnections.entries()) {
      if (id.startsWith(endpoint)) {
        controller.abort();
        this.cleanupConnection(id);
      }
    }
  }

  cleanupConnection(connectionId: string) {
    this.activeConnections.delete(connectionId);
    const timeout = this.connectionTimeouts.get(connectionId);
    if (timeout) {
      clearTimeout(timeout);
      this.connectionTimeouts.delete(connectionId);
    }
  }

  cleanup() {
    // Remove expired connections
    const now = Date.now();
    for (const [id, timeout] of this.connectionTimeouts.entries()) {
      // Check if connection has been active too long
      const connectionTime = parseInt(id.split('-')[1]);
      if (now - connectionTime > 60000) { // 1 minute
        this.cleanupConnection(id);
      }
    }
  }

  getActiveCount(): number {
    this.cleanup();
    return this.activeConnections.size;
  }
}

// Rate limiting implementation
class RateLimiter {
  private requests = new Map<string, number[]>();
  private limits: Record<string, { maxRequests: number; windowMs: number }> = {
    tasks: { maxRequests: 30, windowMs: 60000 }, // 30 requests per minute
    health: { maxRequests: 60, windowMs: 60000 }, // 60 requests per minute
    metrics: { maxRequests: 120, windowMs: 60000 }, // 120 requests per minute
  };

  canMakeRequest(endpoint: string): boolean {
    const limit = this.getLimit(endpoint);
    const now = Date.now();
    const windowStart = now - limit.windowMs;

    const requestTimes = this.requests.get(endpoint) || [];
    const recentRequests = requestTimes.filter(time => time > windowStart);

    return recentRequests.length < limit.maxRequests;
  }

  recordRequest(endpoint: string) {
    const now = Date.now();
    const requestTimes = this.requests.get(endpoint) || [];
    requestTimes.push(now);

    // Keep only recent requests
    const limit = this.getLimit(endpoint);
    const windowStart = now - limit.windowMs;
    const recentRequests = requestTimes.filter(time => time > windowStart);

    this.requests.set(endpoint, recentRequests);
  }

  private getLimit(endpoint: string) {
    // Extract category from endpoint
    if (endpoint.includes('/tasks')) return this.limits.tasks;
    if (endpoint.includes('/health')) return this.limits.health;
    if (endpoint.includes('/metrics')) return this.limits.metrics;
    return { maxRequests: 30, windowMs: 60000 }; // Default
  }
}

// Main API client
export class ApiClient {
  private config: ConnectionConfig;
  private connectionPool = new ConnectionPool();
  private rateLimiter = new RateLimiter();

  constructor(config: Partial<ConnectionConfig> = {}) {
    this.config = {
      baseUrl: config.baseUrl || process.env.NEXT_PUBLIC_V3_BACKEND_URL || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      retryAttempts: config.retryAttempts || 3,
      retryDelay: config.retryDelay || 1000,
    };
  }

  // Core request method with abort controller support
  async request<T>(
    endpoint: string,
    options: RequestInit & RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const {
      signal,
      timeout = this.config.timeout,
      retries = this.config.retryAttempts,
      ...fetchOptions
    } = options;

    // Check rate limits
    if (!this.rateLimiter.canMakeRequest(endpoint)) {
      throw new ApiError('Rate limit exceeded', 429, 'RATE_LIMIT_EXCEEDED', {
        endpoint,
        retryAfter: 60000
      });
    }

    // Create abort controller for this request
    const controller = signal || new AbortController();

    // Set up timeout
    const timeoutId = setTimeout(() => {
      if (controller && typeof controller.abort === 'function') {
        controller.abort();
      }
    }, timeout);

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        // Record the request for rate limiting
        this.rateLimiter.recordRequest(endpoint);

        const url = `${this.config.baseUrl}${endpoint}`;
        const response = await fetch(url, {
          ...fetchOptions,
          signal: controller.signal,
          headers: {
            'Content-Type': 'application/json',
            'User-Agent': 'web-dashboard-api-client',
            ...fetchOptions.headers,
          },
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
          const errorData = await response.json().catch(() => ({}));
          throw new ApiError(
            errorData.message || `HTTP ${response.status}`,
            response.status,
            'HTTP_ERROR',
            errorData
          );
        }

        const data = await response.json();
        const headers: Record<string, string> = {};
        response.headers.forEach((value, key) => {
          headers[key] = value;
        });

        return {
          data,
          status: response.status,
          headers,
          timestamp: new Date().toISOString(),
        };

      } catch (error) {
        clearTimeout(timeoutId);
        lastError = error as Error;

        // Don't retry if aborted or if it's the last attempt
        if (controller.signal.aborted || attempt === retries) {
          break;
        }

        // Exponential backoff
        const delay = this.config.retryDelay * Math.pow(2, attempt);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }

    // All retries failed
    if (lastError instanceof ApiError) {
      throw lastError;
    }

    throw new ApiError(
      lastError?.message || 'Network request failed',
      undefined,
      'NETWORK_ERROR',
      { originalError: lastError?.message }
    );
  }

  // Task API methods
  async getTasks(options?: RequestOptions): Promise<ApiResponse<TaskListResponse>> {
    return this.request<TaskListResponse>('/api/v1/tasks', {
      method: 'GET',
      ...options,
    });
  }

  async getTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<Task>> {
    return this.request<Task>(`/api/v1/tasks/${taskId}`, {
      method: 'GET',
      ...options,
    });
  }

  async createTask(
    taskData: TaskSubmissionRequest,
    options?: RequestOptions
  ): Promise<ApiResponse<TaskSubmissionResponse>> {
    return this.request<TaskSubmissionResponse>('/api/v1/tasks', {
      method: 'POST',
      body: JSON.stringify(taskData),
      ...options,
    });
  }

  async cancelTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<void>> {
    return this.request<void>(`/api/v1/tasks/${taskId}/cancel`, {
      method: 'POST',
      ...options,
    });
  }

  async pauseTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<void>> {
    return this.request<void>(`/api/v1/tasks/${taskId}/pause`, {
      method: 'POST',
      ...options,
    });
  }

  async resumeTask(taskId: string, options?: RequestOptions): Promise<ApiResponse<void>> {
    return this.request<void>(`/api/v1/tasks/${taskId}/resume`, {
      method: 'POST',
      ...options,
    });
  }

  // Health check
  async healthCheck(options?: RequestOptions): Promise<ApiResponse<{ status: string; version: string }>> {
    return this.request<{ status: string; version: string }>('/health', {
      method: 'GET',
      ...options,
    });
  }

  // Metrics
  async getMetrics(options?: RequestOptions): Promise<ApiResponse<Record<string, any>>> {
    return this.request<Record<string, any>>('/metrics', {
      method: 'GET',
      ...options,
    });
  }

  // Connection management
  getActiveConnections(): number {
    return this.connectionPool.getActiveCount();
  }

  abortEndpointConnections(endpoint: string) {
    this.connectionPool.abortConnection(endpoint);
  }

  // Update configuration
  updateConfig(config: Partial<ConnectionConfig>) {
    this.config = { ...this.config, ...config };
  }
}

// Custom error class
export class ApiError extends Error {
  public status?: number;
  public code?: string;
  public details?: any;
  public timestamp: string;

  constructor(message: string, status?: number, code?: string, details?: any) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
    this.details = details;
    this.timestamp = new Date().toISOString();
  }
}

// Singleton instance
let apiClientInstance: ApiClient | null = null;

export function getApiClient(): ApiClient {
  if (!apiClientInstance) {
    apiClientInstance = new ApiClient();
  }
  return apiClientInstance;
}

// Export default instance
export default getApiClient();