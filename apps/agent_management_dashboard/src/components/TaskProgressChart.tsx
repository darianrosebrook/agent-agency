"use client";

import { useState, useEffect } from "react";
import { useAnimatedValue } from "../hooks/useAnimatedValue";
import { getTasksStats, type TasksStats } from "../lib/api/tasks";
import styles from "./TaskProgressChart.module.scss";

interface TaskProgressChartProps {
  completedTasks?: number;
  totalTasks?: number;
  categories?: string[];
  projectId?: string;
}

export function TaskProgressChart({
  completedTasks: providedCompletedTasks,
  totalTasks: providedTotalTasks,
  categories: providedCategories,
  projectId,
}: TaskProgressChartProps) {
  const [taskStats, setTaskStats] = useState<TasksStats | null>(null);
  const [isLoading, setIsLoading] = useState(!providedCompletedTasks && !providedTotalTasks);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    // If props are provided, use them and skip API call
    if (providedCompletedTasks !== undefined && providedTotalTasks !== undefined) {
      setIsLoading(false);
      return;
    }

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
  }, [providedCompletedTasks, providedTotalTasks]);

  // Use provided props or API data
  const completedTasks = providedCompletedTasks ?? taskStats?.completed ?? 0;
  const totalTasks = providedTotalTasks ?? taskStats?.total ?? 0;
  
  // Extract categories from task types if available, otherwise use defaults
  const categories = providedCategories ?? (taskStats ? ["all"] : ["dev", "design"]);

  // Animate values when props change
  const animatedCompletedTasks = useAnimatedValue(completedTasks);
  const animatedTotalTasks = useAnimatedValue(totalTasks);
  const animatedPercentage = useAnimatedValue(
    totalTasks > 0 ? Math.round((completedTasks / totalTasks) * 100) : 0
  );

  if (isLoading) {
    return (
      <div className={styles.container}>
        <div className={styles.innerContainer}>
          <div className={styles.content}>
            <div className={styles.header}>
              <div className={styles.headerTop}>
                <div className={styles.categories}>
                  <div className={styles.categoryBadge}>
                    <div className={styles.categoryBadgeInner}>
                      <div className={styles.categoryText}>
                        <p className={styles.categoryTextParagraph}>Loading...</p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.container}>
        <div className={styles.innerContainer}>
          <div className={styles.content}>
            <div className={styles.header}>
              <div className={styles.headerTop}>
                <div className={styles.categories}>
                  <div className={styles.categoryBadge}>
                    <div className={styles.categoryBadgeInner}>
                      <div className={styles.categoryText}>
                        <p className={styles.categoryTextParagraph}>Error: {error.message}</p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
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
            <div className={styles.headerTop}>
              {/* Category badges */}
              <div className={styles.categories}>
                {categories.map((category) => (
                  <div key={category} className={styles.categoryBadge}>
                    <div className={styles.categoryBadgeInner}>
                      <div className={styles.categoryText}>
                        <p className={styles.categoryTextParagraph}>
                          {category}
                        </p>
                      </div>
                    </div>
                    <div aria-hidden="true" className={styles.categoryBorder} />
                  </div>
                ))}
              </div>
            </div>
            {/* Title */}
            <div className={styles.title}>
              <p className={styles.titleParagraph}>
                All active projects completion rate
              </p>
            </div>
          </div>

          {/* Divider */}
          <div className={styles.divider} />

          {/* Stats */}
          <div className={styles.stats}>
            <div className={styles.statsText}>
              <p className={styles.statsTextParagraph}>
                You have {animatedCompletedTasks} tasks out of{" "}
                {animatedTotalTasks} completed
              </p>
            </div>
            <div className={styles.statsRow}>
              {/* Percentage */}
              <div className={styles.percentage}>
                <p className={styles.percentageParagraph}>
                  {animatedPercentage}%
                </p>
              </div>
              {/* Task count badge */}
              <div className={styles.taskBadge}>
                <div className={styles.taskBadgeInner}>
                  <div className={styles.taskBadgeText}>
                    <p className={styles.taskBadgeTextParagraph}>
                      {animatedCompletedTasks} tasks
                    </p>
                  </div>
                </div>
                <div aria-hidden="true" className={styles.taskBadgeBorder} />
              </div>
              {/* Time reference */}
              <div className={styles.timeReference}>
                <p className={styles.timeReferenceParagraph}>since last week</p>
              </div>
            </div>
          </div>

          {/* Bar chart */}
          <div className={styles.barChart}>
            {Array.from({ length: animatedTotalTasks }).map((_, index) => (
              <div
                key={index}
                className={`${styles.bar} ${
                  index < animatedCompletedTasks
                    ? styles.barCompleted
                    : styles.barIncomplete
                }`}
              />
            ))}
          </div>
        </div>
      </div>
      <div aria-hidden="true" className={styles.borderOverlay} />
    </div>
  );
}
