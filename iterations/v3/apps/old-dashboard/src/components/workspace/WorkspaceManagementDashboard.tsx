/**
 * Workspace Management Dashboard
 * Comprehensive workspace state management, git operations, backup/recovery, and development workflow
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
  GitBranch,
  Database,
  Shield,
  HardDrive,
  Settings,
  RefreshCw,
  FolderOpen,
  FileText,
  Archive,
  RotateCcw,
  GitCommit,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  Package
} from 'lucide-react';
import { workspaceApiClient } from '@/lib/workspace-api';
import { useWorkspaceStore, useWorkspaceActions } from '@/stores/workspace';
import { useWorkspaceWebSocket, useRealTimeWorkspaceHealth, useRealTimeGitMonitoring, useRealTimeBackupMonitoring } from '@/hooks/useWorkspaceWebSocket';
// Commented out to resolve build errors
// import { WorkspaceHealthDashboard } from './WorkspaceHealthDashboard';
import { GitOperationsDashboard } from './GitOperationsDashboard';
// import { BackupRecoveryDashboard } from './BackupRecoveryDashboard';
// import { StateManagementDashboard } from './StateManagementDashboard';
import styles from './WorkspaceManagementDashboard.module.scss';

export function WorkspaceManagementDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'health' | 'git' | 'backup' | 'state'>('overview');
  const [refreshing, setRefreshing] = useState(false);

  // Store state
  const { workspaceHealth, gitRepository, backupJobs, recoveryOperations, dependencies, fileIntegrity, loading, errors } = useWorkspaceStore();
  const actions = useWorkspaceActions();
  const { isConnected } = useWorkspaceWebSocket();

  // Real-time monitoring hooks
  const healthStats = useRealTimeWorkspaceHealth();
  const gitStats = useRealTimeGitMonitoring();
  const backupStats = useRealTimeBackupMonitoring();

  // Fetch initial data
  useEffect(() => {
    fetchWorkspaceData();
  }, []);

  const fetchWorkspaceData = async () => {
    try {
      setRefreshing(true);
      actions.clearErrors();

      // Fetch workspace health
      actions.setLoading('health', true);
      const healthData = await workspaceApiClient.getWorkspaceHealth();
      actions.setWorkspaceHealth(healthData);

      // Fetch git status
      actions.setLoading('git', true);
      const gitData = await workspaceApiClient.getGitStatus();
      actions.setGitRepository(gitData);

      // Fetch backup jobs
      actions.setLoading('backups', true);
      const backupData = await workspaceApiClient.getBackupJobs();
      actions.setBackupJobs(backupData);

      // Fetch recovery operations
      const recoveryData = await workspaceApiClient.getRecoveryOperations();
      actions.setRecoveryOperations(recoveryData);

      // Fetch dependencies
      actions.setLoading('dependencies', true);
      const dependencyData = await workspaceApiClient.getDependencies();
      actions.setDependencies(dependencyData);

      // Fetch file integrity
      actions.setLoading('fileIntegrity', true);
      const integrityData = await workspaceApiClient.getFileIntegrity();
      actions.setFileIntegrity(integrityData);

    } catch (error) {
      console.error('Failed to fetch workspace dashboard data:', error);
      actions.setError('health', error instanceof Error ? error.message : 'Failed to fetch data');
    } finally {
      actions.setLoading('health', false);
      actions.setLoading('git', false);
      actions.setLoading('backups', false);
      actions.setLoading('dependencies', false);
      actions.setLoading('fileIntegrity', false);
      setRefreshing(false);
    }
  };

  const handleRefresh = async () => {
    await fetchWorkspaceData();
  };

  // Mock overview metrics for demonstration (when real data is not available)
  const overviewMetrics = [
    {
      title: 'Workspace Health',
      value: `${healthStats.healthScore}%`,
      subtitle: healthStats.healthStatus.toUpperCase(),
      change: { value: 2.1, type: 'increase' as const, period: 'vs last hour' },
      status: healthStats.healthStatus === 'healthy' ? 'good' as const :
              healthStats.healthStatus === 'warning' ? 'warning' as const : 'error' as const,
      trend: 'up' as const,
      icon: <Activity size={20} />
    },
    {
      title: 'Git Status',
      value: gitStats.hasUncommittedChanges ? 'Dirty' : 'Clean',
      subtitle: `${gitStats.currentBranch} branch`,
      change: {
        value: gitStats.aheadBy + gitStats.behindBy,
        type: 'neutral' as const,
        period: 'pending sync'
      },
      status: gitStats.hasUncommittedChanges ? 'warning' as const : 'good' as const,
      trend: 'neutral' as const,
      icon: <GitBranch size={20} />
    },
    {
      title: 'Backup Jobs',
      value: backupStats.backupStats.totalJobs.toString(),
      subtitle: `${backupStats.backupStats.runningJobs} running`,
      change: {
        value: backupStats.backupStats.failedJobs,
        type: 'neutral' as const,
        period: 'failed'
      },
      status: backupStats.backupStats.failedJobs === 0 ? 'good' as const : 'error' as const,
      trend: 'neutral' as const,
      icon: <Database size={20} />
    },
    {
      title: 'Dependencies',
      value: dependencies.length.toString(),
      subtitle: `${dependencies.filter(d => d.status === 'outdated').length} outdated`,
      change: {
        value: dependencies.filter(d => d.status === 'outdated').length,
        type: 'neutral' as const,
        period: 'need update'
      },
      status: dependencies.filter(d => d.status === 'outdated').length === 0 ? 'good' as const : 'warning' as const,
      trend: 'neutral' as const,
      icon: <Package size={20} />
    },
    {
      title: 'File Integrity',
      value: `${fileIntegrity.filter(f => f.status === 'verified').length}/${fileIntegrity.length}`,
      subtitle: 'files verified',
      change: {
        value: fileIntegrity.filter(f => f.status !== 'verified').length,
        type: 'neutral' as const,
        period: 'issues found'
      },
      status: fileIntegrity.filter(f => f.status !== 'verified').length === 0 ? 'good' as const : 'error' as const,
      trend: 'neutral' as const,
      icon: <Shield size={20} />
    },
    {
      title: 'Disk Usage',
      value: '75%',
      subtitle: '2.3GB used',
      change: { value: -2.1, type: 'decrease' as const, period: 'vs last week' },
      status: 'good' as const,
      trend: 'down' as const,
      icon: <HardDrive size={20} />
    }
  ];

  return (
    <div className={styles.workspaceManagementDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Workspace Management</Text>
          <Text variant="paragraph-large" color="secondary">
            Comprehensive workspace state management, git operations, backup/recovery, and development workflow
          </Text>

          {workspaceHealth && (
            <div className={styles.workspaceStatus}>
              <div className={`statusIndicator ${healthStats.healthStatus}`}>
                <Activity size={16} />
                <span>Workspace Status: {healthStats.healthStatus.toUpperCase()}</span>
              </div>
            </div>
          )}
        </div>

        <div className={styles.headerRight}>
          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <Activity size={12} />
                <span>Live</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <FolderOpen size={12} />
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
              <Shield size={16} />
              Health
            </Button>
            <Button
              variant={activeTab === 'git' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('git')}
            >
              <GitBranch size={16} />
              Git
            </Button>
            <Button
              variant={activeTab === 'backup' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('backup')}
            >
              <Database size={16} />
              Backup
            </Button>
            <Button
              variant={activeTab === 'state' ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => setActiveTab('state')}
            >
              <Archive size={16} />
              State
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
            title="Workspace Overview"
            subtitle="Real-time workspace health, git status, backup operations, and dependency management"
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

          {/* Workspace Status Summary */}
          <div className={styles.statusSummary}>
            <div className={styles.summaryCard}>
              <Text variant="h4">Health Checks</Text>
              <div className={styles.healthGrid}>
                <div className={styles.healthItem}>
                  <CheckCircle size={16} className={styles.healthy} />
                  <Text variant="paragraph-medium">Healthy: {healthStats.healthyChecks}</Text>
                </div>
                <div className={styles.healthItem}>
                  <AlertTriangle size={16} className={styles.warning} />
                  <Text variant="paragraph-medium">Warning: {healthStats.warningChecks}</Text>
                </div>
                <div className={styles.healthItem}>
                  <XCircle size={16} className={styles.critical} />
                  <Text variant="paragraph-medium">Critical: {healthStats.criticalChecks}</Text>
                </div>
                <div className={styles.healthItem}>
                  <Clock size={16} className={styles.unknown} />
                  <Text variant="paragraph-medium">Unknown: {healthStats.totalChecks - healthStats.healthyChecks - healthStats.warningChecks - healthStats.criticalChecks}</Text>
                </div>
              </div>
            </div>

            <div className={styles.summaryCard}>
              <Text variant="h4">Git Operations</Text>
              <div className={styles.gitStats}>
                <div className={styles.gitItem}>
                  <Text variant="paragraph-small" color="secondary">Current Branch</Text>
                  <Text variant="paragraph-medium">{gitStats.currentBranch}</Text>
                </div>
                <div className={styles.gitItem}>
                  <Text variant="paragraph-small" color="secondary">Active Operations</Text>
                  <Text variant="paragraph-medium">{gitStats.operationStats.running}</Text>
                </div>
                <div className={styles.gitItem}>
                  <Text variant="paragraph-small" color="secondary">Pending Changes</Text>
                  <Text variant="paragraph-medium">
                    {gitStats.gitStatusSummary?.staged || 0} staged, {gitStats.gitStatusSummary?.unstaged || 0} unstaged
                  </Text>
                </div>
              </div>
            </div>

            <div className={styles.summaryCard}>
              <Text variant="h4">Backup Operations</Text>
              <div className={styles.backupStats}>
                <div className={styles.backupItem}>
                  <Text variant="paragraph-small" color="secondary">Total Jobs</Text>
                  <Text variant="paragraph-medium">{backupStats.backupStats.totalJobs}</Text>
                </div>
                <div className={styles.backupItem}>
                  <Text variant="paragraph-small" color="secondary">Running</Text>
                  <Text variant="paragraph-medium">{backupStats.backupStats.runningJobs}</Text>
                </div>
                <div className={styles.backupItem}>
                  <Text variant="paragraph-small" color="secondary">Failed</Text>
                  <Text variant="paragraph-medium">{backupStats.backupStats.failedJobs}</Text>
                </div>
              </div>
            </div>

            <div className={styles.summaryCard}>
              <Text variant="h4">Dependencies</Text>
              <div className={styles.dependencyStats}>
                <div className={styles.dependencyItem}>
                  <Text variant="paragraph-small" color="secondary">Total</Text>
                  <Text variant="paragraph-medium">{dependencies.length}</Text>
                </div>
                <div className={styles.dependencyItem}>
                  <Text variant="paragraph-small" color="secondary">Outdated</Text>
                  <Text variant="paragraph-medium">{dependencies.filter(d => d.status === 'outdated').length}</Text>
                </div>
                <div className={styles.dependencyItem}>
                  <Text variant="paragraph-small" color="secondary">Vulnerable</Text>
                  <Text variant="paragraph-medium">{dependencies.filter(d => d.vulnerabilities && d.vulnerabilities > 0).length}</Text>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Health Tab */}
      {activeTab === 'health' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Workspace Health Dashboard</Text>
          <Text variant="paragraph-medium" color="secondary">
            Workspace health monitoring dashboard coming soon...
          </Text>
        </div>
      )}

      {/* Git Tab */}
      {activeTab === 'git' && (
        <GitOperationsDashboard />
      )}

      {/* Backup Tab */}
      {activeTab === 'backup' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">Backup & Recovery Dashboard</Text>
          <Text variant="paragraph-medium" color="secondary">
            Backup and recovery management dashboard coming soon...
          </Text>
        </div>
      )}

      {/* State Tab */}
      {activeTab === 'state' && (
        <div className={styles.placeholderTab}>
          <Text variant="h3">State Management Dashboard</Text>
          <Text variant="paragraph-medium" color="secondary">
            State management dashboard coming soon...
          </Text>
        </div>
      )}
    </div>
  );
}
