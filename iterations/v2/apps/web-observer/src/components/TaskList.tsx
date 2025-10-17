import type { Task } from "@/types/api";

interface TaskListProps {
  tasks: Task[];
  loading: boolean;
  onTaskSelect: (task: Task) => void;
  onRefresh: () => void;
}

export default function TaskList({
  tasks,
  loading,
  onTaskSelect,
  onRefresh,
}: TaskListProps) {
  const getStatusColor = (state: string) => {
    switch (state.toLowerCase()) {
      case "completed":
        return "text-green-600 bg-green-100";
      case "running":
      case "in_progress":
        return "text-blue-600 bg-blue-100";
      case "failed":
      case "error":
        return "text-red-600 bg-red-100";
      case "pending":
      case "queued":
        return "text-yellow-600 bg-yellow-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  if (loading) {
    return (
      <div className="space-y-4">
        {Array.from({ length: 3 }).map((_, i) => (
          <div
            key={i}
            className="animate-pulse border border-gray-200 rounded-lg p-4"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="h-4 bg-gray-200 rounded w-1/4"></div>
              <div className="h-4 bg-gray-200 rounded w-16"></div>
            </div>
            <div className="h-3 bg-gray-200 rounded w-3/4 mb-2"></div>
            <div className="h-3 bg-gray-200 rounded w-1/2"></div>
          </div>
        ))}
      </div>
    );
  }

  if (tasks.length === 0) {
    return (
      <div className="text-center py-12">
        <svg
          className="mx-auto h-12 w-12 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1}
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
        <h3 className="mt-2 text-sm font-medium text-gray-900">No tasks</h3>
        <p className="mt-1 text-sm text-gray-500">
          Submit a task above to get started.
        </p>
        <div className="mt-6">
          <button
            onClick={onRefresh}
            className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            <svg
              className="-ml-1 mr-2 h-4 w-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            Refresh
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <p className="text-sm text-gray-600">
          {tasks.length} task{tasks.length !== 1 ? "s" : ""}
        </p>
        <button
          onClick={onRefresh}
          className="inline-flex items-center px-3 py-1 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          <svg
            className="-ml-1 mr-1 h-4 w-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
          Refresh
        </button>
      </div>

      <div className="space-y-3">
        {tasks.map((task) => (
          <div
            key={task.taskId}
            className="border border-gray-200 rounded-lg p-4 hover:border-gray-300 cursor-pointer transition-colors"
            onClick={() => onTaskSelect(task)}
          >
            <div className="flex items-start justify-between mb-2">
              <div className="flex-1 min-w-0">
                <div className="flex items-center space-x-2 mb-1">
                  <span className="text-sm font-medium text-gray-900 truncate">
                    {task.taskId}
                  </span>
                  <span
                    className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(
                      task.state
                    )}`}
                  >
                    {task.state}
                  </span>
                </div>
                <p className="text-sm text-gray-600 line-clamp-2">
                  {task.currentPlan || "No current plan"}
                </p>
              </div>
            </div>

            <div className="flex items-center justify-between text-xs text-gray-500 mt-3">
              <span>Updated: {formatDate(task.lastUpdated)}</span>
              <div className="flex items-center space-x-4">
                <span>Progress: {task.progress.length} steps</span>
                {task.caws && (
                  <span
                    className={
                      task.caws.passed ? "text-green-600" : "text-red-600"
                    }
                  >
                    CAWS: {task.caws.passed ? "✓" : "✗"}
                  </span>
                )}
              </div>
            </div>

            {task.nextActions && task.nextActions.length > 0 && (
              <div className="mt-2">
                <p className="text-xs text-gray-600">
                  Next: {task.nextActions.slice(0, 2).join(", ")}
                  {task.nextActions.length > 2 &&
                    ` +${task.nextActions.length - 2} more`}
                </p>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
