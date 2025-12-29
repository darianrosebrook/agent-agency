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
 * Returns null if not authenticated (401) - handled gracefully
 */
export async function getCurrentUser(): Promise<CurrentUser | null> {
  try {
    return await apiGet<CurrentUser>(`${API_BASE}/users/me`, {
      showToast: false, // Suppress toast for expected 401
      throwOnError: false,
    });
  } catch (error) {
    // Return null for 401 (not authenticated) - this is expected in development
    if (error && typeof error === 'object' && 'status' in error && error.status === 401) {
      return null;
    }
    throw error;
  }
}












