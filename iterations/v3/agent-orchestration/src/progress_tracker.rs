//! Progress tracking for autonomous task execution
//!
//! Provides real-time progress tracking with persistence and event emission
//! for monitoring task execution across the agent orchestration system.
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

pub mod credit_assignment;
pub mod trajectory_analyzer;
pub mod turn_level;

pub use turn_level::{
    AgentAction, CreditAssignment, ProgressUpdate, TaskOutcome, TurnLevelProgressTracker,
    TurnLevelTracker, TurnOutcome, TurnProgress, TurnTrajectory,
};

pub use trajectory_analyzer::{
    ActionSequenceAnalysis, DetectedPattern, PerformanceMetrics, QualityTrend, TrajectoryAnalyzer,
    TrajectoryInsights, TrajectoryPattern, TrendDirection,
};

pub use credit_assignment::{AdvancedCreditAssigner, TdLearningConfig, ValueFunction};

/// Progress tracking trait for task execution
#[async_trait::async_trait]
pub trait ProgressTracker: Send + Sync {
    /// Update progress for a specific task
    async fn update_progress(
        &self,
        task_id: Uuid,
        progress: ExecutionProgress,
    ) -> Result<(), ProgressError>;

    /// Get current progress for a task
    async fn get_progress(&self, task_id: Uuid)
        -> Result<Option<ExecutionProgress>, ProgressError>;

    /// Get all active task progress
    async fn get_all_progress(&self) -> Result<HashMap<Uuid, ExecutionProgress>, ProgressError>;

    /// Persist progress to storage
    async fn persist_progress(&self, task_id: Uuid) -> Result<(), ProgressError>;

    /// Load progress from storage
    async fn load_progress(
        &self,
        task_id: Uuid,
    ) -> Result<Option<ExecutionProgress>, ProgressError>;

    /// Subscribe to progress updates
    async fn subscribe_to_updates(
        &self,
        task_id: Uuid,
    ) -> Result<mpsc::Receiver<ExecutionProgress>, ProgressError>;

    /// Emit progress event
    async fn emit_event(&self, event: ExecutionEvent) -> Result<(), ProgressError>;
}

/// Execution progress information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionProgress {
    /// Task ID
    #[schemars(with = "String")]
    pub task_id: Uuid,
    /// Current status
    pub status: ExecutionStatus,
    /// Progress percentage (0.0-100.0)
    pub percentage: f64,
    /// Current phase/step
    pub current_phase: String,
    /// Total phases
    pub total_phases: usize,
    /// Current phase index
    pub current_phase_index: usize,
    /// Start time
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
    /// Last updated time
    #[schemars(with = "String")]
    pub last_updated: DateTime<Utc>,
    /// Estimated completion time
    #[schemars(with = "Option<String>")]
    pub estimated_completion: Option<DateTime<Utc>>,
    /// Messages and logs
    pub messages: Vec<ProgressMessage>,
    /// Error information (if any)
    pub error: Option<ProgressError>,
    /// Performance metrics
    pub metrics: ProgressMetrics,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum ExecutionStatus {
    /// Task is pending
    Pending,
    /// Task is running
    Running,
    /// Task is paused
    Paused,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task is retrying
    Retrying,
}

/// Progress message
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProgressMessage {
    /// Message timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Message level
    pub level: MessageLevel,
    /// Message content
    pub content: String,
    /// Additional context
    pub context: Option<HashMap<String, serde_json::Value>>,
}

/// Message level
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MessageLevel {
    /// Debug information
    Debug,
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

/// Progress metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProgressMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Network I/O in bytes
    pub network_io: u64,
    /// Disk I/O in bytes
    pub disk_io: u64,
    /// Processing rate (items per second)
    pub processing_rate: f64,
    /// Error count
    pub error_count: u64,
    /// Retry count
    pub retry_count: u64,
}

/// Progress error
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProgressError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Additional context
    pub context: Option<HashMap<String, serde_json::Value>>,
}

impl std::fmt::Display for ProgressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProgressError[{}]: {}", self.code, self.message)
    }
}

/// Execution event for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEvent {
    /// Event ID
    #[schemars(with = "String")]
    pub event_id: Uuid,
    /// Task ID
    #[schemars(with = "String")]
    pub task_id: Uuid,
    /// Event type
    pub event_type: EventType,
    /// Event timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Event data
    pub data: HashMap<String, serde_json::Value>,
}

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EventType {
    /// Task started
    TaskStarted,
    /// Task completed
    TaskCompleted,
    /// Task failed
    TaskFailed,
    /// Progress updated
    ProgressUpdated,
    /// Phase changed
    PhaseChanged,
    /// Error occurred
    ErrorOccurred,
    /// Task paused
    TaskPaused,
    /// Task resumed
    TaskResumed,
    /// Task cancelled
    TaskCancelled,
}

/// Real-time progress tracker implementation

#[derive(Debug)]
pub struct RealTimeProgressTracker {
    /// In-memory progress storage
    progress_store: Arc<RwLock<HashMap<Uuid, ExecutionProgress>>>,
    /// Event subscribers
    subscribers: Arc<RwLock<HashMap<Uuid, Vec<mpsc::Sender<ExecutionProgress>>>>>,
    /// Event emitter
    event_emitter: Arc<RwLock<Vec<mpsc::Sender<ExecutionEvent>>>>,
    /// Persistence backend
    persistence_backend: Option<Arc<dyn ProgressPersistence + Send + Sync>>,
}

/// Progress persistence trait
#[async_trait::async_trait]
pub trait ProgressPersistence: Send + Sync + std::fmt::Debug {
    /// Save progress to persistent storage
    async fn save_progress(&self, progress: &ExecutionProgress) -> Result<(), ProgressError>;

    /// Load progress from persistent storage
    async fn load_progress(
        &self,
        task_id: Uuid,
    ) -> Result<Option<ExecutionProgress>, ProgressError>;

    /// Delete progress from persistent storage
    async fn delete_progress(&self, task_id: Uuid) -> Result<(), ProgressError>;
}

impl RealTimeProgressTracker {
    /// Create a new real-time progress tracker
    pub fn new(persistence_backend: Option<Arc<dyn ProgressPersistence + Send + Sync>>) -> Self {
        Self {
            progress_store: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_emitter: Arc::new(RwLock::new(Vec::new())),
            persistence_backend,
        }
    }

    /// Add event emitter
    pub async fn add_event_emitter(&self, emitter: mpsc::Sender<ExecutionEvent>) {
        let mut emitters = self.event_emitter.write().await;
        emitters.push(emitter);
    }

    /// Remove event emitter
    pub async fn remove_event_emitter(&self, emitter: mpsc::Sender<ExecutionEvent>) {
        let mut emitters = self.event_emitter.write().await;
        emitters.retain(|e| !e.same_channel(&emitter));
    }

    /// Calculate estimated completion time
    fn calculate_estimated_completion(
        &self,
        progress: &ExecutionProgress,
    ) -> Option<DateTime<Utc>> {
        if progress.percentage <= 0.0 || progress.percentage >= 100.0 {
            return None;
        }

        let elapsed = progress
            .last_updated
            .signed_duration_since(progress.started_at);
        let elapsed_ms = elapsed.num_milliseconds() as f64;
        let total_estimated_ms = elapsed_ms * 100.0 / progress.percentage;
        let remaining_ms = total_estimated_ms - elapsed_ms;

        Some(progress.last_updated + chrono::Duration::milliseconds(remaining_ms as i64))
    }
}

#[async_trait::async_trait]
impl ProgressTracker for RealTimeProgressTracker {
    async fn update_progress(
        &self,
        task_id: Uuid,
        mut progress: ExecutionProgress,
    ) -> Result<(), ProgressError> {
        // Update timestamp
        progress.last_updated = Utc::now();

        // Calculate estimated completion
        progress.estimated_completion = self.calculate_estimated_completion(&progress);

        // Store in memory
        {
            let mut store = self.progress_store.write().await;
            store.insert(task_id, progress.clone());
        }

        // Persist to storage if backend is available
        if let Some(backend) = &self.persistence_backend {
            if let Err(e) = backend.save_progress(&progress).await {
                return Err(e);
            }
        }

        // Notify subscribers
        {
            let subscribers = self.subscribers.read().await;
            if let Some(task_subscribers) = subscribers.get(&task_id) {
                for subscriber in task_subscribers {
                    let _ = subscriber.send(progress.clone()).await;
                }
            }
        }

        // Emit event
        let event = ExecutionEvent {
            event_id: Uuid::new_v4(),
            task_id,
            event_type: EventType::ProgressUpdated,
            timestamp: Utc::now(),
            data: HashMap::new(),
        };
        self.emit_event(event).await?;

        Ok(())
    }

    async fn get_progress(
        &self,
        task_id: Uuid,
    ) -> Result<Option<ExecutionProgress>, ProgressError> {
        let store = self.progress_store.read().await;
        Ok(store.get(&task_id).cloned())
    }

    async fn get_all_progress(&self) -> Result<HashMap<Uuid, ExecutionProgress>, ProgressError> {
        let store = self.progress_store.read().await;
        Ok(store.clone())
    }

    async fn persist_progress(&self, task_id: Uuid) -> Result<(), ProgressError> {
        if let Some(backend) = &self.persistence_backend {
            let store = self.progress_store.read().await;
            if let Some(progress) = store.get(&task_id) {
                backend.save_progress(progress).await?;
            }
        }
        Ok(())
    }

    async fn load_progress(
        &self,
        task_id: Uuid,
    ) -> Result<Option<ExecutionProgress>, ProgressError> {
        if let Some(backend) = &self.persistence_backend {
            let progress = backend.load_progress(task_id).await?;
            if let Some(progress) = &progress {
                let mut store = self.progress_store.write().await;
                store.insert(task_id, progress.clone());
            }
            Ok(progress)
        } else {
            Ok(None)
        }
    }

    async fn subscribe_to_updates(
        &self,
        task_id: Uuid,
    ) -> Result<mpsc::Receiver<ExecutionProgress>, ProgressError> {
        let (tx, rx) = mpsc::channel(100);

        {
            let mut subscribers = self.subscribers.write().await;
            subscribers.entry(task_id).or_insert_with(Vec::new).push(tx);
        }

        Ok(rx)
    }

    async fn emit_event(&self, event: ExecutionEvent) -> Result<(), ProgressError> {
        let emitters = self.event_emitter.read().await;
        for emitter in emitters.iter() {
            let _ = emitter.send(event.clone()).await;
        }
        Ok(())
    }
}

impl Default for ExecutionProgress {
    fn default() -> Self {
        Self {
            task_id: Uuid::new_v4(),
            status: ExecutionStatus::Pending,
            percentage: 0.0,
            current_phase: "Initializing".to_string(),
            total_phases: 1,
            current_phase_index: 0,
            started_at: Utc::now(),
            last_updated: Utc::now(),
            estimated_completion: None,
            messages: Vec::new(),
            error: None,
            metrics: ProgressMetrics::default(),
        }
    }
}

impl Default for ProgressMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            network_io: 0,
            disk_io: 0,
            processing_rate: 0.0,
            error_count: 0,
            retry_count: 0,
        }
    }
}

impl Default for ProgressMessage {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            level: MessageLevel::Info,
            content: String::new(),
            context: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_tracker_basic_operations() {
        let tracker = RealTimeProgressTracker::new(None);
        let task_id = Uuid::new_v4();

        let mut progress = ExecutionProgress::default();
        progress.task_id = task_id;
        progress.status = ExecutionStatus::Running;
        progress.percentage = 50.0;
        progress.current_phase = "Processing".to_string();

        // Update progress
        assert!(tracker
            .update_progress(task_id, progress.clone())
            .await
            .is_ok());

        // Get progress
        let retrieved = tracker.get_progress(task_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().percentage, 50.0);

        // Get all progress
        let all_progress = tracker.get_all_progress().await.unwrap();
        assert_eq!(all_progress.len(), 1);
        assert!(all_progress.contains_key(&task_id));
    }

    #[tokio::test]
    async fn test_progress_subscription() {
        let tracker = RealTimeProgressTracker::new(None);
        let task_id = Uuid::new_v4();

        // Subscribe to updates
        let mut rx = tracker.subscribe_to_updates(task_id).await.unwrap();

        // Update progress
        let mut progress = ExecutionProgress::default();
        progress.task_id = task_id;
        progress.status = ExecutionStatus::Running;
        progress.percentage = 25.0;

        tracker.update_progress(task_id, progress).await.unwrap();

        // Receive update
        let received = rx.recv().await.unwrap();
        assert_eq!(received.percentage, 25.0);
    }
}
