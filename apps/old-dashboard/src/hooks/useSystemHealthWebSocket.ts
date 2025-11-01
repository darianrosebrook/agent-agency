/**
 * System Health WebSocket Hook
 * Real-time updates for system health monitoring and Grafana integration
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useSystemHealthStore, useSystemHealthActions } from '@/stores/system-health';
import { SystemHealth, ComponentHealth, SystemAlert, GrafanaAlert } from '@/lib/system-health-api';

interface SystemHealthWebSocketMessage {
  type: 'health_update' | 'component_update' | 'alert_created' | 'alert_updated' | 'grafana_alert' | 'metrics_update' | 'trend_update';
  data: any;
  timestamp: string;
}

export function useSystemHealthWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useSystemHealthActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/system-health`);

      ws.onopen = () => {
        console.log('System Health WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;

        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));

        // Subscribe to real-time system health updates
        ws.send(JSON.stringify({
          type: 'subscribe',
          channels: ['health', 'components', 'alerts', 'grafana', 'metrics', 'trends']
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: SystemHealthWebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse System Health WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('System Health WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('System Health WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create System Health WebSocket connection:', error);
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

    console.log(`Scheduling System Health WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);

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

  const handleMessage = (message: SystemHealthWebSocketMessage) => {
    const { type, data, timestamp } = message;

    // Update last update timestamp
    actions.setLastUpdate(new Date(timestamp));

    switch (type) {
      case 'health_update':
        actions.setSystemHealth(data as SystemHealth);
        break;

      case 'component_update':
        actions.updateComponent(data.id, data.updates);
        break;

      case 'alert_created':
        actions.addAlert(data as SystemAlert);
        break;

      case 'alert_updated':
        actions.updateAlert(data.id, data.updates);
        break;

      case 'grafana_alert':
        // Update Grafana alerts
        const currentGrafanaAlerts = useSystemHealthStore.getState().grafanaAlerts;
        const existingIndex = currentGrafanaAlerts.findIndex(alert => alert.id === data.id);
        if (existingIndex >= 0) {
          const updated = [...currentGrafanaAlerts];
          updated[existingIndex] = data as GrafanaAlert;
          actions.setGrafanaAlerts(updated);
        } else {
          actions.setGrafanaAlerts([data as GrafanaAlert, ...currentGrafanaAlerts.slice(0, 99)]);
        }
        break;

      case 'metrics_update':
        // Handle real-time metrics updates
        console.log('Real-time metrics update:', data);
        break;

      case 'trend_update':
        actions.addHealthTrend(data);
        break;

      default:
        console.warn('Unknown System Health WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('System Health WebSocket not connected, cannot send message');
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

  // Request current health status
  const requestHealth = () => {
    sendMessage({
      type: 'request_health'
    });
  };

  // Request component updates
  const requestComponents = () => {
    sendMessage({
      type: 'request_components'
    });
  };

  // Request alerts
  const requestAlerts = () => {
    sendMessage({
      type: 'request_alerts'
    });
  };

  // Request Grafana data
  const requestGrafana = () => {
    sendMessage({
      type: 'request_grafana'
    });
  };

  // Request metrics
  const requestMetrics = (componentId?: string) => {
    sendMessage({
      type: 'request_metrics',
      componentId
    });
  };

  // Request health trends
  const requestTrends = (timeRange?: any) => {
    sendMessage({
      type: 'request_trends',
      timeRange
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
    requestComponents,
    requestAlerts,
    requestGrafana,
    requestMetrics,
    requestTrends,
  };
}

// Hook for real-time component monitoring
export function useRealTimeComponentMonitoring() {
  const components = useSystemHealthStore((state) => state.components);
  const loading = useSystemHealthStore((state) => state.loading.components);

  return {
    components,
    loading,
    componentCount: components.length,
    healthyCount: components.filter(c => c.status === 'healthy').length,
    warningCount: components.filter(c => c.status === 'warning').length,
    criticalCount: components.filter(c => c.status === 'critical').length,
    averageAvailability: components.length > 0
      ? components.reduce((sum, c) => sum + c.availability, 0) / components.length
      : 0,
    averageResponseTime: components.length > 0
      ? components.reduce((sum, c) => sum + c.responseTime, 0) / components.length
      : 0,
    byType: components.reduce((acc, component) => {
      acc[component.type] = (acc[component.type] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
  };
}

// Hook for real-time alert monitoring
export function useRealTimeAlertMonitoring() {
  const alerts = useSystemHealthStore((state) => state.alerts);
  const loading = useSystemHealthStore((state) => state.loading.alerts);

  return {
    alerts,
    loading,
    activeAlerts: alerts.filter(a => a.status === 'active'),
    criticalAlerts: alerts.filter(a => a.severity === 'critical' && a.status === 'active'),
    recentAlerts: alerts.slice(0, 10),
    alertCountBySeverity: {
      critical: alerts.filter(a => a.severity === 'critical').length,
      high: alerts.filter(a => a.severity === 'high').length,
      medium: alerts.filter(a => a.severity === 'medium').length,
      low: alerts.filter(a => a.severity === 'low').length,
    },
    alertCountByStatus: {
      active: alerts.filter(a => a.status === 'active').length,
      acknowledged: alerts.filter(a => a.status === 'acknowledged').length,
      resolved: alerts.filter(a => a.status === 'resolved').length,
    },
    alertTrends: {
      lastHour: alerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 60 * 60 * 1000)).length,
      last24Hours: alerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 24 * 60 * 60 * 1000)).length,
      last7Days: alerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)).length,
    },
  };
}

// Hook for real-time Grafana integration
export function useRealTimeGrafanaMonitoring() {
  const grafanaDashboards = useSystemHealthStore((state) => state.grafanaDashboards);
  const grafanaAlerts = useSystemHealthStore((state) => state.grafanaAlerts);
  const embeddedPanels = useSystemHealthStore((state) => state.embeddedPanels);
  const loading = useSystemHealthStore((state) => state.loading.grafana);

  return {
    grafanaDashboards,
    grafanaAlerts,
    embeddedPanels,
    loading,
    activeGrafanaAlerts: grafanaAlerts.filter(a => a.state === 'alerting'),
    dashboardCount: grafanaDashboards.length,
    alertCount: grafanaAlerts.length,
    dashboardsByFolder: grafanaDashboards.reduce((acc, dashboard) => {
      const folder = dashboard.folderTitle || 'General';
      acc[folder] = (acc[folder] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
  };
}

// Hook for real-time health trends
export function useRealTimeHealthTrends() {
  const healthTrends = useSystemHealthStore((state) => state.healthTrends);
  const loading = useSystemHealthStore((state) => state.loading.trends);

  return {
    healthTrends,
    loading,
    latestScore: healthTrends.length > 0 ? healthTrends[healthTrends.length - 1].overallScore : 0,
    trend: healthTrends.length >= 2
      ? healthTrends[healthTrends.length - 1].overallScore - healthTrends[healthTrends.length - 2].overallScore
      : 0,
    averageScore: healthTrends.length > 0
      ? healthTrends.reduce((sum, t) => sum + t.overallScore, 0) / healthTrends.length
      : 0,
    scoreRange: healthTrends.length > 0
      ? {
          min: Math.min(...healthTrends.map(t => t.overallScore)),
          max: Math.max(...healthTrends.map(t => t.overallScore)),
        }
      : { min: 0, max: 0 },
    recentTrends: healthTrends.slice(-20), // Last 20 data points
  };
}

// Hook for real-time dependency monitoring
export function useRealTimeDependencyMonitoring() {
  const dependencyMap = useSystemHealthStore((state) => state.dependencyMap);
  const components = useSystemHealthStore((state) => state.components);

  return {
    dependencyMap,
    components,
    dependencyHealth: dependencyMap ? {
      totalDependencies: dependencyMap.edges.length,
      healthyDependencies: dependencyMap.edges.filter(edge => {
        const sourceComponent = components.find(c => c.id === edge.from);
        const targetComponent = components.find(c => c.id === edge.to);
        return sourceComponent?.status === 'healthy' && targetComponent?.status === 'healthy';
      }).length,
      warningDependencies: dependencyMap.edges.filter(edge => {
        const sourceComponent = components.find(c => c.id === edge.from);
        const targetComponent = components.find(c => c.id === edge.to);
        return sourceComponent?.status === 'warning' || targetComponent?.status === 'warning';
      }).length,
      criticalDependencies: dependencyMap.edges.filter(edge => {
        const sourceComponent = components.find(c => c.id === edge.from);
        const targetComponent = components.find(c => c.id === edge.to);
        return sourceComponent?.status === 'critical' || targetComponent?.status === 'critical';
      }).length,
    } : null,
    componentConnectivity: components.map(component => ({
      component,
      dependencies: dependencyMap?.edges.filter(edge => edge.from === component.id || edge.to === component.id) || [],
      dependencyHealth: dependencyMap?.edges
        .filter(edge => edge.from === component.id || edge.to === component.id)
        .every(edge => {
          const otherId = edge.from === component.id ? edge.to : edge.from;
          const otherComponent = components.find(c => c.id === otherId);
          return otherComponent?.status === 'healthy';
        }) || false,
    })),
  };
}
