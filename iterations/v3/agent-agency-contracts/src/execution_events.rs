//! @darianrosebrook
//! Execution event types and shared working spec definitions.
//!
//! These types are shared between orchestration and workers crates.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Execution event types for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// Execution started
    ExecutionStarted {
        task_id: Uuid,
        working_spec_id: String,
        timestamp: DateTime<Utc>,
    },
    /// Execution completed successfully
    ExecutionCompleted {
        task_id: Uuid,
        success: bool,
        artifacts: crate::ExecutionArtifacts,
        execution_time_ms: u64,
    },
    /// Execution failed
    ExecutionFailed {
        task_id: Uuid,
        error: String,
        working_spec_id: String,
        artifacts: crate::ExecutionArtifacts,
    },
    /// Worker assigned to task
    WorkerAssigned {
        task_id: Uuid,
        worker_id: Uuid,
        estimated_completion_time: DateTime<Utc>,
    },
    /// Quality check completed
    QualityCheckCompleted {
        task_id: Uuid,
        check_type: String,
        passed: bool,
    },
    /// Execution phase started
    ExecutionPhaseStarted {
        task_id: Uuid,
        phase: String,
        timestamp: DateTime<Utc>,
    },
    /// Execution phase completed
    ExecutionPhaseCompleted {
        task_id: Uuid,
        phase: String,
        duration_ms: u64,
    },
    /// Execution progress update
    ExecutionProgress {
        task_id: Uuid,
        phase: String,
        progress_percent: f32,
    },
}

/// Working spec for CAWS compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    /// Unique spec identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Risk tier for quality gates
    pub risk_tier: u8,
    /// Scope boundaries
    pub scope: Option<WorkingSpecScope>,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,
}

/// Working spec scope boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpecScope {
    /// Included paths/patterns
    pub included: Vec<String>,
    /// Excluded paths/patterns
    pub excluded: Vec<String>,
}

