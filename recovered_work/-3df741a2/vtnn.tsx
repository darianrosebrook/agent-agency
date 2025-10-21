"use client";

import React, { useState, useEffect, useCallback } from "react";
import styles from "./SLOAlertsDashboard.module.scss";

interface SLOAlert {
  id: string;
  slo_name: string;
  title: string;
  description: string;
  severity: "info" | "warning" | "error" | "critical";
  status: "active" | "acknowledged" | "resolved";
  current_value: number;
  threshold_value: number;
  triggered_at: string;
  labels: Record<string, string>;
}

interface SLOAlertsDashboardProps {
  refreshInterval?: number;
}

export default function SLOAlertsDashboard({ refreshInterval = 30000 }: SLOAlertsDashboardProps) {
  const [alerts, setAlerts] = useState<SLOAlert[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedSeverity, setSelectedSeverity] = useState<string>("all");
  const [selectedStatus, setSelectedStatus] = useState<string>("all");

  // Fetch SLO alerts data
  const fetchAlerts = useCallback(async () => {
    try {
      setError(null);

      const [alertsResponse] = await Promise.all([
        fetch("/api/slo-alerts"),
      ]);

      if (!alertsResponse.ok) {
        throw new Error(`Failed to fetch SLO alerts: ${alertsResponse.status}`);
      }

      const alertsData = await alertsResponse.json();
      setAlerts(alertsData.alerts || []);
      setLoading(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch SLO alerts");
      setLoading(false);
    }
  }, []);

  // Initial data load
  useEffect(() => {
    fetchAlerts();
  }, [fetchAlerts]);

  // Auto-refresh
  useEffect(() => {
    const interval = setInterval(() => {
      fetchAlerts();
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [fetchAlerts, refreshInterval]);

  // Acknowledge alert
  const acknowledgeAlert = useCallback(async (alertId: string) => {
    try {
      const response = await fetch(`/api/slo-alerts/${alertId}/acknowledge`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to acknowledge alert: ${response.status}`);
      }

      // Update local state
      setAlerts(prevAlerts =>
        prevAlerts.map(alert =>
          alert.id === alertId
            ? { ...alert, status: "acknowledged" as const }
            : alert
        )
      );
    } catch (err) {
      console.error("Failed to acknowledge alert:", err);
      // Could show a toast notification here
    }
  }, []);

  // Filter alerts based on selections
  const filteredAlerts = alerts.filter(alert => {
    if (selectedSeverity !== "all" && alert.severity !== selectedSeverity) return false;
    if (selectedStatus !== "all" && alert.status !== selectedStatus) return false;
    return true;
  });

  // Get severity color
  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "critical": return "#dc2626"; // red-600
      case "error": return "#ea580c"; // orange-600
      case "warning": return "#d97706"; // amber-600
      case "info": return "#2563eb"; // blue-600
      default: return "#6b7280"; // gray-500
    }
  };

  // Get status color
  const getStatusColor = (status: string) => {
    switch (status) {
      case "active": return "#dc2626"; // red-600
      case "acknowledged": return "#d97706"; // amber-600
      case "resolved": return "#16a34a"; // green-600
      default: return "#6b7280"; // gray-500
    }
  };

  // Format percentage
  const formatPercent = (value: number) => `${(value * 100).toFixed(1)}%`;

  // Format time ago
  const formatTimeAgo = (timestamp: string) => {
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

  if (loading) {
    return (
      <div className={styles.container}>
        <div className={styles.loading}>
          <div className={styles.spinner} />
          <span>Loading SLO alerts...</span>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2>SLO Alerts</h2>
        <p>Monitor service level objective violations and breaches</p>
      </div>

      {error && (
        <div className={styles.error}>
          <span>⚠️ {error}</span>
          <button onClick={fetchAlerts}>Retry</button>
        </div>
      )}

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
          </select>
        </div>
      </div>

      <div className={styles.alertsList}>
        {filteredAlerts.length === 0 ? (
          <div className={styles.empty}>
            <span>No SLO alerts match the current filters</span>
          </div>
        ) : (
          filteredAlerts.map(alert => (
            <div key={alert.id} className={styles.alertCard}>
              <div className={styles.alertHeader}>
                <div className={styles.titleSection}>
                  <h3>{alert.title}</h3>
                  <div className={styles.badges}>
                    <span
                      className={styles.severityBadge}
                      style={{ backgroundColor: getSeverityColor(alert.severity) }}
                    >
                      {alert.severity.toUpperCase()}
                    </span>
                    <span
                      className={styles.statusBadge}
                      style={{ backgroundColor: getStatusColor(alert.status) }}
                    >
                      {alert.status.toUpperCase()}
                    </span>
                  </div>
                </div>

                <div className={styles.actions}>
                  {alert.status === "active" && (
                    <button
                      className={styles.acknowledgeBtn}
                      onClick={() => acknowledgeAlert(alert.id)}
                    >
                      Acknowledge
                    </button>
                  )}
                </div>
              </div>

              <p className={styles.description}>{alert.description}</p>

              <div className={styles.metrics}>
                <div className={styles.metric}>
                  <span className={styles.label}>Current</span>
                  <span className={styles.value}>{formatPercent(alert.current_value)}</span>
                </div>
                <div className={styles.metric}>
                  <span className={styles.label}>Threshold</span>
                  <span className={styles.value}>{formatPercent(alert.threshold_value)}</span>
                </div>
                <div className={styles.metric}>
                  <span className={styles.label}>SLO</span>
                  <span className={styles.value}>{alert.slo_name}</span>
                </div>
              </div>

              <div className={styles.footer}>
                <span className={styles.timestamp}>
                  {formatTimeAgo(alert.triggered_at)}
                </span>

                {alert.labels && Object.keys(alert.labels).length > 0 && (
                  <div className={styles.labels}>
                    {Object.entries(alert.labels).map(([key, value]) => (
                      <span key={key} className={styles.label}>
                        {key}: {value}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
