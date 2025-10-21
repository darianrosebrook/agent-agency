"use client";

import React, { useState, useEffect, useCallback } from "react";
import {
  AnalyticsDashboardProps,
  AnalyticsSummary,
  AnalyticsFilters,
  AnalyticsCardProps,
} from "@/types/analytics";
import { analyticsApiClient, AnalyticsApiError } from "@/lib/analytics-api";
import AnomalyDetector from "./AnomalyDetector";
import TrendAnalyzer from "./TrendAnalyzer";
import PerformancePredictor from "./PerformancePredictor";
import CorrelationMatrix from "./CorrelationMatrix";
import styles from "./AnalyticsDashboard.module.scss";

interface AnalyticsDashboardState {
  summary: AnalyticsSummary | null;
  filters: AnalyticsFilters;
  activeTab:
    | "overview"
    | "anomalies"
    | "trends"
    | "predictions"
    | "correlations";
  isLoading: boolean;
  error: string | null;
  lastUpdated: Date | null;
}

export default function AnalyticsDashboard({
  summary: externalSummary,
  filters: externalFilters,
  onFiltersChange,
  onRefresh,
  isLoading: externalLoading,
  error: externalError,
}: AnalyticsDashboardProps) {
  const [state, setState] = useState<AnalyticsDashboardState>({
    summary: externalSummary ?? null,
    filters: externalFilters ?? {
      time_range: {
        start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(), // 7 days ago
        end: new Date().toISOString(),
      },
      granularity: "1h",
    },
    activeTab: "overview",
    isLoading: externalLoading ?? false,
    error: externalError ?? null,
    lastUpdated: null,
  });

  // Load analytics summary if not provided externally
  const loadAnalyticsSummary = useCallback(async () => {
    if (externalSummary) return;

    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));
      const response = await analyticsApiClient.getAnalyticsSummary(
        state.filters
      );
      setState((prev) => ({
        ...prev,
        summary: response.summary,
        isLoading: false,
        lastUpdated: new Date(),
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to load analytics summary";
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      console.error("Failed to load analytics summary:", error);
    }
  }, [externalSummary, state.filters]);

  // Handle filter changes
  const handleFiltersChange = useCallback(
    (newFilters: AnalyticsFilters) => {
      setState((prev) => ({ ...prev, filters: newFilters }));
      onFiltersChange?.(newFilters);
    },
    [onFiltersChange]
  );

  // Handle tab changes
  const handleTabChange = useCallback(
    (tab: AnalyticsDashboardState["activeTab"]) => {
      setState((prev) => ({ ...prev, activeTab: tab }));
    },
    []
  );

  // Handle refresh
  const handleRefresh = useCallback(() => {
    loadAnalyticsSummary();
    onRefresh?.();
  }, [loadAnalyticsSummary, onRefresh]);

  // Initialize data on mount
  useEffect(() => {
    if (!externalSummary) {
      loadAnalyticsSummary();
    }
  }, [externalSummary, loadAnalyticsSummary]);

  // Update state when external props change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      summary: externalSummary ?? prev.summary,
      filters: externalFilters ?? prev.filters,
      isLoading: externalLoading ?? prev.isLoading,
      error: externalError ?? prev.error,
    }));
  }, [externalSummary, externalFilters, externalLoading, externalError]);

  const AnalyticsCard: React.FC<AnalyticsCardProps> = ({
    title,
    value,
    change,
    icon,
    status,
    trend,
    description,
    onClick,
  }) => (
    <div
      className={`${styles.analyticsCard} ${onClick ? styles.clickable : ""}`}
      onClick={onClick}
    >
      <div className={styles.cardHeader}>
        <div className={styles.cardTitle}>
          {icon && <span className={styles.cardIcon}>{icon}</span>}
          <span>{title}</span>
        </div>
        {status && (
          <span className={`${styles.statusBadge} ${styles[status]}`}>
            {status}
          </span>
        )}
      </div>

      <div className={styles.cardValue}>{value}</div>

      {change && (
        <div className={`${styles.cardChange} ${styles[change.type]}`}>
          <span className={styles.changeIcon}>
            {change.type === "increase"
              ? "↗️"
              : change.type === "decrease"
              ? "↘️"
              : "➡️"}
          </span>
          <span className={styles.changeValue}>
            {change.value > 0 ? "+" : ""}
            {change.value}
          </span>
          <span className={styles.changePeriod}>{change.period}</span>
        </div>
      )}

      {trend && (
        <div className={`${styles.cardTrend} ${styles[trend]}`}>
          {trend === "up" ? "📈" : trend === "down" ? "📉" : "➡️"}
        </div>
      )}

      {description && (
        <div className={styles.cardDescription}>{description}</div>
      )}
    </div>
  );

  return (
    <div className={styles.analyticsDashboard}>
      <div className={styles.dashboardHeader}>
        <h1>Analytics & Insights</h1>
        <p className={styles.description}>
          Advanced analytics with anomaly detection, trend analysis, and
          performance predictions for agent research and optimization.
        </p>

        <div className={styles.headerControls}>
          <div className={styles.timeRangeControls}>
            <label>Time Range:</label>
            <select
              value={`${state.filters.time_range.start.split("T")[0]} to ${
                state.filters.time_range.end.split("T")[0]
              }`}
              onChange={(e) => {
                // Parse the selected range and update filters
                const ranges = {
                  "Last 1 hour": {
                    start: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
                    end: new Date().toISOString(),
                  },
                  "Last 24 hours": {
                    start: new Date(
                      Date.now() - 24 * 60 * 60 * 1000
                    ).toISOString(),
                    end: new Date().toISOString(),
                  },
                  "Last 7 days": {
                    start: new Date(
                      Date.now() - 7 * 24 * 60 * 60 * 1000
                    ).toISOString(),
                    end: new Date().toISOString(),
                  },
                  "Last 30 days": {
                    start: new Date(
                      Date.now() - 30 * 24 * 60 * 60 * 1000
                    ).toISOString(),
                    end: new Date().toISOString(),
                  },
                };
                const value = (e.target as HTMLSelectElement).value;
                const selectedRange = Object.entries(ranges).find(([label]) =>
                  value.includes(label.split(" ")[1] || "")
                );
                if (selectedRange) {
                  handleFiltersChange({
                    ...state.filters,
                    time_range: selectedRange[1],
                  });
                }
              }}
            >
              <option>Last 1 hour</option>
              <option>Last 24 hours</option>
              <option>Last 7 days</option>
              <option>Last 30 days</option>
            </select>
          </div>

          <div className={styles.granularityControls}>
            <label>Granularity:</label>
            <select
              value={state.filters.granularity}
              onChange={(e) =>
                handleFiltersChange({
                  ...state.filters,
                  granularity: e.target
                    .value as AnalyticsFilters["granularity"],
                })
              }
            >
              <option value="1m">1 minute</option>
              <option value="5m">5 minutes</option>
              <option value="15m">15 minutes</option>
              <option value="1h">1 hour</option>
              <option value="6h">6 hours</option>
              <option value="1d">1 day</option>
            </select>
          </div>

          <button
            onClick={handleRefresh}
            className={styles.refreshButton}
            disabled={state.isLoading}
          >
            {state.isLoading ? "🔄 Loading..." : "🔄 Refresh"}
          </button>
        </div>
      </div>

      {/* Error Display */}
      {state.error && (
        <div className={styles.errorBanner}>
          <span className={styles.errorIcon}>⚠️</span>
          <span>{state.error}</span>
        </div>
      )}

      {/* Tab Navigation */}
      <div className={styles.tabNavigation}>
        <button
          className={`${styles.tabButton} ${
            state.activeTab === "overview" ? styles.active : ""
          }`}
          onClick={() => handleTabChange("overview")}
        >
          Overview
        </button>
        <button
          className={`${styles.tabButton} ${
            state.activeTab === "anomalies" ? styles.active : ""
          }`}
          onClick={() => handleTabChange("anomalies")}
        >
          Anomalies ({state.summary?.total_anomalies ?? 0})
        </button>
        <button
          className={`${styles.tabButton} ${
            state.activeTab === "trends" ? styles.active : ""
          }`}
          onClick={() => handleTabChange("trends")}
        >
          Trends
        </button>
        <button
          className={`${styles.tabButton} ${
            state.activeTab === "predictions" ? styles.active : ""
          }`}
          onClick={() => handleTabChange("predictions")}
        >
          Predictions
        </button>
        <button
          className={`${styles.tabButton} ${
            state.activeTab === "correlations" ? styles.active : ""
          }`}
          onClick={() => handleTabChange("correlations")}
        >
          Correlations
        </button>
      </div>

      {/* Tab Content */}
      <div className={styles.tabContent}>
        {state.activeTab === "overview" && (
          <div className={styles.overviewTab}>
            {state.isLoading ? (
              <div className={styles.loading}>
                <div className={styles.spinner}></div>
                <p>Loading analytics overview...</p>
              </div>
            ) : state.summary ? (
              <>
                {/* Key Metrics Cards */}
                <div className={styles.metricsGrid}>
                  <AnalyticsCard
                    title="System Health"
                    value={`${state.summary.system_health_score.toFixed(1)}%`}
                    status={
                      state.summary.system_health_score >= 90
                        ? "success"
                        : state.summary.system_health_score >= 70
                        ? "warning"
                        : "error"
                    }
                    icon="🏥"
                    trend={
                      state.summary.system_health_score >= 85
                        ? "up"
                        : state.summary.system_health_score >= 70
                        ? "stable"
                        : "down"
                    }
                    description="Overall system health and performance score"
                  />

                  <AnalyticsCard
                    title="Total Anomalies"
                    value={state.summary.total_anomalies.toString()}
                    status={
                      state.summary.total_anomalies === 0
                        ? "success"
                        : state.summary.total_anomalies <= 5
                        ? "warning"
                        : "error"
                    }
                    icon="⚠️"
                    change={{
                      value: state.summary.anomalies_by_severity.critical,
                      type: "neutral",
                      period: "critical",
                    }}
                    description="Detected anomalies across all metrics"
                  />

                  <AnalyticsCard
                    title="Prediction Accuracy"
                    value={`${(state.summary.prediction_accuracy * 100).toFixed(
                      1
                    )}%`}
                    status={
                      state.summary.prediction_accuracy >= 0.8
                        ? "success"
                        : state.summary.prediction_accuracy >= 0.6
                        ? "warning"
                        : "error"
                    }
                    icon="🔮"
                    trend={
                      state.summary.prediction_accuracy >= 0.75
                        ? "up"
                        : "stable"
                    }
                    description="Accuracy of performance predictions"
                  />

                  <AnalyticsCard
                    title="Active Metrics"
                    value={state.summary.total_metrics.toString()}
                    icon="📊"
                    trend="stable"
                    description="Total metrics being monitored"
                  />
                </div>

                {/* Anomaly Severity Breakdown */}
                <div className={styles.severityBreakdown}>
                  <h3>Anomaly Severity Distribution</h3>
                  <div className={styles.severityGrid}>
                    <div className={styles.severityItem}>
                      <span className={styles.severityLabel}>Critical</span>
                      <span
                        className={`${styles.severityValue} ${styles.critical}`}
                      >
                        {state.summary.anomalies_by_severity.critical}
                      </span>
                    </div>
                    <div className={styles.severityItem}>
                      <span className={styles.severityLabel}>High</span>
                      <span
                        className={`${styles.severityValue} ${styles.high}`}
                      >
                        {state.summary.anomalies_by_severity.high}
                      </span>
                    </div>
                    <div className={styles.severityItem}>
                      <span className={styles.severityLabel}>Medium</span>
                      <span
                        className={`${styles.severityValue} ${styles.medium}`}
                      >
                        {state.summary.anomalies_by_severity.medium}
                      </span>
                    </div>
                    <div className={styles.severityItem}>
                      <span className={styles.severityLabel}>Low</span>
                      <span className={`${styles.severityValue} ${styles.low}`}>
                        {state.summary.anomalies_by_severity.low}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Key Insights */}
                <div className={styles.insightsSection}>
                  <h3>Key Insights</h3>
                  <div className={styles.insightsList}>
                    {state.summary.key_insights.map((insight, index) => (
                      <div key={index} className={styles.insightItem}>
                        <span className={styles.insightIcon}>💡</span>
                        <span>{insight}</span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Recommendations */}
                <div className={styles.recommendationsSection}>
                  <h3>Recommendations</h3>
                  <div className={styles.recommendationsList}>
                    {state.summary.recommendations.map(
                      (recommendation, index) => (
                        <div key={index} className={styles.recommendationItem}>
                          <span className={styles.recommendationIcon}>🎯</span>
                          <span>{recommendation}</span>
                        </div>
                      )
                    )}
                  </div>
                </div>
              </>
            ) : (
              <div className={styles.noData}>
                <div className={styles.emptyIcon}>📊</div>
                <h3>No Analytics Data Available</h3>
                <p>
                  Analytics data will be available once the V3 analytics APIs
                  are implemented. Configure anomaly detection, trend analysis,
                  and forecasting in your agent system.
                </p>
                <div className={styles.setupSteps}>
                  <h4>To enable analytics:</h4>
                  <ol>
                    <li>Implement V3 analytics service endpoints</li>
                    <li>Configure anomaly detection algorithms</li>
                    <li>Set up time series data collection</li>
                    <li>Enable performance prediction models</li>
                  </ol>
                </div>
              </div>
            )}
          </div>
        )}

        {state.activeTab === "anomalies" && (
          <AnomalyDetector
            filters={state.filters}
            isDetecting={state.isLoading}
            error={state.error}
          />
        )}

        {state.activeTab === "trends" && (
          <TrendAnalyzer
            filters={state.filters}
            isAnalyzing={state.isLoading}
            error={state.error}
          />
        )}

        {state.activeTab === "predictions" && (
          <PerformancePredictor
            filters={state.filters}
            isPredicting={state.isLoading}
            error={state.error}
          />
        )}

        {state.activeTab === "correlations" && (
          <CorrelationMatrix
            isAnalyzing={state.isLoading}
            error={state.error}
          />
        )}
      </div>

      {/* Last Updated */}
      {state.lastUpdated && (
        <div className={styles.lastUpdated}>
          Last updated: {state.lastUpdated.toLocaleString()}
        </div>
      )}
    </div>
  );
}
