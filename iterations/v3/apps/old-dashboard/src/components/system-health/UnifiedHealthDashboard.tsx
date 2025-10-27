/**
 * Unified Health Dashboard
 * Comprehensive system component health monitoring and status aggregation
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
  Server,
  Database,
  Cpu,
  HardDrive,
  Wifi,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  TrendingUp,
  RefreshCw,
  Settings,
  Filter
} from 'lucide-react';
import { systemHealthApiClient } from '@/lib/system-health-api';
import { useSystemHealthStore, useSystemHealthActions, useOverallSystemStatus, useActiveAlerts, useCriticalAlerts, useHealthyComponents, useUnhealthyComponents } from '@/stores/system-health';
import { useSystemHealthWebSocket, useRealTimeComponentMonitoring } from '@/hooks/useSystemHealthWebSocket';
import styles from './UnifiedHealthDashboard.module.scss';

interface ComponentStatusCardProps {
  component: any;
  onClick?: () => void;
}

const ComponentStatusCard: React.FC<ComponentStatusCardProps> = ({ component, onClick }) => {
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy':
        return <CheckCircle size={16} className={styles.healthy} />;
      case 'warning':
        return <AlertTriangle size={16} className={styles.warning} />;
      case 'critical':
        return <XCircle size={16} className={styles.critical} />;
      default:
        return <Clock size={16} className={styles.unknown} />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy':
        return 'good';
      case 'warning':
        return 'warning';
      case 'critical':
        return 'error';
      default:
        return 'neutral';
    }
  };

  return (
    <div className={styles.componentCard} onClick={onClick}>
      <div className={styles.componentHeader}>
        <div className={styles.componentInfo}>
          <Text variant="h4">{component.name}</Text>
          <Text variant="paragraph-small" color="secondary">{component.type.toUpperCase()}</Text>
        </div>
        {getStatusIcon(component.status)}
      </div>

      <div className={styles.componentMetrics}>
        <div className={styles.metric}>
          <Text variant="paragraph-small" color="secondary">Availability</Text>
          <Text variant="paragraph-medium">{component.availability.toFixed(1)}%</Text>
        </div>
        <div className={styles.metric}>
          <Text variant="paragraph-small" color="secondary">Response Time</Text>
          <Text variant="paragraph-medium">{component.responseTime.toFixed(0)}ms</Text>
        </div>
        <div className={styles.metric}>
          <Text variant="paragraph-small" color="secondary">Error Rate</Text>
          <Text variant="paragraph-medium">{component.errorRate.toFixed(2)}%</Text>
        </div>
      </div>

      <div className={styles.componentFooter}>
        <Text variant="paragraph-small" color="secondary">
          Last check: {new Date(component.lastCheck).toLocaleTimeString()}
        </Text>
      </div>
    </div>
  );
};

export function UnifiedHealthDashboard() {
  const [selectedComponent, setSelectedComponent] = useState<any>(null);
  const [filterType, setFilterType] = useState<string>('all');

  const { components, alerts, loading } = useSystemHealthStore();
  const actions = useSystemHealthActions();
  const { isConnected } = useSystemHealthWebSocket();

  const componentStats = useRealTimeComponentMonitoring();
  const overallStatus = useOverallSystemStatus();
  const activeAlerts = useActiveAlerts();
  const criticalAlerts = useCriticalAlerts();
  const healthyComponents = useHealthyComponents();
  const unhealthyComponents = useUnhealthyComponents();

  // Fetch component health data
  useEffect(() => {
    const fetchComponentHealth = async () => {
      try {
        actions.setLoading('components', true);
        const componentData = await systemHealthApiClient.getComponentHealth();
        actions.setComponents(componentData);
      } catch (error) {
        console.error('Failed to fetch component health:', error);
        actions.setError('components', error instanceof Error ? error.message : 'Failed to fetch component health');
      } finally {
        actions.setLoading('components', false);
      }
    };

    fetchComponentHealth();
  }, []);

  // Filter components by type
  const filteredComponents = components.filter(component => {
    if (filterType === 'all') return true;
    return component.type === filterType;
  });

  // Group components by status
  const componentsByStatus = {
    healthy: filteredComponents.filter(c => c.status === 'healthy'),
    warning: filteredComponents.filter(c => c.status === 'warning'),
    critical: filteredComponents.filter(c => c.status === 'critical'),
    unknown: filteredComponents.filter(c => c.status === 'unknown'),
  };

  const componentTypeOptions = [
    { value: 'all', label: 'All Components', count: components.length },
    { value: 'api', label: 'API Services', count: components.filter(c => c.type === 'api').length },
    { value: 'database', label: 'Databases', count: components.filter(c => c.type === 'database').length },
    { value: 'cache', label: 'Cache Systems', count: components.filter(c => c.type === 'cache').length },
    { value: 'worker', label: 'Worker Processes', count: components.filter(c => c.type === 'worker').length },
    { value: 'model', label: 'AI Models', count: components.filter(c => c.type === 'model').length },
    { value: 'inference', label: 'Inference Engines', count: components.filter(c => c.type === 'inference').length },
  ];

  return (
    <div className={styles.unifiedHealthDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">System Health Overview</Text>
          <Text variant="paragraph-large" color="secondary">
            Real-time monitoring of all system components and services
          </Text>

          {/* Overall Status Indicator */}
          <div className={styles.overallStatus}>
            <div className={`statusIndicator ${overallStatus}`}>
              <Activity size={16} />
              <span>System Status: {overallStatus.toUpperCase()}</span>
            </div>
          </div>
        </div>

        <div className={styles.headerRight}>
          {/* Component Type Filter */}
          <div className={styles.filterControls}>
            <Text variant="paragraph-medium" color="secondary">Filter by Type:</Text>
            <div className={styles.filterButtons}>
              {componentTypeOptions.map(option => (
                <Button
                  key={option.value}
                  variant={filterType === option.value ? 'primary' : 'secondary'}
                  size="sm"
                  onClick={() => setFilterType(option.value)}
                >
                  {option.label} ({option.count})
                </Button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Health Summary Cards */}
      <AnalyticsGrid
        title="Health Summary"
        subtitle="Overall system health metrics and component status"
        columns={4}
        gap="md"
      >
        <MetricCard
          title="Total Components"
          value={componentStats.componentCount.toString()}
          subtitle={`${componentStats.healthyCount} healthy`}
          change={{
            value: componentStats.healthyCount,
            type: 'neutral' as const,
            period: 'operational'
          }}
          status="good"
          trend="stable"
          icon={<Server size={20} />}
        />

        <MetricCard
          title="Healthy Components"
          value={componentStats.healthyCount.toString()}
          subtitle={`${((componentStats.healthyCount / componentStats.componentCount) * 100).toFixed(1)}% of total`}
          change={{
            value: componentStats.healthyCount,
            type: 'increase' as const,
            period: 'stable'
          }}
          status="good"
          trend="up"
          icon={<CheckCircle size={20} />}
        />

        <MetricCard
          title="Unhealthy Components"
          value={componentStats.warningCount + componentStats.criticalCount}
          subtitle={`${componentStats.warningCount} warning, ${componentStats.criticalCount} critical`}
          change={{
            value: componentStats.warningCount + componentStats.criticalCount,
            type: 'neutral' as const,
            period: 'requires attention'
          }}
          status={componentStats.criticalCount > 0 ? 'error' : componentStats.warningCount > 0 ? 'warning' : 'good'}
          trend="neutral"
          icon={<AlertTriangle size={20} />}
        />

        <MetricCard
          title="Avg Response Time"
          value={`${componentStats.averageResponseTime.toFixed(0)}ms`}
          subtitle="Across all components"
          change={{
            value: -5.2,
            type: 'decrease' as const,
            period: 'vs last hour'
          }}
          status={componentStats.averageResponseTime < 500 ? 'good' : componentStats.averageResponseTime < 1000 ? 'warning' : 'error'}
          trend="down"
          icon={<TrendingUp size={20} />}
        />
      </AnalyticsGrid>

      {/* Component Status Overview */}
      <div className={styles.componentOverview}>
        <div className={styles.overviewHeader}>
          <Text variant="h3">Component Status</Text>
          <div className={styles.statusLegend}>
            <div className={styles.legendItem}>
              <CheckCircle size={12} className={styles.healthy} />
              <Text variant="paragraph-small">Healthy ({componentsByStatus.healthy.length})</Text>
            </div>
            <div className={styles.legendItem}>
              <AlertTriangle size={12} className={styles.warning} />
              <Text variant="paragraph-small">Warning ({componentsByStatus.warning.length})</Text>
            </div>
            <div className={styles.legendItem}>
              <XCircle size={12} className={styles.critical} />
              <Text variant="paragraph-small">Critical ({componentsByStatus.critical.length})</Text>
            </div>
            <div className={styles.legendItem}>
              <Clock size={12} className={styles.unknown} />
              <Text variant="paragraph-small">Unknown ({componentsByStatus.unknown.length})</Text>
            </div>
          </div>
        </div>

        {/* Component Grid */}
        <div className={styles.componentGrid}>
          {filteredComponents.map(component => (
            <ComponentStatusCard
              key={component.id}
              component={component}
              onClick={() => setSelectedComponent(component)}
            />
          ))}
        </div>

        {filteredComponents.length === 0 && (
          <div className={styles.emptyState}>
            <Server size={48} />
            <Text variant="h4">No Components Found</Text>
            <Text variant="paragraph-medium" color="secondary">
              No components match the current filter criteria.
            </Text>
          </div>
        )}
      </div>

      {/* Component Detail Modal */}
      {selectedComponent && (
        <div className={styles.modalOverlay} onClick={() => setSelectedComponent(null)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <Text variant="h3">{selectedComponent.name}</Text>
              <Button variant="secondary" size="sm" onClick={() => setSelectedComponent(null)}>
                ×
              </Button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.componentDetails}>
                <div className={styles.detailSection}>
                  <Text variant="h4">Status Information</Text>
                  <div className={styles.detailGrid}>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Status</Text>
                      <div className={styles.statusBadge}>
                        {selectedComponent.status === 'healthy' && <CheckCircle size={12} className={styles.healthy} />}
                        {selectedComponent.status === 'warning' && <AlertTriangle size={12} className={styles.warning} />}
                        {selectedComponent.status === 'critical' && <XCircle size={12} className={styles.critical} />}
                        <Text variant="paragraph-medium">{selectedComponent.status.toUpperCase()}</Text>
                      </div>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Type</Text>
                      <Text variant="paragraph-medium">{selectedComponent.type.toUpperCase()}</Text>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Availability</Text>
                      <Text variant="paragraph-medium">{selectedComponent.availability.toFixed(1)}%</Text>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Response Time</Text>
                      <Text variant="paragraph-medium">{selectedComponent.responseTime.toFixed(0)}ms</Text>
                    </div>
                  </div>
                </div>

                <div className={styles.detailSection}>
                  <Text variant="h4">Metrics</Text>
                  <div className={styles.metricsGrid}>
                    {selectedComponent.metrics?.map((metric: any, index: number) => (
                      <div key={index} className={styles.metricCard}>
                        <Text variant="paragraph-medium">{metric.name}</Text>
                        <Text variant="h4">{metric.value} {metric.unit}</Text>
                        <div className={styles.metricChange}>
                          <TrendingUp size={12} />
                          <Text variant="paragraph-small">{metric.trend === 'up' ? '+' : ''}{metric.change || 0}%</Text>
                        </div>
                      </div>
                    )) || (
                      <Text variant="paragraph-medium" color="secondary">No metrics available</Text>
                    )}
                  </div>
                </div>

                <div className={styles.detailSection}>
                  <Text variant="h4">Dependencies</Text>
                  <div className={styles.dependenciesList}>
                    {selectedComponent.dependencies?.length > 0 ? (
                      selectedComponent.dependencies.map((dep: string, index: number) => (
                        <div key={index} className={styles.dependencyItem}>
                          <Text variant="paragraph-medium">{dep}</Text>
                        </div>
                      ))
                    ) : (
                      <Text variant="paragraph-medium" color="secondary">No dependencies</Text>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
