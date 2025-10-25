/**
 * Metrics API Module
 * Handles all metrics-related API operations
 */

import { ApiClient } from '../../api-client';

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

  async getRealTimeMetrics() {
    return this.apiClient.request('/api/metrics/stream');
  }
}

