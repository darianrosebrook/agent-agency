import type {
  ObserverMetricsSnapshot,
  ObserverProgressSummary,
} from "@/types/api";

interface MetricsOverviewProps {
  metrics: ObserverMetricsSnapshot | null;
  progress: ObserverProgressSummary | null;
}

export default function MetricsOverview({
  metrics,
  progress,
}: MetricsOverviewProps) {
  if (!metrics || !progress) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Metrics Overview
        </h2>
        <div className="animate-pulse space-y-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {Array.from({ length: 8 }).map((_, i) => (
              <div key={i} className="h-16 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case "running":
        return "text-green-600";
      case "completed":
        return "text-blue-600";
      case "degraded":
        return "text-yellow-600";
      case "not_started":
        return "text-gray-600";
      default:
        return "text-gray-600";
    }
  };

  const formatPercentage = (value: number) => `${(value * 100).toFixed(1)}%`;

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-lg font-semibold text-gray-900 mb-4">
        Metrics Overview
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* Reasoning Metrics */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-gray-600">Reasoning Depth</h3>
          <div className="space-y-2">
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Average</span>
              <span className="text-sm font-medium">
                {metrics.reasoningDepthAvg.toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">P95</span>
              <span className="text-sm font-medium">
                {metrics.reasoningDepthP95.toFixed(2)}
              </span>
            </div>
          </div>
        </div>

        {/* Task Metrics */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-gray-600">Task Status</h3>
          <div className="space-y-2">
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Active</span>
              <span className="text-sm font-medium">{metrics.activeTasks}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Queued</span>
              <span className="text-sm font-medium">{metrics.queuedTasks}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Success Rate</span>
              <span className="text-sm font-medium">
                {formatPercentage(metrics.taskSuccessRate)}
              </span>
            </div>
          </div>
        </div>

        {/* System Health */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-gray-600">System Health</h3>
          <div className="space-y-2">
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Degraded</span>
              <span
                className={`text-sm font-medium ${
                  metrics.observerDegraded ? "text-red-600" : "text-green-600"
                }`}
              >
                {metrics.observerDegraded ? "Yes" : "No"}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Violations</span>
              <span className="text-sm font-medium">
                {metrics.policyViolations}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Queue Depth</span>
              <span className="text-sm font-medium">{metrics.queueDepth}</span>
            </div>
          </div>
        </div>

        {/* Tool Usage */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-gray-600">Tool Usage</h3>
          <div className="space-y-2">
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Budget Util</span>
              <span className="text-sm font-medium">
                {formatPercentage(metrics.toolBudgetUtilization)}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-500">Debate Breadth</span>
              <span className="text-sm font-medium">
                {metrics.debateBreadthAvg.toFixed(2)}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Progress Summary */}
      <div className="mt-6 pt-6 border-t">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-medium text-gray-600">
            Reasoning Progress
          </h3>
          <span
            className={`text-sm font-medium ${getStatusColor(progress.status)}`}
          >
            {progress.status.replace("_", " ").toUpperCase()}
          </span>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
          <div className="text-center">
            <div className="text-lg font-semibold text-gray-900">
              {progress.reasoningSteps.observations}
            </div>
            <div className="text-xs text-gray-500">Observations</div>
          </div>
          <div className="text-center">
            <div className="text-lg font-semibold text-gray-900">
              {progress.reasoningSteps.analyses}
            </div>
            <div className="text-xs text-gray-500">Analyses</div>
          </div>
          <div className="text-center">
            <div className="text-lg font-semibold text-gray-900">
              {progress.reasoningSteps.plans}
            </div>
            <div className="text-xs text-gray-500">Plans</div>
          </div>
          <div className="text-center">
            <div className="text-lg font-semibold text-gray-900">
              {progress.reasoningSteps.decisions}
            </div>
            <div className="text-xs text-gray-500">Decisions</div>
          </div>
          <div className="text-center">
            <div className="text-lg font-semibold text-gray-900">
              {progress.reasoningSteps.executions}
            </div>
            <div className="text-xs text-gray-500">Executions</div>
          </div>
          <div className="text-center">
            <div className="text-lg font-semibold text-gray-900">
              {progress.reasoningSteps.verifications}
            </div>
            <div className="text-xs text-gray-500">Verifications</div>
          </div>
        </div>

        <div className="mt-4 text-center">
          <span className="text-sm text-gray-600">
            Total Steps:{" "}
            <span className="font-medium">{progress.totalReasoningSteps}</span>
          </span>
          <span className="mx-4 text-gray-300">|</span>
          <span className="text-sm text-gray-600">
            Uptime:{" "}
            <span className="font-medium">{progress.uptimeMinutes}m</span>
          </span>
        </div>
      </div>
    </div>
  );
}
