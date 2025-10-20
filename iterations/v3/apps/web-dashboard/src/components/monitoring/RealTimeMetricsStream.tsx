"use client";

import React, { useEffect, useRef, useCallback } from "react";
import { SSEClient } from "@/lib/sse/SSEClient";
import { RealTimeMetricsStreamProps, MetricsStreamEvent } from "@/types/metrics";
import { metricsApiClient } from "@/lib/metrics-api";

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
        const metricsEvent: MetricsStreamEvent = {
          type: event.type,
          timestamp: event.timestamp || new Date().toISOString(),
          data: event.data,
          event_id: event.event_id || `event_${Date.now()}`,
        };

        onMetricsUpdate?.(metricsEvent);
      } catch (error) {
        console.error("Failed to process metrics event:", error, event);
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

  // Initialize SSE connection
  useEffect(() => {
    if (!enabled) return;

    console.warn("Real-time metrics stream not implemented - requires V3 metrics SSE endpoint");
    // TODO: Milestone 3 - Real-time Metrics Streaming
    // - [ ] Implement V3 /metrics/stream SSE endpoint
    // - [ ] Add event types: health_update, agent_performance, coordination_update, business_metrics
    // - [ ] Implement event sequencing and deduplication
    // - [ ] Add configurable event filtering and sampling
    // - [ ] Test SSE reconnection and event replay
    // - [ ] Implement client-side buffering for offline periods
    // - [ ] Add metrics for SSE connection health and event throughput

    // For now, show empty state - no mock data
    return;

    // Future implementation:
    /*
    const streamUrl = metricsApiClient.getMetricsStreamUrl();

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
    */
  }, [enabled, handleMetricsEvent, handleSSEError, handleSSEOpen, handleSSEClose]);

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
