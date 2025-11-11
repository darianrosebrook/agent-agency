"use client";

import { useState, useMemo, useEffect, useRef } from "react";
import { gsap } from "gsap";
import {
  CheckCircle2,
  MoreVertical,
} from "lucide-react";
import { useGSAPNumberAnimation } from "../hooks/useGSAPAnimation";
import { getTasksStats, type TasksStats } from "../lib/api/tasks";
import styles from "./RadialTaskProgress.module.scss";

interface RadialTaskProgressProps {
  totalSegments?: number;
}

export function RadialTaskProgress({
  totalSegments = 24,
}: RadialTaskProgressProps) {
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

  // Calculate overall task completion percentage
  const overallProgress = taskStats && taskStats.total > 0
    ? Math.round((taskStats.completed / taskStats.total) * 100)
    : 0;

  // Determine status based on completion rate
  const getStatus = (progress: number): string => {
    if (progress >= 90) return "Nearly Done";
    if (progress >= 70) return "On Track";
    if (progress >= 50) return "In Progress";
    if (progress >= 30) return "Normal";
    return "Early Stage";
  };

  const status = getStatus(overallProgress);
  const svgRef = useRef<SVGSVGElement>(null);
  const segmentsRef = useRef<SVGPathElement[]>([]);

  // Use GSAP for smooth number animation
  const animatedProgress = useGSAPNumberAnimation(
    overallProgress,
    0.8,
    "power2.out"
  );

  const completedSegments = Math.round(
    (animatedProgress / 100) * totalSegments
  );

  // Generate radial segments
  // Format numbers to fixed decimal places to prevent hydration mismatches
  const formatNumber = (value: number, decimals: number = 2): string => {
    return value.toFixed(decimals);
  };

  // Memoize segments to ensure consistent generation between server and client
  const segments = useMemo(() => {
    const segmentList = [];
    const segmentAngle = 360 / totalSegments;
    const gapAngle = 2; // Gap between segments
    const radius = 100;
    const innerRadius = 70;
    const centerX = 120;
    const centerY = 120;

    for (let i = 0; i < totalSegments; i++) {
      const startAngle = i * segmentAngle - 90; // Start from top
      const endAngle = startAngle + segmentAngle - gapAngle;

      // Calculate coordinates and format to fixed decimal places for consistent rendering
      const x1 = formatNumber(
        centerX + radius * Math.cos((startAngle * Math.PI) / 180)
      );
      const y1 = formatNumber(
        centerY + radius * Math.sin((startAngle * Math.PI) / 180)
      );
      const x2 = formatNumber(
        centerX + radius * Math.cos((endAngle * Math.PI) / 180)
      );
      const y2 = formatNumber(
        centerY + radius * Math.sin((endAngle * Math.PI) / 180)
      );
      const x3 = formatNumber(
        centerX + innerRadius * Math.cos((endAngle * Math.PI) / 180)
      );
      const y3 = formatNumber(
        centerY + innerRadius * Math.sin((endAngle * Math.PI) / 180)
      );
      const x4 = formatNumber(
        centerX + innerRadius * Math.cos((startAngle * Math.PI) / 180)
      );
      const y4 = formatNumber(
        centerY + innerRadius * Math.sin((startAngle * Math.PI) / 180)
      );

      // Build path data string with formatted numbers
      const pathData = `M ${x1} ${y1} A ${radius} ${radius} 0 0 1 ${x2} ${y2} L ${x3} ${y3} A ${innerRadius} ${innerRadius} 0 0 0 ${x4} ${y4} Z`;

      segmentList.push(
        <path
          key={i}
          ref={(el) => {
            if (el) segmentsRef.current[i] = el;
          }}
          d={pathData}
          fill={i < completedSegments ? "#fafafa" : "#454545"}
        />
      );
    }

    return segmentList;
  }, [totalSegments, completedSegments]);

  // Animate segments with GSAP when progress changes
  useEffect(() => {
    if (segmentsRef.current.length === 0) return;

    const completedCount = completedSegments;

    // Animate segments with stagger effect
    segmentsRef.current.forEach((segment, index) => {
      const isCompleted = index < completedCount;
      const targetColor = isCompleted ? "#fafafa" : "#454545";

      gsap.to(segment, {
        fill: targetColor,
        duration: 0.3,
        delay: index * 0.02, // Stagger delay
        ease: "power2.out",
      });
    });
  }, [completedSegments]);

  // Initial animation on mount
  useEffect(() => {
    if (segmentsRef.current.length === 0 || !svgRef.current) return;

    // Animate SVG entrance
    gsap.fromTo(
      svgRef.current,
      { opacity: 0, scale: 0.9 },
      {
        opacity: 1,
        scale: 1,
        duration: 0.6,
        ease: "back.out(1.7)",
      }
    );

    // Animate segments entrance with stagger
    gsap.fromTo(
      segmentsRef.current,
      { opacity: 0, scale: 0.8 },
      {
        opacity: 1,
        scale: 1,
        duration: 0.4,
        stagger: 0.03,
        delay: 0.2,
        ease: "back.out(1.7)",
      }
    );
  }, []);

  return (
    <div className={styles.container}>
      <div className={styles.innerContainer}>
        <div className={styles.content}>
          {/* Main content area */}
          <div className={styles.mainContent}>
            {/* Left side - Radial chart */}
            <div className={styles.chartContainer}>
              <svg
                ref={svgRef}
                width="240"
                height="240"
                viewBox="0 0 240 240"
                xmlns="http://www.w3.org/2000/svg"
                className={styles.svg}
              >
                {segments}
                {/* Center circle with percentage */}
                <circle
                  cx="120"
                  cy="120"
                  r="65"
                  className={styles.centerCircle}
                />
                <text
                  x="120"
                  y="130"
                  textAnchor="middle"
                  className={styles.centerText}
                  style={{
                    fontSize: "56px",
                    fontWeight: "300",
                    letterSpacing: "-2.8px",
                  }}
                >
                  {animatedProgress}
                  <tspan
                    style={{
                      fontSize: "28px",
                      fontWeight: "300",
                    }}
                  >
                    %
                  </tspan>
                </text>
              </svg>
            </div>

            {/* Divider */}
            <div className={styles.divider} />

            {/* Right side - Task details */}
            <div className={styles.detailsContainer}>
              {/* Header */}
              <div className={styles.detailsHeader}>
                <div className={styles.detailsTitleContainer}>
                  <h3 className={styles.detailsTitle}>
                    Overall Task Progress
                  </h3>
                  <p className={styles.detailsDescription}>
                    {isLoading 
                      ? "Loading task statistics..." 
                      : error 
                      ? `Error: ${error.message}`
                      : taskStats
                      ? `Total: ${taskStats.total} tasks`
                      : "No data available"}
                  </p>
                </div>
                <div className={styles.detailsActions}>
                  <button
                    className={`${styles.actionButton} ${styles.actionButtonShrink}`}
                    aria-label="More options"
                  >
                    <MoreVertical className={styles.actionButtonIcon} />
                  </button>
                </div>
              </div>

              {/* Divider */}
              <div className={styles.detailsDivider} />

              {/* Info grid */}
              {taskStats && !isLoading && !error ? (
              <div className={styles.infoGrid}>
                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>Status:</span>
                  <div className={styles.infoValueGroup}>
                    <span className={styles.infoValue}>
                        {status}
                    </span>
                    <CheckCircle2 className={styles.infoIcon} />
                  </div>
                </div>

                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>Progress:</span>
                  <span className={styles.infoValue}>
                      {animatedProgress}%
                  </span>
                </div>

                <div className={styles.infoItem}>
                    <span className={styles.infoLabel}>Completed:</span>
                  <span className={styles.infoValue}>
                      {taskStats.completed}
                  </span>
                </div>

                <div className={styles.infoItem}>
                    <span className={styles.infoLabel}>In Progress:</span>
                  <span className={styles.infoValue}>
                      {taskStats.in_progress}
                  </span>
                </div>
              </div>
              ) : (
                <div className={styles.infoGrid}>
                  <div className={styles.infoItem}>
                    <span className={styles.infoLabel}>
                      {isLoading ? "Loading..." : "No data"}
                    </span>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
