"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { useState, useEffect, useMemo } from "react";
import { getAgentActivity, type AgentActivityPoint } from "../lib/api/agents";
import { getAgents, type Agent } from "../lib/api/agents";
import styles from "./AgentActivityChart.module.scss";

interface ActivityDataPoint {
  timestamp: string;
  [agentId: string]: string | number;
}

interface AgentActivityChartProps {
  title?: string;
  subtitle?: string;
  days?: number;
}

export function AgentActivityChart({
  title = "Agent Activity",
  subtitle = "Activity over time",
  days = 7,
}: AgentActivityChartProps) {
  const [activityData, setActivityData] = useState<AgentActivityPoint[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        const startDate = new Date();
        startDate.setDate(startDate.getDate() - days);
        const startDateStr = startDate.toISOString().split("T")[0];

        const [activityResponse, agentsListResponse] = await Promise.all([
          getAgentActivity({
            start_date: startDateStr,
          }),
          getAgents(),
        ]);

        // Safely extract arrays
        const activity: AgentActivityPoint[] = Array.isArray(activityResponse)
          ? activityResponse
          : [];
        
        const agentsList: Agent[] = Array.isArray(agentsListResponse)
          ? agentsListResponse
          : [];

        setActivityData(activity);
        setAgents(agentsList);
      } catch (err) {
        console.error("Failed to fetch agent activity data:", err);
        setError(err instanceof Error ? err : new Error("Failed to load agent activity data"));
        setActivityData([]);
        setAgents([]);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
    // Refresh every 30 seconds
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, [days]);

  // Transform activity data into chart format
  const chartData = useMemo(() => {
    // Ensure both are arrays before checking length
    const safeActivityData = Array.isArray(activityData) ? activityData : [];
    const safeAgents = Array.isArray(agents) ? agents : [];
    
    if (safeActivityData.length === 0 || safeAgents.length === 0) {
      return [];
    }

    // Group activity by timestamp and agent
    const activityMap: Record<string, Record<string, number>> = {};

    safeActivityData.forEach((point) => {
      const timestamp = point.timestamp.split("T")[0]; // Use date only
      if (!activityMap[timestamp]) {
        activityMap[timestamp] = {};
      }
      if (!activityMap[timestamp][point.agent_id]) {
        activityMap[timestamp][point.agent_id] = 0;
      }
      activityMap[timestamp][point.agent_id] += point.count;
    });

    // Convert to array format for chart
    const dataPoints: ActivityDataPoint[] = Object.entries(activityMap).map(
      ([timestamp, agentCounts]) => {
        const dataPoint: ActivityDataPoint = { timestamp };
        safeAgents.forEach((agent) => {
          dataPoint[agent.id] = agentCounts[agent.id] || 0;
        });
        return dataPoint;
      }
    );

    // Sort by timestamp
    dataPoints.sort((a, b) => a.timestamp.localeCompare(b.timestamp));

    // Format timestamps for display
    return dataPoints.map((point) => ({
      ...point,
      timestamp: new Date(point.timestamp).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      }),
    }));
  }, [activityData, agents]);

  // Generate colors for agents
  const agentColors = useMemo(() => {
    const colors = [
      "#6366f1", // indigo
      "#8b5cf6", // violet
      "#ec4899", // pink
      "#f59e0b", // amber
      "#10b981", // emerald
      "#3b82f6", // blue
    ];
    return agents.reduce((acc, agent, index) => {
      acc[agent.id] = colors[index % colors.length];
      return acc;
    }, {} as Record<string, string>);
  }, [agents]);

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload?.length) {
      return (
        <div className={styles.tooltip}>
          <div className={styles.tooltipLabel}>{label}</div>
          {payload.map((entry: any, index: number) => {
            const agentId = entry.dataKey;
            const agent = agents.find((a) => a.id === agentId);
            return (
              <div key={index} className={styles.tooltipItem}>
                <div
                  className={styles.tooltipDot}
                  style={{ backgroundColor: entry.color }}
                />
                <span className={styles.tooltipName}>
                  {agent?.name ?? agentId}:
                </span>
                <span className={styles.tooltipValue}>{entry.value}</span>
              </div>
            );
          })}
        </div>
      );
    }
    return null;
  };

  if (isLoading) {
    return (
      <div className={styles.container}>
        <div className={styles.innerContainer}>
          <div className={styles.content}>
            <div className={styles.header}>
              <h3 className={styles.title}>{title}</h3>
              <p className={styles.subtitle}>Loading agent activity data...</p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (error || chartData.length === 0) {
    return (
      <div className={styles.container}>
        <div className={styles.innerContainer}>
          <div className={styles.content}>
            <div className={styles.header}>
              <h3 className={styles.title}>{title}</h3>
              <p className={styles.subtitle}>
                {error ? `Error: ${error.message}` : "No activity data available"}
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.innerContainer}>
        <div className={styles.content}>
          {/* Header */}
          <div className={styles.header}>
            <h3 className={styles.title}>{title}</h3>
            <p className={styles.subtitle}>{subtitle}</p>
          </div>

          {/* Chart */}
          <div className={styles.chart}>
            <ResponsiveContainer width="100%" height="100%">
              <LineChart
                data={chartData}
                margin={{
                  top: 10,
                  right: 30,
                  left: 10,
                  bottom: 10,
                }}
              >
                <CartesianGrid
                  strokeDasharray="0"
                  stroke="#39393b"
                  horizontal={true}
                  vertical={false}
                />
                <XAxis
                  dataKey="timestamp"
                  stroke="#9e9ea0"
                  tick={{ fill: "#9e9ea0", fontSize: 10 }}
                  axisLine={{ stroke: "#39393b" }}
                  tickLine={false}
                  interval="preserveStartEnd"
                  minTickGap={50}
                />
                <YAxis
                  stroke="#9e9ea0"
                  tick={{ fill: "#9e9ea0", fontSize: 11 }}
                  axisLine={false}
                  tickLine={false}
                />
                <Tooltip
                  content={<CustomTooltip />}
                  cursor={{
                    stroke: "white",
                    strokeWidth: 1,
                    strokeDasharray: "0",
                  }}
                />
                <Legend
                  wrapperStyle={{ paddingTop: "20px" }}
                  iconType="line"
                  formatter={(value) => {
                    const agent = agents.find((a) => a.id === value);
                    return agent?.name ?? value;
                  }}
                />
                {agents.map((agent) => (
                  <Line
                    key={agent.id}
                    type="monotone"
                    dataKey={agent.id}
                    stroke={agentColors[agent.id]}
                    strokeWidth={2}
                    dot={false}
                    name={agent.id}
                  />
                ))}
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  );
}

