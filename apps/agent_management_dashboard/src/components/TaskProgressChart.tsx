"use client";

import { useAnimatedValue } from "../hooks/useAnimatedValue";
import styles from "./TaskProgressChart.module.scss";

interface TaskProgressChartProps {
  completedTasks?: number;
  totalTasks?: number;
  categories?: string[];
}

export function TaskProgressChart({
  // TODO: Replace default props with data fetched from v3 API with the following requirements:
  // 1. Task statistics fetching: Load task completion statistics from API
  //    - Data source: GET /api/projects/:id/tasks/stats endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `tasks` table with aggregation queries
  //    - Include completedTasks count, totalTasks count, and category breakdowns
  // 2. Props handling: Accept projectId prop to fetch project-specific statistics
  //    - Fetch statistics when projectId changes
  //    - Handle loading and error states
  // 3. Data transformation: Format API response for component props
  //    - Map API response to completedTasks, totalTasks, and categories props
  //    - Handle edge cases (zero tasks, all completed, etc.)
  completedTasks = 19,
  totalTasks = 40,
  categories = ["dev", "design"],
}: TaskProgressChartProps) {
  // Animate values when props change
  const animatedCompletedTasks = useAnimatedValue(completedTasks);
  const animatedTotalTasks = useAnimatedValue(totalTasks);
  const animatedPercentage = useAnimatedValue(
    Math.round((completedTasks / totalTasks) * 100)
  );

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
