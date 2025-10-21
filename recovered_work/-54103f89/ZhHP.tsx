"use client";

import { useEffect, useRef, useCallback } from "react";
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
      onError?.(error);
    },
    [onError]
  );

  const handleSSEOpen = useCallback(() => {
    console.log("Metrics SSE connection opened");
  }, []);

  const handleSSEClose = useCallback(() => {
    console.log("Metrics SSE connection closed");
  }, []);

  // Initialize SSE connection to V3 backend
  useEffect(() => {
    if (!enabled) return;

    console.log("Connecting to V3 metrics stream...");

    // Connect to real V3 backend metrics stream
    const streamUrl = `${process.env.V3_BACKEND_HOST ?? 'http://localhost:8080'}/api/v1/metrics/stream`;

    sseClientRef.current = new SSEClient({
      url: streamUrl,
      onMessage: handleMetricsEvent,
      onError: handleSSEError,
      onOpen: handleSSEOpen,
      onClose: handleSSEClose,
    });

    return () => {
      if (sseClientRef.current) {
        sseClientRef.current.destroy();
        sseClientRef.current = null;
      }
    };
  }, [
    enabled,
    handleMetricsEvent,
    handleSSEError,
    handleSSEOpen,
    handleSSEClose,
  ]);

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

  // This component doesn't render anything visible - it's a data provider
  return null;
}
