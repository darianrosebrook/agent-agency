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
        "/metrics/agents"
      );
      return response.agents;
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
      const response = await apiClient.request<{ agent: AgentPerformance }>(
        `/metrics/agents/${encodeURIComponent(agentId)}`
      );
      return response.agent;
    } catch (error) {
      console.error("Failed to get agent performance:", error);
      if (error instanceof Error && error.message.includes("404")) {
        throw new MetricsApiError(
          "agent_not_found",
          `Agent ${agentId} not found`,
          false
        );
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
      const response = await apiClient.request<GetCoordinationMetricsResponse>(
        "/metrics/coordination"
      );
      return response.metrics;
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
      const params = new URLSearchParams({
        time_range: timeRange,
      });

      const response = await apiClient.request<GetBusinessMetricsResponse>(
        `/metrics/business?${params}`
      );
      return response;
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
      const params = new URLSearchParams({
        limit: limit.toString(),
      });

      if (status) params.append("status", status);
      if (severity) params.append("severity", severity);

      const response = await apiClient.request<GetAlertsResponse>(
        `/alerts?${params}`
      );
      return response;
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
      await apiClient.request(
        `/alerts/${encodeURIComponent(alertId)}/acknowledge`,
        {
          method: "POST",
        }
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
      await apiClient.request(
        `/alerts/${encodeURIComponent(alertId)}/resolve`,
        {
          method: "POST",
        }
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
    return `${this.baseUrl}/metrics/stream`;
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
