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
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload?.length) {
      const total = payload.reduce(
        (sum: number, item: any) => sum + item.value,
        0
      );

      return (
        <div className="bg-[#111111] border border-[#39393b] rounded-[8px] px-4 py-3">
          <div className="text-white text-[12px] mb-2">{label}</div>
          {payload.reverse().map((item: any) => {
            const model = models.find((m) => m.name === item.dataKey);
            return (
              <div key={item.dataKey} className="flex items-center gap-2 mb-1">
                <div
                  className="w-2 h-2 rounded-full"
                  style={{ backgroundColor: item.color }}
                />
                <span className="text-[#9e9ea0] text-[11px]">
                  {model?.label}: {item.value}
                </span>
              </div>
            );
          })}
          <div className="text-white text-[11px] mt-2 pt-2 border-t border-[#39393b]">
            Total: {total}
          </div>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="bg-[#111111] relative rounded-[12px] size-full border border-[#cacaca]">
      <div className="size-full">
        <div className="box-border flex flex-col p-6 relative size-full">
          {/* Header */}
          <div className="mb-4">
            <h3 className="text-white text-[15px] mb-1">{title}</h3>
            <p className="text-[#9e9ea0] text-[10px]">{subtitle}</p>
          </div>

          {/* Chart */}
          <div className="flex-1 relative min-h-0">
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
          <div className="flex flex-wrap gap-3 mt-3">
            {models.reverse().map((model) => (
              <div key={model.name} className="flex items-center gap-1.5">
                <div
                  className="w-2 h-2 rounded-full"
                  style={{ backgroundColor: model.color }}
                />
                <span className="text-[#9e9ea0] text-[9px]">{model.label}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
