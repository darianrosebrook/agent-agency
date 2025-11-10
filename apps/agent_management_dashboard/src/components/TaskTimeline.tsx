"use client";

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
import type { Task } from "../lib/schemas/chat";
import { cn } from "./primitives/utils";
import styles from "./TaskTimeline.module.scss";

interface TaskTimelineProps {
  tasks: Task[];
}

function getRelativeTime(date: Date): string {
  const now = new Date();
  const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

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
  const [expandedTasks, setExpandedTasks] = useState<Set<string>>(new Set());

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
    // Determine icon based on task name/type
    if (
      task.name.toLowerCase().includes("search") ||
      task.name.toLowerCase().includes("analyzing")
    ) {
      return <Search className={styles.icon} />;
    } else if (
      task.name.toLowerCase().includes("think") ||
      task.name.toLowerCase().includes("reasoning")
    ) {
      return <Lightbulb className={styles.icon} />;
    } else if (
      task.name.toLowerCase().includes("generating") ||
      task.name.toLowerCase().includes("creating")
    ) {
      return <Code className={styles.icon} />;
    } else if (
      task.name.toLowerCase().includes("format") ||
      task.name.toLowerCase().includes("output")
    ) {
      return <FileText className={styles.icon} />;
    } else {
      return <Circle className={styles.icon} />;
    }
  };

  const getStatusIcon = (status: Task["status"]) => {
    switch (status) {
      case "completed":
        return <CheckCircle2 className={cn(styles.statusIcon, styles.statusIconCompleted)} />;
      case "in-progress":
        return <Loader2 className={cn(styles.statusIcon, styles.statusIconInProgress)} />;
      case "failed":
        return <Circle className={cn(styles.statusIcon, styles.statusIconFailed)} />;
      default:
        return <Circle className={cn(styles.statusIcon, styles.statusIconDefault)} />;
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
    <div className={styles.taskTimeline}>
      {tasks.map((task, index) => {
        const isExpanded = expandedTasks.has(task.id);
        const canExpand =
          isThinkingTask(task) && task.status === "completed" && task.result;
        const isLast = index === tasks.length - 1;

        return (
          <div key={task.id} className={styles.taskTimelineItem}>
            {/* Vertical connecting line */}
            {!isLast && (
              <div className={styles.connectingLine} />
            )}

            {/* Task row */}
            <div
              className={cn(
                styles.taskRow,
                canExpand && styles.taskRowExpandable
              )}
              onClick={() => canExpand && toggleTask(task.id)}
            >
              {/* Icon container */}
              <div className={styles.iconContainer}>
                <div
                  className={cn(
                    task.status === "completed"
                      ? styles.iconContainerCompleted
                      : task.status === "in-progress"
                      ? styles.iconContainerInProgress
                      : styles.iconContainerDefault
                  )}
                >
                  {getTaskIcon(task)}
                </div>

                {/* Status indicator badge */}
                <div className={styles.statusBadge}>
                  {getStatusIcon(task.status)}
                </div>
              </div>

              {/* Task content */}
              <div className={styles.taskContent}>
                <div className={styles.taskContentRow}>
                  <span
                    className={cn(
                      styles.taskName,
                      task.status === "completed"
                        ? styles.taskNameCompleted
                        : task.status === "in-progress"
                        ? styles.taskNameInProgress
                        : styles.taskNameDefault
                    )}
                  >
                    {task.name}
                  </span>
                  {task.status === "in-progress" && (
                    <span className={styles.taskSeparator}>•</span>
                  )}
                  <span className={styles.taskTime}>
                    {getRelativeTime(task.timestamp)}
                  </span>
                  {canExpand && (
                    <div className={styles.expandIconContainer}>
                      {isExpanded ? (
                        <ChevronDown className={styles.expandIcon} />
                      ) : (
                        <ChevronRight className={styles.expandIcon} />
                      )}
                    </div>
                  )}
                </div>

                {/* Expandable thought process */}
                {canExpand && isExpanded && task.result && (
                  <div className={styles.expandedContent}>
                    <p className={styles.expandedContentText}>
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
