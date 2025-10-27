/**
 * Judge Metrics Dashboard
 * AI judge performance monitoring and analytics
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { MetricCard, AnalyticsGrid } from '@/design-system/analytics';
import {
  Users,
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  Clock,
  Target,
  RefreshCw,
  Filter,
  Search
} from 'lucide-react';
import { Judge } from '@/lib/council-api';
import { councilApiClient } from '@/lib/council-api';
import { useCouncilStore, useCouncilActions } from '@/stores/council';
import { useCouncilWebSocket, useRealTimeJudgeMonitoring } from '@/hooks/useCouncilWebSocket';
import styles from './JudgeMetricsDashboard.module.scss';

export function JudgeMetricsDashboard() {
  const [selectedJudge, setSelectedJudge] = useState<Judge | null>(null);
  const [timeRange, setTimeRange] = useState<'24h' | '7d' | '30d'>('24h');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { judges, loading } = useCouncilStore();
  const actions = useCouncilActions();
  const { isConnected } = useCouncilWebSocket();

  // Real-time monitoring
  const judgeStats = useRealTimeJudgeMonitoring();

  // Fetch data on mount
  useEffect(() => {
    fetchJudgeData();
  }, [timeRange]);

  const fetchJudgeData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch judges
      actions.setLoading('judges', true);
      const judgesData = await councilApiClient.getJudges();
      actions.setJudges(judgesData);

      // Fetch performance metrics for each judge
      for (const judge of judgesData) {
        try {
          const performanceData = await councilApiClient.getJudgePerformance(judge.id, {
            start: new Date(Date.now() - getTimeRangeMs(timeRange)),
            end: new Date()
          });
          // Update judge with performance data (simplified for compatibility)
          actions.updateJudge(judge.id, {
            performance: {
              accuracy: performanceData.metrics.accuracy,
              responseTime: performanceData.metrics.responseTime.average, // Use average for compatibility
              consensusRate: performanceData.metrics.consensusRate,
              biasScore: performanceData.metrics.biasScore
            }
          });
        } catch (error) {
          console.warn(`Failed to fetch performance data for judge ${judge.id}:`, error);
        }
      }

    } catch (error) {
      console.error('Failed to fetch judge data:', error);
      actions.setError('judges', error instanceof Error ? error.message : 'Failed to fetch judge data');
    } finally {
      actions.setLoading('judges', false);
      setRefreshing(false);
    }
  };

  const getTimeRangeMs = (range: '24h' | '7d' | '30d') => {
    switch (range) {
      case '24h':
        return 24 * 60 * 60 * 1000;
      case '7d':
        return 7 * 24 * 60 * 60 * 1000;
      case '30d':
        return 30 * 24 * 60 * 60 * 1000;
      default:
        return 24 * 60 * 60 * 1000;
    }
  };

  const handleRefresh = async () => {
    await fetchJudgeData();
  };

  // Calculate aggregate metrics
  const aggregateMetrics = useMemo(() => {
    if (judges.length === 0) return null;

    const totalJudges = judges.length;
    const activeJudges = judges.filter(j => j.status === 'active').length;
    const avgAccuracy = judges.reduce((sum, j) => sum + j.performance.accuracy, 0) / totalJudges;
    const avgResponseTime = judges.reduce((sum, j) => sum + j.performance.responseTime, 0) / totalJudges;
    const avgConsensusRate = judges.reduce((sum, j) => sum + j.performance.consensusRate, 0) / totalJudges;

    return {
      totalJudges,
      activeJudges,
      avgAccuracy,
      avgResponseTime,
      avgConsensusRate
    };
  }, [judges]);

  // Mock metrics for demonstration (when real data is not available)
  const mockMetrics = aggregateMetrics ? [
    {
      title: 'Total Judges',
      value: aggregateMetrics.totalJudges.toString(),
      subtitle: 'Active AI judges',
      change: { value: 0, type: 'increase' as const, period: 'vs last month' },
      status: 'good' as const,
      trend: 'stable' as const,
      icon: <Users size={20} />
    },
    {
      title: 'Active Judges',
      value: aggregateMetrics.activeJudges.toString(),
      subtitle: `${Math.round((aggregateMetrics.activeJudges / aggregateMetrics.totalJudges) * 100)}% operational`,
      change: { value: 2, type: 'increase' as const, period: 'vs last week' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <CheckCircle size={20} />
    },
    {
      title: 'Avg Accuracy',
      value: `${Math.round(aggregateMetrics.avgAccuracy * 100)}%`,
      subtitle: 'Decision correctness',
      change: { value: 1.2, type: 'increase' as const, period: 'vs last month' },
      status: aggregateMetrics.avgAccuracy > 0.9 ? 'good' as const : 'warning' as const,
      trend: 'up' as const,
      icon: <Target size={20} />
    },
    {
      title: 'Avg Response Time',
      value: `${aggregateMetrics.avgResponseTime.toFixed(1)}s`,
      subtitle: 'Decision speed',
      change: { value: -0.3, type: 'decrease' as const, period: 'vs last month' },
      status: aggregateMetrics.avgResponseTime < 3 ? 'good' as const : 'warning' as const,
      trend: 'down' as const,
      icon: <Clock size={20} />
    },
    {
      title: 'Consensus Rate',
      value: `${Math.round(aggregateMetrics.avgConsensusRate * 100)}%`,
      subtitle: 'Judge agreement',
      change: { value: 0.8, type: 'increase' as const, period: 'vs last month' },
      status: aggregateMetrics.avgConsensusRate > 0.8 ? 'good' as const : 'warning' as const,
      trend: 'up' as const,
      icon: <Users size={20} />
    },
    {
      title: 'System Health',
      value: `${Math.round((judgeStats.activeJudges / judgeStats.totalJudges) * 100)}%`,
      subtitle: 'Operational status',
      change: { value: 0, type: 'increase' as const, period: 'stable' },
      status: judgeStats.errorJudges === 0 ? 'good' as const : 'critical' as const,
      trend: 'stable' as const,
      icon: <TrendingUp size={20} />
    }
  ] : [];


  const getJudgeStatusIcon = (status: Judge['status']) => {
    switch (status) {
      case 'active':
        return <CheckCircle size={16} />;
      case 'inactive':
        return <Clock size={16} />;
      case 'error':
        return <AlertTriangle size={16} />;
      default:
        return <Users size={16} />;
    }
  };

  return (
    <div className={styles.judgeDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Judge Performance</Text>
          <Text variant="paragraph-large" color="secondary">
            AI judge accuracy, reliability, and operational metrics
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
                <Users size={12} />
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

      {/* Aggregate Metrics */}
      {mockMetrics.length > 0 && (
        <AnalyticsGrid
          title="Judge Performance Overview"
          subtitle={`Metrics for ${timeRange} period`}
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
      )}

      {/* Judge List */}
      <div className={styles.judgeList}>
        <div className={styles.listHeader}>
          <Text variant="h4">Individual Judge Metrics</Text>
          <Text variant="paragraph-medium" color="secondary">
            {judges.length} judges • {judgeStats.activeJudges} active
          </Text>
        </div>

        {loading.judges ? (
          <div className={styles.loadingState}>
            <div className={styles.spinner}></div>
            <Text variant="paragraph-large">Loading judge metrics...</Text>
          </div>
        ) : judges.length === 0 ? (
          <div className={styles.emptyState}>
            <Users size={48} />
            <Text variant="h5">No judges found</Text>
            <Text variant="paragraph-medium" color="secondary">
              Judge metrics will appear here when available
            </Text>
          </div>
        ) : (
          <div className={styles.judgeGrid}>
            {judges.map((judge) => (
              <div
                key={judge.id}
                className={`${styles.judgeCard} ${selectedJudge?.id === judge.id ? styles.selected : ''}`}
                onClick={() => setSelectedJudge(judge)}
              >
                {/* Judge Header */}
                <div className={styles.judgeHeader}>
                  <div className={styles.judgeInfo}>
                    <Text variant="h5">{judge.name}</Text>
                    <Text variant="paragraph-small" color="secondary">
                      {judge.role.replace('_', ' ').toUpperCase()}
                    </Text>
                  </div>

                  <div className={`styles.statusBadge} styles[getJudgeStatusColor(judge.status)]`}>
                    {getJudgeStatusIcon(judge.status)}
                    <span>{judge.status.toUpperCase()}</span>
                  </div>
                </div>

                {/* Performance Metrics */}
                <div className={styles.performanceMetrics}>
                  <div className={styles.metric}>
                    <Text variant="paragraph-small">Accuracy</Text>
                    <Text variant="paragraph-medium">
                      {Math.round(judge.performance.accuracy * 100)}%
                    </Text>
                  </div>

                  <div className={styles.metric}>
                    <Text variant="paragraph-small">Response Time</Text>
                    <Text variant="paragraph-medium">
                      {judge.performance.responseTime.toFixed(1)}s
                    </Text>
                  </div>

                  <div className={styles.metric}>
                    <Text variant="paragraph-small">Consensus Rate</Text>
                    <Text variant="paragraph-medium">
                      {Math.round(judge.performance.consensusRate * 100)}%
                    </Text>
                  </div>
                </div>

                {/* Model Info */}
                <div className={styles.modelInfo}>
                  <Text variant="paragraph-small" color="secondary">
                    Model: {judge.model}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    Last Active: {new Date(judge.lastActive).toLocaleString()}
                  </Text>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Selected Judge Detail */}
      {selectedJudge && (
        <div className={styles.judgeDetail}>
          <div className={styles.detailHeader}>
            <Text variant="h4">{selectedJudge.name} - Detailed Metrics</Text>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setSelectedJudge(null)}
            >
              Close
            </Button>
          </div>

          <div className={styles.detailContent}>
            <div className={styles.detailGrid}>
              <div className={styles.detailCard}>
                <Text variant="h5">Performance Breakdown</Text>
                <div className={styles.detailMetrics}>
                  <div className={styles.detailMetric}>
                    <Text variant="paragraph-small">Accuracy Rate</Text>
                    <div className={styles.metricBar}>
                      <div
                        className={styles.metricFill}
                        style={{ width: `${selectedJudge.performance.accuracy * 100}%` }}
                      />
                    </div>
                    <Text variant="paragraph-small">
                      {Math.round(selectedJudge.performance.accuracy * 100)}%
                    </Text>
                  </div>

                  <div className={styles.detailMetric}>
                    <Text variant="paragraph-small">Response Time Distribution</Text>
                    <div className={styles.timeMetrics}>
                      <div className={styles.timeMetric}>
                        <Text variant="paragraph-small">P95</Text>
                        <Text variant="paragraph-medium">2.1s</Text>
                      </div>
                      <div className={styles.timeMetric}>
                        <Text variant="paragraph-small">P99</Text>
                        <Text variant="paragraph-medium">4.2s</Text>
                      </div>
                    </div>
                  </div>

                  <div className={styles.detailMetric}>
                    <Text variant="paragraph-small">Bias Score</Text>
                    <Text variant="paragraph-medium">
                      {selectedJudge.performance.biasScore.toFixed(3)}
                    </Text>
                    <Text variant="paragraph-small" color="secondary">
                      Lower is better
                    </Text>
                  </div>
                </div>
              </div>

              <div className={styles.detailCard}>
                <Text variant="h5">Recent Activity</Text>
                <div className={styles.activityList}>
                  {/* Mock activity data */}
                  <div className={styles.activityItem}>
                    <CheckCircle size={14} />
                    <div>
                      <Text variant="paragraph-small">Approved task #1234</Text>
                      <Text variant="paragraph-small" color="secondary">2 minutes ago</Text>
                    </div>
                  </div>
                  <div className={styles.activityItem}>
                    <AlertTriangle size={14} />
                    <div>
                      <Text variant="paragraph-small">High confidence override</Text>
                      <Text variant="paragraph-small" color="secondary">15 minutes ago</Text>
                    </div>
                  </div>
                  <div className={styles.activityItem}>
                    <Clock size={14} />
                    <div>
                      <Text variant="paragraph-small">Participated in consensus</Text>
                      <Text variant="paragraph-small" color="secondary">1 hour ago</Text>
                    </div>
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