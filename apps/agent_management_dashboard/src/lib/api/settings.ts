// Settings API client
// @author @darianrosebrook

import { apiGet, apiPost, apiPatch, apiDelete } from '../utils/api';

const API_BASE = '/api/proxy/api/v1';

// ============================================================================
// Types
// ============================================================================

export type SettingValue = string | number | boolean | null | undefined;

export interface UserSetting {
  user_id: string;
  setting_key: string;
  setting_value: SettingValue;
  setting_type: string;
  created_at: string;
  updated_at: string;
}

export interface AppSetting {
  setting_key: string;
  setting_value: SettingValue;
  setting_type: string;
  description?: string;
  is_public: boolean;
  created_at: string;
  updated_at: string;
}

export interface Integration {
  id: string;
  user_id: string;
  integration_type: string;
  provider: string;
  name: string;
  config: Record<string, unknown>;
  credentials: Record<string, unknown>;
  is_active: boolean;
  is_enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface ApiKey {
  id: string;
  user_id: string;
  key_name: string;
  key_hash: string;
  scopes: string[];
  rate_limit_per_minute?: number;
  rate_limit_per_hour?: number;
  rate_limit_per_day?: number;
  expires_at?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface TwoFactorAuth {
  user_id: string;
  method: string;
  is_enabled: boolean;
  backup_codes: string[];
}

export interface Setup2FAResponse {
  status: string;
  method: string;
  secret: string;
  qr_url: string;
  backup_codes: string[];
  message: string;
}

// ============================================================================
// User Settings
// ============================================================================

export async function getUserSettings(settingType?: string): Promise<UserSetting[]> {
  const params = settingType ? `?type=${encodeURIComponent(settingType)}` : '';
  return apiGet<UserSetting[]>(`${API_BASE}/settings/user${params}`);
}

export async function getUserSetting(key: string): Promise<UserSetting> {
  return apiGet<UserSetting>(`${API_BASE}/settings/user/${encodeURIComponent(key)}`);
}

/**
 * Get user setting if it exists, returns null if not found
 * 
 * Use this when the setting is optional and missing is expected behavior.
 * For required settings, use getUserSetting() which throws on 404.
 */
export async function getUserSettingOptional(key: string): Promise<UserSetting | null> {
  try {
    return await apiGet<UserSetting>(`${API_BASE}/settings/user/${encodeURIComponent(key)}`, {
      showToast: false, // Suppress toast for expected 404
    });
  } catch (error: unknown) {
    // Check if it's a 404 (setting doesn't exist)
    if (
      error instanceof Error &&
      'status' in error &&
      (error as { status: number }).status === 404
    ) {
      return null;
    }
    // Re-throw other errors (network, auth, etc.)
    throw error;
  }
}

export async function createUserSetting(
  key: string,
  value: SettingValue,
  type: string
): Promise<UserSetting> {
  return apiPost<UserSetting>(`${API_BASE}/settings/user`, {
    setting_key: key,
    setting_value: value,
    setting_type: type,
  });
}

export async function updateUserSetting(
  key: string,
  value?: SettingValue,
  type?: string
): Promise<UserSetting> {
  return apiPatch<UserSetting>(`${API_BASE}/settings/user/${encodeURIComponent(key)}`, {
    setting_value: value,
    setting_type: type,
  });
}

export async function deleteUserSetting(key: string): Promise<void> {
  return apiDelete(`${API_BASE}/settings/user/${encodeURIComponent(key)}`);
}

// ============================================================================
// App Settings
// ============================================================================

export async function getAppSettings(type?: string, isPublic?: boolean): Promise<AppSetting[]> {
  const params = new URLSearchParams();
  if (type) params.append('type', type);
  if (isPublic !== undefined) params.append('is_public', String(isPublic));
  const query = params.toString() ? `?${params.toString()}` : '';
  return apiGet<AppSetting[]>(`${API_BASE}/settings/app${query}`);
}

export async function getAppSetting(key: string): Promise<AppSetting> {
  return apiGet<AppSetting>(`${API_BASE}/settings/app/${encodeURIComponent(key)}`);
}

export async function createAppSetting(
  key: string,
  value: SettingValue,
  type: string,
  description?: string,
  isPublic: boolean = false
): Promise<AppSetting> {
  return apiPost<AppSetting>(`${API_BASE}/settings/app`, {
    setting_key: key,
    setting_value: value,
    setting_type: type,
    description,
    is_public: isPublic,
  });
}

export async function updateAppSetting(
  key: string,
  value?: SettingValue,
  type?: string,
  description?: string,
  isPublic?: boolean
): Promise<AppSetting> {
  return apiPatch<AppSetting>(`${API_BASE}/settings/app/${encodeURIComponent(key)}`, {
    setting_value: value,
    setting_type: type,
    description,
    is_public: isPublic,
  });
}

export async function deleteAppSetting(key: string): Promise<void> {
  return apiDelete(`${API_BASE}/settings/app/${encodeURIComponent(key)}`);
}

// ============================================================================
// Integrations
// ============================================================================

export async function getIntegrations(provider?: string, isActive?: boolean): Promise<Integration[]> {
  const params = new URLSearchParams();
  if (provider) params.append('provider', provider);
  if (isActive !== undefined) params.append('is_active', String(isActive));
  const query = params.toString() ? `?${params.toString()}` : '';
  return apiGet<Integration[]>(`${API_BASE}/settings/integrations${query}`);
}

export async function getIntegration(id: string): Promise<Integration> {
  return apiGet<Integration>(`${API_BASE}/settings/integrations/${id}`);
}

export async function createIntegration(
  name: string,
  integrationType: string,
  provider: string,
  config: Record<string, unknown>,
  credentials: Record<string, unknown>,
  isActive: boolean = true
): Promise<Integration> {
  return apiPost<Integration>(`${API_BASE}/settings/integrations`, {
    name,
    integration_type: integrationType,
    provider,
    configuration: config,
    credentials,
    is_active: isActive,
    is_enabled: true,
  });
}

export async function updateIntegration(
  id: string,
  updates: {
    name?: string;
    configuration?: Record<string, unknown>;
    credentials?: Record<string, unknown>;
    is_active?: boolean;
    is_enabled?: boolean;
  }
): Promise<Integration> {
  return apiPatch<Integration>(`${API_BASE}/settings/integrations/${id}`, updates);
}

export async function deleteIntegration(id: string): Promise<void> {
  return apiDelete(`${API_BASE}/settings/integrations/${id}`);
}

// ============================================================================
// API Keys
// ============================================================================

export async function getApiKeys(): Promise<ApiKey[]> {
  return apiGet<ApiKey[]>(`${API_BASE}/settings/api-keys`);
}

export async function getApiKey(id: string): Promise<ApiKey> {
  return apiGet<ApiKey>(`${API_BASE}/settings/api-keys/${id}`);
}

export async function createApiKey(
  keyName: string,
  scopes: string[],
  rateLimitPerMinute?: number,
  rateLimitPerHour?: number,
  rateLimitPerDay?: number,
  expiresAt?: string
): Promise<{ api_key: ApiKey; key: string }> {
  return apiPost<{ api_key: ApiKey; key: string }>(`${API_BASE}/settings/api-keys`, {
    key_name: keyName,
    scopes,
    rate_limit_per_minute: rateLimitPerMinute,
    rate_limit_per_hour: rateLimitPerHour,
    rate_limit_per_day: rateLimitPerDay,
    expires_at: expiresAt,
  });
}

export async function updateApiKey(
  id: string,
  updates: {
    key_name?: string;
    scopes?: string[];
    rate_limit_per_minute?: number;
    rate_limit_per_hour?: number;
    rate_limit_per_day?: number;
    expires_at?: string;
    is_active?: boolean;
  }
): Promise<ApiKey> {
  return apiPatch<ApiKey>(`${API_BASE}/settings/api-keys/${id}`, updates);
}

export async function revokeApiKey(id: string): Promise<ApiKey> {
  return apiPost<ApiKey>(`${API_BASE}/settings/api-keys/${id}/revoke`);
}

export async function deleteApiKey(id: string): Promise<void> {
  return apiDelete(`${API_BASE}/settings/api-keys/${id}`);
}

// ============================================================================
// Two-Factor Authentication
// ============================================================================

export async function get2FA(): Promise<TwoFactorAuth | null> {
  try {
    return await apiGet<TwoFactorAuth>(`${API_BASE}/settings/2fa`);
  } catch (error: unknown) {
    if (error instanceof Error && 'status' in error && (error as { status: number }).status === 404) {
      return null;
    }
    throw error;
  }
}

export async function setup2FA(method: string = 'totp'): Promise<Setup2FAResponse> {
  return apiPost<Setup2FAResponse>(`${API_BASE}/settings/2fa`, { method });
}

export async function verify2FA(method: string, code: string): Promise<{ status: string; message: string }> {
  return apiPost<{ status: string; message: string }>(`${API_BASE}/settings/2fa/verify`, {
    method,
    code,
  });
}

export async function disable2FA(): Promise<void> {
  return apiDelete(`${API_BASE}/settings/2fa`);
}

// ============================================================================
// Password Change
// ============================================================================

export async function changePassword(currentPassword: string, newPassword: string): Promise<void> {
  return apiPost(`${API_BASE}/settings/password`, {
    current_password: currentPassword,
    new_password: newPassword,
  });
}

