/**
 * Streaming Metrics Component
 * Provides real-time metrics updates using Server-Sent Events
 */

'use client';

import { useEffect, useState, useRef } from 'react';
import { useDashboardStore } from '@/stores/dashboard';
import { SSEClient } from '@/lib/sse/SSEClient';

interface StreamingMetricsProps {
  endpoint?: string;
  enabled?: boolean;
  onError?: (error: Error) => void;
}

export default function StreamingMetrics({
  endpoint = '/api/metrics/stream',
  enabled = true,
  onError,
}: StreamingMetricsProps) {
  const [isConnected, setIsConnected] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<string | null>(null);
  const sseClientRef = useRef<SSEClient | null>(null);
  
  const { setMetrics, setError, updateLastUpdated } = useDashboardStore();

  useEffect(() => {
    if (!enabled) return;

    const sseClient = new SSEClient({
      url: endpoint,
      onOpen: () => {
        setIsConnected(true);
        setError(null);
      },
      onMessage: (event) => {
        try {
          const data = JSON.parse(event.data);
          
          // Update metrics in store
          setMetrics({
            systemHealth: data.system_health,
            businessMetrics: data.business_metrics,
            agentPerformance: data.agent_performance || [],
            coordinationMetrics: data.coordination_metrics,
          });
          
          setLastUpdate(new Date().toISOString());
          updateLastUpdated();
        } catch (error) {
          console.error('Failed to parse metrics data:', error);
          onError?.(error as Error);
        }
      },
      onError: (error) => {
        setIsConnected(false);
        const errorMessage = error instanceof Error ? error.message : 'Connection error';
        setError(errorMessage);
        onError?.(new Error(errorMessage));
      },
      onClose: () => {
        setIsConnected(false);
      },
    });

    sseClientRef.current = sseClient;

    return () => {
      sseClient.disconnect();
      sseClientRef.current = null;
    };
  }, [endpoint, enabled, setMetrics, setError, updateLastUpdated, onError]);

  // Connection status indicator
  return (
    <div className="streaming-metrics">
      <div className="connection-status">
        <div className={`status-indicator ${isConnected ? 'connected' : 'disconnected'}`}>
          <span className="status-dot" />
          {isConnected ? 'Connected' : 'Disconnected'}
        </div>
        {lastUpdate && (
          <div className="last-update">
            Last update: {new Date(lastUpdate).toLocaleTimeString()}
          </div>
        )}
      </div>
    </div>
  );
}
