//! API Type Definitions for Request/Response Structures
//!
//! Contains all request/response structs, configuration types, and data models
//! used throughout the REST API interface.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use agent_agency_contracts::{ExecutionArtifacts, QualityReport, WorkingSpec};

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Redis URL for WebSocket session management (optional, enables multi-instance support)
    pub redis_url: Option<String>,
    /// Stream timeout in seconds (default: 300 seconds / 5 minutes)
    pub stream_timeout_seconds: u64,
}

/// Task submission request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct TaskSubmissionResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct TaskStatusResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub progress_percentage: f32,
    pub current_phase: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub quality_score: Option<f64>,
}

/// Task result response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct TaskResultResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub result: Option<serde_json::Value>, // Task execution result summary
    pub working_spec: Option<WorkingSpec>,
    pub artifacts: Option<ExecutionArtifacts>,
    #[schemars(skip)]
    pub quality_report: Option<QualityReport>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveQueryRequest {
    pub name: String,
    pub query_text: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SavedQueryResponse {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub query_text: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Dashboard iteration summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardIterationSummary {
    pub iteration: usize,
    pub score: f64,
    pub stop_reason: String,
    pub file_changes: usize,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub model_used: String,
}

/// Dashboard task summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardTaskSummary {
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

/// Diff summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardDiffSummary {
    pub iteration: usize,
    pub file_path: String,
    pub change_type: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub diff_preview: String,
}

/// Waiver request for creating new waivers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverRequest {
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
}

// WaiverResponse moved to api_types.rs to eliminate duplication
// Re-export for backward compatibility
pub use super::api_types::WaiverResponse;

/// Waiver approval request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverApprovalRequest {
    pub approver: String,
    pub justification: Option<String>,
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
    pub decision: serde_json::Value,
    pub consensus_score: f32,
    pub caws_compliance: serde_json::Value,
    pub git_commit_hash: Option<String>,
    pub git_trailer: String,
    pub signature: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Link provenance request
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkProvenanceRequest {
    pub provenance_id: String,
    pub commit_hash: String,
}

/// Task persistence structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedTask {
    pub id: String,
    pub spec: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub metadata: String,
}

/// Task storage trait for persistence operations
#[async_trait::async_trait]
pub trait TaskStoreTrait: Send + Sync {
    async fn create_task(&self, task: PersistedTask) -> anyhow::Result<()>;
    async fn get_tasks(&self) -> anyhow::Result<Vec<PersistedTask>>;
    async fn get_task(&self, task_id: String) -> anyhow::Result<Option<PersistedTask>>;
    async fn get_task_events(&self, task_id: String) -> anyhow::Result<Vec<serde_json::Value>>;
}
