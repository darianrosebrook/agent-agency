"use client";

import React from "react";
import Header from "@/components/shared/Header";
import Navigation from "@/components/shared/Navigation";
import TaskMetrics from "@/components/tasks/TaskMetrics";
import { TaskApiClient } from "@/lib/task-api";
import { TaskMetrics as TaskMetricsType } from "@/types/tasks";
import styles from "./page.module.scss";

export default function DashboardPage() {
  const [metrics, setMetrics] = React.useState<TaskMetricsType | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  const taskApi = new TaskApiClient();

  React.useEffect(() => {
    const fetchMetrics = async () => {
      try {
        setError(null);
        const metricsData = await taskApi.getTaskMetrics();
        setMetrics(metricsData);
      } catch (err) {
        console.error("Failed to fetch metrics:", err);
        setError(err instanceof Error ? err.message : "Failed to load metrics");
      } finally {
        setLoading(false);
      }
    };

    fetchMetrics();
  }, []);

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

        {error && (
          <div className={styles.error}>
            <p>{error}</p>
            <button onClick={() => setError(null)}>Dismiss</button>
          </div>
        )}

        {metrics && (
          <div className={styles.metricsSection}>
            <TaskMetrics metrics={metrics} />
          </div>
        )}

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
                <span className={styles.statusValue}>🟢 Online</span>
              </div>
              <div className={styles.statusItem}>
                <span className={styles.statusLabel}>Database</span>
                <span className={styles.statusValue}>🟢 Connected</span>
              </div>
              <div className={styles.statusItem}>
                <span className={styles.statusLabel}>Workers</span>
                <span className={styles.statusValue}>🟡 Simulated</span>
              </div>
              <div className={styles.statusItem}>
                <span className={styles.statusLabel}>Health Monitor</span>
                <span className={styles.statusValue}>🟢 Active</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}