//! Council Coordinator Port
//!
//! Defines the interface for council-based decision making and governance.
//! This port enables dependency injection and testing for constitutional oversight.
//!
//! @author @darianrosebrook

use crate::errors::CouncilResult;
use crate::types::council::CouncilVerdict;
use crate::types::planning::TaskDescriptor;
use schemars::JsonSchema;
use uuid::Uuid;

/// Session identifier for council review sessions
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(transparent)]
pub struct SessionId(#[schemars(with = "String")] pub Uuid);

/// Core council coordinator interface
/// Implementations provide constitutional oversight and decision making
#[async_trait::async_trait]
pub trait CouncilCoordinator: Send + Sync {
    /// Start a new council review session for a task
    ///
    /// # Arguments
    /// * `task` - The task descriptor to be reviewed
    ///
    /// # Returns
    /// Session ID for the review session, or an error if session creation fails
    async fn start_session(&self, task: &TaskDescriptor) -> CouncilResult<SessionId>;

    /// Review a task within an existing session
    ///
    /// # Arguments
    /// * `session_id` - The session ID from start_session
    /// * `task` - The task to review (may be updated from initial session)
    ///
    /// # Returns
    /// Council verdict (Approved, ConditionalApproval, Rejected), or an error if review fails
    async fn review_task(
        &self,
        session_id: &SessionId,
        task: &TaskDescriptor,
    ) -> CouncilResult<CouncilVerdict>;

    /// Get the status of a review session
    ///
    /// # Arguments
    /// * `session_id` - The session ID to query
    ///
    /// # Returns
    /// Current session status information, or an error if session not found
    async fn get_session_status(&self, session_id: &SessionId) -> CouncilResult<SessionStatus>;
}

/// Status of a council review session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct SessionStatus {
    /// Session identifier
    #[schemars(with = "String")]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, JsonSchema)]
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
