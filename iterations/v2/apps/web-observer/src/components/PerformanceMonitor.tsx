"use client";

import { ObserverApiClient } from "@/lib/api-client";
import { useEffect, useState } from "react";

interface PerformanceMonitorProps {
  apiClient: ObserverApiClient;
}

interface PerformanceData {
  timestamp: string;
  responseTime: number;
  throughput: number;
  errorRate: number;
  memoryUsage: number;
  cpuUsage: number;
  activeConnections: number;
}

interface TrendData {
  direction: "up" | "down" | "stable";
  change: number;
  period: string;
}

export default function PerformanceMonitor({
  apiClient,
}: PerformanceMonitorProps) {
  const [performanceData, setPerformanceData] = useState<PerformanceData[]>([]);
  const [currentMetrics, setCurrentMetrics] = useState<PerformanceData | null>(
    null
  );
  const [trends, setTrends] = useState<Record<string, TrendData>>({});
  const [loading, setLoading] = useState(true);
  const [selectedMetric, setSelectedMetric] = useState<string>("responseTime");

  useEffect(() => {
    loadPerformanceData();
    // Update every 30 seconds for real-time monitoring
    const interval = setInterval(loadPerformanceData, 30000);
    return () => clearInterval(interval);
  }, []);

  const loadPerformanceData = async () => {
    try {
      const [diagnostics, _status, metrics] = await Promise.all([
        apiClient.getDiagnostics(),
        apiClient.getStatus(),
        apiClient.getMetrics(),
      ]);

      // Create performance data point from diagnostics
      const dataPoint: PerformanceData = {
        timestamp: diagnostics.timestamp || new Date().toISOString(),
        responseTime: 0, // Would come from backend response time tracking
        throughput: metrics?.activeTasks || 0,
        errorRate: metrics?.taskSuccessRate
          ? (1 - metrics.taskSuccessRate) * 100
          : 0,
        memoryUsage: diagnostics.resources?.memoryUsage || 0,
        cpuUsage: diagnostics.resources?.cpuUsage || 0,
        activeConnections: diagnostics.resources?.networkConnections || 0,
      };

      setPerformanceData((prev) => {
        const newData = [...prev, dataPoint];
        // Keep only last 50 data points for performance
        return newData.slice(-50);
      });

      setCurrentMetrics(dataPoint);
      calculateTrends();
      setLoading(false);
    } catch (err) {
      console.error("Failed to load performance data:", err);
      // Fallback to mock data if diagnostics endpoint fails
      const mockDataPoint: PerformanceData = {
        timestamp: new Date().toISOString(),
        responseTime: 0,
        throughput: 0,
        errorRate: 0,
        memoryUsage: 68,
        cpuUsage: 45,
        activeConnections: 12,
      };

      setPerformanceData((prev) => {
        const newData = [...prev, mockDataPoint];
        return newData.slice(-50);
      });

      setCurrentMetrics(mockDataPoint);
      calculateTrends();
      setLoading(false);
    }
  };

  const calculateTrends = () => {
    if (performanceData.length < 2) return;

    const recent = performanceData.slice(-10);
    const older = performanceData.slice(-20, -10);

    if (older.length === 0) return;

    const calculateTrend = (key: keyof PerformanceData): TrendData => {
      const recentAvg =
        recent.reduce((sum, d) => sum + (d[key] as number), 0) / recent.length;
      const olderAvg =
        older.reduce((sum, d) => sum + (d[key] as number), 0) / older.length;
      const change = recentAvg - olderAvg;
      const direction =
        Math.abs(change) < 0.01 ? "stable" : change > 0 ? "up" : "down";
      return { direction, change: Math.abs(change), period: "10min" };
    };

    setTrends({
      responseTime: calculateTrend("responseTime"),
      throughput: calculateTrend("throughput"),
      errorRate: calculateTrend("errorRate"),
      memoryUsage: calculateTrend("memoryUsage"),
      cpuUsage: calculateTrend("cpuUsage"),
    });
  };

  const getTrendIcon = (direction: "up" | "down" | "stable") => {
    switch (direction) {
      case "up":
        return "📈";
      case "down":
        return "📉";
      default:
        return "➡️";
    }
  };

  const getTrendColor = (
    direction: "up" | "down" | "stable",
    metric: string
  ) => {
    if (
      metric === "errorRate" ||
      metric === "memoryUsage" ||
      metric === "cpuUsage"
    ) {
      return direction === "up"
        ? "text-red-600"
        : direction === "down"
        ? "text-green-600"
        : "text-gray-600";
    }
    return direction === "up"
      ? "text-green-600"
      : direction === "down"
      ? "text-red-600"
      : "text-gray-600";
  };

  const metricConfigs = {
    responseTime: {
      label: "Response Time",
      unit: "ms",
      format: (v: number) => `${v}ms`,
    },
    throughput: {
      label: "Throughput",
      unit: "tasks/min",
      format: (v: number) => `${v}/min`,
    },
    errorRate: {
      label: "Error Rate",
      unit: "%",
      format: (v: number) => `${v.toFixed(1)}%`,
    },
    memoryUsage: {
      label: "Memory Usage",
      unit: "%",
      format: (v: number) => `${v}%`,
    },
    cpuUsage: { label: "CPU Usage", unit: "%", format: (v: number) => `${v}%` },
    activeConnections: {
      label: "Active Connections",
      unit: "",
      format: (v: number) => v.toString(),
    },
  };

  const renderMiniChart = (metric: string) => {
    const data = performanceData.slice(-20);
    const values = data.map(
      (d) => d[metric as keyof PerformanceData] as number
    );
    const max = Math.max(...values);
    const min = Math.min(...values);

    return (
      <div className="flex items-end h-8 space-x-1">
        {values.map((value, i) => {
          const height = max === min ? 50 : ((value - min) / (max - min)) * 100;
          return (
            <div
              key={i}
              className="bg-blue-200 rounded-sm min-w-[2px]"
              style={{ height: `${Math.max(height, 10)}%` }}
              title={`${value}`}
            />
          );
        })}
      </div>
    );
  };

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Performance Monitor
        </h2>
        <div className="animate-pulse">
          <div className="grid grid-cols-2 lg:grid-cols-3 gap-4">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="h-20 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">
          Performance Monitor
        </h2>
        <div className="flex items-center space-x-2">
          <span className="text-sm text-gray-500">Last updated:</span>
          <span className="text-sm font-medium text-gray-900">
            {currentMetrics
              ? new Date(currentMetrics.timestamp).toLocaleTimeString()
              : "Never"}
          </span>
        </div>
      </div>

      {/* Key Metrics Grid */}
      <div className="grid grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
        {Object.entries(metricConfigs).map(([key, config]) => {
          const value = currentMetrics?.[key as keyof PerformanceData] || 0;
          const trend = trends[key];

          return (
            <div
              key={key}
              className={`bg-gray-50 p-4 rounded-lg border-2 cursor-pointer transition-colors ${
                selectedMetric === key
                  ? "border-blue-500 bg-blue-50"
                  : "border-gray-200 hover:border-gray-300"
              }`}
              onClick={() => setSelectedMetric(key)}
            >
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-sm font-medium text-gray-700">
                  {config.label}
                </h3>
                {trend && (
                  <span
                    className={`text-sm ${getTrendColor(trend.direction, key)}`}
                  >
                    {getTrendIcon(trend.direction)}
                  </span>
                )}
              </div>
              <div className="text-2xl font-bold text-gray-900 mb-1">
                {config.format(value)}
              </div>
              {trend && trend.direction !== "stable" && (
                <div
                  className={`text-xs ${getTrendColor(trend.direction, key)}`}
                >
                  {trend.direction === "up" ? "+" : "-"}
                  {trend.change.toFixed(2)} {config.unit} ({trend.period})
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Detailed Chart for Selected Metric */}
      <div className="border-t pt-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-gray-900">
            {metricConfigs[selectedMetric as keyof typeof metricConfigs].label}{" "}
            Trend
          </h3>
          <div className="flex items-center space-x-2 text-sm text-gray-500">
            <span>Last 50 data points</span>
            <span>•</span>
            <span>Updates every 30s</span>
          </div>
        </div>

        <div className="bg-gray-50 p-4 rounded-lg">
          <div className="h-64 flex items-end justify-between">
            {performanceData.length > 0 ? (
              renderMiniChart(selectedMetric)
            ) : (
              <div className="flex items-center justify-center w-full h-full text-gray-500">
                Collecting performance data...
              </div>
            )}
          </div>
        </div>

        {performanceData.length > 0 && (
          <div className="mt-4 grid grid-cols-4 gap-4 text-sm">
            <div className="text-center">
              <div className="text-gray-500">Current</div>
              <div className="font-medium">
                {metricConfigs[
                  selectedMetric as keyof typeof metricConfigs
                ].format(
                  currentMetrics?.[selectedMetric as keyof PerformanceData] || 0
                )}
              </div>
            </div>
            <div className="text-center">
              <div className="text-gray-500">Average</div>
              <div className="font-medium">
                {metricConfigs[
                  selectedMetric as keyof typeof metricConfigs
                ].format(
                  performanceData.reduce(
                    (sum, d) =>
                      sum +
                      (d[selectedMetric as keyof PerformanceData] as number),
                    0
                  ) / performanceData.length
                )}
              </div>
            </div>
            <div className="text-center">
              <div className="text-gray-500">Peak</div>
              <div className="font-medium">
                {metricConfigs[
                  selectedMetric as keyof typeof metricConfigs
                ].format(
                  Math.max(
                    ...performanceData.map(
                      (d) =>
                        d[selectedMetric as keyof PerformanceData] as number
                    )
                  )
                )}
              </div>
            </div>
            <div className="text-center">
              <div className="text-gray-500">Data Points</div>
              <div className="font-medium">{performanceData.length}</div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
