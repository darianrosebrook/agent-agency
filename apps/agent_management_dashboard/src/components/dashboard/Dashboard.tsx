import { Suspense, lazy } from "react";
import { LayoutGrid } from "lucide-react";
import { cn } from "../primitives/utils";
import styles from "./Dashboard.module.scss";

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
  return (
    <div className={styles.dashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerTop}>
          <LayoutGrid className={styles.headerIcon} size={16} />
          <span className={styles.headerLabel}>Dashboard</span>
        </div>
        {/* TODO: Replace hardcoded user name with data from v3 API with the following requirements:
        // 1. User data fetching: Load current user information from API
        //    - Data source: GET /api/users/me endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
        //    - Database table: PostgreSQL `users` table
        //    - Include user name, email, and other profile information
        // 2. Authentication: Ensure user is authenticated before fetching
        //    - Handle 401/403 errors and redirect to login if needed
        //    - Store authentication token securely
        // 3. Loading state: Show loading indicator while fetching user data
        //    - Display fallback text if user data is not yet loaded
        //    - Handle error states gracefully */}
        <h1 className={styles.headerTitle}>Welcome back John Doe!</h1>
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
          <Suspense fallback={<ChartSkeleton />}>
            <TaskProgressChart completedTasks={19} totalTasks={40} />
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
          <Suspense fallback={<ChartSkeleton />}>
            <ServerEfficiencyChart
              title="Server Efficiency Analysis"
              efficiency={55}
            />
          </Suspense>
        </div>
      </div>
    </div>
  );
}
