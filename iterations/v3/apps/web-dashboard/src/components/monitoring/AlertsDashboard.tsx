"use client";

import React, { useState, useEffect, useCallback } from "react";
import styles from "./AlertsDashboard.module.scss";

interface Alert {
  id: string;
  definition_id: string;
  title: string;
  description: string;
  severity: "info" | "warning" | "error" | "critical";
  category: string;
  status: "active" | "acknowledged" | "resolved" | "suppressed";
  triggered_at: string;
  updated_at: string;
  current_value?: number;
  threshold_value?: number;
  affected_services: string[];
  labels: Record<string, string>;
  occurrence_count: number;
  escalation_level: number;
}

interface AlertStatistics {
  total_active_alerts: number;
  alerts_by_severity: Record<string, number>;
  alerts_by_category: Record<string, number>;
  alerts_by_status: Record<string, number>;
  recent_history_count: number;
  average_resolution_time_minutes: number;
}

interface AlertsDashboardProps {
  refreshInterval?: number;
}

export default function AlertsDashboard({ refreshInterval = 30000 }: AlertsDashboardProps) {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [statistics, setStatistics] = useState<AlertStatistics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedSeverity, setSelectedSeverity] = useState<string>("all");
  const [selectedStatus, setSelectedStatus] = useState<string>("all");

  // Fetch alerts data
  const fetchAlerts = useCallback(async () => {
    // try {
    //   setError(null);

    //   const [alertsResponse, statsResponse] = await Promise.all([
    //     fetch("/api/alerts"),
    //     fetch("/api/alerts/statistics")
    //   ]);

    //   if (!alertsResponse.ok) {
    //     throw new Error(`Failed to fetch alerts: ${alertsResponse.status}`);
    //   }

    //   if (!statsResponse.ok) {
    //     throw new Error(`Failed to fetch statistics: ${statsResponse.status}`);
    //   }

    //   const alertsData = await alertsResponse.json();
    //   const statsData = await statsResponse.json();

    //   setAlerts(alertsData.alerts || []);
    //   setStatistics(statsData.statistics || null);
    // } catch (err) {
    //   setError(err instanceof Error ? err.message : "Failed to fetch alerts");
    //   console.error("Failed to fetch alerts:", err);
    // } finally {
    //   setLoading(false);
    // }
  }, []);

  // Acknowledge alert
  const acknowledgeAlert = useCallback(async (alertId: string) => {
    try {
      const response = await fetch(`/api/alerts/${alertId}/acknowledge`, {
        method: "POST",
      });

      if (!response.ok) {
        throw new Error(`Failed to acknowledge alert: ${response.status}`);
      }

      // Refresh alerts after acknowledgment
      await fetchAlerts();
    } catch (err) {
      console.error("Failed to acknowledge alert:", err);
      // Could show a toast notification here
    }
  }, [fetchAlerts]);

  // Resolve alert
  const resolveAlert = useCallback(async (alertId: string) => {
    try {
      const response = await fetch(`/api/alerts/${alertId}/resolve`, {
        method: "POST",
      });

      if (!response.ok) {
        throw new Error(`Failed to resolve alert: ${response.status}`);
      }

      // Refresh alerts after resolution
      await fetchAlerts();
    } catch (err) {
      console.error("Failed to resolve alert:", err);
      // Could show a toast notification here
    }
  }, [fetchAlerts]);

  // Filter alerts based on selections
  const filteredAlerts = alerts.filter(alert => {
    if (selectedSeverity !== "all" && alert.severity !== selectedSeverity) return false;
    if (selectedStatus !== "all" && alert.status !== selectedStatus) return false;
    return true;
  });


  // Get severity icon
  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case "critical": return "🚨";
      case "error": return "❌";
      case "warning": return "⚠️";
      case "info": return "ℹ️";
      default: return "📢";
    }
  };

  // Format relative time
  const formatRelativeTime = (timestamp: string) => {
    const now = new Date();
    const time = new Date(timestamp);
    const diffMs = now.getTime() - time.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${diffDays}d ago`;
  };

  // Initial load and refresh interval
  useEffect(() => {
    fetchAlerts();

    const interval = setInterval(fetchAlerts, refreshInterval);
    return () => clearInterval(interval);
  }, [fetchAlerts, refreshInterval]);

  if (loading) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <h2>Alert Management</h2>
        </div>
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading alerts...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <h2>Alert Management</h2>
        </div>
        <div className={styles.error}>
          <p>❌ {error}</p>
          <button onClick={fetchAlerts} className={styles.retryButton}>
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2>Alert Management</h2>
        <div className={styles.headerActions}>
          <button onClick={fetchAlerts} className={styles.refreshButton}>
            🔄 Refresh
          </button>
        </div>
      </div>

      {/* Statistics Overview */}
      {statistics && (
        <div className={styles.statsOverview}>
          <div className={styles.statCard}>
            <div className={styles.statValue}>{statistics.total_active_alerts}</div>
            <div className={styles.statLabel}>Active Alerts</div>
          </div>
          <div className={styles.statCard}>
            <div className={styles.statValue}>
              {statistics.average_resolution_time_minutes > 0
                ? `${statistics.average_resolution_time_minutes.toFixed(1)}m`
                : "N/A"}
            </div>
            <div className={styles.statLabel}>Avg Resolution Time</div>
          </div>
          <div className={styles.statCard}>
            <div className={styles.statValue}>{statistics.recent_history_count}</div>
            <div className={styles.statLabel}>Recent Events (24h)</div>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className={styles.filters}>
        <div className={styles.filterGroup}>
          <label>Severity:</label>
          <select
            value={selectedSeverity}
            onChange={(e) => setSelectedSeverity(e.target.value)}
          >
            <option value="all">All</option>
            <option value="critical">Critical</option>
            <option value="error">Error</option>
            <option value="warning">Warning</option>
            <option value="info">Info</option>
          </select>
        </div>

        <div className={styles.filterGroup}>
          <label>Status:</label>
          <select
            value={selectedStatus}
            onChange={(e) => setSelectedStatus(e.target.value)}
          >
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="acknowledged">Acknowledged</option>
            <option value="resolved">Resolved</option>
            <option value="suppressed">Suppressed</option>
          </select>
        </div>
      </div>

      {/* Alerts List */}
      <div className={styles.alertsList}>
          {filteredAlerts.length === 0 ? (
            <div className={styles.emptyState}>
              <div className={styles.emptyIcon}>🔔</div>
              <h3>No Alerts Found</h3>
              <p>
                System alerts help you monitor infrastructure health and application performance.
                When connected to your API server, you'll see:
              </p>
              <ul className={styles.emptyFeatures}>
                <li>🚨 <strong>Critical alerts</strong> - System failures and service outages</li>
                <li>⚠️ <strong>Warning alerts</strong> - Performance degradation and capacity issues</li>
                <li>📊 <strong>Metric alerts</strong> - Threshold violations and anomalies</li>
                <li>🔧 <strong>Maintenance alerts</strong> - Scheduled downtime and updates</li>
              </ul>
              <div className={styles.emptyNote}>
                <span className={styles.noteIcon}>ℹ️</span>
                <span>No alerts means your system is running smoothly!</span>
              </div>
            </div>
          ) : (
          filteredAlerts.map((alert) => (
            <div key={alert.id} className={`${styles.alertCard} ${styles[alert.severity]}`}>
              <div className={styles.alertHeader}>
                <div className={styles.alertTitle}>
                  <span className={styles.severityIcon}>
                    {getSeverityIcon(alert.severity)}
                  </span>
                  <span className={styles.title}>{alert.title}</span>
                  <span className={styles.category}>{alert.category}</span>
                </div>

                <div className={styles.alertMeta}>
                  <span className={`${styles.status} ${styles[alert.status]}`}>
                    {alert.status}
                  </span>
                  <span className={styles.time}>
                    {formatRelativeTime(alert.triggered_at)}
                  </span>
                </div>
              </div>

              <div className={styles.alertBody}>
                <p className={styles.description}>{alert.description}</p>

                {alert.affected_services.length > 0 && (
                  <div className={styles.affectedServices}>
                    <span className={styles.label}>Affected:</span>
                    {alert.affected_services.map((service) => (
                      <span key={service} className={styles.serviceTag}>
                        {service}
                      </span>
                    ))}
                  </div>
                )}

                <div className={styles.alertMetrics}>
                  {alert.current_value !== undefined && (
                    <div className={styles.metric}>
                      <span>Current:</span>
                      <span>{alert.current_value}</span>
                    </div>
                  )}
                  {alert.threshold_value !== undefined && (
                    <div className={styles.metric}>
                      <span>Threshold:</span>
                      <span>{alert.threshold_value}</span>
                    </div>
                  )}
                  <div className={styles.metric}>
                    <span>Occurrences:</span>
                    <span>{alert.occurrence_count}</span>
                  </div>
                  {alert.escalation_level > 0 && (
                    <div className={styles.metric}>
                      <span>Escalation:</span>
                      <span>Level {alert.escalation_level}</span>
                    </div>
                  )}
                </div>
              </div>

              <div className={styles.alertActions}>
                {alert.status === "active" && (
                  <>
                    <button
                      onClick={() => acknowledgeAlert(alert.id)}
                      className={`${styles.actionButton} ${styles.acknowledge}`}
                    >
                      ✅ Acknowledge
                    </button>
                    <button
                      onClick={() => resolveAlert(alert.id)}
                      className={`${styles.actionButton} ${styles.resolve}`}
                    >
                      ✅ Resolve
                    </button>
                  </>
                )}
                <button className={`${styles.actionButton} ${styles.details}`}>
                  📋 Details
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Footer with last update time */}
      <div className={styles.footer}>
        <span>Last updated: {new Date().toLocaleTimeString()}</span>
        <span>• Auto-refresh: {refreshInterval / 1000}s</span>
      </div>
    </div>
  );
}
