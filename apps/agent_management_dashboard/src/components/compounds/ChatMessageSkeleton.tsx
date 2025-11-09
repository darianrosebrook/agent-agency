import { Bot } from "lucide-react";
import { Skeleton } from "../ui/skeleton";
import { TaskTimeline } from "../TaskTimeline";
import type { Task } from "../composers/Chat";

interface ChatMessageSkeletonProps {
  tasks?: Task[];
}

export function ChatMessageSkeleton({ tasks = [] }: ChatMessageSkeletonProps) {
  return (
    <div className="space-y-4">
      {/* Task Timeline - appears above message */}
      {tasks.length > 0 && (
        <div className="ml-12">
          <TaskTimeline tasks={tasks} />
        </div>
      )}

      {/* Loading Message */}
      <div className="flex gap-4">
        {/* Avatar */}
        <div className="shrink-0 w-8 h-8 rounded-full bg-gray-800 flex items-center justify-center">
          <Bot className="w-4 h-4 text-gray-300" />
        </div>

        {/* Content */}
        <div className="flex-1 w-full">
          {/* Loading Card */}
          <div className="bg-[#1a1a1a] border border-gray-800 rounded-lg p-4 w-full">
            {/* Skeleton Content */}
            <div className="space-y-3">
              {/* Content Lines Skeleton */}
              <div className="space-y-2">
                <Skeleton className="h-3 w-full bg-gray-800" />
                <Skeleton className="h-3 w-[90%] bg-gray-800" />
                <Skeleton className="h-3 w-[95%] bg-gray-800" />
                <Skeleton className="h-3 w-[85%] bg-gray-800" />
                <Skeleton className="h-3 w-[92%] bg-gray-800" />
                <Skeleton className="h-3 w-[88%] bg-gray-800" />
                <Skeleton className="h-3 w-[75%] bg-gray-800" />
              </div>

              {/* Pulsing Indicator */}
              <div className="flex items-center gap-2 pt-2">
                <div className="flex gap-1">
                  <div
                    className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"
                    style={{ animationDelay: "0ms" }}
                  ></div>
                  <div
                    className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"
                    style={{ animationDelay: "150ms" }}
                  ></div>
                  <div
                    className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"
                    style={{ animationDelay: "300ms" }}
                  ></div>
                </div>
                <span className="text-xs text-gray-500">
                  Generating response...
                </span>
              </div>
            </div>
          </div>

          {/* Timestamp Skeleton */}
          <Skeleton className="h-3 w-16 mt-2 bg-gray-800" />
        </div>
      </div>
    </div>
  );
}
