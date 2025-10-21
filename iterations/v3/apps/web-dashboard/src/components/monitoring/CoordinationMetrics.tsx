"use client";

import React, { useState, useEffect } from "react";
import { CoordinationMetricsProps, CoordinationMetrics } from "@/types/metrics";
import { metricsApiClient, MetricsApiError } from "@/lib/metrics-api";
import MetricTile from "./MetricTile";
import styles from "./CoordinationMetrics.module.scss";

export default function CoordinationMetricsComponent({
  metrics: externalMetrics,
  isLoading: externalLoading,
  error: externalError,
}: CoordinationMetricsProps) {
  const [metrics, setMetrics] = useState<CoordinationMetrics | null>(
    externalMetrics ?? null
  );
  const [isLoading, setIsLoading] = useState(
    externalLoading ?? !externalMetrics
  );
  const [error, setError] = useState<string | null>(externalError ?? null);

  // Load coordination metrics if not provided externally
  const loadCoordinationMetrics = async () => {
    if (externalMetrics) return; // Use external data if provided

    try {
      setIsLoading(true);
      setError(null);

      const coordinationData = await metricsApiClient.getCoordinationMetrics();
      setMetrics(coordinationData);
    } catch (err) {
      const errorMessage =
        err instanceof MetricsApiError
          ? err.message
          : "Failed to load coordination metrics";

      setError(errorMessage);
      console.error("Failed to load coordination metrics:", err);
    } finally {
      setIsLoading(false);
    }
  };

  // Initial load and external prop updates
  useEffect(() => {
    if (externalMetrics) {
      setMetrics(externalMetrics);
      setIsLoading(false);
      setError(null);
    } else if (!externalMetrics && !isLoading) {
      loadCoordinationMetrics();
    }
  }, [externalMetrics]);


  // Format duration (for future use)
  // const formatDuration = (ms: number): string => {
  //   if (ms < 1000) return `${ms.toFixed(0)}ms`;
  //   if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  //   if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
  //   return `${(ms / 3600000).toFixed(1)}h`;
  // };

  if (isLoading) {
    return (
      <div className={styles.coordinationMetrics}>
        <div className={styles.loading}>
          <div className={styles.loadingSpinner}></div>
          <p>Loading coordination metrics...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.coordinationMetrics}>
        <div className={styles.error}>
          <h3>Failed to load coordination metrics</h3>
          <p>{error}</p>
          <button onClick={loadCoordinationMetrics}>Retry</button>
        </div>
      </div>
    );
  }

  if (!metrics) {
    return (
      <div className={styles.coordinationMetrics}>
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>🤝</div>
          <h3>Coordination Metrics</h3>
          <p>
            Real-time agent coordination monitoring requires V3 coordination
            metrics APIs.
          </p>
          <div className={styles.emptyActions}>
            <button className={styles.secondaryButton} disabled>
              Connect to Coordination API
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.coordinationMetrics}>
      <div className={styles.header}>
        <h2>Agent Coordination</h2>
        <div className={styles.timestamp}>
          Last updated: {new Date(metrics.timestamp).toLocaleTimeString()}
        </div>
      </div>

      <div className={styles.metricsGrid}>
        <MetricTile
          title="Consensus Rate"
          value={`${(metrics.consensus_rate * 100).toFixed(1)}%`}
          status={
            metrics.consensus_rate >= 0.95
              ? "success"
              : metrics.consensus_rate >= 0.8
              ? "warning"
              : "error"
          }
          icon="🎯"
          format="percentage"
        />

        <MetricTile
          title="Decision Time"
          value={metrics.average_decision_time_ms}
          status={
            metrics.average_decision_time_ms < 5000
              ? "success"
              : metrics.average_decision_time_ms < 15000
              ? "warning"
              : "error"
          }
          icon="⏱️"
          format="duration"
        />

        <MetricTile
          title="Total Decisions"
          value={metrics.total_decisions}
          status="neutral"
          icon="📊"
        />

        <MetricTile
          title="Failed Decisions"
          value={metrics.failed_decisions}
          status={
            metrics.failed_decisions === 0
              ? "success"
              : metrics.failed_decisions < 5
              ? "warning"
              : "error"
          }
          icon="❌"
        />
      </div>

      <div className={styles.taskCoordination}>
        <h3>Task Coordination</h3>
        <div className={styles.taskMetrics}>
          <MetricTile
            title="Active Tasks"
            value={metrics.active_tasks}
            status={metrics.active_tasks > 0 ? "success" : "neutral"}
            icon="⚡"
          />

          <MetricTile
            title="Queued Tasks"
            value={metrics.queued_tasks}
            status={
              metrics.queued_tasks > 10
                ? "warning"
                : metrics.queued_tasks > 50
                ? "error"
                : "success"
            }
            icon="📋"
          />

          <MetricTile
            title="Tasks/Hour"
            value={metrics.completed_tasks_per_hour}
            status={
              metrics.completed_tasks_per_hour > 5
                ? "success"
                : metrics.completed_tasks_per_hour > 2
                ? "warning"
                : "error"
            }
            icon="📈"
          />

          <MetricTile
            title="Avg Completion Time"
            value={metrics.average_task_duration_ms}
            status={
              metrics.average_task_duration_ms < 300000
                ? "success"
                : metrics.average_task_duration_ms < 900000
                ? "warning"
                : "error"
            }
            icon="⏱️"
            format="duration"
          />
        </div>
      </div>

      <div className={styles.communicationMetrics}>
        <h3>Inter-Agent Communication</h3>
        <div className={styles.commMetrics}>
          <MetricTile
            title="Messages/Minute"
            value={metrics.inter_agent_messages_per_minute}
            status={
              metrics.inter_agent_messages_per_minute < 1000
                ? "success"
                : "warning"
            }
            icon="💬"
          />

          <MetricTile
            title="Message Latency"
            value={metrics.average_message_latency_ms}
            status={
              metrics.average_message_latency_ms < 100
                ? "success"
                : metrics.average_message_latency_ms < 500
                ? "warning"
                : "error"
            }
            icon="⚡"
            format="duration"
          />

          <MetricTile
            title="Message Failure Rate"
            value={`${(metrics.message_failure_rate * 100).toFixed(2)}%`}
            status={
              metrics.message_failure_rate < 0.01
                ? "success"
                : metrics.message_failure_rate < 0.05
                ? "warning"
                : "error"
            }
            icon="🔄"
            format="percentage"
          />
        </div>
      </div>

      <div className={styles.coordinationHealth}>
        <h3>Coordination Health Indicators</h3>
        <div className={styles.healthIndicators}>
          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {metrics.consensus_rate >= 0.9
                  ? "✅"
                  : metrics.consensus_rate >= 0.7
                  ? "⚠️"
                  : "❌"}
              </span>
              <span>Consensus Stability</span>
            </div>
            <div className={styles.indicatorValue}>
              {metrics.consensus_rate >= 0.9
                ? "Healthy"
                : metrics.consensus_rate >= 0.7
                ? "Degraded"
                : "Unhealthy"}
            </div>
          </div>

          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {metrics.message_failure_rate < 0.02
                  ? "✅"
                  : metrics.message_failure_rate < 0.1
                  ? "⚠️"
                  : "❌"}
              </span>
              <span>Communication Reliability</span>
            </div>
            <div className={styles.indicatorValue}>
              {metrics.message_failure_rate < 0.02
                ? "Excellent"
                : metrics.message_failure_rate < 0.1
                ? "Good"
                : "Poor"}
            </div>
          </div>

          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {metrics.average_decision_time_ms < 10000
                  ? "✅"
                  : metrics.average_decision_time_ms < 30000
                  ? "⚠️"
                  : "❌"}
              </span>
              <span>Decision Speed</span>
            </div>
            <div className={styles.indicatorValue}>
              {metrics.average_decision_time_ms < 10000
                ? "Fast"
                : metrics.average_decision_time_ms < 30000
                ? "Moderate"
                : "Slow"}
            </div>
          </div>

          <div className={styles.indicator}>
            <div className={styles.indicatorLabel}>
              <span className={styles.indicatorIcon}>
                {metrics.queued_tasks < 20
                  ? "✅"
                  : metrics.queued_tasks < 100
                  ? "⚠️"
                  : "❌"}
              </span>
              <span>Queue Health</span>
            </div>
            <div className={styles.indicatorValue}>
              {metrics.queued_tasks < 20
                ? "Clear"
                : metrics.queued_tasks < 100
                ? "Backed up"
                : "Overloaded"}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
