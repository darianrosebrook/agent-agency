"use client";

import { useMemo, useState } from "react";
import type { TimelineTask, ZoomLevel } from "./composers/TimelineTab";
import { Avatar, AvatarFallback, AvatarImage } from "./primitives/avatar";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./primitives/tooltip";
import { cn } from "./primitives/utils";
import styles from "./GanttChart.module.scss";

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
            id: currentGroup.map((t: TimelineTask) => t.id).join("-"),
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

  const getStatusClass = (status: string) => {
    switch (status) {
      case "completed":
        return styles.taskStatusCompleted;
      case "in-progress":
        return styles.taskStatusInProgress;
      case "pending":
        return styles.taskStatusPending;
      default:
        return styles.taskStatusDefault;
    }
  };

  const showDetailedView = zoomLevel === "day" || zoomLevel === "week";

  return (
    <TooltipProvider>
      <div className={styles.ganttChart}>
        {/* Timeline Header */}
        <div className={styles.timelineHeader}>
          <div className={styles.timelineHeaderContent}>
            <div className={styles.memberColumn}>
              <span className={styles.memberColumnHeader}>Team Member</span>
            </div>
            <div className={styles.timeColumnsContainer}>
              {timeColumns.map((date, index) => (
                <div
                  key={index}
                  style={{ width: getColumnWidth() }}
                  className={styles.timeColumn}
                >
                  <span className={styles.timeColumnHeader}>
                    {formatColumnHeader(date)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Timeline Rows */}
        <div className={styles.timelineRows}>
          {Array.from(tasksByWorker.entries()).map(
            ([workerId, workerTasks]) => {
              const worker = workerTasks[0];

              return (
                <div key={workerId} className={styles.timelineRow}>
                  {/* Worker Info */}
                  <div className={styles.workerInfo}>
                    <Avatar className={styles.workerAvatar}>
                      <AvatarImage
                        src={`https://i.pravatar.cc/150?u=${workerId}`}
                      />
                      <AvatarFallback>
                        {worker.worker
                          .split(" ")
                          .map((n: string) => n[0])
                          .join("")}
                      </AvatarFallback>
                    </Avatar>
                    <div className={styles.workerInfoContent}>
                      <p className={styles.workerName}>{worker.worker}</p>
                      <p className={styles.workerStats}>
                        {
                          workerTasks.filter(
                            (t: TimelineTask) => t.status === "completed"
                          ).length
                        }
                        /{workerTasks.length} completed
                      </p>
                    </div>
                  </div>

                  {/* Task Timeline */}
                  <div
                    className={styles.taskTimeline}
                    style={{
                      height: showDetailedView ? 80 : 60,
                    }}
                  >
                    {/* Background Grid */}
                    <div className={styles.backgroundGrid}>
                      {timeColumns.map((_, index) => (
                        <div
                          key={index}
                          style={{ width: getColumnWidth() }}
                          className={styles.gridColumn}
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
                                  className={cn(
                                    styles.taskBar,
                                    styles.taskBarDetailed,
                                    getStatusClass(task.status),
                                    hoveredTask === task.id && styles.taskBarHovered
                                  )}
                                  style={position}
                                  onMouseEnter={() => setHoveredTask(task.id)}
                                  onMouseLeave={() => setHoveredTask(null)}
                                >
                                  <p className={styles.taskTitle}>
                                    {task.title}
                                  </p>
                                  {task.tags && task.tags.length > 0 && (
                                    <div className={styles.taskTags}>
                                      {task.tags.slice(0, 2).map((tag: string, i: number) => (
                                        <span key={i} className={styles.taskTag}>
                                          {tag}
                                        </span>
                                      ))}
                                    </div>
                                  )}
                                </div>
                              </TooltipTrigger>
                              <TooltipContent className={styles.tooltipContent}>
                                <div className={styles.tooltipInner}>
                                  <p className={styles.tooltipTitle}>
                                    {task.title}
                                  </p>
                                  {task.description && (
                                    <p className={styles.tooltipDescription}>
                                      {task.description}
                                    </p>
                                  )}
                                  <div className={styles.tooltipDates}>
                                    <span>
                                      {task.startDate.toLocaleDateString()}
                                    </span>
                                    <span>→</span>
                                    <span>
                                      {task.endDate.toLocaleDateString()}
                                    </span>
                                  </div>
                                  {task.tags && task.tags.length > 0 && (
                                    <div className={styles.tooltipTags}>
                                      {task.tags.map((tag: string, i: number) => (
                                        <span key={i} className={styles.tooltipTag}>
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
                                    className={cn(
                                      styles.taskBar,
                                      styles.taskBarGrouped,
                                      getStatusClass(
                                        group.tasks.every(
                                          (t: TimelineTask) =>
                                            t.status === "completed"
                                        )
                                          ? "completed"
                                          : "in-progress"
                                      )
                                    )}
                                    style={position}
                                  >
                                    <div className={styles.groupedTaskContent}>
                                      <p className={styles.groupedTaskTitle}>
                                        {hasMultipleTasks
                                          ? `${group.tasks.length} tasks`
                                          : group.tasks[0].title}
                                      </p>
                                      <span className={styles.groupedTaskCount}>
                                        {
                                          group.tasks.filter(
                                            (t: TimelineTask) =>
                                              t.status === "completed"
                                          ).length
                                        }
                                        /{group.tasks.length}
                                      </span>
                                    </div>
                                  </div>
                                </TooltipTrigger>
                                <TooltipContent className={cn(styles.tooltipContent, styles.tooltipContentWide)}>
                                  <div className={styles.groupedTooltipInner}>
                                    <p className={styles.groupedTooltipTitle}>
                                      Task Group ({group.tasks.length} tasks)
                                    </p>
                                    <div className={styles.groupedTasksList}>
                                      {group.tasks.map((task: TimelineTask) => (
                                        <div
                                          key={task.id}
                                          className={styles.groupedTaskItem}
                                        >
                                          <p className={styles.groupedTaskItemTitle}>
                                            {task.title}
                                          </p>
                                          <p className={styles.groupedTaskItemDate}>
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
