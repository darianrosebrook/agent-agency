"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { useState } from "react";

interface DataPoint {
  day: string;
  baseline: number;
  contribution: number;
}

interface CodeContributionChartProps {
  title?: string;
  subtitle?: string;
  days?: number;
}

export function CodeContributionChart({
  title = "Overall Contribution",
  subtitle = "2 Agents over the last 30 days",
  days = 30,
}: CodeContributionChartProps) {
  // Generate mock data for the last N days
  const generateData = (): DataPoint[] => {
    const data: DataPoint[] = [];
    const today = new Date();

    for (let i = days - 1; i >= 0; i--) {
      const date = new Date(today);
      date.setDate(date.getDate() - i);
      const dayNum = days - i;

      // Generate accepted lines of code (solid line) with wave pattern
      const acceptedValue =
        1200 + Math.sin(dayNum / 4) * 600 + Math.cos(dayNum / 6) * 300;

      // Generate additional non-accepted lines (varies from 10% to 40% of accepted)
      const nonAcceptedValue =
        acceptedValue * (0.3 + Math.sin(dayNum / 7) * 0.05);

      // Total is accepted + not accepted
      const totalValue = acceptedValue + nonAcceptedValue;

      data.push({
        day: date.toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
        }),
        baseline: Math.round(totalValue), // Total overall (accepted + not accepted)
        contribution: Math.round(acceptedValue), // Accepted lines only
      });
    }

    return data;
  };

  const [data] = useState(generateData());

  // Calculate total contribution
  const totalContribution = data.reduce(
    (sum, point) => sum + point.contribution,
    0
  );
  const formattedTotal =
    totalContribution >= 1000
      ? `${(totalContribution / 1000).toFixed(1)}K`
      : totalContribution.toString();

  // Custom tooltip
  const CustomTooltip = ({
    active,
    payload,
  }: {
    active?: boolean;
    payload?: Array<{ dataKey?: string; value?: number }>;
  }) => {
    if (active && payload?.length) {
      // Find the baseline (total) value from payload
      const totalValue =
        payload.find((p) => p.dataKey === "baseline")?.value ?? 0;
      const acceptedValue =
        payload.find((p) => p.dataKey === "contribution")?.value ?? 0;

      return (
        <div className="bg-[#111111] border border-[#39393b] rounded-[8px] px-6 py-4">
          <div className="text-white text-[13px] mb-1">Total Performance</div>
          <div className="text-white text-[13px]">
            {(totalValue / 1000).toFixed(1)}K lines of code
          </div>
          <div className="text-[#9e9ea0] text-[11px] mt-1">
            {(acceptedValue / 1000).toFixed(1)}K accepted
          </div>
        </div>
      );
    }
    return null;
  };

  // Custom dot for the active point
  const CustomDot = (props: {
    cx?: number;
    cy?: number;
    stroke?: string;
    dataKey?: string;
  }) => {
    const { cx, cy, stroke, dataKey } = props;

    if (dataKey === "contribution") {
      return (
        <circle
          cx={cx}
          cy={cy}
          r={0}
          fill={stroke}
          className="transition-all duration-200 hover:r-4"
        />
      );
    }
    return null;
  };

  return (
    <div className="bg-[#111111] relative rounded-[12px] size-full border border-[#cacaca]">
      <div className="size-full">
        <div className="box-border flex flex-col p-8 relative size-full">
          {/* Header */}
          <div className="mb-6">
            <h3 className="text-white text-[17px] mb-1">{title}</h3>
            <p className="text-[#9e9ea0] text-[11px]">{subtitle}</p>
          </div>

          {/* Chart */}
          <div className="flex-1 relative min-h-0">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart
                data={data}
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
                  dataKey="day"
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
                  tickFormatter={(value) => `${(value / 1000).toFixed(0)}K`}
                  domain={[0, 3000]}
                  ticks={[0, 1000, 2000, 3000]}
                />
                <Tooltip
                  content={<CustomTooltip />}
                  cursor={{
                    stroke: "white",
                    strokeWidth: 1,
                    strokeDasharray: "0",
                  }}
                />

                {/* Total overall line (accepted + not accepted) - dashed */}
                <Line
                  type="monotone"
                  dataKey="baseline"
                  stroke="#333"
                  strokeWidth={12}
                  dot={false}
                  activeDot={false}
                  strokeDasharray="6 4"
                />

                {/* Accepted lines of code - solid */}
                <Line
                  type="monotone"
                  dataKey="contribution"
                  stroke="#6366f1"
                  strokeWidth={12}
                  dot={<CustomDot />}
                  activeDot={{ r: 5, fill: "#6366f1" }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {/* Stats overlay */}
          <div className="absolute top-8 right-8 bg-[#111111] border border-[#39393b] rounded-[8px] px-6 py-4">
            <div className="text-white text-[13px] mb-1">Total Performance</div>
            <div className="text-white text-[13px]">
              {formattedTotal} lines of code
            </div>
          </div>

          {/* Legend */}
          <div className="absolute bottom-8 right-8 bg-[#111111] border border-[#39393b] rounded-[8px] px-6 py-4">
            <div className="flex flex-col gap-3">
              <div className="flex items-center gap-3">
                <div className="w-8 h-[4px] bg-[#6366f1] rounded-full" />
                <span className="text-[#9e9ea0] text-[11px]">
                  Accepted lines of code
                </span>
              </div>
              <div className="flex items-center gap-3">
                <svg width="32" height="4" className="flex-shrink-0">
                  <line
                    x1="0"
                    y1="2"
                    x2="32"
                    y2="2"
                    stroke="#333"
                    strokeWidth="4"
                    strokeDasharray="6 4"
                  />
                </svg>
                <span className="text-[#888] text-[11px]">
                  Total overall (accepted + not accepted)
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
