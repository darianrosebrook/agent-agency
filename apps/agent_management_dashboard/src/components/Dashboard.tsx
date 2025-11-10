import { TaskProgressChart } from "./TaskProgressChart";
import { RadialTaskProgress } from "./RadialTaskProgress";
import { MultiRingProgress } from "./MultiRingProgress";
import { CodeContributionChart } from "./CodeContributionChart";
import { HexagonHeatmap } from "./HexagonHeatmap";
import { ModelContributionStream } from "./ModelContributionStream";
import { TaskCompletionGauge } from "./TaskCompletionGauge";
import { ServerEfficiencyChart } from "./ServerEfficiencyChart";
import { LayoutGrid } from "lucide-react";
import styles from "./Dashboard.module.scss";

export function Dashboard() {
  return (
    <div className={styles.dashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerTop}>
          <LayoutGrid className={styles.headerIcon} />
          <span className={styles.headerLabel}>Dashboard</span>
        </div>
        <h1 className={styles.headerTitle}>Welcome back John Doe!</h1>
      </div>

      {/* Bento Grid */}
      <div className={styles.bentoGrid}>
        {/* Task Progress Chart - spans 2 rows and 5 columns */}
        <div className={styles.gridItem5Col2Row}>
          <TaskProgressChart completedTasks={19} totalTasks={40} />
        </div>

        {/* Radial Task Progress - spans 2 rows and 7 columns */}
        <div className={styles.gridItem7Col2Row}>
          <RadialTaskProgress />
        </div>

        {/* Hexagon Heatmap - spans 6 rows and 8 columns */}
        <div className={styles.gridItem8Col6Row}>
          <HexagonHeatmap radius={8} hexSize={28} />
        </div>

        {/* Multi-Ring Progress - spans 6 rows and 4 columns */}
        <div className={styles.gridItem4Col6Row}>
          <MultiRingProgress
            tasks={[
              { name: "Progress 1", progress: 85, color: "#e0e7ff" },
              { name: "Progress 2", progress: 75, color: "#818cf8" },
              { name: "Progress 3", progress: 60, color: "#6366f1" },
            ]}
            projectedTimeline="15 business days"
          />
        </div>

        {/* Code Contribution Chart - spans 3 rows and 12 columns */}
        <div className={styles.gridItem12Col3Row}>
          <CodeContributionChart
            title="Overall Contribution"
            subtitle="2 Agents over the last 30 days"
            days={30}
          />
        </div>

        {/* Small panels row */}
        <div className={styles.gridItem4Col2Row}>
          <ModelContributionStream
            title="Model Contributions"
            subtitle="Lines of code by AI model"
          />
        </div>

        <div className={styles.gridItem4Col2Row}>
          <TaskCompletionGauge
            title="Task Balance"
            subtitle="Completion vs Creation Rate"
          />
        </div>

        <ServerEfficiencyChart
          title="Server Efficiency Analysis"
          efficiency={55}
        />
      </div>
    </div>
  );
}
