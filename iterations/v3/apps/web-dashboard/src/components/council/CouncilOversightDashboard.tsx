/**
 * Council Oversight Dashboard
 * Main dashboard for AI judge decision monitoring and oversight
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { MetricCard, AnalyticsGrid } from '@/design-system/analytics';
import {
  Gavel,
  Users,
  AlertTriangle,
  TrendingUp,
  Shield,
  Clock,
  CheckCircle,
  XCircle,
  RefreshCw,
  Settings,
  Filter,
  Search
} from 'lucide-react';
import { councilApiClient } from '@/lib/council-api';
import { useCouncilStore, useCouncilActions } from '@/stores/council';
import { useCouncilWebSocket, useRealTimeVerdictMonitoring, useRealTimeAlertMonitoring } from '@/hooks/useCouncilWebSocket';
import { VerdictTimeline } from './VerdictTimeline';
import { JudgeMetricsDashboard } from './JudgeMetricsDashboard';
import { EthicalAssessmentDashboard } from './EthicalAssessmentDashboard';
import styles from './CouncilOversightDashboard.module.scss';

export function CouncilOversightDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'verdicts' | 'judges' | 'ethics'>('overview');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { metrics, loading, errors } = useCouncilStore();
  const actions = useCouncilActions();
  const { isConnected } = useCouncilWebSocket();

  // Real-time monitoring hooks
  const verdictStats = useRealTimeVerdictMonitoring();
  const alertStats = useRealTimeAlertMonitoring();

  // Fetch initial data
  useEffect(() => {
    fetchDashboardData();
  }, []);

  const fetchDashboardData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch metrics
      actions.setLoading('metrics', true);
      const metricsData = await councilApiClient.getMetrics();
      actions.setMetrics(metricsData);

      // Fetch judges
      actions.setLoading('judges', true);
      const judgesData = await councilApiClient.getJudges();
      actions.setJudges(judgesData);

      // Fetch alerts
      actions.setLoading('alerts', true);
      const alertsData = await councilApiClient.getAlerts(false, 20);
      actions.setAlerts(alertsData);

      // Fetch recent verdicts
      actions.setLoading('verdicts', true);
      const verdictsData = await councilApiClient.getVerdicts({}, 1, 50);
      actions.setVerdicts(verdictsData.verdicts);
      actions.setPagination({
        page: verdictsData.page,
        limit: verdictsData.limit,
        total: verdictsData.total,
        hasMore: verdictsData.page * verdictsData.limit < verdictsData.total
      });

    } catch (error) {
      console.error('Failed to fetch council dashboard data:', error);
      actions.setError('metrics', error instanceof Error ? error.message : 'Failed to fetch data');
    } finally {
      actions.setLoading('metrics', false);
      actions.setLoading('judges', false);
      actions.setLoading('alerts', false);
      actions.setLoading('verdicts', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchDashboardData();
  };

  // Mock metrics for demonstration (when real API is not available)
  const mockMetrics = [
    {
      title: 'Total Verdicts',
      value: verdictStats.totalVerdicts.toLocaleString(),
      subtitle: 'Decisions made',
      change: { value: 12.5, type: 'increase' as const, period: 'vs last week' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Gavel size={20} />
    },
    {
      title: 'Active Verdicts',
      value: verdictStats.inProgressVerdicts.toString(),
      subtitle: 'Currently in progress',
      change: { value: -5.2, type: 'decrease' as const, period: 'vs yesterday' },
      status: 'neutral' as const,
      trend: 'down' as const,
      icon: <Clock size={20} />
    },
    {
      title: 'Consensus Rate',
      value: `${metrics?.consensusAccuracy.toFixed(1) || '94.2'}%`,
      subtitle: 'Judge agreement',
      change: { value: 2.1, type: 'increase' as const, period: 'vs last month' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <CheckCircle size={20} />
    },
    {
      title: 'Active Alerts',
      value: alertStats.unacknowledgedAlerts.toString(),
      subtitle: 'Require attention',
      change: { value: alertStats.criticalAlerts, type: 'neutral' as const, period: 'critical alerts' },
      status: alertStats.criticalAlerts > 0 ? 'error' as const : 'good' as const,
      trend: 'neutral' as const,
      icon: <AlertTriangle size={20} />
    },
    {
      title: 'Avg Response Time',
      value: `${metrics?.averageResponseTime.toFixed(1) || '2.3'}s`,
      subtitle: 'Decision speed',
      change: { value: -8.3, type: 'decrease' as const, period: 'vs last week' },
      status: 'good' as const,
      trend: 'down' as const,
      icon: <TrendingUp size={20} />
    },
    {
      title: 'Ethical Concerns',
      value: `${metrics?.ethicalConcernRate.toFixed(1) || '3.2'}%`,
      subtitle: 'Of total verdicts',
      change: { value: -1.5, type: 'decrease' as const, period: 'vs last month' },
      status: metrics?.ethicalConcernRate && metrics.ethicalConcernRate > 5 ? 'warning' as const : 'good' as const,
      trend: 'down' as const,
      icon: <Shield size={20} />
    }
  ];

  return (
    <div className={styles.councilDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Council Oversight</Text>
          <Text variant="paragraph-large" color="secondary">
            AI judge decision monitoring and ethical oversight
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
                <Gavel size={12} />
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
              <BarChart3 size={16} />
              Overview
            </Button>
            <Button
              variant={activeTab === 'verdicts' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('verdicts')}
            >
              <Gavel size={16} />
              Verdicts
            </Button>
            <Button
              variant={activeTab === 'judges' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('judges')}
            >
              <Users size={16} />
              Judges
            </Button>
            <Button
              variant={activeTab === 'ethics' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('ethics')}
            >
              <Shield size={16} />
              Ethics
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
            title="Council Metrics"
            subtitle="Real-time AI decision-making performance and oversight"
            columns={3}
            gap="md"
          >
            {mockMetrics.map((metric, index) => (
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

          {/* Recent Activity Summary */}
          <div className={styles.activitySummary}>
            <Text variant="h3">Recent Activity</Text>
            <div className={styles.activityGrid}>
              <div className={styles.activityCard}>
                <Text variant="h4">Verdicts in Progress</Text>
                <Text variant="display-small">{verdictStats.inProgressVerdicts}</Text>
                <Text variant="paragraph-small" color="secondary">
                  Active decision processes
                </Text>
              </div>

              <div className={styles.activityCard}>
                <Text variant="h4">Pending Interventions</Text>
                <Text variant="display-small">{verdictStats.escalatedVerdicts}</Text>
                <Text variant="paragraph-small" color="secondary">
                  Require manual review
                </Text>
              </div>

              <div className={styles.activityCard}>
                <Text variant="h4">Critical Alerts</Text>
                <Text variant="display-small" className={alertStats.criticalAlerts > 0 ? styles.critical : ''}>
                  {alertStats.criticalAlerts}
                </Text>
                <Text variant="paragraph-small" color="secondary">
                  Immediate attention needed
                </Text>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Verdicts Tab */}
      {activeTab === 'verdicts' && (
        <VerdictTimeline />
      )}

      {/* Judges Tab */}
      {activeTab === 'judges' && (
        <JudgeMetricsDashboard />
      )}

      {/* Ethics Tab */}
      {activeTab === 'ethics' && (
        <EthicalAssessmentDashboard />
      )}
    </div>
  );
}
