//! Judge Trait and Evidence Types
//!
//! Defines the core trait for judges in the constitutional council.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use v4_types::council::JudgeType;
use v4_types::task::TaskSpec;

/// Evidence provided to judges for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeEvidence {
    /// Task specification being evaluated
    pub task_spec: TaskSpec,
    /// Proposed operators (serialized)
    pub proposed_operators: Vec<String>,
    /// Code changes (if any)
    pub code_changes: Vec<CodeChange>,
    /// Test results (if any)
    pub test_results: Option<TestEvidence>,
    /// Previous verdicts from other judges (for awareness, not influence)
    pub previous_verdicts: Vec<PreviousVerdict>,
    /// Additional context
    pub context: std::collections::HashMap<String, serde_json::Value>,
}

impl JudgeEvidence {
    /// Create new evidence for a task
    pub fn new(task_spec: TaskSpec) -> Self {
        Self {
            task_spec,
            proposed_operators: Vec::new(),
            code_changes: Vec::new(),
            test_results: None,
            previous_verdicts: Vec::new(),
            context: std::collections::HashMap::new(),
        }
    }

    /// Add proposed operators
    pub fn with_operators(mut self, operators: Vec<String>) -> Self {
        self.proposed_operators = operators;
        self
    }

    /// Add code changes
    pub fn with_code_changes(mut self, changes: Vec<CodeChange>) -> Self {
        self.code_changes = changes;
        self
    }

    /// Add test results
    pub fn with_test_results(mut self, results: TestEvidence) -> Self {
        self.test_results = Some(results);
        self
    }

    /// Add context value
    pub fn with_context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }
}

/// A code change for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    /// File path
    pub path: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Lines added
    pub lines_added: u32,
    /// Lines removed
    pub lines_removed: u32,
    /// Brief description of change
    pub description: String,
}

/// Type of code change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// New file created
    Created,
    /// Existing file modified
    Modified,
    /// File deleted
    Deleted,
}

/// Test evidence for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEvidence {
    /// Total tests run
    pub total_tests: u32,
    /// Tests passed
    pub passed: u32,
    /// Tests failed
    pub failed: u32,
    /// Tests skipped
    pub skipped: u32,
    /// Coverage percentage (if available)
    pub coverage: Option<f64>,
    /// Failed test names
    pub failed_tests: Vec<String>,
}

impl TestEvidence {
    /// Calculate pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.total_tests == 0 {
            1.0
        } else {
            self.passed as f64 / self.total_tests as f64
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Previous verdict from another judge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousVerdict {
    /// Judge type that issued this verdict
    pub judge_type: JudgeType,
    /// Score given
    pub score: f64,
    /// Brief summary
    pub summary: String,
}

/// Result from a judge's evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeEvaluationResult {
    /// Judge identifier
    pub judge_id: String,
    /// Judge type
    pub judge_type: JudgeType,
    /// Numeric score (0.0 to 1.0)
    pub score: f64,
    /// Confidence in the score (0.0 to 1.0)
    pub confidence: f64,
    /// Reasoning for the score
    pub reasoning: String,
    /// Specific issues found
    pub issues: Vec<JudgeIssue>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

impl JudgeEvaluationResult {
    /// Check if this result would trigger a veto (score < 0.5)
    pub fn is_veto(&self) -> bool {
        self.score < 0.5
    }

    /// Check if this result passes minimum threshold
    pub fn passes_threshold(&self, threshold: f64) -> bool {
        self.score >= threshold
    }

    /// Get the weighted score (score * confidence)
    pub fn weighted_score(&self) -> f64 {
        self.score * self.confidence
    }
}

/// An issue identified by a judge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeIssue {
    /// Issue code
    pub code: String,
    /// Severity
    pub severity: IssueSeverity,
    /// Description
    pub description: String,
    /// Location (if applicable)
    pub location: Option<String>,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Severity of a judge issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical - must be fixed
    Critical,
    /// Major - should be fixed
    Major,
    /// Minor - nice to fix
    Minor,
    /// Info - informational only
    Info,
}

/// Core trait for judges in the constitutional council
///
/// Each judge evaluates evidence independently and produces a numeric score.
/// Implementations must ensure:
/// - INV-CORE-06: No nested LLM calls (single evaluation per judge)
/// - Consistent scoring criteria
/// - Clear reasoning
#[async_trait]
pub trait Judge: Send + Sync {
    /// Get the type of this judge
    fn judge_type(&self) -> JudgeType;

    /// Get the judge's unique identifier
    fn judge_id(&self) -> &str;

    /// Evaluate evidence and produce a result
    ///
    /// Must produce a score between 0.0 and 1.0.
    /// Score < 0.5 triggers automatic veto.
    async fn evaluate(&self, evidence: &JudgeEvidence) -> Result<JudgeEvaluationResult, JudgeError>;
}

/// Errors from judge evaluation
#[derive(Debug, thiserror::Error)]
pub enum JudgeError {
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Invalid evidence: {0}")]
    InvalidEvidence(String),

    #[error("Judge timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),
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

    #[test]
    fn test_evidence_creation() {
        let spec = make_task_spec();
        let evidence = JudgeEvidence::new(spec)
            .with_operators(vec!["Seek::ReadFile".to_string()])
            .with_context("key", serde_json::json!("value"));

        assert_eq!(evidence.proposed_operators.len(), 1);
        assert!(evidence.context.contains_key("key"));
    }

    #[test]
    fn test_test_evidence_pass_rate() {
        let evidence = TestEvidence {
            total_tests: 100,
            passed: 95,
            failed: 5,
            skipped: 0,
            coverage: Some(85.0),
            failed_tests: vec!["test_1".to_string()],
        };

        assert!((evidence.pass_rate() - 0.95).abs() < 0.001);
        assert!(!evidence.all_passed());
    }

    #[test]
    fn test_evaluation_result_veto() {
        let result = JudgeEvaluationResult {
            judge_id: "judge-1".to_string(),
            judge_type: JudgeType::Constitutional,
            score: 0.4, // Below veto threshold
            confidence: 0.9,
            reasoning: "Major issues found".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        };

        assert!(result.is_veto());
        assert!(!result.passes_threshold(0.5));
    }

    #[test]
    fn test_weighted_score() {
        let result = JudgeEvaluationResult {
            judge_id: "judge-1".to_string(),
            judge_type: JudgeType::Technical,
            score: 0.8,
            confidence: 0.9,
            reasoning: "Good".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 50,
            evaluated_at: chrono::Utc::now(),
        };

        assert!((result.weighted_score() - 0.72).abs() < 0.001);
    }
}
