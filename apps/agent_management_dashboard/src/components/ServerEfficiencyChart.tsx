"use client";

import { useMemo, useState, useEffect } from "react";
import { BentoPanel } from "./compounds";
import { ArrowUpRight } from "lucide-react";
import { useAnimatedValue } from "../hooks/useAnimatedValue";
import { getEfficiencyMetrics, type EfficiencyMetrics } from "../lib/api/observability";
import styles from "./ServerEfficiencyChart.module.scss";

interface ServerEfficiencyChartProps {
  title?: string;
  efficiency?: number;
}

interface AnimatedBarProps {
  height: number;
}

function AnimatedBar({ height }: AnimatedBarProps) {
  const animatedHeight = useAnimatedValue(height);
  return (
    <div
      className={styles.bar}
      style={{ height: `${animatedHeight}%` }}
    >
      <div className={styles.barTop} />
    </div>
  );
}

export function ServerEfficiencyChart({
  title = "Server Efficiency Analysis",
  efficiency: initialEfficiency = 55,
}: ServerEfficiencyChartProps) {
  const [efficiencyMetrics, setEfficiencyMetrics] = useState<EfficiencyMetrics[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        const metrics = await getEfficiencyMetrics();
        setEfficiencyMetrics(metrics);
      } catch (err) {
        console.error("Failed to fetch efficiency metrics:", err);
        setError(err instanceof Error ? err : new Error("Failed to load efficiency metrics"));
        setEfficiencyMetrics([]);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
    // Refresh every 30 seconds
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, []);

  // Calculate efficiency from metrics or use initial
  const efficiency = useMemo(() => {
    if (efficiencyMetrics.length === 0) return initialEfficiency;
    const avgEfficiency = efficiencyMetrics.reduce(
      (sum, m) => sum + m.efficiency_score,
      0
    ) / efficiencyMetrics.length;
    return Math.round(avgEfficiency);
  }, [efficiencyMetrics, initialEfficiency]);

  // Generate bars from efficiency metrics (take last 8 data points)
  const bars = useMemo(() => {
    if (efficiencyMetrics.length === 0) {
      return [
        { height: 30 },
        { height: 45 },
        { height: 65 },
        { height: 50 },
        { height: 85 },
        { height: 40 },
        { height: 70 },
        { height: 90 },
      ];
    }

    // Take last 8 metrics and map efficiency_score to bar height
    const recentMetrics = efficiencyMetrics.slice(-8);
    return recentMetrics.map((metric) => ({
      height: Math.round(metric.efficiency_score),
    }));
  }, [efficiencyMetrics]);

  // Animate efficiency percentage
  const animatedEfficiency = useAnimatedValue(efficiency);

  // Animate bar heights - use bars directly since useAnimatedValue handles updates
  const animatedBars = useMemo(() => {
    return bars.map((bar) => bar.height);
  }, [bars]);

  return (
    <BentoPanel>
      <div className={styles.content}>
        {/* Header */}
        <div className={styles.header}>
          <h3 className={styles.title}>{title}</h3>
          <button className={styles.moreButton}>
            <ArrowUpRight className={styles.moreButtonIcon} />
          </button>
        </div>

        {/* Efficiency Metric */}
        <div className={styles.efficiencyMetric}>
          <div className={styles.efficiencyValue}>+{animatedEfficiency}%</div>
        </div>

        {/* Bar Chart */}
        <div className={styles.barChart}>
          {animatedBars.map((height, index) => (
            <AnimatedBar key={index} height={height} />
          ))}
        </div>
      </div>
    </BentoPanel>
  );
}
