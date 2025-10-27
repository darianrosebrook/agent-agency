//! API Types and Data Structures
//!
//! Common types used across the API layer for request/response handling,
//! configuration, and data transfer objects.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct TaskSubmissionRequest {
    pub description: String,
    pub context: Option<String>,
    pub priority: Option<String>,
    pub execution_mode: Option<String>,
}

/// Task submission response
#[derive(Debug, Serialize)]
pub struct TaskSubmissionResponse {
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Link provenance request
#[derive(Debug, Deserialize)]
pub struct LinkProvenanceRequest {
    pub task_id: Uuid,
    pub provenance_id: Uuid,
    pub relationship_type: String,
    pub commit_hash: String,
}

/// Working specification (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub created_at: DateTime<Utc>,
}

/// Change budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeBudget {
    pub max_files: u32,
    pub max_loc: u32,
}

/// Blast radius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    pub modules: Vec<String>,
    pub data_migration: bool,
}

/// Scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub r#in: Vec<String>,
    pub out: Vec<String>,
}

/// Acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
}

/// Non-functional requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonFunctionalRequirements {
    pub a11y: Vec<String>,
    pub perf: PerformanceRequirements,
    pub security: Vec<String>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    pub api_p95_ms: u32,
    pub lcp_ms: u32,
}

/// Contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub r#type: String,
    pub path: String,
}

/// Execution artifacts (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionArtifacts {
    pub task_id: Uuid,
    pub working_spec: Option<WorkingSpec>,
    pub quality_report: Option<QualityReport>,
    pub artifacts: Vec<ArtifactMetadata>,
}

/// Artifact metadata (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: u64,
}

/// Quality report (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub task_id: Uuid,
    pub score: f64,
    pub details: String,
    pub overall_score: f64,
    pub checks_passed: u32,
    pub checks_failed: u32,
}

/// Progress tracker (stub)
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    pub task_id: Uuid,
    pub current_step: String,
    pub progress_percentage: u8,
}

impl ProgressTracker {
    /// Get progress for a task (stub implementation)
    pub async fn get_progress(&self, _task_id: Uuid) -> Result<ExecutionProgress, anyhow::Error> {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProgress {
    pub task_id: Uuid,
    pub status: String,
    pub progress: u8,
    pub current_step: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Orchestrator (stub)
#[derive(Debug, Clone)]
pub struct Orchestrator {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceResponse {
    pub id: Uuid,
    pub verdict_id: Uuid,
    pub task_id: Uuid,
    pub decision: Value,
    pub consensus_score: f64,
    pub caws_compliance: Value,
    pub git_commit_hash: Option<String>,
    pub git_trailer: String,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Value,
}

/// Dashboard diff summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDiffSummary {
    pub id: Uuid,
    pub task_id: Uuid,
    pub diff_type: String,
    pub summary: String,
    pub timestamp: DateTime<Utc>,
    pub iteration: u32,
    pub file_path: String,
    pub change_type: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub diff_preview: String,
}

/// Waiver approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverApprovalRequest {
    pub waiver_id: Uuid,
    pub approved_by: String,
    pub approval_notes: Option<String>,
}

/// Waiver request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverRequest {
    pub task_id: Uuid,
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    pub expires_at: DateTime<Utc>,
    pub metadata: Value,
}

/// Waiver response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverResponse {
    pub id: Uuid,
    pub task_id: Uuid,
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub metadata: Value,
}

/// Task result response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultResponse {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQueryResponse {
    pub id: Uuid,
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Save query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveQueryRequest {
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub task_id: Uuid,
    pub status: String,
    pub progress: Option<f64>,
    pub progress_percentage: Option<f64>,
    pub current_phase: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub quality_score: Option<f64>,
}

/// Dashboard iteration summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardIterationSummary {
    pub iteration_id: Uuid,
    pub task_id: Uuid,
    pub iteration_number: u32,
    pub iteration: u32,
    pub status: String,
    pub progress: f64,
    pub score: f64,
    pub stop_reason: Option<String>,
    pub file_changes: u32,
    pub timestamp: DateTime<Utc>,
    pub model_used: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Dashboard task summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTaskSummary {
    pub total_tasks: u64,
    pub active_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub success_rate: f64,
    pub average_completion_time: Option<Duration>,
    pub task_id: Uuid,
    pub description: String,
    pub status: String,
    pub current_iteration: usize,
    pub total_iterations: usize,
    pub score: Option<f64>,
    pub execution_mode: String,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub iterations: Vec<DashboardIterationSummary>,
}