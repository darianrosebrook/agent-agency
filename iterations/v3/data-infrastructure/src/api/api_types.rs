//! API Types and Data Structures
//!
//! Common types used across the API layer for request/response handling,
//! configuration, and data transfer objects.

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Database configuration
    pub database_url: String,
    /// Redis URL (optional)
    pub redis_url: Option<String>,
    /// Log level
    pub log_level: String,
}

/// Task submission request
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskSubmissionRequest {
    pub description: String,
    pub context: Option<String>,
    pub priority: Option<String>,
    pub execution_mode: Option<String>,
}

/// Task submission response
#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskSubmissionResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Link provenance request
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkProvenanceRequest {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub provenance_id: Uuid,
    pub relationship_type: String,
    pub commit_hash: String,
}

/// Working specification (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkingSpec {
    pub id: String,
    pub title: String,
    pub description: String,
    pub risk_tier: u8,
    pub mode: String,
    pub change_budget: ChangeBudget,
    pub blast_radius: BlastRadius,
    pub operational_rollback_slo: String,
    pub scope: Scope,
    pub invariants: Vec<String>,
    pub acceptance: Vec<AcceptanceCriterion>,
    pub non_functional: NonFunctionalRequirements,
    pub contracts: Vec<Contract>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// Change budget
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangeBudget {
    pub max_files: u32,
    pub max_loc: u32,
}

/// Blast radius
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlastRadius {
    pub modules: Vec<String>,
    pub data_migration: bool,
}

/// Scope
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Scope {
    pub r#in: Vec<String>,
    pub out: Vec<String>,
}

/// Acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
}

/// Non-functional requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NonFunctionalRequirements {
    pub a11y: Vec<String>,
    pub perf: PerformanceRequirements,
    pub security: Vec<String>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceRequirements {
    pub api_p95_ms: u32,
    pub lcp_ms: u32,
}

/// Contract
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Contract {
    pub r#type: String,
    pub path: String,
}

/// Execution artifacts (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionArtifacts {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub working_spec: Option<WorkingSpec>,
    pub quality_report: Option<QualityReport>,
    pub artifacts: Vec<ArtifactMetadata>,
}

/// Artifact metadata (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactMetadata {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: u64,
}

/// Quality report (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityReport {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub score: f64,
    pub details: String,
    pub overall_score: f64,
    pub checks_passed: u32,
    pub checks_failed: u32,
}

/// Progress tracker (stub)
#[derive(Debug, Clone, JsonSchema)]
pub struct ProgressTracker {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub current_step: String,
    pub progress_percentage: u8,
}

impl ProgressTracker {
    /// Get progress for a task
    pub async fn get_progress(&self, _task_id: Uuid) -> Result<ExecutionProgress, anyhow::Error> {
        // TODO: Implement real progress tracking for tasks
        //       Currently returns cached progress from tracker; should query actual task execution state.
        //
        // COMPLETION CHECKLIST:
        // [ ] Query actual task execution state from task executor
        // [ ] Calculate progress percentage from task steps
        // [ ] Return current step and status from execution
        // [ ] Handle task not found errors
        // [ ] Add unit tests with mock task execution
        // [ ] Add integration tests with real task progress tracking
        //
        // ACCEPTANCE CRITERIA:
        // - Progress reflects actual task execution state
        // - Task not found returns appropriate error
        // - Progress percentage is accurate
        //
        // DEPENDENCIES:
        // - Task executor integration (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (API functionality)
        // - Change Budget: ~100 LOC
        Ok(ExecutionProgress {
            task_id: self.task_id,
            status: "in_progress".to_string(),
            progress: self.progress_percentage,
            current_step: self.current_step.clone(),
            estimated_completion: None,
        })
    }
}

/// Execution progress (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionProgress {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub progress: u8,
    pub current_step: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Orchestrator (stub)
#[derive(Debug, Clone, JsonSchema)]
pub struct Orchestrator {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub status: String,
}

impl Orchestrator {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            status: "active".to_string(),
        }
    }
}

/// Provenance response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProvenanceResponse {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub verdict_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub decision: Value,
    pub consensus_score: f64,
    pub caws_compliance: Value,
    pub git_commit_hash: Option<String>,
    pub git_trailer: String,
    pub signature: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub metadata: Value,
}

/// Dashboard diff summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardDiffSummary {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub diff_type: String,
    pub summary: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub iteration: u32,
    pub file_path: String,
    pub change_type: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub diff_preview: String,
}

/// Waiver approval request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverApprovalRequest {
    #[schemars(with = "String")]
    pub waiver_id: Uuid,
    pub approved_by: String,
    pub approval_notes: Option<String>,
}

/// Waiver request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverRequest {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    pub metadata: Value,
}

/// Waiver response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverResponse {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub metadata: Value,
}

/// Task result response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResultResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub result: Option<Value>,
    pub working_spec: Option<WorkingSpec>,
    pub artifacts: Option<ExecutionArtifacts>,
    pub quality_report: Option<QualityReport>,
    pub error_message: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Saved query response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SavedQueryResponse {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
}

/// Save query request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveQueryRequest {
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskStatusResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub progress: Option<f64>,
    pub progress_percentage: Option<f64>,
    pub current_phase: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
    pub quality_score: Option<f64>,
}

/// Dashboard iteration summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardIterationSummary {
    #[schemars(with = "String")]
    pub iteration_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub iteration_number: u32,
    pub iteration: u32,
    pub status: String,
    pub progress: f64,
    pub score: f64,
    pub stop_reason: Option<String>,
    pub file_changes: u32,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub model_used: String,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Dashboard task summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardTaskSummary {
    pub total_tasks: u64,
    pub active_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub success_rate: f64,
    #[schemars(with = "String")]
    pub average_completion_time: Option<Duration>,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub description: String,
    pub status: String,
    pub current_iteration: usize,
    pub total_iterations: usize,
    pub score: Option<f64>,
    pub execution_mode: String,
    #[schemars(with = "String")]
    pub start_time: DateTime<Utc>,
    #[schemars(with = "String")]
    pub last_update: DateTime<Utc>,
    pub iterations: Vec<DashboardIterationSummary>,
}
