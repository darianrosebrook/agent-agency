/**
 * Workspace WebSocket Hook
 * Real-time updates for workspace management, git operations, and backup/recovery
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useWorkspaceStore, useWorkspaceActions } from '@/stores/workspace';
import { GitOperation, BackupResult, RecoveryOperation, WorkspaceHealth } from '@/lib/workspace-api';

interface WorkspaceWebSocketMessage {
  type: 'health_update' | 'git_status_update' | 'git_operation_update' | 'snapshot_created' | 'backup_job_update' | 'backup_result' | 'recovery_operation_update' | 'dependency_update' | 'file_integrity_update' | 'disk_usage_update';
  data: any;
  timestamp: string;
}

export function useWorkspaceWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useWorkspaceActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/workspace`);

      ws.onopen = () => {
        console.log('Workspace WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;

        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));

        // Subscribe to real-time workspace updates
        ws.send(JSON.stringify({
          type: 'subscribe',
          channels: ['health', 'git', 'snapshots', 'backups', 'recovery', 'dependencies', 'files', 'disk']
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: WorkspaceWebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse Workspace WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('Workspace WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('Workspace WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create Workspace WebSocket connection:', error);
      setConnectionStatus('error');
      scheduleReconnect();
    }
  };

  const scheduleReconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    const delay = reconnectDelay * Math.pow(2, reconnectAttempts.current);
    reconnectAttempts.current++;

    console.log(`Scheduling Workspace WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);

    reconnectTimeoutRef.current = setTimeout(() => {
      connect();
    }, delay);
  };

  const disconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setIsConnected(false);
    setConnectionStatus('disconnected');
  };

  const handleMessage = (message: WorkspaceWebSocketMessage) => {
    const { type, data, timestamp } = message;

    switch (type) {
      case 'health_update':
        actions.setWorkspaceHealth(data as WorkspaceHealth);
        break;

      case 'git_status_update':
        actions.setGitRepository(data);
        break;

      case 'git_operation_update':
        const operation = data as GitOperation;
        if (operation.status === 'completed' || operation.status === 'failed' || operation.status === 'cancelled') {
          actions.removeActiveOperation(operation.id);
        } else {
          actions.updateActiveOperation(operation.id, operation);
        }
        break;

      case 'snapshot_created':
        actions.addSnapshot(data);
        break;

      case 'backup_job_update':
        actions.updateBackupJob(data.id, data.updates);
        break;

      case 'backup_result':
        // Update backup job with result
        const result = data as BackupResult;
        actions.updateBackupJob(result.jobId, {
          lastResult: result,
          lastRun: result.startedAt,
          status: 'completed'
        });
        break;

      case 'recovery_operation_update':
        actions.updateRecoveryOperation(data.id, data.updates);
        break;

      case 'dependency_update':
        actions.updateDependency(data.name, data.updates);
        break;

      case 'file_integrity_update':
        actions.updateFileIntegrity(data.path, data.updates);
        break;

      case 'disk_usage_update':
        // Handle disk usage updates if needed
        console.log('Disk usage update:', data);
        break;

      default:
        console.warn('Unknown Workspace WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('Workspace WebSocket not connected, cannot send message');
    }
  };

  // Subscribe to specific channels
  const subscribe = (channels: string[]) => {
    sendMessage({
      type: 'subscribe',
      channels
    });
  };

  // Unsubscribe from channels
  const unsubscribe = (channels: string[]) => {
    sendMessage({
      type: 'unsubscribe',
      channels
    });
  };

  // Request current workspace health
  const requestHealth = () => {
    sendMessage({
      type: 'request_health'
    });
  };

  // Request git status
  const requestGitStatus = () => {
    sendMessage({
      type: 'request_git_status'
    });
  };

  // Request backup status
  const requestBackupStatus = () => {
    sendMessage({
      type: 'request_backup_status'
    });
  };

  // Request recovery operations
  const requestRecoveryStatus = () => {
    sendMessage({
      type: 'request_recovery_status'
    });
  };

  // Request dependency status
  const requestDependencyStatus = () => {
    sendMessage({
      type: 'request_dependency_status'
    });
  };

  // Request file integrity status
  const requestFileIntegrity = () => {
    sendMessage({
      type: 'request_file_integrity'
    });
  };

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, []);

  return {
    isConnected,
    connectionStatus,
    connect,
    disconnect,
    sendMessage,
    subscribe,
    unsubscribe,
    requestHealth,
    requestGitStatus,
    requestBackupStatus,
    requestRecoveryStatus,
    requestDependencyStatus,
    requestFileIntegrity,
  };
}

// Hook for real-time workspace health monitoring
export function useRealTimeWorkspaceHealth() {
  const workspaceHealth = useWorkspaceStore((state) => state.workspaceHealth);
  const loading = useWorkspaceStore((state) => state.loading.health);

  return {
    workspaceHealth,
    loading,
    healthScore: workspaceHealth?.overallScore || 0,
    healthStatus: workspaceHealth?.overallStatus || 'unknown',
    totalChecks: workspaceHealth?.checks.length || 0,
    healthyChecks: workspaceHealth?.checks.filter(check => check.status === 'healthy').length || 0,
    warningChecks: workspaceHealth?.checks.filter(check => check.status === 'warning').length || 0,
    criticalChecks: workspaceHealth?.checks.filter(check => check.status === 'critical').length || 0,
    totalIssues: workspaceHealth?.issues.length || 0,
    criticalIssues: workspaceHealth?.issues.filter(issue => issue.severity === 'critical').length || 0,
    unresolvedIssues: workspaceHealth?.issues.filter(issue => !issue.resolved).length || 0,
  };
}

// Hook for real-time git operations monitoring
export function useRealTimeGitMonitoring() {
  const gitRepository = useWorkspaceStore((state) => state.gitRepository);
  const activeOperations = useWorkspaceStore((state) => state.activeOperations);
  const loading = useWorkspaceStore((state) => state.loading.git);

  return {
    gitRepository,
    activeOperations,
    loading,
    hasUncommittedChanges: gitRepository?.hasUncommittedChanges || false,
    hasUntrackedFiles: gitRepository?.hasUntrackedFiles || false,
    currentBranch: gitRepository?.currentBranch || 'unknown',
    aheadBy: gitRepository?.aheadBy || 0,
    behindBy: gitRepository?.behindBy || 0,
    runningOperations: activeOperations.filter(op => op.status === 'running'),
    pendingOperations: activeOperations.filter(op => op.status === 'pending'),
    failedOperations: activeOperations.filter(op => op.status === 'failed'),
    operationStats: {
      total: activeOperations.length,
      running: activeOperations.filter(op => op.status === 'running').length,
      completed: activeOperations.filter(op => op.status === 'completed').length,
      failed: activeOperations.filter(op => op.status === 'failed').length,
      pending: activeOperations.filter(op => op.status === 'pending').length,
    },
  };
}

// Hook for real-time backup monitoring
export function useRealTimeBackupMonitoring() {
  const backupJobs = useWorkspaceStore((state) => state.backupJobs);
  const recoveryOperations = useWorkspaceStore((state) => state.recoveryOperations);
  const loading = useWorkspaceStore((state) => state.loading.backups);

  return {
    backupJobs,
    recoveryOperations,
    loading,
    runningBackups: backupJobs.filter(job => job.status === 'running'),
    failedBackups: backupJobs.filter(job => job.status === 'failed'),
    completedBackups: backupJobs.filter(job => job.lastResult?.status === 'success'),
    runningRecoveries: recoveryOperations.filter(op => op.status === 'running'),
    failedRecoveries: recoveryOperations.filter(op => op.status === 'failed'),
    completedRecoveries: recoveryOperations.filter(op => op.status === 'completed'),
    backupStats: {
      totalJobs: backupJobs.length,
      runningJobs: backupJobs.filter(job => job.status === 'running').length,
      failedJobs: backupJobs.filter(job => job.status === 'failed').length,
      idleJobs: backupJobs.filter(job => job.status === 'idle').length,
      totalRecoveries: recoveryOperations.length,
      runningRecoveries: recoveryOperations.filter(op => op.status === 'running').length,
      failedRecoveries: recoveryOperations.filter(op => op.status === 'failed').length,
    },
  };
}

// Hook for real-time dependency monitoring
export function useRealTimeDependencyMonitoring() {
  const dependencies = useWorkspaceStore((state) => state.dependencies);
  const loading = useWorkspaceStore((state) => state.loading.dependencies);

  return {
    dependencies,
    loading,
    totalDependencies: dependencies.length,
    installedDependencies: dependencies.filter(dep => dep.status === 'installed'),
    outdatedDependencies: dependencies.filter(dep => dep.status === 'outdated'),
    missingDependencies: dependencies.filter(dep => dep.status === 'missing'),
    conflictedDependencies: dependencies.filter(dep => dep.status === 'conflicted'),
    vulnerableDependencies: dependencies.filter(dep => dep.vulnerabilities && dep.vulnerabilities > 0),
    dependencyStats: {
      total: dependencies.length,
      installed: dependencies.filter(dep => dep.status === 'installed').length,
      outdated: dependencies.filter(dep => dep.status === 'outdated').length,
      missing: dependencies.filter(dep => dep.status === 'missing').length,
      conflicted: dependencies.filter(dep => dep.status === 'conflicted').length,
      withVulnerabilities: dependencies.filter(dep => dep.vulnerabilities && dep.vulnerabilities > 0).length,
    },
    byType: dependencies.reduce((acc, dep) => {
      acc[dep.type] = (acc[dep.type] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
  };
}

// Hook for real-time file integrity monitoring
export function useRealTimeFileIntegrityMonitoring() {
  const fileIntegrity = useWorkspaceStore((state) => state.fileIntegrity);
  const loading = useWorkspaceStore((state) => state.loading.fileIntegrity);

  return {
    fileIntegrity,
    loading,
    verifiedFiles: fileIntegrity.filter(check => check.status === 'verified'),
    modifiedFiles: fileIntegrity.filter(check => check.status === 'modified'),
    missingFiles: fileIntegrity.filter(check => check.status === 'missing'),
    corruptedFiles: fileIntegrity.filter(check => check.status === 'corrupted'),
    integrityStats: {
      total: fileIntegrity.length,
      verified: fileIntegrity.filter(check => check.status === 'verified').length,
      modified: fileIntegrity.filter(check => check.status === 'modified').length,
      missing: fileIntegrity.filter(check => check.status === 'missing').length,
      corrupted: fileIntegrity.filter(check => check.status === 'corrupted').length,
      healthy: fileIntegrity.filter(check => check.status === 'verified').length,
      unhealthy: fileIntegrity.filter(check => !['verified'].includes(check.status)).length,
    },
    lastVerification: fileIntegrity.length > 0
      ? new Date(Math.max(...fileIntegrity.map(check => check.lastVerified.getTime())))
      : null,
  };
}

// Hook for real-time workspace operations monitoring
export function useRealTimeWorkspaceOperations() {
  const activeOperations = useWorkspaceStore((state) => state.activeOperations);
  const recoveryOperations = useWorkspaceStore((state) => state.recoveryOperations);
  const backupJobs = useWorkspaceStore((state) => state.backupJobs);

  return {
    activeOperations,
    recoveryOperations,
    backupJobs,
    allOperations: [
      ...activeOperations,
      ...recoveryOperations.map(op => ({
        ...op,
        type: 'recovery' as const,
      })),
      ...backupJobs.filter(job => job.status === 'running').map(job => ({
        id: job.id,
        type: 'backup' as const,
        status: job.status,
        startedAt: job.lastRun || new Date(),
      })),
    ],
    operationStats: {
      git: activeOperations.length,
      recovery: recoveryOperations.filter(op => ['pending', 'running'].includes(op.status)).length,
      backup: backupJobs.filter(job => job.status === 'running').length,
      total: activeOperations.length +
             recoveryOperations.filter(op => ['pending', 'running'].includes(op.status)).length +
             backupJobs.filter(job => job.status === 'running').length,
    },
    hasActiveOperations: activeOperations.length > 0 ||
                        recoveryOperations.some(op => ['pending', 'running'].includes(op.status)) ||
                        backupJobs.some(job => job.status === 'running'),
  };
}
