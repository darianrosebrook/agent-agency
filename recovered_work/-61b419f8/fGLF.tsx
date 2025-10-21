"use client";

import { ObserverApiClient } from "@/lib/api-client";
import { useEffect, useState } from "react";

interface DebugPanelProps {
  apiClient: ObserverApiClient;
}

interface DebugInfo {
  systemHealth: {
    registryTimeout: boolean;
    taskProcessingIssues: string[];
    performanceIssues: string[];
    connectionIssues: string[];
    agentHealthIssues: string[];
    resourceIssues: string[];
  };
  recentErrors: Array<{
    timestamp: string;
    type: string;
    message: string;
    severity: "low" | "medium" | "high";
    taskId?: string;
    agentId?: string;
  }>;
  performanceMetrics: {
    avgResponseTime: number;
    errorRate: number;
    queueDepth: number;
    activeConnections: number;
    taskSuccessRate: number;
    agentUtilization: number;
  };
  agentHealth: Array<{
    agentId: string;
    status: "healthy" | "degraded" | "unhealthy";
    lastActive: string;
    currentLoad: number;
    successRate: number;
  }>;
  systemResources: {
    memoryUsage: number;
    cpuUsage: number;
    diskUsage: number;
    networkConnections: number;
  };
  taskExecutionStats: {
    totalTasks: number;
    completedTasks: number;
    failedTasks: number;
    averageExecutionTime: number;
  };
}

export default function DebugPanel({ apiClient }: DebugPanelProps) {
  const [debugInfo, setDebugInfo] = useState<DebugInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    loadDebugInfo();
    // Refresh debug info every 10 seconds
    const interval = setInterval(loadDebugInfo, 10000);
    return () => clearInterval(interval);
  }, []);

  const loadDebugInfo = async () => {
    try {
      // Get system status and metrics
      const [status, metrics, events] = await Promise.all([
        apiClient.getStatus(),
        apiClient.getMetrics(),
        apiClient.getEvents({ limit: 20, severity: "error" }),
      ]);

      // Analyze recent events for debugging insights
      const recentErrors = events.events
        .filter(
          (event) => event.severity === "error" || event.severity === "warn"
        )
        .slice(0, 5)
        .map((event) => ({
          timestamp: event.timestamp,
          type: event.type,
          message: (event.metadata?.message as string) || "No message",
          severity:
            event.severity === "error"
              ? "high"
              : ("medium" as "low" | "medium" | "high"),
        }));

      // Identify system health issues based on patterns
      const systemHealth = {
        registryTimeout: events.events.some((e) =>
          (e.metadata?.message as string)?.includes("timeout")
        ),
        taskProcessingIssues: events.events
          .filter((e) => e.type.includes("task") && e.severity === "error")
          .map((e) => (e.metadata?.message as string) || e.type),
        performanceIssues: events.events
          .filter(
            (e) =>
              (e.metadata?.message as string)?.includes("slow") ||
              (e.metadata?.message as string)?.includes("timeout")
          )
          .map((e) => (e.metadata?.message as string) || e.type),
        connectionIssues: events.events
          .filter(
            (e) =>
              (e.metadata?.message as string)?.includes("connection") ||
              (e.metadata?.message as string)?.includes("disconnect")
          )
          .map((e) => (e.metadata?.message as string) || e.type),
      };

      const performanceMetrics = {
        avgResponseTime: 0, // Placeholder - would need actual response time tracking
        errorRate: metrics?.taskSuccessRate
          ? (1 - metrics.taskSuccessRate) * 100
          : 0,
        queueDepth: status?.queueDepth || 0,
        activeConnections: metrics?.activeTasks || 0,
      };

      setDebugInfo({
        systemHealth,
        recentErrors,
        performanceMetrics,
      });
      setLoading(false);
    } catch (err) {
      console.error("Failed to load debug info:", err);
      setLoading(false);
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "high":
        return "text-red-600 bg-red-100";
      case "medium":
        return "text-yellow-600 bg-yellow-100";
      case "low":
        return "text-blue-600 bg-blue-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  if (loading) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <div className="flex items-center">
          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-yellow-600 mr-2"></div>
          <span className="text-yellow-800 text-sm">
            Loading debug information...
          </span>
        </div>
      </div>
    );
  }

  if (!debugInfo) {
    return null;
  }

  const hasIssues =
    debugInfo.systemHealth.registryTimeout ||
    debugInfo.systemHealth.taskProcessingIssues.length > 0 ||
    debugInfo.systemHealth.performanceIssues.length > 0 ||
    debugInfo.systemHealth.connectionIssues.length > 0 ||
    debugInfo.recentErrors.length > 0;

  return (
    <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          <svg
            className="h-5 w-5 text-yellow-600 mr-2"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 19.5c-.77.833.192 2.5 1.732 2.5z"
            />
          </svg>
          <h3 className="text-yellow-800 font-medium">
            Debug Panel {hasIssues ? "⚠️ Issues Detected" : "✅ All Good"}
          </h3>
        </div>
        <button
          onClick={() => setExpanded(!expanded)}
          className="text-yellow-600 hover:text-yellow-800 text-sm font-medium"
        >
          {expanded ? "Hide Details" : "Show Details"}
        </button>
      </div>

      {expanded && (
        <div className="mt-4 space-y-4">
          {/* System Health Issues */}
          {(debugInfo.systemHealth.registryTimeout ||
            debugInfo.systemHealth.taskProcessingIssues.length > 0 ||
            debugInfo.systemHealth.performanceIssues.length > 0 ||
            debugInfo.systemHealth.connectionIssues.length > 0) && (
            <div>
              <h4 className="text-yellow-800 font-medium mb-2">
                System Health Issues
              </h4>
              <div className="space-y-2">
                {debugInfo.systemHealth.registryTimeout && (
                  <div className="text-sm text-yellow-700 bg-yellow-100 p-2 rounded">
                    ⚠️ Registry timeout detected - proceeding anyway
                  </div>
                )}
                {debugInfo.systemHealth.taskProcessingIssues.map((issue, i) => (
                  <div
                    key={i}
                    className="text-sm text-red-700 bg-red-100 p-2 rounded"
                  >
                    🚫 Task Processing: {issue}
                  </div>
                ))}
                {debugInfo.systemHealth.performanceIssues.map((issue, i) => (
                  <div
                    key={i}
                    className="text-sm text-orange-700 bg-orange-100 p-2 rounded"
                  >
                    🐌 Performance: {issue}
                  </div>
                ))}
                {debugInfo.systemHealth.connectionIssues.map((issue, i) => (
                  <div
                    key={i}
                    className="text-sm text-blue-700 bg-blue-100 p-2 rounded"
                  >
                    🔌 Connection: {issue}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Errors */}
          {debugInfo.recentErrors.length > 0 && (
            <div>
              <h4 className="text-yellow-800 font-medium mb-2">
                Recent Errors & Warnings
              </h4>
              <div className="space-y-2">
                {debugInfo.recentErrors.map((error, i) => (
                  <div key={i} className="text-sm bg-white p-2 rounded border">
                    <div className="flex items-center justify-between mb-1">
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${getSeverityColor(
                          error.severity
                        )}`}
                      >
                        {error.severity.toUpperCase()}
                      </span>
                      <span className="text-gray-500 text-xs">
                        {error.timestamp}
                      </span>
                    </div>
                    <div className="text-gray-700">
                      <strong>{error.type}:</strong> {error.message}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Performance Metrics */}
          <div>
            <h4 className="text-yellow-800 font-medium mb-2">
              Performance Metrics
            </h4>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="bg-white p-2 rounded border">
                <div className="text-gray-600">Avg Response Time</div>
                <div className="font-medium">
                  {debugInfo.performanceMetrics.avgResponseTime}ms
                </div>
              </div>
              <div className="bg-white p-2 rounded border">
                <div className="text-gray-600">Error Rate</div>
                <div className="font-medium">
                  {debugInfo.performanceMetrics.errorRate.toFixed(1)}%
                </div>
              </div>
              <div className="bg-white p-2 rounded border">
                <div className="text-gray-600">Queue Depth</div>
                <div className="font-medium">
                  {debugInfo.performanceMetrics.queueDepth}
                </div>
              </div>
              <div className="bg-white p-2 rounded border">
                <div className="text-gray-600">Active Connections</div>
                <div className="font-medium">
                  {debugInfo.performanceMetrics.activeConnections}
                </div>
              </div>
            </div>
          </div>

          {/* Debugging Suggestions */}
          {hasIssues && (
            <div>
              <h4 className="text-yellow-800 font-medium mb-2">
                Debugging Suggestions
              </h4>
              <div className="space-y-1 text-sm text-yellow-700">
                {debugInfo.systemHealth.taskProcessingIssues.length > 0 && (
                  <div>• Check task intake validation and field mapping</div>
                )}
                {debugInfo.systemHealth.registryTimeout && (
                  <div>• Monitor agent registry initialization timing</div>
                )}
                {debugInfo.performanceMetrics.errorRate > 5 && (
                  <div>
                    • High error rate detected - review recent task failures
                  </div>
                )}
                {debugInfo.performanceMetrics.queueDepth > 10 && (
                  <div>
                    • Queue depth is high - consider scaling or optimization
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
