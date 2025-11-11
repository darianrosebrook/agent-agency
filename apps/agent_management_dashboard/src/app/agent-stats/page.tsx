"use client";

import { useState, useEffect } from "react";
import styles from "./page.module.scss";
import {
  getAgentsStats,
  getAgents,
  getAgentActivity,
  getModelContributions,
  getContributions,
  getEfficiencyMetrics,
  type AgentStats as AgentStatsType,
  type Agent,
  type AgentActivityPoint,
  type ModelContribution,
  type ContributionStats,
  type EfficiencyMetrics,
} from "../../lib/api/agents";
import { ErrorDisplay } from "../../components/ErrorDisplay";
import { BentoPanel } from "../../components/compounds/BentoPanel";
import { AgentActivityChart } from "../../components/AgentActivityChart";

/**
 * Agent Stats Page
 *
 * Comprehensive analytics and statistics about AI agents,
 * their performance, usage patterns, and contribution metrics.
 *
 * @author @darianrosebrook
 */
export default function AgentStatsPage() {
  const [stats, setStats] = useState<AgentStatsType | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [activity, setActivity] = useState<AgentActivityPoint[]>([]);
  const [modelContributions, setModelContributions] = useState<
    ModelContribution[]
  >([]);
  const [contributions, setContributions] = useState<ContributionStats[]>([]);
  const [efficiency, setEfficiency] = useState<EfficiencyMetrics[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [timeRange, setTimeRange] = useState<"7d" | "30d" | "90d" | "all">(
    "30d"
  );

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        const [
          statsData,
          agentsData,
          activityData,
          modelData,
          contributionsData,
          efficiencyData,
        ] = await Promise.all([
          getAgentsStats(),
          getAgents(),
          getAgentActivity({ start_date: getStartDate(timeRange) }),
          getModelContributions(),
          getContributions({ start_date: getStartDate(timeRange) }),
          getEfficiencyMetrics(),
        ]);

        setStats(statsData);
        setAgents(agentsData);
        setActivity(activityData);
        setModelContributions(modelData);
        setContributions(contributionsData);
        setEfficiency(efficiencyData);
      } catch (err) {
        setError(
          err instanceof Error
            ? err
            : new Error("Failed to load agent statistics")
        );
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, [timeRange]);

  function getStartDate(range: string): string {
    const now = new Date();
    const days =
      range === "7d" ? 7 : range === "30d" ? 30 : range === "90d" ? 90 : 365;
    const startDate = new Date(now);
    startDate.setDate(startDate.getDate() - days);
    return startDate.toISOString().split("T")[0];
  }

  if (isLoading) {
    return (
      <div className={styles.agentStatsPage}>
        <div className={styles.container}>
          <div className={styles.header}>
            <h1 className={styles.headerTitle}>Agent Stats</h1>
            <p className={styles.headerDescription}>
              Loading agent statistics...
            </p>
          </div>
          <div className={styles.loadingState}>
            <div className={styles.spinner}></div>
            <p>Loading data...</p>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.agentStatsPage}>
        <div className={styles.container}>
          <div className={styles.header}>
            <h1 className={styles.headerTitle}>Agent Stats</h1>
          </div>
          <ErrorDisplay
            error={error}
            onRetry={async () => {
              setIsLoading(true);
              setError(null);
              try {
                const [
                  statsData,
                  agentsData,
                  activityData,
                  modelData,
                  contributionsData,
                  efficiencyData,
                ] = await Promise.all([
                  getAgentsStats(),
                  getAgents(),
                  getAgentActivity({ start_date: getStartDate(timeRange) }),
                  getModelContributions(),
                  getContributions({ start_date: getStartDate(timeRange) }),
                  getEfficiencyMetrics(),
                ]);
                setStats(statsData);
                setAgents(agentsData);
                setActivity(activityData);
                setModelContributions(modelData);
                setContributions(contributionsData);
                setEfficiency(efficiencyData);
              } catch (err) {
                setError(
                  err instanceof Error
                    ? err
                    : new Error("Failed to load agent statistics")
                );
              } finally {
                setIsLoading(false);
              }
            }}
          />
        </div>
      </div>
    );
  }

  const filteredContributions = selectedAgentId
    ? contributions.filter((c) => c.agent_id === selectedAgentId)
    : contributions;

  const totalLinesChanged = filteredContributions.reduce(
    (sum, c) => sum + c.lines_added + c.lines_modified + c.lines_deleted,
    0
  );

  return (
    <div className={styles.agentStatsPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.headerTitle}>Agent Stats</h1>
          <p className={styles.headerDescription}>
            Comprehensive analytics and performance metrics for AI agents
          </p>
        </div>

        {/* Controls */}
        <div className={styles.controls}>
          <div className={styles.controlGroup}>
            <label htmlFor="timeRange" className={styles.controlLabel}>
              Time Range:
            </label>
            <select
              id="timeRange"
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as typeof timeRange)}
              className={styles.select}
            >
              <option value="7d">Last 7 days</option>
              <option value="30d">Last 30 days</option>
              <option value="90d">Last 90 days</option>
              <option value="all">All time</option>
            </select>
          </div>

          <div className={styles.controlGroup}>
            <label htmlFor="agentFilter" className={styles.controlLabel}>
              Agent:
            </label>
            <select
              id="agentFilter"
              value={selectedAgentId ?? ""}
              onChange={(e) => setSelectedAgentId(e.target.value || null)}
              className={styles.select}
            >
              <option value="">All Agents</option>
              {agents.map((agent) => (
                <option key={agent.id} value={agent.id}>
                  {agent.name}
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Overview Metrics */}
        {stats && (
          <div className={styles.metricsGrid}>
            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Total Agents</div>
              <div className={styles.metricValue}>{stats.total}</div>
            </BentoPanel>

            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Active Agents</div>
              <div className={styles.metricValue}>{stats.active}</div>
            </BentoPanel>

            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Inactive Agents</div>
              <div className={styles.metricValue}>{stats.inactive}</div>
            </BentoPanel>

            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Total Lines Changed</div>
              <div className={styles.metricValue}>
                {totalLinesChanged.toLocaleString()}
              </div>
            </BentoPanel>
          </div>
        )}

        {/* Agent Type Breakdown */}
        {stats && Object.keys(stats.by_type).length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Agents by Type</h2>
            <div className={styles.typeList}>
              {Object.entries(stats.by_type).map(([type, count]) => (
                <div key={type} className={styles.typeItem}>
                  <span className={styles.typeName}>{type}</span>
                  <span className={styles.typeCount}>{count}</span>
                </div>
              ))}
            </div>
          </BentoPanel>
        )}

        {/* Model Usage */}
        {modelContributions.length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Model Usage</h2>
            <div className={styles.modelList}>
              {modelContributions.map((model) => (
                <div key={model.model_name} className={styles.modelItem}>
                  <div className={styles.modelHeader}>
                    <span className={styles.modelName}>{model.model_name}</span>
                    <span className={styles.modelTasks}>
                      {model.task_count} tasks
                    </span>
                  </div>
                  <div className={styles.modelMetrics}>
                    <div className={styles.modelMetric}>
                      <span className={styles.metricLabel}>Success Rate:</span>
                      <span className={styles.metricValue}>
                        {model.success_rate.toFixed(1)}%
                      </span>
                    </div>
                    <div className={styles.modelMetric}>
                      <span className={styles.metricLabel}>
                        Avg Completion:
                      </span>
                      <span className={styles.metricValue}>
                        {model.avg_completion_time.toFixed(1)}s
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </BentoPanel>
        )}

        {/* Code Contributions */}
        {filteredContributions.length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Code Contributions</h2>
            <div className={styles.contributionsList}>
              {filteredContributions.map((contribution) => (
                <div
                  key={contribution.agent_id}
                  className={styles.contributionItem}
                >
                  <div className={styles.contributionHeader}>
                    <span className={styles.agentName}>
                      {contribution.agent_name}
                    </span>
                  </div>
                  <div className={styles.contributionMetrics}>
                    <div className={styles.contributionMetric}>
                      <span className={styles.metricLabel}>Lines Added:</span>
                      <span className={styles.metricValuePositive}>
                        +{contribution.lines_added.toLocaleString()}
                      </span>
                    </div>
                    <div className={styles.contributionMetric}>
                      <span className={styles.metricLabel}>
                        Lines Modified:
                      </span>
                      <span className={styles.metricValueNeutral}>
                        {contribution.lines_modified.toLocaleString()}
                      </span>
                    </div>
                    <div className={styles.contributionMetric}>
                      <span className={styles.metricLabel}>Lines Deleted:</span>
                      <span className={styles.metricValueNegative}>
                        -{contribution.lines_deleted.toLocaleString()}
                      </span>
                    </div>
                    <div className={styles.contributionMetric}>
                      <span className={styles.metricLabel}>Files Changed:</span>
                      <span className={styles.metricValue}>
                        {contribution.files_changed}
                      </span>
                    </div>
                    <div className={styles.contributionMetric}>
                      <span className={styles.metricLabel}>Commits:</span>
                      <span className={styles.metricValue}>
                        {contribution.commits}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </BentoPanel>
        )}

        {/* Efficiency Metrics */}
        {efficiency.length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Efficiency Metrics</h2>
            <div className={styles.efficiencyList}>
              {efficiency.map((metric) => {
                const agent = agents.find((a) => a.id === metric.agent_id);
                return (
                  <div key={metric.agent_id} className={styles.efficiencyItem}>
                    <div className={styles.efficiencyHeader}>
                      <span className={styles.agentName}>
                        {agent?.name ?? metric.agent_id}
                      </span>
                    </div>
                    <div className={styles.efficiencyMetrics}>
                      <div className={styles.efficiencyMetric}>
                        <span className={styles.metricLabel}>
                          Efficiency Score:
                        </span>
                        <span className={styles.metricValue}>
                          {(metric.efficiency_score * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className={styles.efficiencyMetric}>
                        <span className={styles.metricLabel}>
                          Resource Utilization:
                        </span>
                        <span className={styles.metricValue}>
                          {(metric.resource_utilization * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className={styles.efficiencyMetric}>
                        <span className={styles.metricLabel}>Throughput:</span>
                        <span className={styles.metricValue}>
                          {metric.throughput.toFixed(2)} tasks/s
                        </span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </BentoPanel>
        )}

        {/* Agent Activity Chart */}
        {activity.length > 0 && (
          <BentoPanel className={styles.section}>
            <AgentActivityChart
              title="Agent Activity Over Time"
              subtitle={`${activity.length} data points for ${
                timeRange === "7d"
                  ? "last 7 days"
                  : timeRange === "30d"
                  ? "last 30 days"
                  : timeRange === "90d"
                  ? "last 90 days"
                  : "all time"
              }`}
              days={
                timeRange === "7d"
                  ? 7
                  : timeRange === "30d"
                  ? 30
                  : timeRange === "90d"
                  ? 90
                  : 365
              }
            />
          </BentoPanel>
        )}

        {/* Empty State */}
        {!stats && agents.length === 0 && (
          <BentoPanel className={styles.section}>
            <div className={styles.emptyState}>
              <p className={styles.emptyText}>No agent data available</p>
              <p className={styles.emptyDescription}>
                Agent statistics will appear here once agents are registered and
                active.
              </p>
            </div>
          </BentoPanel>
        )}
      </div>
    </div>
  );
}
