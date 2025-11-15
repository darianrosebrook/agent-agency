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
import { useProjectStore } from "../../lib/stores";

export function Dashboard() {
  const [user, setUser] = useState<CurrentUser | null>(null);
  const [isLoadingUser, setIsLoadingUser] = useState(true);
  const { currentProjectId } = useProjectStore();

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
        {/* Note: TaskProgressChart fetches from API automatically if props not provided */}
        <div className={cn(styles.colSpan5, styles.rowSpan2)}>
          <TaskProgressChart />
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
        {/* Note: MultiRingProgress fetches milestone data from API when projectId provided */}
        <div className={cn(styles.colSpan4, styles.rowSpan6)}>
          <BentoPanel>
            <MultiRingProgress projectId={currentProjectId || undefined} />
          </BentoPanel>
        </div>

        {/* Code Contribution Chart - spans 3 rows and 12 columns */}
        {/* Note: CodeContributionChart fetches from API automatically */}
        <div className={cn(styles.colSpan12, styles.rowSpan3)}>
          <BentoPanel>
            <CodeContributionChart
              title="Overall Contribution"
              subtitle="2 Agents over the last 30 days"
              days={30}
            />
          </BentoPanel>
        </div>

        {/* Small panels row */}
        {/* Note: ModelContributionStream fetches from API automatically */}
        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
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

        {/* Note: ServerEfficiencyChart fetches from API automatically if efficiency prop not provided */}
        <div className={cn(styles.colSpan4, styles.rowSpan2)}>
          <BentoPanel>
            <ServerEfficiencyChart title="Server Efficiency Analysis" />
          </BentoPanel>
        </div>
      </div>
    </div>
  );
}
