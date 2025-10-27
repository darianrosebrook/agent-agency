/**
 * Ethical Assessment Dashboard
 * Monitor ethical considerations and stakeholder impacts
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
  TrendingUp,
  Eye,
  RefreshCw,
  Filter,
  Search,
  Target,
  AlertCircle
} from 'lucide-react';
import { councilApiClient } from '@/lib/council-api';
import { useCouncilStore, useCouncilActions } from '@/stores/council';
import { useCouncilWebSocket, useRealTimeAlertMonitoring } from '@/hooks/useCouncilWebSocket';
import styles from './EthicalAssessmentDashboard.module.scss';

export function EthicalAssessmentDashboard() {
  const [timeRange, setTimeRange] = useState<'24h' | '7d' | '30d'>('24h');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { alerts, loading, errors } = useCouncilStore();
  const actions = useCouncilActions();
  const { isConnected } = useCouncilWebSocket();

  // Real-time monitoring
  const alertStats = useRealTimeAlertMonitoring();

  // Fetch data on mount
  useEffect(() => {
    fetchEthicalData();
  }, [timeRange]);

  const fetchEthicalData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch alerts
      actions.setLoading('alerts', true);
      const alertsData = await councilApiClient.getAlerts(false, 50);
      actions.setAlerts(alertsData);

      // Fetch ethical assessments
      const ethicalData = await councilApiClient.getEthicalAssessments();

    } catch (error) {
      console.error('Failed to fetch ethical data:', error);
      actions.setError('alerts', error instanceof Error ? error.message : 'Failed to fetch ethical data');
    } finally {
      actions.setLoading('alerts', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchEthicalData();
  };

  // Mock ethical metrics for demonstration
  const mockEthicalMetrics = [
    {
      title: 'High Risk Verdicts',
      value: '12',
      subtitle: 'Require immediate attention',
      change: { value: -3, type: 'decrease' as const, period: 'vs last week' },
      status: 'warning' as const,
      trend: 'down' as const,
      icon: <AlertTriangle size={20} />
    },
    {
      title: 'Ethical Compliance',
      value: '96.8%',
      subtitle: 'Overall compliance rate',
      change: { value: 1.2, type: 'increase' as const, period: 'vs last month' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Shield size={20} />
    },
    {
      title: 'Stakeholder Impact',
      value: '1.2k',
      subtitle: 'People affected this month',
      change: { value: 8.5, type: 'increase' as const, period: 'vs last month' },
      status: 'neutral' as const,
      trend: 'up' as const,
      icon: <Users size={20} />
    },
    {
      title: 'Active Concerns',
      value: alertStats.unacknowledgedAlerts.toString(),
      subtitle: 'Unresolved ethical issues',
      change: { value: -2, type: 'decrease' as const, period: 'vs yesterday' },
      status: alertStats.criticalAlerts > 0 ? 'error' as const : 'warning' as const,
      trend: 'down' as const,
      icon: <AlertCircle size={20} />
    },
    {
      title: 'Bias Detection',
      value: '0.023',
      subtitle: 'Average bias score',
      change: { value: -0.005, type: 'decrease' as const, period: 'vs last month' },
      status: 'good' as const,
      trend: 'down' as const,
      icon: <Target size={20} />
    },
    {
      title: 'Transparency Score',
      value: '94.2%',
      subtitle: 'Decision explainability',
      change: { value: 2.1, type: 'increase' as const, period: 'vs last month' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Eye size={20} />
    }
  ];

  const getAlertSeverityColor = (severity: 'low' | 'medium' | 'high' | 'critical') => {
    switch (severity) {
      case 'critical':
        return 'error';
      case 'high':
        return 'error';
      case 'medium':
        return 'warning';
      case 'low':
        return 'neutral';
      default:
        return 'neutral';
    }
  };

  const getAlertSeverityIcon = (severity: 'low' | 'medium' | 'high' | 'critical') => {
    return <AlertTriangle size={16} />;
  };

  const acknowledgeAlert = async (alertId: string) => {
    try {
      await councilApiClient.acknowledgeAlert(alertId);
      actions.acknowledgeAlert(alertId);
    } catch (error) {
      console.error('Failed to acknowledge alert:', error);
    }
  };

  return (
    <div className={styles.ethicalDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Ethical Assessment</Text>
          <Text variant="paragraph-large" color="secondary">
            Monitor ethical considerations and stakeholder impacts
          </Text>
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

          {/* Time Range Selector */}
          <div className={styles.timeRangeSelector}>
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as typeof timeRange)}
              className={styles.timeRangeSelect}
            >
              <option value="24h">Last 24 Hours</option>
              <option value="7d">Last 7 Days</option>
              <option value="30d">Last 30 Days</option>
            </select>
          </div>

          {/* Actions */}
          <div className={styles.actions}>
            <Button variant="secondary" size="sm">
              <Filter size={16} />
            </Button>
            <Button variant="secondary" size="sm">
              <Search size={16} />
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

      {/* Ethical Metrics */}
      <AnalyticsGrid
        title="Ethical Oversight Metrics"
        subtitle={`Key ethical indicators for ${timeRange} period`}
        columns={3}
        gap="md"
      >
        {mockEthicalMetrics.map((metric, index) => (
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

      {/* Active Alerts */}
      <div className={styles.alertsSection}>
        <div className={styles.sectionHeader}>
          <Text variant="h4">Active Ethical Alerts</Text>
          <Text variant="paragraph-medium" color="secondary">
            {alertStats.unacknowledgedAlerts} unacknowledged • {alertStats.criticalAlerts} critical
          </Text>
        </div>

        {loading.alerts ? (
          <div className={styles.loadingState}>
            <div className={styles.spinner}></div>
            <Text variant="paragraph-large">Loading alerts...</Text>
          </div>
        ) : alertStats.unacknowledgedAlerts === 0 ? (
          <div className={styles.emptyState}>
            <Shield size={48} />
            <Text variant="h5">No active alerts</Text>
            <Text variant="paragraph-medium" color="secondary">
              All ethical concerns have been addressed
            </Text>
          </div>
        ) : (
          <div className={styles.alertsList}>
            {alerts
              .filter(alert => !alert.acknowledged)
              .map((alert) => (
                <div key={alert.id} className={styles.alertCard}>
                  <div className={styles.alertHeader}>
                    <div className={`styles.alertSeverity} styles[getAlertSeverityColor(alert.severity)]`}>
                      {getAlertSeverityIcon(alert.severity)}
                      <span>{alert.severity.toUpperCase()}</span>
                    </div>

                    <div className={styles.alertType}>
                      <Text variant="paragraph-small" color="secondary">
                        {alert.type.replace('_', ' ').toUpperCase()}
                      </Text>
                    </div>
                  </div>

                  <div className={styles.alertContent}>
                    <Text variant="paragraph-medium">{alert.message}</Text>
                    <Text variant="paragraph-small" color="secondary">
                      {new Date(alert.createdAt).toLocaleString()}
                    </Text>
                  </div>

                  <div className={styles.alertActions}>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => acknowledgeAlert(alert.id)}
                    >
                      Acknowledge
                    </Button>
                    {alert.verdictId && (
                      <Button variant="secondary" size="sm">
                        View Verdict
                      </Button>
                    )}
                  </div>
                </div>
              ))}
          </div>
        )}
      </div>

      {/* Ethical Categories Breakdown */}
      <div className={styles.categoriesSection}>
        <Text variant="h4">Ethical Concern Categories</Text>
        <div className={styles.categoriesGrid}>
          {[
            { category: 'Privacy', count: 45, percentage: 36, color: '#3B82F6' },
            { category: 'Bias', count: 28, percentage: 22, color: '#EF4444' },
            { category: 'Safety', count: 22, percentage: 18, color: '#F59E0B' },
            { category: 'Fairness', count: 18, percentage: 14, color: '#10B981' },
            { category: 'Transparency', count: 12, percentage: 10, color: '#8B5CF6' }
          ].map((category) => (
            <div key={category.category} className={styles.categoryCard}>
              <div className={styles.categoryHeader}>
                <div
                  className={styles.categoryColor}
                  style={{ backgroundColor: category.color }}
                />
                <Text variant="h5">{category.category}</Text>
              </div>

              <div className={styles.categoryStats}>
                <Text variant="display-medium">{category.count}</Text>
                <Text variant="paragraph-small" color="secondary">
                  {category.percentage}% of concerns
                </Text>
              </div>

              <div className={styles.categoryBar}>
                <div
                  className={styles.categoryFill}
                  style={{
                    width: `${category.percentage}%`,
                    backgroundColor: category.color
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Stakeholder Impact Analysis */}
      <div className={styles.impactSection}>
        <Text variant="h4">Stakeholder Impact Analysis</Text>
        <div className={styles.impactGrid}>
          <div className={styles.impactCard}>
            <Text variant="h5">Individual Impact</Text>
            <div className={styles.impactMetrics}>
              <div className={styles.impactMetric}>
                <Text variant="display-medium">1,247</Text>
                <Text variant="paragraph-small" color="secondary">People affected</Text>
              </div>
              <div className={styles.impactMetric}>
                <Text variant="display-medium">89%</Text>
                <Text variant="paragraph-small" color="secondary">Positive outcomes</Text>
              </div>
            </div>
          </div>

          <div className={styles.impactCard}>
            <Text variant="h5">Organizational Impact</Text>
            <div className={styles.impactMetrics}>
              <div className={styles.impactMetric}>
                <Text variant="display-medium">156</Text>
                <Text variant="paragraph-small" color="secondary">Organizations</Text>
              </div>
              <div className={styles.impactMetric}>
                <Text variant="display-medium">94%</Text>
                <Text variant="paragraph-small" color="secondary">Compliance rate</Text>
              </div>
            </div>
          </div>

          <div className={styles.impactCard}>
            <Text variant="h5">Societal Impact</Text>
            <div className={styles.impactMetrics}>
              <div className={styles.impactMetric}>
                <Text variant="display-medium">High</Text>
                <Text variant="paragraph-small" color="secondary">Overall benefit</Text>
              </div>
              <div className={styles.impactMetric}>
                <Text variant="display-medium">2.1x</Text>
                <Text variant="paragraph-small" color="secondary">Efficiency gain</Text>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
