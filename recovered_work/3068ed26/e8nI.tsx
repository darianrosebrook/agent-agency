"use client";

import React, { useMemo } from "react";
import { ForecastingChartProps } from "@/types/analytics";
import styles from "./ForecastingChart.module.scss";

export default function ForecastingChart({
  prediction,
  historicalData,
  showConfidenceIntervals = true,
  onTimeRangeChange,
  isLoading,
  error,
}: ForecastingChartProps) {
  // Time range state management
  const [timeRange, setTimeRange] = React.useState({
    start: "",
    end: "",
  });

  // Initialize time range from available data
  React.useEffect(() => {
    if (historicalData?.data && historicalData.data.length > 0) {
      const timestamps = historicalData.data.map((d) => new Date(d.timestamp));
      if (timestamps.length > 0) {
        const latest = new Date(Math.max(...timestamps.map((d) => d.getTime())));

        // Default to last 30 days
        const thirtyDaysAgo = new Date(latest);
        thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

        setTimeRange({
          start: thirtyDaysAgo.toISOString().split("T")[0], // YYYY-MM-DD format
          end: latest.toISOString().split("T")[0],
        });
      }
    }
  }, [historicalData]);

  // Handle time range changes
  const handleTimeRangeChange = (start: string, end: string) => {
    setTimeRange({ start, end });
    onTimeRangeChange?.(start, end);
  };

  // Generate mock chart data for demonstration
  const chartData = useMemo(() => {
    if (!prediction) return null;

    const historical = historicalData?.data ?? [];
    const predicted = prediction.predicted_values;

    // Combine historical and predicted data
    const combined = [
      ...historical.map((point) => ({
        timestamp: point.timestamp,
        value: point.value,
        type: "historical" as const,
      })),
      ...predicted.map((point) => ({
        timestamp: point.timestamp,
        value: point.value,
        type: "predicted" as const,
      })),
    ];

    // Sort by timestamp
    combined.sort(
      (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    );

    return combined;
  }, [prediction, historicalData]);

  if (isLoading) {
    return (
      <div className={styles.forecastingChart}>
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading forecast data...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.forecastingChart}>
        <div className={styles.error}>
          <span className={styles.errorIcon}>⚠️</span>
          <span>{error}</span>
        </div>
      </div>
    );
  }

  if (!prediction || !chartData) {
    return (
      <div className={styles.forecastingChart}>
        <div className={styles.noData}>
          <div className={styles.emptyIcon}>📊</div>
          <h3>No Forecast Data</h3>
          <p>Forecast data will be available once prediction models are run.</p>
        </div>
      </div>
    );
  }

  // Calculate chart dimensions
  const maxValue = Math.max(...chartData.map((d) => d.value));
  const minValue = Math.min(...chartData.map((d) => d.value));
  const range = maxValue - minValue;
  const padding = range * 0.1;

  // Create simple SVG chart
  const width = 800;
  const height = 400;
  const margin = { top: 20, right: 30, bottom: 40, left: 50 };

  const chartWidth = width - margin.left - margin.right;
  const chartHeight = height - margin.top - margin.bottom;

  const xScale = (timestamp: string) => {
    const timestamps = chartData.map((d) => new Date(d.timestamp).getTime());
    const minTime = Math.min(...timestamps);
    const maxTime = Math.max(...timestamps);
    const timeRange = maxTime - minTime;
    return ((new Date(timestamp).getTime() - minTime) / timeRange) * chartWidth;
  };

  const yScale = (value: number) => {
    return (
      chartHeight -
      ((value - (minValue - padding)) / (range + 2 * padding)) * chartHeight
    );
  };

  // Create paths for the lines (used directly in JSX)

  // Split data by type for different styling
  const historicalDataPoints = chartData.filter((d) => d.type === "historical");
  const predictedDataPoints = chartData.filter((d) => d.type === "predicted");

  return (
    <div className={styles.forecastingChart}>
      <div className={styles.chartHeader}>
        <div className={styles.headerLeft}>
          <h3>Forecast: {prediction.metric}</h3>
          <div className={styles.chartMeta}>
            <span>
              Model Accuracy: {(prediction.model_accuracy * 100).toFixed(1)}%
            </span>
            <span>
              Forecast Horizon: {prediction.predicted_values.length} periods
            </span>
          </div>
        </div>

        {/* Time Range Controls */}
        <div className={styles.timeRangeControls}>
          <label className={styles.timeRangeLabel}>
            Time Range:
            <div className={styles.dateInputs}>
              <input
                type="date"
                value={timeRange.start}
                onChange={(e) =>
                  handleTimeRangeChange(e.target.value, timeRange.end)
                }
                className={styles.dateInput}
                title="Start date"
              />
              <span className={styles.dateSeparator}>to</span>
              <input
                type="date"
                value={timeRange.end}
                onChange={(e) =>
                  handleTimeRangeChange(timeRange.start, e.target.value)
                }
                className={styles.dateInput}
                title="End date"
              />
            </div>
          </label>

          {/* Quick range buttons */}
          <div className={styles.quickRanges}>
            <button
              type="button"
              onClick={() => {
                const end = new Date();
                const start = new Date();
                start.setDate(end.getDate() - 7);
                handleTimeRangeChange(
                  start.toISOString().split("T")[0],
                  end.toISOString().split("T")[0]
                );
              }}
              className={styles.quickRangeBtn}
            >
              7d
            </button>
            <button
              type="button"
              onClick={() => {
                const end = new Date();
                const start = new Date();
                start.setDate(end.getDate() - 30);
                handleTimeRangeChange(
                  start.toISOString().split("T")[0],
                  end.toISOString().split("T")[0]
                );
              }}
              className={styles.quickRangeBtn}
            >
              30d
            </button>
            <button
              type="button"
              onClick={() => {
                const end = new Date();
                const start = new Date();
                start.setDate(end.getDate() - 90);
                handleTimeRangeChange(
                  start.toISOString().split("T")[0],
                  end.toISOString().split("T")[0]
                );
              }}
              className={styles.quickRangeBtn}
            >
              90d
            </button>
          </div>
        </div>
      </div>

      <div className={styles.chartContainer}>
        <svg width={width} height={height} className={styles.chart}>
          {/* Background grid */}
          <defs>
            <pattern
              id="grid"
              width="40"
              height="40"
              patternUnits="userSpaceOnUse"
            >
              <path
                d="M 40 0 L 0 0 0 40"
                fill="none"
                stroke="var(--color-border-light)"
                strokeWidth="1"
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />

          {/* X and Y axes */}
          <line
            x1={margin.left}
            y1={height - margin.bottom}
            x2={width - margin.right}
            y2={height - margin.bottom}
            stroke="var(--color-text-secondary)"
            strokeWidth="2"
          />
          <line
            x1={margin.left}
            y1={margin.top}
            x2={margin.left}
            y2={height - margin.bottom}
            stroke="var(--color-text-secondary)"
            strokeWidth="2"
          />

          {/* Historical data line */}
          {historicalDataPoints.length > 1 && (
            <path
              d={historicalDataPoints
                .map((point, index) => {
                  const x = xScale(point.timestamp) + margin.left;
                  const y = yScale(point.value) + margin.top;
                  return `${index === 0 ? "M" : "L"} ${x} ${y}`;
                })
                .join(" ")}
              fill="none"
              stroke="var(--color-primary-500)"
              strokeWidth="3"
              strokeLinejoin="round"
              strokeLinecap="round"
            />
          )}

          {/* Predicted data line */}
          {predictedDataPoints.length > 1 && (
            <path
              d={predictedDataPoints
                .map((point, index) => {
                  const x = xScale(point.timestamp) + margin.left;
                  const y = yScale(point.value) + margin.top;
                  return `${index === 0 ? "M" : "L"} ${x} ${y}`;
                })
                .join(" ")}
              fill="none"
              stroke="var(--color-accent-500)"
              strokeWidth="3"
              strokeLinejoin="round"
              strokeLinecap="round"
              strokeDasharray="5,5"
            />
          )}

          {/* Confidence intervals */}
          {showConfidenceIntervals && prediction.confidence_intervals && (
            <g>
              {prediction.confidence_intervals.map((interval, index) => {
                const x = xScale(interval.timestamp) + margin.left;
                const y1 = yScale(interval.upper_bound) + margin.top;
                const y2 = yScale(interval.lower_bound) + margin.top;
                const height = Math.abs(y2 - y1);

                return (
                  <rect
                    key={index}
                    x={x - 2}
                    y={Math.min(y1, y2)}
                    width="4"
                    height={height}
                    fill="var(--color-accent-200)"
                    opacity="0.3"
                  />
                );
              })}
            </g>
          )}

          {/* Data points */}
          {chartData.map((point, index) => {
            const x = xScale(point.timestamp) + margin.left;
            const y = yScale(point.value) + margin.top;
            const isPredicted = point.type === "predicted";

            return (
              <circle
                key={index}
                cx={x}
                cy={y}
                r="4"
                fill={
                  isPredicted
                    ? "var(--color-accent-500)"
                    : "var(--color-primary-500)"
                }
                stroke="var(--color-bg-primary)"
                strokeWidth="2"
              />
            );
          })}
        </svg>
      </div>

      <div className={styles.chartLegend}>
        <div className={styles.legendItem}>
          <div
            className={styles.legendColor}
            style={{ backgroundColor: "var(--color-primary-500)" }}
          ></div>
          <span>Historical Data</span>
        </div>
        <div className={styles.legendItem}>
          <div
            className={styles.legendColor}
            style={{
              backgroundColor: "var(--color-accent-500)",
              borderStyle: "dashed",
            }}
          ></div>
          <span>Predicted Data</span>
        </div>
        {showConfidenceIntervals && (
          <div className={styles.legendItem}>
            <div
              className={styles.legendColor}
              style={{ backgroundColor: "var(--color-accent-200)" }}
            ></div>
            <span>Confidence Interval</span>
          </div>
        )}
      </div>

      {/* Forecast details */}
      <div className={styles.forecastDetails}>
        <div className={styles.detailSection}>
          <h4>Next Prediction</h4>
          <div className={styles.predictionGrid}>
            <div className={styles.predictionItem}>
              <span className={styles.predictionLabel}>Value:</span>
              <span className={styles.predictionValue}>
                {prediction.predicted_values[0]?.value.toFixed(2) || "N/A"}
              </span>
            </div>
            <div className={styles.predictionItem}>
              <span className={styles.predictionLabel}>Date:</span>
              <span className={styles.predictionValue}>
                {prediction.predicted_values[0]
                  ? new Date(
                      prediction.predicted_values[0].timestamp
                    ).toLocaleDateString()
                  : "N/A"}
              </span>
            </div>
            <div className={styles.predictionItem}>
              <span className={styles.predictionLabel}>Confidence:</span>
              <span className={styles.predictionValue}>
                {(prediction.model_accuracy * 100).toFixed(1)}%
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
