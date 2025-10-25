/**
 * Task Detail Page
 * Displays detailed task information with tabs
 * 
 * @author @darianrosebrook
 */

"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import DashboardLayout from "@/components/shared/DashboardLayout";
import { Task, AuditLogEntry } from "@/types/tasks";
import AuditTrailViewer from "@/components/tasks/AuditTrailViewer";
import { TaskApiClient } from "@/lib/task-api";
import { Text } from "@/design-system/primitives";
import { StatusBadge } from "@/design-system/compounds";
import { useScrollAnimation } from "@/interactions";
import { 
  Brain, 
  Search, 
  Zap, 
  CheckCircle, 
  Settings, 
  TestTube, 
  Target,
  FileText,
  Pause,
  XCircle
} from "lucide-react";
import styles from "./page.module.scss";

export default function TaskDetailPage() {
  const params = useParams();
  const taskId = params.taskId as string;
  
  const [task, setTask] = useState<Task | null>(null);
  const [auditTrail, setAuditTrail] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"overview" | "audit" | "artifacts">("overview");

  // GSAP animations
  const headerAnimation = useScrollAnimation({ type: 'fade', duration: 0.6, delay: 100 });
  const tabsAnimation = useScrollAnimation({ type: 'slideUp', duration: 0.5, delay: 200 });
  const contentAnimation = useScrollAnimation({ type: 'slideUp', duration: 0.6, delay: 300 });

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
    const iconProps = { size: 18, className: styles.phaseIcon };
    switch (phase) {
      case "planning":
        return <Brain {...iconProps} />;
      case "analysis":
        return <Search {...iconProps} />;
      case "execution":
        return <Zap {...iconProps} />;
      case "validation":
        return <CheckCircle {...iconProps} />;
      case "refinement":
        return <Settings {...iconProps} />;
      case "qa":
        return <TestTube {...iconProps} />;
      case "finalization":
        return <Target {...iconProps} />;
      default:
        return <FileText {...iconProps} />;
    }
  };

  if (loading) {
    return (
      <DashboardLayout>
        <div className={styles.loading}>
          <div className={styles.spinner} aria-hidden="true"></div>
          <Text variant="paragraph-large" color="secondary">
            Loading task details...
          </Text>
        </div>
      </DashboardLayout>
    );
  }

  if (error || !task) {
    return (
      <DashboardLayout>
        <div className={styles.errorContainer} role="alert">
          <XCircle size={48} className={styles.errorIcon} />
          <Text variant="h2" align="center">
            Error Loading Task
          </Text>
          <Text variant="paragraph-large" color="secondary" align="center">
            {error || "Task not found"}
          </Text>
        </div>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <main role="main" aria-label="Task Details" className={styles.container}>
        {/* Task Header */}
        <header ref={headerAnimation.ref} className={styles.header}>
          <div className={styles.headerContent}>
            <div className={styles.titleSection}>
              <Text variant="h1" className={styles.title}>
                {task.title}
              </Text>
              <div className={styles.meta}>
                <StatusBadge 
                  status={task.status as any}
                  size="md"
                />
                <span className={styles.phase}>
                  {getPhaseIcon(task.phase)}
                  <Text variant="paragraph-small" color="secondary">
                    {task.phase}
                  </Text>
                </span>
                <Text variant="paragraph-small" color="secondary">
                  Priority: {task.priority}
                </Text>
              </div>
            </div>
            
            <div className={styles.actions}>
              <button className={styles.actionButton} aria-label="Pause task">
                <Pause size={18} />
                <span>Pause</span>
              </button>
              <button className={styles.actionButton} aria-label="Cancel task">
                <XCircle size={18} />
                <span>Cancel</span>
              </button>
            </div>
          </div>
        </header>

        {/* Tabs Navigation */}
        <nav ref={tabsAnimation.ref} className={styles.tabs} role="tablist" aria-label="Task sections">
          <button
            role="tab"
            aria-selected={activeTab === "overview"}
            aria-controls="overview-panel"
            className={`${styles.tab} ${activeTab === "overview" ? styles.active : ""}`}
            onClick={() => setActiveTab("overview")}
          >
            Overview
          </button>
          <button
            role="tab"
            aria-selected={activeTab === "audit"}
            aria-controls="audit-panel"
            className={`${styles.tab} ${activeTab === "audit" ? styles.active : ""}`}
            onClick={() => setActiveTab("audit")}
          >
            Audit Trail ({auditTrail.length})
          </button>
          <button
            role="tab"
            aria-selected={activeTab === "artifacts"}
            aria-controls="artifacts-panel"
            className={`${styles.tab} ${activeTab === "artifacts" ? styles.active : ""}`}
            onClick={() => setActiveTab("artifacts")}
          >
            Artifacts ({task.artifacts.length})
          </button>
        </nav>

        {/* Tab Content */}
        <div ref={contentAnimation.ref} className={styles.content}>
          {activeTab === "overview" && (
            <div id="overview-panel" role="tabpanel" className={styles.overview}>
              <div className={styles.section}>
                <Text variant="h3" className={styles.sectionTitle}>
                  Task Information
                </Text>
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
                  <Text variant="h3" className={styles.sectionTitle}>
                    Description
                  </Text>
                  <Text variant="paragraph-medium" className={styles.description}>
                    {task.description}
                  </Text>
                </div>
              )}

              {task.context && (
                <div className={styles.section}>
                  <Text variant="h3" className={styles.sectionTitle}>
                    Context
                  </Text>
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
                  <Text variant="h3" className={styles.sectionTitle}>
                    Progress
                  </Text>
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
                  <Text variant="h3" className={styles.sectionTitle}>
                    Quality Report
                  </Text>
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
                  <Text variant="h3" className={styles.sectionTitle}>
                    Error
                  </Text>
                  <div className={styles.errorMessage}>
                    <pre>{task.error_message}</pre>
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === "audit" && (
            <div id="audit-panel" role="tabpanel" className={styles.auditTab}>
              <AuditTrailViewer
                auditTrail={auditTrail}
                taskId={task.id}
                showFullTrail={true}
              />
            </div>
          )}

          {activeTab === "artifacts" && (
            <div id="artifacts-panel" role="tabpanel" className={styles.artifactsTab}>
              <Text variant="h3" className={styles.sectionTitle}>
                Task Artifacts
              </Text>
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
                <Text variant="paragraph-medium" color="secondary" className={styles.noArtifacts}>
                  No artifacts found for this task.
                </Text>
              )}
            </div>
          )}
        </div>
      </main>
    </DashboardLayout>
  );
}
