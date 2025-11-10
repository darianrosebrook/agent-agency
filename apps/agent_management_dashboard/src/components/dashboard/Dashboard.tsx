"use client";

import { Suspense, lazy, useEffect, useState } from "react";
import { LayoutGrid } from "lucide-react";
import { cn } from "../primitives/utils";
import styles from "./Dashboard.module.scss";
import { getCurrentUser } from "../../lib/api/users";
import type { CurrentUser } from "../../lib/api/users";
import { getTasksStats } from "../../lib/api/tasks";
import type { TasksStats } from "../../lib/api/tasks";
import { getContributions, getModelContributions } from "../../lib/api/agents";
import { getEfficiencyMetrics } from "../../lib/api/observability";

const TaskProgressChart = lazy(() =>
  import("../TaskProgressChart").then((mod) => ({
    default: mod.TaskProgressChart,
  }))
);
const RadialTaskProgress = lazy(() =>
  import("../RadialTaskProgress").then((mod) => ({
    default: mod.RadialTaskProgress,
  }))
);
const MultiRingProgress = lazy(() =>
  import("../MultiRingProgress").then((mod) => ({
    default: mod.MultiRingProgress,
  }))
);
const CodeContributionChart = lazy(() =>
  import("../CodeContributionChart").then((mod) => ({
    default: mod.CodeContributionChart,
  }))
);
const HexagonHeatmap = lazy(() =>
  import("../HexagonHeatmap").then((mod) => ({ default: mod.HexagonHeatmap }))
);
const ModelContributionStream = lazy(() =>
  import("../ModelContributionStream").then((mod) => ({
    default: mod.ModelContributionStream,
  }))
);
const TaskCompletionGauge = lazy(() =>
  import("../TaskCompletionGauge").then((mod) => ({
    default: mod.TaskCompletionGauge,
  }))
);
const ServerEfficiencyChart = lazy(() =>
  import("../ServerEfficiencyChart").then((mod) => ({
    default: mod.ServerEfficiencyChart,
  }))
);

const ChartSkeleton = () => (
  <div className={styles.chartSkeleton}>
    <div className={styles.chartSkeletonText}>Loading chart...</div>
  </div>
);

export function Dashboard() {
  const [user, setUser] = useState<CurrentUser | null>(null);
  const [isLoadingUser, setIsLoadingUser] = useState(true);
  const [taskStats, setTaskStats] = useState<TasksStats | null>(null);
  const [isLoadingStats, setIsLoadingStats] = useState(true);

  useEffect(() => {
    async function fetchUser() {
      try {
        const userData = await getCurrentUser();
        setUser(userData);
      } catch (error) {
        console.error("Failed to fetch user:", error);
        // Continue with null user - component will show fallback
      } finally {
        setIsLoadingUser(false);
      }
    }

    async function fetchStats() {
      try {
        const stats = await getTasksStats();
        setTaskStats(stats);
      } catch (error) {
        console.error("Failed to fetch task stats:", error);
        // Continue with null stats - component will use defaults
      } finally {
        setIsLoadingStats(false);
      }
    }

    fetchUser();
    fetchStats();
  }, []);

  const userName = user?.name || "User";

  return (
    <div className={styles.dashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerTop}>
          <LayoutGrid className={styles.headerIcon} size={16} />
          <span className={styles.headerLabel}>Dashboard</span>
        </div>
        <h1 className={styles.headerTitle}>
          {isLoadingUser ? "Loading..." : `Welcome back ${userName}!`}
        </h1>
      </div>

      {/* Bento Grid */}
      <div className={styles.bentoGrid}>
        {/* Task Progress Chart - spans 2 rows and 5 columns */}
        <div className={cn(styles.colSpan5, styles.rowSpan2)}>
          <Suspense fallback={<ChartSkeleton />}>
            <TaskProgressChart
              completedTasks={taskStats?.completed ?? 19}
              totalTasks={taskStats?.total ?? 40}
            />
          </Suspense>
        </div>

        {/* Radial Task Progress - spans 2 rows and 7 columns */}
        <div className={cn(styles.colSpan7, styles.rowSpan2)}>
          <Suspense fallback={<ChartSkeleton />}>
            <RadialTaskProgress />
          </Suspense>
        </div>

        {/* Hexagon Heatmap - spans 6 rows and 8 columns */}
        <div className={cn(styles.colSpan8, styles.rowSpan6)}>
          <Suspense fallback={<ChartSkeleton />}>
            <HexagonHeatmap radius={8} hexSize={28} />
          </Suspense>
        </div>

        {/* Multi-Ring Progress - spans 6 rows and 4 columns */}
        <div className={cn(styles.colSpan4, styles.rowSpan6)}>
          {/* TODO: Replace hardcoded progress data with project milestone data from v3 database with the following requirements:
          // 1. Milestone data fetching: Load project milestones with progress calculations
          //    - Data source: GET /api/projects/:id/milestones endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
          //    - Database table: PostgreSQL `milestones` table
          //    - Calculate progress percentage based on completed tasks per milestone
          // 2. Timeline estimation: Calculate projected timeline from milestone data
          //    - Use milestone completion dates and task estimates
          //    - Handle missing or incomplete milestone data
          // 3. Data transformation: Format API response for MultiRingProgress component
          //    - Map API response to tasks array with name, progress, and color
          //    - Calculate projectedTimeline string from milestone dates */}
          <Suspense fallback={<ChartSkeleton />}>
            <MultiRingProgress
              tasks={[
                { name: "Progress 1", progress: 85, color: "#e0e7ff" },
                { name: "Progress 2", progress: 75, color: "#818cf8" },
                { name: "Progress 3", progress: 60, color: "#6366f1" },
              ]}
              projectedTimeline="15 business days"
            />
          </Suspense>
        </div>

        {/* Code Contribution Chart - spans 3 rows and 12 columns */}
        <div className={cn(styles.colSpan12, styles.rowSpan3)}>
          <Suspense fallback={<ChartSkeleton />}>
            <CodeContributionChart
              title="Overall Contribution"
              subtitle="2 Agents over the last 30 days"
              days={30}
            />
          </Suspense>
        </div>

        {/* Small panels row */}
        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
          <Suspense fallback={<ChartSkeleton />}>
            <ModelContributionStream
              title="Model Contributions"
              subtitle="Lines of code by AI model"
            />
          </Suspense>
        </div>

        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
          <Suspense fallback={<ChartSkeleton />}>
            <TaskCompletionGauge
              title="Task Balance"
              subtitle="Completion vs Creation Rate"
            />
          </Suspense>
        </div>

        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
          <Suspense fallback={<ChartSkeleton />}>
            <ServerEfficiencyChart
              title="Server Efficiency Analysis"
            />
          </Suspense>
        </div>
      </div>
    </div>
  );
}
