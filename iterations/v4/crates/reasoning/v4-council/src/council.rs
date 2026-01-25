//! Council Coordinator
//!
//! Coordinates the 3-judge constitutional council review process.

use v4_types::council::{CouncilVerdict, JudgeScores, SessionId};
use v4_types::task::TaskSpec;

use crate::aggregation::{aggregate, AggregatedVerdict, AggregationConfig};
use crate::constitutional::ConstitutionalJudge;
use crate::quality::QualityJudge;
use crate::session::SessionManager;
use crate::technical::TechnicalJudge;
use crate::traits::{Judge, JudgeEvidence};

/// Council configuration
#[derive(Debug, Clone)]
pub struct CouncilConfig {
    /// Aggregation configuration
    pub aggregation: AggregationConfig,
    /// Whether to run judges in parallel
    pub parallel_evaluation: bool,
    /// Maximum time for a single judge evaluation (ms)
    pub judge_timeout_ms: u64,
}

impl Default for CouncilConfig {
    fn default() -> Self {
        Self {
            aggregation: AggregationConfig::default(),
            parallel_evaluation: true,
            judge_timeout_ms: 30000, // 30 seconds
        }
    }
}

/// The Constitutional Council
///
/// Coordinates three judges:
/// - Constitutional: Ethical compliance and CAWS adherence
/// - Technical: Code quality and security
/// - Quality: Task completion and correctness
///
/// Implements veto logic: Any score < 0.5 = automatic rejection
pub struct Council {
    /// Constitutional judge
    constitutional: Box<dyn Judge>,
    /// Technical judge
    technical: Box<dyn Judge>,
    /// Quality judge
    quality: Box<dyn Judge>,
    /// Configuration
    config: CouncilConfig,
    /// Session manager
    sessions: std::sync::Mutex<SessionManager>,
}

impl Council {
    /// Create a new council with default judges
    pub fn new() -> Self {
        Self {
            constitutional: Box::new(ConstitutionalJudge::new()),
            technical: Box::new(TechnicalJudge::new()),
            quality: Box::new(QualityJudge::new()),
            config: CouncilConfig::default(),
            sessions: std::sync::Mutex::new(SessionManager::new()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CouncilConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Create with custom judges
    pub fn with_judges(
        constitutional: Box<dyn Judge>,
        technical: Box<dyn Judge>,
        quality: Box<dyn Judge>,
    ) -> Self {
        Self {
            constitutional,
            technical,
            quality,
            config: CouncilConfig::default(),
            sessions: std::sync::Mutex::new(SessionManager::new()),
        }
    }

    /// Create a new review session
    pub fn create_session(&self, task_id: impl Into<String>) -> SessionId {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.create_session(task_id)
    }

    /// Get session status
    pub fn get_session_status(
        &self,
        session_id: SessionId,
    ) -> Option<v4_types::council::SessionStatus> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get_session(session_id).map(|s| s.to_session_status())
    }

    /// Get session verdict
    pub fn get_verdict(&self, session_id: SessionId) -> Option<CouncilVerdict> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .get_session(session_id)
            .and_then(|s| s.verdict.as_ref())
            .map(|v| v.verdict.clone())
    }

    /// Get session scores
    pub fn get_scores(&self, session_id: SessionId) -> Option<JudgeScores> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .get_session(session_id)
            .and_then(|s| s.verdict.as_ref())
            .map(|v| v.scores.clone())
    }

    /// Run a full council review
    ///
    /// This is the main entry point for reviews. It:
    /// 1. Creates a session
    /// 2. Runs all three judges
    /// 3. Aggregates results
    /// 4. Returns the final verdict
    pub async fn review(
        &self,
        task_spec: TaskSpec,
        evidence: JudgeEvidence,
    ) -> Result<AggregatedVerdict, CouncilError> {
        // Create session
        let session_id = self.create_session(&task_spec.id);

        // Start session
        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_session_mut(session_id) {
                session.start();
            }
        }

        // Run judges (sequentially for now, to ensure INV-CORE-06)
        // INV-CORE-06: No nested LLM calls - each judge runs independently
        let constitutional_result = self.constitutional.evaluate(&evidence).await.map_err(|e| {
            self.fail_session(session_id, &e.to_string());
            CouncilError::JudgeError(format!("Constitutional: {}", e))
        })?;

        self.record_result(session_id, constitutional_result.clone());

        let technical_result = self.technical.evaluate(&evidence).await.map_err(|e| {
            self.fail_session(session_id, &e.to_string());
            CouncilError::JudgeError(format!("Technical: {}", e))
        })?;

        self.record_result(session_id, technical_result.clone());

        let quality_result = self.quality.evaluate(&evidence).await.map_err(|e| {
            self.fail_session(session_id, &e.to_string());
            CouncilError::JudgeError(format!("Quality: {}", e))
        })?;

        self.record_result(session_id, quality_result.clone());

        // Aggregate results
        let results = vec![constitutional_result, technical_result, quality_result];
        let verdict = aggregate(&results, &self.config.aggregation).map_err(|e| {
            self.fail_session(session_id, &e.to_string());
            CouncilError::AggregationError(e.to_string())
        })?;

        // Complete session
        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_session_mut(session_id) {
                session.complete(verdict.clone());
            }
        }

        Ok(verdict)
    }

    /// Quick review without session management
    pub async fn quick_review(&self, evidence: &JudgeEvidence) -> Result<AggregatedVerdict, CouncilError> {
        // Run all judges
        let constitutional_result = self
            .constitutional
            .evaluate(evidence)
            .await
            .map_err(|e| CouncilError::JudgeError(format!("Constitutional: {}", e)))?;

        let technical_result = self
            .technical
            .evaluate(evidence)
            .await
            .map_err(|e| CouncilError::JudgeError(format!("Technical: {}", e)))?;

        let quality_result = self
            .quality
            .evaluate(evidence)
            .await
            .map_err(|e| CouncilError::JudgeError(format!("Quality: {}", e)))?;

        // Aggregate
        let results = vec![constitutional_result, technical_result, quality_result];
        aggregate(&results, &self.config.aggregation)
            .map_err(|e| CouncilError::AggregationError(e.to_string()))
    }

    fn record_result(
        &self,
        session_id: SessionId,
        result: crate::traits::JudgeEvaluationResult,
    ) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_session_mut(session_id) {
            session.record_result(result);
        }
    }

    fn fail_session(&self, session_id: SessionId, error: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_session_mut(session_id) {
            session.fail(error);
        }
    }
}

impl Default for Council {
    fn default() -> Self {
        Self::new()
    }
}

/// Council errors
#[derive(Debug, thiserror::Error)]
pub enum CouncilError {
    #[error("Judge error: {0}")]
    JudgeError(String),

    #[error("Aggregation error: {0}")]
    AggregationError(String),

    #[error("Session not found: {0:?}")]
    SessionNotFound(SessionId),

    #[error("Session already complete")]
    SessionComplete,

    #[error("Timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::task::{AcceptanceCriterion, TaskConstraints};

    fn make_task_spec() -> TaskSpec {
        TaskSpec {
            id: "task-123".to_string(),
            version: "1.0".to_string(),
            title: "Test task".to_string(),
            description: "A test task".to_string(),
            goals: vec!["Goal 1".to_string()],
            risk_tier: 2,
            constraints: TaskConstraints::default(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1".to_string(),
                given: "Given state".to_string(),
                when: "When action".to_string(),
                then: "Then result".to_string(),
            }],
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_council_review() {
        let council = Council::new();
        let spec = make_task_spec();
        let mut evidence = JudgeEvidence::new(spec.clone());
        evidence.proposed_operators = vec!["Seek::ReadFile".to_string()];

        let verdict = council.review(spec, evidence).await.unwrap();

        // Should get some verdict
        assert!(verdict.scores.aggregate > 0.0);
    }

    #[tokio::test]
    async fn test_council_quick_review() {
        let council = Council::new();
        let spec = make_task_spec();
        let evidence = JudgeEvidence::new(spec);

        let verdict = council.quick_review(&evidence).await.unwrap();

        // All three judges should have contributed
        assert!(verdict.scores.constitutional > 0.0);
        assert!(verdict.scores.technical > 0.0);
        assert!(verdict.scores.quality > 0.0);
    }

    #[tokio::test]
    async fn test_council_session_management() {
        let council = Council::new();
        let spec = make_task_spec();
        let evidence = JudgeEvidence::new(spec.clone());

        // Create session first - this returns the session_id
        let session_id = council.create_session(&spec.id);

        // Check initial status
        let status = council.get_session_status(session_id).unwrap();
        assert_eq!(status.progress, 0.0);

        // Instead of calling review() (which creates its own session),
        // we'll just verify the session was created correctly
        // The integration of session + review is tested in test_council_review

        // We can verify the session exists
        assert!(council.get_session_status(session_id).is_some());
    }

    #[tokio::test]
    async fn test_council_full_review_with_verdict() {
        let council = Council::new();
        let spec = make_task_spec();
        let evidence = JudgeEvidence::new(spec.clone());

        // Run review - this manages its own session
        let verdict = council.review(spec, evidence).await.unwrap();

        // Verify the verdict is complete
        assert!(verdict.scores.aggregate > 0.0);
        assert!(verdict.scores.constitutional > 0.0);
        assert!(verdict.scores.technical > 0.0);
        assert!(verdict.scores.quality > 0.0);
    }

    #[test]
    fn test_council_config() {
        let config = CouncilConfig {
            aggregation: AggregationConfig::strict(),
            parallel_evaluation: false,
            judge_timeout_ms: 60000,
        };

        let council = Council::with_config(config);
        assert!(!council.config.parallel_evaluation);
    }
}
