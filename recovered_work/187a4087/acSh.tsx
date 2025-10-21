"use client";

import { useState } from "react";
import styles from "./Header.module.scss";

interface HealthStatus {
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  timestamp: string;
  version?: string;
  uptime?: number;
}

interface HeaderProps {
  healthStatus: HealthStatus | null;
  isLoading: boolean;
  error: string | null;
  onRetryHealthCheck: () => void;
}

export default function Header({
  healthStatus,
  isLoading,
  error,
  onRetryHealthCheck,
}: HeaderProps) {
  const [showHealthDetails, setShowHealthDetails] = useState(false);

  const getStatusClass = (status: string | undefined) => {
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

  const getStatusText = (status: string | undefined) => {
    switch (status) {
      case "healthy":
        return "System Healthy";
      case "degraded":
        return "System Degraded";
      case "unhealthy":
        return "System Unhealthy";
      default:
        return "Status Unknown";
    }
  };

  return (
    <header className={styles.header}>
      <div className={styles.headerContent}>
        <div className={styles.logo}>
          <h1 className={styles.title}>Agent Agency V3 Dashboard</h1>
          <span className={styles.subtitle}>
            Real-time monitoring & conversational interface
          </span>
        </div>

        <div className={styles.statusSection}>
          <div className={styles.healthIndicator}>
            <button
              className={`${styles.statusButton} ${getStatusClass(
                healthStatus?.status
              )}`}
              onClick={() => setShowHealthDetails(!showHealthDetails)}
              aria-expanded={showHealthDetails}
              aria-controls="health-details"
            >
              <span className={styles.statusDot} aria-hidden="true"></span>
              <span className={styles.statusText}>
                {isLoading
                  ? "Checking..."
                  : getStatusText(healthStatus?.status ?? "unknown")}
              </span>
              <svg
                className={`${styles.chevron} ${
                  showHealthDetails ? styles.rotated : ""
                }`}
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                aria-hidden="true"
              >
                <path d="M6 9l6 6 6-6" />
              </svg>
            </button>

            {error && (
              <button
                className={styles.retryButton}
                onClick={onRetryHealthCheck}
                title="Retry health check"
                aria-label="Retry health check"
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
              </button>
            )}
          </div>

          {showHealthDetails && (
            <div id="health-details" className={styles.healthDetails}>
              {error ? (
                <div className={styles.errorState}>
                  <h4>Connection Error</h4>
                  <p>{error}</p>
                  <button
                    className={styles.retryButton}
                    onClick={onRetryHealthCheck}
                  >
                    Retry Connection
                  </button>
                </div>
              ) : (
                <div className={styles.healthInfo}>
                  <div className={styles.healthRow}>
                    <span>Status:</span>
                    <span className={getStatusClass(healthStatus?.status)}>
                      {healthStatus?.status ?? "unknown"}
                    </span>
                  </div>
                  <div className={styles.healthRow}>
                    <span>Dashboard:</span>
                    <span>v{healthStatus?.version ?? "0.1.0"}</span>
                  </div>
                  {healthStatus?.uptime && (
                    <div className={styles.healthRow}>
                      <span>Uptime:</span>
                      <span>{Math.floor(healthStatus.uptime)}s</span>
                    </div>
                  )}
                  <div className={styles.healthRow}>
                    <span>Last Check:</span>
                    <span>
                      {healthStatus?.timestamp
                        ? new Date(healthStatus.timestamp).toLocaleTimeString()
                        : "Never"}
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </header>
  );
}
