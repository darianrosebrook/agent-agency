//! API Type Definitions for Request/Response Structures
//!
//! Contains all request/response structs, configuration types, and data models
//! used throughout the REST API interface.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use agent_agency_contracts::{WorkingSpec, ExecutionArtifacts, QualityReport};

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// API key authentication required
    pub require_api_key: bool,
    /// API keys (if authentication enabled)
    pub api_keys: Vec<String>,
    /// Rate limiting enabled
    pub enable_rate_limiting: bool,
    /// Rate limit requests per minute
    pub rate_limit_per_minute: u32,
}

/// Task submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmissionRequest {
    /// Natural language task description
    pub description: String,
    /// Execution mode (strict/auto/dry-run)
    pub execution_mode: Option<String>,
    /// Risk tier override (optional)
    pub risk_tier: Option<String>,
    /// Additional context or requirements
    pub context: Option<String>,
    /// Priority level
    pub priority: Option<String>,
    /// Deadline (optional)
    pub deadline: Option<DateTime<Utc>>,
}

/// Task submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmissionResponse {
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub task_id: Uuid,
    pub status: String,
    pub progress_percentage: f32,
    pub current_phase: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub quality_score: Option<f64>,
}

/// Task result response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultResponse {
    pub task_id: Uuid,
    pub status: String,
    pub result: Option<serde_json::Value>, // Task execution result summary
    pub working_spec: Option<WorkingSpec>,
    pub artifacts: Option<ExecutionArtifacts>,
    pub quality_report: Option<QualityReport>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveQueryRequest {
    pub name: String,
    pub query_text: String,
}

#[derive(Debug, Serialize)]
pub struct SavedQueryResponse {
    pub id: Uuid,
    pub name: String,
    pub query_text: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Dashboard iteration summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardIterationSummary {
    pub iteration: usize,
    pub score: f64,
    pub stop_reason: String,
    pub file_changes: usize,
    pub timestamp: DateTime<Utc>,
    pub model_used: String,
}

/// Dashboard task summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTaskSummary {
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

/// Diff summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDiffSummary {
    pub iteration: usize,
    pub file_path: String,
    pub change_type: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub diff_preview: String,
}

/// Waiver request for creating new waivers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverRequest {
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    pub expires_at: DateTime<Utc>,
}

/// Waiver response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverResponse {
    pub id: Uuid,
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
    pub metadata: serde_json::Value,
}

/// Waiver approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverApprovalRequest {
    pub approver: String,
    pub justification: Option<String>,
}

/// Provenance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceResponse {
    pub id: Uuid,
    pub verdict_id: Uuid,
    pub task_id: Uuid,
    pub decision: serde_json::Value,
    pub consensus_score: f32,
    pub caws_compliance: serde_json::Value,
    pub git_commit_hash: Option<String>,
    pub git_trailer: String,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Link provenance request
#[derive(Debug, Deserialize)]
pub struct LinkProvenanceRequest {
    pub provenance_id: String,
    pub commit_hash: String,
}
