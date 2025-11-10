import { useState } from "react";
import {
  Search,
  Lightbulb,
  FileText,
  Code,
  CheckCircle2,
  Loader2,
  Circle,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import type { Task } from "./Chat";

interface TaskTimelineProps {
  tasks: Task[];
}

function getRelativeTime(date: Date): string {
  const now = new Date();
  const diffInSeconds = Math.floor(
    (now.getTime() - date.getTime()) / 1000,
  );

  if (diffInSeconds < 60) {
    return `${diffInSeconds}s ago`;
  }

  const diffInMinutes = Math.floor(diffInSeconds / 60);
  if (diffInMinutes < 60) {
    return `${diffInMinutes}m ago`;
  }

  const diffInHours = Math.floor(diffInMinutes / 60);
  if (diffInHours < 24) {
    return `${diffInHours}h ago`;
  }

  const diffInDays = Math.floor(diffInHours / 24);
  if (diffInDays < 7) {
    return `${diffInDays}d ago`;
  }

  const diffInWeeks = Math.floor(diffInDays / 7);
  if (diffInWeeks < 4) {
    return `${diffInWeeks}w ago`;
  }

  const diffInMonths = Math.floor(diffInDays / 30);
  if (diffInMonths < 12) {
    return `${diffInMonths}mo ago`;
  }

  const diffInYears = Math.floor(diffInDays / 365);
  return `${diffInYears}y ago`;
}

export function TaskTimeline({ tasks }: TaskTimelineProps) {
  const [expandedTasks, setExpandedTasks] = useState<
    Set<string>
  >(new Set());

  const toggleTask = (taskId: string) => {
    const newExpanded = new Set(expandedTasks);
    if (newExpanded.has(taskId)) {
      newExpanded.delete(taskId);
    } else {
      newExpanded.add(taskId);
    }
    setExpandedTasks(newExpanded);
  };

  const getTaskIcon = (task: Task) => {
    const iconClass = "w-4 h-4";

    // Determine icon based on task name/type
    if (
      task.name.toLowerCase().includes("search") ||
      task.name.toLowerCase().includes("analyzing")
    ) {
      return <Search className={iconClass} />;
    } else if (
      task.name.toLowerCase().includes("think") ||
      task.name.toLowerCase().includes("reasoning")
    ) {
      return <Lightbulb className={iconClass} />;
    } else if (
      task.name.toLowerCase().includes("generating") ||
      task.name.toLowerCase().includes("creating")
    ) {
      return <Code className={iconClass} />;
    } else if (
      task.name.toLowerCase().includes("format") ||
      task.name.toLowerCase().includes("output")
    ) {
      return <FileText className={iconClass} />;
    } else {
      return <Circle className={iconClass} />;
    }
  };

  const getStatusIcon = (status: Task["status"]) => {
    switch (status) {
      case "completed":
        return (
          <CheckCircle2 className="w-4 h-4 text-green-700" />
        );
      case "in-progress":
        return (
          <Loader2 className="w-4 h-4 text-slate-600 animate-spin" />
        );
      case "failed":
        return <Circle className="w-4 h-4 text-red-500" />;
      default:
        return <Circle className="w-4 h-4 text-gray-600" />;
    }
  };

  const isThinkingTask = (task: Task) => {
    return (
      task.name.toLowerCase().includes("think") ||
      task.name.toLowerCase().includes("reasoning") ||
      task.name.toLowerCase().includes("analyzing")
    );
  };

  if (tasks.length === 0) return null;

  return (
    <div className="space-y-0">
      {tasks.map((task, index) => {
        const isExpanded = expandedTasks.has(task.id);
        const canExpand =
          isThinkingTask(task) &&
          task.status === "completed" &&
          task.result;
        const isLast = index === tasks.length - 1;

        return (
          <div key={task.id} className="relative">
            {/* Vertical connecting line */}
            {!isLast && (
              <div className="absolute left-[19px] top-[28px] w-[1px] h-[calc(100%+8px)] bg-gray-800" />
            )}

            {/* Task row */}
            <div
              className={`flex items-start gap-3 py-2 ${canExpand ? "cursor-pointer hover:bg-zinc-800/30 rounded-lg -mx-2 px-2" : ""}`}
              onClick={() => canExpand && toggleTask(task.id)}
            >
              {/* Icon container */}
              <div className="relative shrink-0 w-8 h-8 rounded-full border border-gray-800 flex items-center justify-center bg-[rgb(15,15,15)]">
                <div
                  className={`${
                    task.status === "completed"
                      ? "text-gray-300"
                      : task.status === "in-progress"
                        ? "text-slate-600"
                        : "text-gray-600"
                  }`}
                >
                  {getTaskIcon(task)}
                </div>

                {/* Status indicator badge */}
                <div className="absolute -bottom-2 -right-2 bg-[#0f0f0f] rounded-full p-0.5">
                  {getStatusIcon(task.status)}
                </div>
              </div>

              {/* Task content */}
              <div className="flex-1 min-w-0 pt-2">
                <div className="flex items-center gap-2">
                  <span
                    className={`text-sm ${
                      task.status === "completed"
                        ? "text-gray-300"
                        : task.status === "in-progress"
                          ? "text-slate-600"
                          : "text-gray-500"
                    }`}
                  >
                    {task.name}
                  </span>
                  {task.status === "in-progress" && (
                    <span className="text-xs text-gray-600">
                      •
                    </span>
                  )}
                  <span className="text-xs text-gray-600">
                    {getRelativeTime(task.timestamp)}
                  </span>
                  {canExpand && (
                    <div className="shrink-0">
                      {isExpanded ? (
                        <ChevronDown className="w-3.5 h-3.5 text-gray-500" />
                      ) : (
                        <ChevronRight className="w-3.5 h-3.5 text-gray-500" />
                      )}
                    </div>
                  )}
                </div>

                {/* Expandable thought process */}
                {canExpand && isExpanded && task.result && (
                  <div className="mt-2 p-3 rounded-lg">
                    <p className="text-xs text-gray-400 leading-relaxed whitespace-pre-wrap">
                      {task.result}
                    </p>
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}