/**
 * API utility functions for consistent error handling
 * 
 * Provides helpers for making API calls with standardized error handling
 * that matches the backend ErrorResponse format.
 * 
 * @author @darianrosebrook
 */

import { parseApiError, AppError } from '../errors/types';
import { retryWithBackoff, RetryOptions } from './retry';

/**
 * Safely show toast notification for API errors
 * Uses lazy import to avoid circular dependencies
 */
function showApiErrorToast(error: AppError): void {
  try {
    const errorMessage = error.getUserMessage();
    
    // Lazy import to avoid circular dependency with toast utilities
    import('./toast').then(({ toastError }) => {
      try {
        toastError(error);
      } catch (toastErr) {
        // If toast call fails, log and fall back to console.error
        console.error('Failed to show toast:', toastErr);
        console.error(`[${error.code}]`, errorMessage);
      }
    }).catch((importErr) => {
      // If toast import fails, fall back to console.error
      console.error('Failed to import toast utilities:', importErr);
      console.error(`[${error.code}]`, errorMessage);
    });
  } catch (err) {
    // If dynamic import is not available, use console.error
    const errorMessage = error.getUserMessage();
    console.error('Error showing toast:', err);
    console.error(`[${error.code}]`, errorMessage);
  }
}

/**
 * Options for API fetch calls
 */
export interface ApiFetchOptions extends RequestInit {
  /**
   * Whether to throw errors or return them
   * @default true
   */
  throwOnError?: boolean;
  
  /**
   * Custom error handler
   */
  onError?: (error: AppError) => void;
  
  /**
   * Whether to show toast notifications for errors
   * @default true
   */
  showToast?: boolean;
  
  /**
   * Retry configuration
   * If provided, automatically retries retryable errors
   */
  retry?: RetryOptions | boolean;
}

/**
 * Standardized API fetch wrapper with error handling and retry logic
 * 
 * Automatically parses error responses using the standardized ErrorResponse format
 * and converts them to AppError instances. Supports automatic retry for retryable errors.
 * 
 * @example
 * ```ts
 * // Basic usage
 * const data = await apiFetch('/api/projects', {
 *   method: 'GET',
 *   headers: { 'Authorization': `Bearer ${token}` }
 * });
 * 
 * // With retry
 * const data = await apiFetch('/api/projects', {
 *   method: 'GET',
 *   retry: { maxAttempts: 3, initialDelay: 1000 }
 * });
 * ```
 */
export async function apiFetch<T = unknown>(
  url: string,
  options: ApiFetchOptions = {}
): Promise<T> {
  const {
    throwOnError = true,
    onError,
    showToast = true,
    retry: retryConfig,
    headers = {},
    ...fetchOptions
  } = options;

  // Create the fetch function
  const fetchFn = async (): Promise<T> => {
    // Get auth token from localStorage
    const authToken = typeof window !== 'undefined' ? localStorage.getItem('auth_token') : null;
    const authHeaders: Record<string, string> = {};

    // Only add Authorization header for real tokens (not mock tokens)
    if (authToken && !authToken.startsWith('mock-jwt-token-')) {
      authHeaders['Authorization'] = `Bearer ${authToken}`;
    }

    const response = await fetch(url, {
      ...fetchOptions,
      headers: {
        'Content-Type': 'application/json',
        ...authHeaders,
        ...headers,
      },
    });

    // Parse error response if not OK
    if (!response.ok) {
      let errorData: unknown;
      
      try {
        errorData = await response.json();
      } catch {
        // If response is not JSON, create error from status
        errorData = {
          error: response.statusText || `HTTP ${response.status}`,
          code: getErrorCodeFromStatus(response.status),
          status: response.status,
        };
      }

      const appError = parseApiError(errorData);
      
      // Show toast notification for API errors (unless explicitly disabled)
      if (throwOnError && showToast) {
        showApiErrorToast(appError);
      }
      
      if (onError) {
        onError(appError);
      }
      
      if (throwOnError) {
        throw appError;
      }
      
      return appError as unknown as T;
    }

    // Parse successful response
    const contentType = response.headers.get('content-type');
    if (contentType?.includes('application/json')) {
      return await response.json() as T;
    }
    
    return await response.text() as unknown as T;
  };

  // Wrap with retry logic if enabled
  if (retryConfig) {
    const retryOptions = retryConfig === true ? {} : retryConfig;
    try {
      return await retryWithBackoff(fetchFn, retryOptions);
    } catch (error) {
      const appError = error instanceof AppError 
        ? error 
        : parseApiError(error);
      
      // Show toast notification for retry failures
      if (throwOnError && showToast) {
        showApiErrorToast(appError);
      }
      
      if (onError) {
        onError(appError);
      }
      
      if (throwOnError) {
        throw appError;
      }
      
      return appError as unknown as T;
    }
  }

  // No retry - execute directly
  try {
    return await fetchFn();
  } catch (error) {
    // Handle network errors and other exceptions
    const appError = error instanceof AppError 
      ? error 
      : parseApiError(error);
    
    // Show toast notification for network/other errors
    if (throwOnError && showToast) {
      showApiErrorToast(appError);
    }
    
    if (onError) {
      onError(appError);
    }
    
    if (throwOnError) {
      throw appError;
    }
    
    return appError as unknown as T;
  }
}

/**
 * Map HTTP status codes to error codes
 */
function getErrorCodeFromStatus(status: number): string {
  const statusMap: Record<number, string> = {
    400: 'BAD_REQUEST',
    401: 'AUTHENTICATION_ERROR',
    403: 'AUTHORIZATION_ERROR',
    404: 'NOT_FOUND',
    409: 'CONFLICT',
    422: 'VALIDATION_ERROR',
    429: 'RATE_LIMIT_EXCEEDED',
    500: 'INTERNAL_ERROR',
    502: 'SERVER_ERROR',
    503: 'SERVER_ERROR',
    504: 'TIMEOUT',
  };
  
  return statusMap[status] || 'OPERATION_FAILED';
}

/**
 * GET request helper
 */
export async function apiGet<T = unknown>(
  url: string,
  options?: ApiFetchOptions
): Promise<T> {
  return apiFetch<T>(url, {
    ...options,
    method: 'GET',
  });
}

/**
 * POST request helper
 */
export async function apiPost<T = unknown>(
  url: string,
  body?: unknown,
  options?: ApiFetchOptions
): Promise<T> {
  return apiFetch<T>(url, {
    ...options,
    method: 'POST',
    body: body ? JSON.stringify(body) : undefined,
  });
}

/**
 * PUT request helper
 */
export async function apiPut<T = unknown>(
  url: string,
  body?: unknown,
  options?: ApiFetchOptions
): Promise<T> {
  return apiFetch<T>(url, {
    ...options,
    method: 'PUT',
    body: body ? JSON.stringify(body) : undefined,
  });
}

/**
 * PATCH request helper
 */
export async function apiPatch<T = unknown>(
  url: string,
  body?: unknown,
  options?: ApiFetchOptions
): Promise<T> {
  return apiFetch<T>(url, {
    ...options,
    method: 'PATCH',
    body: body ? JSON.stringify(body) : undefined,
  });
}

/**
 * DELETE request helper
 */
export async function apiDelete<T = unknown>(
  url: string,
  options?: ApiFetchOptions
): Promise<T> {
  return apiFetch<T>(url, {
    ...options,
    method: 'DELETE',
  });
}

