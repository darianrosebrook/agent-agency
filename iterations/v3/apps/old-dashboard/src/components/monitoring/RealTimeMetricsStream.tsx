"use client";

import { useEffect, useRef, useCallback, useState } from "react";
import { SSEClient } from "@/lib/sse/SSEClient";
import {
  RealTimeMetricsStreamProps,
  MetricsStreamEvent,
} from "@/types/metrics";

// Real-time system metrics from V3 backend
interface V3MetricsData {
  timestamp: number;
  metrics: {
    cpu_usage_percent: number;
    memory_usage_percent: number;
    active_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    avg_response_time_ms: number;
  };
  components: {
    api: string;
    database: string;
    orchestrator: string;
    workers: string;
  };
}

export default function RealTimeMetricsStream({
  onMetricsUpdate,
  onError,
  enabled = true,
}: RealTimeMetricsStreamProps) {
  const sseClientRef = useRef<SSEClient | null>(null);
  const enabledRef = useRef(enabled);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const [lastUpdateTime, setLastUpdateTime] = useState<Date | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  // Update enabled ref when prop changes
  useEffect(() => {
    enabledRef.current = enabled;
  }, [enabled]);

  const handleMetricsEvent = useCallback(
    (event: any) => {
      if (!enabledRef.current) return;

      try {
        // Parse the SSE data from V3 backend
        const v3Data: V3MetricsData = JSON.parse(event.data);

        // Convert to standardized metrics event format
        const metricsEvent: MetricsStreamEvent = {
          type: "health_update",
          timestamp: new Date(v3Data.timestamp).toISOString(),
          data: {
            system_health: {
              components: [
                { name: "api", status: v3Data.components.api },
                { name: "database", status: v3Data.components.database },
                { name: "orchestrator", status: v3Data.components.orchestrator },
                { name: "workers", status: v3Data.components.workers },
              ],
            },
            coordination_metrics: {
              tasks_per_minute: Math.floor(v3Data.metrics.active_tasks * 6), // Estimate based on active tasks
              efficiency_percentage: Math.max(0, Math.min(100,
                100 - (v3Data.metrics.failed_tasks / Math.max(1, v3Data.metrics.completed_tasks + v3Data.metrics.active_tasks)) * 100
              )),
            },
            agent_performance: [{
              agent_id: "system",
              name: "System Agents",
              status: "active",
              average_response_time_ms: v3Data.metrics.avg_response_time_ms,
              active_tasks: v3Data.metrics.active_tasks,
            }],
            business_metrics: {
              error_rate: v3Data.metrics.failed_tasks / Math.max(1, v3Data.metrics.completed_tasks + v3Data.metrics.failed_tasks),
              throughput: v3Data.metrics.completed_tasks,
            },
          },
          event_id: `v3_metrics_${v3Data.timestamp}`,
        };

        onMetricsUpdate?.(metricsEvent);
      } catch (error) {
        console.error("Failed to process V3 metrics event:", error, event.data);
        onError?.(error as Event);
      }
    },
    [onMetricsUpdate, onError]
  );

  const handleSSEError = useCallback(
    (error: Event) => {
      console.error("SSE connection error:", error);
      setConnectionStatus('error');
      onError?.(error);
      
      // Attempt reconnection with exponential backoff
      if (reconnectAttempts.current < maxReconnectAttempts) {
        const delay = reconnectDelay * Math.pow(2, reconnectAttempts.current);
        console.log(`Attempting reconnection in ${delay}ms (attempt ${reconnectAttempts.current + 1}/${maxReconnectAttempts})`);
        
        reconnectTimeoutRef.current = setTimeout(() => {
          reconnectAttempts.current++;
          initializeConnection();
        }, delay);
      } else {
        console.error("Max reconnection attempts reached");
        setConnectionStatus('disconnected');
      }
    },
    [onError]
  );

  const handleSSEOpen = useCallback(() => {
    console.log("Metrics SSE connection opened");
    setConnectionStatus('connected');
    setLastUpdateTime(new Date());
    reconnectAttempts.current = 0; // Reset on successful connection
  }, []);

  const handleSSEClose = useCallback(() => {
    console.log("Metrics SSE connection closed");
    setConnectionStatus('disconnected');
  }, []);

  // Initialize connection function
  const initializeConnection = useCallback(() => {
    if (!enabledRef.current) return;

    console.log("Connecting to V3 metrics stream...");
    setConnectionStatus('connecting');

    // Connect to real V3 backend metrics stream
    const streamUrl = `${process.env.V3_BACKEND_HOST ?? 'http://localhost:8080'}/api/v1/metrics/stream`;

    sseClientRef.current = new SSEClient({
      url: streamUrl,
      onMessage: (event) => {
        handleMetricsEvent(event);
        setLastUpdateTime(new Date());
      },
      onError: handleSSEError,
      onOpen: handleSSEOpen,
      onClose: handleSSEClose,
    });
  }, [handleMetricsEvent, handleSSEError, handleSSEOpen, handleSSEClose]);

  // Initialize SSE connection to V3 backend
  useEffect(() => {
    if (!enabled) return;

    initializeConnection();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (sseClientRef.current) {
        sseClientRef.current.destroy();
        sseClientRef.current = null;
      }
    };
  }, [enabled, initializeConnection]);

  // Handle enabled/disabled changes
  useEffect(() => {
    if (!sseClientRef.current) return;

    if (enabled) {
      // Reconnect if enabled
      sseClientRef.current.reconnect();
    } else {
      // Disconnect if disabled
      sseClientRef.current.disconnect();
    }
  }, [enabled]);

  // Return connection status for debugging and monitoring
  return (
    <div style={{ display: 'none' }} data-connection-status={connectionStatus} data-last-update={lastUpdateTime?.toISOString()}>
      {/* Hidden status indicator for debugging */}
    </div>
  );
}
