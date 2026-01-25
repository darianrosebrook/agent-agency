//! Council and judgment types
//!
//! Data structures for constitutional council operations, verdicts, and decisions.
//! Ported from V3 agent-agency-contracts with enhancements for numeric scoring.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Review priority levels for constitutional council reviews
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

/// Council verdict enumeration with numeric scoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum CouncilVerdict {
    /// Plan approved for execution (score >= 0.90)
    Approved { score: f64 },
    /// Plan conditionally approved with requirements (0.70 <= score < 0.90)
    ConditionalApproval { score: f64, requirements: Vec<String> },
    /// Plan rejected with reasons (score < 0.70)
    Rejected { score: f64, reasons: Vec<String> },
}

impl CouncilVerdict {
    /// Get the numeric score for this verdict
    pub fn score(&self) -> f64 {
        match self {
            Self::Approved { score } => *score,
            Self::ConditionalApproval { score, .. } => *score,
            Self::Rejected { score, .. } => *score,
        }
    }

    /// Check if this verdict passes the minimum threshold
    pub fn passes_threshold(&self, threshold: f64) -> bool {
        self.score() >= threshold
    }

    /// Create verdict from a numeric score
    pub fn from_score(score: f64, context: Option<Vec<String>>) -> Self {
        if score >= 0.90 {
            Self::Approved { score }
        } else if score >= 0.70 {
            Self::ConditionalApproval {
                score,
                requirements: context.unwrap_or_default(),
            }
        } else {
            Self::Rejected {
                score,
                reasons: context.unwrap_or_default(),
            }
        }
    }
}

/// Final decision from the constitutional council
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FinalDecision {
    /// Decision identifier
    pub id: String,
    /// Council verdict with score
    pub verdict: CouncilVerdict,
    /// Reasoning for the decision
    pub reasoning: String,
    /// Council members who participated
    pub participants: Vec<String>,
    /// Timestamp of decision
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Individual judge scores
    pub judge_scores: JudgeScores,
}

/// Scores from each judge in the council
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeScores {
    /// Constitutional judge score (ethical compliance)
    pub constitutional: f64,
    /// Technical judge score (code quality)
    pub technical: f64,
    /// Quality judge score (requirements satisfaction)
    pub quality: f64,
    /// Weighted aggregate score
    pub aggregate: f64,
}

impl JudgeScores {
    /// Calculate aggregate score with default weights
    pub fn calculate_aggregate(&self) -> f64 {
        // Weights: Constitutional 30%, Technical 35%, Quality 35%
        (self.constitutional * 0.30) + (self.technical * 0.35) + (self.quality * 0.35)
    }

    /// Check if all individual scores meet minimum thresholds
    pub fn all_pass_minimum(&self, min: f64) -> bool {
        self.constitutional >= min && self.technical >= min && self.quality >= min
    }
}

/// Judge engine result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeResult {
    /// Judge identifier
    pub judge_id: String,
    /// Judge type
    pub judge_type: JudgeType,
    /// Numeric score (0.0 to 1.0)
    pub score: f64,
    /// Confidence in the score (0.0 to 1.0)
    pub confidence: f64,
    /// Evidence considered
    pub evidence: Vec<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Types of judges in the council
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum JudgeType {
    /// Constitutional judge - ethical compliance and CAWS rules
    Constitutional,
    /// Technical judge - code quality and security
    Technical,
    /// Quality judge - requirements satisfaction
    Quality,
}

/// Session identifier for council review sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct SessionId(
    #[schemars(with = "String")]
    #[serde(with = "uuid::serde::simple")]
    pub uuid::Uuid,
);

impl SessionId {
    /// Create a new random session ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

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
    #[schemars(with = "Option<String>")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_from_score() {
        let approved = CouncilVerdict::from_score(0.95, None);
        assert!(matches!(approved, CouncilVerdict::Approved { .. }));

        let conditional = CouncilVerdict::from_score(0.80, Some(vec!["fix tests".to_string()]));
        assert!(matches!(conditional, CouncilVerdict::ConditionalApproval { .. }));

        let rejected = CouncilVerdict::from_score(0.50, Some(vec!["major issues".to_string()]));
        assert!(matches!(rejected, CouncilVerdict::Rejected { .. }));
    }

    #[test]
    fn test_verdict_score_returns_correct_value() {
        // Test that score() returns the actual score value, not 0 or -1
        let approved = CouncilVerdict::Approved { score: 0.95 };
        assert!((approved.score() - 0.95).abs() < 0.001);

        let conditional = CouncilVerdict::ConditionalApproval {
            score: 0.75,
            requirements: vec![],
        };
        assert!((conditional.score() - 0.75).abs() < 0.001);

        let rejected = CouncilVerdict::Rejected {
            score: 0.3,
            reasons: vec![],
        };
        assert!((rejected.score() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_verdict_passes_threshold() {
        let verdict = CouncilVerdict::Approved { score: 0.85 };

        // At threshold - should pass
        assert!(verdict.passes_threshold(0.85));

        // Below threshold - should pass
        assert!(verdict.passes_threshold(0.80));

        // Above threshold - should fail
        assert!(!verdict.passes_threshold(0.90));
    }

    #[test]
    fn test_verdict_passes_threshold_boundary() {
        // Test that >= is used, not just >
        let verdict = CouncilVerdict::Approved { score: 0.5 };
        assert!(verdict.passes_threshold(0.5)); // Exactly at threshold
        assert!(!verdict.passes_threshold(0.500001)); // Just above
    }

    #[test]
    fn test_judge_scores_aggregate() {
        let scores = JudgeScores {
            constitutional: 0.90,
            technical: 0.85,
            quality: 0.80,
            aggregate: 0.0, // Will be calculated
        };
        let aggregate = scores.calculate_aggregate();
        // 0.90 * 0.30 + 0.85 * 0.35 + 0.80 * 0.35 = 0.27 + 0.2975 + 0.28 = 0.8475
        assert!((aggregate - 0.8475).abs() < 0.001);
    }

    #[test]
    fn test_judge_scores_all_pass_minimum() {
        let passing = JudgeScores {
            constitutional: 0.8,
            technical: 0.8,
            quality: 0.8,
            aggregate: 0.8,
        };
        assert!(passing.all_pass_minimum(0.8));
        assert!(passing.all_pass_minimum(0.79));
        assert!(!passing.all_pass_minimum(0.81));
    }

    #[test]
    fn test_judge_scores_all_pass_minimum_checks_all_fields() {
        // Only constitutional fails
        let const_fails = JudgeScores {
            constitutional: 0.5,
            technical: 0.9,
            quality: 0.9,
            aggregate: 0.8,
        };
        assert!(!const_fails.all_pass_minimum(0.7));

        // Only technical fails
        let tech_fails = JudgeScores {
            constitutional: 0.9,
            technical: 0.5,
            quality: 0.9,
            aggregate: 0.8,
        };
        assert!(!tech_fails.all_pass_minimum(0.7));

        // Only quality fails
        let quality_fails = JudgeScores {
            constitutional: 0.9,
            technical: 0.9,
            quality: 0.5,
            aggregate: 0.8,
        };
        assert!(!quality_fails.all_pass_minimum(0.7));
    }

    #[test]
    fn test_judge_scores_all_pass_minimum_boundary() {
        // Exactly at minimum - should pass
        let at_minimum = JudgeScores {
            constitutional: 0.7,
            technical: 0.7,
            quality: 0.7,
            aggregate: 0.7,
        };
        assert!(at_minimum.all_pass_minimum(0.7));

        // Just below minimum - should fail
        let below_minimum = JudgeScores {
            constitutional: 0.699,
            technical: 0.7,
            quality: 0.7,
            aggregate: 0.7,
        };
        assert!(!below_minimum.all_pass_minimum(0.7));
    }

    #[test]
    fn test_session_id_uniqueness() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }
}
