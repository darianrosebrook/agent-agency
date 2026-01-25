//! Review Session Management
//!
//! Manages council review sessions and tracks their progress.

use serde::{Deserialize, Serialize};
use v4_types::council::{SessionId, SessionStatus, SessionStatusType};

use crate::aggregation::AggregatedVerdict;
use crate::traits::JudgeEvaluationResult;

/// A council review session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    /// Session identifier
    pub session_id: SessionId,
    /// Task being reviewed
    pub task_id: String,
    /// Current status
    pub status: SessionState,
    /// Judge results (as they come in)
    pub judge_results: Vec<JudgeEvaluationResult>,
    /// Final verdict (when complete)
    pub verdict: Option<AggregatedVerdict>,
    /// Session creation time
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Session completion time (if complete)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Error message (if failed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session created, not yet started
    Created,
    /// Waiting for constitutional judge
    AwaitingConstitutional,
    /// Waiting for technical judge
    AwaitingTechnical,
    /// Waiting for quality judge
    AwaitingQuality,
    /// All judges evaluated, aggregating
    Aggregating,
    /// Session complete with verdict
    Complete,
    /// Session failed
    Failed,
}

impl ReviewSession {
    /// Create a new review session
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            session_id: SessionId::new(),
            task_id: task_id.into(),
            status: SessionState::Created,
            judge_results: Vec::new(),
            verdict: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
            error: None,
        }
    }

    /// Get the session status in v4-types format
    pub fn to_session_status(&self) -> SessionStatus {
        let (status_type, progress) = match self.status {
            SessionState::Created => (SessionStatusType::Initializing, 0.0),
            SessionState::AwaitingConstitutional => (SessionStatusType::Reviewing, 0.1),
            SessionState::AwaitingTechnical => (SessionStatusType::Reviewing, 0.4),
            SessionState::AwaitingQuality => (SessionStatusType::Reviewing, 0.7),
            SessionState::Aggregating => (SessionStatusType::Reviewing, 0.9),
            SessionState::Complete => (SessionStatusType::Completed, 1.0),
            SessionState::Failed => (SessionStatusType::Failed, 0.0),
        };

        SessionStatus {
            session_id: self.session_id,
            status: status_type,
            progress,
            pending_requirements: self.pending_requirements(),
            estimated_completion: None,
        }
    }

    /// Get pending requirements based on state
    fn pending_requirements(&self) -> Vec<String> {
        match self.status {
            SessionState::Created => vec!["Start review".to_string()],
            SessionState::AwaitingConstitutional => {
                vec!["Awaiting constitutional judge evaluation".to_string()]
            }
            SessionState::AwaitingTechnical => {
                vec!["Awaiting technical judge evaluation".to_string()]
            }
            SessionState::AwaitingQuality => vec!["Awaiting quality judge evaluation".to_string()],
            SessionState::Aggregating => vec!["Aggregating verdicts".to_string()],
            SessionState::Complete | SessionState::Failed => vec![],
        }
    }

    /// Start the review process
    pub fn start(&mut self) {
        if self.status == SessionState::Created {
            self.status = SessionState::AwaitingConstitutional;
        }
    }

    /// Record a judge result
    pub fn record_result(&mut self, result: JudgeEvaluationResult) {
        use v4_types::council::JudgeType;

        self.judge_results.push(result.clone());

        // Update state based on which judge reported
        match result.judge_type {
            JudgeType::Constitutional => {
                if self.status == SessionState::AwaitingConstitutional {
                    self.status = SessionState::AwaitingTechnical;
                }
            }
            JudgeType::Technical => {
                if self.status == SessionState::AwaitingTechnical {
                    self.status = SessionState::AwaitingQuality;
                }
            }
            JudgeType::Quality => {
                if self.status == SessionState::AwaitingQuality {
                    self.status = SessionState::Aggregating;
                }
            }
        }
    }

    /// Check if all judges have reported
    pub fn all_judges_complete(&self) -> bool {
        use v4_types::council::JudgeType;

        let has_constitutional = self
            .judge_results
            .iter()
            .any(|r| r.judge_type == JudgeType::Constitutional);
        let has_technical = self
            .judge_results
            .iter()
            .any(|r| r.judge_type == JudgeType::Technical);
        let has_quality = self
            .judge_results
            .iter()
            .any(|r| r.judge_type == JudgeType::Quality);

        has_constitutional && has_technical && has_quality
    }

    /// Complete the session with a verdict
    pub fn complete(&mut self, verdict: AggregatedVerdict) {
        self.verdict = Some(verdict);
        self.status = SessionState::Complete;
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark the session as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = SessionState::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Get elapsed time since creation
    pub fn elapsed(&self) -> chrono::Duration {
        chrono::Utc::now() - self.created_at
    }

    /// Check if session is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, SessionState::Complete | SessionState::Failed)
    }

    /// Get the number of judges that have reported
    pub fn judge_count(&self) -> usize {
        self.judge_results.len()
    }
}

/// Session manager for tracking multiple sessions
pub struct SessionManager {
    /// Active sessions
    sessions: std::collections::HashMap<SessionId, ReviewSession>,
    /// Maximum sessions to track
    max_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            max_sessions: 1000,
        }
    }

    /// Create with custom max sessions
    pub fn with_max_sessions(max: usize) -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            max_sessions: max,
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, task_id: impl Into<String>) -> SessionId {
        // Clean up old completed sessions if at limit
        if self.sessions.len() >= self.max_sessions {
            self.cleanup_completed();
        }

        let session = ReviewSession::new(task_id);
        let session_id = session.session_id;
        self.sessions.insert(session_id, session);
        session_id
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: SessionId) -> Option<&ReviewSession> {
        self.sessions.get(&session_id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, session_id: SessionId) -> Option<&mut ReviewSession> {
        self.sessions.get_mut(&session_id)
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: SessionId) -> Option<ReviewSession> {
        self.sessions.remove(&session_id)
    }

    /// Get all active (non-complete) sessions
    pub fn active_sessions(&self) -> Vec<&ReviewSession> {
        self.sessions.values().filter(|s| !s.is_complete()).collect()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Clean up completed sessions
    fn cleanup_completed(&mut self) {
        let completed: Vec<SessionId> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_complete())
            .map(|(id, _)| *id)
            .collect();

        for id in completed.into_iter().take(self.max_sessions / 2) {
            self.sessions.remove(&id);
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::council::JudgeType;

    fn make_result(judge_type: JudgeType, score: f64) -> JudgeEvaluationResult {
        JudgeEvaluationResult {
            judge_id: format!("{:?}-1", judge_type),
            judge_type,
            score,
            confidence: 0.9,
            reasoning: "Test".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_session_creation() {
        let session = ReviewSession::new("task-123");
        assert_eq!(session.task_id, "task-123");
        assert_eq!(session.status, SessionState::Created);
        assert!(session.judge_results.is_empty());
    }

    #[test]
    fn test_session_progress() {
        let mut session = ReviewSession::new("task-123");

        session.start();
        assert_eq!(session.status, SessionState::AwaitingConstitutional);

        session.record_result(make_result(JudgeType::Constitutional, 0.9));
        assert_eq!(session.status, SessionState::AwaitingTechnical);

        session.record_result(make_result(JudgeType::Technical, 0.85));
        assert_eq!(session.status, SessionState::AwaitingQuality);

        session.record_result(make_result(JudgeType::Quality, 0.88));
        assert_eq!(session.status, SessionState::Aggregating);

        assert!(session.all_judges_complete());
    }

    #[test]
    fn test_session_status_conversion() {
        let mut session = ReviewSession::new("task-123");
        session.start();

        let status = session.to_session_status();
        assert_eq!(status.status, SessionStatusType::Reviewing);
        assert!(status.progress > 0.0);
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();

        let session_id = manager.create_session("task-1");
        assert!(manager.get_session(session_id).is_some());
        assert_eq!(manager.session_count(), 1);

        manager.create_session("task-2");
        assert_eq!(manager.session_count(), 2);
    }

    #[test]
    fn test_session_manager_active_filter() {
        let mut manager = SessionManager::new();

        let id1 = manager.create_session("task-1");
        let id2 = manager.create_session("task-2");

        // Complete one session
        if let Some(session) = manager.get_session_mut(id1) {
            session.fail("test error");
        }

        let active = manager.active_sessions();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].session_id, id2);
    }
}
