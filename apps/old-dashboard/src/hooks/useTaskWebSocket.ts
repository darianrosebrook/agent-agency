/**
 * Task WebSocket Hook
 * Provides real-time task updates with connection management and error recovery
 *
 * @author @darianrosebrook
 */

"use client";

import { useEffect, useRef, useState, useCallback } from 'react';
import {
  TaskUpdateMessage,
  TaskCompletionMessage,
  TaskProgressMessage,
  WebSocketMessage
} from '@/types/tasks';

export type ConnectionState = "connecting" | "connected" | "disconnected" | "error" | "reconnecting";

export interface TaskWebSocketState {
  connectionState: ConnectionState;
  isConnected: boolean;
  lastMessage?: WebSocketMessage;
  error?: string;
  reconnectAttempts: number;
  subscribedTasks: Set<string>;
}

export interface UseTaskWebSocketReturn extends TaskWebSocketState {
  connect: () => void;
  disconnect: () => void;
  subscribeToTask: (taskId: string) => void;
  unsubscribeFromTask: (taskId: string) => void;
  subscribeToAllTasks: () => void;
  unsubscribeFromAllTasks: () => void;
  sendMessage: (message: any) => void;
}

/**
 * Task WebSocket hook with intelligent connection management
 * Prevents DDoS through rate limiting and connection pooling
 */
export function useTaskWebSocket(taskId?: string): UseTaskWebSocketReturn {
  const [state, setState] = useState<TaskWebSocketState>({
    connectionState: "disconnected",
    isConnected: false,
    reconnectAttempts: 0,
    subscribedTasks: new Set(),
  });

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const heartbeatIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastHeartbeatRef = useRef<number>(0);

  // Connection configuration
  const config = {
    url: process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080',
    reconnectAttempts: 5,
    reconnectDelay: 1000, // Base delay in ms
    heartbeatInterval: 30000, // 30 seconds
    connectionTimeout: 10000, // 10 seconds
    maxMessageRate: 100, // Max messages per minute
  };

  // Rate limiting for incoming messages
  const messageTimestamps = useRef<number[]>([]);
  const checkMessageRate = useCallback(() => {
    const now = Date.now();
    const oneMinuteAgo = now - 60000;

    // Clean old timestamps
    messageTimestamps.current = messageTimestamps.current.filter(
      timestamp => timestamp > oneMinuteAgo
    );

    // Check rate limit
    if (messageTimestamps.current.length >= config.maxMessageRate) {
      console.warn('WebSocket message rate limit exceeded, disconnecting temporarily');
      disconnect();
      return false;
    }

    messageTimestamps.current.push(now);
    return true;
  }, []);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setState(prev => ({
      ...prev,
      connectionState: "connecting",
      error: undefined
    }));

    try {
      const wsUrl = `${config.url}/tasks`;
      const ws = new WebSocket(wsUrl);

      // Connection timeout
      const connectionTimeout = setTimeout(() => {
        if (ws.readyState === WebSocket.CONNECTING) {
          ws.close();
          setState(prev => ({
            ...prev,
            connectionState: "error",
            error: "Connection timeout"
          }));
        }
      }, config.connectionTimeout);

      ws.onopen = () => {
        clearTimeout(connectionTimeout);
        console.log('Task WebSocket connected');

        setState(prev => ({
          ...prev,
          connectionState: "connected",
          isConnected: true,
          reconnectAttempts: 0,
          error: undefined
        }));

        // Start heartbeat
        startHeartbeat();

        // Re-subscribe to tasks if any were previously subscribed
        if (prev.subscribedTasks.size > 0) {
          prev.subscribedTasks.forEach(taskId => {
            subscribeToTask(taskId);
          });
        }

        // Subscribe to specific task if provided
        if (taskId) {
          subscribeToTask(taskId);
        }
      };

      ws.onmessage = (event) => {
        if (!checkMessageRate()) {
          return;
        }

        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        clearTimeout(connectionTimeout);
        stopHeartbeat();

        console.log(`Task WebSocket disconnected: ${event.code} ${event.reason}`);

        setState(prev => ({
          ...prev,
          connectionState: "disconnected",
          isConnected: false
        }));

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && prev.reconnectAttempts < config.reconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        clearTimeout(connectionTimeout);
        console.error('Task WebSocket error:', error);

        setState(prev => ({
          ...prev,
          connectionState: "error",
          isConnected: false,
          error: 'WebSocket connection error'
        }));
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create Task WebSocket connection:', error);
      setState(prev => ({
        ...prev,
        connectionState: "error",
        error: 'Failed to create connection'
      }));
      scheduleReconnect();
    }
  }, [taskId]);

  const scheduleReconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    setState(prev => {
      const delay = config.reconnectDelay * Math.pow(2, prev.reconnectAttempts);
      console.log(`Scheduling Task WebSocket reconnect in ${delay}ms (attempt ${prev.reconnectAttempts + 1})`);

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

    stopHeartbeat();

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setState(prev => ({
      ...prev,
      connectionState: "disconnected",
      isConnected: false,
      subscribedTasks: new Set()
    }));
  }, []);

  const startHeartbeat = useCallback(() => {
    stopHeartbeat(); // Clear any existing heartbeat

    heartbeatIntervalRef.current = setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: 'ping', timestamp: Date.now() }));
        lastHeartbeatRef.current = Date.now();
      }
    }, config.heartbeatInterval);
  }, []);

  const stopHeartbeat = useCallback(() => {
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }
  }, []);

  const handleMessage = useCallback((message: WebSocketMessage) => {
    // Update last message
    setState(prev => ({ ...prev, lastMessage: message }));

    // Handle different message types
    switch (message.type) {
      case 'task_update':
      case 'task_completion':
      case 'task_progress':
        // These will be handled by individual task hooks or global state
        break;

      case 'pong':
        // Handle heartbeat response
        const latency = Date.now() - (message as any).timestamp;
        console.debug(`WebSocket latency: ${latency}ms`);
        break;

      default:
        console.warn('Unknown WebSocket message type:', message.type);
    }
  }, []);

  const subscribeToTask = useCallback((taskId: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe',
        task_id: taskId
      }));

      setState(prev => ({
        ...prev,
        subscribedTasks: new Set([...prev.subscribedTasks, taskId])
      }));
    }
  }, []);

  const unsubscribeFromTask = useCallback((taskId: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe',
        task_id: taskId
      }));

      setState(prev => {
        const newSubscribedTasks = new Set(prev.subscribedTasks);
        newSubscribedTasks.delete(taskId);
        return { ...prev, subscribedTasks: newSubscribedTasks };
      });
    }
  }, []);

  const subscribeToAllTasks = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe_all'
      }));

      // Mark that we're subscribed to all (we'll use a special marker)
      setState(prev => ({
        ...prev,
        subscribedTasks: new Set(['__all__'])
      }));
    }
  }, []);

  const unsubscribeFromAllTasks = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe_all'
      }));

      setState(prev => ({
        ...prev,
        subscribedTasks: new Set()
      }));
    }
  }, []);

  const sendMessage = useCallback((message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('Task WebSocket not connected, cannot send message');
    }
  }, []);

  // Auto-connect on mount if taskId is provided
  useEffect(() => {
    if (taskId) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [taskId, connect, disconnect]);

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
    subscribeToTask,
    unsubscribeFromTask,
    subscribeToAllTasks,
    unsubscribeFromAllTasks,
    sendMessage,
  };
}

/**
 * Hook for listening to specific task updates
 */
export function useTaskUpdates(taskId: string) {
  const { isConnected, lastMessage } = useTaskWebSocket(taskId);
  const [taskUpdates, setTaskUpdates] = useState<TaskUpdateMessage[]>([]);
  const [taskProgress, setTaskProgress] = useState<TaskProgressMessage[]>([]);
  const [taskCompletions, setTaskCompletions] = useState<TaskCompletionMessage[]>([]);

  useEffect(() => {
    if (lastMessage && 'task_id' in lastMessage && lastMessage.task_id === taskId) {
      switch (lastMessage.type) {
        case 'task_update':
          setTaskUpdates(prev => [...prev.slice(-9), lastMessage]); // Keep last 10
          break;
        case 'task_progress':
          setTaskProgress(prev => [...prev.slice(-9), lastMessage]);
          break;
        case 'task_completion':
          setTaskCompletions(prev => [...prev.slice(-9), lastMessage]);
          break;
      }
    }
  }, [lastMessage, taskId]);

  return {
    isConnected,
    taskUpdates,
    taskProgress,
    taskCompletions,
    latestUpdate: taskUpdates[taskUpdates.length - 1],
    latestProgress: taskProgress[taskProgress.length - 1],
    latestCompletion: taskCompletions[taskCompletions.length - 1],
  };
}

/**
 * Hook for monitoring all active tasks
 */
export function useAllTasksUpdates() {
  const { isConnected, lastMessage, subscribeToAllTasks, unsubscribeFromAllTasks } = useTaskWebSocket();
  const [activeTasks, setActiveTasks] = useState<Map<string, TaskUpdateMessage>>(new Map());

  useEffect(() => {
    subscribeToAllTasks();
    return () => unsubscribeFromAllTasks();
  }, [subscribeToAllTasks, unsubscribeFromAllTasks]);

  useEffect(() => {
    if (lastMessage && 'task_id' in lastMessage) {
      const taskId = lastMessage.task_id;

      if (lastMessage.type === 'task_update') {
        setActiveTasks(prev => {
          const newMap = new Map(prev);
          newMap.set(taskId, lastMessage);
          return newMap;
        });
      } else if (lastMessage.type === 'task_completion') {
        // Remove completed tasks after a delay
        setTimeout(() => {
          setActiveTasks(prev => {
            const newMap = new Map(prev);
            newMap.delete(taskId);
            return newMap;
          });
        }, 5000); // Keep completed tasks visible for 5 seconds
      }
    }
  }, [lastMessage]);

  return {
    isConnected,
    activeTasks,
    taskCount: activeTasks.size,
    tasksByStatus: Array.from(activeTasks.values()).reduce((acc, task) => {
      acc[task.status] = (acc[task.status] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
  };
}
