/**
 * Security API Client
 * API client for security monitoring and access control
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface LoginEvent {
  id: string;
  userId: string;
  username: string;
  timestamp: Date;
  success: boolean;
  failureReason?: string;
  ipAddress: string;
  userAgent: string;
  location?: {
    country: string;
    city: string;
    latitude: number;
    longitude: number;
  };
  deviceFingerprint: string;
  mfaUsed: boolean;
  sessionId?: string;
}

export interface Session {
  id: string;
  userId: string;
  username: string;
  startTime: Date;
  lastActivity: Date;
  ipAddress: string;
  userAgent: string;
  deviceType: 'desktop' | 'mobile' | 'tablet';
  location?: {
    country: string;
    city: string;
  };
  isActive: boolean;
  expiresAt: Date;
}

export interface MFAEnrollment {
  userId: string;
  username: string;
  enrolled: boolean;
  method: 'totp' | 'sms' | 'email' | 'hardware';
  enrolledAt?: Date;
  lastUsed?: Date;
  backupCodesRemaining?: number;
}

export interface UserRole {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  isSystemRole: boolean;
  createdAt: Date;
  updatedAt: Date;
  userCount: number;
}

export interface Permission {
  id: string;
  resource: string;
  action: string;
  description: string;
  scope: 'global' | 'organization' | 'project' | 'resource';
  roles: string[];
}

export interface AccessAuditEvent {
  id: string;
  timestamp: Date;
  userId: string;
  username: string;
  action: string;
  resource: string;
  resourceId?: string;
  success: boolean;
  failureReason?: string;
  ipAddress: string;
  userAgent: string;
  context?: Record<string, any>;
}

export interface Secret {
  id: string;
  name: string;
  type: 'api_key' | 'database' | 'encryption_key' | 'service_account' | 'certificate' | 'config';
  description?: string;
  createdAt: Date;
  updatedAt: Date;
  expiresAt?: Date;
  lastRotated?: Date;
  rotationPolicy: 'manual' | 'automatic' | 'never';
  accessCount: number;
  lastAccessed?: Date;
  healthStatus: 'healthy' | 'expiring' | 'expired' | 'compromised';
}

export interface SecretAccessEvent {
  id: string;
  secretId: string;
  secretName: string;
  userId: string;
  username: string;
  timestamp: Date;
  action: 'read' | 'write' | 'rotate' | 'delete';
  ipAddress: string;
  userAgent: string;
  success: boolean;
}

export interface SecurityAlert {
  id: string;
  type: 'brute_force' | 'suspicious_login' | 'privilege_escalation' | 'data_exfiltration' | 'policy_violation' | 'malware_detected';
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  userId?: string;
  username?: string;
  resource?: string;
  resourceId?: string;
  ipAddress?: string;
  location?: {
    country: string;
    city: string;
  };
  timestamp: Date;
  acknowledged: boolean;
  acknowledgedBy?: string;
  acknowledgedAt?: Date;
  resolved: boolean;
  resolvedBy?: string;
  resolvedAt?: Date;
  evidence: SecurityEvidence[];
}

export interface SecurityEvidence {
  type: 'log' | 'metric' | 'network' | 'file' | 'memory';
  source: string;
  timestamp: Date;
  data: Record<string, any>;
  confidence: number;
}

export interface ThreatIntelligence {
  id: string;
  source: string;
  type: 'ip' | 'domain' | 'hash' | 'signature';
  value: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  firstSeen: Date;
  lastSeen: Date;
  tags: string[];
  confidence: number;
}

export interface SecurityIncident {
  id: string;
  title: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  status: 'detected' | 'investigating' | 'contained' | 'eradicating' | 'recovering' | 'resolved' | 'closed';
  category: 'authentication' | 'authorization' | 'data_breach' | 'malware' | 'ddos' | 'insider_threat' | 'other';
  detectedAt: Date;
  assignedTo?: string;
  sla: {
    detection: Date;
    response: Date;
    resolution: Date;
  };
  impact: {
    users: number;
    systems: number;
    data: string[];
    business: 'low' | 'medium' | 'high' | 'critical';
  };
  alerts: string[]; // Alert IDs
  evidence: SecurityEvidence[];
  timeline: IncidentTimelineEvent[];
  responseActions: ResponseAction[];
  lessonsLearned?: string;
}

export interface IncidentTimelineEvent {
  id: string;
  timestamp: Date;
  event: string;
  description: string;
  userId?: string;
  automated: boolean;
}

export interface ResponseAction {
  id: string;
  timestamp: Date;
  action: string;
  description: string;
  userId?: string;
  automated: boolean;
  success: boolean;
  result?: string;
}

export interface SecurityMetrics {
  auth: {
    totalLogins: number;
    successfulLogins: number;
    failedLogins: number;
    mfaAdoption: number;
    activeSessions: number;
    suspiciousActivities: number;
  };
  access: {
    totalRequests: number;
    authorizedRequests: number;
    deniedRequests: number;
    privilegeEscalations: number;
    policyViolations: number;
  };
  secrets: {
    totalSecrets: number;
    healthySecrets: number;
    expiringSecrets: number;
    compromisedSecrets: number;
    rotationRate: number;
  };
  threats: {
    activeAlerts: number;
    criticalAlerts: number;
    incidents: number;
    activeIncidents: number;
    mttr: number; // Mean Time To Resolution in minutes
    mttr: number; // Mean Time To Response in minutes
  };
}

export interface SecurityPolicy {
  id: string;
  name: string;
  description: string;
  category: 'authentication' | 'authorization' | 'data_protection' | 'network' | 'endpoint';
  enabled: boolean;
  priority: 'low' | 'medium' | 'high' | 'critical';
  rules: PolicyRule[];
  createdAt: Date;
  updatedAt: Date;
  lastEnforced?: Date;
}

export interface PolicyRule {
  id: string;
  condition: string;
  action: 'allow' | 'deny' | 'alert' | 'quarantine' | 'block';
  parameters: Record<string, any>;
}

export class SecurityApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/security') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Authentication monitoring endpoints
   */
  async getAuthEvents(
    period: '1h' | '24h' | '7d' = '24h',
    userId?: string,
    success?: boolean
  ): Promise<LoginEvent[]> {
    const params = new URLSearchParams({ period });
    if (userId) params.append('userId', userId);
    if (success !== undefined) params.append('success', success.toString());

    const response = await this.apiClient.request<LoginEvent[]>(
      `/auth/events?${params}`
    );
    return response;
  }

  async getActiveSessions(): Promise<Session[]> {
    const response = await this.apiClient.request<Session[]>('/auth/sessions/active');
    return response;
  }

  async terminateSession(sessionId: string): Promise<void> {
    await this.apiClient.request<void>(`/auth/sessions/${sessionId}/terminate`, {
      method: 'POST'
    });
  }

  async getMFAStatus(): Promise<MFAEnrollment[]> {
    const response = await this.apiClient.request<MFAEnrollment[]>('/auth/mfa/status');
    return response;
  }

  /**
   * Access control endpoints
   */
  async getRoles(): Promise<UserRole[]> {
    const response = await this.apiClient.request<UserRole[]>('/access/roles');
    return response;
  }

  async getPermissions(): Promise<Permission[]> {
    const response = await this.apiClient.request<Permission[]>('/access/permissions');
    return response;
  }

  async getAccessAudit(
    period: '1h' | '24h' | '7d' = '24h',
    userId?: string,
    resource?: string,
    limit: number = 100
  ): Promise<AccessAuditEvent[]> {
    const params = new URLSearchParams({ period, limit: limit.toString() });
    if (userId) params.append('userId', userId);
    if (resource) params.append('resource', resource);

    const response = await this.apiClient.request<AccessAuditEvent[]>(
      `/access/audit?${params}`
    );
    return response;
  }

  async assignRole(userId: string, roleId: string): Promise<void> {
    await this.apiClient.request<void>('/access/roles/assign', {
      method: 'POST',
      body: JSON.stringify({ userId, roleId })
    });
  }

  async revokeRole(userId: string, roleId: string): Promise<void> {
    await this.apiClient.request<void>('/access/roles/revoke', {
      method: 'POST',
      body: JSON.stringify({ userId, roleId })
    });
  }

  /**
   * Secrets management endpoints
   */
  async getSecrets(): Promise<Secret[]> {
    const response = await this.apiClient.request<Secret[]>('/secrets');
    return response;
  }

  async getSecret(secretId: string): Promise<Secret> {
    const response = await this.apiClient.request<Secret>(`/secrets/${secretId}`);
    return response;
  }

  async rotateSecret(secretId: string): Promise<void> {
    await this.apiClient.request<void>(`/secrets/${secretId}/rotate`, {
      method: 'POST'
    });
  }

  async getSecretAccessLog(
    secretId?: string,
    period: '1h' | '24h' | '7d' = '24h',
    limit: number = 100
  ): Promise<SecretAccessEvent[]> {
    const params = new URLSearchParams({ period, limit: limit.toString() });
    if (secretId) params.append('secretId', secretId);

    const response = await this.apiClient.request<SecretAccessEvent[]>(
      `/secrets/access-log?${params}`
    );
    return response;
  }

  /**
   * Threat detection endpoints
   */
  async getAlerts(
    acknowledged: boolean = false,
    severity?: 'low' | 'medium' | 'high' | 'critical',
    limit: number = 50
  ): Promise<SecurityAlert[]> {
    const params = new URLSearchParams({
      acknowledged: acknowledged.toString(),
      limit: limit.toString()
    });

    if (severity) params.append('severity', severity);

    const response = await this.apiClient.request<SecurityAlert[]>(
      `/threats/alerts?${params}`
    );
    return response;
  }

  async acknowledgeAlert(alertId: string): Promise<void> {
    await this.apiClient.request<void>(`/threats/alerts/${alertId}/acknowledge`, {
      method: 'POST'
    });
  }

  async getIncidents(
    status?: SecurityIncident['status'],
    severity?: SecurityIncident['severity'],
    limit: number = 20
  ): Promise<SecurityIncident[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (status) params.append('status', status);
    if (severity) params.append('severity', severity);

    const response = await this.apiClient.request<SecurityIncident[]>(
      `/threats/incidents?${params}`
    );
    return response;
  }

  async getIncident(incidentId: string): Promise<SecurityIncident> {
    const response = await this.apiClient.request<SecurityIncident>(
      `/threats/incidents/${incidentId}`
    );
    return response;
  }

  async updateIncidentStatus(
    incidentId: string,
    status: SecurityIncident['status'],
    notes?: string
  ): Promise<void> {
    await this.apiClient.request<void>(`/threats/incidents/${incidentId}/status`, {
      method: 'PATCH',
      body: JSON.stringify({ status, notes })
    });
  }

  async assignIncident(incidentId: string, userId: string): Promise<void> {
    await this.apiClient.request<void>(`/threats/incidents/${incidentId}/assign`, {
      method: 'POST',
      body: JSON.stringify({ userId })
    });
  }

  async getThreatIntelligence(limit: number = 100): Promise<ThreatIntelligence[]> {
    const response = await this.apiClient.request<ThreatIntelligence[]>(
      `/threats/intelligence?limit=${limit}`
    );
    return response;
  }

  /**
   * Security metrics endpoints
   */
  async getSecurityMetrics(): Promise<SecurityMetrics> {
    const response = await this.apiClient.request<SecurityMetrics>('/metrics');
    return response;
  }

  /**
   * Security policies endpoints
   */
  async getPolicies(): Promise<SecurityPolicy[]> {
    const response = await this.apiClient.request<SecurityPolicy[]>('/policies');
    return response;
  }

  async updatePolicy(policyId: string, updates: Partial<SecurityPolicy>): Promise<void> {
    await this.apiClient.request<void>(`/policies/${policyId}`, {
      method: 'PATCH',
      body: JSON.stringify(updates)
    });
  }

  async createPolicy(policy: Omit<SecurityPolicy, 'id' | 'createdAt' | 'updatedAt'>): Promise<SecurityPolicy> {
    const response = await this.apiClient.request<SecurityPolicy>('/policies', {
      method: 'POST',
      body: JSON.stringify(policy)
    });
    return response;
  }

  /**
   * Emergency security controls
   */
  async emergencyLockdown(reason: string, operator: string): Promise<{
    success: boolean;
    lockedUsers: number;
    message: string;
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      lockedUsers: number;
      message: string;
    }>('/emergency/lockdown', {
      method: 'POST',
      body: JSON.stringify({ reason, operator })
    });
    return response;
  }

  async emergencyUnlock(operator: string): Promise<{
    success: boolean;
    message: string;
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
    }>('/emergency/unlock', {
      method: 'POST',
      body: JSON.stringify({ operator })
    });
    return response;
  }

  /**
   * Security reporting
   */
  async generateSecurityReport(
    period: '1h' | '24h' | '7d' | '30d' = '7d',
    format: 'json' | 'pdf' | 'csv' = 'pdf',
    includeIncidents: boolean = true,
    includeAudit: boolean = true
  ): Promise<Blob> {
    const params = new URLSearchParams({
      period,
      format,
      includeIncidents: includeIncidents.toString(),
      includeAudit: includeAudit.toString()
    });

    const response = await fetch(`${this.apiClient['config'].baseUrl}/reports/security?${params}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${this.apiClient['config'].authToken}`
      }
    });

    if (!response.ok) {
      throw new Error(`Report generation failed: ${response.statusText}`);
    }

    return response.blob();
  }

  /**
   * User management (admin only)
   */
  async lockUser(userId: string, reason: string, operator: string): Promise<void> {
    await this.apiClient.request<void>(`/users/${userId}/lock`, {
      method: 'POST',
      body: JSON.stringify({ reason, operator })
    });
  }

  async unlockUser(userId: string, operator: string): Promise<void> {
    await this.apiClient.request<void>(`/users/${userId}/unlock`, {
      method: 'POST',
      body: JSON.stringify({ operator })
    });
  }

  async resetUserPassword(userId: string, operator: string): Promise<{
    tempPassword: string;
    expiresAt: Date;
  }> {
    const response = await this.apiClient.request<{
      tempPassword: string;
      expiresAt: Date;
    }>(`/users/${userId}/reset-password`, {
      method: 'POST',
      body: JSON.stringify({ operator })
    });
    return response;
  }
}

// Export singleton instance
export const securityApiClient = new SecurityApiClient();
