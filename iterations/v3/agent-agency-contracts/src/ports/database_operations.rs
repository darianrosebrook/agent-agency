//! Database Operations Port
//!
//! Defines the core interface for database operations that can be implemented
//! by different database backends. This port enables dependency injection and
//! breaks circular dependencies between orchestration and data infrastructure.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Core Database Operations Port
// ============================================================================

/// Core database operations port
///
/// This trait defines the full interface needed for database operations
/// across the system. Implementations can provide full database access or
/// mock implementations for testing.
#[async_trait]
pub trait DatabaseOperationsPort: Send + Sync {
    // ========================================================================
    // Execution Plan Operations
    // ========================================================================

    /// Create a new execution plan
    async fn create_execution_plan(
        &self,
        plan: CreateExecutionPlanRequest,
    ) -> Result<ExecutionPlanRecord, DatabaseError>;

    /// Get an execution plan by ID
    async fn get_execution_plan(
        &self,
        id: Uuid,
    ) -> Result<Option<ExecutionPlanRecord>, DatabaseError>;

    /// List all execution plans
    async fn list_execution_plans(&self) -> Result<Vec<ExecutionPlanRecord>, DatabaseError>;

    /// Update an execution plan
    async fn update_execution_plan(
        &self,
        id: Uuid,
        update: UpdateExecutionPlanRequest,
    ) -> Result<ExecutionPlanRecord, DatabaseError>;

    /// Delete an execution plan
    async fn delete_execution_plan(&self, id: Uuid) -> Result<(), DatabaseError>;

    // ========================================================================
    // Audit Trail Operations
    // ========================================================================

    /// Create an audit trail entry
    async fn create_audit_entry(
        &self,
        entry: CreateAuditEntryRequest,
    ) -> Result<AuditEntryRecord, DatabaseError>;

    /// Get audit entries for a task
    async fn get_audit_entries(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<AuditEntryRecord>, DatabaseError>;

    /// Get a single audit entry by ID
    async fn get_audit_entry(&self, id: Uuid) -> Result<Option<AuditEntryRecord>, DatabaseError>;

    // ========================================================================
    // Planning Session Operations
    // ========================================================================

    /// Create a new planning session
    async fn create_planning_session(
        &self,
        session: CreatePlanningSessionRequest,
    ) -> Result<PlanningSessionRecord, DatabaseError>;

    /// Get a planning session by ID
    async fn get_planning_session(
        &self,
        id: Uuid,
    ) -> Result<Option<PlanningSessionRecord>, DatabaseError>;

    /// Update a planning session
    async fn update_planning_session(
        &self,
        id: Uuid,
        update: UpdatePlanningSessionRequest,
    ) -> Result<(), DatabaseError>;

    // ========================================================================
    // Planning Telemetry Operations
    // ========================================================================

    /// Create planning telemetry entry
    async fn create_planning_telemetry(
        &self,
        telemetry: CreatePlanningTelemetryRequest,
    ) -> Result<PlanningTelemetryRecord, DatabaseError>;

    /// Get planning telemetry for a plan
    async fn get_planning_telemetry(
        &self,
        plan_id: Uuid,
        metric_type: Option<String>,
    ) -> Result<Vec<PlanningTelemetryRecord>, DatabaseError>;

    // ========================================================================
    // Planning Audit Event Operations
    // ========================================================================

    /// Create a planning audit event
    async fn create_planning_audit_event(
        &self,
        event: CreatePlanningAuditEventRequest,
    ) -> Result<(), DatabaseError>;

    /// Get planning audit events for a plan
    async fn get_planning_audit_events(
        &self,
        plan_id: Uuid,
    ) -> Result<Vec<PlanningAuditEventRecord>, DatabaseError>;

    // ========================================================================
    // Judge Operations
    // ========================================================================

    /// Create a new judge
    async fn create_judge(&self, judge: CreateJudgeRequest) -> Result<JudgeRecord, DatabaseError>;

    /// Get a judge by ID
    async fn get_judge(&self, id: Uuid) -> Result<Option<JudgeRecord>, DatabaseError>;

    /// Get all judges
    async fn get_judges(&self) -> Result<Vec<JudgeRecord>, DatabaseError>;

    // ========================================================================
    // Judge Evaluation Operations
    // ========================================================================

    /// Create a judge evaluation
    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluationRequest,
    ) -> Result<JudgeEvaluationRecord, DatabaseError>;

    /// Get judge evaluations for a task
    async fn get_judge_evaluations(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<JudgeEvaluationRecord>, DatabaseError>;

    // ========================================================================
    // Worker Operations
    // ========================================================================

    /// Get all workers
    async fn get_workers(&self) -> Result<Vec<WorkerRecord>, DatabaseError>;

    /// Get a worker by ID
    async fn get_worker(&self, id: Uuid) -> Result<Option<WorkerRecord>, DatabaseError>;

    /// Create a new worker
    async fn create_worker(
        &self,
        worker: CreateWorkerRequest,
    ) -> Result<WorkerRecord, DatabaseError>;

    /// Update a worker
    async fn update_worker(
        &self,
        id: Uuid,
        update: UpdateWorkerRequest,
    ) -> Result<WorkerRecord, DatabaseError>;

    // ========================================================================
    // Waiver Operations
    // ========================================================================

    /// Get waivers with optional status filter
    async fn get_waivers(
        &self,
        status: Option<String>,
    ) -> Result<Vec<WaiverRecord>, DatabaseError>;

    /// Create a new waiver
    async fn create_waiver(
        &self,
        waiver: CreateWaiverRequest,
    ) -> Result<WaiverRecord, DatabaseError>;

    /// Update a waiver
    async fn update_waiver(
        &self,
        id: Uuid,
        update: UpdateWaiverRequest,
    ) -> Result<WaiverRecord, DatabaseError>;

    // ========================================================================
    // Execution Result Operations
    // ========================================================================

    /// Create an execution result
    async fn create_execution_result(
        &self,
        result: CreateExecutionResultRequest,
    ) -> Result<ExecutionResultRecord, DatabaseError>;

    /// Get execution result for a plan
    async fn get_execution_result(
        &self,
        plan_id: Uuid,
    ) -> Result<Option<ExecutionResultRecord>, DatabaseError>;

    // ========================================================================
    // Council Session Operations
    // ========================================================================

    /// Create a council session
    async fn create_council_session(
        &self,
        session: CreateCouncilSessionRequest,
    ) -> Result<CouncilSessionRecord, DatabaseError>;

    /// Get a council session by ID
    async fn get_council_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<CouncilSessionRecord>, DatabaseError>;

    /// Get council session by task ID
    async fn get_council_session_by_task(
        &self,
        task_id: Uuid,
    ) -> Result<Option<CouncilSessionRecord>, DatabaseError>;

    /// Update a council session
    async fn update_council_session(
        &self,
        session_id: Uuid,
        update: UpdateCouncilSessionRequest,
    ) -> Result<CouncilSessionRecord, DatabaseError>;

    // ========================================================================
    // Health Check
    // ========================================================================

    /// Health check for database connection
    async fn health_check(&self) -> Result<bool, DatabaseError>;
}

// ============================================================================
// Execution Plan Types
// ============================================================================

/// Request to create an execution plan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateExecutionPlanRequest {
    /// Plan identifier (optional - will be generated if not provided)
    #[schemars(with = "Option<String>")]
    pub id: Option<Uuid>,
    /// Workspace ID (optional)
    pub workspace_id: Option<String>,
    /// Working spec ID this plan is for
    pub working_spec_id: String,
    /// Plan title
    pub title: String,
    /// Plan overview/description
    pub overview: String,
    /// Initial state
    pub state: String,
    /// Milestones as JSON
    pub milestones: serde_json::Value,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Request to update an execution plan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateExecutionPlanRequest {
    /// Updated title
    pub title: Option<String>,
    /// Updated overview
    pub overview: Option<String>,
    /// Updated state
    pub state: Option<String>,
    /// Updated milestones
    pub milestones: Option<serde_json::Value>,
    /// Updated metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Execution plan record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPlanRecord {
    /// Plan identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Session identifier
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Workspace identifier (optional)
    pub workspace_id: Option<String>,
    /// Working spec ID
    pub working_spec_id: String,
    /// Plan title
    pub title: String,
    /// Plan overview
    pub overview: Option<String>,
    /// Current state
    pub state: String,
    /// Milestones as JSON
    pub milestones: serde_json::Value,
    /// Dependency graph as JSON
    pub dependency_graph: serde_json::Value,
    /// Change budget as JSON
    pub change_budget: serde_json::Value,
    /// Quality gates as JSON
    pub quality_gates: serde_json::Value,
    /// Evidence requirements as JSON
    pub evidence_requirements: serde_json::Value,
    /// Active waivers as JSON
    pub active_waivers: serde_json::Value,
    /// Metadata as JSON
    pub metadata: serde_json::Value,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
    /// Approval timestamp
    #[schemars(with = "Option<String>")]
    pub approved_at: Option<DateTime<Utc>>,
    /// Completion timestamp
    #[schemars(with = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Audit Entry Types
// ============================================================================

/// Request to create an audit entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAuditEntryRequest {
    /// Task ID this entry is for
    #[schemars(with = "String")]
    pub task_id: Uuid,
    /// Event type
    pub event_type: String,
    /// Event description
    pub description: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Audit entry record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditEntryRecord {
    /// Entry identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Task ID
    #[schemars(with = "String")]
    pub task_id: Uuid,
    /// Event type
    pub event_type: String,
    /// Event description
    pub description: String,
    /// Event timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Planning Session Types
// ============================================================================

/// Request to create a planning session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningSessionRequest {
    /// Plan ID this session is for
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Request to update a planning session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdatePlanningSessionRequest {
    /// Updated status
    pub status: Option<String>,
    /// Updated metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Planning session record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningSessionRecord {
    /// Session identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Plan identifier
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    /// Session status
    pub status: String,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Planning Telemetry Types
// ============================================================================

/// Request to create planning telemetry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningTelemetryRequest {
    /// Session ID
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Metric name
    pub metric_name: String,
    /// Metric value
    pub metric_value: f64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Planning telemetry record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningTelemetryRecord {
    /// Telemetry identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Session identifier
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Metric name
    pub metric_name: String,
    /// Metric value
    pub metric_value: f64,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Planning Audit Event Types
// ============================================================================

/// Request to create a planning audit event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningAuditEventRequest {
    /// Plan ID
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    /// Event type
    pub event_type: String,
    /// Event description
    pub description: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Planning audit event record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningAuditEventRecord {
    /// Event identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Session identifier
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Event type
    pub event_type: String,
    /// Event description
    pub description: String,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Judge Types
// ============================================================================

/// Request to create a judge
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateJudgeRequest {
    /// Judge identifier (optional - will be generated if not provided)
    #[schemars(with = "Option<String>")]
    pub id: Option<Uuid>,
    /// Judge name
    pub name: String,
    /// Judge type
    pub judge_type: String,
    /// Configuration as JSON
    pub configuration: serde_json::Value,
}

/// Judge record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeRecord {
    /// Judge identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Judge name
    pub name: String,
    /// Judge type
    pub judge_type: String,
    /// Configuration as JSON
    pub configuration: serde_json::Value,
    /// Whether the judge is active
    pub is_active: bool,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Judge Evaluation Types
// ============================================================================

/// Request to create a judge evaluation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateJudgeEvaluationRequest {
    /// Judge ID
    #[schemars(with = "String")]
    pub judge_id: Uuid,
    /// Task ID
    #[schemars(with = "String")]
    pub task_id: Uuid,
    /// Evaluation as JSON
    pub evaluation: serde_json::Value,
    /// Score
    pub score: f64,
}

/// Judge evaluation record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeEvaluationRecord {
    /// Evaluation identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Judge identifier
    #[schemars(with = "String")]
    pub judge_id: Uuid,
    /// Task identifier
    #[schemars(with = "String")]
    pub task_id: Uuid,
    /// Evaluation as JSON
    pub evaluation: serde_json::Value,
    /// Score
    pub score: f64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Worker Types
// ============================================================================

/// Request to create a worker
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWorkerRequest {
    /// Worker name
    pub name: String,
    /// Worker type
    pub worker_type: String,
    /// Specialty (optional)
    pub specialty: Option<String>,
    /// Model name
    pub model_name: String,
    /// Endpoint
    pub endpoint: String,
    /// Capabilities as JSON
    pub capabilities: serde_json::Value,
    /// Performance history as JSON
    pub performance_history: serde_json::Value,
    /// Whether the worker is active
    pub is_active: bool,
}

/// Request to update a worker
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateWorkerRequest {
    /// Updated name
    pub name: Option<String>,
    /// Updated worker type
    pub worker_type: Option<String>,
    /// Updated specialty
    pub specialty: Option<String>,
    /// Updated model name
    pub model_name: Option<String>,
    /// Updated endpoint
    pub endpoint: Option<String>,
    /// Updated capabilities
    pub capabilities: Option<serde_json::Value>,
    /// Updated performance history
    pub performance_history: Option<serde_json::Value>,
    /// Updated active status
    pub is_active: Option<bool>,
}

/// Worker record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerRecord {
    /// Worker identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Worker name
    pub name: String,
    /// Worker type
    pub worker_type: String,
    /// Specialty (optional)
    pub specialty: Option<String>,
    /// Model name
    pub model_name: String,
    /// Endpoint
    pub endpoint: String,
    /// Capabilities as JSON
    pub capabilities: serde_json::Value,
    /// Performance history as JSON
    pub performance_history: serde_json::Value,
    /// Whether the worker is active
    pub is_active: bool,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Waiver Types
// ============================================================================

/// Request to create a waiver
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWaiverRequest {
    /// Plan ID
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    /// Waiver type
    pub waiver_type: String,
    /// Reason for waiver
    pub reason: String,
    /// Approved by
    pub approved_by: String,
    /// Waived gates
    pub gates: Vec<String>,
    /// Impact level (low, medium, high, critical)
    pub impact_level: String,
    /// Mitigation plan (optional)
    pub mitigation_plan: Option<String>,
    /// Expiration time (optional)
    #[schemars(with = "Option<String>")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request to update a waiver
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateWaiverRequest {
    /// Updated status
    pub status: Option<String>,
    /// Updated mitigation plan
    pub mitigation_plan: Option<String>,
    /// Updated expiration
    #[schemars(with = "Option<String>")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Waiver record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverRecord {
    /// Waiver identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Plan identifier
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    /// Waiver type
    pub waiver_type: String,
    /// Reason for waiver
    pub reason: String,
    /// Approved by
    pub approved_by: String,
    /// Status
    pub status: String,
    /// Waived gates
    pub gates: Vec<String>,
    /// Impact level
    pub impact_level: String,
    /// Mitigation plan (optional)
    pub mitigation_plan: Option<String>,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp (optional)
    #[schemars(with = "Option<String>")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Execution Result Types
// ============================================================================

/// Request to create an execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateExecutionResultRequest {
    /// Plan ID
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    /// Success flag
    pub success: bool,
    /// Number of milestones completed
    pub milestones_completed: i32,
    /// Total duration in milliseconds
    pub total_duration_ms: i64,
    /// Evidence as JSON
    pub evidence: serde_json::Value,
    /// Metrics as JSON
    pub metrics: serde_json::Value,
    /// Final state
    pub final_state: String,
    /// Timeline as JSON
    pub timeline: serde_json::Value,
}

/// Execution result record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionResultRecord {
    /// Plan identifier
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    /// Success flag
    pub success: bool,
    /// Number of milestones completed
    pub milestones_completed: i32,
    /// Total duration in milliseconds
    pub total_duration_ms: i64,
    /// Evidence as JSON
    pub evidence: serde_json::Value,
    /// Metrics as JSON
    pub metrics: serde_json::Value,
    /// Final state
    pub final_state: String,
    /// Timeline as JSON
    pub timeline: serde_json::Value,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Council Session Types
// ============================================================================

/// Request to create a council session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCouncilSessionRequest {
    /// Session ID
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Task ID (optional)
    #[schemars(with = "Option<String>")]
    pub task_id: Option<Uuid>,
    /// Working spec ID (optional)
    pub working_spec_id: Option<String>,
    /// Review context as JSON
    pub review_context: serde_json::Value,
    /// Initial status (optional)
    pub status: Option<String>,
    /// Selected judges as JSON (optional)
    pub selected_judges: Option<serde_json::Value>,
    /// Contributions as JSON (optional)
    pub contributions: Option<serde_json::Value>,
    /// Progress (optional)
    pub progress: Option<f64>,
    /// Metadata as JSON (optional)
    pub metadata: Option<serde_json::Value>,
}

/// Request to update a council session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCouncilSessionRequest {
    /// Updated status
    pub status: Option<String>,
    /// Updated selected judges
    pub selected_judges: Option<serde_json::Value>,
    /// Updated contributions
    pub contributions: Option<serde_json::Value>,
    /// Updated aggregation result
    pub aggregation_result: Option<serde_json::Value>,
    /// Updated final decision
    pub final_decision: Option<serde_json::Value>,
    /// Updated progress
    pub progress: Option<f64>,
    /// Completion timestamp
    #[schemars(with = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Updated metadata
    pub metadata: Option<serde_json::Value>,
}

/// Council session record from database
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CouncilSessionRecord {
    /// Record identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Session identifier
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Task identifier (optional)
    #[schemars(with = "Option<String>")]
    pub task_id: Option<Uuid>,
    /// Working spec ID (optional)
    pub working_spec_id: Option<String>,
    /// Review context as JSON
    pub review_context: serde_json::Value,
    /// Status
    pub status: String,
    /// Selected judges as JSON
    pub selected_judges: serde_json::Value,
    /// Contributions as JSON
    pub contributions: serde_json::Value,
    /// Aggregation result as JSON (optional)
    pub aggregation_result: Option<serde_json::Value>,
    /// Final decision as JSON (optional)
    pub final_decision: Option<serde_json::Value>,
    /// Progress (0.0 to 1.0)
    pub progress: f64,
    /// Start timestamp
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
    /// Completion timestamp (optional)
    #[schemars(with = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
    /// Metadata as JSON
    pub metadata: serde_json::Value,
}

// ============================================================================
// Error Types
// ============================================================================

/// Database operation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseError {
    /// Connection error
    ConnectionError(String),
    /// Query error
    QueryError(String),
    /// Not found error
    NotFound(String),
    /// Constraint violation
    ConstraintViolation(String),
    /// Serialization error
    SerializationError(String),
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "Query error: {}", msg),
            DatabaseError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DatabaseError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            DatabaseError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            DatabaseError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<anyhow::Error> for DatabaseError {
    fn from(err: anyhow::Error) -> Self {
        DatabaseError::Unknown(err.to_string())
    }
}
