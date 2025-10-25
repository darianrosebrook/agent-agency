/**
 * Alerts API Module
 * Handles all alert-related API operations
 */

import { ApiClient } from '../../api-client';

export class AlertsModule {
  constructor(private apiClient: ApiClient) {}

  async getAlerts(filters?: {
    severity?: string;
    status?: string;
    limit?: number;
  }) {
    const params = new URLSearchParams();
    if (filters?.severity) params.append('severity', filters.severity);
    if (filters?.status) params.append('status', filters.status);
    if (filters?.limit) params.append('limit', filters.limit.toString());
    
    const queryString = params.toString();
    return this.apiClient.request(`/api/alerts${queryString ? `?${queryString}` : ''}`);
  }

  async getAlert(alertId: string) {
    return this.apiClient.request(`/api/alerts/${alertId}`);
  }

  async acknowledgeAlert(alertId: string) {
    return this.apiClient.request(`/api/alerts/${alertId}/acknowledge`, {
      method: 'POST',
    });
  }

  async resolveAlert(alertId: string) {
    return this.apiClient.request(`/api/alerts/${alertId}/resolve`, {
      method: 'POST',
    });
  }

  async getAlertStatistics() {
    return this.apiClient.request('/api/alerts/statistics');
  }

  async getSLOAlerts() {
    return this.apiClient.request('/api/slo-alerts');
  }

  async acknowledgeSLOAlert(alertId: string) {
    return this.apiClient.request(`/api/slo-alerts/${alertId}/acknowledge`, {
      method: 'POST',
    });
  }
}

