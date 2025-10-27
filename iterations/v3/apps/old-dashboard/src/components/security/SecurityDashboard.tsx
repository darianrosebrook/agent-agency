/**
 * Security Dashboard
 * Comprehensive security monitoring and access control interface
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { MetricCard, AnalyticsGrid } from '@/design-system/analytics';
import {
  Shield,
  AlertTriangle,
  Users,
  Lock,
  Key,
  Eye,
  TrendingUp,
  RefreshCw,
  Settings,
  Filter,
  Search,
  Activity,
  UserCheck,
  AlertCircle
} from 'lucide-react';
import { securityApiClient } from '@/lib/security-api';
import { useSecurityStore, useSecurityActions } from '@/stores/security';
import { useSecurityWebSocket, useRealTimeThreatMonitoring } from '@/hooks/useSecurityWebSocket';
// import { AuthMonitoringDashboard } from './AuthMonitoringDashboard';
// import { AccessControlDashboard } from './AccessControlDashboard';
// import { SecretsManagementDashboard } from './SecretsManagementDashboard';
// import { ThreatDetectionDashboard } from './ThreatDetectionDashboard';
import styles from './SecurityDashboard.module.scss';

export function SecurityDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'auth' | 'access' | 'secrets' | 'threats'>('overview');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { metrics, policies, loading, errors } = useSecurityStore();
  const actions = useSecurityActions();
  const { isConnected } = useSecurityWebSocket();

  // Real-time monitoring hooks
  const threatStats = useRealTimeThreatMonitoring();

  // Fetch initial data
  useEffect(() => {
    fetchSecurityData();
  }, []);

  const fetchSecurityData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch security metrics
      actions.setLoading('metrics', true);
      const metricsData = await securityApiClient.getSecurityMetrics();
      actions.setMetrics(metricsData);

      // Fetch policies
      actions.setLoading('policies', true);
      const policiesData = await securityApiClient.getPolicies();
      actions.setPolicies(policiesData);

      // Fetch authentication data
      actions.setLoading('auth', true);
      const authEvents = await securityApiClient.getAuthEvents('24h');
      actions.setAuthEvents(authEvents);

      actions.setLoading('sessions', true);
      const sessions = await securityApiClient.getActiveSessions();
      actions.setActiveSessions(sessions);

      actions.setLoading('mfa', true);
      const mfaStatus = await securityApiClient.getMFAStatus();
      actions.setMFAStatus(mfaStatus);

      // Fetch access control data
      actions.setLoading('roles', true);
      const roles = await securityApiClient.getRoles();
      actions.setRoles(roles);

      actions.setLoading('permissions', true);
      const permissions = await securityApiClient.getPermissions();
      actions.setPermissions(permissions);

      // Fetch secrets data
      actions.setLoading('secrets', true);
      const secrets = await securityApiClient.getSecrets();
      actions.setSecrets(secrets);

      // Fetch threat data
      actions.setLoading('alerts', true);
      const alerts = await securityApiClient.getAlerts(false, undefined, 50);
      actions.setAlerts(alerts);

      actions.setLoading('incidents', true);
      const incidents = await securityApiClient.getIncidents();
      actions.setIncidents(incidents);

    } catch (error) {
      console.error('Failed to fetch security dashboard data:', error);
      actions.setError('metrics', error instanceof Error ? error.message : 'Failed to fetch data');
    } finally {
      actions.setLoading('metrics', false);
      actions.setLoading('policies', false);
      actions.setLoading('auth', false);
      actions.setLoading('sessions', false);
      actions.setLoading('mfa', false);
      actions.setLoading('roles', false);
      actions.setLoading('permissions', false);
      actions.setLoading('secrets', false);
      actions.setLoading('alerts', false);
      actions.setLoading('incidents', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchSecurityData();
  };

  // Mock overview metrics for demonstration (when real data is not available)
  const overviewMetrics = metrics ? [
    {
      title: 'Auth Success Rate',
      value: `${((metrics.auth.successfulLogins / Math.max(metrics.auth.totalLogins, 1)) * 100).toFixed(1)}%`,
      subtitle: `${metrics.auth.successfulLogins}/${metrics.auth.totalLogins} successful`,
      change: { value: 2.1, type: 'increase' as const, period: 'vs last week' },
      status: (metrics.auth.successfulLogins / Math.max(metrics.auth.totalLogins, 1)) > 0.95 ? 'good' as const : 'warning' as const,
      trend: 'up' as const,
      icon: <UserCheck size={20} />
    },
    {
      title: 'Active Sessions',
      value: metrics.auth.activeSessions.toString(),
      subtitle: 'Current active users',
      change: { value: -5, type: 'decrease' as const, period: 'vs yesterday' },
      status: metrics.auth.activeSessions < 1000 ? 'good' as const : 'warning' as const,
      trend: 'down' as const,
      icon: <Users size={20} />
    },
    {
      title: 'Security Alerts',
      value: threatStats.unacknowledgedAlerts.length.toString(),
      subtitle: 'Require attention',
      change: { value: threatStats.criticalAlerts.length, type: 'neutral' as const, period: 'critical alerts' },
      status: threatStats.criticalAlerts.length > 0 ? 'error' as const : threatStats.unacknowledgedAlerts.length > 10 ? 'warning' as const : 'good' as const,
      trend: 'neutral' as const,
      icon: <AlertTriangle size={20} />
    },
    {
      title: 'Secrets Health',
      value: `${metrics.secrets.healthySecrets}/${metrics.secrets.totalSecrets}`,
      subtitle: 'Healthy secrets',
      change: { value: metrics.secrets.expiringSecrets, type: 'neutral' as const, period: 'expiring soon' },
      status: metrics.secrets.expiringSecrets > 5 ? 'warning' as const : 'good' as const,
      trend: 'neutral' as const,
      icon: <Key size={20} />
    },
    {
      title: 'Access Violations',
      value: metrics.access.deniedRequests.toString(),
      subtitle: 'Denied requests',
      change: { value: -12, type: 'decrease' as const, period: 'vs last week' },
      status: metrics.access.deniedRequests < 100 ? 'good' as const : 'warning' as const,
      trend: 'down' as const,
      icon: <Lock size={20} />
    },
    {
      title: 'MTTR',
      value: `${(metrics.threats.mttr / 60).toFixed(0)}m`,
      subtitle: 'Mean time to resolve',
      change: { value: -5, type: 'decrease' as const, period: 'vs last month' },
      status: metrics.threats.mttr < 900 ? 'good' as const : 'warning' as const, // Less than 15 minutes
      trend: 'down' as const,
      icon: <Activity size={20} />
    }
  ] : [];

  return (
    <div className={styles.securityDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Security Center</Text>
          <Text variant="paragraph-large" color="secondary">
            Authentication, access control, and threat monitoring
          </Text>

          <div className={styles.securityStatus}>
            <div className={`statusIndicator ${threatStats.criticalAlerts.length > 0 ? 'critical' : threatStats.unacknowledgedAlerts.length > 10 ? 'warning' : 'good'}`}>
              <Shield size={16} />
              <span>
                {threatStats.criticalAlerts.length > 0
                  ? 'Critical Security Issues'
                  : threatStats.unacknowledgedAlerts.length > 10
                  ? 'Elevated Security Risk'
                  : 'Security Status: Good'
                }
              </span>
            </div>
          </div>
        </div>

        <div className={styles.headerRight}>
          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <TrendingUp size={12} />
                <span>Live</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <Shield size={12} />
                <span>Offline</span>
              </div>
            )}
          </div>

          {/* Tab Navigation */}
          <div className={styles.tabNavigation}>
            <Button
              variant={activeTab === 'overview' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('overview')}
            >
              <Eye size={16} />
              Overview
            </Button>
            <Button
              variant={activeTab === 'auth' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('auth')}
            >
              <UserCheck size={16} />
              Auth
            </Button>
            <Button
              variant={activeTab === 'access' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('access')}
            >
              <Lock size={16} />
              Access
            </Button>
            <Button
              variant={activeTab === 'secrets' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('secrets')}
            >
              <Key size={16} />
              Secrets
            </Button>
            <Button
              variant={activeTab === 'threats' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('threats')}
            >
              <AlertTriangle size={16} />
              Threats
            </Button>
          </div>

          {/* Actions */}
          <div className={styles.actions}>
            <Button variant="secondary" size="sm">
              <Filter size={16} />
            </Button>
            <Button variant="secondary" size="sm">
              <Search size={16} />
            </Button>
            <Button variant="secondary" size="sm">
              <Settings size={16} />
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleRefresh}
              disabled={refreshing}
            >
              <RefreshCw size={16} className={refreshing ? styles.spinning : ''} />
            </Button>
          </div>
        </div>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className={styles.overview}>
          <AnalyticsGrid
            title="Security Overview"
            subtitle="Real-time security metrics and threat monitoring"
            columns={3}
            gap="md"
          >
            {overviewMetrics.map((metric, index) => (
              <MetricCard
                key={index}
                title={metric.title}
                value={metric.value}
                subtitle={metric.subtitle}
                change={metric.change}
                status={metric.status}
                trend={metric.trend}
                icon={metric.icon}
                size="medium"
              />
            ))}
          </AnalyticsGrid>

          {/* Quick Security Status */}
          <div className={styles.securityStatusGrid}>
            <div className={styles.statusCard}>
              <Text variant="h4">Alert Summary</Text>
              <div className={styles.alertSummary}>
                <div className={styles.alertItem}>
                  <div className={styles.alertDot} data-severity="critical"></div>
                  <Text variant="paragraph-medium">Critical: {threatStats.alertCountBySeverity.critical}</Text>
                </div>
                <div className={styles.alertItem}>
                  <div className={styles.alertDot} data-severity="high"></div>
                  <Text variant="paragraph-medium">High: {threatStats.alertCountBySeverity.high}</Text>
                </div>
                <div className={styles.alertItem}>
                  <div className={styles.alertDot} data-severity="medium"></div>
                  <Text variant="paragraph-medium">Medium: {threatStats.alertCountBySeverity.medium}</Text>
                </div>
                <div className={styles.alertItem}>
                  <div className={styles.alertDot} data-severity="low"></div>
                  <Text variant="paragraph-medium">Low: {threatStats.alertCountBySeverity.low}</Text>
                </div>
              </div>
            </div>

            <div className={styles.statusCard}>
              <Text variant="h4">Incident Status</Text>
              <div className={styles.incidentSummary}>
                <div className={styles.incidentItem}>
                  <Activity size={16} />
                  <Text variant="paragraph-medium">Active: {threatStats.incidentCountByStatus.investigating + threatStats.incidentCountByStatus.contained}</Text>
                </div>
                <div className={styles.incidentItem}>
                  <AlertCircle size={16} />
                  <Text variant="paragraph-medium">Detected: {threatStats.incidentCountByStatus.detected}</Text>
                </div>
                <div className={styles.incidentItem}>
                  <Activity size={16} />
                  <Text variant="paragraph-medium">Resolved: {threatStats.incidentCountByStatus.resolved}</Text>
                </div>
              </div>
            </div>

            <div className={styles.statusCard}>
              <Text variant="h4">Policy Compliance</Text>
              <div className={styles.policySummary}>
                <div className={styles.policyItem}>
                  <Text variant="paragraph-medium">Enabled: {policies.filter(p => p.enabled).length}</Text>
                </div>
                <div className={styles.policyItem}>
                  <Text variant="paragraph-medium">Violations: {metrics?.access.policyViolations || 0}</Text>
                </div>
                <div className={styles.policyItem}>
                  <Text variant="paragraph-medium">High Priority: {policies.filter(p => p.priority === 'high' || p.priority === 'critical').length}</Text>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Authentication Tab */}
      {activeTab === 'auth' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Authentication Monitoring</Text>
          <Text variant="paragraph-medium" color="secondary">
            Authentication monitoring dashboard coming soon...
          </Text>
        </div>
      )}

      {/* Access Control Tab */}
      {activeTab === 'access' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Access Control</Text>
          <Text variant="paragraph-medium" color="secondary">
            Access control dashboard coming soon...
          </Text>
        </div>
      )}

      {/* Secrets Management Tab */}
      {activeTab === 'secrets' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Secrets Management</Text>
          <Text variant="paragraph-medium" color="secondary">
            Secrets management dashboard coming soon...
          </Text>
        </div>
      )}

      {/* Threat Detection Tab */}
      {activeTab === 'threats' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Threat Detection</Text>
          <Text variant="paragraph-medium" color="secondary">
            Threat detection dashboard coming soon...
          </Text>
        </div>
      )}
    </div>
  );
}
