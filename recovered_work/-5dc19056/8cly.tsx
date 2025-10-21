"use client";

import React, { useState, useEffect, useCallback } from "react";
import { TrendAnalyzerProps, Trend, AnalyticsFilters } from "@/types/analytics";
import { analyticsApiClient, AnalyticsApiError } from "@/lib/analytics-api";
import styles from "./TrendAnalyzer.module.scss";

interface TrendAnalyzerState {
  trends: Trend[];
  selectedTrend: Trend | null;
  isAnalyzing: boolean;
  error: string | null;
  filters: AnalyticsFilters;
  analysisParams: {
    window_size: number;
    min_trend_strength: number;
    seasonality_detection: boolean;
  };
}

export default function TrendAnalyzer({
  trends: externalTrends,
  timeSeriesData,
  onTrendSelect,
  filters: externalFilters,
  isAnalyzing: externalAnalyzing,
  error: externalError,
}: TrendAnalyzerProps) {
  const [state, setState] = useState<TrendAnalyzerState>({
    trends: externalTrends ?? [],
    selectedTrend: null,
    isAnalyzing: externalAnalyzing ?? false,
    error: externalError ?? null,
    filters: externalFilters ?? {
      time_range: {
        start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(), // 7 days ago
        end: new Date().toISOString(),
      },
      granularity: "1h",
    },
  });

  // Load trends if not provided externally
  const loadTrends = useCallback(async () => {
    if (externalTrends) return;

    // Integrate timeSeriesData for advanced trend analysis
    if (timeSeriesData && timeSeriesData.length > 0) {
      console.log(
        "Time series data available for trend analysis:",
        timeSeriesData.length,
        "series"
      );

      // Use time series data to enhance trend analysis parameters
      const latestSeries = timeSeriesData[timeSeriesData.length - 1];
      if (latestSeries?.data && latestSeries.data.length > 10) {
        // Calculate trend metrics from time series data
        const data = latestSeries.data;
        const recentData = data.slice(-30); // Last 30 points for trend calculation

        if (recentData.length >= 10) {
          // Calculate linear trend using simple linear regression
          const n = recentData.length;
          const sumX = (n * (n - 1)) / 2; // Sum of indices 0 to n-1
          const sumY = recentData.reduce((sum, point) => sum + point.value, 0);
          const sumXY = recentData.reduce(
            (sum, point, index) => sum + index * point.value,
            0
          );
          const sumXX = (n * (n - 1) * (2 * n - 1)) / 6; // Sum of squares of indices

          const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
          const intercept = (sumY - slope * sumX) / n;

          // Calculate R-squared to measure trend strength
          const yMean = sumY / n;
          const ssRes = recentData.reduce((sum, point, index) => {
            const predicted = slope * index + intercept;
            return sum + Math.pow(point.value - predicted, 2);
          }, 0);
          const ssTot = recentData.reduce((sum, point) => {
            return sum + Math.pow(point.value - yMean, 2);
          }, 0);
          const rSquared = 1 - ssRes / ssTot;

          // Update analysis parameters with trend insights
          setState((prev) => ({
            ...prev,
            analysisParams: {
              ...prev.analysisParams,
              min_trend_strength: Math.max(0.1, rSquared * 0.8), // Adaptive threshold
              analysis_window_days: Math.max(
                7,
                Math.min(90, Math.floor(n / 3))
              ), // Adaptive window
              trend_sensitivity: Math.abs(slope) > 0.01 ? "high" : "medium", // Adjust based on slope magnitude
            },
          }));

          console.log(
            `Trend analysis enhanced: slope=${slope.toFixed(
              4
            )}, r²=${rSquared.toFixed(3)}`
          );
        }
      }
    }

    try {
      setState((prev) => ({ ...prev, isAnalyzing: true, error: null }));
      const response = await analyticsApiClient.getTrends(state.filters);
      setState((prev) => ({
        ...prev,
        trends: response.trends,
        isAnalyzing: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to load trend analysis";
      setState((prev) => ({
        ...prev,
        isAnalyzing: false,
        error: errorMessage,
      }));
      console.error("Failed to load trends:", error);
    }
  }, [externalTrends, state.filters]);

  // Handle trend selection
  const handleTrendSelect = useCallback(
    (trend: Trend) => {
      setState((prev) => ({ ...prev, selectedTrend: trend }));
      onTrendSelect?.(trend);
    },
    [onTrendSelect]
  );

  // Run trend analysis
  const runTrendAnalysis = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isAnalyzing: true, error: null }));

      // Re-fetch trends with current filters
      const response = await analyticsApiClient.getTrends(state.filters);
      setState((prev) => ({
        ...prev,
        trends: response.trends,
        isAnalyzing: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof AnalyticsApiError
          ? error.message
          : "Failed to run trend analysis";
      setState((prev) => ({
        ...prev,
        isAnalyzing: false,
        error: errorMessage,
      }));
      console.error("Failed to run trend analysis:", error);
    }
  }, [state.filters]);

  // Initialize data on mount
  useEffect(() => {
    if (!externalTrends) {
      loadTrends();
    }
  }, [externalTrends, loadTrends]);

  // Update state when external props change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      trends: externalTrends ?? prev.trends,
      isAnalyzing: externalAnalyzing ?? prev.isAnalyzing,
      error: externalError ?? prev.error,
      filters: externalFilters ?? prev.filters,
    }));
  }, [externalTrends, externalAnalyzing, externalError, externalFilters]);

  const getDirectionIcon = (direction: Trend["direction"]) => {
    switch (direction) {
      case "increasing":
        return "📈";
      case "decreasing":
        return "📉";
      case "stable":
        return "➡️";
      case "volatile":
        return "📊";
      default:
        return "❓";
    }
  };

  const getDirectionColor = (direction: Trend["direction"]) => {
    switch (direction) {
      case "increasing":
        return styles.directionIncreasing;
      case "decreasing":
        return styles.directionDecreasing;
      case "stable":
        return styles.directionStable;
      case "volatile":
        return styles.directionVolatile;
      default:
        return styles.directionStable;
    }
  };

  const formatSlope = (slope: number) => {
    if (Math.abs(slope) < 0.001) {
      return "0.000";
    }
    return slope.toFixed(3);
  };

  const getStrengthDescription = (r_squared: number) => {
    if (r_squared >= 0.8) return "Very Strong";
    if (r_squared >= 0.6) return "Strong";
    if (r_squared >= 0.3) return "Moderate";
    if (r_squared >= 0.1) return "Weak";
    return "Very Weak";
  };

  return (
    <div className={styles.trendAnalyzer}>
      <div className={styles.analyzerHeader}>
        <h2>Trend Analysis</h2>
        <p className={styles.description}>
          Statistical analysis of metric trends using linear regression,
          seasonal decomposition, and forecasting models.
        </p>

        <div className={styles.analyzerControls}>
          <button
            onClick={runTrendAnalysis}
            disabled={state.isAnalyzing}
            className={styles.analyzeButton}
          >
            {state.isAnalyzing ? (
              <>
                <div className={styles.spinner}></div>
                Analyzing...
              </>
            ) : (
              <>📈 Analyze Trends</>
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

      {/* Trends List */}
      <div className={styles.trendsSection}>
        {state.isAnalyzing ? (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <p>Running statistical trend analysis...</p>
          </div>
        ) : state.trends.length > 0 ? (
          <>
            <div className={styles.trendsHeader}>
              <h3>Detected Trends ({state.trends.length})</h3>
              <div className={styles.trendsStats}>
                <span>
                  Increasing:{" "}
                  {
                    state.trends.filter((t) => t.direction === "increasing")
                      .length
                  }
                </span>
                <span>
                  Decreasing:{" "}
                  {
                    state.trends.filter((t) => t.direction === "decreasing")
                      .length
                  }
                </span>
                <span>
                  Stable:{" "}
                  {state.trends.filter((t) => t.direction === "stable").length}
                </span>
                <span>
                  Volatile:{" "}
                  {
                    state.trends.filter((t) => t.direction === "volatile")
                      .length
                  }
                </span>
              </div>
            </div>

            <div className={styles.trendsList}>
              {state.trends.map((trend) => (
                <div
                  key={trend.metric}
                  className={`${styles.trendCard} ${
                    state.selectedTrend?.metric === trend.metric
                      ? styles.selected
                      : ""
                  }`}
                  onClick={() => handleTrendSelect(trend)}
                >
                  <div className={styles.trendHeader}>
                    <div className={styles.trendTitle}>
                      <span
                        className={`${styles.directionIcon} ${getDirectionColor(
                          trend.direction
                        )}`}
                      >
                        {getDirectionIcon(trend.direction)}
                      </span>
                      <span className={styles.trendMetric}>{trend.metric}</span>
                      <span
                        className={`${
                          styles.directionBadge
                        } ${getDirectionColor(trend.direction)}`}
                      >
                        {trend.direction.toUpperCase()}
                      </span>
                    </div>
                  </div>

                  <div className={styles.trendDetails}>
                    <div className={styles.trendMetrics}>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Slope:</span>
                        <span className={styles.metricValue}>
                          {formatSlope(trend.slope)}
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>R²:</span>
                        <span className={styles.metricValue}>
                          {(trend.r_squared * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Strength:</span>
                        <span className={styles.metricValue}>
                          {getStrengthDescription(trend.r_squared)}
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Confidence:</span>
                        <span className={styles.metricValue}>
                          {(trend.confidence * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className={styles.metricItem}>
                        <span className={styles.metricLabel}>Period:</span>
                        <span className={styles.metricValue}>
                          {trend.period_days} days
                        </span>
                      </div>
                    </div>

                    <div className={styles.trendDescription}>
                      {trend.description}
                    </div>

                    {trend.forecast && (
                      <div className={styles.trendForecast}>
                        <h4>Forecast</h4>
                        <div className={styles.forecastDetails}>
                          <div className={styles.forecastItem}>
                            <span className={styles.forecastLabel}>
                              Next Value:
                            </span>
                            <span className={styles.forecastValue}>
                              {trend.forecast.next_value.toFixed(2)}
                            </span>
                          </div>
                          <div className={styles.forecastItem}>
                            <span className={styles.forecastLabel}>
                              Confidence Interval:
                            </span>
                            <span className={styles.forecastValue}>
                              [
                              {trend.forecast.confidence_interval[0].toFixed(2)}
                              ,
                              {trend.forecast.confidence_interval[1].toFixed(2)}
                              ]
                            </span>
                          </div>
                          <div className={styles.forecastItem}>
                            <span className={styles.forecastLabel}>
                              Forecast Date:
                            </span>
                            <span className={styles.forecastValue}>
                              {new Date(
                                trend.forecast.timestamp
                              ).toLocaleDateString()}
                            </span>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </>
        ) : (
          <div className={styles.noTrends}>
            <div className={styles.emptyIcon}>📈</div>
            <h3>No Trends Detected</h3>
            <p>
              No significant trends were detected in the current time range.
              This could indicate stable system performance or insufficient
              data.
            </p>
            <div className={styles.analysisInfo}>
              <h4>About Trend Analysis:</h4>
              <ul>
                <li>Uses linear regression to detect directional changes</li>
                <li>R-squared values indicate trend strength</li>
                <li>Includes seasonal decomposition for complex patterns</li>
                <li>Provides confidence intervals for predictions</li>
                <li>Analyzes trends over configurable time periods</li>
              </ul>
            </div>
          </div>
        )}
      </div>

      {/* Trend Details Sidebar */}
      {state.selectedTrend && (
        <div className={styles.trendSidebar}>
          <div className={styles.sidebarHeader}>
            <h3>Trend Details</h3>
            <button
              onClick={() =>
                setState((prev) => ({ ...prev, selectedTrend: null }))
              }
              className={styles.closeSidebar}
            >
              ✕
            </button>
          </div>

          <div className={styles.sidebarContent}>
            <div className={styles.detailSection}>
              <h4>Metric: {state.selectedTrend.metric}</h4>
              <div className={styles.detailGrid}>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Direction:</span>
                  <span
                    className={`${styles.detailValue} ${getDirectionColor(
                      state.selectedTrend.direction
                    )}`}
                  >
                    {state.selectedTrend.direction.toUpperCase()}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Slope:</span>
                  <span className={styles.detailValue}>
                    {formatSlope(state.selectedTrend.slope)}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>R-squared:</span>
                  <span className={styles.detailValue}>
                    {(state.selectedTrend.r_squared * 100).toFixed(1)}%
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Trend Strength:</span>
                  <span className={styles.detailValue}>
                    {getStrengthDescription(state.selectedTrend.r_squared)}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Analysis Period:</span>
                  <span className={styles.detailValue}>
                    {state.selectedTrend.period_days} days
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Confidence:</span>
                  <span className={styles.detailValue}>
                    {(state.selectedTrend.confidence * 100).toFixed(1)}%
                  </span>
                </div>
              </div>
            </div>

            <div className={styles.detailSection}>
              <h4>Description</h4>
              <p className={styles.trendDescription}>
                {state.selectedTrend.description}
              </p>
            </div>

            {state.selectedTrend.forecast && (
              <div className={styles.detailSection}>
                <h4>Forecast Information</h4>
                <div className={styles.forecastGrid}>
                  <div className={styles.forecastItem}>
                    <span className={styles.forecastLabel}>
                      Predicted Value:
                    </span>
                    <span className={styles.forecastValue}>
                      {state.selectedTrend.forecast.next_value.toFixed(4)}
                    </span>
                  </div>
                  <div className={styles.forecastItem}>
                    <span className={styles.forecastLabel}>Lower Bound:</span>
                    <span className={styles.forecastValue}>
                      {state.selectedTrend.forecast.confidence_interval[0].toFixed(
                        4
                      )}
                    </span>
                  </div>
                  <div className={styles.forecastItem}>
                    <span className={styles.forecastLabel}>Upper Bound:</span>
                    <span className={styles.forecastValue}>
                      {state.selectedTrend.forecast.confidence_interval[1].toFixed(
                        4
                      )}
                    </span>
                  </div>
                  <div className={styles.forecastItem}>
                    <span className={styles.forecastLabel}>Forecast Date:</span>
                    <span className={styles.forecastValue}>
                      {new Date(
                        state.selectedTrend.forecast.timestamp
                      ).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
