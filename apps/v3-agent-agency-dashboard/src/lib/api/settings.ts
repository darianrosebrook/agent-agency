// Settings API client
// @author @darianrosebrook

import { serverApi } from './server';

// ============================================================================
// Types
// ============================================================================

export interface UserSetting {
  user_id: string;
  setting_key: string;
  setting_value: any;
  setting_type: string;
  created_at: string;
  updated_at: string;
}

export interface AppSetting {
  setting_key: string;
  setting_value: any;
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
  config: Record<string, any>;
  credentials: Record<string, any>;
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

export interface Verify2FARequest {
  method: string;
  code: string;
}

// ============================================================================
// User Settings
// ============================================================================

export async function getUserSettings(settingType?: string): Promise<UserSetting[]> {
  const params = settingType ? `?type=${encodeURIComponent(settingType)}` : '';
  return serverApi.get<UserSetting[]>(`/api/v1/settings/user${params}`);
}

export async function getUserSetting(key: string): Promise<UserSetting> {
  return serverApi.get<UserSetting>(`/api/v1/settings/user/${encodeURIComponent(key)}`);
}

export async function createUserSetting(
  key: string,
  value: any,
  type: string
): Promise<UserSetting> {
  return serverApi.post<UserSetting>('/api/v1/settings/user', {
    setting_key: key,
    setting_value: value,
    setting_type: type,
  });
}

export async function updateUserSetting(
  key: string,
  value?: any,
  type?: string
): Promise<UserSetting> {
  return serverApi.patch<UserSetting>(`/api/v1/settings/user/${encodeURIComponent(key)}`, {
    setting_value: value,
    setting_type: type,
  });
}

export async function deleteUserSetting(key: string): Promise<void> {
  return serverApi.delete(`/api/v1/settings/user/${encodeURIComponent(key)}`);
}

// ============================================================================
// App Settings
// ============================================================================

export async function getAppSettings(type?: string, isPublic?: boolean): Promise<AppSetting[]> {
  const params = new URLSearchParams();
  if (type) params.append('type', type);
  if (isPublic !== undefined) params.append('is_public', String(isPublic));
  const query = params.toString() ? `?${params.toString()}` : '';
  return serverApi.get<AppSetting[]>(`/api/v1/settings/app${query}`);
}

export async function getAppSetting(key: string): Promise<AppSetting> {
  return serverApi.get<AppSetting>(`/api/v1/settings/app/${encodeURIComponent(key)}`);
}

export async function createAppSetting(
  key: string,
  value: any,
  type: string,
  description?: string,
  isPublic: boolean = false
): Promise<AppSetting> {
  return serverApi.post<AppSetting>('/api/v1/settings/app', {
    setting_key: key,
    setting_value: value,
    setting_type: type,
    description,
    is_public: isPublic,
  });
}

export async function updateAppSetting(
  key: string,
  value?: any,
  type?: string,
  description?: string,
  isPublic?: boolean
): Promise<AppSetting> {
  return serverApi.patch<AppSetting>(`/api/v1/settings/app/${encodeURIComponent(key)}`, {
    setting_value: value,
    setting_type: type,
    description,
    is_public: isPublic,
  });
}

export async function deleteAppSetting(key: string): Promise<void> {
  return serverApi.delete(`/api/v1/settings/app/${encodeURIComponent(key)}`);
}

// ============================================================================
// Integrations
// ============================================================================

export async function getIntegrations(provider?: string, isActive?: boolean): Promise<Integration[]> {
  const params = new URLSearchParams();
  if (provider) params.append('provider', provider);
  if (isActive !== undefined) params.append('is_active', String(isActive));
  const query = params.toString() ? `?${params.toString()}` : '';
  return serverApi.get<Integration[]>(`/api/v1/settings/integrations${query}`);
}

export async function getIntegration(id: string): Promise<Integration> {
  return serverApi.get<Integration>(`/api/v1/settings/integrations/${id}`);
}

export async function createIntegration(
  name: string,
  integrationType: string,
  provider: string,
  config: Record<string, any>,
  credentials: Record<string, any>,
  isActive: boolean = true
): Promise<Integration> {
  return serverApi.post<Integration>('/api/v1/settings/integrations', {
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
    configuration?: Record<string, any>;
    credentials?: Record<string, any>;
    is_active?: boolean;
    is_enabled?: boolean;
  }
): Promise<Integration> {
  return serverApi.patch<Integration>(`/api/v1/settings/integrations/${id}`, updates);
}

export async function deleteIntegration(id: string): Promise<void> {
  return serverApi.delete(`/api/v1/settings/integrations/${id}`);
}

// ============================================================================
// API Keys
// ============================================================================

export async function getApiKeys(): Promise<ApiKey[]> {
  return serverApi.get<ApiKey[]>('/api/v1/settings/api-keys');
}

export async function getApiKey(id: string): Promise<ApiKey> {
  return serverApi.get<ApiKey>(`/api/v1/settings/api-keys/${id}`);
}

export async function createApiKey(
  keyName: string,
  scopes: string[],
  rateLimitPerMinute?: number,
  rateLimitPerHour?: number,
  rateLimitPerDay?: number,
  expiresAt?: string
): Promise<{ api_key: ApiKey; key: string }> {
  return serverApi.post<{ api_key: ApiKey; key: string }>('/api/v1/settings/api-keys', {
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
  return serverApi.patch<ApiKey>(`/api/v1/settings/api-keys/${id}`, updates);
}

export async function revokeApiKey(id: string): Promise<ApiKey> {
  return serverApi.post<ApiKey>(`/api/v1/settings/api-keys/${id}/revoke`);
}

export async function deleteApiKey(id: string): Promise<void> {
  return serverApi.delete(`/api/v1/settings/api-keys/${id}`);
}

// ============================================================================
// Two-Factor Authentication
// ============================================================================

export async function get2FA(): Promise<TwoFactorAuth | null> {
  try {
    return await serverApi.get<TwoFactorAuth>('/api/v1/settings/2fa');
  } catch (error: any) {
    if (error.status === 404) {
      return null;
    }
    throw error;
  }
}

export async function setup2FA(method: string = 'totp'): Promise<Setup2FAResponse> {
  return serverApi.post<Setup2FAResponse>('/api/v1/settings/2fa', { method });
}

export async function verify2FA(method: string, code: string): Promise<{ status: string; message: string }> {
  return serverApi.post<{ status: string; message: string }>('/api/v1/settings/2fa/verify', {
    method,
    code,
  });
}

export async function disable2FA(): Promise<void> {
  return serverApi.delete('/api/v1/settings/2fa');
}

// ============================================================================
// Password Change
// ============================================================================

export async function changePassword(currentPassword: string, newPassword: string): Promise<void> {
  return serverApi.post('/api/v1/settings/password', {
    current_password: currentPassword,
    new_password: newPassword,
  });
}

