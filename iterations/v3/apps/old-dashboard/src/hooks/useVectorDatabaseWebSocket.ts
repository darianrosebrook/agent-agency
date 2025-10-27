/**
 * Vector Database WebSocket Hook
 * Real-time updates for vector database operations
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useVectorDatabaseStore, useVectorDatabaseActions } from '@/stores/vector-database';
import { VectorEmbedding, VectorSearchResult, VectorCluster, VectorAnalytics } from '@/lib/vector-database-api';

interface WebSocketMessage {
  type: 'vector_added' | 'vector_updated' | 'vector_deleted' | 'search_completed' | 'cluster_updated' | 'analytics_updated' | 'performance_updated';
  data: any;
  timestamp: string;
}

export function useVectorDatabaseWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useVectorDatabaseActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');
    
    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/vector-database`);
      
      ws.onopen = () => {
        console.log('Vector Database WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;
        
        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('Vector Database WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');
        
        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('Vector Database WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create Vector Database WebSocket connection:', error);
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

    console.log(`Scheduling Vector Database WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);
    
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

  const handleMessage = (message: WebSocketMessage) => {
    const { type, data, timestamp } = message;

    switch (type) {
      case 'vector_added':
        actions.addVector(data as VectorEmbedding);
        break;

      case 'vector_updated':
        actions.updateVector(data.id, data.updates);
        break;

      case 'vector_deleted':
        actions.deleteVector(data.id);
        break;

      case 'search_completed':
        actions.setSearchResults(data.results as VectorSearchResult[]);
        actions.setLoading('search', false);
        break;

      case 'cluster_updated':
        if (data.clusters) {
          actions.setClusters(data.clusters as VectorCluster[]);
        } else if (data.cluster) {
          actions.addCluster(data.cluster as VectorCluster);
        }
        break;

      case 'analytics_updated':
        actions.setAnalytics(data as VectorAnalytics);
        actions.setLoading('analytics', false);
        break;

      case 'performance_updated':
        actions.setPerformanceMetrics(data);
        actions.setLoading('performance', false);
        break;

      default:
        console.warn('Unknown Vector Database WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('Vector Database WebSocket not connected, cannot send message');
    }
  };

  // Subscribe to specific events
  const subscribe = (eventTypes: string[]) => {
    sendMessage({
      type: 'subscribe',
      events: eventTypes
    });
  };

  // Unsubscribe from events
  const unsubscribe = (eventTypes: string[]) => {
    sendMessage({
      type: 'unsubscribe',
      events: eventTypes
    });
  };

  // Request real-time analytics
  const requestAnalytics = () => {
    sendMessage({
      type: 'request_analytics'
    });
  };

  // Request performance metrics
  const requestPerformanceMetrics = (timeRange: '1h' | '6h' | '24h' | '7d' = '24h') => {
    sendMessage({
      type: 'request_performance',
      timeRange
    });
  };

  // Request cluster updates
  const requestClusterUpdates = () => {
    sendMessage({
      type: 'request_clusters'
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
    requestAnalytics,
    requestPerformanceMetrics,
    requestClusterUpdates,
  };
}

// Hook for real-time vector monitoring
export function useRealTimeVectorMonitoring() {
  const vectors = useVectorDatabaseStore((state) => state.vectors);
  const clusters = useVectorDatabaseStore((state) => state.clusters);
  const analytics = useVectorDatabaseStore((state) => state.analytics);
  const loading = useVectorDatabaseStore((state) => state.loading);

  return {
    vectors,
    clusters,
    analytics,
    loading,
    totalVectors: vectors.length,
    clusterCount: clusters.length,
    averageDimensions: vectors.length > 0 
      ? vectors.reduce((sum, v) => sum + v.embedding.length, 0) / vectors.length 
      : 0,
  };
}

// Hook for real-time search monitoring
export function useRealTimeSearchMonitoring() {
  const searchResults = useVectorDatabaseStore((state) => state.searchResults);
  const searchQuery = useVectorDatabaseStore((state) => state.searchQuery);
  const loading = useVectorDatabaseStore((state) => state.loading.search);

  return {
    searchResults,
    searchQuery,
    loading,
    resultCount: searchResults.length,
    averageSimilarity: searchResults.length > 0
      ? searchResults.reduce((sum, r) => sum + r.similarity, 0) / searchResults.length
      : 0,
  };
}
