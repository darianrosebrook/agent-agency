import { apiClient } from "@/lib/api-client";
import {
  SystemHealth,
  AgentPerformance,
  CoordinationMetrics,
  GetSystemHealthResponse,
  GetAgentPerformanceResponse,
  GetCoordinationMetricsResponse,
  GetBusinessMetricsResponse,
  GetAlertsResponse,
  MetricsError,
} from "@/types/metrics";

// Metrics API Error Class
export class MetricsApiError extends Error {
  constructor(
    public code: MetricsError["code"],
    message: string,
    public retryable: boolean = true
  ) {
    super(message);
    this.name = "MetricsApiError";
  }
}

// Metrics API Client
// Handles REST API calls for system metrics, health, and observability data

export class MetricsApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl ?? "/api/metrics";
  }

  // Get system health status
  async getSystemHealth(): Promise<SystemHealth> {
    try {
      // Use the health check API we implemented
      const response = await apiClient.request<any>("/health");

      // Transform the health response to match our SystemHealth interface
      return {
        status: response.status,
        timestamp: response.timestamp,
        version: response.dashboard?.version,
        uptime: response.dashboard?.uptime,
        components: {
          dashboard: {
            status: response.dashboard?.status || "unknown",
            version: response.dashboard?.version,
            uptime: response.dashboard?.uptime,
          },
          backend: {
            status: response.backend?.status || "unknown",
            url: response.backend?.url,
            response_time_ms: response.backend?.response_time_ms,
            error: response.backend?.error,
          },
        },
      };
    } catch (error) {
      console.error("Failed to get system health:", error);
      throw new MetricsApiError(
        "server_error",
        "Failed to retrieve system health",
        true
      );
    }
  }

  // Get agent performance metrics
  async getAgentPerformance(): Promise<AgentPerformance[]> {
    try {
      const response = await apiClient.request<GetAgentPerformanceResponse>(
        `${this.baseUrl}?metric_type=agent_performance`
      );
      return response.metrics.filter(
        (m: any) => m.type === "agent_performance"
      ) as AgentPerformance[];
    } catch (error) {
      console.error("Failed to get agent performance:", error);
      throw new MetricsApiError(
        "metrics_unavailable",
        "Failed to retrieve agent performance metrics",
        true
      );
    }
  }

  // Get specific agent performance
  async getAgentPerformanceById(agentId: string): Promise<AgentPerformance> {
    try {
      const response = await apiClient.request<any>(
        `${
          this.baseUrl
        }?metric_type=agent_performance&agent_id=${encodeURIComponent(agentId)}`
      );
      const agentMetrics = response.metrics.find(
        (m: any) => m.agent_id === agentId
      );
      if (!agentMetrics) {
        throw new MetricsApiError(
          "agent_not_found",
          `Agent ${agentId} not found`,
          false
        );
      }
      return agentMetrics as AgentPerformance;
    } catch (error) {
      console.error("Failed to get agent performance:", error);
      if (error instanceof MetricsApiError) {
        throw error;
      }
      throw new MetricsApiError(
        "server_error",
        "Failed to retrieve agent performance",
        true
      );
    }
  }

  // Get coordination metrics
  async getCoordinationMetrics(): Promise<CoordinationMetrics> {
    try {
      const response = await apiClient.request<any>(
        `${this.baseUrl}?metric_type=coordination`
      );
      const coordinationMetrics = response.metrics.find(
        (m: any) => m.type === "coordination"
      );
      return (
        coordinationMetrics || {
          total_agents: 0,
          active_coordinations: 0,
          coordination_efficiency: 0,
          average_response_time: 0,
          conflict_resolution_rate: 0,
          timestamp: new Date().toISOString(),
        }
      );
    } catch (error) {
      console.error("Failed to get coordination metrics:", error);
      throw new MetricsApiError(
        "metrics_unavailable",
        "Failed to retrieve coordination metrics",
        true
      );
    }
  }

  // Get business intelligence metrics
  async getBusinessMetrics(
    timeRange: "1h" | "6h" | "24h" | "7d" | "30d" = "24h"
  ): Promise<GetBusinessMetricsResponse> {
    try {
      // Calculate time range
      const endTime = new Date();
      const startTime = new Date();

      switch (timeRange) {
        case "1h":
          startTime.setHours(endTime.getHours() - 1);
          break;
        case "6h":
          startTime.setHours(endTime.getHours() - 6);
          break;
        case "24h":
          startTime.setHours(endTime.getHours() - 24);
          break;
        case "7d":
          startTime.setDate(endTime.getDate() - 7);
          break;
        case "30d":
          startTime.setDate(endTime.getDate() - 30);
          break;
      }

      const response = await apiClient.request<any>(
        `${
          this.baseUrl
        }?metric_type=business&start_time=${startTime.toISOString()}&end_time=${endTime.toISOString()}`
      );

      return {
        summary: response.summary || {},
        trends:
          response.metrics.filter((m: any) => m.type === "business_trend") ||
          [],
        alerts: response.alerts || [],
        time_range: timeRange,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error("Failed to get business metrics:", error);
      throw new MetricsApiError(
        "metrics_unavailable",
        "Failed to retrieve business metrics",
        true
      );
    }
  }

  // Get system alerts
  async getAlerts(
    status?: "active" | "acknowledged" | "resolved",
    severity?: "info" | "warning" | "error" | "critical",
    limit: number = 50
  ): Promise<GetAlertsResponse> {
    try {
      const response = await apiClient.request<any>(this.baseUrl);
      const alerts = response.alerts || [];

      // Filter alerts based on parameters
      let filteredAlerts = alerts;
      if (status) {
        filteredAlerts = filteredAlerts.filter(
          (alert: any) => alert.status === status
        );
      }
      if (severity) {
        filteredAlerts = filteredAlerts.filter(
          (alert: any) => alert.severity === severity
        );
      }

      return {
        alerts: filteredAlerts.slice(0, limit),
        total: filteredAlerts.length,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error("Failed to get alerts:", error);
      throw new MetricsApiError(
        "server_error",
        "Failed to retrieve alerts",
        true
      );
    }
  }

  // Acknowledge an alert
  async acknowledgeAlert(alertId: string): Promise<void> {
    try {
      // For now, this is a no-op since we're using the unified metrics endpoint
      // In a real implementation, this would call a specific alert management endpoint
      console.warn(
        `Alert acknowledgment for ${alertId} not implemented - requires V3 alert management API`
      );
    } catch (error) {
      console.error("Failed to acknowledge alert:", error);
      throw new MetricsApiError(
        "server_error",
        "Failed to acknowledge alert",
        true
      );
    }
  }

  // Resolve an alert
  async resolveAlert(alertId: string): Promise<void> {
    try {
      // For now, this is a no-op since we're using the unified metrics endpoint
      // In a real implementation, this would call a specific alert management endpoint
      console.warn(
        `Alert resolution for ${alertId} not implemented - requires V3 alert management API`
      );
    } catch (error) {
      console.error("Failed to resolve alert:", error);
      throw new MetricsApiError(
        "server_error",
        "Failed to resolve alert",
        true
      );
    }
  }

  // Get real-time metrics stream URL for SSE
  getMetricsStreamUrl(): string {
    return `${this.baseUrl.replace("/api/metrics", "/api/metrics/stream")}`;
  }

  // Get metrics aggregation endpoints
  async getMetricsAggregation(
    metricName: string,
    timeRange: "1h" | "6h" | "24h" | "7d" | "30d" = "24h",
    aggregation: "sum" | "avg" | "min" | "max" | "count" = "avg"
  ): Promise<any> {
    try {
      const params = new URLSearchParams({
        time_range: timeRange,
        aggregation,
      });

      const response = await apiClient.request(
        `/metrics/aggregate/${encodeURIComponent(metricName)}?${params}`
      );
      return response;
    } catch (error) {
      console.error("Failed to get metrics aggregation:", error);
      throw new MetricsApiError(
        "metrics_unavailable",
        "Failed to retrieve metrics aggregation",
        true
      );
    }
  }
}

// Default metrics API client instance
export const metricsApiClient = new MetricsApiClient();
