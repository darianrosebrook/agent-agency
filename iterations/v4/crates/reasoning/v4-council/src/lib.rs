//! V4 Constitutional Council
//!
//! This crate implements the 3-judge constitutional council for Agent Agency V4.
//! It provides numeric scoring with veto logic for comprehensive task evaluation.
//!
//! ## Design Principles
//!
//! 1. **Three Independent Judges** - Constitutional, Technical, and Quality
//! 2. **Veto Logic** - Any score < 0.5 triggers automatic rejection
//! 3. **Weighted Aggregation** - Configurable weights for each judge
//! 4. **No Nested LLM Calls** - Each judge runs independently (INV-CORE-06)
//!
//! ## Judges
//!
//! - **Constitutional**: Evaluates ethical compliance, CAWS adherence, risk tier
//! - **Technical**: Evaluates code quality, test coverage, security
//! - **Quality**: Evaluates task completion, acceptance criteria, correctness
//!
//! ## Scoring
//!
//! - 0.90+ : Approved
//! - 0.70-0.89: Conditional Approval (with requirements)
//! - <0.70: Rejected
//! - Any <0.50: Automatic veto regardless of aggregate
//!
//! ## Example
//!
//! ```rust,ignore
//! use v4_council::{Council, JudgeEvidence};
//!
//! let council = Council::new();
//! let evidence = JudgeEvidence::new(task_spec);
//!
//! let verdict = council.quick_review(&evidence).await?;
//!
//! if verdict.vetoed {
//!     println!("Vetoed by: {:?}", verdict.vetoing_judges);
//! } else {
//!     println!("Score: {:.2}", verdict.scores.aggregate);
//! }
//! ```

pub mod aggregation;
pub mod constitutional;
pub mod council;
pub mod quality;
pub mod session;
pub mod technical;
pub mod traits;

// Re-export main types
pub use aggregation::{aggregate, AggregatedIssue, AggregatedVerdict, AggregationConfig, AggregationError};
pub use constitutional::ConstitutionalJudge;
pub use council::{Council, CouncilConfig, CouncilError};
pub use quality::QualityJudge;
pub use session::{ReviewSession, SessionManager, SessionState};
pub use technical::TechnicalJudge;
pub use traits::{
    ChangeType, CodeChange, IssueSeverity, Judge, JudgeError, JudgeEvaluationResult,
    JudgeEvidence, JudgeIssue, PreviousVerdict, TestEvidence,
};

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::task::{AcceptanceCriterion, TaskConstraints, TaskSpec};

    fn make_task_spec(risk_tier: u8) -> TaskSpec {
        TaskSpec {
            id: "test-task".to_string(),
            version: "1.0".to_string(),
            title: "Test task".to_string(),
            description: "A test task for council review".to_string(),
            goals: vec!["Complete the test".to_string()],
            risk_tier,
            constraints: TaskConstraints::default(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1".to_string(),
                given: "A test environment".to_string(),
                when: "Tests are run".to_string(),
                then: "All tests pass".to_string(),
            }],
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_full_council_pipeline() {
        let council = Council::new();

        // Create evidence for a low-risk task with good test results
        let mut evidence = JudgeEvidence::new(make_task_spec(1));
        evidence.proposed_operators = vec!["Seek::ReadFile".to_string()];
        evidence.code_changes = vec![CodeChange {
            path: "src/lib.rs".to_string(),
            change_type: ChangeType::Modified,
            lines_added: 50,
            lines_removed: 10,
            description: "Added new feature".to_string(),
        }];
        evidence.test_results = Some(TestEvidence {
            total_tests: 100,
            passed: 100,
            failed: 0,
            skipped: 0,
            coverage: Some(85.0),
            failed_tests: vec![],
        });

        let verdict = council.quick_review(&evidence).await.unwrap();

        // Low risk + good tests should get good scores
        assert!(verdict.scores.aggregate >= 0.7);
        assert!(!verdict.vetoed);
    }

    #[tokio::test]
    async fn test_high_risk_lower_score() {
        let council = Council::new();

        // Create evidence for a very high risk task
        let evidence = JudgeEvidence::new(make_task_spec(5));

        let verdict = council.quick_review(&evidence).await.unwrap();

        // Risk tier 5 gives lower scores but doesn't trigger veto in default config
        // (constitutional score ~0.825 due to weighted dimensions)
        // The aggregate score should be lower than a low-risk task
        assert!(verdict.scores.constitutional < 0.9);
        assert!(verdict.scores.aggregate < verdict.scores.constitutional + 0.1);
    }

    #[tokio::test]
    async fn test_conditional_approval() {
        let council = Council::new();

        // Create evidence for a medium-quality task
        let mut evidence = JudgeEvidence::new(make_task_spec(2));
        evidence.proposed_operators = vec!["Seek::ReadFile".to_string()];
        evidence.test_results = Some(TestEvidence {
            total_tests: 100,
            passed: 90,
            failed: 10,
            skipped: 0,
            coverage: Some(70.0),
            failed_tests: vec!["test_edge_case".to_string()],
        });

        let verdict = council.quick_review(&evidence).await.unwrap();

        // Should get conditional approval or rejection based on aggregate
        assert!(!verdict.vetoed || verdict.scores.aggregate < 0.5);
    }

    #[test]
    fn test_aggregation_weights() {
        use v4_types::council::JudgeType;

        let results = vec![
            JudgeEvaluationResult {
                judge_id: "const-1".to_string(),
                judge_type: JudgeType::Constitutional,
                score: 0.90,
                confidence: 0.9,
                reasoning: "Good".to_string(),
                issues: vec![],
                recommendations: vec![],
                processing_time_ms: 100,
                evaluated_at: chrono::Utc::now(),
            },
            JudgeEvaluationResult {
                judge_id: "tech-1".to_string(),
                judge_type: JudgeType::Technical,
                score: 0.85,
                confidence: 0.9,
                reasoning: "Good".to_string(),
                issues: vec![],
                recommendations: vec![],
                processing_time_ms: 100,
                evaluated_at: chrono::Utc::now(),
            },
            JudgeEvaluationResult {
                judge_id: "qual-1".to_string(),
                judge_type: JudgeType::Quality,
                score: 0.80,
                confidence: 0.9,
                reasoning: "Good".to_string(),
                issues: vec![],
                recommendations: vec![],
                processing_time_ms: 100,
                evaluated_at: chrono::Utc::now(),
            },
        ];

        let config = AggregationConfig::default();
        let verdict = aggregate(&results, &config).unwrap();

        // Check weighted average: 0.90*0.30 + 0.85*0.35 + 0.80*0.35 = 0.27 + 0.2975 + 0.28 = 0.8475
        assert!((verdict.scores.aggregate - 0.8475).abs() < 0.01);
    }
}
