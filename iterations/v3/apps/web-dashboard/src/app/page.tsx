"use client";

import React from "react";
import Header from "@/components/shared/Header";
import Navigation from "@/components/shared/Navigation";
import TaskMetrics from "@/components/tasks/TaskMetrics";
import SLODashboard from "@/components/monitoring/SLODashboard";
import SLOAlertsDashboard from "@/components/monitoring/SLOAlertsDashboard";
import { TaskApiClient } from "@/lib/task-api";
import { TaskMetrics as TaskMetricsType } from "@/types/tasks";
import styles from "./page.module.scss";

export default function DashboardPage() {
  const [metrics] = React.useState<TaskMetricsType | null>(null);
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [apiConnected, setApiConnected] = React.useState(false);

  // const taskApi = new TaskApiClient(); // Commented out - no API calls

  React.useEffect(() => {
    const fetchMetrics = async () => {
      try {
        setError(null);
        setLoading(true);
        setApiConnected(false);

        // const metricsData = await taskApi.getTaskMetrics();
        // setMetrics(metricsData);
        // setApiConnected(true);
        setLoading(false);
        setApiConnected(false);
      } catch (err) {
        console.error("Failed to fetch metrics:", err);
        let errorMessage = "Failed to load metrics";
        
        if (err instanceof Error) {
          if (err.message.includes("fetch") || err.message.includes("NetworkError")) {
            errorMessage = "API server is not available. Please ensure the backend server is running on port 8080.";
          } else {
            errorMessage = err.message;
          }
        }
        
        setError(errorMessage);
        setApiConnected(false);
        setLoading(false);
      }
    };

    fetchMetrics();
  }, []);

  const handleRetry = () => {
    setError(null);
    setLoading(true);
    setApiConnected(false);
    
    const fetchMetrics = async () => {
      try {
        // const metricsData = await taskApi.getTaskMetrics();
        // setMetrics(metricsData);
        // setApiConnected(true);
        setLoading(false);
        setApiConnected(false);
      } catch (err) {
        console.error("Retry failed:", err);
        let errorMessage = "Failed to load metrics";
        
        if (err instanceof Error) {
          if (err.message.includes("fetch") || err.message.includes("NetworkError")) {
            errorMessage = "API server is not available. Please ensure the backend server is running on port 8080.";
          } else {
            errorMessage = err.message;
          }
        }
        
        setError(errorMessage);
        setApiConnected(false);
        setLoading(false);
      }
    };

    fetchMetrics();
  };

  if (loading) {
    return (
      <div className={styles.page}>
        <Header />
        <Navigation />
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading dashboard...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <Header />
      <Navigation />
      
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Dashboard</h1>
          <p className={styles.subtitle}>
            Welcome to Agent Agency V3. Monitor task execution and system health.
          </p>
        </div>

        {/* Error State */}
        {error && (
          <div className={styles.error}>
            <div className={styles.errorIcon}>⚠️</div>
            <div className={styles.errorContent}>
              <h3>Connection Error</h3>
              <p>{error}</p>
              <div className={styles.errorActions}>
                <button onClick={handleRetry} className={styles.retryButton}>
                  🔄 Retry Connection
                </button>
                <button onClick={() => setError(null)} className={styles.dismissButton}>
                  ✕ Dismiss
                </button>
              </div>
            </div>
          </div>
        )}

        {/* API Status Indicator */}
        <div className={styles.apiStatus}>
          <div className={styles.statusIndicator}>
            <span className={styles.statusIcon}>
              {apiConnected ? "🟢" : "🔴"}
            </span>
            <span className={styles.statusText}>
              API Server: {apiConnected ? "Connected" : "Disconnected"}
            </span>
          </div>
        </div>

        {/* Metrics Section - Show if connected or with fallback */}
        <div className={styles.metricsSection}>
          {apiConnected && metrics ? (
            <TaskMetrics metrics={metrics} />
          ) : (
            <div className={styles.emptyState}>
              <div className={styles.emptyIcon}>📊</div>
              <h3>No Metrics Available</h3>
              <p>
                {apiConnected 
                  ? "No task metrics data available at this time."
                  : "Connect to API server to view real-time metrics."
                }
              </p>
              {!apiConnected && (
                <button onClick={handleRetry} className={styles.connectButton}>
                  🔗 Connect to API
                </button>
              )}
            </div>
          )}
        </div>

        {/* SLO Dashboard - Always show with fallback */}
        <div className={styles.sloSection}>
          <SLODashboard />
        </div>

        {/* Alerts Dashboard - Always show with fallback */}
        <div className={styles.alertsSection}>
          <SLOAlertsDashboard />
        </div>

        <div className={styles.content}>
          <div className={styles.card}>
            <h2>Quick Actions</h2>
            <div className={styles.actions}>
              <a href="/tasks" className={styles.actionButton}>
                <span className={styles.actionIcon}>📋</span>
                <span className={styles.actionText}>View Tasks</span>
              </a>
              <a href="/chat" className={styles.actionButton}>
                <span className={styles.actionIcon}>💬</span>
                <span className={styles.actionText}>Start Chat</span>
              </a>
              <a href="/metrics" className={styles.actionButton}>
                <span className={styles.actionIcon}>📊</span>
                <span className={styles.actionText}>View Metrics</span>
              </a>
              <a href="/settings" className={styles.actionButton}>
                <span className={styles.actionIcon}>⚙️</span>
                <span className={styles.actionText}>Settings</span>
              </a>
            </div>
          </div>

          <div className={styles.card}>
            <h2>System Status</h2>
            <div className={styles.status}>
              <div className={styles.statusItem}>
                <span className={styles.statusLabel}>API Server</span>
                <span className={styles.statusValue}>
                  {apiConnected ? "🟢 Connected" : "🔴 Disconnected"}
                </span>
              </div>
              <div className={styles.statusItem}>
                <span className={styles.statusLabel}>Database</span>
                <span className={styles.statusValue}>
                  {apiConnected ? "🟢 Connected" : "🟡 Unknown"}
                </span>
                {/* STATIC: Database status inferred from API connection */}
              </div>
              <div className={styles.statusItem}>
                <span className={styles.statusLabel}>Workers</span>
                <span className={styles.statusValue}>
                  {apiConnected ? "🟢 Active" : "🟡 Simulated"}
                </span>
                {/* STATIC: Worker status inferred from API connection */}
              </div>
              <div className={styles.statusItem}>
                <span className={styles.statusLabel}>Health Monitor</span>
                <span className={styles.statusValue}>
                  {apiConnected ? "🟢 Active" : "🟡 Limited"}
                </span>
                {/* STATIC: Health monitor status inferred from API connection */}
              </div>
            </div>
            {!apiConnected && (
              <div className={styles.statusNote}>
                <p>⚠️ Running in offline mode. Some features may be limited.</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}