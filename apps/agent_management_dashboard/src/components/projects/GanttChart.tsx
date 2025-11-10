"use client";

import { useMemo, useState } from "react";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";

export type ZoomLevel = "day" | "week" | "month" | "quarter";

export interface TimelineTask {
  id: string;
  title: string;
  startDate: Date;
  endDate: Date;
  workerId: string;
  worker: string;
  status: "completed" | "in-progress" | "pending" | "backlog" | "todo" | "done";
  tags?: string[];
  progress?: number;
  description?: string;
}

interface GanttChartProps {
  tasks: TimelineTask[];
  zoomLevel: ZoomLevel;
}

interface GroupedTask {
  id: string;
  tasks: TimelineTask[];
  startDate: Date;
  endDate: Date;
  worker: string;
  workerId: string;
}

export function GanttChart({ tasks, zoomLevel }: GanttChartProps) {
  const [hoveredTask, setHoveredTask] = useState<string | null>(null);

  // Group tasks by worker
  const tasksByWorker = useMemo(() => {
    const grouped = new Map<string, TimelineTask[]>();
    tasks.forEach((task) => {
      const existing = grouped.get(task.workerId) ?? [];
      grouped.set(task.workerId, [...existing, task]);
    });

    // Sort tasks by start date for each worker
    grouped.forEach((tasks) => {
      tasks.sort((a, b) => a.startDate.getTime() - b.startDate.getTime());
    });

    return grouped;
  }, [tasks]);

  // Calculate date range
  const dateRange = useMemo(() => {
    if (tasks.length === 0) {
      return { start: new Date(), end: new Date() };
    }
    const start = new Date(
      Math.min(...tasks.map((t) => t.startDate.getTime()))
    );
    const end = new Date(Math.max(...tasks.map((t) => t.endDate.getTime())));

    // Extend range slightly for padding
    start.setDate(start.getDate() - 2);
    end.setDate(end.getDate() + 2);

    return { start, end };
  }, [tasks]);

  // Generate time columns based on zoom level
  const timeColumns = useMemo(() => {
    const columns: Date[] = [];
    const current = new Date(dateRange.start);

    while (current <= dateRange.end) {
      columns.push(new Date(current));

      switch (zoomLevel) {
        case "day":
          current.setDate(current.getDate() + 1);
          break;
        case "week":
          current.setDate(current.getDate() + 7);
          break;
        case "month":
          current.setMonth(current.getMonth() + 1);
          break;
        case "quarter":
          current.setMonth(current.getMonth() + 3);
          break;
      }
    }

    return columns;
  }, [dateRange, zoomLevel]);

  // Group adjacent tasks when zoomed out
  const groupedTasksByWorker = useMemo((): Map<string, GroupedTask[]> => {
    if (zoomLevel === "day" || zoomLevel === "week") {
      // Show individual tasks at detailed zoom levels
      // Convert TimelineTask[] to GroupedTask[] for consistency
      const grouped = new Map<string, GroupedTask[]>();
      tasksByWorker.forEach((tasks, workerId) => {
        const groups: GroupedTask[] = tasks.map((task) => ({
          id: task.id,
          tasks: [task],
          startDate: task.startDate,
          endDate: task.endDate,
          worker: task.worker,
          workerId: task.workerId,
        }));
        grouped.set(workerId, groups);
      });
      return grouped;
    }

    // Group adjacent tasks at higher zoom levels
    const grouped = new Map<string, GroupedTask[]>();

    tasksByWorker.forEach((tasks, workerId) => {
      const groups: GroupedTask[] = [];
      let currentGroup: TimelineTask[] = [];

      tasks.forEach((task, index) => {
        if (currentGroup.length === 0) {
          currentGroup.push(task);
        } else {
          const lastTask = currentGroup[currentGroup.length - 1];
          const daysBetween = Math.floor(
            (task.startDate.getTime() - lastTask.endDate.getTime()) /
              (1000 * 60 * 60 * 24)
          );

          // Group if tasks are within threshold
          const threshold = zoomLevel === "month" ? 7 : 14;
          if (daysBetween <= threshold) {
            currentGroup.push(task);
          } else {
            // Save current group and start new one
            if (currentGroup.length > 0) {
              groups.push({
                id: currentGroup.map((t: TimelineTask) => t.id).join("-"),
                tasks: [...currentGroup],
                startDate: currentGroup[0].startDate,
                endDate: currentGroup[currentGroup.length - 1].endDate,
                worker: currentGroup[0].worker,
                workerId: currentGroup[0].workerId,
              });
            }
            currentGroup = [task];
          }
        }

        // Handle last group
        if (index === tasks.length - 1 && currentGroup.length > 0) {
          groups.push({
            id: currentGroup.map((t) => t.id).join("-"),
            tasks: [...currentGroup],
            startDate: currentGroup[0].startDate,
            endDate: currentGroup[currentGroup.length - 1].endDate,
            worker: currentGroup[0].worker,
            workerId: currentGroup[0].workerId,
          });
        }
      });

      grouped.set(workerId, groups);
    });

    return grouped;
  }, [tasksByWorker, zoomLevel]);

  const getColumnWidth = () => {
    switch (zoomLevel) {
      case "day":
        return 80;
      case "week":
        return 100;
      case "month":
        return 120;
      case "quarter":
        return 150;
    }
  };

  const formatColumnHeader = (date: Date) => {
    switch (zoomLevel) {
      case "day":
        return date.toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
        });
      case "week":
        return `Week ${Math.ceil(
          date.getDate() / 7
        )}, ${date.toLocaleDateString("en-US", { month: "short" })}`;
      case "month":
        return date.toLocaleDateString("en-US", {
          month: "short",
          year: "numeric",
        });
      case "quarter":
        return `Q${Math.floor(date.getMonth() / 3) + 1} ${date.getFullYear()}`;
    }
  };

  const calculateTaskPosition = (startDate: Date, endDate: Date) => {
    const totalDays =
      (dateRange.end.getTime() - dateRange.start.getTime()) /
      (1000 * 60 * 60 * 24);
    const startOffset =
      (startDate.getTime() - dateRange.start.getTime()) / (1000 * 60 * 60 * 24);
    const duration =
      (endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24);

    const left = (startOffset / totalDays) * 100;
    const width = (duration / totalDays) * 100;

    return {
      left: `${left}%`,
      width: `${Math.max(width, 0.5)}%`,
    };
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "completed":
        return "bg-[#1f3a2d] border-[#5cd18c]";
      case "in-progress":
        return "bg-[#1f2d3a] border-[#54a0ff]";
      case "pending":
        return "bg-[#2a2a2a] border-[#888888]";
      default:
        return "bg-[#262626] border-[#404040]";
    }
  };

  const showDetailedView = zoomLevel === "day" || zoomLevel === "week";

  return (
    <TooltipProvider>
      <div className="min-w-max">
        {/* Timeline Header */}
        <div className="sticky top-0 z-20 bg-[#0d0d0d] border-b border-[#262626]">
          <div className="flex">
            <div className="w-64 border-r border-[#262626] px-4 py-3">
              <span className="text-[#888888] text-sm">Team Member</span>
            </div>
            <div className="flex-1 flex">
              {timeColumns.map((date, index) => (
                <div
                  key={index}
                  style={{ width: getColumnWidth() }}
                  className="border-r border-[#262626] px-2 py-3 text-center"
                >
                  <span className="text-[#888888] text-xs">
                    {formatColumnHeader(date)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Timeline Rows */}
        <div className="relative">
          {Array.from(tasksByWorker.entries()).map(
            ([workerId, workerTasks]) => {
              const worker = workerTasks[0];

              return (
                <div
                  key={workerId}
                  className="flex border-b border-[#262626] hover:bg-[#1a1a1a]/30"
                >
                  {/* Worker Info */}
                  <div className="w-64 border-r border-[#262626] px-4 py-6 flex items-center gap-3">
                    <Avatar className="w-8 h-8">
                      <AvatarImage
                        src={`https://i.pravatar.cc/150?u=${workerId}`}
                      />
                      <AvatarFallback>
                        {worker.worker
                          .split(" ")
                          .map((n) => n[0])
                          .join("")}
                      </AvatarFallback>
                    </Avatar>
                    <div className="flex-1 min-w-0">
                      <p className="text-white text-sm truncate">
                        {worker.worker}
                      </p>
                      <p className="text-[#888888] text-xs">
                        {
                          workerTasks.filter((t: TimelineTask) => t.status === "completed")
                            .length
                        }
                        /{workerTasks.length} completed
                      </p>
                    </div>
                  </div>

                  {/* Task Timeline */}
                  <div
                    className="flex-1 relative"
                    style={{
                      height: showDetailedView ? 80 : 60,
                    }}
                  >
                    {/* Background Grid */}
                    <div className="absolute inset-0 flex">
                      {timeColumns.map((_, index) => (
                        <div
                          key={index}
                          style={{ width: getColumnWidth() }}
                          className="border-r border-[#1a1a1a]"
                        />
                      ))}
                    </div>

                    {/* Tasks */}
                    {showDetailedView
                      ? // Detailed view - show individual tasks
                        workerTasks.map((task) => {
                          const position = calculateTaskPosition(
                            task.startDate,
                            task.endDate
                          );

                          return (
                            <Tooltip key={task.id}>
                              <TooltipTrigger asChild>
                                <div
                                  className={`absolute top-2 h-12 rounded-lg border-l-4 ${getStatusColor(
                                    task.status
                                  )} px-3 py-1.5 cursor-pointer transition-all hover:shadow-lg hover:z-10 ${
                                    hoveredTask === task.id
                                      ? "ring-2 ring-white/20"
                                      : ""
                                  }`}
                                  style={position}
                                  onMouseEnter={() => setHoveredTask(task.id)}
                                  onMouseLeave={() => setHoveredTask(null)}
                                >
                                  <p className="text-white text-xs truncate mb-0.5">
                                    {task.title}
                                  </p>
                                  <div className="flex gap-1">
                                    {task.tags?.slice(0, 2).map((tag: string, i: number) => (
                                      <span
                                        key={i}
                                        className="text-[10px] text-[#888888] bg-[#262626] px-1.5 py-0.5 rounded"
                                      >
                                        {tag}
                                      </span>
                                    ))}
                                  </div>
                                </div>
                              </TooltipTrigger>
                              <TooltipContent className="bg-[#1a1a1a] border-[#262626] text-white max-w-xs">
                                <div className="space-y-2">
                                  <p className="font-semibold">{task.title}</p>
                                  {task.description && (
                                    <p className="text-sm text-[#888888]">
                                      {task.description}
                                    </p>
                                  )}
                                  <div className="flex gap-2 text-xs text-[#888888]">
                                    <span>
                                      {task.startDate.toLocaleDateString()}
                                    </span>
                                    <span>→</span>
                                    <span>
                                      {task.endDate.toLocaleDateString()}
                                    </span>
                                  </div>
                                  {task.tags && task.tags.length > 0 && (
                                    <div className="flex gap-1.5 flex-wrap">
                                      {task.tags.map((tag: string, i: number) => (
                                        <span
                                          key={i}
                                          className="text-xs bg-[#262626] px-2 py-0.5 rounded"
                                        >
                                          {tag}
                                        </span>
                                      ))}
                                    </div>
                                  )}
                                </div>
                              </TooltipContent>
                            </Tooltip>
                          );
                        })
                      : // Grouped view - show combined task groups
                        (groupedTasksByWorker.get(workerId) ?? []).map(
                          (group: GroupedTask) => {
                            const position = calculateTaskPosition(
                              group.startDate,
                              group.endDate
                            );
                            const hasMultipleTasks = group.tasks.length > 1;

                            return (
                              <Tooltip key={group.id}>
                                <TooltipTrigger asChild>
                                  <div
                                    className={`absolute top-2 h-10 rounded-lg border-l-4 ${getStatusColor(
                                      group.tasks.every(
                                        (t: TimelineTask) => t.status === "completed"
                                      )
                                        ? "completed"
                                        : "in-progress"
                                    )} px-3 py-2 cursor-pointer transition-all hover:shadow-lg hover:z-10`}
                                    style={position}
                                  >
                                    <div className="flex items-center justify-between">
                                      <p className="text-white text-xs">
                                        {hasMultipleTasks
                                          ? `${group.tasks.length} tasks`
                                          : group.tasks[0].title}
                                      </p>
                                      <span className="text-[10px] text-[#888888] ml-2">
                                        {
                                          group.tasks.filter(
                                            (t: TimelineTask) => t.status === "completed"
                                          ).length
                                        }
                                        /{group.tasks.length}
                                      </span>
                                    </div>
                                  </div>
                                </TooltipTrigger>
                                <TooltipContent className="bg-[#1a1a1a] border-[#262626] text-white max-w-md">
                                  <div className="space-y-2">
                                    <p className="font-semibold text-sm">
                                      Task Group ({group.tasks.length} tasks)
                                    </p>
                                    <div className="space-y-1.5 max-h-48 overflow-y-auto">
                                      {group.tasks.map((task: TimelineTask) => (
                                        <div
                                          key={task.id}
                                          className="text-xs border-l-2 border-[#404040] pl-2"
                                        >
                                          <p className="text-white">
                                            {task.title}
                                          </p>
                                          <p className="text-[#888888] text-[10px]">
                                            {task.startDate.toLocaleDateString()}{" "}
                                            -{" "}
                                            {task.endDate.toLocaleDateString()}
                                          </p>
                                        </div>
                                      ))}
                                    </div>
                                  </div>
                                </TooltipContent>
                              </Tooltip>
                            );
                          }
                        )}
                  </div>
                </div>
              );
            }
          )}
        </div>
      </div>
    </TooltipProvider>
  );
}
