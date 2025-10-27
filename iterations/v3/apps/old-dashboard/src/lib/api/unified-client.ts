/**
 * Unified API Client for Agent Agency V3 Dashboard
 * Consolidates all API operations into a single, optimized client
 * 
 * @author @darianrosebrook
 */

import { ApiClient } from '../api-client';

// Re-export types for convenience
export type { ApiConfig } from '../api-client';

/**
 * Unified API Client with modular architecture
 * Provides access to all dashboard APIs through a single interface
 */
export class UnifiedApiClient {
  private baseClient: ApiClient;

  constructor(config?: Partial<{
    baseUrl: string;
    timeout: number;
    maxRetries: number;
    retryDelay: number;
    authToken: string;
  }>) {
    this.baseClient = new ApiClient(config);
  }

  // Health & System APIs
  async getHealth() {
    return this.baseClient.request('/api/health');
  }

  async getSystemHealth() {
    return this.baseClient.request('/api/metrics/health');
  }

  // Metrics APIs
  async getMetrics(timeRange?: string) {
    const params = timeRange ? `?time_range=${timeRange}` : '';
    return this.baseClient.request(`/api/metrics${params}`);
  }

  async getBusinessMetrics(timeRange?: string) {
    const params = timeRange ? `?time_range=${timeRange}` : '';
    return this.baseClient.request(`/api/metrics/business${params}`);
  }

  async getAgentPerformance() {
    return this.baseClient.request('/api/metrics/agents');
  }

  // Tasks APIs
  async getTasks(filters?: {
    status?: string;
    priority?: string;
    limit?: number;
    offset?: number;
  }) {
    const params = new URLSearchParams();
    if (filters?.status) params.append('status', filters.status);
    if (filters?.priority) params.append('priority', filters.priority);
    if (filters?.limit) params.append('limit', filters.limit.toString());
    if (filters?.offset) params.append('offset', filters.offset.toString());
    
    const queryString = params.toString();
    return this.baseClient.request(`/api/tasks${queryString ? `?${queryString}` : ''}`);
  }

  async getTask(taskId: string) {
    return this.baseClient.request(`/api/tasks/${taskId}`);
  }

  async updateTask(taskId: string, action: string, data?: any) {
    return this.baseClient.request(`/api/tasks/${taskId}/action`, {
      method: 'POST',
      body: JSON.stringify({ action, ...data }),
    });
  }

  // Alerts APIs
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
    return this.baseClient.request(`/api/alerts${queryString ? `?${queryString}` : ''}`);
  }

  async acknowledgeAlert(alertId: string) {
    return this.baseClient.request(`/api/alerts/${alertId}/acknowledge`, {
      method: 'POST',
    });
  }

  async resolveAlert(alertId: string) {
    return this.baseClient.request(`/api/alerts/${alertId}/resolve`, {
      method: 'POST',
    });
  }

  // SLO APIs
  async getSLOs() {
    return this.baseClient.request('/api/slos');
  }

  async getSLOStatus(sloName: string) {
    return this.baseClient.request(`/api/slos/${sloName}/status`);
  }

  async getSLOMeasurements(sloName: string, timeRange?: string) {
    const params = timeRange ? `?time_range=${timeRange}` : '';
    return this.baseClient.request(`/api/slos/${sloName}/measurements${params}`);
  }

  // Database APIs
  async getDatabaseConnections() {
    return this.baseClient.request('/api/database/connections');
  }

  async getDatabaseTables(connectionId: string) {
    return this.baseClient.request(`/api/database/tables?connection_id=${connectionId}`);
  }

  async queryDatabase(query: string, connectionId: string) {
    return this.baseClient.request('/api/database/query', {
      method: 'POST',
      body: JSON.stringify({ query, connection_id: connectionId }),
    });
  }

  // Analytics APIs
  async getAnalytics(timeRange?: string) {
    const params = timeRange ? `?time_range=${timeRange}` : '';
    return this.baseClient.request(`/api/analytics${params}`);
  }

  // TTS APIs
  async generateSpeech(text: string, options?: {
    voice?: string;
    speed?: number;
    pitch?: number;
  }) {
    return this.baseClient.request('/api/tts', {
      method: 'POST',
      body: JSON.stringify({ text, ...options }),
    });
  }

  // WebSocket connection for real-time updates
  createWebSocketConnection(endpoint: string) {
    const wsUrl = endpoint.startsWith('/') 
      ? `${window.location.origin.replace('http', 'ws')}${endpoint}`
      : endpoint;
    
    return new WebSocket(wsUrl);
  }

  // SSE connection for real-time metrics
  createSSEConnection(endpoint: string) {
    const sseUrl = endpoint.startsWith('/') 
      ? `${window.location.origin}${endpoint}`
      : endpoint;
    
    return new EventSource(sseUrl);
  }

  // Update configuration
  updateConfig(config: Partial<{
    baseUrl: string;
    timeout: number;
    maxRetries: number;
    retryDelay: number;
    authToken: string;
  }>) {
    this.baseClient.updateConfig(config);
  }
}

// Export singleton instance
export const apiClient = new UnifiedApiClient();

// Export individual modules for tree-shaking
export { MetricsModule } from './modules/metrics';
export { TasksModule } from './modules/tasks';
export { AlertsModule } from './modules/alerts';
export { DatabaseModule } from './modules/database';
