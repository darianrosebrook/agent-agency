//! Bridge between parallel workers and orchestration monitoring/tracking
//!
//! This module provides integration points that allow parallel workers
//! to publish progress updates to the orchestration layer's monitoring system.

use crate::error::{ParallelError, ParallelResult};
use crate::types::{TaskId, WorkerId, Progress};
use async_trait::async_trait;
use std::sync::Arc;

// Stub types for orchestration monitoring integration (replace with actual imports when available)
#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    pub task_id: TaskId,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ExecutionProgress {
    pub task_id: TaskId,
    pub status: ExecutionStatus,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub events: Vec<ExecutionEvent>,
    pub current_phase: Option<String>,
    pub completion_percentage: f32,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Handle for accessing orchestration monitoring functionality
#[async_trait]
pub trait OrchestrationMonitoringHandle: Send + Sync {
    /// Publish progress update for a task
    async fn update_task_progress(
        &self,
        task_id: &TaskId,
        progress: &ExecutionProgress,
    ) -> Result<(), OrchestrationMonitoringError>;

    /// Publish worker-specific progress update
    async fn update_worker_progress(
        &self,
        task_id: &TaskId,
        worker_id: &WorkerId,
        progress: &Progress,
    ) -> Result<(), OrchestrationMonitoringError>;

    /// Publish execution event
    async fn publish_event(
        &self,
        event: ExecutionEvent,
    ) -> Result<(), OrchestrationMonitoringError>;

    /// Get current task progress
    async fn get_task_progress(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<ExecutionProgress>, OrchestrationMonitoringError>;
}

/// Error type for orchestration monitoring bridge operations
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationMonitoringError {
    #[error("Progress update failed: {message}")]
    ProgressUpdateFailed { message: String },

    #[error("Event publishing failed: {message}")]
    EventPublishFailed { message: String },

    #[error("Progress retrieval failed: {message}")]
    ProgressRetrievalFailed { message: String },

    #[error("Invalid progress data: {message}")]
    InvalidProgressData { message: String },
}

/// Bridge to orchestration monitoring system
#[derive(Clone)]
pub struct OrchestrationMonitoringBridge {
    /// Handle to orchestration monitoring system
    monitoring_handle: Arc<dyn OrchestrationMonitoringHandle>,
}

impl OrchestrationMonitoringBridge {
    /// Create a new monitoring bridge
    pub fn new(handle: Arc<dyn OrchestrationMonitoringHandle>) -> Self {
        Self {
            monitoring_handle: handle,
        }
    }

    /// Update overall task progress in orchestration monitoring
    pub async fn update_task_progress(
        &self,
        task_id: &TaskId,
        status: ExecutionStatus,
        completion_percentage: f32,
        current_phase: Option<String>,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> ParallelResult<()> {
        let progress = ExecutionProgress {
            task_id: task_id.clone(),
            status,
            start_time: chrono::Utc::now(), // TODO: Track actual start time
            last_update: chrono::Utc::now(),
            events: vec![], // TODO: Collect events
            current_phase,
            completion_percentage,
            metadata,
        };

        self.monitoring_handle
            .update_task_progress(task_id, &progress)
            .await
            .map_err(|e| ParallelError::ProgressTracking {
                message: format!("Failed to update task progress: {}", e),
                source: None,
            })
    }

    /// Update individual worker progress in orchestration monitoring
    pub async fn update_worker_progress(
        &self,
        task_id: &TaskId,
        worker_id: &WorkerId,
        progress: &Progress,
    ) -> ParallelResult<()> {
        self.monitoring_handle
            .update_worker_progress(task_id, worker_id, progress)
            .await
            .map_err(|e| ParallelError::ProgressTracking {
                message: format!("Failed to update worker progress: {}", e),
                source: None,
            })
    }

    /// Publish execution event to orchestration monitoring
    pub async fn publish_event(
        &self,
        task_id: TaskId,
        event_type: String,
        data: serde_json::Value,
    ) -> ParallelResult<()> {
        let event = ExecutionEvent {
            task_id,
            event_type,
            timestamp: chrono::Utc::now(),
            data,
        };

        self.monitoring_handle
            .publish_event(event)
            .await
            .map_err(|e| ParallelError::ProgressTracking {
                message: format!("Failed to publish event: {}", e),
                source: None,
            })
    }

    /// Get current task progress from orchestration monitoring
    pub async fn get_task_progress(
        &self,
        task_id: &TaskId,
    ) -> ParallelResult<Option<ExecutionProgress>> {
        self.monitoring_handle
            .get_task_progress(task_id)
            .await
            .map_err(|e| ParallelError::ProgressTracking {
                message: format!("Failed to get task progress: {}", e),
                source: None,
            })
    }
}

/// Stub implementation for testing (replace with actual orchestration integration)
pub struct StubOrchestrationMonitoringHandle;

#[async_trait]
impl OrchestrationMonitoringHandle for StubOrchestrationMonitoringHandle {
    async fn update_task_progress(
        &self,
        _task_id: &TaskId,
        _progress: &ExecutionProgress,
    ) -> Result<(), OrchestrationMonitoringError> {
        // TODO: Implement actual progress update
        tracing::debug!("Stub: Updating task progress for {:?}", _task_id);
        Ok(())
    }

    async fn update_worker_progress(
        &self,
        _task_id: &TaskId,
        _worker_id: &WorkerId,
        _progress: &Progress,
    ) -> Result<(), OrchestrationMonitoringError> {
        // TODO: Implement actual worker progress update
        tracing::debug!("Stub: Updating worker progress for {:?} worker {:?}", _task_id, _worker_id);
        Ok(())
    }

    async fn publish_event(
        &self,
        event: ExecutionEvent,
    ) -> Result<(), OrchestrationMonitoringError> {
        // TODO: Implement actual event publishing
        tracing::debug!("Stub: Publishing event {} for task {:?}", event.event_type, event.task_id);
        Ok(())
    }

    async fn get_task_progress(
        &self,
        _task_id: &TaskId,
    ) -> Result<Option<ExecutionProgress>, OrchestrationMonitoringError> {
        // TODO: Implement actual progress retrieval
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_bridge_task_progress() {
        let stub_handle = Arc::new(StubOrchestrationMonitoringHandle);
        let bridge = OrchestrationMonitoringBridge::new(stub_handle);

        let task_id = TaskId::new();
        let result = bridge.update_task_progress(
            &task_id,
            ExecutionStatus::Running,
            0.5,
            Some("decomposition".to_string()),
            std::collections::HashMap::new(),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_monitoring_bridge_event_publishing() {
        let stub_handle = Arc::new(StubOrchestrationMonitoringHandle);
        let bridge = OrchestrationMonitoringBridge::new(stub_handle);

        let task_id = TaskId::new();
        let result = bridge.publish_event(
            task_id,
            "worker_started".to_string(),
            serde_json::json!({"worker_count": 3}),
        ).await;

        assert!(result.is_ok());
    }
}
