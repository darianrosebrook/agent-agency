//! @darianrosebrook
//! Execution event types and shared working spec definitions.
//!
//! These types are shared between orchestration and workers crates.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;

/// Execution event types for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionEvent {
    /// Execution started
    ExecutionStarted {
        #[schemars(with = "String")]
        task_id: Uuid,
        working_spec_id: String,
        #[schemars(with = "String")]
    timestamp: DateTime<Utc>,
    },
    /// Execution completed successfully
    ExecutionCompleted {
        #[schemars(with = "String")]
        task_id: Uuid,
        success: bool,
        artifacts: crate::ExecutionArtifacts,
        execution_time_ms: u64,
    },
    /// Execution failed
    ExecutionFailed {
        #[schemars(with = "String")]
        task_id: Uuid,
        error: String,
        working_spec_id: String,
        artifacts: crate::ExecutionArtifacts,
    },
    /// Worker assigned to task
    WorkerAssigned {
        #[schemars(with = "String")]
        task_id: Uuid,
        #[schemars(with = "String")]
        worker_id: Uuid,
        #[schemars(with = "String")]
    estimated_completion_time: DateTime<Utc>,
    },
    /// Quality check completed
    QualityCheckCompleted {
        #[schemars(with = "String")]
        task_id: Uuid,
        check_type: String,
        passed: bool,
    },
    /// Execution phase started
    ExecutionPhaseStarted {
        #[schemars(with = "String")]
        task_id: Uuid,
        phase: String,
        #[schemars(with = "String")]
    timestamp: DateTime<Utc>,
    },
    /// Execution phase completed
    ExecutionPhaseCompleted {
        #[schemars(with = "String")]
        task_id: Uuid,
        phase: String,
        duration_ms: u64,
    },
    /// Execution progress update
    ExecutionProgress {
        #[schemars(with = "String")]
        task_id: Uuid,
        phase: String,
        progress_percent: f32,
    },
}

// WorkingSpec and WorkingSpecScope definitions moved to working_spec.rs for consolidation

