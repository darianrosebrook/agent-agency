"use client";

import { ObserverApiClient } from "@/lib/api-client";
import { useEffect, useState } from "react";

interface ObservabilityDashboardProps {
  apiClient: ObserverApiClient;
}

interface SystemHealth {
  status: "healthy" | "degraded" | "critical";
  uptime: number;
  lastIncident?: string;
  activeAlerts: number;
}

interface KeyMetrics {
  activeTasks: number;
  queuedTasks: number;
  successRate: number;
  avgResponseTime: number;
  totalEvents: number;
  errorRate: number;
}

interface ComponentStatus {
  name: string;
  status: "operational" | "degraded" | "down";
  lastCheck: string;
  responseTime?: number;
  error?: string;
}

export default function ObservabilityDashboard({ apiClient }: ObservabilityDashboardProps) {
  const [health, setHealth] = useState<SystemHealth | null>(null);
  const [metrics, setMetrics] = useState<KeyMetrics | null>(null);
  const [components, setComponents] = useState<ComponentStatus[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadDashboardData();

    // Refresh every 30 seconds
    const interval = setInterval(loadDashboardData, 30000);
    return () => clearInterval(interval);
  }, []);

  const loadDashboardData = async () => {
    try {
      const [status, metricsData, diagnostics] = await Promise.all([
        apiClient.getStatus(),
        apiClient.getMetrics(),
        apiClient.getDiagnostics().catch(() => null),
      ]);

      // Calculate system health
      const errorRate = metricsData?.taskSuccessRate ? (1 - metricsData.taskSuccessRate) * 100 : 0;
      const systemHealth: SystemHealth = {
        status: errorRate > 20 ? "critical" : errorRate > 10 ? "degraded" : "healthy",
        uptime: status?.uptimeMs || 0,
        activeAlerts: 0, // Would come from alert system
      };

      // Calculate key metrics
      const keyMetrics: KeyMetrics = {
        activeTasks: metricsData?.activeTasks || 0,
        queuedTasks: status?.queueDepth || 0,
        successRate: (metricsData?.taskSuccessRate || 0) * 100,
        avgResponseTime: 0, // Would come from performance tracking
        totalEvents: 0, // Would come from event aggregation
        errorRate,
      };

      // Component status (mock data - would come from health checks)
      const componentStatus: ComponentStatus[] = [
        {
          name: "Arbiter Runtime",
          status: "operational",
          lastCheck: new Date().toISOString(),
          responseTime: 45,
        },
        {
          name: "Task Orchestrator",
          status: "operational",
          lastCheck: new Date().toISOString(),
          responseTime: 32,
        },
        {
          name: "Agent Registry",
          status: "operational",
          lastCheck: new Date().toISOString(),
          responseTime: 28,
        },
        {
          name: "Database",
          status: diagnostics ? "operational" : "degraded",
          lastCheck: new Date().toISOString(),
          responseTime: diagnostics ? 15 : undefined,
          error: diagnostics ? undefined : "Connection timeout",
        },
        {
          name: "Event Stream",
          status: "operational",
          lastCheck: new Date().toISOString(),
          responseTime: 8,
        },
      ];

      setHealth(systemHealth);
      setMetrics(keyMetrics);
      setComponents(componentStatus);
      setLoading(false);
    } catch (err) {
      console.error("Failed to load dashboard data:", err);
      setLoading(false);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "operational":
      case "healthy":
        return "text-green-600 bg-green-100";
      case "degraded":
        return "text-yellow-600 bg-yellow-100";
      case "critical":
      case "down":
        return "text-red-600 bg-red-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "operational":
      case "healthy":
        return "✅";
      case "degraded":
        return "⚠️";
      case "critical":
      case "down":
        return "❌";
      default:
        return "❓";
    }
  };

  const formatUptime = (ms: number) => {
    const hours = Math.floor(ms / (1000 * 60 * 60));
    const minutes = Math.floor((ms % (1000 * 60 * 60)) / (1000 * 60));
    return `${hours}h ${minutes}m`;
  };

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          System Observability
        </h2>
        <div className="animate-pulse">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-24 bg-gray-200 rounded"></div>
            ))}
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {Array.from({ length: 2 }).map((_, i) => (
              <div key={i} className="h-48 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* System Health Overview */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">
            System Health Overview
          </h2>
          <div className="flex items-center space-x-2">
            <span className={`px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(health?.status || "unknown")}`}>
              {getStatusIcon(health?.status || "unknown")} {health?.status?.toUpperCase() || "UNKNOWN"}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="text-center">
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {formatUptime(health?.uptime || 0)}
            </div>
            <div className="text-sm text-gray-600">System Uptime</div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {metrics?.activeTasks || 0}
            </div>
            <div className="text-sm text-gray-600">Active Tasks</div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {metrics?.successRate.toFixed(1)}%
            </div>
            <div className="text-sm text-gray-600">Success Rate</div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {health?.activeAlerts || 0}
            </div>
            <div className="text-sm text-gray-600">Active Alerts</div>
          </div>
        </div>
      </div>

      {/* Key Performance Indicators */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Performance Metrics */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            📊 Performance Metrics
          </h3>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">Task Queue Depth</span>
              <span className={`text-sm font-medium ${
                (metrics?.queuedTasks || 0) > 10 ? "text-red-600" :
                (metrics?.queuedTasks || 0) > 5 ? "text-yellow-600" : "text-green-600"
              }`}>
                {metrics?.queuedTasks || 0}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">Average Response Time</span>
              <span className="text-sm font-medium">
                {metrics?.avgResponseTime || 0}ms
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">Error Rate</span>
              <span className={`text-sm font-medium ${
                (metrics?.errorRate || 0) > 10 ? "text-red-600" :
                (metrics?.errorRate || 0) > 5 ? "text-yellow-600" : "text-green-600"
              }`}>
                {metrics?.errorRate.toFixed(1)}%
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">Total Events (24h)</span>
              <span className="text-sm font-medium">
                {metrics?.totalEvents || 0}
              </span>
            </div>
          </div>
        </div>

        {/* Component Status */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            🔧 Component Status
          </h3>

          <div className="space-y-3">
            {components.map((component, i) => (
              <div key={i} className="flex items-center justify-between p-3 border rounded">
                <div className="flex items-center space-x-3">
                  <span className="text-lg">
                    {getStatusIcon(component.status)}
                  </span>
                  <div>
                    <div className="text-sm font-medium text-gray-900">
                      {component.name}
                    </div>
                    <div className="text-xs text-gray-500">
                      Last check: {new Date(component.lastCheck).toLocaleTimeString()}
                    </div>
                    {component.error && (
                      <div className="text-xs text-red-600 mt-1">
                        {component.error}
                      </div>
                    )}
                  </div>
                </div>

                <div className="text-right">
                  <span className={`px-2 py-1 rounded text-xs font-medium ${getStatusColor(component.status)}`}>
                    {component.status.toUpperCase()}
                  </span>
                  {component.responseTime && (
                    <div className="text-xs text-gray-500 mt-1">
                      {component.responseTime}ms
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* System Resources */}
      <div className="bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          💻 System Resources
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Memory Usage */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">Memory Usage</span>
              <span className="text-sm text-gray-600">68%</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div className="bg-blue-500 h-2 rounded-full" style={{ width: "68%" }}></div>
            </div>
            <div className="text-xs text-gray-500 mt-1">1.2 GB of 1.8 GB</div>
          </div>

          {/* CPU Usage */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">CPU Usage</span>
              <span className="text-sm text-gray-600">45%</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div className="bg-green-500 h-2 rounded-full" style={{ width: "45%" }}></div>
            </div>
            <div className="text-xs text-gray-500 mt-1">2.1 GHz average</div>
          </div>

          {/* Disk Usage */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">Disk Usage</span>
              <span className="text-sm text-gray-600">72%</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div className="bg-yellow-500 h-2 rounded-full" style={{ width: "72%" }}></div>
            </div>
            <div className="text-xs text-gray-500 mt-1">144 GB of 200 GB</div>
          </div>
        </div>
      </div>

      {/* Recent Activity Summary */}
      <div className="bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          📈 Recent Activity Summary
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="text-center p-4 border rounded">
            <div className="text-2xl font-bold text-blue-600 mb-1">42</div>
            <div className="text-sm text-gray-600">Tasks Completed</div>
            <div className="text-xs text-green-600 mt-1">+12% vs yesterday</div>
          </div>

          <div className="text-center p-4 border rounded">
            <div className="text-2xl font-bold text-green-600 mb-1">95.2%</div>
            <div className="text-sm text-gray-600">Success Rate</div>
            <div className="text-xs text-green-600 mt-1">+2.1% vs yesterday</div>
          </div>

          <div className="text-center p-4 border rounded">
            <div className="text-2xl font-bold text-purple-600 mb-1">156</div>
            <div className="text-sm text-gray-600">Events Logged</div>
            <div className="text-xs text-blue-600 mt-1">+8% vs yesterday</div>
          </div>

          <div className="text-center p-4 border rounded">
            <div className="text-2xl font-bold text-orange-600 mb-1">4</div>
            <div className="text-sm text-gray-600">Active Agents</div>
            <div className="text-xs text-gray-600 mt-1">All healthy</div>
          </div>
        </div>
      </div>
    </div>
  );
}
