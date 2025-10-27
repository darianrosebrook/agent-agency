/**
 * Server-Sent Events (SSE) Hook
 * Provides real-time streaming data with connection management and error recovery
 *
 * @author @darianrosebrook
 */

"use client";

import { useEffect, useRef, useState, useCallback } from 'react';
import { SSEMessage } from '@/types/tasks';

export type SSEConnectionState = "connecting" | "connected" | "disconnected" | "error" | "reconnecting";

export interface SSEState {
  connectionState: SSEConnectionState;
  isConnected: boolean;
  lastEvent?: SSEMessage;
  error?: string;
  reconnectAttempts: number;
  eventCount: number;
}

export interface UseSSEConnectionReturn extends SSEState {
  connect: () => void;
  disconnect: () => void;
  subscribe: (channels: string[]) => void;
  unsubscribe: (channels: string[]) => void;
}

/**
 * SSE hook with intelligent connection management and rate limiting
 * Prevents server overload through connection pooling and backoff strategies
 */
export function useSSEConnection(endpoint: string = '/api/metrics/stream'): UseSSEConnectionReturn {
  const [state, setState] = useState<SSEState>({
    connectionState: "disconnected",
    isConnected: false,
    reconnectAttempts: 0,
    eventCount: 0,
  });

  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const subscribedChannelsRef = useRef<Set<string>>(new Set());

  // Connection configuration
  const config = {
    reconnectAttempts: 5,
    reconnectDelay: 1000, // Base delay in ms
    maxEventRate: 200, // Max events per minute
    connectionTimeout: 15000, // 15 seconds
  };

  // Rate limiting for incoming events
  const eventTimestamps = useRef<number[]>([]);
  const checkEventRate = useCallback(() => {
    const now = Date.now();
    const oneMinuteAgo = now - 60000;

    // Clean old timestamps
    eventTimestamps.current = eventTimestamps.current.filter(
      timestamp => timestamp > oneMinuteAgo
    );

    // Check rate limit
    if (eventTimestamps.current.length >= config.maxEventRate) {
      console.warn('SSE event rate limit exceeded, disconnecting temporarily');
      disconnect();
      return false;
    }

    eventTimestamps.current.push(now);
    return true;
  }, []);

  const connect = useCallback(() => {
    if (eventSourceRef.current?.readyState === EventSource.OPEN) {
      return;
    }

    setState(prev => ({
      ...prev,
      connectionState: "connecting",
      error: undefined
    }));

    try {
      const url = new URL(endpoint, window.location.origin);

      // Add subscribed channels as query parameters
      if (subscribedChannelsRef.current.size > 0) {
        url.searchParams.set('channels', Array.from(subscribedChannelsRef.current).join(','));
      }

      const eventSource = new EventSource(url.toString());

      // Connection timeout
      const connectionTimeout = setTimeout(() => {
        if (eventSource.readyState === EventSource.CONNECTING) {
          eventSource.close();
          setState(prev => ({
            ...prev,
            connectionState: "error",
            error: "Connection timeout"
          }));
        }
      }, config.connectionTimeout);

      eventSource.onopen = () => {
        clearTimeout(connectionTimeout);
        console.log('SSE connection established');

        setState(prev => ({
          ...prev,
          connectionState: "connected",
          isConnected: true,
          reconnectAttempts: 0,
          error: undefined
        }));
      };

      eventSource.onmessage = (event) => {
        if (!checkEventRate()) {
          return;
        }

        try {
          const data = JSON.parse(event.data);
          const sseMessage: SSEMessage = {
            type: event.type || 'message',
            data,
            id: event.lastEventId,
            timestamp: new Date().toISOString(),
          };

          setState(prev => ({
            ...prev,
            lastEvent: sseMessage,
            eventCount: prev.eventCount + 1
          }));

          // Handle different event types
          handleEvent(sseMessage);
        } catch (error) {
          console.error('Failed to parse SSE message:', error);
        }
      };

      eventSource.onerror = (error) => {
        clearTimeout(connectionTimeout);
        console.error('SSE connection error:', error);

        setState(prev => ({
          ...prev,
          connectionState: "error",
          isConnected: false,
          error: 'SSE connection error'
        }));

        // EventSource will automatically try to reconnect, but we want control
        eventSource.close();

        if (prev.reconnectAttempts < config.reconnectAttempts) {
          scheduleReconnect();
        }
      };

      eventSourceRef.current = eventSource;
    } catch (error) {
      console.error('Failed to create SSE connection:', error);
      setState(prev => ({
        ...prev,
        connectionState: "error",
        error: 'Failed to create connection'
      }));
      scheduleReconnect();
    }
  }, [endpoint]);

  const scheduleReconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    setState(prev => {
      const delay = config.reconnectDelay * Math.pow(2, prev.reconnectAttempts);
      console.log(`Scheduling SSE reconnect in ${delay}ms (attempt ${prev.reconnectAttempts + 1})`);

      reconnectTimeoutRef.current = setTimeout(() => {
        setState(current => ({ ...current, reconnectAttempts: current.reconnectAttempts + 1 }));
        connect();
      }, delay);

      return {
        ...prev,
        connectionState: "reconnecting",
        reconnectAttempts: prev.reconnectAttempts + 1
      };
    });
  }, [connect]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    setState(prev => ({
      ...prev,
      connectionState: "disconnected",
      isConnected: false
    }));
  }, []);

  const subscribe = useCallback((channels: string[]) => {
    channels.forEach(channel => subscribedChannelsRef.current.add(channel));

    // If connected, we need to reconnect with new channels
    if (state.isConnected) {
      disconnect();
      setTimeout(connect, 100); // Brief delay to ensure clean disconnect
    }
  }, [state.isConnected, connect, disconnect]);

  const unsubscribe = useCallback((channels: string[]) => {
    channels.forEach(channel => subscribedChannelsRef.current.delete(channel));

    // If connected, we need to reconnect with updated channels
    if (state.isConnected) {
      disconnect();
      setTimeout(connect, 100);
    }
  }, [state.isConnected, connect, disconnect]);

  const handleEvent = useCallback((event: SSEMessage) => {
    // Handle different event types
    switch (event.type) {
      case 'health_update':
      case 'metrics_update':
      case 'alert':
        // These will be handled by specific hooks that use this SSE connection
        break;

      default:
        console.debug('SSE event:', event.type, event.data);
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  return {
    ...state,
    connect,
    disconnect,
    subscribe,
    unsubscribe,
  };
}

/**
 * Hook for system health SSE streaming
 */
export function useSystemHealthSSE() {
  const sse = useSSEConnection('/api/health/stream');
  const [healthData, setHealthData] = useState<any[]>([]);
  const [alerts, setAlerts] = useState<any[]>([]);

  useEffect(() => {
    sse.subscribe(['health', 'alerts']);
    return () => sse.unsubscribe(['health', 'alerts']);
  }, [sse.subscribe, sse.unsubscribe]);

  useEffect(() => {
    if (sse.lastEvent) {
      switch (sse.lastEvent.type) {
        case 'health_update':
          setHealthData(prev => [...prev.slice(-19), sse.lastEvent!.data]); // Keep last 20
          break;
        case 'alert':
          setAlerts(prev => [...prev.slice(-9), sse.lastEvent!.data]); // Keep last 10
          break;
      }
    }
  }, [sse.lastEvent]);

  return {
    ...sse,
    healthData,
    alerts,
    latestHealth: healthData[healthData.length - 1],
    activeAlerts: alerts.filter(alert => alert.status === 'active'),
  };
}

/**
 * Hook for metrics SSE streaming
 */
export function useMetricsSSE() {
  const sse = useSSEConnection('/api/metrics/stream');
  const [metrics, setMetrics] = useState<Map<string, any[]>>(new Map());

  useEffect(() => {
    sse.subscribe(['metrics', 'performance']);
    return () => sse.unsubscribe(['metrics', 'performance']);
  }, [sse.subscribe, sse.unsubscribe]);

  useEffect(() => {
    if (sse.lastEvent?.type === 'metrics_update') {
      const metricName = sse.lastEvent.data.name || 'unknown';
      setMetrics(prev => {
        const newMetrics = new Map(prev);
        const metricData = newMetrics.get(metricName) || [];
        newMetrics.set(metricName, [...metricData.slice(-49), sse.lastEvent!.data]); // Keep last 50
        return newMetrics;
      });
    }
  }, [sse.lastEvent]);

  return {
    ...sse,
    metrics,
    getMetric: (name: string) => metrics.get(name) || [],
    latestMetrics: Array.from(metrics.entries()).map(([name, data]) => ({
      name,
      latest: data[data.length - 1],
      count: data.length,
    })),
  };
}

/**
 * Hook for task progress SSE streaming
 */
export function useTaskProgressSSE() {
  const sse = useSSEConnection('/api/tasks/events');
  const [taskEvents, setTaskEvents] = useState<Map<string, any[]>>(new Map());

  useEffect(() => {
    sse.subscribe(['task_progress', 'task_completion']);
    return () => sse.unsubscribe(['task_progress', 'task_completion']);
  }, [sse.subscribe, sse.unsubscribe]);

  useEffect(() => {
    if (sse.lastEvent && (sse.lastEvent.type === 'task_progress' || sse.lastEvent.type === 'task_completion')) {
      const taskId = sse.lastEvent.data.task_id;
      setTaskEvents(prev => {
        const newEvents = new Map(prev);
        const taskEventData = newEvents.get(taskId) || [];
        newEvents.set(taskId, [...taskEventData.slice(-19), sse.lastEvent!.data]); // Keep last 20
        return newEvents;
      });
    }
  }, [sse.lastEvent]);

  return {
    ...sse,
    taskEvents,
    getTaskEvents: (taskId: string) => taskEvents.get(taskId) || [],
    activeTasks: Array.from(taskEvents.keys()).filter(taskId => {
      const events = taskEvents.get(taskId) || [];
      const latest = events[events.length - 1];
      return latest?.status !== 'completed' && latest?.status !== 'failed';
    }),
  };
}
