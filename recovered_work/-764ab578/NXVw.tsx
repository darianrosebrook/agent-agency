"use client";

import React, { useState, useEffect } from "react";
import { AgentPerformanceGridProps, AgentPerformance } from "@/types/metrics";
import { metricsApiClient, MetricsApiError } from "@/lib/metrics-api";
import MetricTile from "./MetricTile";
import styles from "./AgentPerformanceGrid.module.scss";

export default function AgentPerformanceGrid({
  agents: externalAgents,
  isLoading: externalLoading,
  error: externalError,
  onAgentSelect,
  selectedAgentId,
}: AgentPerformanceGridProps) {
  const [agents, setAgents] = useState<AgentPerformance[]>(
    externalAgents ?? []
  );
  const [isLoading, setIsLoading] = useState(
    externalLoading ?? !externalAgents
  );
  const [error, setError] = useState<string | null>(externalError ?? null);

  // Load agent performance if not provided externally
  const loadAgentPerformance = async () => {
    if (externalAgents) return; // Use external data if provided

    try {
      setIsLoading(true);
      setError(null);

      const agentData = await metricsApiClient.getAgentPerformance();
      setAgents(agentData);
    } catch (err) {
      const errorMessage =
        err instanceof MetricsApiError
          ? err.message
          : "Failed to load agent performance";

      setError(errorMessage);
      console.error("Failed to load agent performance:", err);
    } finally {
      setIsLoading(false);
    }
  };

  // Initial load and external prop updates
  useEffect(() => {
    if (externalAgents) {
      setAgents(externalAgents);
      setIsLoading(false);
      setError(null);
    } else if (!externalAgents && !isLoading) {
      // For now, load mock data directly to test rendering
      const loadMockData = async () => {
        try {
          setIsLoading(true);
          setError(null);
          const agentData = await metricsApiClient.getAgentPerformance();
          setAgents(agentData);
        } catch (err) {
          console.error("Failed to load agent performance:", err);
          setError("Failed to load agent performance");
        } finally {
          setIsLoading(false);
        }
      };
      loadMockData();
    }
  }, [externalAgents]);

  // Get agent type icon
  const getAgentTypeIcon = (type: AgentPerformance["type"]) => {
    switch (type) {
      case "planning":
        return "🧠";
      case "execution":
        return "⚡";
      case "coordination":
        return "🤝";
      case "validation":
        return "✅";
      case "specialized":
        return "🔧";
      default:
        return "🤖";
    }
  };

  // Get status color
  const getStatusColor = (status: AgentPerformance["status"]) => {
    switch (status) {
      case "active":
        return styles.active;
      case "idle":
        return styles.idle;
      case "error":
        return styles.error;
      case "maintenance":
        return styles.maintenance;
      default:
        return styles.unknown;
    }
  };

  // Format performance score for status
  const getPerformanceStatus = (
    score: number
  ): "success" | "warning" | "error" => {
    if (score >= 85) return "success";
    if (score >= 70) return "warning";
    return "error";
  };

  if (isLoading) {
    return (
      <div className={styles.agentGrid}>
        <div className={styles.loading}>
          <div className={styles.loadingSpinner}></div>
          <p>Loading agent performance...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.agentGrid}>
        <div className={styles.error}>
          <h3>Failed to load agent performance</h3>
          <p>{error}</p>
          <button onClick={loadAgentPerformance}>Retry</button>
        </div>
      </div>
    );
  }

  if (!agents || agents.length === 0) {
    return (
      <div className={styles.agentGrid}>
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>🤖</div>
          <h3>Agent Performance Monitoring</h3>
          <p>
            Real-time agent performance metrics require V3 agent monitoring
            APIs.
          </p>
          <div className={styles.emptyActions}>
            <button className={styles.secondaryButton} disabled>
              Connect to Agent Metrics API
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.agentGrid}>
      <div className={styles.header}>
        <h2>Agent Performance</h2>
        <div className={styles.summary}>
          <span className={styles.summaryText}>
            {agents.length} agents •{" "}
            {agents.filter((a) => a.status === "active").length} active
          </span>
        </div>
      </div>

      <div className={styles.agentsList}>
        {agents.map((agent) => (
          <div
            key={agent.agent_id}
            className={`${styles.agentCard} ${
              selectedAgentId === agent.agent_id ? styles.selected : ""
            } ${getStatusColor(agent.status)}`}
            onClick={() => onAgentSelect?.(agent.agent_id)}
            role={onAgentSelect ? "button" : undefined}
            tabIndex={onAgentSelect ? 0 : undefined}
          >
            <div className={styles.agentHeader}>
              <div className={styles.agentIdentity}>
                <span className={styles.agentIcon}>
                  {getAgentTypeIcon(agent.type)}
                </span>
                <div className={styles.agentInfo}>
                  <h3 className={styles.agentName}>{agent.name}</h3>
                  <span className={styles.agentType}>
                    {agent.type.charAt(0).toUpperCase() + agent.type.slice(1)}{" "}
                    Agent
                  </span>
                </div>
              </div>

              <div
                className={`${styles.agentStatus} ${getStatusColor(
                  agent.status
                )}`}
              >
                <span className={styles.statusDot}></span>
                <span className={styles.statusText}>
                  {agent.status.charAt(0).toUpperCase() + agent.status.slice(1)}
                </span>
              </div>
            </div>

            <div className={styles.agentMetrics}>
              <div className={styles.metricsRow}>
                <MetricTile
                  title="Success Rate"
                  value={`${(agent.success_rate * 100).toFixed(1)}%`}
                  status={
                    agent.success_rate >= 0.95
                      ? "success"
                      : agent.success_rate >= 0.8
                      ? "warning"
                      : "error"
                  }
                  format="percentage"
                />

                <MetricTile
                  title="Response Time"
                  value={agent.average_response_time_ms}
                  format="duration"
                  status={
                    agent.average_response_time_ms < 1000
                      ? "success"
                      : agent.average_response_time_ms < 5000
                      ? "warning"
                      : "error"
                  }
                />

                <MetricTile
                  title="Tasks/Hour"
                  value={agent.throughput_per_hour}
                  status={
                    agent.throughput_per_hour > 10
                      ? "success"
                      : agent.throughput_per_hour > 5
                      ? "warning"
                      : "error"
                  }
                />
              </div>

              <div className={styles.metricsRow}>
                <MetricTile
                  title="CPU Usage"
                  value={`${agent.cpu_usage_percent.toFixed(1)}%`}
                  format="percentage"
                  status={
                    agent.cpu_usage_percent < 70
                      ? "success"
                      : agent.cpu_usage_percent < 90
                      ? "warning"
                      : "error"
                  }
                />

                <MetricTile
                  title="Memory"
                  value={agent.memory_usage_mb}
                  format="bytes"
                  status={
                    agent.memory_usage_mb < 500
                      ? "success"
                      : agent.memory_usage_mb < 1000
                      ? "warning"
                      : "error"
                  }
                />

                <MetricTile
                  title="Efficiency"
                  value={`${agent.efficiency_score}%`}
                  format="percentage"
                  status={getPerformanceStatus(agent.efficiency_score)}
                />
              </div>

              <div className={styles.agentStats}>
                <div className={styles.stat}>
                  <span className={styles.statLabel}>Completed:</span>
                  <span className={styles.statValue}>
                    {agent.tasks_completed}
                  </span>
                </div>
                <div className={styles.stat}>
                  <span className={styles.statLabel}>Failed:</span>
                  <span className={styles.statValue}>{agent.tasks_failed}</span>
                </div>
                <div className={styles.stat}>
                  <span className={styles.statLabel}>Active Connections:</span>
                  <span className={styles.statValue}>
                    {agent.active_connections}
                  </span>
                </div>
              </div>

              {agent.last_error && (
                <div className={styles.errorSection}>
                  <h4>Last Error</h4>
                  <p className={styles.errorMessage}>{agent.last_error}</p>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
