import React, { useMemo } from "react";
import { ModelPerformanceChartProps } from "../../types/tasks";

import styles from "./ModelPerformanceChart.module.scss";

export const ModelPerformanceChart: React.FC<ModelPerformanceChartProps> = ({
  models,
  timeRange = "24h",
  onModelSelect,
}) => {
  const chartData = useMemo(() => {
    return models.map((model) => ({
      ...model,
      successRate:
        model.performance_stats.total_requests > 0
          ? (model.performance_stats.successful_requests /
              model.performance_stats.total_requests) *
            100
          : 0,
      efficiency:
        model.performance_stats.average_latency_ms > 0
          ? (1 / model.performance_stats.average_latency_ms) * 1000 // Requests per second efficiency
          : 0,
    }));
  }, [models]);

  const sortedBySuccess = [...chartData].sort(
    (a, b) => b.successRate - a.successRate
  );
  const sortedByLatency = [...chartData].sort(
    (a, b) =>
      a.performance_stats.average_latency_ms -
      b.performance_stats.average_latency_ms
  );

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h3>Model Performance</h3>
        <select
          className={styles.timeRange}
          value={timeRange}
          onChange={() => {
            // TODO: Implement time range filtering
          }}
        >
          <option value="1h">Last Hour</option>
          <option value="24h">Last 24 Hours</option>
          <option value="7d">Last 7 Days</option>
          <option value="30d">Last 30 Days</option>
        </select>
      </div>

      <div className={styles.charts}>
        {/* Success Rate Chart */}
        <div className={styles.chart}>
          <h4>Success Rate (%)</h4>
          <div className={styles.barChart}>
            {sortedBySuccess.map((model, index) => (
              <div
                key={model.id}
                className={`${styles.bar} ${index === 0 ? styles.best : ""}`}
                style={{
                  width: `${Math.max(model.successRate, 5)}%`,
                  backgroundColor: index === 0 ? "#28a745" : "#007bff",
                }}
                onClick={() => onModelSelect?.(model.id)}
              >
                <div className={styles.barLabel}>
                  <span className={styles.modelName}>{model.name}</span>
                  <span className={styles.value}>
                    {model.successRate.toFixed(1)}%
                  </span>
                </div>
                <div className={styles.barFill} />
              </div>
            ))}
          </div>
        </div>

        {/* Latency Chart */}
        <div className={styles.chart}>
          <h4>Average Latency (ms)</h4>
          <div className={styles.barChart}>
            {sortedByLatency.map((model, index) => (
              <div
                key={model.id}
                className={`${styles.bar} ${index === 0 ? styles.best : ""}`}
                style={{
                  width: `${Math.min(
                    model.performance_stats.average_latency_ms / 10,
                    100
                  )}%`,
                  backgroundColor: index === 0 ? "#28a745" : "#ffc107",
                }}
                onClick={() => onModelSelect?.(model.id)}
              >
                <div className={styles.barLabel}>
                  <span className={styles.modelName}>{model.name}</span>
                  <span className={styles.value}>
                    {model.performance_stats.average_latency_ms.toFixed(0)}ms
                  </span>
                </div>
                <div className={styles.barFill} />
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Model Details Table */}
      <div className={styles.table}>
        <table>
          <thead>
            <tr>
              <th>Model</th>
              <th>Provider</th>
              <th>Requests</th>
              <th>Success Rate</th>
              <th>Avg Latency</th>
              <th>Error Rate</th>
              <th>Last Used</th>
            </tr>
          </thead>
          <tbody>
            {chartData.map((model) => (
              <tr
                key={model.id}
                className={styles.clickable}
                onClick={() => onModelSelect?.(model.id)}
              >
                <td>{model.name}</td>
                <td>{model.provider}</td>
                <td>{model.performance_stats.total_requests}</td>
                <td
                  className={
                    model.successRate >= 95
                      ? styles.good
                      : model.successRate >= 80
                      ? styles.warning
                      : styles.bad
                  }
                >
                  {model.successRate.toFixed(1)}%
                </td>
                <td>
                  {model.performance_stats.average_latency_ms.toFixed(0)}ms
                </td>
                <td
                  className={
                    model.performance_stats.error_rate <= 0.05
                      ? styles.good
                      : model.performance_stats.error_rate <= 0.15
                      ? styles.warning
                      : styles.bad
                  }
                >
                  {(model.performance_stats.error_rate * 100).toFixed(1)}%
                </td>
                <td>
                  {new Date(model.performance_stats.last_used).toLocaleString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Model Capabilities */}
      <div className={styles.capabilities}>
        <h4>Model Capabilities</h4>
        <div className={styles.capabilityGrid}>
          {chartData.map((model) => (
            <div key={model.id} className={styles.capabilityCard}>
              <h5>{model.name}</h5>
              <div className={styles.capabilityList}>
                <div className={styles.capability}>
                  <span className={styles.label}>Max Context:</span>
                  <span className={styles.value}>
                    {model.capabilities.max_context.toLocaleString()}
                  </span>
                </div>
                <div className={styles.capability}>
                  <span className={styles.label}>Streaming:</span>
                  <span
                    className={`${styles.value} ${
                      model.capabilities.supports_streaming
                        ? styles.enabled
                        : styles.disabled
                    }`}
                  >
                    {model.capabilities.supports_streaming ? "✓" : "✗"}
                  </span>
                </div>
                <div className={styles.capability}>
                  <span className={styles.label}>Function Calling:</span>
                  <span
                    className={`${styles.value} ${
                      model.capabilities.supports_function_calling
                        ? styles.enabled
                        : styles.disabled
                    }`}
                  >
                    {model.capabilities.supports_function_calling ? "✓" : "✗"}
                  </span>
                </div>
                <div className={styles.capability}>
                  <span className={styles.label}>Vision:</span>
                  <span
                    className={`${styles.value} ${
                      model.capabilities.supports_vision
                        ? styles.enabled
                        : styles.disabled
                    }`}
                  >
                    {model.capabilities.supports_vision ? "✓" : "✗"}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
