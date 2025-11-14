"use client";

import { useState, useEffect } from "react";
import { TaskProgressChart } from "../TaskProgressChart";
import { RadialTaskProgress } from "../RadialTaskProgress";
import { MultiRingProgress } from "../MultiRingProgress";
import { CodeContributionChart } from "../CodeContributionChart";
import { HexagonHeatmap } from "../HexagonHeatmap";
import { ModelContributionStream } from "../ModelContributionStream";
import { TaskCompletionGauge } from "../TaskCompletionGauge";
import { ServerEfficiencyChart } from "../ServerEfficiencyChart";
import { BentoPanel } from "../compounds/BentoPanel";
import { LayoutGrid } from "lucide-react";
import { cn } from "../primitives/utils";
import styles from "./Dashboard.module.scss";
import { getCurrentUser } from "../../lib/api/users";
import type { CurrentUser } from "../../lib/api/users";

export function Dashboard() {
  const [user, setUser] = useState<CurrentUser | null>(null);
  const [isLoadingUser, setIsLoadingUser] = useState(true);

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

    fetchUser();
  }, []);

  const userName = user?.name || "User";

  return (
    <div className={styles.dashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerTop}>
          <LayoutGrid className={styles.headerIcon} />
          <span className={styles.headerLabel}>Dashboard</span>
        </div>
        <h1 className={styles.headerTitle}>
          {isLoadingUser ? "Loading..." : `Welcome back ${userName}!`}
        </h1>
      </div>

      {/* Bento Grid */}
      <div className={styles.bentoGrid}>
        {/* Task Progress Chart - spans 2 rows and 5 columns */}
        {/* TODO: Replace hardcoded task counts with aggregated data from v3 database with the following requirements:
        // 1. Task statistics fetching: Load aggregated task completion statistics
        //    - Data source: GET /api/projects/:id/tasks/stats endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
        //    - Database table: PostgreSQL `tasks` table with aggregation queries
        //    - Include completedTasks count, totalTasks count, and category breakdowns
        // 2. Real-time updates: Refresh statistics when tasks are created or updated
        //    - Subscribe to task update events or poll periodically
        //    - Handle loading and error states
        // 3. Data transformation: Format API response for chart component
        //    - Map API response to TaskProgressChart props (completedTasks, totalTasks, categories)
        //    - Handle edge cases (zero tasks, all completed, etc.) */}
        <div className={cn(styles.colSpan5, styles.rowSpan2)}>
          <TaskProgressChart completedTasks={19} totalTasks={40} />
        </div>

        {/* Radial Task Progress - spans 2 rows and 7 columns */}
        <div className={cn(styles.colSpan7, styles.rowSpan2)}>
          <BentoPanel>
            <RadialTaskProgress />
          </BentoPanel>
        </div>

        {/* Hexagon Heatmap - spans 6 rows and 8 columns */}
        <div className={cn(styles.colSpan8, styles.rowSpan6)}>
          <BentoPanel>
            <HexagonHeatmap radius={8} hexSize={28} />
          </BentoPanel>
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
          //    - Map milestones to tasks array with name, progress, and color
          //    - Calculate projectedTimeline string from milestone dates */}
          <BentoPanel>
            <MultiRingProgress />
          </BentoPanel>
        </div>

        {/* Code Contribution Chart - spans 3 rows and 12 columns */}
        <div className={cn(styles.colSpan12, styles.rowSpan3)}>
          {/* TODO: Replace hardcoded contribution data with provenance/telemetry data from v3 database with the following requirements:
          // 1. Contribution data fetching: Load code contribution statistics over time
          //    - Data source: GET /api/telemetry/contributions?days={days} endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
          //    - Database tables: PostgreSQL `provenance` and `telemetry` tables
          //    - Aggregate accepted lines of code vs total lines by day
          // 2. Time range filtering: Support configurable time ranges (last 7, 30, 90 days)
          //    - Pass days parameter to API endpoint
          //    - Handle date range calculations and timezone issues
          // 3. Data transformation: Format API response for CodeContributionChart component
          //    - Map API response to DataPoint array with day, baseline (total), and contribution (accepted)
          //    - Calculate total contribution for display */}
          <BentoPanel>
            <CodeContributionChart
              title="Overall Contribution"
              subtitle="2 Agents over the last 30 days"
              days={30}
            />
          </BentoPanel>
        </div>

        {/* Small panels row */}
        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
          {/* TODO: Replace hardcoded model contribution data with telemetry data from v3 database with the following requirements:
          // 1. Model contribution data fetching: Load model usage statistics by month
          //    - Data source: GET /api/telemetry/model-contributions endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
          //    - Database table: PostgreSQL `telemetry` table
          //    - Aggregate lines of code contributed by each AI model (gemma3n, qwen, instruct, mistral)
          // 2. Time aggregation: Group contributions by month for stream chart
          //    - Handle month boundaries and date range calculations
          //    - Support configurable time ranges
          // 3. Data transformation: Format API response for ModelContributionStream component
          //    - Map API response to StreamDataPoint array with month and model-specific values
          //    - Handle missing model data gracefully */}
          <BentoPanel>
            <ModelContributionStream
              title="Model Contributions"
              subtitle="Lines of code by AI model"
            />
          </BentoPanel>
        </div>

        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
          <BentoPanel>
            <TaskCompletionGauge
              title="Task Balance"
              subtitle="Completion vs Creation Rate"
            />
          </BentoPanel>
        </div>

        {/* TODO: Replace hardcoded efficiency metric with observability data from v3 API with the following requirements:
        // 1. Efficiency metrics fetching: Load server efficiency time-series data
        //    - Data source: GET /api/observability/efficiency endpoint from `iterations/v3/system-observability` crate
        //    - Return server efficiency metrics over time
        //    - Include efficiency percentage and time-series bar data
        // 2. Real-time updates: Refresh efficiency metrics periodically
        //    - Poll API endpoint at configurable intervals
        //    - Handle loading and error states
        // 3. Data transformation: Format API response for ServerEfficiencyChart component
        //    - Map API response to bars array with height values
        //    - Calculate overall efficiency percentage for display */}
        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
          <BentoPanel>
            <ServerEfficiencyChart
              title="Server Efficiency Analysis"
              efficiency={55}
            />
          </BentoPanel>
        </div>
      </div>
    </div>
  );
}
