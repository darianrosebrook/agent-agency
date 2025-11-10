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
import { useState } from "react";
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
  // TODO: Replace mock data generation with API call to v3 telemetry service with the following requirements:
  // 1. Model contribution data fetching: Load model usage statistics by month
  //    - Data source: GET /api/telemetry/model-contributions endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `telemetry` table
  //    - Aggregate lines of code contributed by each AI model (gemma3n, qwen, instruct, mistral)
  // 2. Time aggregation: Group contributions by month for stream chart
  //    - Handle month boundaries and date range calculations
  //    - Support configurable time ranges
  // 3. Data transformation: Format API response for stream chart component
  //    - Map API response to StreamDataPoint array with month and model-specific values
  //    - Handle missing model data gracefully
  // Generate mock data for stream graph
  const generateData = (): StreamDataPoint[] => {
    const months = ["Jan", "Feb", "Mar", "Apr"];
    const data: StreamDataPoint[] = [];

    months.forEach((month, index) => {
      // Create high variance contributions for each model with dramatic transitions
      const t = index / (months.length - 1);

      data.push({
        month,
        gemma3n: Math.round(
          400 + Math.sin(t * Math.PI * 3) * 350 + Math.random() * 200
        ),
        qwen: Math.round(
          500 + Math.cos(t * Math.PI * 2.5) * 400 + Math.random() * 250
        ),
        instruct: Math.round(
          350 + Math.sin(t * Math.PI * 4) * 320 + Math.random() * 180
        ),
        mistral: Math.round(
          450 + Math.cos(t * Math.PI * 1.8) * 380 + Math.random() * 220
        ),
      });
    });

    return data;
  };

  const [data] = useState(generateData());

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
