//! Progress Tracker for Autonomous Execution
//!
//! Tracks execution progress, manages event streaming, and coordinates
//! real-time monitoring of autonomous task execution.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::planning::types::ExecutionEvent;
use agent_agency_observability::metrics::{MetricsCollector, MetricsBackend};

/// Progress tracking configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressTrackerConfig {
    /// Enable progress tracking
    pub enabled: bool,
    /// Maximum events to keep in memory per task
    pub max_events_per_task: usize,
    /// Event retention time (seconds)
    pub event_retention_seconds: u64,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Progress report interval (seconds)
    pub report_interval_seconds: u64,
}

/// Execution progress state
#[derive(Debug, Clone)]
pub struct ExecutionProgress {
    pub task_id: Uuid,
    pub status: ExecutionStatus,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub events: Vec<ExecutionEvent>,
    pub current_phase: Option<String>,
    pub completion_percentage: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Starting,
    Running,
    AwaitingApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Progress tracker for autonomous execution
#[derive(Debug)]
pub struct ProgressTracker {
    config: ProgressTrackerConfig,
    executions: Arc<RwLock<HashMap<Uuid, ExecutionProgress>>>,
    metrics_collector: Option<Arc<dyn MetricsBackend>>,
}

impl ProgressTracker {
    pub fn new(
        config: ProgressTrackerConfig,
        metrics_collector: Option<Arc<dyn MetricsBackend>>,
    ) -> Self {
        Self {
            config,
            executions: Arc::new(RwLock::new(HashMap::new())),
            metrics_collector,
        }
    }

    /// Start tracking a new execution
    pub async fn start_execution(&self, task_id: Uuid, working_spec_id: String) -> Result<(), ProgressTrackerError> {
        if !self.config.enabled {
            return Ok(());
        }

        let progress = ExecutionProgress {
            task_id,
            status: ExecutionStatus::Starting,
            start_time: Utc::now(),
            last_update: Utc::now(),
            events: Vec::new(),
            current_phase: None,
            completion_percentage: 0.0,
            metadata: HashMap::from([
                ("working_spec_id".to_string(), serde_json::json!(working_spec_id)),
            ]),
        };

        let mut executions = self.executions.write().await;
        executions.insert(task_id, progress);

        self.record_metric("execution_started", 1.0).await;

        tracing::info!("Started tracking execution for task: {}", task_id);
        Ok(())
    }

    /// Record an execution event
    pub async fn record_event(&self, event: ExecutionEvent) -> Result<(), ProgressTrackerError> {
        if !self.config.enabled {
            return Ok(());
        }

        let task_id = self.get_task_id_from_event(&event)?;
        let mut executions = self.executions.write().await;

        if let Some(progress) = executions.get_mut(&task_id) {
            // Update progress based on event type
            self.update_progress_from_event(progress, &event);

            // Add event to history
            progress.events.push(event.clone());
            progress.last_update = Utc::now();

            // Trim old events if needed
            if progress.events.len() > self.config.max_events_per_task {
                progress.events.remove(0);
            }

            // Record metrics
            self.record_event_metric(&event).await;

            tracing::debug!("Recorded event for task {}: {:?}", task_id, event);
        } else {
            tracing::warn!("Received event for unknown task: {}", task_id);
        }

        Ok(())
    }

    /// Get current progress for a task
    pub async fn get_progress(&self, task_id: Uuid) -> Result<Option<ExecutionProgress>, ProgressTrackerError> {
        let executions = self.executions.read().await;
        Ok(executions.get(&task_id).cloned())
    }

    /// Get all active executions
    pub async fn get_active_executions(&self) -> Result<Vec<ExecutionProgress>, ProgressTrackerError> {
        let executions = self.executions.read().await;
        Ok(executions.values()
            .filter(|p| matches!(p.status, ExecutionStatus::Starting | ExecutionStatus::Running | ExecutionStatus::Paused))
            .cloned()
            .collect())
    }

    /// Get execution events for a task
    pub async fn get_events(&self, task_id: Uuid, since: Option<DateTime<Utc>>) -> Result<Vec<ExecutionEvent>, ProgressTrackerError> {
        let executions = self.executions.read().await;

        if let Some(progress) = executions.get(&task_id) {
            let mut events = progress.events.clone();

            if let Some(since_time) = since {
                events.retain(|event| self.get_event_timestamp(event) > since_time);
            }

            Ok(events)
        } else {
            Ok(Vec::new())
        }
    }

    /// Complete an execution
    pub async fn complete_execution(&self, task_id: Uuid, success: bool) -> Result<(), ProgressTrackerError> {
        let mut executions = self.executions.write().await;

        if let Some(progress) = executions.get_mut(&task_id) {
            progress.status = if success { ExecutionStatus::Completed } else { ExecutionStatus::Failed };
            progress.completion_percentage = 100.0;
            progress.last_update = Utc::now();

            self.record_metric(if success { "execution_completed" } else { "execution_failed" }, 1.0).await;
        }

        tracing::info!("Completed execution tracking for task: {} (success: {})", task_id, success);
        Ok(())
    }

    /// Cancel an execution
    pub async fn cancel_execution(&self, task_id: Uuid) -> Result<(), ProgressTrackerError> {
        let mut executions = self.executions.write().await;

        if let Some(progress) = executions.get_mut(&task_id) {
            progress.status = ExecutionStatus::Cancelled;
            progress.last_update = Utc::now();

            self.record_metric("execution_cancelled", 1.0).await;
        }

        tracing::info!("Cancelled execution tracking for task: {}", task_id);
        Ok(())
    }

    /// Pause an execution
    pub async fn pause_execution(&self, task_id: Uuid) -> Result<(), ProgressTrackerError> {
        let mut executions = self.executions.write().await;

        if let Some(progress) = executions.get_mut(&task_id) {
            if matches!(progress.status, ExecutionStatus::Running) {
                progress.status = ExecutionStatus::Paused;
                progress.last_update = Utc::now();

                self.record_metric("execution_paused", 1.0).await;
                tracing::info!("Paused execution tracking for task: {}", task_id);
            }
        }

        Ok(())
    }

    /// Resume a paused execution
    pub async fn resume_execution(&self, task_id: Uuid) -> Result<(), ProgressTrackerError> {
        let mut executions = self.executions.write().await;

        if let Some(progress) = executions.get_mut(&task_id) {
            if matches!(progress.status, ExecutionStatus::Paused) {
                progress.status = ExecutionStatus::Running;
                progress.last_update = Utc::now();

                self.record_metric("execution_resumed", 1.0).await;
                tracing::info!("Resumed execution tracking for task: {}", task_id);
            }
        }

        Ok(())
    }

    /// Clean up old executions
    pub async fn cleanup_old_executions(&self) -> Result<usize, ProgressTrackerError> {
        let retention_cutoff = Utc::now() - chrono::Duration::seconds(self.config.event_retention_seconds as i64);
        let mut executions = self.executions.write().await;

        let initial_count = executions.len();
        executions.retain(|_, progress| {
            // Keep if recently updated or still active
            progress.last_update > retention_cutoff ||
            matches!(progress.status, ExecutionStatus::Starting | ExecutionStatus::Running | ExecutionStatus::Paused)
        });

        let removed_count = initial_count - executions.len();

        if removed_count > 0 {
            tracing::info!("Cleaned up {} old executions", removed_count);
        }

        Ok(removed_count)
    }

    /// Update progress state based on event
    fn update_progress_from_event(&self, progress: &mut ExecutionProgress, event: &ExecutionEvent) {
        match event {
            ExecutionEvent::ExecutionStarted { .. } => {
                progress.status = ExecutionStatus::Running;
                progress.completion_percentage = 0.0;
            }
            ExecutionEvent::ExecutionPhaseStarted { phase, .. } => {
                progress.current_phase = Some(phase.clone());
                progress.completion_percentage = (progress.completion_percentage + 10.0).min(90.0);
            }
            ExecutionEvent::ExecutionPhaseCompleted { phase, success, .. } => {
                if *success {
                    progress.completion_percentage = (progress.completion_percentage + 20.0).min(90.0);
                }
                if progress.current_phase.as_ref() == Some(phase) {
                    progress.current_phase = None;
                }
            }
            ExecutionEvent::WorkerAssigned { .. } => {
                progress.completion_percentage = 10.0;
            }
            ExecutionEvent::ArtifactProduced { .. } => {
                progress.completion_percentage = (progress.completion_percentage + 5.0).min(80.0);
            }
            ExecutionEvent::QualityCheckCompleted { passed, .. } => {
                if *passed {
                    progress.completion_percentage = (progress.completion_percentage + 10.0).min(95.0);
                }
            }
            ExecutionEvent::ExecutionCompleted { .. } => {
                progress.status = ExecutionStatus::Completed;
                progress.completion_percentage = 100.0;
            }
            ExecutionEvent::ExecutionFailed { .. } => {
                progress.status = ExecutionStatus::Failed;
                progress.completion_percentage = 100.0;
            }
            _ => {} // Other events don't change overall status
        }
    }

    /// Extract task ID from execution event
    fn get_task_id_from_event(&self, event: &ExecutionEvent) -> Result<Uuid, ProgressTrackerError> {
        match event {
            ExecutionEvent::ExecutionStarted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::WorkerAssignmentStarted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::WorkerAssigned { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionPhaseStarted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionPhaseCompleted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ArtifactProduced { task_id, .. } => Ok(*task_id),
            ExecutionEvent::QualityCheckCompleted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionCompleted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionFailed { task_id, .. } => Ok(*task_id),
        }
    }

    /// Get timestamp from execution event
    fn get_event_timestamp(&self, event: &ExecutionEvent) -> DateTime<Utc> {
        match event {
            ExecutionEvent::ExecutionStarted { timestamp, .. } => *timestamp,
            ExecutionEvent::WorkerAssignmentStarted { timestamp, .. } => *timestamp,
            ExecutionEvent::WorkerAssigned { timestamp, .. } => *timestamp,
            ExecutionEvent::ExecutionPhaseStarted { timestamp, .. } => *timestamp,
            ExecutionEvent::ExecutionPhaseCompleted { timestamp, .. } => *timestamp,
            ExecutionEvent::ArtifactProduced { timestamp, .. } => *timestamp,
            ExecutionEvent::QualityCheckCompleted { timestamp, .. } => *timestamp,
            ExecutionEvent::ExecutionCompleted { timestamp, .. } => *timestamp,
            ExecutionEvent::ExecutionFailed { timestamp, .. } => *timestamp,
        }
    }

    /// Record metrics for events
    async fn record_event_metric(&self, event: &ExecutionEvent) {
        if !self.config.enable_metrics {
            return;
        }

        let metric_name = match event {
            ExecutionEvent::ExecutionStarted { .. } => "execution_started",
            ExecutionEvent::WorkerAssigned { .. } => "worker_assigned",
            ExecutionEvent::ExecutionPhaseStarted { .. } => "phase_started",
            ExecutionEvent::ExecutionPhaseCompleted { success, .. } => {
                if *success { "phase_completed_success" } else { "phase_completed_failure" }
            }
            ExecutionEvent::ArtifactProduced { .. } => "artifact_produced",
            ExecutionEvent::QualityCheckCompleted { passed, .. } => {
                if *passed { "quality_check_passed" } else { "quality_check_failed" }
            }
            ExecutionEvent::ExecutionCompleted { success, .. } => {
                if *success { "execution_success" } else { "execution_failure" }
            }
            ExecutionEvent::ExecutionFailed { .. } => "execution_failed",
            _ => return,
        };

        self.record_metric(metric_name, 1.0).await;
    }

    /// Record a metric
    async fn record_metric(&self, name: &str, value: f64) {
        if let Some(metrics) = &self.metrics_collector {
            let _ = metrics.record_counter(name, value as u64).await;
        }
    }
}

pub type Result<T, E = ProgressTrackerError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum ProgressTrackerError {
    #[error("Execution not found: {0}")]
    ExecutionNotFound(Uuid),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    #[error("Metrics collection failed: {0}")]
    MetricsError(String),

    #[error("Storage operation failed: {0}")]
    StorageError(String),
}
