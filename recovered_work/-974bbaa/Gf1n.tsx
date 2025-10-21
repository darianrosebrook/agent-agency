"use client";

import React, { useState, useEffect } from "react";
import {
  BusinessIntelligenceProps,
  BusinessMetrics,
  GetBusinessMetricsResponse,
} from "@/types/metrics";
import { metricsApiClient, MetricsApiError } from "@/lib/metrics-api";
import MetricTile from "./MetricTile";
import styles from "./BusinessIntelligence.module.scss";

export default function BusinessIntelligence({
  metrics: externalMetrics,
  isLoading: externalLoading,
  error: externalError,
  timeRange = "24h",
  onTimeRangeChange,
}: BusinessIntelligenceProps) {
  const [metrics, setMetrics] = useState<GetBusinessMetricsResponse | null>(
    externalMetrics ? { metrics: externalMetrics, time_series: [] } : null
  );
  const [isLoading, setIsLoading] = useState(
    externalLoading || !externalMetrics
  );
  const [error, setError] = useState<string | null>(externalError || null);
  const [selectedTimeRange, setSelectedTimeRange] = useState(timeRange);

  // Load business metrics if not provided externally
  const loadBusinessMetrics = async (range: string = selectedTimeRange) => {
    if (externalMetrics) return; // Use external data if provided

    try {
      setIsLoading(true);
      setError(null);

      const businessData = await metricsApiClient.getBusinessMetrics(range as any);
      setMetrics(businessData);
    } catch (err) {
      const errorMessage =
        err instanceof MetricsApiError
          ? err.message
          : "Failed to load business metrics";

      setError(errorMessage);
      console.error("Failed to load business metrics:", err);
    } finally {
      setIsLoading(false);
    }
  };

  // Handle time range changes
  const handleTimeRangeChange = (newRange: string) => {
    setSelectedTimeRange(newRange);
    onTimeRangeChange?.(newRange);
    loadBusinessMetrics(newRange);
  };

  // Initial load and external prop updates
  useEffect(() => {
    if (externalMetrics) {
      setMetrics({ metrics: externalMetrics, time_series: [] });
      setIsLoading(false);
      setError(null);
    } else if (!externalMetrics && !isLoading) {
      loadBusinessMetrics();
    }
  }, [externalMetrics]);

  // Format currency
  const formatCurrency = (amount: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
    }).format(amount);
  };

  // Calculate trends (mock for now - would come from time series data)
  const calculateTrend = (current: number, previous?: number): number | undefined => {
    if (!previous || previous === 0) return undefined;
    return ((current - previous) / previous) * 100;
  };

  if (isLoading) {
    return (
      <div className={styles.businessIntelligence}>
        <div className={styles.loading}>
          <div className={styles.loadingSpinner}></div>
          <p>Loading business metrics...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.businessIntelligence}>
        <div className={styles.error}>
          <h3>Failed to load business metrics</h3>
          <p>{error}</p>
          <button onClick={() => loadBusinessMetrics()}>Retry</button>
        </div>
      </div>
    );
  }

  if (!metrics?.metrics) {
    return (
      <div className={styles.businessIntelligence}>
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>📊</div>
          <h3>Business Intelligence</h3>
          <p>Real-time business metrics and KPIs require V3 business intelligence APIs.</p>
          <div className={styles.emptyActions}>
            <button className={styles.secondaryButton} disabled>
              Connect to Business Metrics API
            </button>
          </div>
        </div>
      </div>
    );
  }

  const businessMetrics = metrics.metrics;

  return (
    <div className={styles.businessIntelligence}>
      <div className={styles.header}>
        <h2>Business Intelligence</h2>
        <div className={styles.timeRangeSelector}>
          {(['1h', '6h', '24h', '7d', '30d'] as const).map((range) => (
            <button
              key={range}
              className={`${styles.timeRangeButton} ${
                selectedTimeRange === range ? styles.active : ''
              }`}
              onClick={() => handleTimeRangeChange(range)}
            >
              {range}
            </button>
          ))}
        </div>
      </div>

      <div className={styles.kpiGrid}>
        <MetricTile
          title="Tasks Created"
          value={businessMetrics.total_tasks_created}
          status="neutral"
          icon="📝"
        />

        <MetricTile
          title="Completed Today"
          value={businessMetrics.tasks_completed_today}
          status="success"
          icon="✅"
        />

        <MetricTile
          title="Success Rate"
          value={`${(businessMetrics.task_success_rate * 100).toFixed(1)}%`}
          status={businessMetrics.task_success_rate >= 0.95 ? "success" : businessMetrics.task_success_rate >= 0.8 ? "warning" : "error"}
          icon="🎯"
          format="percentage"
        />

        <MetricTile
          title="Avg Completion Time"
          value={businessMetrics.average_task_completion_time_ms}
          status={businessMetrics.average_task_completion_time_ms < 300000 ? "success" : businessMetrics.average_task_completion_time_ms < 900000 ? "warning" : "error"}
          icon="⏱️"
          format="duration"
        />
      </div>

      <div className={styles.qualitySection}>
        <h3>Quality Metrics</h3>
        <div className={styles.qualityMetrics}>
          <MetricTile
            title="Quality Checks Passed"
            value={businessMetrics.quality_checks_passed}
            status="success"
            icon="✅"
          />

          <MetricTile
            title="Quality Checks Failed"
            value={businessMetrics.quality_checks_failed}
            status={businessMetrics.quality_checks_failed === 0 ? "success" : businessMetrics.quality_checks_failed < 10 ? "warning" : "error"}
            icon="❌"
          />

          <MetricTile
            title="Average Quality Score"
            value={`${businessMetrics.average_quality_score.toFixed(1)}%`}
            status={businessMetrics.average_quality_score >= 85 ? "success" : businessMetrics.average_quality_score >= 70 ? "warning" : "error"}
            icon="📊"
            format="percentage"
          />
        </div>
      </div>

      <div className={styles.costSection}>
        <h3>Cost & Efficiency</h3>
        <div className={styles.costMetrics}>
          <MetricTile
            title="Total Cost Today"
            value={businessMetrics.total_cost_today}
            status={businessMetrics.total_cost_today < 100 ? "success" : businessMetrics.total_cost_today < 500 ? "warning" : "error"}
            icon="💰"
            format="currency"
          />

          <MetricTile
            title="Cost Per Task"
            value={businessMetrics.cost_per_task}
            status={businessMetrics.cost_per_task < 10 ? "success" : businessMetrics.cost_per_task < 25 ? "warning" : "error"}
            icon="💵"
            format="currency"
          />

          <MetricTile
            title="Efficiency Trend"
            value={`${businessMetrics.efficiency_trend.toFixed(1)}%`}
            change={businessMetrics.efficiency_trend}
            status={businessMetrics.efficiency_trend > 0 ? "success" : businessMetrics.efficiency_trend > -10 ? "warning" : "error"}
            icon="📈"
            format="percentage"
            trend={businessMetrics.efficiency_trend > 0 ? "up" : businessMetrics.efficiency_trend < 0 ? "down" : "stable"}
          />
        </div>
      </div>

      {(businessMetrics.active_sessions !== undefined || businessMetrics.average_session_duration_ms !== undefined) && (
        <div className={styles.engagementSection}>
          <h3>User Engagement</h3>
          <div className={styles.engagementMetrics}>
            {businessMetrics.active_sessions !== undefined && (
              <MetricTile
                title="Active Sessions"
                value={businessMetrics.active_sessions}
                status="neutral"
                icon="👥"
              />
            )}

            {businessMetrics.average_session_duration_ms !== undefined && (
              <MetricTile
                title="Avg Session Duration"
                value={businessMetrics.average_session_duration_ms}
                status={businessMetrics.average_session_duration_ms > 300000 ? "success" : businessMetrics.average_session_duration_ms > 60000 ? "warning" : "error"}
                icon="⏱️"
                format="duration"
              />
            )}
          </div>
        </div>
      )}

      <div className={styles.performanceIndicators}>
        <h3>Performance Indicators</h3>
        <div className={styles.indicatorsGrid}>
          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {businessMetrics.task_success_rate >= 0.9 ? "✅" : businessMetrics.task_success_rate >= 0.7 ? "⚠️" : "❌"}
              </span>
              <span>Task Success Rate</span>
            </div>
            <div className={styles.indicatorValue}>
              {businessMetrics.task_success_rate >= 0.9 ? "Excellent" : businessMetrics.task_success_rate >= 0.7 ? "Good" : "Needs Improvement"}
            </div>
          </div>

          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {businessMetrics.average_task_completion_time_ms < 600000 ? "✅" : businessMetrics.average_task_completion_time_ms < 1800000 ? "⚠️" : "❌"}
              </span>
              <span>Task Completion Speed</span>
            </div>
            <div className={styles.indicatorValue}>
              {businessMetrics.average_task_completion_time_ms < 600000 ? "Fast" : businessMetrics.average_task_completion_time_ms < 1800000 ? "Moderate" : "Slow"}
            </div>
          </div>

          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {businessMetrics.cost_per_task < 20 ? "✅" : businessMetrics.cost_per_task < 50 ? "⚠️" : "❌"}
              </span>
              <span>Cost Efficiency</span>
            </div>
            <div className={styles.indicatorValue}>
              {businessMetrics.cost_per_task < 20 ? "Efficient" : businessMetrics.cost_per_task < 50 ? "Moderate" : "High Cost"}
            </div>
          </div>

          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {businessMetrics.quality_checks_failed === 0 ? "✅" : businessMetrics.quality_checks_failed < 5 ? "⚠️" : "❌"}
              </span>
              <span>Quality Assurance</span>
            </div>
            <div className={styles.indicatorValue}>
              {businessMetrics.quality_checks_failed === 0 ? "Perfect" : businessMetrics.quality_checks_failed < 5 ? "Good" : "Needs Attention"}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
