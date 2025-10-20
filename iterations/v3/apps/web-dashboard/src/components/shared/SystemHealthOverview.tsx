"use client";

import styles from "./SystemHealthOverview.module.scss";

interface HealthStatus {
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  timestamp: string;
  version?: string;
  uptime?: number;
}

interface SystemHealthOverviewProps {
  healthStatus: HealthStatus | null;
  isLoading: boolean;
  error: string | null;
  onRetry: () => void;
}

export default function SystemHealthOverview({
  healthStatus,
  isLoading,
  error,
  onRetry,
}: SystemHealthOverviewProps) {
  const getStatusColor = (status: string | undefined) => {
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

  const getStatusDescription = (status: string | undefined) => {
    switch (status) {
      case "healthy":
        return "All systems operational";
      case "degraded":
        return "Some systems experiencing issues";
      case "unhealthy":
        return "Critical system failures detected";
      default:
        return "Unable to determine system status";
    }
  };

  if (isLoading) {
    return (
      <div className={styles.healthCard}>
        <div className={styles.header}>
          <h3>System Health</h3>
          <div className={`${styles.statusIndicator} ${styles.loading}`}>
            <div className={styles.loadingSpinner}></div>
            <span>Checking...</span>
          </div>
        </div>

        <div className={styles.content}>
          <div className={styles.loadingSkeleton}></div>
          <div className={`${styles.loadingSkeleton} ${styles.large}`}></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.healthCard}>
        <div className={styles.header}>
          <h3>System Health</h3>
          <div className={`${styles.statusIndicator} ${styles.error}`}>
            <span>Connection Error</span>
          </div>
        </div>

        <div className={styles.content}>
          <div className={styles.errorMessage}>
            <p>{error}</p>
            <button
              className={styles.retryButton}
              onClick={onRetry}
              aria-label="Retry health check"
            >
              Retry Connection
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.healthCard}>
      <div className={styles.header}>
        <h3>System Health</h3>
        <div
          className={`${styles.statusIndicator} ${getStatusColor(
            healthStatus?.status
          )}`}
        >
          <div className={styles.statusDot} aria-hidden="true"></div>
          <span>
            {healthStatus?.status
              ? healthStatus.status.charAt(0).toUpperCase() +
                healthStatus.status.slice(1)
              : "Unknown"}
          </span>
        </div>
      </div>

      <div className={styles.content}>
        <p className={styles.description}>
          {getStatusDescription(healthStatus?.status)}
        </p>

        <div className={styles.metrics}>
          <div className={styles.metric}>
            <span className={styles.metricLabel}>Dashboard Version</span>
            <span className={styles.metricValue}>
              v{healthStatus?.version ?? "0.1.0"}
            </span>
          </div>

          {healthStatus?.uptime && (
            <div className={styles.metric}>
              <span className={styles.metricLabel}>Uptime</span>
              <span className={styles.metricValue}>
                {formatUptime(healthStatus.uptime)}
              </span>
            </div>
          )}

          <div className={styles.metric}>
            <span className={styles.metricLabel}>Last Check</span>
            <span className={styles.metricValue}>
              {healthStatus?.timestamp
                ? new Date(healthStatus.timestamp).toLocaleString()
                : "Never"}
            </span>
          </div>
        </div>

        <div className={styles.actions}>
          <button
            className={styles.refreshButton}
            onClick={onRetry}
            aria-label="Refresh health status"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
              <path d="M21 3v5h-5" />
              <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
              <path d="M8 16H3v5" />
            </svg>
            Refresh
          </button>
        </div>
      </div>
    </div>
  );
}

function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m ${secs}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}
