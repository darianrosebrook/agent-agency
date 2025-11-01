//! Council and judgment types
//!
//! Data structures for constitutional council operations, verdicts, and decisions.
//! These types define the governance domain and are shared between council and orchestration.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};

/// Council verdict enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouncilVerdict {
    /// Plan approved for execution
    Approved,
    /// Plan conditionally approved with requirements
    ConditionalApproval,
    /// Plan rejected with reasons
    Rejected,
}

/// Final decision from the constitutional council
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Judge engine result
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(#[serde(with = "uuid::serde::simple")] pub uuid::Uuid);

/// Status of a council review session
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

/// Session status types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

