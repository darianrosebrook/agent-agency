"use client";

import { BarChart3, Zap, CheckCircle, XCircle, Clock, Target } from "lucide-react";
import { TaskMetrics as TaskMetricsType } from "@/types/tasks";
import styles from "./TaskMetrics.module.scss";

interface TaskMetricsProps {
  metrics: TaskMetricsType;
}

export default function TaskMetrics({ metrics }: TaskMetricsProps) {
  const formatNumber = (num: number | undefined | null) => {
    if (num === undefined || num === null || isNaN(num)) return "0";
    return new Intl.NumberFormat("en-US").format(num);
  };

  const formatPercentage = (num: number | undefined | null) => {
    if (num === undefined || num === null || isNaN(num)) return "0.0%";
    return `${num.toFixed(1)}%`;
  };

  const formatDuration = (milliseconds: number | undefined | null) => {
    if (milliseconds === undefined || milliseconds === null || isNaN(milliseconds)) return "0s";
    
    const seconds = Math.floor(milliseconds / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) {
      return `${days}d ${hours % 24}h`;
    } else if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  };

  const getSuccessRateColor = (rate: number | undefined | null) => {
    if (rate === undefined || rate === null || isNaN(rate)) return styles.error;
    if (rate >= 90) return styles.success;
    if (rate >= 70) return styles.warning;
    return styles.error;
  };

  const getCompletionTimeColor = (time: number | undefined | null) => {
    if (time === undefined || time === null || isNaN(time)) return styles.error;
    // Assuming good completion time is under 5 minutes
    if (time < 300000) return styles.success; // 5 minutes
    if (time < 900000) return styles.warning; // 15 minutes
    return styles.error;
  };

  return (
    <div className={styles.metrics}>
      <div className={styles.header}>
        <h3>Task Metrics</h3>
        <p className={styles.subtitle}>Real-time task execution statistics</p>
      </div>

      <div className={styles.metricsGrid}>
        <div className={styles.metricCard}>
          <BarChart3 className={styles.metricIcon} size={24} />
          <div className={styles.metricContent}>
            <div className={styles.metricValue}>{formatNumber(metrics.total_tasks)}</div>
            <div className={styles.metricLabel}>Total Tasks</div>
          </div>
        </div>

        <div className={styles.metricCard}>
          <Zap className={styles.metricIcon} size={24} />
          <div className={styles.metricContent}>
            <div className={styles.metricValue}>{formatNumber(metrics.active_tasks)}</div>
            <div className={styles.metricLabel}>Active Tasks</div>
          </div>
        </div>

        <div className={styles.metricCard}>
          <CheckCircle className={styles.metricIcon} size={24} />
          <div className={styles.metricContent}>
            <div className={styles.metricValue}>{formatNumber(metrics.completed_tasks)}</div>
            <div className={styles.metricLabel}>Completed</div>
          </div>
        </div>

        <div className={styles.metricCard}>
          <XCircle className={styles.metricIcon} size={24} />
          <div className={styles.metricContent}>
            <div className={styles.metricValue}>{formatNumber(metrics.failed_tasks)}</div>
            <div className={styles.metricLabel}>Failed</div>
          </div>
        </div>

        <div className={styles.metricCard}>
          <Clock className={styles.metricIcon} size={24} />
          <div className={styles.metricContent}>
            <div className={`${styles.metricValue} ${getCompletionTimeColor(metrics.average_completion_time)}`}>
              {formatDuration(metrics.average_completion_time)}
            </div>
            <div className={styles.metricLabel}>Avg Completion Time</div>
          </div>
        </div>

        <div className={styles.metricCard}>
          <Target className={styles.metricIcon} size={24} />
          <div className={styles.metricContent}>
            <div className={`${styles.metricValue} ${getSuccessRateColor(metrics.success_rate)}`}>
              {formatPercentage(metrics.success_rate)}
            </div>
            <div className={styles.metricLabel}>Success Rate</div>
          </div>
        </div>
      </div>

      <div className={styles.summary}>
        <div className={styles.summaryItem}>
          <span className={styles.summaryLabel}>Active Rate:</span>
          <span className={styles.summaryValue}>
            {formatPercentage(
              metrics.total_tasks && metrics.total_tasks > 0 
                ? (metrics.active_tasks / metrics.total_tasks) * 100 
                : 0
            )}
          </span>
        </div>
        <div className={styles.summaryItem}>
          <span className={styles.summaryLabel}>Completion Rate:</span>
          <span className={styles.summaryValue}>
            {formatPercentage(
              metrics.total_tasks && metrics.total_tasks > 0 
                ? (metrics.completed_tasks / metrics.total_tasks) * 100 
                : 0
            )}
          </span>
        </div>
        <div className={styles.summaryItem}>
          <span className={styles.summaryLabel}>Failure Rate:</span>
          <span className={styles.summaryValue}>
            {formatPercentage(
              metrics.total_tasks && metrics.total_tasks > 0 
                ? (metrics.failed_tasks / metrics.total_tasks) * 100 
                : 0
            )}
          </span>
        </div>
      </div>
    </div>
  );
}
