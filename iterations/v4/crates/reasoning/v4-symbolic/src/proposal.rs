//! Operator Proposal Types
//!
//! Defines the proposal and validation types for symbolic reasoning.

use serde::{Deserialize, Serialize};
use v4_types::OperatorType;

use crate::provenance::ProvenanceChain;

/// A proposal for operators to execute a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorProposal {
    /// Unique proposal identifier
    pub proposal_id: String,
    /// Task this proposal is for
    pub task_id: String,
    /// Proposed operators in execution order
    pub operators: Vec<OperatorType>,
    /// Hash of the reasoning process (for determinism verification)
    pub reasoning_hash: String,
    /// Provenance chain for audit trail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ProvenanceChain>,
    /// Estimated complexity (1-10)
    pub complexity: u8,
    /// Timestamp of proposal creation
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl OperatorProposal {
    /// Create a new operator proposal
    pub fn new(task_id: impl Into<String>, operators: Vec<OperatorType>) -> Self {
        let task_id = task_id.into();
        let reasoning_hash = Self::compute_reasoning_hash(&task_id, &operators);

        Self {
            proposal_id: uuid::Uuid::new_v4().to_string(),
            task_id,
            operators,
            reasoning_hash,
            provenance: None,
            complexity: 1,
            created_at: chrono::Utc::now(),
        }
    }

    /// Attach a provenance chain to this proposal
    pub fn with_provenance(mut self, provenance: ProvenanceChain) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Set the complexity estimate
    pub fn with_complexity(mut self, complexity: u8) -> Self {
        self.complexity = complexity.clamp(1, 10);
        self
    }

    /// Compute a deterministic hash of the reasoning
    fn compute_reasoning_hash(task_id: &str, operators: &[OperatorType]) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(task_id.as_bytes());
        for op in operators {
            hasher.update(format!("{:?}", op).as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    /// Verify the reasoning hash matches the current state
    pub fn verify_reasoning_hash(&self) -> bool {
        let expected = Self::compute_reasoning_hash(&self.task_id, &self.operators);
        self.reasoning_hash == expected
    }

    /// Get the operator classes (S/M/P/K/C) as a string
    pub fn operator_classes(&self) -> String {
        self.operators
            .iter()
            .map(|op| op.class())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Count operators by class
    pub fn operator_counts(&self) -> OperatorCounts {
        let mut counts = OperatorCounts::default();
        for op in &self.operators {
            match op.class() {
                "S" => counts.seek += 1,
                "M" => counts.memorize += 1,
                "P" => counts.perceive += 1,
                "K" => counts.knowledge += 1,
                "C" => counts.control += 1,
                _ => {}
            }
        }
        counts
    }

    /// Check if proposal has side effects
    pub fn has_side_effects(&self) -> bool {
        self.operators.iter().any(|op| op.has_side_effects())
    }
}

/// Counts of operators by class
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperatorCounts {
    /// Seek operators (S)
    pub seek: usize,
    /// Memorize operators (M)
    pub memorize: usize,
    /// Perceive operators (P)
    pub perceive: usize,
    /// Knowledge operators (K)
    pub knowledge: usize,
    /// Control operators (C)
    pub control: usize,
}

impl OperatorCounts {
    /// Total number of operators
    pub fn total(&self) -> usize {
        self.seek + self.memorize + self.perceive + self.knowledge + self.control
    }
}

/// Result of validating an operator proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the proposal is valid
    pub valid: bool,
    /// Validation score (0.0 to 1.0)
    pub score: f64,
    /// List of issues found
    pub issues: Vec<ValidationIssue>,
    /// List of warnings (non-blocking)
    pub warnings: Vec<String>,
    /// Timestamp of validation
    #[serde(with = "chrono::serde::ts_seconds")]
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

impl ValidationResult {
    /// Create a passing validation result
    pub fn pass(score: f64) -> Self {
        Self {
            valid: true,
            score: score.clamp(0.0, 1.0),
            issues: Vec::new(),
            warnings: Vec::new(),
            validated_at: chrono::Utc::now(),
        }
    }

    /// Create a failing validation result
    pub fn fail(issues: Vec<ValidationIssue>) -> Self {
        Self {
            valid: false,
            score: 0.0,
            issues,
            warnings: Vec::new(),
            validated_at: chrono::Utc::now(),
        }
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Check if there are critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Critical))
    }
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Severity
    pub severity: IssueSeverity,
    /// Related invariant (if any)
    pub invariant: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        severity: IssueSeverity,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity,
            invariant: None,
        }
    }

    /// Associate with an invariant
    pub fn with_invariant(mut self, invariant: impl Into<String>) -> Self {
        self.invariant = Some(invariant.into());
        self
    }
}

/// Severity of a validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical - blocks execution
    Critical,
    /// Error - should be fixed
    Error,
    /// Warning - advisory
    Warning,
    /// Info - informational only
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::operators::SeekOp;

    #[test]
    fn test_proposal_creation() {
        let operators = vec![
            OperatorType::Seek(SeekOp::ReadFile {
                path: "test.rs".to_string(),
            }),
            OperatorType::Seek(SeekOp::SearchCode {
                pattern: "fn main".to_string(),
                path: None,
            }),
        ];

        let proposal = OperatorProposal::new("task-123", operators);

        assert_eq!(proposal.task_id, "task-123");
        assert_eq!(proposal.operators.len(), 2);
        assert!(!proposal.reasoning_hash.is_empty());
        assert!(proposal.verify_reasoning_hash());
    }

    #[test]
    fn test_operator_classes() {
        let operators = vec![
            OperatorType::Seek(SeekOp::ReadFile {
                path: "a.rs".to_string(),
            }),
            OperatorType::Seek(SeekOp::ReadFile {
                path: "b.rs".to_string(),
            }),
        ];

        let proposal = OperatorProposal::new("task-123", operators);
        assert_eq!(proposal.operator_classes(), "SS");
    }

    #[test]
    fn test_operator_counts() {
        let operators = vec![
            OperatorType::Seek(SeekOp::ReadFile {
                path: "a.rs".to_string(),
            }),
            OperatorType::Seek(SeekOp::ReadFile {
                path: "b.rs".to_string(),
            }),
        ];

        let proposal = OperatorProposal::new("task-123", operators);
        let counts = proposal.operator_counts();

        assert_eq!(counts.seek, 2);
        assert_eq!(counts.total(), 2);
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult::pass(0.95).with_warning("Minor style issue");
        assert!(result.valid);
        assert!(!result.has_critical_issues());
        assert_eq!(result.warnings.len(), 1);

        let issue = ValidationIssue::new("CYCLE-001", "Cycle detected", IssueSeverity::Critical)
            .with_invariant("INV-CORE-07");
        let result = ValidationResult::fail(vec![issue]);
        assert!(!result.valid);
        assert!(result.has_critical_issues());
    }

    #[test]
    fn test_reasoning_hash_determinism() {
        let operators = vec![OperatorType::Seek(SeekOp::ReadFile {
            path: "test.rs".to_string(),
        })];

        let proposal1 = OperatorProposal::new("task-123", operators.clone());
        let proposal2 = OperatorProposal::new("task-123", operators);

        assert_eq!(proposal1.reasoning_hash, proposal2.reasoning_hash);
    }
}
