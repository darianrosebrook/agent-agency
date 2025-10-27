/**
 * Model Performance Dashboard
 * Comprehensive model performance analytics and monitoring
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Progress } from '@/design-system/primitives';
import {
  Brain,
  TrendingUp,
  TrendingDown,
  Activity,
  Clock,
  Zap,
  Target,
  BarChart3,
  RefreshCw,
  Settings,
  Download,
  Filter
} from 'lucide-react';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';
import { useAppleSiliconWebSocket } from '@/hooks/useAppleSiliconWebSocket';
import { useAppleSiliconStore } from '@/stores/apple-silicon';
import styles from './ModelPerformanceDashboard.module.scss';

export function ModelPerformanceDashboard() {
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h'>('1h');
  const [refreshing, setRefreshing] = useState(false);
  const [modelData, setModelData] = useState<any>(null);
  const [modelHistory, setModelHistory] = useState<any[]>([]);
  const [modelMetrics, setModelMetrics] = useState<any[]>([]);
  const [modelAlerts, setModelAlerts] = useState<any[]>([]);

  const { isConnected, lastUpdate } = useAppleSiliconWebSocket();
  const { activeModels, modelPerformance } = useAppleSiliconStore();

  useEffect(() => {
    loadModelData();
  }, [timeRange]);

  const loadModelData = async () => {
    try {
      setRefreshing(true);
      const [models, history, metrics, alerts] = await Promise.all([
        appleSiliconApiClient.getActiveModels(),
        appleSiliconApiClient.getModelHistory(timeRange),
        appleSiliconApiClient.getModelMetrics(),
        appleSiliconApiClient.getModelAlerts(timeRange)
      ]);

      setModelData(models);
      setModelHistory(history);
      setModelMetrics(metrics);
      setModelAlerts(alerts);
    } catch (error) {
      console.error('Failed to load model data:', error);
    } finally {
      setRefreshing(false);
    }
  };

  const getModelStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'green';
      case 'idle':
        return 'orange';
      case 'error':
        return 'red';
      default:
        return 'gray';
    }
  };

  const getModelStatusIcon = (status: string) => {
    switch (status) {
      case 'active':
        return <Activity size={16} />;
      case 'idle':
        return <Clock size={16} />;
      case 'error':
        return <Zap size={16} />;
      default:
        return <Brain size={16} />;
    }
  };

  const formatLatency = (latency: number) => {
    return `${latency.toFixed(2)}ms`;
  };

  const formatThroughput = (throughput: number) => {
    return `${throughput.toFixed(1)} req/s`;
  };

  const formatAccuracy = (accuracy: number) => {
    return `${(accuracy * 100).toFixed(1)}%`;
  };

  const formatTimestamp = (timestamp: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(timestamp));
  };

  return (
    <div className={styles.modelPerformanceDashboard}>
      {/* Header */}
      <div className={styles.dashboardHeader}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Model Performance Dashboard</Text>
          <Text variant="paragraph-medium" color="secondary">
            Comprehensive model performance analytics and monitoring
          </Text>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.connectionStatus}>
            <Activity size={16} />
            <Text variant="paragraph-small">
              {isConnected ? 'Connected' : 'Disconnected'}
            </Text>
          </div>

          <div className={styles.timeRangeSelector}>
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as '1h' | '6h' | '24h')}
              className={styles.timeRangeSelect}
            >
              <option value="1h">Last Hour</option>
              <option value="6h">Last 6 Hours</option>
              <option value="24h">Last 24 Hours</option>
            </select>
          </div>

          <div className={styles.actions}>
            <Button
              variant="secondary"
              size="sm"
              onClick={loadModelData}
              disabled={refreshing}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
            <Button variant="secondary" size="sm">
              <Settings size={14} />
              Settings
            </Button>
          </div>
        </div>
      </div>

      {/* Model Overview */}
      <div className={styles.modelOverviewSection}>
        <div className={styles.overviewHeader}>
          <Text variant="h4">Model Overview</Text>
          <div className={styles.overviewControls}>
            <Button variant="secondary" size="sm">
              <Filter size={14} />
              Filter Models
            </Button>
          </div>
        </div>

        <div className={styles.modelGrid}>
          {activeModels.map((model, index) => (
            <div key={index} className={styles.modelCard}>
              <div className={styles.modelHeader}>
                <div className={styles.modelIcon}>
                  <Brain size={20} />
                </div>
                <div className={styles.modelInfo}>
                  <Text variant="paragraph-medium" className={styles.modelName}>
                    {model.name}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    {model.type}
                  </Text>
                </div>
                <div className={styles.modelStatus}>
                  <span className={`${styles.statusBadge} ${styles[getModelStatusColor(model.status)]}`}>
                    {model.status}
                  </span>
                </div>
              </div>

              <div className={styles.modelMetrics}>
                <div className={styles.metricItem}>
                  <Text variant="label">Latency</Text>
                  <Text variant="paragraph-medium">{formatLatency(model.latency)}</Text>
                </div>
                <div className={styles.metricItem}>
                  <Text variant="label">Throughput</Text>
                  <Text variant="paragraph-medium">{formatThroughput(model.throughput)}</Text>
                </div>
                <div className={styles.metricItem}>
                  <Text variant="label">Accuracy</Text>
                  <Text variant="paragraph-medium">{formatAccuracy(model.accuracy)}</Text>
                </div>
              </div>

              <div className={styles.modelProgress}>
                <div className={styles.progressItem}>
                  <Text variant="label">CPU Usage</Text>
                  <Progress
                    value={model.cpuUsage}
                    max={100}
                    className={styles.progressBar}
                  />
                </div>
                <div className={styles.progressItem}>
                  <Text variant="label">Memory Usage</Text>
                  <Progress
                    value={model.memoryUsage}
                    max={100}
                    className={styles.progressBar}
                  />
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Performance Metrics */}
      <div className={styles.performanceMetricsSection}>
        <div className={styles.metricsHeader}>
          <Text variant="h4">Performance Metrics</Text>
          <div className={styles.metricsControls}>
            <Button variant="secondary" size="sm">
              <BarChart3 size={14} />
              View Details
            </Button>
          </div>
        </div>

        <div className={styles.metricsGrid}>
          {modelMetrics.map((metric, index) => (
            <div key={index} className={styles.metricCard}>
              <div className={styles.metricHeader}>
                <div className={styles.metricIcon}>
                  {metric.trend === 'up' ? <TrendingUp size={20} /> : <TrendingDown size={20} />}
                </div>
                <div className={styles.metricInfo}>
                  <Text variant="paragraph-medium" className={styles.metricName}>
                    {metric.name}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    {metric.description}
                  </Text>
                </div>
                <div className={styles.metricTrend}>
                  <span className={`${styles.trendBadge} ${styles[metric.trend]}`}>
                    {metric.trend === 'up' ? '+' : '-'}{metric.change}%
                  </span>
                </div>
              </div>

              <div className={styles.metricValue}>
                <Text variant="display-small">{metric.value}</Text>
                <Text variant="paragraph-small" color="secondary">
                  {metric.unit}
                </Text>
              </div>

              <div className={styles.metricProgress}>
                <Progress
                  value={metric.percentage}
                  max={100}
                  className={styles.progressBar}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Performance History Chart */}
      <div className={styles.performanceHistorySection}>
        <div className={styles.historyHeader}>
          <Text variant="h4">Performance History</Text>
          <div className={styles.historyControls}>
            <Button variant="secondary" size="sm">
              <Download size={14} />
              Export Data
            </Button>
          </div>
        </div>

        <div className={styles.historyChart}>
          <div className={styles.chartContainer}>
            <div className={styles.chartPlaceholder}>
              <BarChart3 size={48} />
              <Text variant="h5">Performance History Chart</Text>
              <Text variant="paragraph-medium" color="secondary">
                Model performance metrics over time
              </Text>
            </div>
          </div>
        </div>
      </div>

      {/* Model Alerts */}
      <div className={styles.modelAlertsSection}>
        <div className={styles.alertsHeader}>
          <Text variant="h4">Model Alerts</Text>
          <div className={styles.alertsControls}>
            <Button variant="secondary" size="sm">
              <Activity size={14} />
              View All Alerts
            </Button>
          </div>
        </div>

        <div className={styles.alertsList}>
          {modelAlerts.map((alert, index) => (
            <div key={index} className={styles.alertCard}>
              <div className={styles.alertHeader}>
                <div className={styles.alertIcon}>
                  {getModelStatusIcon(alert.severity)}
                </div>
                <div className={styles.alertInfo}>
                  <Text variant="paragraph-medium" className={styles.alertTitle}>
                    {alert.title}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    {formatTimestamp(alert.timestamp)}
                  </Text>
                </div>
                <div className={styles.alertSeverity}>
                  <span className={`${styles.severityBadge} ${styles[alert.severity]}`}>
                    {alert.severity}
                  </span>
                </div>
              </div>
              <Text variant="paragraph-small" color="secondary" className={styles.alertDescription}>
                {alert.description}
              </Text>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
