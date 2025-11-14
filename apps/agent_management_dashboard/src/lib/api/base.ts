/**
 * Base API Client
 *
 * Provides the foundation for all API clients with consistent error handling,
 * request/response validation, and retry logic.
 *
 * @author @darianrosebrook
 */

import { apiFetch, ApiFetchOptions } from '../utils/api';
import { z } from 'zod';

/**
 * Base API configuration
 */
export const API_BASE = '/api/proxy/api/v1';

/**
 * API request options with Zod validation
 */
export interface ApiRequestOptions<TRequest = unknown, TResponse = unknown> extends ApiFetchOptions {
  /**
   * Zod schema for request validation
   */
  requestSchema?: z.ZodSchema<TRequest>;
  
  /**
   * Zod schema for response validation
   */
  responseSchema?: z.ZodSchema<TResponse>;
  
  /**
   * Whether to validate request (default: true)
   */
  validateRequest?: boolean;
  
  /**
   * Whether to validate response (default: true)
   */
  validateResponse?: boolean;
}

/**
 * Make an API request with Zod validation
 *
 * @param url - API endpoint URL
 * @param options - Request options including validation schemas
 * @returns Validated response data
 *
 * @example
 * ```ts
 * const schema = z.object({
 *   id: z.string(),
 *   name: z.string(),
 * });
 *
 * const data = await apiRequest('/api/v1/agents', {
 *   method: 'GET',
 *   responseSchema: schema,
 * });
 * ```
 */
export async function apiRequest<TRequest = unknown, TResponse = unknown>(
  url: string,
  options: ApiRequestOptions<TRequest, TResponse> = {}
): Promise<TResponse> {
  const {
    requestSchema,
    responseSchema,
    validateRequest = true,
    validateResponse = true,
    body,
    ...fetchOptions
  } = options;

  // Validate request body if schema provided
  let validatedBody: string | undefined;
  if (body && requestSchema && validateRequest) {
    const parsed = requestSchema.parse(JSON.parse(typeof body === 'string' ? body : JSON.stringify(body)));
    validatedBody = JSON.stringify(parsed);
  } else {
    validatedBody = typeof body === 'string' ? body : body ? JSON.stringify(body) : undefined;
  }

  // Make API call
  const response = await apiFetch<TResponse>(url, {
    ...fetchOptions,
    body: validatedBody,
  });

  // Validate response if schema provided
  if (responseSchema && validateResponse) {
    return responseSchema.parse(response);
  }

  return response;
}

/**
 * GET request helper
 */
export async function apiGet<TResponse = unknown>(
  endpoint: string,
  options: Omit<ApiRequestOptions<never, TResponse>, 'method' | 'body'> = {}
): Promise<TResponse> {
  return apiRequest<never, TResponse>(`${API_BASE}${endpoint}`, {
    ...options,
    method: 'GET',
  });
}

/**
 * POST request helper
 */
export async function apiPost<TRequest = unknown, TResponse = unknown>(
  endpoint: string,
  body?: TRequest,
  options: Omit<ApiRequestOptions<TRequest, TResponse>, 'method' | 'body'> = {}
): Promise<TResponse> {
  return apiRequest<TRequest, TResponse>(`${API_BASE}${endpoint}`, {
    ...options,
    method: 'POST',
    body: body ? JSON.stringify(body) : undefined,
  });
}

/**
 * PATCH request helper
 */
export async function apiPatch<TRequest = unknown, TResponse = unknown>(
  endpoint: string,
  body?: TRequest,
  options: Omit<ApiRequestOptions<TRequest, TResponse>, 'method' | 'body'> = {}
): Promise<TResponse> {
  return apiRequest<TRequest, TResponse>(`${API_BASE}${endpoint}`, {
    ...options,
    method: 'PATCH',
    body: body ? JSON.stringify(body) : undefined,
  });
}

/**
 * DELETE request helper
 */
export async function apiDelete<TResponse = unknown>(
  endpoint: string,
  options: Omit<ApiRequestOptions<never, TResponse>, 'method' | 'body'> = {}
): Promise<TResponse> {
  return apiRequest<never, TResponse>(`${API_BASE}${endpoint}`, {
    ...options,
    method: 'DELETE',
  });
}

