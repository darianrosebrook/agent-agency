/**
 * Users API Client
 * 
 * Provides functions for fetching user data from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet } from '../utils/api';

/**
 * Current user information
 */
export interface CurrentUser {
  id: string;
  email: string;
  name: string;
  created_at: string;
  updated_at: string;
  preferences?: Record<string, unknown>;
}

const API_BASE = '/api/proxy/api/v1';

/**
 * Get current authenticated user
 */
export async function getCurrentUser(): Promise<CurrentUser> {
  return apiGet<CurrentUser>(`${API_BASE}/users/me`);
}











