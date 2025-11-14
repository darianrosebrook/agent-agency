"use client";

import { useState, useEffect } from "react";
import { useAnimatedValue } from "../hooks/useAnimatedValue";
import { getTasksStats, type TasksStats } from "../lib/api/tasks";
import styles from "./TaskCompletionGauge.module.scss";

interface TaskCompletionGaugeProps {
  title?: string;
  subtitle?: string;
}

export function TaskCompletionGauge({
  title = "Task Balance",
  subtitle = "Completion vs Creation Rate",
}: TaskCompletionGaugeProps) {
  const [taskStats, setTaskStats] = useState<TasksStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchTaskStats() {
      setIsLoading(true);
      setError(null);
      try {
        const stats = await getTasksStats();
        setTaskStats(stats);
      } catch (err) {
        console.error("Failed to fetch task stats:", err);
        setError(err instanceof Error ? err : new Error("Failed to load task statistics"));
      } finally {
        setIsLoading(false);
      }
    }

    fetchTaskStats();
    // Refresh every 30 seconds
    const interval = setInterval(fetchTaskStats, 30000);
    return () => clearInterval(interval);
  }, []);

  // Color constants matching gauge colors
  const COLORS = {
    created: "#27272a", // zinc-800 - dark gray for created tasks
    inProgress: "#6366f1", // indigo-500 - indigo for in-progress tasks
    completed: "#e0e7ff", // indigo-100 - light indigo for completed tasks
  };

  // Calculate completion rate from available stats (completed / total * 100)
  // TODO: Implement month-over-month comparison when historical data endpoint is available
  const completionRate = taskStats && taskStats.total > 0
    ? Math.round((taskStats.completed / taskStats.total) * 100)
    : 0;

  // Task distribution from API stats
  const created = taskStats?.pending ?? 0;
  const inProgress = taskStats?.in_progress ?? 0;
  const completed = taskStats?.completed ?? 0;

  // Animate task counts
  const animatedCreated = useAnimatedValue(created);
  const animatedInProgress = useAnimatedValue(inProgress);
  const animatedCompleted = useAnimatedValue(completed);
  const animatedCompletionRate = useAnimatedValue(completionRate);

  // Calculate animated totals and percentages
  const animatedTotal =
    animatedCreated + animatedInProgress + animatedCompleted;
  const createdPercent =
    animatedTotal > 0 ? (animatedCreated / animatedTotal) * 100 : 0;
  const inProgressPercent =
    animatedTotal > 0 ? (animatedInProgress / animatedTotal) * 100 : 0;

  // Total number of dashes in the semi-circle
  const totalDashes = 60;

  // Calculate number of dashes for each section using animated values
  const createdDashes = Math.round((createdPercent / 100) * totalDashes);
  const inProgressDashes = Math.round((inProgressPercent / 100) * totalDashes);

  // Format numbers to fixed decimal places to prevent hydration mismatches
  const formatNumber = (value: number, decimals: number = 2): string => {
    return value.toFixed(decimals);
  };

  // Semi-circle parameters
  const centerX = 120;
  const centerY = 120;
  const radius = 85;
  const dashLength = 12;
  const dashWidth = 3;
  const gapAngle = 2; // degrees between dashes

  // Create dashes
  const renderDashes = () => {
    const dashes = [];
    const startAngle = 180; // Start from left (180 degrees)
    const totalAngle = 180; // Semi-circle
    const anglePerDash = totalAngle / totalDashes;

    let currentDash = 0;

    for (let i = 0; i < totalDashes; i++) {
      const angle = startAngle - i * anglePerDash - gapAngle / 2;
      const angleRad = (angle * Math.PI) / 180;

      // Determine color based on section
      let color;
      if (currentDash < createdDashes) {
        color = COLORS.created;
      } else if (currentDash < createdDashes + inProgressDashes) {
        color = COLORS.inProgress;
      } else {
        color = COLORS.completed;
      }

      // Calculate start and end points of the dash and format to fixed decimal places
      const innerRadius = radius - dashLength;
      const x1 = formatNumber(centerX + innerRadius * Math.cos(angleRad));
      const y1 = formatNumber(centerY - innerRadius * Math.sin(angleRad));
      const x2 = formatNumber(centerX + radius * Math.cos(angleRad));
      const y2 = formatNumber(centerY - radius * Math.sin(angleRad));

      dashes.push(
        <line
          key={i}
          x1={x1}
          y1={y1}
          x2={x2}
          y2={y2}
          stroke={color}
          strokeWidth={dashWidth}
          strokeLinecap="round"
          className={styles.dash}
        />
      );

      currentDash++;
    }

    return dashes;
  };

  // Show loading or error state
  if (isLoading) {
    return (
      <div className={styles.container}>
        <div className={styles.innerContainer}>
          <div className={styles.content}>
            <div className={styles.header}>
              <h3 className={styles.title}>{title}</h3>
              <p className={styles.subtitle}>Loading task statistics...</p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (error || !taskStats) {
    return (
      <div className={styles.container}>
        <div className={styles.innerContainer}>
          <div className={styles.content}>
            <div className={styles.header}>
              <h3 className={styles.title}>{title}</h3>
              <p className={styles.subtitle}>
                {error ? `Error: ${error.message}` : "No data available"}
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

          {/* Gauge */}
          <div className={styles.gaugeContainer}>
            <div className={styles.gaugeWrapper}>
              <svg className={styles.gaugeSvg} viewBox="0 0 240 140">
                {renderDashes()}
              </svg>

              {/* Center percentage */}
              <div className={styles.centerPercentage}>
                <div className={styles.centerContent}>
                  <div className={styles.percentageValue}>
                    +{animatedCompletionRate}%
                  </div>
                  <div className={styles.percentageLabel}>vs last month</div>
                </div>
              </div>
            </div>

            {/* Legend */}
            <div className={styles.legend}>
              <div className={styles.legendItem}>
                <div
                  className={styles.legendDot}
                  style={{ backgroundColor: COLORS.created }}
                />
                <span className={styles.legendLabel}>Created</span>
              </div>
              <div className={styles.legendItem}>
                <div
                  className={styles.legendDot}
                  style={{ backgroundColor: COLORS.inProgress }}
                />
                <span className={styles.legendLabel}>In Progress</span>
              </div>
              <div className={styles.legendItem}>
                <div
                  className={styles.legendDot}
                  style={{ backgroundColor: COLORS.completed }}
                />
                <span className={styles.legendLabel}>Completed</span>
              </div>
            </div>

            {/* Stats */}
            <div className={styles.stats}>
              <div className={styles.statItem}>
                <div
                  className={styles.statValue}
                  style={{ color: COLORS.created }}
                >
                  {animatedCreated}
                </div>
                <div className={styles.statLabel}>created</div>
              </div>
              <div className={styles.statItem}>
                <div
                  className={styles.statValue}
                  style={{ color: COLORS.inProgress }}
                >
                  {animatedInProgress}
                </div>
                <div className={styles.statLabel}>in progress</div>
              </div>
              <div className={styles.statItem}>
                <div
                  className={styles.statValue}
                  style={{ color: COLORS.completed }}
                >
                  {animatedCompleted}
                </div>
                <div className={styles.statLabel}>completed</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
