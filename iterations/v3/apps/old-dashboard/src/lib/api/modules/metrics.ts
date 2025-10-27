/**
 * Metrics API Module
 * Handles all metrics-related API operations
 */

import { ApiClient } from '../../api-client';
import { SSEClient, SSEOptions } from '../../sse/SSEClient';

export interface MetricsData {
  timestamp: number;
  metrics: {
    cpu_usage_percent: number;
    memory_usage_percent: number;
    disk_usage_percent: number;
    network_rx_bytes: number;
    network_tx_bytes: number;
    active_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    total_requests: number;
    successful_requests: number;
    failed_requests: number;
    avg_response_time_ms: number;
    p95_response_time_ms: number;
    p99_response_time_ms: number;
  };
  components: {
    api: string;
    database: string;
    orchestrator: string;
    workers: string;
  };
}

export class MetricsModule {
  constructor(private apiClient: ApiClient) {}

  async getSystemHealth() {
    return this.apiClient.request('/api/metrics/health');
  }

  async getBusinessMetrics(timeRange?: string) {
    const params = timeRange ? `?time_range=${timeRange}` : '';
    return this.apiClient.request(`/api/metrics/business${params}`);
  }

  async getAgentPerformance() {
    return this.apiClient.request('/api/metrics/agents');
  }

  async getCoordinationMetrics() {
    return this.apiClient.request('/api/metrics/coordination');
  }

  /**
   * Connect to real-time metrics stream using Server-Sent Events
   * @param onMetricsUpdate Callback function called when new metrics arrive
   * @param onError Error callback
   * @returns SSEClient instance for managing the connection
   */
  connectRealTimeMetrics(
    onMetricsUpdate: (data: MetricsData) => void,
    onError?: (error: Event) => void
  ): SSEClient {
    const options: SSEOptions = {
      url: '/api/v1/metrics/stream',
      onMessage: (event) => {
        try {
          const data: MetricsData = JSON.parse(event.data);
          onMetricsUpdate(data);
        } catch (error) {
          console.error('Failed to parse metrics data:', error);
        }
      },
      onError: (error) => {
        console.error('Metrics SSE connection error:', error);
        onError?.(error);
      },
      onOpen: () => {
        console.log('Connected to metrics stream');
      },
      onClose: () => {
        console.log('Disconnected from metrics stream');
      },
      reconnectInterval: 2000, // Reconnect every 2 seconds on failure
      maxReconnectAttempts: 5
    };

    return new SSEClient(options);
  }

  // Legacy method for backward compatibility - returns a promise that never resolves
  // Use connectRealTimeMetrics() instead for actual real-time updates
  async getRealTimeMetrics() {
    console.warn('getRealTimeMetrics() is deprecated. Use connectRealTimeMetrics() for real-time updates.');
    return this.apiClient.request('/api/metrics/stream');
  }
}

