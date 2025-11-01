/**
 * System Health Monitoring Dashboard
 * Unified health monitoring across all Agent Agency V3 components with Grafana integration
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { MetricCard, AnalyticsGrid } from '@/design-system/analytics';
import {
  Activity,
  AlertTriangle,
  CheckCircle,
  XCircle,
  TrendingUp,
  Server,
  BarChart3,
  Settings,
  RefreshCw,
  Monitor,
  Database,
  Zap,
  Globe
} from 'lucide-react';
import { systemHealthApiClient } from '@/lib/system-health-api';
import { useSystemHealthStore, useSystemHealthActions } from '@/stores/system-health';
import { useSystemHealthWebSocket, useRealTimeComponentMonitoring, useRealTimeAlertMonitoring } from '@/hooks/useSystemHealthWebSocket';
import { UnifiedHealthDashboard } from './UnifiedHealthDashboard';
// import { GrafanaIntegrationPanel } from './GrafanaIntegrationPanel';
import { AlertCorrelationDashboard } from './AlertCorrelationDashboard';
// import { MetricsVisualizationPanel } from './MetricsVisualizationPanel';
import styles from './SystemHealthDashboard.module.scss';

export function SystemHealthDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'health' | 'grafana' | 'alerts' | 'metrics'>('overview');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { systemHealth, grafanaDashboards, customDashboards, loading, errors } = useSystemHealthStore();
  const actions = useSystemHealthActions();
  const { isConnected } = useSystemHealthWebSocket();

  // Real-time monitoring hooks
  const componentStats = useRealTimeComponentMonitoring();
  const alertStats = useRealTimeAlertMonitoring();

  // Fetch initial data
  useEffect(() => {
    fetchSystemHealthData();
  }, []);

  const fetchSystemHealthData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch system health
      actions.setLoading('health', true);
      const healthData = await systemHealthApiClient.getSystemHealth();
      actions.setSystemHealth(healthData);

      // Fetch components
      actions.setLoading('components', true);
      const componentsData = await systemHealthApiClient.getComponentHealth();
      actions.setComponents(componentsData);

      // Fetch alerts
      actions.setLoading('alerts', true);
      const alertsData = await systemHealthApiClient.getAlerts();
      actions.setAlerts(alertsData);

      // Fetch Grafana data
      actions.setLoading('grafana', true);
      const grafanaData = await systemHealthApiClient.getGrafanaDashboards();
      actions.setGrafanaDashboards(grafanaData);

      const grafanaAlerts = await systemHealthApiClient.getGrafanaAlerts();
      actions.setGrafanaAlerts(grafanaAlerts);

      // Fetch custom dashboards
      actions.setLoading('customDashboards', true);
      const customDashboardsData = await systemHealthApiClient.getCustomDashboards();
      actions.setCustomDashboards(customDashboardsData);

    } catch (error) {
      console.error('Failed to fetch system health dashboard data:', error);
      actions.setError('health', error instanceof Error ? error.message : 'Failed to fetch data');
    } finally {
      actions.setLoading('health', false);
      actions.setLoading('components', false);
      actions.setLoading('alerts', false);
      actions.setLoading('grafana', false);
      actions.setLoading('customDashboards', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchSystemHealthData();
  };

  // Mock overview metrics for demonstration (when real data is not available)
  const overviewMetrics = systemHealth ? [
    {
      title: 'Overall Health',
      value: `${systemHealth.overallScore}%`,
      subtitle: systemHealth.overallStatus.toUpperCase(),
      change: { value: 2.1, type: 'increase' as const, period: 'vs last hour' },
      status: systemHealth.overallStatus === 'healthy' ? 'good' as const :
              systemHealth.overallStatus === 'warning' ? 'warning' as const : 'error' as const,
      trend: 'up' as const,
      icon: <Activity size={20} />
    },
    {
      title: 'Active Components',
      value: systemHealth.components.length.toString(),
      subtitle: `${systemHealth.metrics.healthyComponents} healthy`,
      change: { value: 0, type: 'neutral' as const, period: 'stable' },
      status: systemHealth.metrics.criticalComponents === 0 ? 'good' as const :
              systemHealth.metrics.warningComponents > 0 ? 'warning' as const : 'error' as const,
      trend: 'stable' as const,
      icon: <Server size={20} />
    },
    {
      title: 'Active Alerts',
      value: systemHealth.alerts.length.toString(),
      subtitle: `${systemHealth.metrics.criticalAlerts} critical`,
      change: { value: alertStats.criticalAlerts.length, type: 'neutral' as const, period: 'currently active' },
      status: systemHealth.metrics.criticalAlerts === 0 ? 'good' as const :
              systemHealth.metrics.criticalAlerts < 5 ? 'warning' as const : 'error' as const,
      trend: 'neutral' as const,
      icon: <AlertTriangle size={20} />
    },
    {
      title: 'Avg Response Time',
      value: `${systemHealth.metrics.averageResponseTime.toFixed(0)}ms`,
      subtitle: 'Across all components',
      change: { value: -5.2, type: 'decrease' as const, period: 'vs last week' },
      status: systemHealth.metrics.averageResponseTime < 500 ? 'good' as const :
              systemHealth.metrics.averageResponseTime < 1000 ? 'warning' as const : 'error' as const,
      trend: 'down' as const,
      icon: <TrendingUp size={20} />
    },
    {
      title: 'System Uptime',
      value: `${systemHealth.metrics.uptime.toFixed(1)}%`,
      subtitle: 'Last 30 days',
      change: { value: 0.1, type: 'increase' as const, period: 'vs last month' },
      status: systemHealth.metrics.uptime > 99.5 ? 'good' as const :
              systemHealth.metrics.uptime > 99 ? 'warning' as const : 'error' as const,
      trend: 'up' as const,
      icon: <CheckCircle size={20} />
    },
    {
      title: 'Grafana Dashboards',
      value: grafanaDashboards.length.toString(),
      subtitle: 'Integrated monitoring',
      change: { value: 0, type: 'neutral' as const, period: 'stable' },
      status: 'good' as const,
      trend: 'neutral' as const,
      icon: <Monitor size={20} />
    }
  ] : [];

  return (
    <div className={styles.systemHealthDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">System Health Monitor</Text>
          <Text variant="paragraph-large" color="secondary">
            Unified health monitoring across all Agent Agency V3 components
          </Text>

          {systemHealth && (
            <div className={styles.healthStatus}>
              <div className={`statusIndicator ${systemHealth.overallStatus}`}>
                <Activity size={16} />
                <span>System Status: {systemHealth.overallStatus.toUpperCase()}</span>
              </div>
            </div>
          )}
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
                <Server size={12} />
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
              <Activity size={16} />
              Overview
            </Button>
            <Button
              variant={activeTab === 'health' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('health')}
            >
              <Server size={16} />
              Health
            </Button>
            <Button
              variant={activeTab === 'grafana' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('grafana')}
            >
              <Monitor size={16} />
              Grafana
            </Button>
            <Button
              variant={activeTab === 'alerts' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('alerts')}
            >
              <AlertTriangle size={16} />
              Alerts
            </Button>
            <Button
              variant={activeTab === 'metrics' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('metrics')}
            >
              <BarChart3 size={16} />
              Metrics
            </Button>
          </div>

          {/* Actions */}
          <div className={styles.actions}>
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
            title="System Health Overview"
            subtitle="Real-time health status across all Agent Agency V3 components"
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

          {/* System Status Summary */}
          <div className={styles.statusSummary}>
            <div className={styles.summaryCard}>
              <Text variant="h4">Component Status</Text>
              <div className={styles.statusGrid}>
                <div className={styles.statusItem}>
                  <div className={styles.statusDot} data-status="healthy"></div>
                  <Text variant="paragraph-medium">Healthy: {componentStats.healthyCount}</Text>
                </div>
                <div className={styles.statusItem}>
                  <div className={styles.statusDot} data-status="warning"></div>
                  <Text variant="paragraph-medium">Warning: {componentStats.warningCount}</Text>
                </div>
                <div className={styles.statusItem}>
                  <div className={styles.statusDot} data-status="critical"></div>
                  <Text variant="paragraph-medium">Critical: {componentStats.criticalCount}</Text>
                </div>
                <div className={styles.statusItem}>
                  <div className={styles.statusDot} data-status="unknown"></div>
                  <Text variant="paragraph-medium">Unknown: {componentStats.componentCount - componentStats.healthyCount - componentStats.warningCount - componentStats.criticalCount}</Text>
                </div>
              </div>
            </div>

            <div className={styles.summaryCard}>
              <Text variant="h4">Alert Priority</Text>
              <div className={styles.alertPriority}>
                <div className={styles.priorityItem}>
                  <AlertTriangle size={16} className={styles.critical} />
                  <Text variant="paragraph-medium">Critical: {alertStats.alertCountBySeverity.critical}</Text>
                </div>
                <div className={styles.priorityItem}>
                  <AlertTriangle size={16} className={styles.high} />
                  <Text variant="paragraph-medium">High: {alertStats.alertCountBySeverity.high}</Text>
                </div>
                <div className={styles.priorityItem}>
                  <AlertTriangle size={16} className={styles.medium} />
                  <Text variant="paragraph-medium">Medium: {alertStats.alertCountBySeverity.medium}</Text>
                </div>
                <div className={styles.priorityItem}>
                  <AlertTriangle size={16} className={styles.low} />
                  <Text variant="paragraph-medium">Low: {alertStats.alertCountBySeverity.low}</Text>
                </div>
              </div>
            </div>

            <div className={styles.summaryCard}>
              <Text variant="h4">Performance Metrics</Text>
              <div className={styles.performanceMetrics}>
                <div className={styles.metricItem}>
                  <Text variant="paragraph-small" color="secondary">Avg Response Time</Text>
                  <Text variant="paragraph-medium">{componentStats.averageResponseTime.toFixed(0)}ms</Text>
                </div>
                <div className={styles.metricItem}>
                  <Text variant="paragraph-small" color="secondary">Avg Availability</Text>
                  <Text variant="paragraph-medium">{componentStats.averageAvailability.toFixed(1)}%</Text>
                </div>
                <div className={styles.metricItem}>
                  <Text variant="paragraph-small" color="secondary">Grafana Dashboards</Text>
                  <Text variant="paragraph-medium">{grafanaDashboards.length}</Text>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Health Tab */}
      {activeTab === 'health' && (
        <UnifiedHealthDashboard />
      )}

      {/* Grafana Tab */}
      {activeTab === 'grafana' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Grafana Integration</Text>
          <Text variant="paragraph-medium" color="secondary">
            Grafana integration panel coming soon...
          </Text>
        </div>
      )}

      {/* Alerts Tab */}
      {activeTab === 'alerts' && (
        <AlertCorrelationDashboard />
      )}

      {/* Metrics Tab */}
      {activeTab === 'metrics' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Metrics Visualization</Text>
          <Text variant="paragraph-medium" color="secondary">
            Metrics visualization panel coming soon...
          </Text>
        </div>
      )}
    </div>
  );
}
