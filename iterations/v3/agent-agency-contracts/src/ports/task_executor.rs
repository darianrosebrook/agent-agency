//! Task Executor Port
//!
//! Defines the interface for task execution that can be implemented
//! by different execution backends. This port enables dependency injection
//! and breaks circular dependencies between orchestration and workers.
//!
//! @author @darianrosebrook

use crate::execution_artifacts::ExecutionArtifacts;
use crate::types::planning::TaskDescriptor;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Task Executor Port
// ============================================================================

/// Core task executor port
///
/// This trait defines the interface for executing tasks. Implementations
/// can provide full orchestration, worker-based execution, or mock
/// implementations for testing.
#[async_trait]
pub trait TaskExecutorPort: Send + Sync {
    /// Execute a task and return artifacts
    ///
    /// # Arguments
    /// * `task_descriptor` - The task to execute
    ///
    /// # Returns
    /// Execution artifacts on success, or an error if execution fails
    async fn execute_task(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> Result<ExecutionArtifacts, TaskExecutionError>;

    /// Execute a task and return artifacts with observability data
    ///
    /// Default implementation calls execute_task and returns empty observability.
    ///
    /// # Arguments
    /// * `task_descriptor` - The task to execute
    ///
    /// # Returns
    /// Execution result with observability data, or an error if execution fails
    async fn execute_task_with_observability(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> Result<ExecutionResultWithObservability, TaskExecutionError> {
        let artifacts = self.execute_task(task_descriptor).await?;
        Ok(ExecutionResultWithObservability {
            artifacts,
            observability: None,
        })
    }
}

// ============================================================================
// Execution Result Types
// ============================================================================

/// Extended execution result with observability data
#[derive(Debug, Clone)]
pub struct ExecutionResultWithObservability {
    /// The primary execution artifacts
    pub artifacts: ExecutionArtifacts,
    /// Observability data collected during execution
    pub observability: Option<TaskObservabilityData>,
}

/// Observability data collected during task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskObservabilityData {
    /// Chain of thought entries
    pub chain_of_thought: Vec<ChainOfThoughtEntry>,
    /// Council decisions made during execution
    pub council_decisions: Vec<CouncilDecisionData>,
    /// Worker actions performed
    pub worker_actions: Vec<WorkerActionData>,
    /// Decision points encountered
    pub decision_points: Vec<DecisionPointData>,
    /// Coordination events
    pub coordination_events: Vec<CoordinationEventData>,
}

impl Default for TaskObservabilityData {
    fn default() -> Self {
        Self {
            chain_of_thought: Vec::new(),
            council_decisions: Vec::new(),
            worker_actions: Vec::new(),
            decision_points: Vec::new(),
            coordination_events: Vec::new(),
        }
    }
}

// ============================================================================
// Chain of Thought Types
// ============================================================================

/// Chain of thought entry for reasoning transparency
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainOfThoughtEntry {
    /// Entry identifier
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Timestamp of the thought
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Phase of execution (planning, execution, review, etc.)
    pub phase: String,
    /// The thought or reasoning step
    pub thought: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Supporting evidence or context
    pub evidence: Vec<String>,
    /// Alternatives considered
    pub alternatives: Vec<String>,
    /// Why this thought was chosen
    pub rationale: Option<String>,
}

// ============================================================================
// Council Decision Types
// ============================================================================

/// Council decision data for observability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CouncilDecisionData {
    /// Session identifier
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Decision type (approval, rejection, conditional)
    pub decision_type: String,
    /// Timestamp of the decision
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Overall verdict
    pub verdict: String,
    /// Confidence level
    pub confidence: f64,
    /// Individual judge contributions
    pub judge_contributions: Vec<JudgeContributionData>,
    /// Conditions or requirements (if conditional approval)
    pub conditions: Vec<String>,
}

/// Judge contribution data for observability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeContributionData {
    /// Judge identifier
    pub judge_id: String,
    /// Judge name
    pub judge_name: String,
    /// Verdict from this judge
    pub verdict: String,
    /// Confidence level
    pub confidence: f64,
    /// Reasoning provided
    pub reasoning: String,
}

// ============================================================================
// Worker Action Types
// ============================================================================

/// Worker action data for observability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerActionData {
    /// Worker identifier
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    /// Action performed
    pub action: String,
    /// Milestone identifier
    pub milestone_id: String,
    /// Timestamp of the action
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether the action succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

// ============================================================================
// Decision Point Types
// ============================================================================

/// Decision point data for observability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DecisionPointData {
    /// Decision identifier
    #[schemars(with = "String")]
    pub decision_id: Uuid,
    /// Type of decision
    pub decision_type: String,
    /// Timestamp of the decision
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Option that was chosen
    pub chosen_option: String,
    /// Number of alternatives considered
    pub alternatives_count: usize,
    /// Confidence in the decision
    pub confidence: f64,
    /// Reasoning for the decision
    pub reasoning: String,
}

// ============================================================================
// Coordination Event Types
// ============================================================================

/// Coordination event data for observability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationEventData {
    /// Event type
    pub event_type: String,
    /// Timestamp of the event
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Event details as JSON
    pub details: serde_json::Value,
}

// ============================================================================
// Error Types
// ============================================================================

/// Task execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskExecutionError {
    /// Task validation failed
    ValidationError(String),
    /// Execution failed
    ExecutionError(String),
    /// Task was cancelled
    Cancelled(String),
    /// Timeout occurred
    Timeout(String),
    /// Resource unavailable
    ResourceUnavailable(String),
    /// Permission denied
    PermissionDenied(String),
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for TaskExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskExecutionError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TaskExecutionError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            TaskExecutionError::Cancelled(msg) => write!(f, "Cancelled: {}", msg),
            TaskExecutionError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            TaskExecutionError::ResourceUnavailable(msg) => {
                write!(f, "Resource unavailable: {}", msg)
            }
            TaskExecutionError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            TaskExecutionError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for TaskExecutionError {}

impl From<anyhow::Error> for TaskExecutionError {
    fn from(err: anyhow::Error) -> Self {
        TaskExecutionError::Unknown(err.to_string())
    }
}
