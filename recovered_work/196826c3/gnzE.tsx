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
        case "health_update":
          if (event.data?.system_health) {
            const health = event.data.system_health;
            // Calculate system health percentage from components
            const healthyComponents = health.components?.filter(
              (c: any) => c.status === "healthy"
            ).length || 0;
            const totalComponents = health.components?.length || 1;
            const healthPercent = (healthyComponents / totalComponents * 100).toFixed(1);

            newKpis.systemHealth = {
              value: `${healthPercent}%`,
              status: health.status === "healthy" ? "success" :
                     health.status === "degraded" ? "warning" : "error",
              trend: "+0.0%", // TODO: Calculate trend from historical data
            };
          }
          break;

        case "agent_performance":
          if (event.data?.agent_performance) {
            const agents = Array.isArray(event.data.agent_performance)
              ? event.data.agent_performance
              : [event.data.agent_performance];

            const activeCount = agents.filter((a: any) => a.status === "active").length;
            newKpis.activeAgents = {
              value: activeCount.toString(),
              status: activeCount > 0 ? "success" : "neutral",
              trend: "+0", // TODO: Calculate trend
            };

            // Calculate average response time
            const avgResponseTime = agents.length > 0
              ? agents.reduce((sum: number, a: any) => sum + (a.average_response_time_ms || 0), 0) / agents.length
              : 0;

            newKpis.avgResponseTime = {
              value: `${Math.round(avgResponseTime)}ms`,
              status: avgResponseTime < 1000 ? "success" :
                     avgResponseTime < 2000 ? "warning" : "error",
              trend: "0ms", // TODO: Calculate trend
            };
          }
          break;

        case "coordination_update":
          if (event.data?.coordination_metrics) {
            const metrics = event.data.coordination_metrics;

            // Update task throughput
            if (metrics.tasks_per_minute !== undefined) {
              newKpis.taskThroughput = {
                value: `${metrics.tasks_per_minute}/min`,
                status: metrics.tasks_per_minute > 30 ? "success" :
                       metrics.tasks_per_minute > 10 ? "warning" : "error",
                trend: "+0", // TODO: Calculate trend
              };
            }

            // Update coordination efficiency
            if (metrics.efficiency_percentage !== undefined) {
              newKpis.coordinationEfficiency = {
                value: `${metrics.efficiency_percentage}%`,
                status: metrics.efficiency_percentage > 85 ? "success" :
                       metrics.efficiency_percentage > 70 ? "warning" : "error",
                trend: "+0.0%", // TODO: Calculate trend
              };
            }
          }
          break;

        case "business_metrics":
          if (event.data?.error_rate !== undefined) {
            const errorRate = event.data.error_rate;
            newKpis.errorRate = {
              value: `${(errorRate * 100).toFixed(1)}%`,
              status: errorRate < 0.01 ? "success" :
                     errorRate < 0.05 ? "warning" : "error",
              trend: "+0.0%", // TODO: Calculate trend
            };
          }
          break;
      }

      return newKpis;
    });
  }, []);

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
        <CoordinationMetrics />
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
