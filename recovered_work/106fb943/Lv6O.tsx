"use client";

import React, { useState, useEffect } from "react";
import {
  DataQualityDashboardProps,
  DatabaseMetrics,
  DataQualityMetric,
} from "@/types/database";
import { databaseApiClient, DatabaseApiError } from "@/lib/database-api";
import styles from "./DataQualityDashboard.module.scss";

interface DataQualityDashboardState {
  metrics: DatabaseMetrics | null;
  isLoading: boolean;
  error: string | null;
  lastRefresh: Date | null;
}

export default function DataQualityDashboard({
  metrics: externalMetrics,
  isLoading: externalLoading,
  error: externalError,
  onRefresh,
}: DataQualityDashboardProps) {
  const [state, setState] = useState<DataQualityDashboardState>({
    metrics: externalMetrics ?? null,
    isLoading: externalLoading ?? false,
    error: externalError ?? null,
    lastRefresh: null,
  });

  // Load metrics if not provided externally
  const loadMetrics = async () => {
    if (externalMetrics) return;

    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));
      const metrics = await databaseApiClient.getDatabaseMetrics();
      setState((prev) => ({
        ...prev,
        metrics: metrics.metrics,
        isLoading: false,
        lastRefresh: new Date(),
      }));
    } catch (error) {
      const errorMessage =
        error instanceof DatabaseApiError
          ? error.message
          : "Failed to load data quality metrics";
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      console.error("Failed to load metrics:", error);
    }
  };

  // Handle refresh
  const handleRefresh = () => {
    loadMetrics();
    onRefresh?.();
  };

  // Update state when external props change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      metrics: externalMetrics ?? prev.metrics,
      isLoading: externalLoading ?? prev.isLoading,
      error: externalError ?? prev.error,
    }));
  }, [externalMetrics, externalLoading, externalError]);

  // Auto-load metrics on mount if not provided externally
  useEffect(() => {
    if (!externalMetrics) {
      loadMetrics();
    }
  }, [externalMetrics]);

  const getStatusColor = (status: DataQualityMetric["status"]) => {
    switch (status) {
      case "good":
        return styles.statusGood;
      case "warning":
        return styles.statusWarning;
      case "error":
        return styles.statusError;
      default:
        return styles.statusNeutral;
    }
  };

  const getStatusIcon = (status: DataQualityMetric["status"]) => {
    switch (status) {
      case "good":
        return "✅";
      case "warning":
        return "⚠️";
      case "error":
        return "❌";
      default:
        return "❓";
    }
  };

  const formatMetricValue = (metric: DataQualityMetric) => {
    const { value, unit } = metric ?? { value: 0, unit: "" };

    switch (unit) {
      case "percentage":
        return `${(value * 100).toFixed(1)}%`;
      case "currency":
        return `$${value.toFixed(2)}`;
      case "duration": {
        if (value < 1000) {
          return `${value.toFixed(0)}ms`;
        } else if (value < 60000) {
          return `${(value / 1000).toFixed(1)}s`;
        } else {
          return `${(value / 60000).toFixed(1)}m`;
        }
      }
      case "bytes": {
        const units = ["B", "KB", "MB", "GB", "TB"];
        let size = value;
        let unitIndex = 0;
        while (size >= 1024 && unitIndex < units.length - 1) {
          size /= 1024;
          unitIndex++;
        }
        return `${size.toFixed(1)} ${units[unitIndex]}`;
      }
      default:
        return unit ? `${value.toFixed(2)} ${unit}` : value.toFixed(2);
    }
  };

  const getTrendIcon = (trend?: DataQualityMetric["trend"]) => {
    switch (trend) {
      case "up":
        return "📈";
      case "down":
        return "📉";
      case "stable":
        return "➡️";
      default:
        return "";
    }
  };

  if (state.isLoading) {
    return (
      <div className={styles.dataQualityDashboard}>
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading data quality metrics...</p>
        </div>
      </div>
    );
  }

  if (state.error ?? !state.metrics) {
    return (
      <div className={styles.dataQualityDashboard}>
        <div className={styles.error}>
          <div className={styles.emptyIcon}>📊</div>
          <h3>Failed to load data quality metrics</h3>
          <p>{state.error}</p>
          <button onClick={handleRefresh} className={styles.retryButton}>
            Retry
          </button>
        </div>
      </div>
    );
  }

  const { metrics } = state;

  return (
    <div className={styles.dataQualityDashboard}>
      <div className={styles.dashboardHeader}>
        <h2>Data Quality & Performance Dashboard</h2>
        <div className={styles.headerActions}>
          <div className={styles.lastRefresh}>
            {state.lastRefresh && (
              <span>
                Last updated: {state.lastRefresh.toLocaleTimeString()}
              </span>
            )}
          </div>
          <button onClick={handleRefresh} className={styles.refreshButton}>
            🔄 Refresh
          </button>
        </div>
      </div>

      {/* Overall Quality Score */}
      <div className={styles.overallScore}>
        <div className={styles.scoreCard}>
          <div className={styles.scoreValue}>
            {metrics.overall_quality_score.toFixed(1)}%
          </div>
          <div className={styles.scoreLabel}>Overall Quality Score</div>
          <div className={styles.scoreBar}>
            <div
              className={styles.scoreFill}
              style={{ width: `${metrics.overall_quality_score}%` }}
            ></div>
          </div>
        </div>
      </div>

      <div className={styles.dashboardGrid}>
        {/* Database Overview */}
        <div className={styles.section}>
          <h3>Database Overview</h3>
          <div className={styles.metricGrid}>
            <div className={styles.metricCard}>
              <div className={styles.metricValue}>
                {metrics.total_tables.toLocaleString()}
              </div>
              <div className={styles.metricLabel}>Total Tables</div>
            </div>
            <div className={styles.metricCard}>
              <div className={styles.metricValue}>
                {metrics.total_rows.toLocaleString()}
              </div>
              <div className={styles.metricLabel}>Total Rows</div>
            </div>
            <div className={styles.metricCard}>
              <div className={styles.metricValue}>
                {(metrics.total_size_bytes / 1024 / 1024 / 1024).toFixed(2)} GB
              </div>
              <div className={styles.metricLabel}>Total Size</div>
            </div>
            <div className={styles.metricCard}>
              {(metrics.cache_hit_ratio * 100).toFixed(1)}%
              <div className={styles.metricLabel}>Cache Hit Ratio</div>
            </div>
          </div>
        </div>

        {/* Connection Pool Metrics */}
        <div className={styles.section}>
          <h3>Connection Pool</h3>
          <div className={styles.metricGrid}>
            <div className={styles.metricCard}>
              <div className={styles.metricValue}>
                {metrics.connections_active}
              </div>
              <div className={styles.metricLabel}>Active Connections</div>
            </div>
            <div className={styles.metricCard}>
              <div className={styles.metricValue}>
                {metrics.connections_idle}
              </div>
              <div className={styles.metricLabel}>Idle Connections</div>
            </div>
          </div>
        </div>

        {/* Data Quality Metrics */}
        <div className={styles.section}>
          <h3>Data Quality Metrics</h3>
          <div className={styles.qualityMetrics}>
            {metrics.table_metrics
              .flatMap((table) => table.data_quality)
              .slice(0, 12) // Show top 12 metrics
              .map((metric, index) => (
                <div
                  key={`${metric.name}-${index}`}
                  className={styles.qualityMetric}
                >
                  <div className={styles.metricHeader}>
                    <span className={styles.metricName}>{metric.name}</span>
                    <span
                      className={`${styles.metricStatus} ${getStatusColor(
                        metric.status
                      )}`}
                    >
                      {getStatusIcon(metric.status)}
                    </span>
                  </div>
                  <div className={styles.metricValue}>
                    {formatMetricValue(metric)}
                  </div>
                  <div className={styles.metricTrend}>
                    {getTrendIcon(metric.trend)}
                    {metric.change_percent !== undefined && (
                      <span className={styles.changePercent}>
                        {metric.change_percent > 0 ? "+" : ""}
                        {metric.change_percent.toFixed(1)}%
                      </span>
                    )}
                  </div>
                  <div className={styles.metricDescription}>
                    {metric.description}
                  </div>
                </div>
              ))}
          </div>
        </div>

        {/* Table Performance Metrics */}
        <div className={styles.section}>
          <h3>Table Performance</h3>
          <div className={styles.tablePerformance}>
            <div className={styles.performanceTable}>
              <div className={styles.tableHeader}>
                <div className={styles.tableCell}>Table</div>
                <div className={styles.tableCell}>Rows</div>
                <div className={styles.tableCell}>Avg Query Time</div>
                <div className={styles.tableCell}>Slow Queries</div>
                <div className={styles.tableCell}>Quality Score</div>
              </div>
              {metrics.table_metrics
                .sort(
                  (a, b) =>
                    b.performance.avg_query_time_ms -
                    a.performance.avg_query_time_ms
                )
                .slice(0, 10)
                .map((table) => (
                  <div key={table.table_name} className={styles.tableRow}>
                    <div className={styles.tableCell}>
                      <code className={styles.tableName}>
                        {table.table_name}
                      </code>
                    </div>
                    <div className={styles.tableCell}>
                      {table.row_count.toLocaleString()}
                    </div>
                    <div className={styles.tableCell}>
                      {table.performance.avg_query_time_ms.toFixed(0)}ms
                    </div>
                    <div className={styles.tableCell}>
                      {table.performance.slow_queries}
                    </div>
                    <div className={styles.tableCell}>
                      <div className={styles.qualityScore}>
                        <div
                          className={styles.qualityBar}
                          style={{
                            width: `${Math.min(
                              (table.data_quality.reduce(
                                (sum, m) =>
                                  sum +
                                  (m.status === "good"
                                    ? 1
                                    : m.status === "warning"
                                    ? 0.5
                                    : 0),
                                0
                              ) /
                                table.data_quality.length) *
                                100,
                              100
                            )}%`,
                          }}
                        ></div>
                        <span className={styles.qualityPercent}>
                          {(
                            (table.data_quality.filter(
                              (m) => m.status === "good"
                            ).length /
                              table.data_quality.length) *
                            100
                          ).toFixed(0)}
                          %
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
            </div>
          </div>
        </div>

        {/* Performance Insights */}
        <div className={styles.section}>
          <h3>Performance Insights</h3>
          <div className={styles.insights}>
            {metrics.table_metrics.some(
              (t) => t.performance.slow_queries > 5
            ) && (
              <div className={styles.insight}>
                <span className={styles.insightIcon}>🐌</span>
                <div className={styles.insightContent}>
                  <h4>Slow Query Alert</h4>
                  <p>
                    {
                      metrics.table_metrics.filter(
                        (t) => t.performance.slow_queries > 5
                      ).length
                    }{" "}
                    tables have high slow query counts. Consider adding indexes
                    or optimizing queries.
                  </p>
                </div>
              </div>
            )}

            {metrics.cache_hit_ratio < 0.8 && (
              <div className={styles.insight}>
                <span className={styles.insightIcon}>💾</span>
                <div className={styles.insightContent}>
                  <h4>Low Cache Hit Ratio</h4>
                  <p>
                    Cache hit ratio is{" "}
                    {(metrics.cache_hit_ratio * 100).toFixed(1)}%. Consider
                    increasing shared buffer size or adding more indexes.
                  </p>
                </div>
              </div>
            )}

            {metrics.connections_active > metrics.connections_idle * 2 && (
              <div className={styles.insight}>
                <span className={styles.insightIcon}>🔗</span>
                <div className={styles.insightContent}>
                  <h4>High Connection Usage</h4>
                  <p>
                    Active connections ({metrics.connections_active})
                    significantly exceed idle connections (
                    {metrics.connections_idle}). Consider connection pooling.
                  </p>
                </div>
              </div>
            )}

            {metrics.table_metrics.some((t) =>
              t.data_quality.some((m) => m.status === "error")
            ) && (
              <div className={styles.insight}>
                <span className={styles.insightIcon}>⚠️</span>
                <div className={styles.insightContent}>
                  <h4>Data Quality Issues</h4>
                  <p>
                    Some tables have failing data quality checks. Review
                    constraints and data validation rules.
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
