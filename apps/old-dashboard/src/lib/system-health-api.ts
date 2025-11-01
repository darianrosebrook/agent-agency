/**
 * System Health API Client
 * API client for system health monitoring and Grafana integration
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface GrafanaDashboard {
  id: string;
  uid: string;
  title: string;
  description?: string;
  tags: string[];
  folderId?: number;
  folderTitle?: string;
  url: string;
  panels: GrafanaPanel[];
  created: Date;
  updated: Date;
}

export interface GrafanaPanel {
  id: number;
  title: string;
  type: string;
  description?: string;
  targets: any[];
  options: any;
  fieldConfig: any;
  transformations?: any[];
  pluginVersion?: string;
}

export interface GrafanaAlert {
  id: number;
  dashboardId: number;
  dashboardUID: string;
  panelId: number;
  name: string;
  state: 'ok' | 'paused' | 'alerting' | 'pending' | 'no_value';
  severity: 'info' | 'warning' | 'error' | 'critical';
  message: string;
  value: string;
  newStateDate: Date;
  evalDate: Date;
  annotations: Record<string, string>;
  labels: Record<string, string>;
  dashboardSlug?: string;
  panelURL?: string;
}

export interface ComponentHealth {
  id: string;
  name: string;
  type: 'api' | 'database' | 'cache' | 'worker' | 'model' | 'inference' | 'monitoring' | 'other';
  status: 'healthy' | 'warning' | 'critical' | 'unknown';
  availability: number; // percentage 0-100
  responseTime: number; // milliseconds
  errorRate: number; // percentage 0-100
  lastCheck: Date;
  nextCheck: Date;
  dependencies: string[]; // component IDs
  metrics: ComponentMetric[];
  alerts: ComponentAlert[];
}

export interface ComponentMetric {
  name: string;
  value: number;
  unit: string;
  timestamp: Date;
  trend: 'up' | 'down' | 'stable';
  threshold?: {
    warning: number;
    critical: number;
  };
}

export interface ComponentAlert {
  id: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  timestamp: Date;
  acknowledged: boolean;
  resolved: boolean;
  source: string;
}

export interface SystemHealth {
  overallStatus: 'healthy' | 'warning' | 'critical' | 'unknown';
  overallScore: number; // 0-100
  components: ComponentHealth[];
  alerts: SystemAlert[];
  metrics: SystemMetrics;
  lastUpdated: Date;
}

export interface SystemAlert {
  id: string;
  title: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  status: 'active' | 'acknowledged' | 'resolved';
  source: 'grafana' | 'prometheus' | 'application' | 'infrastructure' | 'custom';
  componentId?: string;
  timestamp: Date;
  acknowledgedAt?: Date;
  acknowledgedBy?: string;
  resolvedAt?: Date;
  resolvedBy?: string;
  tags: string[];
  annotations: Record<string, any>;
}

export interface SystemMetrics {
  totalComponents: number;
  healthyComponents: number;
  warningComponents: number;
  criticalComponents: number;
  totalAlerts: number;
  activeAlerts: number;
  acknowledgedAlerts: number;
  resolvedAlerts: number;
  averageResponseTime: number;
  averageAvailability: number;
  uptime: number; // percentage
}

export interface HealthTrend {
  timestamp: Date;
  overallScore: number;
  componentScores: Record<string, number>;
  alertCount: number;
  responseTime: number;
  availability: number;
}

export interface DependencyMap {
  nodes: DependencyNode[];
  edges: DependencyEdge[];
}

export interface DependencyNode {
  id: string;
  name: string;
  type: ComponentHealth['type'];
  status: ComponentHealth['status'];
  position: { x: number; y: number };
}

export interface DependencyEdge {
  from: string;
  to: string;
  type: 'depends_on' | 'communicates_with' | 'monitors';
  status: 'healthy' | 'warning' | 'critical' | 'unknown';
}

export interface MetricsQuery {
  componentId?: string;
  metricName: string;
  startTime: Date;
  endTime: Date;
  interval?: string; // e.g., '1m', '5m', '1h'
  aggregation?: 'avg' | 'sum' | 'min' | 'max' | 'count';
}

export interface MetricsResponse {
  metricName: string;
  data: {
    timestamp: Date;
    value: number;
  }[];
  metadata: {
    unit: string;
    aggregation: string;
    interval: string;
  };
}

export interface CustomDashboard {
  id: string;
  name: string;
  description?: string;
  panels: CustomPanel[];
  layout: 'grid' | 'masonry' | 'flex';
  createdBy: string;
  createdAt: Date;
  updatedAt: Date;
  isPublic: boolean;
  tags: string[];
}

export interface CustomPanel {
  id: string;
  title: string;
  type: 'line' | 'bar' | 'gauge' | 'table' | 'heatmap' | 'pie';
  metrics: MetricsQuery[];
  position: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  config: {
    showLegend?: boolean;
    showGrid?: boolean;
    colors?: string[];
    thresholds?: Array<{
      value: number;
      color: string;
      label?: string;
    }>;
  };
}

export class SystemHealthApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/system-health') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Grafana integration endpoints
   */
  async getGrafanaDashboards(folderId?: number): Promise<GrafanaDashboard[]> {
    const params = folderId ? `?folderId=${folderId}` : '';
    const response = await this.apiClient.request<GrafanaDashboard[]>(
      `/grafana/dashboards${params}`
    );
    return response;
  }

  async getGrafanaDashboard(uid: string): Promise<GrafanaDashboard> {
    const response = await this.apiClient.request<GrafanaDashboard>(
      `/grafana/dashboards/${uid}`
    );
    return response;
  }

  async getGrafanaAlerts(): Promise<GrafanaAlert[]> {
    const response = await this.apiClient.request<GrafanaAlert[]>(
      '/grafana/alerts'
    );
    return response;
  }

  async embedGrafanaPanel(dashboardUid: string, panelId: number, options?: {
    width?: number;
    height?: number;
    theme?: 'light' | 'dark';
    from?: Date;
    to?: Date;
  }): Promise<{
    embedUrl: string;
    iframeHtml: string;
    refreshInterval: number;
  }> {
    const response = await this.apiClient.request<{
      embedUrl: string;
      iframeHtml: string;
      refreshInterval: number;
    }>('/grafana/panels/embed', {
      method: 'POST',
      body: JSON.stringify({ dashboardUid, panelId, options })
    });
    return response;
  }

  /**
   * System health endpoints
   */
  async getSystemHealth(): Promise<SystemHealth> {
    const response = await this.apiClient.request<SystemHealth>('/health');
    return response;
  }

  async getComponentHealth(componentId?: string): Promise<ComponentHealth[]> {
    const params = componentId ? `?componentId=${componentId}` : '';
    const response = await this.apiClient.request<ComponentHealth[]>(
      `/health/components${params}`
    );
    return response;
  }

  async getHealthTrends(
    startTime: Date,
    endTime: Date,
    interval: '1m' | '5m' | '15m' | '1h' | '6h' | '24h' = '15m'
  ): Promise<HealthTrend[]> {
    const params = new URLSearchParams({
      startTime: startTime.toISOString(),
      endTime: endTime.toISOString(),
      interval
    });

    const response = await this.apiClient.request<HealthTrend[]>(
      `/health/trends?${params}`
    );
    return response;
  }

  /**
   * Alert management endpoints
   */
  async getAlerts(
    status?: SystemAlert['status'][],
    severity?: SystemAlert['severity'][],
    source?: SystemAlert['source'][],
    limit: number = 50
  ): Promise<SystemAlert[]> {
    const params = new URLSearchParams({ limit: limit.toString() });

    if (status) params.append('status', status.join(','));
    if (severity) params.append('severity', severity.join(','));
    if (source) params.append('source', source.join(','));

    const response = await this.apiClient.request<SystemAlert[]>(
      `/alerts?${params}`
    );
    return response;
  }

  async acknowledgeAlert(alertId: string): Promise<void> {
    await this.apiClient.request<void>(`/alerts/${alertId}/acknowledge`, {
      method: 'POST'
    });
  }

  async resolveAlert(alertId: string, resolution?: string): Promise<void> {
    await this.apiClient.request<void>(`/alerts/${alertId}/resolve`, {
      method: 'POST',
      body: JSON.stringify({ resolution })
    });
  }

  async getAlertCorrelation(alertId: string): Promise<{
    alert: SystemAlert;
    correlatedAlerts: SystemAlert[];
    rootCause: string;
    recommendations: string[];
    impact: {
      affectedComponents: string[];
      affectedUsers: number;
      businessImpact: 'low' | 'medium' | 'high' | 'critical';
    };
  }> {
    const response = await this.apiClient.request<{
      alert: SystemAlert;
      correlatedAlerts: SystemAlert[];
      rootCause: string;
      recommendations: string[];
      impact: {
        affectedComponents: string[];
        affectedUsers: number;
        businessImpact: 'low' | 'medium' | 'high' | 'critical';
      };
    }>(`/alerts/${alertId}/correlation`);
    return response;
  }

  /**
   * Metrics endpoints
   */
  async queryMetrics(query: MetricsQuery): Promise<MetricsResponse> {
    const response = await this.apiClient.request<MetricsResponse>('/metrics/query', {
      method: 'POST',
      body: JSON.stringify(query)
    });
    return response;
  }

  async getAvailableMetrics(): Promise<{
    categories: {
      name: string;
      metrics: Array<{
        name: string;
        description: string;
        unit: string;
        componentTypes: string[];
      }>;
    }[];
  }> {
    const response = await this.apiClient.request<{
      categories: {
        name: string;
        metrics: Array<{
          name: string;
          description: string;
          unit: string;
          componentTypes: string[];
        }>;
      }[];
    }>('/metrics/available');
    return response;
  }

  /**
   * Dependency mapping endpoints
   */
  async getDependencyMap(): Promise<DependencyMap> {
    const response = await this.apiClient.request<DependencyMap>('/dependencies/map');
    return response;
  }

  async getComponentDependencies(componentId: string): Promise<{
    component: ComponentHealth;
    dependencies: ComponentHealth[];
    dependents: ComponentHealth[];
    impact: {
      failureImpact: 'low' | 'medium' | 'high' | 'critical';
      affectedServices: string[];
      recoveryTime: number; // minutes
    };
  }> {
    const response = await this.apiClient.request<{
      component: ComponentHealth;
      dependencies: ComponentHealth[];
      dependents: ComponentHealth[];
      impact: {
        failureImpact: 'low' | 'medium' | 'high' | 'critical';
        affectedServices: string[];
        recoveryTime: number; // minutes
      };
    }>(`/dependencies/components/${componentId}`);
    return response;
  }

  /**
   * Custom dashboard endpoints
   */
  async getCustomDashboards(): Promise<CustomDashboard[]> {
    const response = await this.apiClient.request<CustomDashboard[]>('/dashboards/custom');
    return response;
  }

  async getCustomDashboard(dashboardId: string): Promise<CustomDashboard> {
    const response = await this.apiClient.request<CustomDashboard>(
      `/dashboards/custom/${dashboardId}`
    );
    return response;
  }

  async createCustomDashboard(dashboard: Omit<CustomDashboard, 'id' | 'createdAt' | 'updatedAt'>): Promise<CustomDashboard> {
    const response = await this.apiClient.request<CustomDashboard>('/dashboards/custom', {
      method: 'POST',
      body: JSON.stringify(dashboard)
    });
    return response;
  }

  async updateCustomDashboard(dashboardId: string, updates: Partial<CustomDashboard>): Promise<CustomDashboard> {
    const response = await this.apiClient.request<CustomDashboard>(
      `/dashboards/custom/${dashboardId}`,
      {
        method: 'PATCH',
        body: JSON.stringify(updates)
      }
    );
    return response;
  }

  async deleteCustomDashboard(dashboardId: string): Promise<void> {
    await this.apiClient.request<void>(`/dashboards/custom/${dashboardId}`, {
      method: 'DELETE'
    });
  }

  /**
   * System control endpoints
   */
  async restartComponent(componentId: string, reason?: string): Promise<{
    success: boolean;
    message: string;
    estimatedDowntime: number; // seconds
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      estimatedDowntime: number;
    }>(`/control/components/${componentId}/restart`, {
      method: 'POST',
      body: JSON.stringify({ reason })
    });
    return response;
  }

  async scaleComponent(
    componentId: string,
    action: 'scale_up' | 'scale_down',
    instances: number,
    reason?: string
  ): Promise<{
    success: boolean;
    message: string;
    newInstanceCount: number;
    estimatedCompletion: number; // seconds
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      newInstanceCount: number;
      estimatedCompletion: number;
    }>(`/control/components/${componentId}/scale`, {
      method: 'POST',
      body: JSON.stringify({ action, instances, reason })
    });
    return response;
  }

  /**
   * Maintenance mode endpoints
   */
  async enableMaintenanceMode(
    componentIds: string[],
    reason: string,
    duration?: number // minutes
  ): Promise<{
    success: boolean;
    message: string;
    maintenanceId: string;
    affectedComponents: string[];
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      maintenanceId: string;
      affectedComponents: string[];
    }>('/maintenance/enable', {
      method: 'POST',
      body: JSON.stringify({ componentIds, reason, duration })
    });
    return response;
  }

  async disableMaintenanceMode(maintenanceId: string): Promise<{
    success: boolean;
    message: string;
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
    }>(`/maintenance/${maintenanceId}/disable`, {
      method: 'POST'
    });
    return response;
  }

  /**
   * Report generation endpoints
   */
  async generateHealthReport(
    period: '1h' | '24h' | '7d' | '30d' = '24h',
    format: 'json' | 'pdf' | 'html' = 'pdf',
    includeTrends: boolean = true,
    includeAlerts: boolean = true
  ): Promise<Blob> {
    const params = new URLSearchParams({
      period,
      format,
      includeTrends: includeTrends.toString(),
      includeAlerts: includeAlerts.toString()
    });

    const response = await fetch(`${this.apiClient['config'].baseUrl}/reports/health?${params}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${this.apiClient['config'].authToken}`
      }
    });

    if (!response.ok) {
      throw new Error(`Report generation failed: ${response.statusText}`);
    }

    return response.blob();
  }

  /**
   * Configuration endpoints
   */
  async updateHealthCheckConfig(
    componentId: string,
    config: {
      interval?: number; // seconds
      timeout?: number; // seconds
      thresholds?: {
        responseTimeWarning?: number;
        responseTimeCritical?: number;
        errorRateWarning?: number;
        errorRateCritical?: number;
        availabilityWarning?: number;
        availabilityCritical?: number;
      };
    }
  ): Promise<void> {
    await this.apiClient.request<void>(
      `/config/health-check/${componentId}`,
      {
        method: 'PATCH',
        body: JSON.stringify(config)
      }
    );
  }
}

// Export singleton instance
export const systemHealthApiClient = new SystemHealthApiClient();
