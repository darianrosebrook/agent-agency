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

  console.warn(
    "Using mock metrics dashboard - V3 metrics aggregation not available"
  );

  // Mock KPI data
  const mockKPIs = {
    systemHealth: { value: "98.5%", status: "success" as const, trend: "+0.2%" },
    activeAgents: { value: "12", status: "success" as const, trend: "+2" },
    taskThroughput: { value: "45/min", status: "warning" as const, trend: "-5%" },
    errorRate: { value: "0.8%", status: "success" as const, trend: "-0.1%" },
    avgResponseTime: { value: "245ms", status: "success" as const, trend: "-12ms" },
    coordinationEfficiency: { value: "92%", status: "success" as const, trend: "+1.5%" },
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
            value="Loading..."
            status="neutral"
            icon="🏥"
            loading={true}
          />
          <MetricTile
            title="Active Agents"
            value="Loading..."
            status="neutral"
            icon="🤖"
            loading={true}
          />
          <MetricTile
            title="Task Success Rate"
            value="Loading..."
            status="neutral"
            icon="✅"
            loading={true}
          />
          <MetricTile
            title="Avg Response Time"
            value="Loading..."
            status="neutral"
            icon="⚡"
            format="duration"
            loading={true}
          />
          <MetricTile
            title="Messages/Minute"
            value="Loading..."
            status="neutral"
            icon="💬"
            loading={true}
          />
          <MetricTile
            title="Cost Efficiency"
            value="Loading..."
            status="neutral"
            icon="💰"
            loading={true}
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
              // TODO: Implement refresh all metrics functionality
            }}
          >
            🔄 Refresh All
          </button>
          <button
            className={styles.secondaryButton}
            onClick={() => {
              console.log("Exporting metrics...");
              // TODO: Implement metrics export functionality
            }}
            disabled
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
