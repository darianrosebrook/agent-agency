"use client";

import React from "react";
import Link from "next/link";
import { Task } from "@/types/tasks";
import styles from "./TaskList.module.scss";

interface TaskListProps {
  tasks: Task[];
  onTaskAction: (taskId: string, action: string) => void;
  loading?: boolean;
}

export default function TaskList({ tasks, onTaskAction, loading = false }: TaskListProps) {
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getStatusColor = (status: Task["status"]) => {
    switch (status) {
      case "completed":
        return styles.success;
      case "running":
        return styles.primary;
      case "pending":
        return styles.warning;
      case "paused":
        return styles.secondary;
      case "failed":
        return styles.error;
      case "cancelled":
        return styles.neutral;
      default:
        return styles.neutral;
    }
  };

  const getPhaseIcon = (phase: Task["phase"]) => {
    switch (phase) {
      case "planning":
        return "🧠";
      case "analysis":
        return "🔍";
      case "execution":
        return "⚡";
      case "validation":
        return "✅";
      case "refinement":
        return "🔧";
      case "qa":
        return "🧪";
      case "finalization":
        return "🎯";
      default:
        return "📋";
    }
  };

  const getPriorityColor = (priority: Task["priority"]) => {
    switch (priority) {
      case "critical":
        return styles.critical;
      case "high":
        return styles.high;
      case "medium":
        return styles.medium;
      case "low":
        return styles.low;
      default:
        return styles.medium;
    }
  };

  const canPause = (status: Task["status"]) => status === "running";
  const canResume = (status: Task["status"]) => status === "paused";
  const canCancel = (status: Task["status"]) => ["running", "pending", "paused"].includes(status);
  const canRetry = (status: Task["status"]) => status === "failed";

  if (loading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <p>Loading tasks...</p>
      </div>
    );
  }

  if (tasks.length === 0) {
    return (
      <div className={styles.emptyState}>
        <div className={styles.emptyIcon}>📋</div>
        <h3>No Tasks Found</h3>
        <p>No tasks match your current filters.</p>
      </div>
    );
  }

  return (
    <div className={styles.taskList}>
      <div className={styles.header}>
        <div className={styles.headerCell}>Task</div>
        <div className={styles.headerCell}>Status</div>
        <div className={styles.headerCell}>Phase</div>
        <div className={styles.headerCell}>Priority</div>
        <div className={styles.headerCell}>Created</div>
        <div className={styles.headerCell}>Actions</div>
      </div>

      <div className={styles.body}>
        {tasks.map((task) => (
          <div key={task.id} className={styles.taskRow}>
            <div className={styles.taskCell}>
              <div className={styles.taskInfo}>
                <Link href={`/tasks/${task.id}`} className={styles.taskTitle}>
                  {task.title}
                </Link>
                {task.description && (
                  <p className={styles.taskDescription}>
                    {task.description.length > 100
                      ? `${task.description.substring(0, 100)}...`
                      : task.description}
                  </p>
                )}
                <div className={styles.taskMeta}>
                  <span className={styles.taskId}>ID: {task.id}</span>
                  {task.retry_count > 0 && (
                    <span className={styles.retryCount}>
                      Retries: {task.retry_count}/{task.max_retries}
                    </span>
                  )}
                </div>
              </div>
            </div>

            <div className={styles.statusCell}>
              <span className={`${styles.status} ${getStatusColor(task.status)}`}>
                {task.status}
              </span>
            </div>

            <div className={styles.phaseCell}>
              <span className={styles.phase}>
                {getPhaseIcon(task.phase)} {task.phase}
              </span>
            </div>

            <div className={styles.priorityCell}>
              <span className={`${styles.priority} ${getPriorityColor(task.priority)}`}>
                {task.priority}
              </span>
            </div>

            <div className={styles.dateCell}>
              <span className={styles.date}>
                {formatDate(task.created_at)}
              </span>
            </div>

            <div className={styles.actionsCell}>
              <div className={styles.actions}>
                {canPause(task.status) && (
                  <button
                    onClick={() => onTaskAction(task.id, "pause")}
                    className={styles.actionButton}
                    title="Pause task"
                  >
                    ⏸️
                  </button>
                )}
                {canResume(task.status) && (
                  <button
                    onClick={() => onTaskAction(task.id, "resume")}
                    className={styles.actionButton}
                    title="Resume task"
                  >
                    ▶️
                  </button>
                )}
                {canCancel(task.status) && (
                  <button
                    onClick={() => onTaskAction(task.id, "cancel")}
                    className={styles.actionButton}
                    title="Cancel task"
                  >
                    🛑
                  </button>
                )}
                {canRetry(task.status) && (
                  <button
                    onClick={() => onTaskAction(task.id, "retry")}
                    className={styles.actionButton}
                    title="Retry task"
                  >
                    🔄
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}