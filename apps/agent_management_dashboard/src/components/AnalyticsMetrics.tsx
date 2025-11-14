"use client";

import { useState, useEffect } from "react";
import { BentoPanel } from "./compounds/BentoPanel";
import {
  getTaskAnalytics,
  getPerformanceAnalytics,
  getSuccessRates,
  type TaskAnalytics,
  type PerformanceAnalytics,
  type SuccessRates,
} from "../lib/api/analytics";
import styles from "./AnalyticsMetrics.module.scss";

interface AnalyticsMetricsProps {
  title?: string;
}

export function AnalyticsMetrics({
  title = "Analytics Overview",
}: AnalyticsMetricsProps) {
  const [taskAnalytics, setTaskAnalytics] = useState<TaskAnalytics | null>(
    null
  );
  const [performanceAnalytics, setPerformanceAnalytics] =
    useState<PerformanceAnalytics | null>(null);
  const [successRates, setSuccessRates] = useState<SuccessRates | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchAnalytics() {
      setIsLoading(true);
      setError(null);
      try {
        const [taskData, performanceData, successData] = await Promise.all([
          getTaskAnalytics(),
          getPerformanceAnalytics(),
          getSuccessRates(),
        ]);
        setTaskAnalytics(taskData);
        setPerformanceAnalytics(performanceData);
        setSuccessRates(successData);
      } catch (err) {
        console.error("Failed to fetch analytics:", err);
        setError(
          err instanceof Error ? err : new Error("Failed to load analytics")
        );
      } finally {
        setIsLoading(false);
      }
    }

    fetchAnalytics();
    // Refresh every 30 seconds
    const interval = setInterval(fetchAnalytics, 30000);
    return () => clearInterval(interval);
  }, []);

  if (isLoading) {
    return (
      <BentoPanel>
        <div className={styles.container}>
          <h3 className={styles.title}>{title}</h3>
          <p className={styles.loading}>Loading analytics...</p>
        </div>
      </BentoPanel>
    );
  }

  if (error) {
    return (
      <BentoPanel>
        <div className={styles.container}>
          <h3 className={styles.title}>{title}</h3>
          <p className={styles.error}>Error: {error.message}</p>
        </div>
      </BentoPanel>
    );
  }

  // Parse success rate from string format "XX.XX%" to number
  const parseSuccessRate = (rateStr: string | undefined | null): number => {
    if (!rateStr || typeof rateStr !== 'string') {
      return 0;
    }
    const match = rateStr.match(/(\d+\.?\d*)/);
    return match ? parseFloat(match[1]) : 0;
  };

  // Safely format a number with toFixed, handling NaN and undefined
  const safeToFixed = (value: number | undefined | null, decimals: number = 1): string => {
    const num = typeof value === 'number' && !isNaN(value) ? value : 0;
    return num.toFixed(decimals);
  };

  const successRate = successRates?.success_rate
    ? parseSuccessRate(successRates.success_rate)
    : 0;
  const performanceScore = performanceAnalytics?.performance_score ?? 0;
  const avgDuration = performanceAnalytics?.average_task_duration_ms ?? 0;
  const avgDurationSeconds = safeToFixed(avgDuration / 1000, 1);

  return (
    <BentoPanel>
      <div className={styles.container}>
        <h3 className={styles.title}>{title}</h3>

        <div className={styles.metricsGrid}>
          {/* Success Rate */}
          <div className={styles.metricCard}>
            <div className={styles.metricLabel}>Success Rate</div>
            <div className={styles.metricValue}>{safeToFixed(successRate, 1)}%</div>
            {successRates && (
              <div className={styles.metricDetail}>
                {successRates.completed} / {successRates.total_tasks} tasks
              </div>
            )}
          </div>

          {/* Performance Score */}
          <div className={styles.metricCard}>
            <div className={styles.metricLabel}>Performance Score</div>
            <div className={styles.metricValue}>
              {safeToFixed(performanceScore, 1)}
            </div>
            {performanceAnalytics && (
              <div className={styles.metricDetail}>
                {performanceAnalytics.completed_tasks_count} completed
              </div>
            )}
          </div>

          {/* Average Duration */}
          <div className={styles.metricCard}>
            <div className={styles.metricLabel}>Avg Task Duration</div>
            <div className={styles.metricValue}>{avgDurationSeconds}s</div>
            {performanceAnalytics && (
              <div className={styles.metricDetail}>
                {performanceAnalytics.total_tasks} total tasks
              </div>
            )}
          </div>

          {/* Task Breakdown */}
          {taskAnalytics && (
            <>
              <div className={styles.metricCard}>
                <div className={styles.metricLabel}>In Progress</div>
                <div className={styles.metricValue}>
                  {taskAnalytics.in_progress}
                </div>
                <div className={styles.metricDetail}>Active tasks</div>
              </div>

              <div className={styles.metricCard}>
                <div className={styles.metricLabel}>Failed</div>
                <div className={styles.metricValue}>{taskAnalytics.failed}</div>
                <div className={styles.metricDetail}>Failed tasks</div>
              </div>

              <div className={styles.metricCard}>
                <div className={styles.metricLabel}>Avg Chain of Thought</div>
                <div className={styles.metricValue}>
                  {safeToFixed(taskAnalytics.average_chain_of_thought_entries, 1)}
                </div>
                <div className={styles.metricDetail}>Entries per task</div>
              </div>
            </>
          )}
        </div>
      </div>
    </BentoPanel>
  );
}









