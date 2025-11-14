"use client";

import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { useState, useEffect } from "react";
import { getModelContributions, type ModelContribution } from "../lib/api/agents";
import styles from "./ModelContributionStream.module.scss";

interface StreamDataPoint {
  month: string;
  gemma3n: number;
  qwen: number;
  instruct: number;
  mistral: number;
}

interface ModelContributionStreamProps {
  title?: string;
  subtitle?: string;
}

export function ModelContributionStream({
  title = "Model Contributions",
  subtitle = "Lines of code by AI model",
}: ModelContributionStreamProps) {
  const [data, setData] = useState<StreamDataPoint[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        const modelContributionsResponse = await getModelContributions();

        // Safely extract modelContributions array
        const modelContributions = Array.isArray(modelContributionsResponse?.contributions)
          ? modelContributionsResponse.contributions
          : Array.isArray(modelContributionsResponse)
          ? modelContributionsResponse
          : [];

        // Transform API response to stream chart format
        // Map model names to chart keys (normalize to lowercase, handle variations)
        const modelMap: Record<string, string> = {
          'gemma3n': 'gemma3n',
          'gemma': 'gemma3n',
          'qwen': 'qwen',
          'instruct': 'instruct',
          'mistral': 'mistral',
        };

        // Group by month (simplified - API might need to return monthly data)
        // For now, we'll create monthly data from the contributions
        const months = ["Jan", "Feb", "Mar", "Apr"];
        const monthlyData: StreamDataPoint[] = months.map((month) => ({
          month,
          gemma3n: 0,
          qwen: 0,
          instruct: 0,
          mistral: 0,
        }));

        // Distribute contributions across months
        modelContributions.forEach((contrib: ModelContribution) => {
          const modelKey = modelMap[contrib.model_name.toLowerCase()] || contrib.model_name.toLowerCase();
          if (modelKey in monthlyData[0]) {
            // Distribute task_count (as proxy for lines of code) across months
            const avgPerMonth = contrib.task_count / months.length;
            monthlyData.forEach((monthData) => {
              (monthData as any)[modelKey] += Math.round(avgPerMonth);
            });
          }
        });

        setData(monthlyData.length > 0 ? monthlyData : generateFallbackData());
      } catch (err) {
        console.error("Failed to fetch model contribution data:", err);
        setError(err instanceof Error ? err : new Error("Failed to load model contribution data"));
        setData(generateFallbackData());
      } finally {
        setIsLoading(false);
      }
    }

    function generateFallbackData(): StreamDataPoint[] {
      return [
        { month: "Jan", gemma3n: 0, qwen: 0, instruct: 0, mistral: 0 },
        { month: "Feb", gemma3n: 0, qwen: 0, instruct: 0, mistral: 0 },
        { month: "Mar", gemma3n: 0, qwen: 0, instruct: 0, mistral: 0 },
        { month: "Apr", gemma3n: 0, qwen: 0, instruct: 0, mistral: 0 },
      ];
    }

    fetchData();
  }, []);

  // Model colors - purple-blue palette matching heatmap
  const models = [
    { name: "gemma3n", color: "#e0e7ff", label: "Gemma 3N" },
    { name: "qwen", color: "#a5b4fc", label: "Qwen" },
    { name: "instruct", color: "#6366f1", label: "Instruct" },
    { name: "mistral", color: "#4338ca", label: "Mistral" },
  ];

  // Custom tooltip
  const CustomTooltip = ({
    active,
    payload,
    label,
  }: {
    active?: boolean;
    payload?: Array<{ dataKey?: string; value?: number; color?: string }>;
    label?: string;
  }) => {
    if (active && payload?.length) {
      const total = payload.reduce(
        (sum: number, item) => sum + (item.value ?? 0),
        0
      );

      return (
        <div className={styles.tooltip}>
          <div className={styles.tooltipLabel}>{label}</div>
          {payload.reverse().map((item) => {
            const model = models.find((m) => m.name === item.dataKey);
            return (
              <div key={item.dataKey} className={styles.tooltipItem}>
                <div
                  className={styles.tooltipDot}
                  style={{ backgroundColor: item.color }}
                />
                <span className={styles.tooltipText}>
                  {model?.label}: {item.value}
                </span>
              </div>
            );
          })}
          <div className={styles.tooltipTotal}>Total: {total}</div>
        </div>
      );
    }
    return null;
  };

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
              <AreaChart
                data={data}
                margin={{
                  top: 5,
                  right: 10,
                  left: 0,
                  bottom: 5,
                }}
                stackOffset="silhouette"
              >
                <CartesianGrid
                  strokeDasharray="0"
                  stroke="#39393b"
                  horizontal={true}
                  vertical={false}
                />
                <XAxis
                  dataKey="month"
                  stroke="#9e9ea0"
                  tick={{ fill: "#9e9ea0", fontSize: 10 }}
                  axisLine={{ stroke: "#39393b" }}
                  tickLine={false}
                />
                <YAxis
                  stroke="#9e9ea0"
                  tick={{ fill: "#9e9ea0", fontSize: 10 }}
                  axisLine={false}
                  tickLine={false}
                  width={35}
                  hide={true}
                />
                <Tooltip content={<CustomTooltip />} />

                {/* Stacked areas for stream graph (center-weighted) */}
                <Area
                  type="basis"
                  dataKey="mistral"
                  stackId="1"
                  stroke="#111111"
                  strokeWidth={2}
                  fill="#4338ca"
                  fillOpacity={1}
                />
                <Area
                  type="basis"
                  dataKey="instruct"
                  stackId="1"
                  stroke="#111111"
                  strokeWidth={2}
                  fill="#6366f1"
                  fillOpacity={1}
                />
                <Area
                  type="basis"
                  dataKey="qwen"
                  stackId="1"
                  stroke="#111111"
                  strokeWidth={2}
                  fill="#a5b4fc"
                  fillOpacity={1}
                />
                <Area
                  type="basis"
                  dataKey="gemma3n"
                  stackId="1"
                  stroke="#111111"
                  strokeWidth={2}
                  fill="#e0e7ff"
                  fillOpacity={1}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>

          {/* Legend */}
          <div className={styles.legend}>
            {models.reverse().map((model) => (
              <div key={model.name} className={styles.legendItem}>
                <div
                  className={styles.legendDot}
                  style={{ backgroundColor: model.color }}
                />
                <span className={styles.legendLabel}>{model.label}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
