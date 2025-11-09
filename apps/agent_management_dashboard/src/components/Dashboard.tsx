import { TaskProgressChart } from "./TaskProgressChart";
import { RadialTaskProgress } from "./RadialTaskProgress";
import { MultiRingProgress } from "./MultiRingProgress";
import { CodeContributionChart } from "./CodeContributionChart";
import { HexagonHeatmap } from "./HexagonHeatmap";
import { ModelContributionStream } from "./ModelContributionStream";
import { TaskCompletionGauge } from "./TaskCompletionGauge";
import { ServerEfficiencyChart } from "./ServerEfficiencyChart";
import { LayoutGrid } from "lucide-react";

export function Dashboard() {
  return (
    <div className="p-8">
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center gap-2 text-zinc-300 mb-4">
          <LayoutGrid className="w-4 h-4" />
          <span className="text-sm">Dashboard</span>
        </div>
        <h1 className="text-3xl text-white">Welcome back John Doe!</h1>
      </div>

      {/* Bento Grid */}
      <div className="grid grid-cols-12 gap-4 auto-rows-[140px]">
        {/* Task Progress Chart - spans 2 rows and 5 columns */}
        <div className="col-span-5 row-span-2">
          <TaskProgressChart completedTasks={19} totalTasks={40} />
        </div>

        {/* Radial Task Progress - spans 2 rows and 7 columns */}
        <div className="col-span-7 row-span-2">
          <RadialTaskProgress />
        </div>

        {/* Hexagon Heatmap - spans 6 rows and 8 columns */}
        <div className="col-span-8 row-span-6">
          <HexagonHeatmap rows={12} cols={16} hexSize={28} />
        </div>

        {/* Multi-Ring Progress - spans 6 rows and 4 columns */}
        <div className="col-span-4 row-span-6">
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
        <div className="col-span-12 row-span-3">
          <CodeContributionChart
            title="Overall Contribution"
            subtitle="2 Agents over the last 30 days"
            days={30}
          />
        </div>

        {/* Small panels row */}
        <div className="col-span-4 row-span-2">
          <ModelContributionStream
            title="Model Contributions"
            subtitle="Lines of code by AI model"
          />
        </div>

        <div className="col-span-4 row-span-2">
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
