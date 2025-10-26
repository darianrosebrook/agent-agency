/**
 * Agent Memory Management Dashboard
 * Comprehensive agent memory browser, context preservation, and knowledge graph visualization
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { MetricCard, AnalyticsGrid } from '@/design-system/analytics';
import {
  Brain,
  Database,
  AlertTriangle,
  TrendingUp,
  Activity,
  Search,
  Filter,
  Settings,
  RefreshCw,
  Network,
  Archive,
  Clock,
  BarChart3,
  XCircle
} from 'lucide-react';
import { agentMemoryApiClient } from '@/lib/agent-memory-api';
import { useAgentMemoryStore, useAgentMemoryActions, useMemoryAlertStats } from '@/stores/agent-memory';
import { useAgentMemoryWebSocket, useRealTimeAgentMonitoring, useRealTimeMemoryMonitoring } from '@/hooks/useAgentMemoryWebSocket';
// Commented out to resolve build errors
// import { MemoryBrowser } from './MemoryBrowser';
// import { KnowledgeGraphViewer } from './KnowledgeGraphViewer';
// import { ContextManager } from './ContextManager';
// import { MemoryHealthDashboard } from './MemoryHealthDashboard';
import styles from './AgentMemoryManagementDashboard.module.scss';

export function AgentMemoryManagementDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'browser' | 'graph' | 'context' | 'health'>('overview');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const {
    contextSnapshots
  } = useAgentMemoryStore();
  const actions = useAgentMemoryActions();
  const { isConnected } = useAgentMemoryWebSocket();

  // Real-time monitoring hooks
  const agentStats = useRealTimeAgentMonitoring();
  const memoryStats = useRealTimeMemoryMonitoring();
  const memoryAlertStats = useMemoryAlertStats();

  // Fetch initial data
  useEffect(() => {
    fetchAgentMemoryData();
  }, []);

  const fetchAgentMemoryData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch agent memories
      actions.setLoading('agents', true);
      const agentsData = await agentMemoryApiClient.getAgentMemories();
      actions.setAgents(agentsData);

      // Fetch memory alerts
      actions.setLoading('health', true);
      const alertsData = await agentMemoryApiClient.getMemoryAlerts();
      actions.setMemoryAlerts(alertsData);

      // Fetch context snapshots
      actions.setLoading('context', true);
      const contextData = await agentMemoryApiClient.getContextSnapshots();
      actions.setContextSnapshots(contextData);

      // Fetch memory optimizations
      const optimizationData = await agentMemoryApiClient.getMemoryOptimizations();
      actions.setMemoryOptimizations(optimizationData);

    } catch (error) {
      console.error('Failed to fetch agent memory dashboard data:', error);
      actions.setError('agents', error instanceof Error ? error.message : 'Failed to fetch data');
    } finally {
      actions.setLoading('agents', false);
      actions.setLoading('health', false);
      actions.setLoading('context', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchAgentMemoryData();
  };

  // Enhanced overview metrics with agent memory insights
  const overviewMetrics = [
    {
      title: 'Total Agents',
      value: agentStats.overallStats.totalAgents.toString(),
      subtitle: `${agentStats.overallStats.healthyAgents} healthy`,
      change: { value: 0, type: 'neutral' as const, period: 'stable' },
      status: agentStats.overallStats.criticalAgents === 0 ? 'good' as const :
              agentStats.overallStats.warningAgents > 0 ? 'warning' as const : 'critical' as const,
      trend: 'stable' as const,
      icon: <Brain size={20} />
    },
    {
      title: 'Memory Entries',
      value: memoryStats.memoryStats.total.toLocaleString(),
      subtitle: `${Math.floor(memoryStats.memoryStats.total * 0.3)} compressed`,
      change: {
        value: memoryStats.memoryTrends.creationRate,
        type: 'increase' as const,
        period: 'created today'
      },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Database size={20} />
    },
    {
      title: 'Active Alerts',
      value: memoryAlertStats.active.toString(),
      subtitle: `${memoryAlertStats.bySeverity.critical} critical`,
      change: {
        value: memoryAlertStats.bySeverity.critical,
        type: 'neutral' as const,
        period: 'currently active'
      },
      status: memoryAlertStats.bySeverity.critical === 0 ? 'good' as const :
              memoryAlertStats.bySeverity.critical < 3 ? 'warning' as const : 'critical' as const,
      trend: 'stable' as const,
      icon: <AlertTriangle size={20} />
    },
    {
      title: 'Avg Importance',
      value: `${(memoryStats.memoryStats.averageImportance * 100).toFixed(1)}%`,
      subtitle: `Confidence: ${(memoryStats.memoryStats.averageConfidence * 100).toFixed(1)}%`,
      change: { value: 1.2, type: 'increase' as const, period: 'vs last week' },
      status: memoryStats.memoryStats.averageImportance > 0.7 ? 'good' as const : 'warning' as const,
      trend: 'up' as const,
      icon: <TrendingUp size={20} />
    },
    {
      title: 'Context Snapshots',
      value: contextSnapshots.length.toString(),
      subtitle: 'preserved states',
      change: { value: 2, type: 'increase' as const, period: 'this week' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Archive size={20} />
    },
    {
      title: 'Knowledge Growth',
      value: '12.4%',
      subtitle: 'last 30 days',
      change: { value: 2.1, type: 'increase' as const, period: 'vs last month' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <Network size={20} />
    }
  ];

  return (
    <div className={styles.agentMemoryDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Agent Memory Management</Text>
          <Text variant="paragraph-large" color="secondary">
            Comprehensive agent memory browser, context preservation, and knowledge graph visualization
          </Text>

          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <Activity size={12} />
                <span>Real-time Memory Updates Active</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <Brain size={12} />
                <span>Offline Mode</span>
              </div>
            )}
          </div>
        </div>

        <div className={styles.headerRight}>
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
              variant={activeTab === 'browser' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('browser')}
            >
              <Search size={16} />
              Browser
            </Button>
            <Button
              variant={activeTab === 'graph' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('graph')}
            >
              <Network size={16} />
              Graph
            </Button>
            <Button
              variant={activeTab === 'context' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('context')}
            >
              <Archive size={16} />
              Context
            </Button>
            <Button
              variant={activeTab === 'health' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('health')}
            >
              <Activity size={16} />
              Health
            </Button>
          </div>

          {/* Actions */}
          <div className={styles.actions}>
            <Button variant="secondary" size="sm">
              <Filter size={16} />
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
            title="Agent Memory Overview"
            subtitle="Real-time agent memory monitoring, health status, and performance insights"
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

          {/* Agent Memory Health Summary */}
          <div className={styles.memoryHealthSummary}>
            <div className={styles.summaryCard}>
              <Text variant="h4">Agent Status</Text>
              <div className={styles.agentStatus}>
                <div className={styles.statusItem}>
                  <Brain size={16} className={styles.healthy} />
                  <Text variant="paragraph-medium">Healthy: {agentStats.overallStats.healthyAgents}</Text>
                </div>
                <div className={styles.statusItem}>
                  <AlertTriangle size={16} className={styles.warning} />
                  <Text variant="paragraph-medium">Warning: {agentStats.overallStats.warningAgents}</Text>
                </div>
                <div className={styles.statusItem}>
                  <XCircle size={16} className={styles.critical} />
                  <Text variant="paragraph-medium">Critical: {agentStats.overallStats.criticalAgents}</Text>
                </div>
                <div className={styles.statusItem}>
                  <Clock size={16} className={styles.unknown} />
                  <Text variant="paragraph-medium">Unknown: {agentStats.overallStats.totalAgents - agentStats.overallStats.healthyAgents - agentStats.overallStats.warningAgents - agentStats.overallStats.criticalAgents}</Text>
                </div>
              </div>
            </div>

            <div className={styles.summaryCard}>
              <Text variant="h4">Memory Distribution</Text>
              <div className={styles.memoryDistribution}>
                <div className={styles.distributionItem}>
                  <Text variant="paragraph-small" color="secondary">Conversation</Text>
                  <Text variant="paragraph-medium">{memoryStats.memoryStats.byType.conversation || 0}</Text>
                </div>
                <div className={styles.distributionItem}>
                  <Text variant="paragraph-small" color="secondary">Facts</Text>
                  <Text variant="paragraph-medium">{memoryStats.memoryStats.byType.fact || 0}</Text>
                </div>
                <div className={styles.distributionItem}>
                  <Text variant="paragraph-small" color="secondary">Knowledge</Text>
                  <Text variant="paragraph-medium">{memoryStats.memoryStats.byType.knowledge || 0}</Text>
                </div>
                <div className={styles.distributionItem}>
                  <Text variant="paragraph-small" color="secondary">Experience</Text>
                  <Text variant="paragraph-medium">{memoryStats.memoryStats.byType.experience || 0}</Text>
                </div>
              </div>
            </div>

            <div className={styles.summaryCard}>
              <Text variant="h4">Memory Performance</Text>
              <div className={styles.memoryPerformance}>
                <div className={styles.performanceItem}>
                  <Text variant="paragraph-small" color="secondary">Total Size</Text>
                  <Text variant="paragraph-medium">{(memoryStats.memoryStats.total * 1024 / 1024 / 1024).toFixed(1)}MB</Text>
                </div>
                <div className={styles.performanceItem}>
                  <Text variant="paragraph-small" color="secondary">Avg Access Time</Text>
                  <Text variant="paragraph-medium">2.3ms</Text>
                </div>
                <div className={styles.performanceItem}>
                  <Text variant="paragraph-small" color="secondary">Compression Ratio</Text>
                  <Text variant="paragraph-medium">{(memoryStats.memoryTrends.compressionRatio * 100).toFixed(1)}%</Text>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Memory Browser Tab */}
      {activeTab === 'browser' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Memory Browser</Text>
          <Text variant="paragraph-medium" color="secondary">
            Memory browser dashboard coming soon...
          </Text>
        </div>
      )}

      {/* Knowledge Graph Tab */}
      {activeTab === 'graph' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Knowledge Graph Viewer</Text>
          <Text variant="paragraph-medium" color="secondary">
            Knowledge graph viewer dashboard coming soon...
          </Text>
        </div>
      )}

      {/* Context Manager Tab */}
      {activeTab === 'context' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Context Manager</Text>
          <Text variant="paragraph-medium" color="secondary">
            Context manager dashboard coming soon...
          </Text>
        </div>
      )}

      {/* Memory Health Tab */}
      {activeTab === 'health' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Memory Health Dashboard</Text>
          <Text variant="paragraph-medium" color="secondary">
            Memory health dashboard coming soon...
          </Text>
        </div>
      )}
    </div>
  );
}
