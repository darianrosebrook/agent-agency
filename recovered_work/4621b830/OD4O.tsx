"use client";

import React, { useState, useEffect, useCallback } from "react";
import {
  AnomalyDetectorProps,
  Anomaly,
  AnalyticsFilters,
} from "@/types/analytics";
import { analyticsApiClient, AnalyticsApiError } from "@/lib/analytics-api";
import styles from "./AnomalyDetector.module.scss";

interface AnomalyDetectorState {
  anomalies: Anomaly[];
  selectedAnomaly: Anomaly | null;
  isDetecting: boolean;
  error: string | null;
  filters: AnalyticsFilters;
  detectionParams: {
    sensitivity: string;
    min_anomaly_score: number;
    time_window_hours: number;
  };
}

export default function AnomalyDetector({
  anomalies: externalAnomalies,
  timeSeriesData,
  onAnomalySelect,
  onAnomalyDismiss,
  filters: externalFilters,
  isDetecting: externalDetecting,
  error: externalError,
}: AnomalyDetectorProps) {
  const [state, setState] = useState<AnomalyDetectorState>({
    anomalies: externalAnomalies ?? [],
    selectedAnomaly: null,
    isDetecting: externalDetecting ?? false,
    error: externalError ?? null,
    filters: externalFilters ?? {
      time_range: {
        start: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(), // 24 hours ago
        end: new Date().toISOString(),
      },
      granularity: "1h",
      anomaly_severity: ["high", "critical"],
    },
    detectionParams: {
      sensitivity: "medium",
      min_anomaly_score: 2.0,
      time_window_hours: 24,
    },
  });

  // Load anomalies if not provided externally
  const loadAnomalies = useCallback(async () => {
    if (externalAnomalies) return;

    // Integrate timeSeriesData for advanced anomaly detection
    if (timeSeriesData && timeSeriesData.length > 0) {
      console.log(
        "Time series data available for anomaly detection:",
        timeSeriesData.length,
        "series"
      );

      // Use time series data to enhance anomaly detection parameters
      const latestSeries = timeSeriesData[timeSeriesData.length - 1];
      if (latestSeries?.data && latestSeries.data.length > 0) {
        // Calculate statistical properties from recent data
        const recentData = latestSeries.data.slice(-50); // Last 50 points
        const values = recentData.map((d) => d.value);

        const mean = values.reduce((a, b) => a + b, 0) / values.length;
        const variance =
          values.reduce((a, b) => a + Math.pow(b - mean, 2), 0) / values.length;
        const stdDev = Math.sqrt(variance);

        // Adjust sensitivity based on data volatility
        const volatilityFactor = stdDev / Math.abs(mean);
        const adjustedSensitivity = Math.max(
          0.1,
          Math.min(0.9, volatilityFactor)
        );

        // Update detection parameters with time series insights
        setState((prev) => ({
          ...prev,
          detectionParams: {
            ...prev.detectionParams,
            sensitivity: adjustedSensitivity < 0.3 ? "low" : adjustedSensitivity < 0.7 ? "medium" : "high",
            min_anomaly_score: stdDev * 2, // 2-sigma threshold
            time_window_hours: Math.max(1, Math.min(24, recentData.length / 4)), // Adaptive window
          },
        }));
      }
    }

    try {
      setState((prev) => ({ ...prev, isDetecting: true, error: null }));
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const response = await analyticsApiClient.getAnomalies(state.filters);
      setState((prev) => ({
        ...prev,
        anomalies: response.anomalies,
        isDetecting: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to load anomaly data";
      setState((prev) => ({
        ...prev,
        isDetecting: false,
        error: errorMessage,
      }));
      console.error("Failed to load anomalies:", error);
    }
  }, [externalAnomalies, state.filters]);

  // Handle anomaly selection
  const handleAnomalySelect = useCallback(
    (anomaly: Anomaly) => {
      setState((prev) => ({ ...prev, selectedAnomaly: anomaly }));
      onAnomalySelect?.(anomaly);
    },
    [onAnomalySelect]
  );

  // Handle anomaly dismissal
  const handleAnomalyDismiss = useCallback(
    async (anomalyId: string) => {
      try {
        await analyticsApiClient.dismissAnomaly(anomalyId);
        setState((prev) => ({
          ...prev,
          anomalies: prev.anomalies.filter((a) => a.id !== anomalyId),
          selectedAnomaly:
            prev.selectedAnomaly?.id === anomalyId
              ? null
              : prev.selectedAnomaly,
        }));
        onAnomalyDismiss?.(anomalyId);
      } catch (error) {
        console.error("Failed to dismiss anomaly:", error);
        // Could show a toast notification here
      }
    },
    [onAnomalyDismiss]
  );

  // Handle filter changes
  const handleFilterChange = useCallback(
    (filterType: keyof AnalyticsFilters, value: any) => {
      setState((prev) => ({
        ...prev,
        filters: {
          ...prev.filters,
          [filterType]: value,
        },
      }));
    },
    []
  );

  // Run anomaly detection
  const runAnomalyDetection = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isDetecting: true, error: null }));

      const detectionRequest = {
        metrics: [], // Will be populated from available metrics
        time_range: state.filters.time_range,
        sensitivity: "medium" as const,
        algorithms: ["zscore", "isolation_forest"],
      };

      const response = await analyticsApiClient.detectAnomalies(
        detectionRequest
      );
      setState((prev) => ({
        ...prev,
        anomalies: response.anomalies,
        isDetecting: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to run anomaly detection";
      setState((prev) => ({
        ...prev,
        isDetecting: false,
        error: errorMessage,
      }));
      console.error("Failed to run anomaly detection:", error);
    }
  }, [state.filters.time_range]);

  // Initialize data on mount
  useEffect(() => {
    if (!externalAnomalies) {
      loadAnomalies();
    }
  }, [externalAnomalies, loadAnomalies]);

  // Update state when external props change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      anomalies: externalAnomalies ?? prev.anomalies,
      isDetecting: externalDetecting ?? prev.isDetecting,
      error: externalError ?? prev.error,
      filters: externalFilters ?? prev.filters,
    }));
  }, [externalAnomalies, externalDetecting, externalError, externalFilters]);

  const getSeverityColor = (severity: Anomaly["severity"]) => {
    switch (severity) {
      case "critical":
        return styles.severityCritical;
      case "high":
        return styles.severityHigh;
      case "medium":
        return styles.severityMedium;
      case "low":
        return styles.severityLow;
      default:
        return styles.severityLow;
    }
  };

  const getSeverityIcon = (severity: Anomaly["severity"]) => {
    switch (severity) {
      case "critical":
        return "🚨";
      case "high":
        return "⚠️";
      case "medium":
        return "🟡";
      case "low":
        return "ℹ️";
      default:
        return "❓";
    }
  };

  const formatDeviation = (anomaly: Anomaly) => {
    const deviation = anomaly.deviation;
    if (deviation > 0) {
      return `+${(deviation * 100).toFixed(1)}%`;
    } else {
      return `${(deviation * 100).toFixed(1)}%`;
    }
  };

  return (
    <div className={styles.anomalyDetector}>
      <div className={styles.detectorHeader}>
        <h2>Anomaly Detection</h2>
        <p className={styles.description}>
          Automated detection of unusual patterns and deviations in system
          metrics using advanced algorithms.
        </p>

        <div className={styles.detectorControls}>
          <div className={styles.filterControls}>
            <div className={styles.filterGroup}>
              <label>Min Severity:</label>
              <select
                value={state.filters.anomaly_severity?.[0] ?? "low"}
                onChange={(e) =>
                  handleFilterChange("anomaly_severity", [
                    e.target.value as Anomaly["severity"],
                    ...(state.filters.anomaly_severity?.slice(1) ?? []),
                  ])
                }
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
            </div>

            <div className={styles.filterGroup}>
              <label>Confidence Threshold:</label>
              <input
                type="number"
                min="0"
                max="1"
                step="0.1"
                value={state.filters.confidence_threshold ?? 0.5}
                onChange={(e) =>
                  handleFilterChange(
                    "confidence_threshold",
                    parseFloat(e.target.value)
                  )
                }
                className={styles.confidenceInput}
              />
            </div>
          </div>

          <button
            onClick={runAnomalyDetection}
            disabled={state.isDetecting}
            className={styles.detectButton}
          >
            {state.isDetecting ? (
              <>
                <div className={styles.spinner}></div>
                Detecting...
              </>
            ) : (
              <>🔍 Detect Anomalies</>
            )}
          </button>
        </div>
      </div>

      {/* Error Display */}
      {state.error && (
        <div className={styles.errorBanner}>
          <span className={styles.errorIcon}>⚠️</span>
          <span>{state.error}</span>
        </div>
      )}

      {/* Anomalies List */}
      <div className={styles.anomaliesSection}>
        {state.isDetecting ? (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <p>Running anomaly detection algorithms...</p>
          </div>
        ) : state.anomalies.length > 0 ? (
          <>
            <div className={styles.anomaliesHeader}>
              <h3>Detected Anomalies ({state.anomalies.length})</h3>
              <div className={styles.anomaliesStats}>
                <span>
                  Critical:{" "}
                  {
                    state.anomalies.filter((a) => a.severity === "critical")
                      .length
                  }
                </span>
                <span>
                  High:{" "}
                  {state.anomalies.filter((a) => a.severity === "high").length}
                </span>
                <span>
                  Medium:{" "}
                  {
                    state.anomalies.filter((a) => a.severity === "medium")
                      .length
                  }
                </span>
                <span>
                  Low:{" "}
                  {state.anomalies.filter((a) => a.severity === "low").length}
                </span>
              </div>
            </div>

            <div className={styles.anomaliesList}>
              {state.anomalies.map((anomaly) => (
                <div
                  key={anomaly.id}
                  className={`${styles.anomalyCard} ${
                    state.selectedAnomaly?.id === anomaly.id
                      ? styles.selected
                      : ""
                  }`}
                  onClick={() => handleAnomalySelect(anomaly)}
                >
                  <div className={styles.anomalyHeader}>
                    <div className={styles.anomalyTitle}>
                      <span
                        className={`${styles.severityIcon} ${getSeverityColor(
                          anomaly.severity
                        )}`}
                      >
                        {getSeverityIcon(anomaly.severity)}
                      </span>
                      <span className={styles.anomalyMetric}>
                        {anomaly.metric}
                      </span>
                      <span
                        className={`${styles.severityBadge} ${getSeverityColor(
                          anomaly.severity
                        )}`}
                      >
                        {anomaly.severity.toUpperCase()}
                      </span>
                    </div>

                    <div className={styles.anomalyActions}>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleAnomalyDismiss(anomaly.id);
                        }}
                        className={styles.dismissButton}
                      >
                        ✕
                      </button>
                    </div>
                  </div>

                  <div className={styles.anomalyDetails}>
                    <div className={styles.anomalyMetrics}>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Value:</span>
                        <span className={styles.metricValue}>
                          {anomaly.value.toFixed(2)}
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Expected:</span>
                        <span className={styles.metricValue}>
                          {anomaly.expected_value.toFixed(2)}
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Deviation:</span>
                        <span
                          className={`${styles.metricValue} ${styles.deviation}`}
                        >
                          {formatDeviation(anomaly)}
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Confidence:</span>
                        <span className={styles.metricValue}>
                          {(anomaly.confidence * 100).toFixed(1)}%
                        </span>
                      </div>
                    </div>

                    <div className={styles.anomalyTime}>
                      {new Date(anomaly.timestamp).toLocaleString()}
                    </div>

                    <div className={styles.anomalyDescription}>
                      {anomaly.description}
                    </div>

                    {anomaly.context && (
                      <details className={styles.anomalyContext}>
                        <summary>Additional Context</summary>
                        <pre className={styles.contextData}>
                          {JSON.stringify(anomaly.context, null, 2)}
                        </pre>
                      </details>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </>
        ) : (
          <div className={styles.noAnomalies}>
            <div className={styles.emptyIcon}>✅</div>
            <h3>No Anomalies Detected</h3>
            <p>
              The system is operating within normal parameters. No significant
              deviations from expected behavior have been detected.
            </p>
            <div className={styles.detectionInfo}>
              <h4>About Anomaly Detection:</h4>
              <ul>
                <li>Uses statistical algorithms (Z-score, Isolation Forest)</li>
                <li>Analyzes patterns in time series data</li>
                <li>Configurable sensitivity and confidence thresholds</li>
                <li>Real-time detection with alerting capabilities</li>
              </ul>
            </div>
          </div>
        )}
      </div>

      {/* Anomaly Details Sidebar */}
      {state.selectedAnomaly && (
        <div className={styles.anomalySidebar}>
          <div className={styles.sidebarHeader}>
            <h3>Anomaly Details</h3>
            <button
              onClick={() =>
                setState((prev) => ({ ...prev, selectedAnomaly: null }))
              }
              className={styles.closeSidebar}
            >
              ✕
            </button>
          </div>

          <div className={styles.sidebarContent}>
            <div className={styles.detailSection}>
              <h4>Metric: {state.selectedAnomaly.metric}</h4>
              <div className={styles.detailGrid}>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Timestamp:</span>
                  <span className={styles.detailValue}>
                    {new Date(state.selectedAnomaly.timestamp).toLocaleString()}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Severity:</span>
                  <span
                    className={`${styles.detailValue} ${getSeverityColor(
                      state.selectedAnomaly.severity
                    )}`}
                  >
                    {state.selectedAnomaly.severity.toUpperCase()}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Actual Value:</span>
                  <span className={styles.detailValue}>
                    {state.selectedAnomaly.value.toFixed(4)}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Expected Value:</span>
                  <span className={styles.detailValue}>
                    {state.selectedAnomaly.expected_value.toFixed(4)}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Confidence:</span>
                  <span className={styles.detailValue}>
                    {(state.selectedAnomaly.confidence * 100).toFixed(1)}%
                  </span>
                </div>
              </div>
            </div>

            {state.selectedAnomaly.context && (
              <div className={styles.detailSection}>
                <h4>Context Information</h4>
                <pre className={styles.contextJson}>
                  {JSON.stringify(state.selectedAnomaly.context, null, 2)}
                </pre>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
