/**
 * Authentication API Client
 *
 * Provides functions for authentication endpoints.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost } from './base';
import { z } from 'zod';

/**
 * Login request schema
 */
export const LoginRequestSchema = z.object({
  username: z.string().min(1),
  password: z.string().min(1),
});

export type LoginRequest = z.infer<typeof LoginRequestSchema>;

/**
 * Login response schema
 */
export const LoginResponseSchema = z.object({
  token: z.string(),
  refresh_token: z.string().optional(),
  user: z.object({
    id: z.string(),
    username: z.string(),
    name: z.string().optional(),
    roles: z.array(z.string()),
    is_active: z.boolean(),
    last_login: z.string().optional(),
  }),
  expires_at: z.string(),
});

export type LoginResponse = z.infer<typeof LoginResponseSchema>;

/**
 * Refresh token request schema
 */
export const RefreshTokenRequestSchema = z.object({
  refresh_token: z.string(),
});

export type RefreshTokenRequest = z.infer<typeof RefreshTokenRequestSchema>;

/**
 * User response schema
 */
export const UserResponseSchema = z.object({
  id: z.string(),
  username: z.string(),
  name: z.string().optional(),
  roles: z.array(z.string()),
  is_active: z.boolean(),
  last_login: z.string().optional(),
});

export type UserResponse = z.infer<typeof UserResponseSchema>;

/**
 * User login
 */
export async function login(credentials: LoginRequest): Promise<LoginResponse> {
  return apiPost<LoginRequest, LoginResponse>('/api/v1/auth/login', credentials, {
    requestSchema: LoginRequestSchema,
    responseSchema: LoginResponseSchema,
  });
}

/**
 * User logout
 */
export async function logout(): Promise<void> {
  await apiPost('/api/v1/auth/logout', undefined);
}

/**
 * Refresh authentication token
 */
export async function refreshToken(request: RefreshTokenRequest): Promise<LoginResponse> {
  return apiPost<RefreshTokenRequest, LoginResponse>('/api/v1/auth/refresh', request, {
    requestSchema: RefreshTokenRequestSchema,
    responseSchema: LoginResponseSchema,
  });
}

/**
 * Get current user
 */
export async function getCurrentUser(): Promise<UserResponse> {
  return apiGet<UserResponse>('/api/v1/users/me', {
    responseSchema: UserResponseSchema,
  });
}

