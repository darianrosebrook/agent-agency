"use client";

import React, { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Header from "@/components/shared/Header";
import Navigation from "@/components/shared/Navigation";
import { Task, AuditLogEntry } from "@/types/tasks";
import AuditTrailViewer from "@/components/tasks/AuditTrailViewer";
import { TaskApiClient } from "@/lib/task-api";
import styles from "./page.module.scss";

export default function TaskDetailPage() {
  const params = useParams();
  const taskId = params.taskId as string;
  
  const [task, setTask] = useState<Task | null>(null);
  const [auditTrail, setAuditTrail] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"overview" | "audit" | "artifacts">("overview");

  const taskApi = new TaskApiClient();

  useEffect(() => {
    const fetchTaskDetails = async () => {
      try {
        setLoading(true);
        setError(null);

        // Fetch task details
        const taskData = await taskApi.getTask(taskId);
        setTask(taskData);

        // Fetch audit trail
        const auditData = await taskApi.getTaskAuditTrail(taskId);
        setAuditTrail(auditData);

      } catch (err) {
        console.error("Failed to fetch task details:", err);
        setError(err instanceof Error ? err.message : "Failed to load task details");
      } finally {
        setLoading(false);
      }
    };

    if (taskId) {
      fetchTaskDetails();
    }
  }, [taskId]);

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString("en-US", {
      year: "numeric",
      month: "long",
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

  if (loading) {
    return (
      <div className={styles.page}>
        <Header />
        <Navigation />
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading task details...</p>
        </div>
      </div>
    );
  }

  if (error || !task) {
    return (
      <div className={styles.page}>
        <Header />
        <Navigation />
        <div className={styles.error}>
          <h2>Error Loading Task</h2>
          <p>{error || "Task not found"}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <Header />
      <Navigation />
      
      <div className={styles.container}>
        <div className={styles.header}>
          <div className={styles.titleSection}>
            <h1 className={styles.title}>{task.title}</h1>
            <div className={styles.meta}>
              <span className={`${styles.status} ${getStatusColor(task.status)}`}>
                {task.status}
              </span>
              <span className={styles.phase}>
                {getPhaseIcon(task.phase)} {task.phase}
              </span>
              <span className={styles.priority}>
                Priority: {task.priority}
              </span>
            </div>
          </div>
          
          <div className={styles.actions}>
            <button className={styles.actionButton}>
              Pause
            </button>
            <button className={styles.actionButton}>
              Cancel
            </button>
          </div>
        </div>

        <div className={styles.tabs}>
          <button
            className={`${styles.tab} ${activeTab === "overview" ? styles.active : ""}`}
            onClick={() => setActiveTab("overview")}
          >
            Overview
          </button>
          <button
            className={`${styles.tab} ${activeTab === "audit" ? styles.active : ""}`}
            onClick={() => setActiveTab("audit")}
          >
            Audit Trail ({auditTrail.length})
          </button>
          <button
            className={`${styles.tab} ${activeTab === "artifacts" ? styles.active : ""}`}
            onClick={() => setActiveTab("artifacts")}
          >
            Artifacts ({task.artifacts.length})
          </button>
        </div>

        <div className={styles.content}>
          {activeTab === "overview" && (
            <div className={styles.overview}>
              <div className={styles.section}>
                <h3>Task Information</h3>
                <div className={styles.infoGrid}>
                  <div className={styles.infoItem}>
                    <label>Task ID</label>
                    <span className={styles.mono}>{task.id}</span>
                  </div>
                  <div className={styles.infoItem}>
                    <label>Working Spec ID</label>
                    <span className={styles.mono}>{task.working_spec_id}</span>
                  </div>
                  <div className={styles.infoItem}>
                    <label>Created</label>
                    <span>{formatDate(task.created_at)}</span>
                  </div>
                  <div className={styles.infoItem}>
                    <label>Updated</label>
                    <span>{formatDate(task.updated_at)}</span>
                  </div>
                  {task.started_at && (
                    <div className={styles.infoItem}>
                      <label>Started</label>
                      <span>{formatDate(task.started_at)}</span>
                    </div>
                  )}
                  {task.completed_at && (
                    <div className={styles.infoItem}>
                      <label>Completed</label>
                      <span>{formatDate(task.completed_at)}</span>
                    </div>
                  )}
                  <div className={styles.infoItem}>
                    <label>Retry Count</label>
                    <span>{task.retry_count} / {task.max_retries}</span>
                  </div>
                </div>
              </div>

              {task.description && (
                <div className={styles.section}>
                  <h3>Description</h3>
                  <p className={styles.description}>{task.description}</p>
                </div>
              )}

              {task.context && (
                <div className={styles.section}>
                  <h3>Context</h3>
                  <div className={styles.context}>
                    <div className={styles.goals}>
                      <h4>Goals</h4>
                      <ul>
                        {task.context.goals.map((goal, index) => (
                          <li key={index}>{goal}</li>
                        ))}
                      </ul>
                    </div>
                    {task.context.constraints.length > 0 && (
                      <div className={styles.constraints}>
                        <h4>Constraints</h4>
                        <ul>
                          {task.context.constraints.map((constraint, index) => (
                            <li key={index}>{constraint}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                </div>
              )}

              {task.progress && (
                <div className={styles.section}>
                  <h3>Progress</h3>
                  <div className={styles.progress}>
                    <div className={styles.progressBar}>
                      <div 
                        className={styles.progressFill}
                        style={{ width: `${task.progress.percentage}%` }}
                      ></div>
                    </div>
                    <span className={styles.progressText}>
                      {task.progress.percentage}% - {task.progress.current_step}
                    </span>
                  </div>
                </div>
              )}

              {task.quality_report && (
                <div className={styles.section}>
                  <h3>Quality Report</h3>
                  <div className={styles.qualityReport}>
                    <div className={styles.qualityScore}>
                      <span className={styles.score}>
                        {task.quality_report.overall_score}%
                      </span>
                      <span className={styles.status}>
                        {task.quality_report.passed ? "PASSED" : "FAILED"}
                      </span>
                    </div>
                    {task.quality_report.details && (
                      <div className={styles.qualityDetails}>
                        {Object.entries(task.quality_report.details).map(([key, value]) => (
                          <div key={key} className={styles.qualityItem}>
                            <span className={styles.qualityLabel}>{key}</span>
                            <span className={styles.qualityValue}>{value}</span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {task.error_message && (
                <div className={styles.section}>
                  <h3>Error</h3>
                  <div className={styles.errorMessage}>
                    <pre>{task.error_message}</pre>
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === "audit" && (
            <div className={styles.auditTab}>
              <AuditTrailViewer
                auditTrail={auditTrail}
                taskId={task.id}
                showFullTrail={true}
              />
            </div>
          )}

          {activeTab === "artifacts" && (
            <div className={styles.artifactsTab}>
              <h3>Task Artifacts</h3>
              {task.artifacts.length > 0 ? (
                <div className={styles.artifactsList}>
                  {task.artifacts.map((artifact, index) => (
                    <div key={index} className={styles.artifact}>
                      <div className={styles.artifactHeader}>
                        <h4>{artifact.name}</h4>
                        <span className={styles.artifactType}>{artifact.type}</span>
                      </div>
                      {artifact.description && (
                        <p className={styles.artifactDescription}>
                          {artifact.description}
                        </p>
                      )}
                      <div className={styles.artifactMeta}>
                        <span>Size: {artifact.size} bytes</span>
                        <span>Created: {formatDate(artifact.created_at)}</span>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className={styles.noArtifacts}>No artifacts found for this task.</p>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
