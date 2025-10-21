"use client";

import React, { useState, useEffect } from "react";
import { SystemHealthOverviewProps, SystemHealth } from "@/types/metrics";
import { metricsApiClient, MetricsApiError } from "@/lib/metrics-api";
import MetricTile from "./MetricTile";
import styles from "./SystemHealthOverview.module.scss";

export default function SystemHealthOverview({
  healthStatus: externalHealthStatus,
  isLoading: externalLoading,
  error: externalError,
  onRetry,
}: SystemHealthOverviewProps) {
  const [healthStatus, setHealthStatus] = useState<SystemHealth | null>(
    externalHealthStatus ?? null
  );
  const [isLoading, setIsLoading] = useState(
    externalLoading ?? !externalHealthStatus
  );
  const [error, setError] = useState<string | null>(externalError ?? null);

  // Load health status if not provided externally
  const loadHealthStatus = async () => {
    if (externalHealthStatus) return; // Use external data if provided

    try {
      setIsLoading(true);
      setError(null);

      const health = await metricsApiClient.getSystemHealth();
      setHealthStatus(health);
    } catch (err) {
      const errorMessage =
        err instanceof MetricsApiError
          ? err.message
          : "Failed to load system health";

      setError(errorMessage);
      console.error("Failed to load system health:", err);
    } finally {
      setIsLoading(false);
    }
  };

  // Initial load and external prop updates
  useEffect(() => {
    if (externalHealthStatus) {
      setHealthStatus(externalHealthStatus);
      setIsLoading(false);
      setError(null);
    } else if (!externalHealthStatus && !isLoading) {
      loadHealthStatus();
    }
  }, [externalHealthStatus]);

  // Format uptime duration
  const formatUptime = (seconds: number): string => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (days > 0) {
      return `${days}d ${hours}h ${minutes}m`;
    } else if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  };

  // Get status color class
  const getStatusColor = (status: string) => {
    switch (status) {
      case "healthy":
        return styles.healthy;
      case "degraded":
        return styles.degraded;
      case "unhealthy":
        return styles.unhealthy;
      default:
        return styles.unknown;
    }
  };

  // Get component status icon
  const getComponentIcon = (status: string) => {
    switch (status) {
      case "healthy":
        return "✅";
      case "degraded":
        return "⚠️";
      case "unhealthy":
        return "❌";
      default:
        return "❓";
    }
  };

  if (isLoading) {
    return (
      <div className={styles.healthOverview}>
        <div className={styles.loading}>
          <div className={styles.loadingSpinner}></div>
          <p>Loading system health...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.healthOverview}>
        <div className={styles.error}>
          <h3>Failed to load system health</h3>
          <p>{error}</p>
          <button
            onClick={() => {
              loadHealthStatus();
              onRetry?.();
            }}
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (!healthStatus) {
    return (
      <div className={styles.healthOverview}>
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>🏥</div>
          <h3>System Health Monitoring</h3>
          <p>Real-time health monitoring requires V3 system health APIs.</p>
          <div className={styles.emptyActions}>
            <button className={styles.secondaryButton} disabled>
              Connect to Health API
            </button>
          </div>
        </div>
      </div>
    );
  }

  const healthyComponents = healthStatus.components.filter(
    (c) => c.status === "healthy"
  ).length;
  const totalComponents = healthStatus.components.length;
  const activeAlerts = healthStatus.alerts.filter((a) => !a.resolved).length;

  return (
    <div className={styles.healthOverview}>
      <div className={styles.header}>
        <h2>System Health Overview</h2>
        <div
          className={`${styles.overallStatus} ${getStatusColor(
            healthStatus.status
          )}`}
        >
          <span className={styles.statusDot}></span>
          <span className={styles.statusText}>
            {healthStatus.status.charAt(0).toUpperCase() +
              healthStatus.status.slice(1)}
          </span>
        </div>
      </div>

      <div className={styles.metricsGrid}>
        <MetricTile
          title="System Uptime"
          value={formatUptime(healthStatus.uptime_seconds)}
          status="success"
          icon="⏱️"
        />

        <MetricTile
          title="Components"
          value={`${healthyComponents}/${totalComponents}`}
          status={
            healthyComponents === totalComponents
              ? "success"
              : healthyComponents > totalComponents / 2
              ? "warning"
              : "error"
          }
          icon="🔧"
        />

        <MetricTile
          title="Active Alerts"
          value={activeAlerts}
          status={activeAlerts > 0 ? "warning" : "success"}
          icon="🚨"
        />

        <MetricTile
          title="Version"
          value={healthStatus.version}
          status="neutral"
          icon="🏷️"
        />
      </div>

      <div className={styles.componentsSection}>
        <h3>Component Status</h3>
        <div className={styles.componentsGrid}>
          {healthStatus.components.map((component) => (
            <div
              key={component.name}
              className={`${styles.componentCard} ${getStatusColor(
                component.status
              )}`}
            >
              <div className={styles.componentHeader}>
                <span className={styles.componentIcon}>
                  {getComponentIcon(component.status)}
                </span>
                <h4 className={styles.componentName}>{component.name}</h4>
              </div>

              <div className={styles.componentMetrics}>
                {component.response_time_ms && (
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>Response Time</span>
                    <span className={styles.metricValue}>
                      {component.response_time_ms}ms
                    </span>
                  </div>
                )}

                {component.error_rate !== undefined && (
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>Error Rate</span>
                    <span className={styles.metricValue}>
                      {(component.error_rate * 100).toFixed(1)}%
                    </span>
                  </div>
                )}

                <div className={styles.metric}>
                  <span className={styles.metricLabel}>Last Check</span>
                  <span className={styles.metricValue}>
                    {new Date(component.last_check).toLocaleTimeString()}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {healthStatus.alerts.length > 0 && (
        <div className={styles.alertsSection}>
          <h3>Recent Alerts</h3>
          <div className={styles.alertsList}>
            {healthStatus.alerts.slice(0, 5).map((alert) => (
              <div
                key={alert.id}
                className={`${styles.alertItem} ${styles[alert.severity]}`}
              >
                <div className={styles.alertHeader}>
                  <span className={styles.alertSeverity}>
                    {alert.severity.toUpperCase()}
                  </span>
                  <span className={styles.alertTime}>
                    {new Date(alert.timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <h4 className={styles.alertTitle}>{alert.title}</h4>
                <p className={styles.alertDescription}>{alert.description}</p>
                {alert.component && (
                  <span className={styles.alertComponent}>
                    {alert.component}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
