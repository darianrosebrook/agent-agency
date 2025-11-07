// Base API client with authentication
import axios, { AxiosInstance, AxiosError, InternalAxiosRequestConfig } from 'axios';
import type { ApiResponse, ApiError } from '@/types';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

class ApiClient {
  private client: AxiosInstance;
  private token: string | null = null;
  private tokenExpiry: number | null = null;

  constructor() {
    this.client = axios.create({
      baseURL: API_URL,
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: 30000,
    });

    // Request interceptor to add auth token
    this.client.interceptors.request.use(
      async (config) => {
        if (!this.token || this.isTokenExpired()) {
          await this.authenticate();
        }
        if (this.token) {
          config.headers.Authorization = `Bearer ${this.token}`;
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      async (error: AxiosError) => {
        if (error.response?.status === 401) {
          // Token expired or invalid, try to re-authenticate
          await this.authenticate();
          // Retry the original request
          if (error.config) {
            return this.client.request(error.config);
          }
        }
        return Promise.reject(this.handleError(error));
      }
    );
  }

  private async authenticate(): Promise<void> {
    try {
      // Use admin credentials from environment
      // In production, these should be server-side only
      const username = process.env.NEXT_PUBLIC_API_ADMIN_USERNAME || 'admin';
      const password = process.env.NEXT_PUBLIC_API_ADMIN_PASSWORD || '';

      // Use proxy API route to handle authentication server-side
      // This prevents exposing credentials to the client
      const response = await axios.post('/api/auth/login', {
        username,
        password,
      });

      if (response.data.token) {
        this.token = response.data.token;
        // Assume token expires in 24 hours (adjust based on actual JWT expiry)
        this.tokenExpiry = Date.now() + 24 * 60 * 60 * 1000;
      }
    } catch (error) {
      console.error('Authentication failed:', error);
      // Don't throw in client-side - allow API calls to fail gracefully
      // The API proxy will handle authentication
    }
  }

  private isTokenExpired(): boolean {
    if (!this.tokenExpiry) return true;
    // Refresh token 5 minutes before expiry
    return Date.now() >= this.tokenExpiry - 5 * 60 * 1000;
  }

  private handleError(error: AxiosError): ApiError {
    if (error.response) {
      return {
        message: (error.response.data as { message?: string })?.message || error.message,
        code: error.response.status.toString(),
        details: error.response.data as Record<string, unknown>,
      };
    }
    if (error.request) {
      return {
        message: 'Network error: Unable to reach the API server',
        code: 'NETWORK_ERROR',
      };
    }
    return {
      message: error.message || 'An unexpected error occurred',
      code: 'UNKNOWN_ERROR',
    };
  }

  async get<T>(url: string, config?: InternalAxiosRequestConfig): Promise<T> {
    // Use proxy route for client-side requests
    const proxyUrl = `/api/proxy${url}`;
    const response = await axios.get<T>(proxyUrl, {
      ...config,
      headers: {
        ...config?.headers,
        ...(this.token && { Authorization: `Bearer ${this.token}` }),
      },
    });
    return response.data;
  }

  async post<T>(url: string, data?: unknown, config?: InternalAxiosRequestConfig): Promise<T> {
    // Use proxy route for client-side requests
    const proxyUrl = `/api/proxy${url}`;
    const response = await axios.post<T>(proxyUrl, data, {
      ...config,
      headers: {
        ...config?.headers,
        ...(this.token && { Authorization: `Bearer ${this.token}` }),
      },
    });
    return response.data;
  }

  async put<T>(url: string, data?: unknown, config?: InternalAxiosRequestConfig): Promise<T> {
    const proxyUrl = `/api/proxy${url}`;
    const response = await axios.put<T>(proxyUrl, data, {
      ...config,
      headers: {
        ...config?.headers,
        ...(this.token && { Authorization: `Bearer ${this.token}` }),
      },
    });
    return response.data;
  }

  async delete<T>(url: string, config?: InternalAxiosRequestConfig): Promise<T> {
    const proxyUrl = `/api/proxy${url}`;
    const response = await axios.delete<T>(proxyUrl, {
      ...config,
      headers: {
        ...config?.headers,
        ...(this.token && { Authorization: `Bearer ${this.token}` }),
      },
    });
    return response.data;
  }
}

export const apiClient = new ApiClient();

