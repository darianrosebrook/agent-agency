"use client";

// import React from "react";
import { Task } from "@/types/tasks";
import AuditTrailViewer from "./AuditTrailViewer";
import styles from "./TaskCard.module.scss";

interface TaskCardProps {
  task: Task;
  onTaskClick?: (task: Task) => void;
  onTaskUpdate?: (task: Task) => void;
  showAuditTrail?: boolean;
}

export default function TaskCard({
  task,
  onTaskClick: _onTaskClick,
  onTaskUpdate: _onTaskUpdate,
  showAuditTrail: _showAuditTrail = false,
}: TaskCardProps) {
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

  // const _getAuditIcon = (action: string) => {
  //   switch (action.toLowerCase()) {
  //     case "task_created":
  //       return "🆕";
  //     case "task_started":
  //       return "▶️";
  //     case "task_paused":
  //       return "⏸️";
  //     case "task_resumed":
  //       return "▶️";
  //     case "task_completed":
  //       return "✅";
  //     case "task_failed":
  //       return "❌";
  //     case "task_cancelled":
  //       return "🛑";
  //     case "task_state_change":
  //       return "🔄";
  //     case "waiver_created":
  //       return "📋";
  //     case "waiver_approved":
  //       return "✅";
  //     case "waiver_expired":
  //       return "⏰";
  //     case "quality_gate_passed":
  //       return "✅";
  //     case "quality_gate_failed":
  //       return "❌";
  //     case "quality_gate_waived":
  //       return "⚠️";
  //     case "worker_assigned":
  //       return "👷";
  //     case "worker_completed":
  //       return "🏁";
  //     case "model_switched":
  //       return "🔄";
  //     case "iteration_started":
  //       return "🔄";
  //     case "iteration_completed":
  //       return "✅";
  //     default:
  //       return "📝";
  //   }
  // };

  const getPriorityIcon = (priority: Task["priority"]) => {
    switch (priority) {
      case "critical":
        return "🔴";
      case "high":
        return "🟠";
      case "medium":
        return "🟡";
      case "low":
        return "🟢";
      default:
        return "⚪";
    }
  };

  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));

    if (diffMinutes < 1) {
      return "Just now";
    } else if (diffMinutes < 60) {
      return `${diffMinutes}m ago`;
    } else if (diffMinutes < 1440) {
      // 24 hours
      return `${Math.floor(diffMinutes / 60)}h ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  return (
    <div
      className={`${styles.taskCard} ${
        false ? styles.selected : ""
      } ${getStatusColor(task.status)}`}
      onClick={() => {}}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === "Enter" && (() => {})()}
    >
      <div className={styles.cardHeader}>
        <div className={styles.titleSection}>
          <h3 className={styles.title}>{task.title}</h3>
          <div className={styles.meta}>
            <span className={styles.id}>#{task.id.slice(-8)}</span>
            <span className={styles.priority}>
              {getPriorityIcon(task.priority)} {task.priority}
            </span>
          </div>
        </div>

        <div className={styles.statusSection}>
          <div className={`${styles.status} ${getStatusColor(task.status)}`}>
            <span className={styles.statusDot}></span>
            <span className={styles.statusText}>{task.status}</span>
          </div>
        </div>
      </div>

      <div className={styles.cardBody}>
        <div className={styles.phase}>
          <span className={styles.phaseIcon}>{getPhaseIcon(task.phase)}</span>
          <span className={styles.phaseText}>{task.phase}</span>
          <span className={styles.progressText}>
            {task.progress?.percentage || 0}%
          </span>
        </div>

        <div className={styles.progressBar}>
          <div
            className={styles.progressFill}
            style={{ width: `${task.progress?.percentage || 0}%` }}
          />
        </div>

        {task.progress?.current_step && (
          <div className={styles.currentStep}>
            {task.progress.current_step}
            {task.progress.total_steps && (
              <span className={styles.stepCount}>
                ({task.progress.steps_completed}/
                {task.progress.total_steps})
              </span>
            )}
          </div>
        )}

        {task.description && (
          <p className={styles.description}>
            {task.description.length > 100
              ? `${task.description.substring(0, 100)}...`
              : task.description}
          </p>
        )}
      </div>

      <div className={styles.cardFooter}>
        <div className={styles.timing}>
          <div className={styles.timeInfo}>
            <span className={styles.timeLabel}>Elapsed:</span>
            <span className={styles.timeValue}>
              {formatDuration(0)}
            </span>
          </div>

          {false && (
            <div className={styles.timeInfo}>
              <span className={styles.timeLabel}>Remaining:</span>
              <span className={styles.timeValue}>
                {formatDuration(0)}
              </span>
            </div>
          )}
        </div>

        <div className={styles.actions}>
          {task.status === "running" && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                // onPause();
              }}
              className={`${styles.actionButton} ${styles.pause}`}
              title="Pause task"
            >
              ⏸️
            </button>
          )}

          {task.status === "paused" && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                // onResume();
              }}
              className={`${styles.actionButton} ${styles.resume}`}
              title="Resume task"
            >
              ▶️
            </button>
          )}

          {(task.status === "running" || task.status === "paused") && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                // onCancel();
              }}
              className={`${styles.actionButton} ${styles.cancel}`}
              title="Cancel task"
            >
              ❌
            </button>
          )}
        </div>

        <div className={styles.metadata}>
          <span className={styles.createdAt}>
            {formatDate(task.created_at)}
          </span>

          {task.quality_report && (
            <span
              className={`${styles.qualityScore} ${
                task.quality_report.passed ? styles.passed : styles.failed
              }`}
            >
              Q: {task.quality_report.overall_score}%
            </span>
          )}

          {task.artifacts.length > 0 && (
            <span className={styles.artifactCount}>
              📎 {task.artifacts.length}
            </span>
          )}
        </div>
      </div>

      {false && task.context && (
        <div className={styles.details}>
          <div className={styles.context}>
            <h4>Goals</h4>
            <ul>
              {task.context?.goals?.slice(0, 3).map((goal, index) => (
                <li key={index}>{goal}</li>
              ))}
            </ul>
          </div>

          {task.error_message && (
            <div className={styles.error}>
              <h4>Error</h4>
              <p>{task.error_message}</p>
            </div>
          )}

          {/* Audit Trail Section */}
          {false && (
            <div className={styles.auditTrailSection}>
              <AuditTrailViewer
                auditTrail={[]}
                taskId={task.id}
                showFullTrail={false}
              />
            </div>
          )}
        </div>
      )}
    </div>
  );
}
