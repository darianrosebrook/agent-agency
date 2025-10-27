/**
 * Memory Health Dashboard
 * Agent memory health monitoring, optimization, and maintenance interface
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
  RefreshCw,
  Zap,
  Database,
  Brain,
  Archive,
  Clock,
  Target,
  BarChart3,
  Play,
  Square,
} from 'lucide-react';
import { agentMemoryApiClient } from '@/lib/agent-memory-api';
import { useAgentMemoryStore, useAgentMemoryActions, useMemoryAlertStats } from '@/stores/agent-memory';
import { useAgentMemoryWebSocket, useRealTimeMemoryAlertMonitoring, useRealTimeOptimizationMonitoring } from '@/hooks/useAgentMemoryWebSocket';
import styles from './MemoryHealthDashboard.module.scss';

interface OptimizationCardProps {
  optimization: any;
  onCancel?: (id: string) => void;
  onViewDetails?: (optimization: any) => void;
}

const OptimizationCard: React.FC<OptimizationCardProps> = ({ optimization, onCancel, onViewDetails }) => {
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
        return <RefreshCw size={16} className={styles.running} />;
      case 'completed':
        return <CheckCircle size={16} className={styles.completed} />;
      case 'failed':
        return <XCircle size={16} className={styles.failed} />;
      case 'pending':
        return <Clock size={16} className={styles.pending} />;
      default:
        return <Clock size={16} className={styles.pending} />;
    }
  };

  const getProgressColor = (status: string) => {
    switch (status) {
      case 'running':
        return '#3B82F6';
      case 'completed':
        return '#10B981';
      case 'failed':
        return '#EF4444';
      case 'pending':
        return '#F59E0B';
      default:
        return '#6B7280';
    }
  };

  const formatDuration = (start: Date, end?: Date) => {
    const endTime = end || new Date();
    const duration = endTime.getTime() - start.getTime();
    const minutes = Math.floor(duration / 60000);
    const seconds = Math.floor((duration % 60000) / 1000);

    if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    }
    return `${seconds}s`;
  };

  return (
    <div className={styles.optimizationCard}>
      <div className={styles.optimizationHeader}>
        <div className={styles.optimizationInfo}>
          {getStatusIcon(optimization.status)}
          <div>
            <Text variant="h4">{optimization.type.charAt(0).toUpperCase() + optimization.type.slice(1)}</Text>
            <Text variant="paragraph-small" color="secondary">
              Agent: {optimization.agentId}
            </Text>
          </div>
        </div>
        <Text variant="paragraph-small" color="secondary">
          {optimization.status.toUpperCase()}
        </Text>
      </div>

      {optimization.progress && (
        <div className={styles.progressSection}>
          <div className={styles.progressBar}>
            <div
              className={styles.progressFill}
              style={{
                width: `${(optimization.progress.current / optimization.progress.total) * 100}%`,
                backgroundColor: getProgressColor(optimization.status)
              }}
            />
          </div>
          <Text variant="paragraph-small" color="secondary">
            {optimization.progress.current} / {optimization.progress.total} ({optimization.progress.message})
          </Text>
        </div>
      )}

      <div className={styles.optimizationMeta}>
        <div className={styles.metaItem}>
          <Text variant="paragraph-small" color="secondary">Started</Text>
          <Text variant="paragraph-small">
            {new Date(optimization.startedAt).toLocaleString()}
          </Text>
        </div>

        {optimization.completedAt && (
          <div className={styles.metaItem}>
            <Text variant="paragraph-small" color="secondary">Duration</Text>
            <Text variant="paragraph-small">
              {formatDuration(new Date(optimization.startedAt), new Date(optimization.completedAt))}
            </Text>
          </div>
        )}

        {optimization.results && (
          <div className={styles.metaItem}>
            <Text variant="paragraph-small" color="secondary">Results</Text>
            <Text variant="paragraph-small">
              {optimization.results.spaceSaved ? `${(optimization.results.spaceSaved / 1024).toFixed(1)} KB saved` : `${optimization.results.entriesProcessed} entries processed`}
            </Text>
          </div>
        )}
      </div>

      <div className={styles.optimizationActions}>
        <Button variant="secondary" size="sm" onClick={() => onViewDetails?.(optimization)}>
          Details
        </Button>

        {optimization.status === 'running' && (
          <Button variant="secondary" size="sm" onClick={() => onCancel?.(optimization.id)}>
            <Square size={14} />
            Cancel
          </Button>
        )}
      </div>
    </div>
  );
};

export function MemoryHealthDashboard() {
  const [selectedAgent, setSelectedAgent] = useState<string>('all');
  const [autoRefresh, setAutoRefresh] = useState(true);

  const { agents, memoryOptimizations, memoryAlerts, memoryHealth } = useAgentMemoryStore();
  const actions = useAgentMemoryActions();
  const { isConnected } = useAgentMemoryWebSocket();

  const memoryAlertStats = useMemoryAlertStats();
  const realTimeAlerts = useRealTimeMemoryAlertMonitoring();
  const realTimeOptimizations = useRealTimeOptimizationMonitoring();

  // Fetch health data
  useEffect(() => {
    const fetchHealthData = async () => {
      try {
        actions.setLoading('health', true);

        // Fetch memory health metrics
        const healthData = await agentMemoryApiClient.getMemoryHealthMetrics();
        healthData.forEach(health => actions.setMemoryHealth(health.agentId, health));

        // Fetch memory alerts
        const alertsData = await agentMemoryApiClient.getMemoryAlerts();
        actions.setMemoryAlerts(alertsData);

        // Fetch memory optimizations
        const optimizationData = await agentMemoryApiClient.getMemoryOptimizations();
        actions.setMemoryOptimizations(optimizationData);

      } catch (error) {
        console.error('Failed to fetch memory health data:', error);
        actions.setError('health', error instanceof Error ? error.message : 'Failed to fetch memory health data');
      } finally {
        actions.setLoading('health', false);
      }
    };

    fetchHealthData();

    // Auto-refresh every 30 seconds if enabled
    let interval: NodeJS.Timeout;
    if (autoRefresh) {
      interval = setInterval(fetchHealthData, 30000);
    }

    return () => {
      if (interval) clearInterval(interval);
    };
  }, [selectedAgent, autoRefresh]);

  const handleStartOptimization = async (agentId: string, type: string) => {
    try {
      const optimization = await agentMemoryApiClient.startMemoryOptimization(agentId, type as any);
      actions.addMemoryOptimization(optimization);
    } catch (error) {
      console.error('Failed to start optimization:', error);
    }
  };

  const handleCancelOptimization = async (optimizationId: string) => {
    if (confirm('Are you sure you want to cancel this optimization?')) {
      try {
        await agentMemoryApiClient.cancelMemoryOptimization(optimizationId);
        // Update will come via WebSocket
      } catch (error) {
        console.error('Failed to cancel optimization:', error);
      }
    }
  };

  const filteredOptimizations = memoryOptimizations.filter(opt =>
    selectedAgent === 'all' || opt.agentId === selectedAgent
  );

  const agentOptions = [
    { value: 'all', label: 'All Agents', count: agents.length },
    ...agents.map(agent => ({
      value: agent.agentId || `agent-${agents.indexOf(agent)}`,
      label: agent.name,
      count: 1
    }))
  ];

  // Load mock health data for demonstration
  const [mockHealthMetrics, setMockHealthMetrics] = useState<any>({});
  
  useEffect(() => {
    const loadMockData = async () => {
      try {
        const { agentMemoryMockApi } = await import('@/lib/mock-data-loader');
        const healthData = await agentMemoryMockApi.getMemoryHealth();
        setMockHealthMetrics(healthData);
      } catch (error) {
        console.warn('Mock data not available, using default values');
        setMockHealthMetrics({
          totalMemoryUsage: 256 * 1024 * 1024,
          activeMemoryUsage: 128 * 1024 * 1024,
          fragmentationRatio: 0.15,
          accessLatency: 2.3,
          hitRate: 0.94,
          consistencyViolations: 2,
          compressionRatio: 0.75,
          cleanupOperations: 5,
          memoryPressure: 0.25
        });
      }
    };
    
    loadMockData();
  }, []);

  return (
    <div className={styles.memoryHealthDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Memory Health Dashboard</Text>
          <Text variant="paragraph-large" color="secondary">
            Monitor memory health, optimize performance, and manage maintenance operations
          </Text>
        </div>

        <div className={styles.headerRight}>
          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <Activity size={12} />
                <span>Real-time health monitoring</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <AlertTriangle size={12} />
                <span>Offline mode</span>
              </div>
            )}
          </div>

          {/* Controls */}
          <div className={styles.controls}>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setAutoRefresh(!autoRefresh)}
            >
              {autoRefresh ? <RefreshCw size={14} className={styles.spinning} /> : <RefreshCw size={14} />}
              {autoRefresh ? 'Auto-refresh ON' : 'Auto-refresh OFF'}
            </Button>
          </div>
        </div>
      </div>

      {/* Health Overview */}
      <AnalyticsGrid
        title="Memory Health Overview"
        subtitle="Real-time memory health metrics and performance indicators"
        columns={4}
        gap="md"
      >
        <MetricCard
          title="Memory Pressure"
          value={`${(mockHealthMetrics.memoryPressure * 100).toFixed(1)}%`}
          subtitle="Current pressure level"
          change={{
            value: -5.2,
            type: 'decrease' as const,
            period: 'vs last hour'
          }}
          status={mockHealthMetrics.memoryPressure < 0.5 ? 'good' :
                  mockHealthMetrics.memoryPressure < 0.8 ? 'warning' : 'critical'}
          trend="down"
          icon={<TrendingUp size={20} />}
        />

        <MetricCard
          title="Access Latency"
          value={`${mockHealthMetrics.accessLatency.toFixed(1)}ms`}
          subtitle="Average response time"
          change={{
            value: -0.3,
            type: 'decrease' as const,
            period: 'vs last hour'
          }}
          status={mockHealthMetrics.accessLatency < 5 ? 'good' :
                  mockHealthMetrics.accessLatency < 10 ? 'warning' : 'critical'}
          trend="down"
          icon={<Zap size={20} />}
        />

        <MetricCard
          title="Cache Hit Rate"
          value={`${(mockHealthMetrics.hitRate * 100).toFixed(1)}%`}
          subtitle="Memory cache efficiency"
          change={{
            value: 2.1,
            type: 'increase' as const,
            period: 'vs last hour'
          }}
          status={mockHealthMetrics.hitRate > 0.9 ? 'good' :
                  mockHealthMetrics.hitRate > 0.8 ? 'warning' : 'critical'}
          trend="up"
          icon={<Target size={20} />}
        />

        <MetricCard
          title="Fragmentation"
          value={`${(mockHealthMetrics.fragmentationRatio * 100).toFixed(1)}%`}
          subtitle="Memory fragmentation ratio"
          change={{
            value: -1.8,
            type: 'decrease' as const,
            period: 'vs last hour'
          }}
          status={mockHealthMetrics.fragmentationRatio < 0.2 ? 'good' :
                  mockHealthMetrics.fragmentationRatio < 0.4 ? 'warning' : 'critical'}
          trend="down"
          icon={<BarChart3 size={20} />}
        />
      </AnalyticsGrid>

      {/* Agent-Specific Health */}
      <div className={styles.agentHealthSection}>
        <div className={styles.sectionHeader}>
          <Text variant="h3">Agent Memory Health</Text>
          <div className={styles.agentFilter}>
            <label>Agent:</label>
            <select
              value={selectedAgent}
              onChange={(e) => setSelectedAgent(e.target.value)}
              className={styles.select}
            >
              {agentOptions.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label} ({option.count})
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className={styles.agentHealthGrid}>
          {agents.map(agent => {
            const health = memoryHealth[agent.agentId || `agent-${agents.indexOf(agent)}`] || mockHealthMetrics;
            const alerts = memoryAlerts.filter(alert => alert.agentId === (agent.agentId || `agent-${agents.indexOf(agent)}`));

            return (
              <div key={agent.agentId || `agent-${agents.indexOf(agent)}`} className={styles.agentHealthCard}>
                <div className={styles.agentHeader}>
                  <div className={styles.agentInfo}>
                    <Brain size={20} />
                    <div>
                      <Text variant="h4">{agent.name}</Text>
                      <Text variant="paragraph-small" color="secondary">
                        {agent.type.replace('_', ' ').toUpperCase()}
                      </Text>
                    </div>
                  </div>
                  <div className={styles.agentStatus}>
                    {agent.health.status === 'healthy' && <CheckCircle size={16} className={styles.healthy} />}
                    {agent.health.status === 'warning' && <AlertTriangle size={16} className={styles.warning} />}
                    {agent.health.status === 'critical' && <XCircle size={16} className={styles.critical} />}
                  </div>
                </div>

                <div className={styles.agentMetrics}>
                  <div className={styles.metric}>
                    <Text variant="paragraph-small" color="secondary">Memory Usage</Text>
                    <Text variant="paragraph-medium">
                      {(health.totalMemoryUsage / (1024 * 1024)).toFixed(1)} MB
                    </Text>
                  </div>
                  <div className={styles.metric}>
                    <Text variant="paragraph-small" color="secondary">Active Alerts</Text>
                    <Text variant="paragraph-medium">{alerts.length}</Text>
                  </div>
                  <div className={styles.metric}>
                    <Text variant="paragraph-small" color="secondary">Compression</Text>
                    <Text variant="paragraph-medium">
                      {(health.compressionRatio * 100).toFixed(0)}%
                    </Text>
                  </div>
                </div>

                <div className={styles.agentActions}>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => handleStartOptimization(agent.agentId || `agent-${agents.indexOf(agent)}`, 'compression')}
                  >
                    <Archive size={14} />
                    Compress
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => handleStartOptimization(agent.agentId || `agent-${agents.indexOf(agent)}`, 'cleanup')}
                  >
                    <RefreshCw size={14} />
                    Cleanup
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Active Optimizations */}
      <div className={styles.optimizationsSection}>
        <div className={styles.sectionHeader}>
          <Text variant="h3">Active Optimizations</Text>
          <Text variant="paragraph-medium" color="secondary">
            {realTimeOptimizations.runningOptimizations.length} running, {realTimeOptimizations.pendingOptimizations.length} pending
          </Text>
        </div>

        <div className={styles.optimizationsGrid}>
          {filteredOptimizations.map(optimization => (
            <OptimizationCard
              key={optimization.id}
              optimization={optimization}
              onCancel={handleCancelOptimization}
              onViewDetails={() => {}}
            />
          ))}

          {filteredOptimizations.length === 0 && (
            <div className={styles.emptyState}>
              <CheckCircle size={48} />
              <Text variant="h3">No Active Optimizations</Text>
              <Text variant="paragraph-medium" color="secondary">
                All memory optimizations are complete or no optimizations are currently running.
              </Text>
            </div>
          )}
        </div>
      </div>

      {/* Health Alerts */}
      <div className={styles.alertsSection}>
        <div className={styles.sectionHeader}>
          <Text variant="h3">Memory Health Alerts</Text>
          <div className={styles.alertsSummary}>
            <div className={styles.alertSummaryItem}>
              <XCircle size={14} className={styles.critical} />
              <Text variant="paragraph-small">{memoryAlertStats.bySeverity.critical} Critical</Text>
            </div>
            <div className={styles.alertSummaryItem}>
              <AlertTriangle size={14} className={styles.warning} />
              <Text variant="paragraph-small">{memoryAlertStats.bySeverity.medium + memoryAlertStats.bySeverity.high} Warnings</Text>
            </div>
            <div className={styles.alertSummaryItem}>
              <CheckCircle size={14} className={styles.resolved} />
              <Text variant="paragraph-small">{memoryAlertStats.resolved} Resolved</Text>
            </div>
          </div>
        </div>

        <div className={styles.alertsList}>
          {realTimeAlerts.activeAlerts.slice(0, 5).map(alert => (
            <div key={alert.id} className={styles.alertItem}>
              <div className={styles.alertInfo}>
                {alert.severity === 'critical' && <XCircle size={16} className={styles.critical} />}
                {alert.severity === 'high' && <AlertTriangle size={16} className={styles.high} />}
                {alert.severity === 'medium' && <AlertTriangle size={16} className={styles.medium} />}
                {alert.severity === 'low' && <Clock size={16} className={styles.low} />}
                <div>
                  <Text variant="paragraph-medium">{alert.message}</Text>
                  <Text variant="paragraph-small" color="secondary">
                    Agent: {alert.agentId} • {new Date(alert.timestamp).toLocaleString()}
                  </Text>
                </div>
              </div>
              <div className={styles.alertValue}>
                <Text variant="paragraph-medium">{alert.value}</Text>
                <Text variant="paragraph-small" color="secondary">Threshold: {alert.threshold}</Text>
              </div>
            </div>
          ))}

          {realTimeAlerts.activeAlerts.length === 0 && (
            <div className={styles.noAlerts}>
              <CheckCircle size={24} />
              <Text variant="paragraph-medium">No active memory health alerts</Text>
            </div>
          )}
        </div>
      </div>

      {/* Optimization Actions */}
      <div className={styles.actionsSection}>
        <Text variant="h3">Optimization Actions</Text>
        <div className={styles.actionsGrid}>
          <div className={styles.actionCard}>
            <div className={styles.actionIcon}>
              <Archive size={24} />
            </div>
            <div className={styles.actionInfo}>
              <Text variant="h4">Memory Compression</Text>
              <Text variant="paragraph-medium" color="secondary">
                Compress unused memory to reduce storage footprint and improve performance.
              </Text>
            </div>
            <Button
              variant="secondary"
              onClick={() => handleStartOptimization(selectedAgent === 'all' ? (agents[0]?.agentId || `agent-0`) : selectedAgent, 'compression')}
              disabled={selectedAgent === 'all' && agents.length === 0}
            >
              <Play size={14} />
              Run Compression
            </Button>
          </div>

          <div className={styles.actionCard}>
            <div className={styles.actionIcon}>
              <RefreshCw size={24} />
            </div>
            <div className={styles.actionInfo}>
              <Text variant="h4">Memory Cleanup</Text>
              <Text variant="paragraph-medium" color="secondary">
                Remove expired, irrelevant, or redundant memories to optimize storage.
              </Text>
            </div>
            <Button
              variant="secondary"
              onClick={() => handleStartOptimization(selectedAgent === 'all' ? (agents[0]?.agentId || `agent-0`) : selectedAgent, 'cleanup')}
              disabled={selectedAgent === 'all' && agents.length === 0}
            >
              <Play size={14} />
              Run Cleanup
            </Button>
          </div>

          <div className={styles.actionCard}>
            <div className={styles.actionIcon}>
              <Database size={24} />
            </div>
            <div className={styles.actionInfo}>
              <Text variant="h4">Defragmentation</Text>
              <Text variant="paragraph-medium" color="secondary">
                Reorganize memory structure to reduce fragmentation and improve access speed.
              </Text>
            </div>
            <Button
              variant="secondary"
              onClick={() => handleStartOptimization(selectedAgent === 'all' ? (agents[0]?.agentId || `agent-0`) : selectedAgent, 'defragmentation')}
              disabled={selectedAgent === 'all' && agents.length === 0}
            >
              <Play size={14} />
              Run Defrag
            </Button>
          </div>

          <div className={styles.actionCard}>
            <div className={styles.actionIcon}>
              <Target size={24} />
            </div>
            <div className={styles.actionInfo}>
              <Text variant="h4">Index Rebuilding</Text>
              <Text variant="paragraph-medium" color="secondary">
                Rebuild memory indexes for faster search and retrieval operations.
              </Text>
            </div>
            <Button
              variant="secondary"
              onClick={() => handleStartOptimization(selectedAgent === 'all' ? (agents[0]?.agentId || `agent-0`) : selectedAgent, 'reindexing')}
              disabled={selectedAgent === 'all' && agents.length === 0}
            >
              <Play size={14} />
              Rebuild Index
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
