"use client";

import React, { useState, useEffect } from "react";
import styles from "./SystemResourcesMonitor.module.scss";

interface SystemResources {
  cpu_usage_percent: number;
  memory_usage_percent: number;
  active_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  avg_response_time_ms: number;
}

interface SystemResourcesMonitorProps {
  resources?: SystemResources;
  isLoading?: boolean;
  error?: string | null;
}

export default function SystemResourcesMonitor({
  resources: externalResources,
  isLoading: externalLoading,
  error: externalError,
}: SystemResourcesMonitorProps) {
  const [resources, setResources] = useState<SystemResources | null>(
    externalResources ?? null
  );
  const [isLoading, setIsLoading] = useState(externalLoading ?? !externalResources);
  const [error, setError] = useState<string | null>(externalError ?? null);

  // Update local state when external props change
  useEffect(() => {
    if (externalResources !== undefined) {
      setResources(externalResources);
      setIsLoading(false);
    }
    if (externalError !== undefined) {
      setError(externalError);
    }
    if (externalLoading !== undefined) {
      setIsLoading(externalLoading);
    }
  }, [externalResources, externalError, externalLoading]);

  const getStatusColor = (value: number, thresholds: { warning: number; error: number }) => {
    if (value >= thresholds.error) return "#ef4444"; // red-500
    if (value >= thresholds.warning) return "#f59e0b"; // amber-500
    return "#10b981"; // emerald-500
  };

  const getProgressBarColor = (percentage: number) => {
    if (percentage >= 90) return "#ef4444"; // red
    if (percentage >= 70) return "#f59e0b"; // amber
    return "#10b981"; // green
  };

  if (isLoading) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <h3>System Resources</h3>
        </div>
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading system resources...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <h3>System Resources</h3>
        </div>
        <div className={styles.error}>
          <p>⚠️ {error}</p>
        </div>
      </div>
    );
  }

  if (!resources) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <h3>System Resources</h3>
        </div>
        <div className={styles.noData}>
          <p>No system resource data available</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h3>System Resources</h3>
        <span className={styles.timestamp}>
          Last updated: {new Date().toLocaleTimeString()}
        </span>
      </div>

      <div className={styles.metricsGrid}>
        {/* CPU Usage */}
        <div className={styles.metric}>
          <div className={styles.metricHeader}>
            <span className={styles.metricIcon}>🖥️</span>
            <span className={styles.metricLabel}>CPU Usage</span>
            <span
              className={styles.metricValue}
              style={{ color: getStatusColor(resources.cpu_usage_percent, { warning: 70, error: 90 }) }}
            >
              {resources.cpu_usage_percent.toFixed(1)}%
            </span>
          </div>
          <div className={styles.progressBar}>
            <div
              className={styles.progressFill}
              style={{
                width: `${Math.min(resources.cpu_usage_percent, 100)}%`,
                backgroundColor: getProgressBarColor(resources.cpu_usage_percent),
              }}
            ></div>
          </div>
        </div>

        {/* Memory Usage */}
        <div className={styles.metric}>
          <div className={styles.metricHeader}>
            <span className={styles.metricIcon}>🧠</span>
            <span className={styles.metricLabel}>Memory Usage</span>
            <span
              className={styles.metricValue}
              style={{ color: getStatusColor(resources.memory_usage_percent, { warning: 75, error: 90 }) }}
            >
              {resources.memory_usage_percent.toFixed(1)}%
            </span>
          </div>
          <div className={styles.progressBar}>
            <div
              className={styles.progressFill}
              style={{
                width: `${Math.min(resources.memory_usage_percent, 100)}%`,
                backgroundColor: getProgressBarColor(resources.memory_usage_percent),
              }}
            ></div>
          </div>
        </div>

        {/* Task Statistics */}
        <div className={styles.metric}>
          <div className={styles.metricHeader}>
            <span className={styles.metricIcon}>📊</span>
            <span className={styles.metricLabel}>Active Tasks</span>
            <span className={styles.metricValue}>
              {resources.active_tasks}
            </span>
          </div>
          <div className={styles.taskStats}>
            <div className={styles.taskStat}>
              <span>Completed:</span>
              <span className={styles.successText}>{resources.completed_tasks}</span>
            </div>
            <div className={styles.taskStat}>
              <span>Failed:</span>
              <span className={styles.errorText}>{resources.failed_tasks}</span>
            </div>
          </div>
        </div>

        {/* Response Time */}
        <div className={styles.metric}>
          <div className={styles.metricHeader}>
            <span className={styles.metricIcon}>⚡</span>
            <span className={styles.metricLabel}>Avg Response Time</span>
            <span
              className={styles.metricValue}
              style={{ color: getStatusColor(resources.avg_response_time_ms, { warning: 1000, error: 2000 }) }}
            >
              {resources.avg_response_time_ms.toFixed(0)}ms
            </span>
          </div>
          <div className={styles.responseTimeIndicator}>
            <div className={styles.responseTimeBar}>
              <div
                className={styles.responseTimeFill}
                style={{
                  width: `${Math.min((resources.avg_response_time_ms / 2000) * 100, 100)}%`,
                  backgroundColor: getStatusColor(resources.avg_response_time_ms, { warning: 1000, error: 2000 }),
                }}
              ></div>
            </div>
          </div>
        </div>
      </div>

      {/* System Status Summary */}
      <div className={styles.statusSummary}>
        <div className={styles.statusItem}>
          <span className={styles.statusLabel}>Overall Status:</span>
          <span className={`${styles.statusBadge} ${
            resources.cpu_usage_percent < 70 && resources.memory_usage_percent < 75
              ? styles.statusHealthy
              : resources.cpu_usage_percent < 90 && resources.memory_usage_percent < 90
              ? styles.statusWarning
              : styles.statusError
          }`}>
            {resources.cpu_usage_percent < 70 && resources.memory_usage_percent < 75
              ? "Healthy"
              : resources.cpu_usage_percent < 90 && resources.memory_usage_percent < 90
              ? "Warning"
              : "Critical"}
          </span>
        </div>

        <div className={styles.statusItem}>
          <span className={styles.statusLabel}>Task Success Rate:</span>
          <span className={styles.statusValue}>
            {resources.completed_tasks + resources.failed_tasks > 0
              ? ((resources.completed_tasks / (resources.completed_tasks + resources.failed_tasks)) * 100).toFixed(1)
              : "100.0"}%
          </span>
        </div>
      </div>
    </div>
  );
}
