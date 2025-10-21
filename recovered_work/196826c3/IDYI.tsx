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
  console.log("MetricsDashboard component rendering");

  const [timeRange, setTimeRange] = useState<
    "1h" | "6h" | "24h" | "7d" | "30d"
  >("24h");

  console.warn(
    "Using mock metrics dashboard - V3 metrics aggregation not available"
  );

  // Mock KPI data
  const mockKPIs = {
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
  };

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
            value="98.5%"
            status="success"
            icon="🏥"
            trend="up"
          />
          <MetricTile
            title="Active Agents"
            value="12"
            status="success"
            icon="🤖"
            trend="up"
          />
          <MetricTile
            title="Task Success Rate"
            value="94.2%"
            status="success"
            icon="✅"
            trend="up"
          />
          <MetricTile
            title="Avg Response Time"
            value="245ms"
            status="success"
            icon="⚡"
            format="duration"
            trend="down"
          />
          <MetricTile
            title="Messages/Minute"
            value="156"
            status="warning"
            icon="💬"
            trend="down"
          />
          <MetricTile
            title="Cost Efficiency"
            value="$0.023/token"
            status="success"
            icon="💰"
            trend="down"
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
        onMetricsUpdate={(event) => {
          console.log("Real-time metrics update:", event);
          // TODO: Update KPI tiles and components with real-time data
        }}
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
