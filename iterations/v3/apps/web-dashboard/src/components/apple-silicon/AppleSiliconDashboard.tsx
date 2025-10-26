/**
 * Apple Silicon Performance Monitoring Dashboard
 * Comprehensive hardware monitoring and optimization interface
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { MetricCard, AnalyticsGrid } from '@/design-system/analytics';
import {
  Cpu,
  Zap,
  Thermometer,
  HardDrive,
  Activity,
  TrendingUp,
  Settings,
  RefreshCw,
  Monitor,
  Gauge,
  Layers
} from 'lucide-react';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';
import { useAppleSiliconStore, useAppleSiliconActions } from '@/stores/apple-silicon';
import { useAppleSiliconWebSocket, useRealTimeHardwareMonitoring, useRealTimeThermalMonitoring } from '@/hooks/useAppleSiliconWebSocket';
import { HardwareMetricsGrid } from './HardwareMetricsGrid';
import { ThermalManagementPanel } from './ThermalManagementPanel';
import { ModelPerformanceDashboard } from './ModelPerformanceDashboard';
import { RoutingVisualization } from './RoutingVisualization';
import styles from './AppleSiliconDashboard.module.scss';

export function AppleSiliconDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'hardware' | 'thermal' | 'models' | 'routing'>('overview');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { deviceStatus, recommendations } = useAppleSiliconStore();
  const actions = useAppleSiliconActions();
  const { isConnected } = useAppleSiliconWebSocket();

  // Real-time monitoring hooks
  const hardwareStats = useRealTimeHardwareMonitoring();
  const thermalStats = useRealTimeThermalMonitoring();

  // Fetch initial data
  useEffect(() => {
    fetchDashboardData();
  }, []);

  const fetchDashboardData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch device status
      actions.setLoading('metrics', true);
      const deviceData = await appleSiliconApiClient.getDeviceStatus();
      actions.setDeviceStatus(deviceData);

      // Fetch current metrics
      const metricsData = await appleSiliconApiClient.getCurrentMetrics();
      actions.setCurrentMetrics(metricsData);

      // Fetch active models
      actions.setLoading('models', true);
      const modelsData = await appleSiliconApiClient.getActiveModels();
      actions.setActiveModels(modelsData);

      // Fetch alerts
      actions.setLoading('alerts', true);
      const alertsData = await appleSiliconApiClient.getAlerts(false, 'high');
      actions.setAlerts(alertsData);

      // Fetch recommendations
      actions.setLoading('recommendations', true);
      const recommendationsData = await appleSiliconApiClient.getRecommendations();
      actions.setRecommendations(recommendationsData);

      // Fetch routing decisions
      actions.setLoading('routing', true);
      const routingData = await appleSiliconApiClient.getRoutingDecisions(20, '1h');
      actions.setRoutingDecisions(routingData);

    } catch (error) {
      console.error('Failed to fetch Apple Silicon dashboard data:', error);
      actions.setError('metrics', error instanceof Error ? error.message : 'Failed to fetch data');
    } finally {
      actions.setLoading('metrics', false);
      actions.setLoading('models', false);
      actions.setLoading('alerts', false);
      actions.setLoading('recommendations', false);
      actions.setLoading('routing', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchDashboardData();
  };

  // Mock overview metrics for demonstration
  const overviewMetrics = [
    {
      title: 'ANE Utilization',
      value: hardwareStats.utilization ? `${Math.round(hardwareStats.utilization.ane)}%` : 'N/A',
      subtitle: 'Neural Engine usage',
      change: { value: 5.2, type: 'increase' as const, period: 'vs last hour' },
      status: hardwareStats.utilization?.ane && hardwareStats.utilization.ane > 80 ? 'warning' as const : 'good' as const,
      trend: 'up' as const,
      icon: <Cpu size={20} />
    },
    {
      title: 'GPU Utilization',
      value: hardwareStats.utilization ? `${Math.round(hardwareStats.utilization.gpu)}%` : 'N/A',
      subtitle: 'Metal GPU usage',
      change: { value: -2.1, type: 'decrease' as const, period: 'vs last hour' },
      status: hardwareStats.utilization?.gpu && hardwareStats.utilization.gpu > 90 ? 'warning' as const : 'good' as const,
      trend: 'down' as const,
      icon: <Monitor size={20} />
    },
    {
      title: 'Thermal Status',
      value: thermalStats.isThrottling ? 'Throttling' : 'Optimal',
      subtitle: `${thermalStats.thermalMetrics?.cpuTemperature.toFixed(1) || 'N/A'}°C CPU`,
      change: { value: thermalStats.thermalAlerts.length, type: 'neutral' as const, period: 'active alerts' },
      status: thermalStats.isThrottling ? 'critical' as const : 'good' as const,
      trend: 'stable' as const,
      icon: <Thermometer size={20} />
    },
    {
      title: 'Power Efficiency',
      value: deviceStatus ? `${deviceStatus.powerEfficiency.toFixed(1)}` : 'N/A',
      subtitle: 'Performance/watt ratio',
      change: { value: 8.3, type: 'increase' as const, period: 'vs last week' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Zap size={20} />
    },
    {
      title: 'Active Models',
      value: deviceStatus?.activeModels.toString() || '0',
      subtitle: 'Models running',
      change: { value: 2, type: 'increase' as const, period: 'vs yesterday' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Activity size={20} />
    },
    {
      title: 'Memory Usage',
      value: hardwareStats.utilization ? `${Math.round(hardwareStats.utilization.memory)}%` : 'N/A',
      subtitle: 'Unified memory',
      change: { value: -3.4, type: 'decrease' as const, period: 'vs last hour' },
      status: hardwareStats.utilization?.memory && hardwareStats.utilization.memory > 85 ? 'warning' as const : 'good' as const,
      trend: 'down' as const,
      icon: <HardDrive size={20} />
    }
  ];

  return (
    <div className={styles.appleSiliconDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Apple Silicon Monitor</Text>
          <Text variant="paragraph-large" color="secondary">
            Hardware performance monitoring and optimization for Apple Silicon
          </Text>

          {deviceStatus && (
            <div className={styles.deviceInfo}>
              <Text variant="paragraph-medium">
                {deviceStatus.deviceInfo.chip} • {deviceStatus.deviceInfo.memory}GB •
                macOS {deviceStatus.deviceInfo.macosVersion}
              </Text>
              <div className={`statusBadge ${styles[deviceStatus.overallHealth]}`}>
                {deviceStatus.overallHealth.toUpperCase()}
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
                <Cpu size={12} />
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
              <Gauge size={16} />
              Overview
            </Button>
            <Button
              variant={activeTab === 'hardware' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('hardware')}
            >
              <Monitor size={16} />
              Hardware
            </Button>
            <Button
              variant={activeTab === 'thermal' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('thermal')}
            >
              <Thermometer size={16} />
              Thermal
            </Button>
            <Button
              variant={activeTab === 'models' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('models')}
            >
              <Layers size={16} />
              Models
            </Button>
            <Button
              variant={activeTab === 'routing' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('routing')}
            >
              <Activity size={16} />
              Routing
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
            title="Apple Silicon Performance Overview"
            subtitle="Real-time hardware utilization and performance metrics"
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

          {/* Quick Stats */}
          <div className={styles.quickStats}>
            <div className={styles.statCard}>
              <Text variant="h4">System Health</Text>
              <div className={styles.healthIndicators}>
                <div className={styles.healthItem}>
                  <span className={styles.healthLabel}>Temperature</span>
                  <span className={`${styles.healthValue} ${thermalStats.isThrottling ? styles.warning : styles.good}`}>
                    {thermalStats.thermalMetrics ? `${thermalStats.thermalMetrics.cpuTemperature}°C` : 'N/A'}
                  </span>
                </div>
                <div className={styles.healthItem}>
                  <span className={styles.healthLabel}>Utilization</span>
                  <span className={`${styles.healthValue} ${hardwareStats.utilization && (hardwareStats.utilization.ane + hardwareStats.utilization.gpu + hardwareStats.utilization.cpu + hardwareStats.utilization.memory) / 4 > 80 ? styles.warning : styles.good}`}>
                    {hardwareStats.utilization ? `${Math.round((hardwareStats.utilization.ane + hardwareStats.utilization.gpu + hardwareStats.utilization.cpu + hardwareStats.utilization.memory) / 4)}%` : 'N/A'}
                  </span>
                </div>
                <div className={styles.healthItem}>
                  <span className={styles.healthLabel}>Active Alerts</span>
                  <span className={`${styles.healthValue} ${thermalStats.thermalAlerts.length > 0 ? styles.error : styles.good}`}>
                    {thermalStats.thermalAlerts.length}
                  </span>
                </div>
              </div>
            </div>

            <div className={styles.statCard}>
              <Text variant="h4">Recommendations</Text>
              <div className={styles.recommendationsList}>
                {recommendations.slice(0, 3).map((rec) => (
                  <div key={rec.id} className={styles.recommendationItem}>
                    <div className={`priorityBadge ${styles[rec.priority]}`}>
                      {rec.priority.toUpperCase()}
                    </div>
                    <Text variant="paragraph-small">{rec.title}</Text>
                  </div>
                ))}
                {recommendations.length === 0 && (
                  <Text variant="paragraph-small" color="secondary">
                    No recommendations available
                  </Text>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Hardware Tab */}
      {activeTab === 'hardware' && (
        <HardwareMetricsGrid />
      )}

      {/* Thermal Tab */}
      {activeTab === 'thermal' && (
        <ThermalManagementPanel />
      )}

      {/* Models Tab */}
      {activeTab === 'models' && (
        <ModelPerformanceDashboard />
      )}

      {/* Routing Tab */}
      {activeTab === 'routing' && (
        <RoutingVisualization />
      )}
    </div>
  );
}
