/**
 * Security Store
 * Zustand store for security monitoring and access control state management
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  LoginEvent,
  Session,
  MFAEnrollment,
  UserRole,
  Permission,
  AccessAuditEvent,
  Secret,
  SecretAccessEvent,
  SecurityAlert,
  SecurityIncident,
  SecurityMetrics,
  SecurityPolicy,
  ThreatIntelligence
} from '@/lib/security-api';

interface SecurityState {
  // Core data
  authEvents: LoginEvent[];
  activeSessions: Session[];
  mfaStatus: MFAEnrollment[];
  roles: UserRole[];
  permissions: Permission[];
  accessAudit: AccessAuditEvent[];
  secrets: Secret[];
  secretAccessLog: SecretAccessEvent[];
  alerts: SecurityAlert[];
  incidents: SecurityIncident[];
  threatIntelligence: ThreatIntelligence[];
  metrics: SecurityMetrics | null;
  policies: SecurityPolicy[];

  // UI state
  selectedTimeRange: '1h' | '24h' | '7d' | '30d';
  selectedIncident: SecurityIncident | null;
  selectedAlert: SecurityAlert | null;
  realTimeEnabled: boolean;
  lastUpdate: Date | null;

  // Loading states
  loading: {
    auth: boolean;
    sessions: boolean;
    mfa: boolean;
    roles: boolean;
    permissions: boolean;
    audit: boolean;
    secrets: boolean;
    secretAccess: boolean;
    alerts: boolean;
    incidents: boolean;
    intelligence: boolean;
    metrics: boolean;
    policies: boolean;
  };

  // Error states
  errors: {
    auth: string | null;
    sessions: string | null;
    mfa: string | null;
    roles: string | null;
    permissions: string | null;
    audit: string | null;
    secrets: string | null;
    secretAccess: string | null;
    alerts: string | null;
    incidents: string | null;
    intelligence: string | null;
    metrics: string | null;
    policies: string | null;
  };

  // Pagination and filtering
  pagination: {
    authPage: number;
    auditPage: number;
    alertsPage: number;
    incidentsPage: number;
    limit: number;
  };

  filters: {
    alertSeverity: ('low' | 'medium' | 'high' | 'critical')[] | null;
    incidentStatus: SecurityIncident['status'][] | null;
    incidentSeverity: SecurityIncident['severity'][] | null;
    auditUserId: string | null;
    auditResource: string | null;
    secretType: Secret['type'][] | null;
  };
}

interface SecurityActions {
  // Core data actions
  setAuthEvents: (events: LoginEvent[]) => void;
  addAuthEvent: (event: LoginEvent) => void;
  setActiveSessions: (sessions: Session[]) => void;
  updateSession: (sessionId: string, updates: Partial<Session>) => void;
  setMFAStatus: (status: MFAEnrollment[]) => void;
  setRoles: (roles: UserRole[]) => void;
  setPermissions: (permissions: Permission[]) => void;
  setAccessAudit: (audit: AccessAuditEvent[]) => void;
  addAuditEvent: (event: AccessAuditEvent) => void;
  setSecrets: (secrets: Secret[]) => void;
  updateSecret: (secretId: string, updates: Partial<Secret>) => void;
  setSecretAccessLog: (log: SecretAccessEvent[]) => void;
  setAlerts: (alerts: SecurityAlert[]) => void;
  addAlert: (alert: SecurityAlert) => void;
  acknowledgeAlert: (alertId: string) => void;
  setIncidents: (incidents: SecurityIncident[]) => void;
  updateIncident: (incidentId: string, updates: Partial<SecurityIncident>) => void;
  setSelectedIncident: (incident: SecurityIncident | null) => void;
  setSelectedAlert: (alert: SecurityAlert | null) => void;
  setThreatIntelligence: (intelligence: ThreatIntelligence[]) => void;
  setMetrics: (metrics: SecurityMetrics) => void;
  setPolicies: (policies: SecurityPolicy[]) => void;
  updatePolicy: (policyId: string, updates: Partial<SecurityPolicy>) => void;

  // UI state actions
  setTimeRange: (range: SecurityState['selectedTimeRange']) => void;
  setRealTimeEnabled: (enabled: boolean) => void;
  setLastUpdate: (timestamp: Date) => void;

  // Loading actions
  setLoading: (key: keyof SecurityState['loading'], loading: boolean) => void;
  setError: (key: keyof SecurityState['errors'], error: string | null) => void;
  clearErrors: () => void;

  // Pagination actions
  setPagination: (pagination: Partial<SecurityState['pagination']>) => void;
  nextAuthPage: () => void;
  nextAuditPage: () => void;
  nextAlertsPage: () => void;
  nextIncidentsPage: () => void;
  resetPagination: () => void;

  // Filter actions
  setFilters: (filters: Partial<SecurityState['filters']>) => void;
  clearFilters: () => void;

  // Utility actions
  reset: () => void;
}

const initialState: SecurityState = {
  authEvents: [],
  activeSessions: [],
  mfaStatus: [],
  roles: [],
  permissions: [],
  accessAudit: [],
  secrets: [],
  secretAccessLog: [],
  alerts: [],
  incidents: [],
  threatIntelligence: [],
  metrics: null,
  policies: [],
  selectedTimeRange: '24h',
  selectedIncident: null,
  selectedAlert: null,
  realTimeEnabled: true,
  lastUpdate: null,
  loading: {
    auth: false,
    sessions: false,
    mfa: false,
    roles: false,
    permissions: false,
    audit: false,
    secrets: false,
    secretAccess: false,
    alerts: false,
    incidents: false,
    intelligence: false,
    metrics: false,
    policies: false,
  },
  errors: {
    auth: null,
    sessions: null,
    mfa: null,
    roles: null,
    permissions: null,
    audit: null,
    secrets: null,
    secretAccess: null,
    alerts: null,
    incidents: null,
    intelligence: null,
    metrics: null,
    policies: null,
  },
  pagination: {
    authPage: 1,
    auditPage: 1,
    alertsPage: 1,
    incidentsPage: 1,
    limit: 50,
  },
  filters: {
    alertSeverity: null,
    incidentStatus: null,
    incidentSeverity: null,
    auditUserId: null,
    auditResource: null,
    secretType: null,
  },
};

export const useSecurityStore = create<SecurityState & SecurityActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Core data actions
      setAuthEvents: (events) => set({ authEvents: events }),
      addAuthEvent: (event) => set((state) => ({
        authEvents: [event, ...state.authEvents.slice(0, 999)] // Keep last 1000 events
      })),
      setActiveSessions: (sessions) => set({ activeSessions: sessions }),
      updateSession: (sessionId, updates) => set((state) => ({
        activeSessions: state.activeSessions.map(session =>
          session.id === sessionId ? { ...session, ...updates } : session
        )
      })),
      setMFAStatus: (status) => set({ mfaStatus: status }),
      setRoles: (roles) => set({ roles }),
      setPermissions: (permissions) => set({ permissions }),
      setAccessAudit: (audit) => set({ accessAudit: audit }),
      addAuditEvent: (event) => set((state) => ({
        accessAudit: [event, ...state.accessAudit.slice(0, 499)] // Keep last 500 events
      })),
      setSecrets: (secrets) => set({ secrets }),
      updateSecret: (secretId, updates) => set((state) => ({
        secrets: state.secrets.map(secret =>
          secret.id === secretId ? { ...secret, ...updates } : secret
        )
      })),
      setSecretAccessLog: (log) => set({ secretAccessLog: log }),
      setAlerts: (alerts) => set({ alerts }),
      addAlert: (alert) => set((state) => ({
        alerts: [alert, ...state.alerts]
      })),
      acknowledgeAlert: (alertId) => set((state) => ({
        alerts: state.alerts.map(alert =>
          alert.id === alertId ? { ...alert, acknowledged: true, acknowledgedAt: new Date() } : alert
        )
      })),
      setIncidents: (incidents) => set({ incidents }),
      updateIncident: (incidentId, updates) => set((state) => ({
        incidents: state.incidents.map(incident =>
          incident.id === incidentId ? { ...incident, ...updates } : incident
        ),
        selectedIncident: state.selectedIncident?.id === incidentId
          ? { ...state.selectedIncident, ...updates }
          : state.selectedIncident
      })),
      setSelectedIncident: (incident) => set({ selectedIncident: incident }),
      setSelectedAlert: (alert) => set({ selectedAlert: alert }),
      setThreatIntelligence: (intelligence) => set({ threatIntelligence: intelligence }),
      setMetrics: (metrics) => set({ metrics }),
      setPolicies: (policies) => set({ policies }),
      updatePolicy: (policyId, updates) => set((state) => ({
        policies: state.policies.map(policy =>
          policy.id === policyId ? { ...policy, ...updates } : policy
        )
      })),

      // UI state actions
      setTimeRange: (range) => set({ selectedTimeRange: range }),
      setRealTimeEnabled: (enabled) => set({ realTimeEnabled: enabled }),
      setLastUpdate: (timestamp) => set({ lastUpdate: timestamp }),

      // Loading actions
      setLoading: (key, loading) => set((state) => ({
        loading: { ...state.loading, [key]: loading }
      })),
      setError: (key, error) => set((state) => ({
        errors: { ...state.errors, [key]: error }
      })),
      clearErrors: () => set({ errors: initialState.errors }),

      // Pagination actions
      setPagination: (pagination) => set((state) => ({
        pagination: { ...state.pagination, ...pagination }
      })),
      nextAuthPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          authPage: state.pagination.authPage + 1
        }
      })),
      nextAuditPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          auditPage: state.pagination.auditPage + 1
        }
      })),
      nextAlertsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          alertsPage: state.pagination.alertsPage + 1
        }
      })),
      nextIncidentsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          incidentsPage: state.pagination.incidentsPage + 1
        }
      })),
      resetPagination: () => set({ pagination: initialState.pagination }),

      // Filter actions
      setFilters: (filters) => set((state) => ({
        filters: { ...state.filters, ...filters }
      })),
      clearFilters: () => set({ filters: initialState.filters }),

      // Utility actions
      reset: () => set(initialState),
    }),
    {
      name: 'security-store',
    }
  )
);

// Selector hooks for better performance
export const useAuthEvents = () => useSecurityStore((state) => state.authEvents);
export const useActiveSessions = () => useSecurityStore((state) => state.activeSessions);
export const useMFAStatus = () => useSecurityStore((state) => state.mfaStatus);
export const useSecurityRoles = () => useSecurityStore((state) => state.roles);
export const useSecurityPermissions = () => useSecurityStore((state) => state.permissions);
export const useAccessAudit = () => useSecurityStore((state) => state.accessAudit);
export const useSecrets = () => useSecurityStore((state) => state.secrets);
export const useSecretAccessLog = () => useSecurityStore((state) => state.secretAccessLog);
export const useSecurityAlerts = () => useSecurityStore((state) => state.alerts);
export const useSecurityIncidents = () => useSecurityStore((state) => state.incidents);
export const useSelectedSecurityIncident = () => useSecurityStore((state) => state.selectedIncident);
export const useSelectedSecurityAlert = () => useSecurityStore((state) => state.selectedAlert);
export const useThreatIntelligence = () => useSecurityStore((state) => state.threatIntelligence);
export const useSecurityMetrics = () => useSecurityStore((state) => state.metrics);
export const useSecurityPolicies = () => useSecurityStore((state) => state.policies);
export const useSecurityLoading = () => useSecurityStore((state) => state.loading);
export const useSecurityErrors = () => useSecurityStore((state) => state.errors);

// Computed selectors
export const useUnacknowledgedAlerts = () => useSecurityStore((state) =>
  state.alerts.filter(alert => !alert.acknowledged)
);

export const useCriticalAlerts = () => useSecurityStore((state) =>
  state.alerts.filter(alert => alert.severity === 'critical' && !alert.acknowledged)
);

export const useActiveIncidents = () => useSecurityStore((state) =>
  state.incidents.filter(incident =>
    ['detected', 'investigating', 'contained', 'eradicating', 'recovering'].includes(incident.status)
  )
);

export const useAuthSuccessRate = () => useSecurityStore((state) => {
  if (!state.metrics) return 0;
  const { successfulLogins, totalLogins } = state.metrics.auth;
  return totalLogins > 0 ? (successfulLogins / totalLogins) * 100 : 0;
});

export const useSecretsHealth = () => useSecurityStore((state) => {
  if (!state.secrets.length) return { healthy: 0, expiring: 0, expired: 0, compromised: 0 };

  return state.secrets.reduce((acc, secret) => {
    acc[secret.healthStatus]++;
    return acc;
  }, { healthy: 0, expiring: 0, expired: 0, compromised: 0 });
});

export const useFailedLoginAttempts = () => useSecurityStore((state) =>
  state.authEvents.filter(event => !event.success)
);

export const useSuspiciousActivities = () => useSecurityStore((state) =>
  state.authEvents.filter(event => {
    // Simple heuristic: failed login from different country recently
    const recentEvents = state.authEvents
      .filter(e => e.userId === event.userId && e.timestamp > new Date(Date.now() - 24 * 60 * 60 * 1000));
    const failedFromDifferentCountry = recentEvents.some(e =>
      !e.success && e.location?.country !== event.location?.country
    );
    return failedFromDifferentCountry;
  })
);

export const useSecurityActions = () => useSecurityStore((state) => ({
  setAuthEvents: state.setAuthEvents,
  addAuthEvent: state.addAuthEvent,
  setActiveSessions: state.setActiveSessions,
  updateSession: state.updateSession,
  setMFAStatus: state.setMFAStatus,
  setRoles: state.setRoles,
  setPermissions: state.setPermissions,
  setAccessAudit: state.setAccessAudit,
  addAuditEvent: state.addAuditEvent,
  setSecrets: state.setSecrets,
  updateSecret: state.updateSecret,
  setSecretAccessLog: state.setSecretAccessLog,
  setAlerts: state.setAlerts,
  addAlert: state.addAlert,
  acknowledgeAlert: state.acknowledgeAlert,
  setIncidents: state.setIncidents,
  updateIncident: state.updateIncident,
  setSelectedIncident: state.setSelectedIncident,
  setSelectedAlert: state.setSelectedAlert,
  setThreatIntelligence: state.setThreatIntelligence,
  setMetrics: state.setMetrics,
  setPolicies: state.setPolicies,
  updatePolicy: state.updatePolicy,
  setTimeRange: state.setTimeRange,
  setRealTimeEnabled: state.setRealTimeEnabled,
  setLastUpdate: state.setLastUpdate,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextAuthPage: state.nextAuthPage,
  nextAuditPage: state.nextAuditPage,
  nextAlertsPage: state.nextAlertsPage,
  nextIncidentsPage: state.nextIncidentsPage,
  resetPagination: state.resetPagination,
  setFilters: state.setFilters,
  clearFilters: state.clearFilters,
  reset: state.reset,
}));
