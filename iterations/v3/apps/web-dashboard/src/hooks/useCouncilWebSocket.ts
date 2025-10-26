/**
 * Council WebSocket Hook
 * Real-time updates for council oversight operations
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useCouncilStore, useCouncilActions } from '@/stores/council';
import { Verdict, Judge, CouncilMetrics, CouncilAlert } from '@/lib/council-api';

interface CouncilWebSocketMessage {
  type: 'verdict_created' | 'verdict_updated' | 'verdict_completed' | 'judge_updated' | 'metrics_updated' | 'alert_created' | 'alert_acknowledged';
  data: any;
  timestamp: string;
}

export function useCouncilWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useCouncilActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/council`);

      ws.onopen = () => {
        console.log('Council WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;

        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));

        // Subscribe to council updates
        ws.send(JSON.stringify({
          type: 'subscribe',
          channels: ['verdicts', 'judges', 'metrics', 'alerts']
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: CouncilWebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse Council WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('Council WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('Council WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create Council WebSocket connection:', error);
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

    console.log(`Scheduling Council WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);

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

  const handleMessage = (message: CouncilWebSocketMessage) => {
    const { type, data, timestamp } = message;

    // Update last update timestamp
    actions.setLastUpdate(new Date(timestamp));

    switch (type) {
      case 'verdict_created':
        actions.addVerdict(data as Verdict);
        break;

      case 'verdict_updated':
        actions.updateVerdict(data.id, data.updates);
        break;

      case 'verdict_completed':
        actions.updateVerdict(data.id, {
          ...data,
          status: 'completed',
          completedAt: new Date(timestamp)
        });
        break;

      case 'judge_updated':
        actions.updateJudge(data.id, data.updates);
        break;

      case 'metrics_updated':
        actions.setMetrics(data as CouncilMetrics);
        actions.setLoading('metrics', false);
        break;

      case 'alert_created':
        actions.addAlert(data as CouncilAlert);
        break;

      case 'alert_acknowledged':
        actions.acknowledgeAlert(data.alertId);
        break;

      default:
        console.warn('Unknown Council WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('Council WebSocket not connected, cannot send message');
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

  // Request real-time metrics
  const requestMetrics = () => {
    sendMessage({
      type: 'request_metrics'
    });
  };

  // Request active alerts
  const requestAlerts = () => {
    sendMessage({
      type: 'request_alerts'
    });
  };

  // Request judge updates
  const requestJudgeUpdates = () => {
    sendMessage({
      type: 'request_judge_updates'
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
    requestMetrics,
    requestAlerts,
    requestJudgeUpdates,
  };
}

// Hook for real-time verdict monitoring
export function useRealTimeVerdictMonitoring() {
  const verdicts = useCouncilStore((state) => state.verdicts);
  const loading = useCouncilStore((state) => state.loading.verdicts);

  return {
    verdicts,
    loading,
    totalVerdicts: verdicts.length,
    pendingVerdicts: verdicts.filter(v => v.status === 'pending').length,
    inProgressVerdicts: verdicts.filter(v => v.status === 'in_progress').length,
    completedVerdicts: verdicts.filter(v => v.status === 'completed').length,
    escalatedVerdicts: verdicts.filter(v => v.status === 'escalated').length,
    recentVerdicts: verdicts.slice(0, 10), // Most recent 10
  };
}

// Hook for real-time judge monitoring
export function useRealTimeJudgeMonitoring() {
  const judges = useCouncilStore((state) => state.judges);
  const loading = useCouncilStore((state) => state.loading.judges);

  return {
    judges,
    loading,
    totalJudges: judges.length,
    activeJudges: judges.filter(j => j.status === 'active').length,
    inactiveJudges: judges.filter(j => j.status === 'inactive').length,
    errorJudges: judges.filter(j => j.status === 'error').length,
    averageAccuracy: judges.length > 0
      ? judges.reduce((sum, j) => sum + j.performance.accuracy, 0) / judges.length
      : 0,
    averageResponseTime: judges.length > 0
      ? judges.reduce((sum, j) => sum + j.performance.responseTime, 0) / judges.length
      : 0,
  };
}

// Hook for real-time alert monitoring
export function useRealTimeAlertMonitoring() {
  const alerts = useCouncilStore((state) => state.alerts);
  const loading = useCouncilStore((state) => state.loading.alerts);

  return {
    alerts,
    loading,
    totalAlerts: alerts.length,
    unacknowledgedAlerts: alerts.filter(a => !a.acknowledged).length,
    criticalAlerts: alerts.filter(a => a.severity === 'critical' && !a.acknowledged).length,
    highAlerts: alerts.filter(a => a.severity === 'high' && !a.acknowledged).length,
    recentAlerts: alerts.slice(0, 5), // Most recent 5
  };
}