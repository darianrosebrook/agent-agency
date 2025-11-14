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
  getAgentsTaskCompletion,
  type AgentStats as AgentStatsType,
  type Agent,
  type AgentActivityPoint,
  type ModelContribution,
  type ModelContributionsResponse,
  type ContributionStats,
  type ContributionsResponse,
  type EfficiencyResponse,
  type TaskCompletionResponse,
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
    ModelContribution[] | ModelContributionsResponse | null
  >(null);
  const [contributions, setContributions] = useState<ContributionStats[] | ContributionsResponse | null>(null);
  const [efficiency, setEfficiency] = useState<EfficiencyResponse | null>(null);
  const [taskCompletion, setTaskCompletion] =
    useState<TaskCompletionResponse | null>(null);
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
        const hours =
          timeRange === "7d"
            ? 24 * 7
            : timeRange === "30d"
            ? 24 * 30
            : timeRange === "90d"
            ? 24 * 90
            : 24 * 365;

        const [
          statsData,
          agentsData,
          activityData,
          modelData,
          contributionsData,
          efficiencyData,
          completionData,
        ] = await Promise.all([
          getAgentsStats(),
          getAgents(),
          getAgentActivity({ start_date: getStartDate(timeRange) }),
          getModelContributions(),
          getContributions({ start_date: getStartDate(timeRange) }),
          getEfficiencyMetrics({ hours }),
          getAgentsTaskCompletion({ hours }),
        ]);

        setStats(statsData);
        // Ensure agents is an array - handle case where API returns object with agents property
        const agentsArray = Array.isArray(agentsData) 
          ? agentsData 
          : (agentsData?.agents || []);
        setAgents(agentsArray);
        setActivity(activityData);
        // Handle both array and object responses from getModelContributions
        const modelArray = Array.isArray(modelData) 
          ? modelData 
          : (modelData?.models || modelData?.monthly_contributions || []);
        setModelContributions(modelArray);
        // Handle both array and object responses from getContributions
        setContributions(Array.isArray(contributionsData) ? contributionsData : contributionsData as ContributionsResponse);
        setEfficiency(efficiencyData);
        setTaskCompletion(completionData);
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
                const hours =
                  timeRange === "7d"
                    ? 24 * 7
                    : timeRange === "30d"
                    ? 24 * 30
                    : timeRange === "90d"
                    ? 24 * 90
                    : 24 * 365;

                const [
                  statsData,
                  agentsData,
                  activityData,
                  modelData,
                  contributionsData,
                  efficiencyData,
                  completionData,
                ] = await Promise.all([
                  getAgentsStats(),
                  getAgents(),
                  getAgentActivity({ start_date: getStartDate(timeRange) }),
                  getModelContributions(),
                  getContributions({ start_date: getStartDate(timeRange) }),
                  getEfficiencyMetrics({ hours }),
                  getAgentsTaskCompletion({ hours }),
                ]);
                setStats(statsData);
                // Ensure agents is an array - handle case where API returns object with agents property
                const agentsArray = Array.isArray(agentsData) 
                  ? agentsData 
                  : (agentsData?.agents || []);
                setAgents(agentsArray);
                setActivity(activityData);
                // Handle both array and object responses from getModelContributions
                const modelArray = Array.isArray(modelData) 
                  ? modelData 
                  : (modelData?.models || modelData?.monthly_contributions || []);
                setModelContributions(modelArray);
                // Handle both array and object responses from getContributions
        setContributions(Array.isArray(contributionsData) ? contributionsData : contributionsData as ContributionsResponse);
                setEfficiency(efficiencyData);
                setTaskCompletion(completionData);
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

  // Ensure contributions is an array - handle case where API returns ContributionsResponse object
  const contributionsArray = Array.isArray(contributions) 
    ? contributions 
    : [];

  // Ensure modelContributions is an array - handle case where API returns ModelContributionsResponse object
  const modelContributionsArray = Array.isArray(modelContributions)
    ? modelContributions
    : [];

  const filteredContributions = selectedAgentId
    ? contributionsArray.filter((c) => c.agent_id === selectedAgentId)
    : contributionsArray;

  const totalLinesChanged = filteredContributions.reduce(
    (sum, c) => sum + (c.lines_added || 0) + (c.lines_modified || 0) + (c.lines_deleted || 0),
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
        {stats && stats.by_type && Object.keys(stats.by_type).length > 0 && (
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
        {modelContributionsArray.length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Model Usage</h2>
            <div className={styles.modelList}>
              {modelContributionsArray.map((model) => (
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

        {/* Task Completion Metrics */}
        {taskCompletion && taskCompletion.agents && taskCompletion.agents.length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Task Completion Metrics</h2>
            {taskCompletion.summary && (
              <div className={styles.summaryMetrics}>
                <div className={styles.summaryMetric}>
                  <span className={styles.metricLabel}>
                    Overall Completion Rate:
                  </span>
                  <span className={styles.metricValue}>
                    {taskCompletion.summary.overall_completion_rate.toFixed(1)}%
                  </span>
                </div>
                <div className={styles.summaryMetric}>
                  <span className={styles.metricLabel}>
                    Overall Success Rate:
                  </span>
                  <span className={styles.metricValue}>
                    {taskCompletion.summary.overall_success_rate.toFixed(1)}%
                  </span>
                </div>
                <div className={styles.summaryMetric}>
                  <span className={styles.metricLabel}>Total Tasks:</span>
                  <span className={styles.metricValue}>
                    {taskCompletion.summary.total_tasks.toLocaleString()}
                  </span>
                </div>
              </div>
            )}
            <div className={styles.completionList}>
              {taskCompletion.agents.map((metric) => (
                <div key={metric.agent_id} className={styles.completionItem}>
                  <div className={styles.completionHeader}>
                    <span className={styles.agentName}>
                      {metric.agent_name}
                    </span>
                    <span className={styles.workerType}>
                      {metric.worker_type}
                    </span>
                  </div>
                  <div className={styles.completionMetrics}>
                    <div className={styles.completionMetric}>
                      <span className={styles.metricLabel}>Total Tasks:</span>
                      <span className={styles.metricValue}>
                        {metric.total_tasks}
                      </span>
                    </div>
                    <div className={styles.completionMetric}>
                      <span className={styles.metricLabel}>Completed:</span>
                      <span className={styles.metricValuePositive}>
                        {metric.completed_tasks}
                      </span>
                    </div>
                    <div className={styles.completionMetric}>
                      <span className={styles.metricLabel}>Failed:</span>
                      <span className={styles.metricValueNegative}>
                        {metric.failed_tasks}
                      </span>
                    </div>
                    <div className={styles.completionMetric}>
                      <span className={styles.metricLabel}>
                        Completion Rate:
                      </span>
                      <span className={styles.metricValue}>
                        {metric.completion_rate_percent.toFixed(1)}%
                      </span>
                    </div>
                    <div className={styles.completionMetric}>
                      <span className={styles.metricLabel}>Success Rate:</span>
                      <span className={styles.metricValue}>
                        {(metric.success_rate * 100).toFixed(1)}%
                      </span>
                    </div>
                    {metric.avg_execution_time_ms && (
                      <div className={styles.completionMetric}>
                        <span className={styles.metricLabel}>
                          Avg Execution Time:
                        </span>
                        <span className={styles.metricValue}>
                          {(metric.avg_execution_time_ms / 1000).toFixed(2)}s
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </BentoPanel>
        )}

        {/* Efficiency Metrics */}
        {efficiency && efficiency.agents && efficiency.agents.length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Efficiency Metrics</h2>
            {efficiency.summary && (
              <div className={styles.summaryMetrics}>
                <div className={styles.summaryMetric}>
                  <span className={styles.metricLabel}>
                    Overall Tasks/Hour:
                  </span>
                  <span className={styles.metricValue}>
                    {efficiency.summary.overall_tasks_per_hour.toFixed(2)}
                  </span>
                </div>
                <div className={styles.summaryMetric}>
                  <span className={styles.metricLabel}>
                    Avg Efficiency Score:
                  </span>
                  <span className={styles.metricValue}>
                    {efficiency.summary.avg_efficiency_score.toFixed(2)}
                  </span>
                </div>
              </div>
            )}
            <div className={styles.efficiencyList}>
              {efficiency.agents.map((metric) => (
                <div key={metric.agent_id} className={styles.efficiencyItem}>
                  <div className={styles.efficiencyHeader}>
                    <span className={styles.agentName}>
                      {metric.agent_name}
                    </span>
                    <span className={styles.workerType}>
                      {metric.worker_type}
                    </span>
                  </div>
                  <div className={styles.efficiencyMetrics}>
                    <div className={styles.efficiencyMetric}>
                      <span className={styles.metricLabel}>Tasks/Hour:</span>
                      <span className={styles.metricValue}>
                        {metric.tasks_per_hour.toFixed(2)}
                      </span>
                    </div>
                    <div className={styles.efficiencyMetric}>
                      <span className={styles.metricLabel}>
                        Efficiency Score:
                      </span>
                      <span className={styles.metricValue}>
                        {metric.efficiency_score.toFixed(2)}
                      </span>
                    </div>
                    <div className={styles.efficiencyMetric}>
                      <span className={styles.metricLabel}>Success Rate:</span>
                      <span className={styles.metricValue}>
                        {(metric.success_rate * 100).toFixed(1)}%
                      </span>
                    </div>
                    {metric.avg_execution_time_ms && (
                      <div className={styles.efficiencyMetric}>
                        <span className={styles.metricLabel}>
                          Avg Execution Time:
                        </span>
                        <span className={styles.metricValue}>
                          {(metric.avg_execution_time_ms / 1000).toFixed(2)}s
                        </span>
                      </div>
                    )}
                    {metric.p95_execution_time_ms && (
                      <div className={styles.efficiencyMetric}>
                        <span className={styles.metricLabel}>
                          P95 Execution Time:
                        </span>
                        <span className={styles.metricValue}>
                          {(metric.p95_execution_time_ms / 1000).toFixed(2)}s
                        </span>
                      </div>
                    )}
                    {metric.total_tokens_used && (
                      <div className={styles.efficiencyMetric}>
                        <span className={styles.metricLabel}>
                          Total Tokens:
                        </span>
                        <span className={styles.metricValue}>
                          {metric.total_tokens_used.toLocaleString()}
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              ))}
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
