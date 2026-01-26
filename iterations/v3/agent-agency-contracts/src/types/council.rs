//! Council and judgment types
//!
//! Data structures for constitutional council operations, verdicts, and decisions.
//! These types define the governance domain and are shared between council and orchestration.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Review priority levels for constitutional council reviews
///
/// This enum is shared between agent-constitutional-council and agent-orchestration
/// to avoid circular dependencies. Both crates should import from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ReviewPriority {
    /// Low priority - can be deferred
    Low,
    /// Normal priority - standard review timeline
    Normal,
    /// High priority - expedited review
    High,
    /// Critical priority - immediate review required
    Critical,
}

impl Default for ReviewPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<crate::types::planning::TaskPriority> for ReviewPriority {
    fn from(priority: crate::types::planning::TaskPriority) -> Self {
        match priority {
            crate::types::planning::TaskPriority::Low => ReviewPriority::Low,
            crate::types::planning::TaskPriority::Normal => ReviewPriority::Normal,
            crate::types::planning::TaskPriority::Medium => ReviewPriority::Normal,
            crate::types::planning::TaskPriority::High => ReviewPriority::High,
            crate::types::planning::TaskPriority::Urgent => ReviewPriority::High,
            crate::types::planning::TaskPriority::Critical => ReviewPriority::Critical,
        }
    }
}

/// Council verdict enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CouncilVerdict {
    /// Plan approved for execution
    Approved,
    /// Plan conditionally approved with requirements
    ConditionalApproval,
    /// Plan rejected with reasons
    Rejected,
}

/// Final decision from the constitutional council
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FinalDecision {
    /// Decision identifier
    pub id: String,
    /// Council verdict
    pub verdict: CouncilVerdict,
    /// Reasoning for the decision
    pub reasoning: String,
    /// Additional requirements if conditionally approved
    pub requirements: Vec<String>,
    /// Council members who participated
    pub participants: Vec<String>,
    /// Timestamp of decision
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Judge engine result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeResult {
    /// Judge identifier
    pub judge_id: String,
    /// Verdict reached
    pub verdict: CouncilVerdict,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Evidence considered
    pub evidence: Vec<String>,
    /// Processing time
    pub processing_time_ms: u64,
}

/// Session identifier for council review sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct SessionId(
    #[schemars(with = "String")]
    #[serde(with = "uuid::serde::simple")]
    pub uuid::Uuid,
);

/// Status of a council review session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionStatus {
    /// Session identifier
    pub session_id: SessionId,
    /// Current status
    pub status: SessionStatusType,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
    /// Any pending requirements or issues
    pub pending_requirements: Vec<String>,
    /// Estimated completion time
    #[schemars(with = "String")]
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

/// Session status types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SessionStatusType {
    /// Session is initializing
    Initializing,
    /// Session is actively reviewing
    Reviewing,
    /// Session is waiting for additional information
    WaitingForInfo,
    /// Session has reached a decision
    Completed,
    /// Session failed or was cancelled
    Failed,
}
