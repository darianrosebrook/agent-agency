/**
 * Security WebSocket Hook
 * Real-time updates for security monitoring and threat detection
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useSecurityStore, useSecurityActions } from '@/stores/security';
import { LoginEvent, SecurityAlert, SecurityIncident, AccessAuditEvent } from '@/lib/security-api';

interface SecurityWebSocketMessage {
  type: 'auth_event' | 'session_update' | 'alert_created' | 'incident_created' | 'incident_updated' | 'audit_event' | 'threat_intelligence' | 'metrics_update';
  data: any;
  timestamp: string;
}

export function useSecurityWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useSecurityActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/security`);

      ws.onopen = () => {
        console.log('Security WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;

        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));

        // Subscribe to real-time security updates
        ws.send(JSON.stringify({
          type: 'subscribe',
          channels: ['auth', 'alerts', 'incidents', 'audit', 'threats', 'metrics']
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: SecurityWebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse Security WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('Security WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('Security WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create Security WebSocket connection:', error);
      setConnectionStatus('error');
      scheduleReconnect();
    }
  };

  const scheduleReconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    const delay = reconnectDelay * Math.pow(2, reconnectAttempts.current);
    reconnectAttempts.current++;

    console.log(`Scheduling Security WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);

    reconnectTimeoutRef.current = setTimeout(() => {
      connect();
    }, delay);
  };

  const disconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setIsConnected(false);
    setConnectionStatus('disconnected');
  };

  const handleMessage = (message: SecurityWebSocketMessage) => {
    const { type, data, timestamp } = message;

    // Update last update timestamp
    actions.setLastUpdate(new Date(timestamp));

    switch (type) {
      case 'auth_event':
        actions.addAuthEvent(data as LoginEvent);
        break;

      case 'session_update':
        actions.updateSession(data.id, data.updates);
        break;

      case 'alert_created':
        actions.addAlert(data as SecurityAlert);
        break;

      case 'incident_created':
        actions.setIncidents([data, ...useSecurityStore.getState().incidents]);
        break;

      case 'incident_updated':
        actions.updateIncident(data.id, data.updates);
        break;

      case 'audit_event':
        actions.addAuditEvent(data as AccessAuditEvent);
        break;

      case 'threat_intelligence':
        // Update threat intelligence if it doesn't exist, or update existing
        const existing = useSecurityStore.getState().threatIntelligence;
        const existingIndex = existing.findIndex(ti => ti.id === data.id);
        if (existingIndex >= 0) {
          const updated = [...existing];
          updated[existingIndex] = data;
          actions.setThreatIntelligence(updated);
        } else {
          actions.setThreatIntelligence([data, ...existing.slice(0, 99)]); // Keep last 100
        }
        break;

      case 'metrics_update':
        actions.setMetrics(data);
        break;

      default:
        console.warn('Unknown Security WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('Security WebSocket not connected, cannot send message');
    }
  };

  // Subscribe to specific channels
  const subscribe = (channels: string[]) => {
    sendMessage({
      type: 'subscribe',
      channels
    });
  };

  // Unsubscribe from channels
  const unsubscribe = (channels: string[]) => {
    sendMessage({
      type: 'unsubscribe',
      channels
    });
  };

  // Request current security metrics
  const requestMetrics = () => {
    sendMessage({
      type: 'request_metrics'
    });
  };

  // Request active alerts
  const requestAlerts = () => {
    sendMessage({
      type: 'request_alerts'
    });
  };

  // Request active incidents
  const requestIncidents = () => {
    sendMessage({
      type: 'request_incidents'
    });
  };

  // Request audit events
  const requestAudit = () => {
    sendMessage({
      type: 'request_audit'
    });
  };

  // Request threat intelligence
  const requestThreatIntelligence = () => {
    sendMessage({
      type: 'request_threat_intelligence'
    });
  };

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, []);

  return {
    isConnected,
    connectionStatus,
    connect,
    disconnect,
    sendMessage,
    subscribe,
    unsubscribe,
    requestMetrics,
    requestAlerts,
    requestIncidents,
    requestAudit,
    requestThreatIntelligence,
  };
}

// Hook for real-time authentication monitoring
export function useRealTimeAuthMonitoring() {
  const authEvents = useSecurityStore((state) => state.authEvents);
  const activeSessions = useSecurityStore((state) => state.activeSessions);
  const mfaStatus = useSecurityStore((state) => state.mfaStatus);

  return {
    authEvents,
    activeSessions,
    mfaStatus,
    recentFailedLogins: authEvents.filter(event => !event.success).slice(0, 10),
    activeSessionCount: activeSessions.filter(session => session.isActive).length,
    mfaAdoptionRate: mfaStatus.length > 0
      ? (mfaStatus.filter(user => user.enrolled).length / mfaStatus.length) * 100
      : 0,
    suspiciousActivities: authEvents.filter(event => {
      // Simple heuristic: multiple failed logins from same IP
      const recentFailures = authEvents.filter(e =>
        e.ipAddress === event.ipAddress &&
        !e.success &&
        e.timestamp > new Date(Date.now() - 60 * 60 * 1000) // Last hour
      );
      return recentFailures.length >= 3;
    }),
  };
}

// Hook for real-time threat monitoring
export function useRealTimeThreatMonitoring() {
  const alerts = useSecurityStore((state) => state.alerts);
  const incidents = useSecurityStore((state) => state.incidents);

  return {
    alerts,
    incidents,
    unacknowledgedAlerts: alerts.filter(alert => !alert.acknowledged),
    criticalAlerts: alerts.filter(alert => alert.severity === 'critical' && !alert.acknowledged),
    activeIncidents: incidents.filter(incident =>
      ['detected', 'investigating', 'contained', 'eradicating', 'recovering'].includes(incident.status)
    ),
    recentAlerts: alerts.slice(0, 5),
    alertCountBySeverity: {
      critical: alerts.filter(a => a.severity === 'critical').length,
      high: alerts.filter(a => a.severity === 'high').length,
      medium: alerts.filter(a => a.severity === 'medium').length,
      low: alerts.filter(a => a.severity === 'low').length,
    },
    incidentCountByStatus: {
      detected: incidents.filter(i => i.status === 'detected').length,
      investigating: incidents.filter(i => i.status === 'investigating').length,
      contained: incidents.filter(i => i.status === 'contained').length,
      resolved: incidents.filter(i => i.status === 'resolved').length,
    },
  };
}

// Hook for real-time access monitoring
export function useRealTimeAccessMonitoring() {
  const accessAudit = useSecurityStore((state) => state.accessAudit);
  const roles = useSecurityStore((state) => state.roles);
  const permissions = useSecurityStore((state) => state.permissions);

  return {
    accessAudit,
    roles,
    permissions,
    recentAuditEvents: accessAudit.slice(0, 10),
    failedAccessAttempts: accessAudit.filter(event => !event.success),
    privilegeEscalations: accessAudit.filter(event =>
      event.action.includes('escalate') || event.action.includes('admin')
    ),
    roleDistribution: roles.reduce((acc, role) => {
      acc[role.name] = role.userCount;
      return acc;
    }, {} as Record<string, number>),
    recentPolicyViolations: accessAudit.filter(event => {
      // Simple heuristic: denied access to sensitive resources
      return !event.success && (
        event.resource.includes('admin') ||
        event.resource.includes('security') ||
        event.resource.includes('secret')
      );
    }),
  };
}

// Hook for real-time secrets monitoring
export function useRealTimeSecretsMonitoring() {
  const secrets = useSecurityStore((state) => state.secrets);
  const secretAccessLog = useSecurityStore((state) => state.secretAccessLog);

  return {
    secrets,
    secretAccessLog,
    expiringSecrets: secrets.filter(secret =>
      secret.expiresAt && new Date(secret.expiresAt) < new Date(Date.now() + 7 * 24 * 60 * 60 * 1000) // Next 7 days
    ),
    unhealthySecrets: secrets.filter(secret =>
      secret.healthStatus !== 'healthy'
    ),
    recentSecretAccess: secretAccessLog.slice(0, 10),
    secretAccessByType: secretAccessLog.reduce((acc, access) => {
      acc[access.action] = (acc[access.action] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
    secretsNeedingRotation: secrets.filter(secret =>
      secret.rotationPolicy === 'manual' &&
      secret.lastRotated &&
      new Date(secret.lastRotated) < new Date(Date.now() - 90 * 24 * 60 * 60 * 1000) // Older than 90 days
    ),
  };
}
