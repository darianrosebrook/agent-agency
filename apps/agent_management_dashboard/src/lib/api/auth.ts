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
  return apiPost<LoginRequest, LoginResponse>('/auth/login', credentials, {
    requestSchema: LoginRequestSchema,
    responseSchema: LoginResponseSchema,
  });
}

/**
 * User logout
 */
export async function logout(): Promise<void> {
  await apiPost('/auth/logout', undefined);
}

/**
 * Refresh authentication token
 */
export async function refreshToken(request: RefreshTokenRequest): Promise<LoginResponse> {
  return apiPost<RefreshTokenRequest, LoginResponse>('/auth/refresh', request, {
    requestSchema: RefreshTokenRequestSchema,
    responseSchema: LoginResponseSchema,
  });
}

/**
 * Get current user
 */
export async function getCurrentUser(): Promise<UserResponse> {
  return apiGet<UserResponse>('/users/me', {
    responseSchema: UserResponseSchema,
  });
}

/**
 * Register request schema
 */
export const RegisterRequestSchema = z.object({
  username: z.string().min(3).max(255),
  email: z.string().email(),
  password: z.string().min(8),
  name: z.string().optional(),
});

export type RegisterRequest = z.infer<typeof RegisterRequestSchema>;

/**
 * User registration
 */
export async function register(credentials: RegisterRequest): Promise<UserResponse> {
  return apiPost<RegisterRequest, UserResponse>('/auth/register', credentials, {
    requestSchema: RegisterRequestSchema,
    responseSchema: UserResponseSchema,
  });
}

