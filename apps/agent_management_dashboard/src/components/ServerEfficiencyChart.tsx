"use client";

import { useMemo } from "react";
import { BentoPanel } from "./compounds";
import { ArrowUpRight } from "lucide-react";
import { useAnimatedValue } from "../hooks/useAnimatedValue";

interface ServerEfficiencyChartProps {
  title?: string;
  efficiency?: number;
}

export function ServerEfficiencyChart({
  title = "Server Efficiency Analysis",
  efficiency = 55,
}: ServerEfficiencyChartProps) {
  // Animate efficiency percentage
  const animatedEfficiency = useAnimatedValue(efficiency);
  
  // TODO: Replace hardcoded bar data with observability metrics from v3 API with the following requirements:
  // 1. Efficiency metrics fetching: Load server efficiency time-series data
  //    - Data source: GET /api/observability/efficiency endpoint from `iterations/v3/system-observability` crate
  //    - Return server efficiency metrics over time
  //    - Include efficiency percentage and time-series bar data
  // 2. Real-time updates: Refresh efficiency metrics periodically
  //    - Poll API endpoint at configurable intervals
  //    - Handle loading and error states
  // 3. Data transformation: Format API response for chart component
  //    - Map API response to bars array with height values
  //    - Calculate overall efficiency percentage for display
  // Generate bar data with varying heights
  const bars = [
    { height: 30 },
    { height: 45 },
    { height: 65 },
    { height: 50 },
    { height: 85 },
    { height: 40 },
    { height: 70 },
    { height: 90 },
  ];

  // Animate bar heights - must call hooks at top level
  const animatedBar1 = useAnimatedValue(bars[0]?.height ?? 30);
  const animatedBar2 = useAnimatedValue(bars[1]?.height ?? 45);
  const animatedBar3 = useAnimatedValue(bars[2]?.height ?? 65);
  const animatedBar4 = useAnimatedValue(bars[3]?.height ?? 50);
  const animatedBar5 = useAnimatedValue(bars[4]?.height ?? 85);
  const animatedBar6 = useAnimatedValue(bars[5]?.height ?? 40);
  const animatedBar7 = useAnimatedValue(bars[6]?.height ?? 70);
  const animatedBar8 = useAnimatedValue(bars[7]?.height ?? 90);

  const animatedBars = useMemo(
    () => [
      animatedBar1,
      animatedBar2,
      animatedBar3,
      animatedBar4,
      animatedBar5,
      animatedBar6,
      animatedBar7,
      animatedBar8,
    ],
    [animatedBar1, animatedBar2, animatedBar3, animatedBar4, animatedBar5, animatedBar6, animatedBar7, animatedBar8]
  );

  return (
    <BentoPanel>
      <div className="h-full flex flex-col p-6">
        {/* Header */}
        <div className="flex items-start justify-between mb-8">
          <h3 className="text-zinc-400 text-sm">{title}</h3>
          <button className="text-zinc-500 hover:text-zinc-300 transition-colors">
            <ArrowUpRight className="w-4 h-4" />
          </button>
        </div>

        {/* Efficiency Metric */}
        <div className="mb-6">
          <div className="text-white text-4xl transition-none">+{animatedEfficiency}%</div>
        </div>

        {/* Bar Chart */}
        <div className="flex-1 flex items-end gap-[6px] justify-between">
          {animatedBars.map((animatedHeight, index) => (
            <div
              key={index}
              className="flex-1 bg-zinc-700 rounded-t-sm relative transition-all duration-500 ease-out"
              style={{ height: `${animatedHeight}%` }}
            >
              <div className="absolute top-0 left-0 right-0 h-[3px] bg-zinc-300 rounded-t-sm" />
            </div>
          ))}
        </div>
      </div>
    </BentoPanel>
  );
}
