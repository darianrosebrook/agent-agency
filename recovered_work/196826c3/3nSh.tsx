"use client";

import React, { useState } from "react";
import SystemHealthOverview from "../monitoring/SystemHealthOverview";
import AgentPerformanceGrid from "../monitoring/AgentPerformanceGrid";
import CoordinationMetrics from "../monitoring/CoordinationMetrics";
import BusinessIntelligence from "../monitoring/BusinessIntelligence";
import RealTimeMetricsStream from "../monitoring/RealTimeMetricsStream";
import MetricTile from "../monitoring/MetricTile";
import styles from "./MetricsDashboard.module.scss";

export default function MetricsDashboard() {
  const [timeRange, setTimeRange] = useState<
    "1h" | "6h" | "24h" | "7d" | "30d"
  >("24h");

  // Real-time KPI state
  const [kpis, setKpis] = useState({
    systemHealth: {
      value: "98.5%",
      status: "success" as const,
      trend: "+0.2%",
    },
    activeAgents: { value: "12", status: "success" as const, trend: "+2" },
    taskThroughput: {
      value: "45/min",
      status: "warning" as const,
      trend: "-5%",
    },
    errorRate: { value: "0.8%", status: "success" as const, trend: "-0.1%" },
    avgResponseTime: {
      value: "245ms",
      status: "success" as const,
      trend: "-12ms",
    },
    coordinationEfficiency: {
      value: "92%",
      status: "success" as const,
      trend: "+1.5%",
    },
  });

  // Coordination metrics state for real-time updates
  const [coordinationMetrics, setCoordinationMetrics] = useState<any>(null);

  // Historical KPI data for trend calculation (store last 10 values)
  const [kpiHistory, setKpiHistory] = useState<Record<string, number[]>>({});

  // Calculate trend from historical data
  const calculateTrend = useCallback((key: string, currentValue: number): string => {
    const history = kpiHistory[key] || [];
    if (history.length < 2) {
      return "0%"; // Not enough data for trend
    }

    const recentValues = history.slice(-5); // Use last 5 values for trend
    const avgRecent = recentValues.reduce((sum, val) => sum + val, 0) / recentValues.length;
    const avgPrevious = history.slice(-10, -5).reduce((sum, val) => sum + val, 0) / Math.min(5, history.length - 5) || avgRecent;

    if (avgPrevious === 0) return "0%";

    const trendPercent = ((avgRecent - avgPrevious) / avgPrevious * 100);
    const sign = trendPercent >= 0 ? "+" : "";
    return `${sign}${trendPercent.toFixed(1)}%`;
  }, [kpiHistory]);

  // Handle real-time metrics updates
  const handleMetricsUpdate = useCallback((event: any) => {
    console.log("Real-time metrics update:", event);

    setKpis(prevKpis => {
      const newKpis = { ...prevKpis };

      switch (event.type) {
        case "system_metrics":
          // Handle unified system metrics from V3 backend
          const systemData = event.data;

          // System health from component statuses
          if (systemData?.system_health?.components) {
            const components = systemData.system_health.components;
            const healthyComponents = components.filter((c: any) => c.status === "healthy").length;
            const totalComponents = components.length;
            const healthPercent = totalComponents > 0 ? (healthyComponents / totalComponents * 100).toFixed(1) : "0.0";

            const healthScore = parseFloat(healthPercent);
            newKpis.systemHealth = {
              value: `${healthPercent}%`,
              status: healthScore > 90 ? "success" : healthScore > 70 ? "warning" : "error",
              trend: calculateTrend("systemHealth", healthScore),
            };
          }

          // Agent performance from system metrics
          if (systemData?.agent_performance && Array.isArray(systemData.agent_performance)) {
            const agents = systemData.agent_performance;
            const activeAgents = agents.filter((a: any) => a.status === "active" || a.active_tasks > 0);
            const activeCount = activeAgents.length;

            newKpis.activeAgents = {
              value: activeCount.toString(),
              status: activeCount > 0 ? "success" : "neutral",
              trend: calculateTrend("activeAgents", activeCount),
            };

            // Calculate average response time
            const avgResponseTime = activeAgents.length > 0
              ? activeAgents.reduce((sum: number, a: any) => sum + (a.average_response_time_ms || 0), 0) / activeAgents.length
              : 0;

            newKpis.avgResponseTime = {
              value: `${Math.round(avgResponseTime)}ms`,
              status: avgResponseTime < 1000 ? "success" :
                     avgResponseTime < 2000 ? "warning" : "error",
              trend: calculateTrend("avgResponseTime", avgResponseTime),
            };
          }

          // Coordination metrics
          if (systemData?.coordination_metrics) {
            const metrics = systemData.coordination_metrics;

            // Update task throughput
            if (metrics.tasks_per_minute !== undefined) {
              newKpis.taskThroughput = {
                value: `${metrics.tasks_per_minute}/min`,
                status: metrics.tasks_per_minute > 30 ? "success" :
                       metrics.tasks_per_minute > 10 ? "warning" : "error",
                trend: calculateTrend("taskThroughput", metrics.tasks_per_minute),
              };
            }

            // Update coordination efficiency
            if (metrics.efficiency_percentage !== undefined) {
              newKpis.coordinationEfficiency = {
                value: `${metrics.efficiency_percentage.toFixed(1)}%`,
                status: metrics.efficiency_percentage > 85 ? "success" :
                       metrics.efficiency_percentage > 70 ? "warning" : "error",
                trend: calculateTrend("coordinationEfficiency", metrics.efficiency_percentage),
              };
            }

            // Update coordination metrics state for the CoordinationMetrics component
            setCoordinationMetrics(prevMetrics => ({
              ...prevMetrics,
              ...metrics,
              timestamp: new Date().toISOString(),
            }));
          }

          // Business metrics (error rate)
          if (systemData?.business_metrics?.error_rate !== undefined) {
            const errorRate = systemData.business_metrics.error_rate;
            newKpis.errorRate = {
              value: `${(errorRate * 100).toFixed(1)}%`,
              status: errorRate < 0.01 ? "success" :
                     errorRate < 0.05 ? "warning" : "error",
              trend: calculateTrend("errorRate", errorRate),
            };
          }
          break;

        // Legacy event types for backward compatibility
        case "health_update":
        case "agent_performance":
        case "coordination_update":
        case "business_metrics":
          // Handle legacy event formats if needed
          console.warn(`Received legacy event type: ${event.type}, consider updating to 'system_metrics'`);
          break;
      }

      return newKpis;
    });

    // Update historical data for trend calculations
    setKpiHistory(prevHistory => {
      const newHistory = { ...prevHistory };

      // Update history for each KPI that was modified
      if (event.type === "system_metrics") {
        const systemData = event.data;

        // System health history
        if (systemData?.system_health?.components) {
          const components = systemData.system_health.components;
          const healthyComponents = components.filter((c: any) => c.status === "healthy").length;
          const totalComponents = components.length;
          const healthScore = totalComponents > 0 ? (healthyComponents / totalComponents * 100) : 0;
          newHistory.systemHealth = [...(newHistory.systemHealth || []), healthScore].slice(-10);
        }

        // Agent performance history
        if (systemData?.agent_performance && Array.isArray(systemData.agent_performance)) {
          const agents = systemData.agent_performance;
          const activeCount = agents.filter((a: any) => a.status === "active" || a.active_tasks > 0).length;
          newHistory.activeAgents = [...(newHistory.activeAgents || []), activeCount].slice(-10);

          const activeAgents = agents.filter((a: any) => a.status === "active" || a.active_tasks > 0);
          const avgResponseTime = activeAgents.length > 0
            ? activeAgents.reduce((sum: number, a: any) => sum + (a.average_response_time_ms || 0), 0) / activeAgents.length
            : 0;
          newHistory.avgResponseTime = [...(newHistory.avgResponseTime || []), avgResponseTime].slice(-10);
        }

        // Coordination metrics history
        if (systemData?.coordination_metrics) {
          const metrics = systemData.coordination_metrics;
          if (metrics.tasks_per_minute !== undefined) {
            newHistory.taskThroughput = [...(newHistory.taskThroughput || []), metrics.tasks_per_minute].slice(-10);
          }
          if (metrics.efficiency_percentage !== undefined) {
            newHistory.coordinationEfficiency = [...(newHistory.coordinationEfficiency || []), metrics.efficiency_percentage].slice(-10);
          }
        }

        // Business metrics history
        if (systemData?.business_metrics?.error_rate !== undefined) {
          newHistory.errorRate = [...(newHistory.errorRate || []), systemData.business_metrics.error_rate].slice(-10);
        }
      }

      return newHistory;
    });
  }, [calculateTrend]);

  return (
    <div className={styles.metricsDashboard}>
      <div className={styles.header}>
        <h1>Metrics & Observability</h1>
        <p className={styles.description}>
          Comprehensive view of system performance, agent activity, coordination
          patterns, and business intelligence metrics.
        </p>
      </div>

      {/* Key Performance Indicators Row */}
      <div className={styles.kpiSection}>
        <h2>Key Performance Indicators</h2>
        <div className={styles.kpiGrid}>
          <MetricTile
            title="System Health"
            value={kpis.systemHealth.value}
            status={kpis.systemHealth.status}
            icon="🏥"
            trend="up"
          />
          <MetricTile
            title="Active Agents"
            value={kpis.activeAgents.value}
            status={kpis.activeAgents.status}
            icon="🤖"
            trend="up"
          />
          <MetricTile
            title="Task Throughput"
            value={kpis.taskThroughput.value}
            status={kpis.taskThroughput.status}
            icon="⚙️"
            trend="stable"
          />
          <MetricTile
            title="Avg Response Time"
            value={kpis.avgResponseTime.value}
            status={kpis.avgResponseTime.status}
            icon="⚡"
            format="duration"
            trend="down"
          />
          <MetricTile
            title="Error Rate"
            value={kpis.errorRate.value}
            status={kpis.errorRate.status}
            icon="🚨"
            trend="down"
          />
          <MetricTile
            title="Coordination Efficiency"
            value={kpis.coordinationEfficiency.value}
            status={kpis.coordinationEfficiency.status}
            icon="🤝"
            trend="up"
          />
        </div>
      </div>

      {/* System Health Section */}
      <div className={styles.section}>
        <SystemHealthOverview
          onRetry={() => {
            console.log("Retrying system health check...");
          }}
        />
      </div>

      {/* Agent Performance Section */}
      <div className={styles.section}>
        <AgentPerformanceGrid
          onAgentSelect={(agentId) => {
            console.log("Selected agent:", agentId);
          }}
        />
      </div>

      {/* Coordination Metrics Section */}
      <div className={styles.section}>
        <CoordinationMetrics
          metrics={coordinationMetrics}
        />
      </div>

      {/* Business Intelligence Section */}
      <div className={styles.section}>
        <BusinessIntelligence
          timeRange={timeRange}
          onTimeRangeChange={(range) => {
            const validRange = range as "1h" | "6h" | "24h" | "7d" | "30d";
            setTimeRange(validRange);
          }}
        />
      </div>

      {/* Real-time Metrics Stream */}
      <RealTimeMetricsStream
        onMetricsUpdate={handleMetricsUpdate}
        onError={(error) => {
          console.error("Real-time metrics stream error:", error);
        }}
        enabled={true}
      />

      {/* Footer with refresh and export options */}
      <div className={styles.footer}>
        <div className={styles.footerActions}>
          <button
            className={styles.secondaryButton}
            onClick={() => {
              console.log("Refreshing all metrics...");
              // Mock refresh - update timestamps and slight variations
              if (typeof window !== "undefined") {
                // eslint-disable-next-line no-undef
                window.location.reload();
              }
            }}
          >
            🔄 Refresh All
          </button>
          <button
            className={styles.secondaryButton}
            onClick={() => {
              console.log("Exporting metrics...");
              // Mock export - create downloadable JSON
              const exportData = {
                timestamp: new Date().toISOString(),
                kpis: mockKPIs,
                timeRange,
                exportedAt: new Date().toISOString(),
              };

              const blob = new Blob([JSON.stringify(exportData, null, 2)], {
                type: "application/json",
              });
              const url = URL.createObjectURL(blob);
              if (typeof document !== "undefined") {
                // eslint-disable-next-line no-undef
                const a = document.createElement("a");
                a.href = url;
                a.download = `metrics-export-${
                  new Date().toISOString().split("T")[0]
                }.json`;
                // eslint-disable-next-line no-undef
                document.body.appendChild(a);
                a.click();
                // eslint-disable-next-line no-undef
                document.body.removeChild(a);
              }
              URL.revokeObjectURL(url);
            }}
          >
            📥 Export Data
          </button>
        </div>
        <div className={styles.lastUpdated}>
          <span>Last updated: {new Date().toLocaleTimeString()}</span>
        </div>
      </div>
    </div>
  );
}
