//! Verdict and quality gate types
//!
//! Types for quality reports and CAWS gate enforcement.
//! Based on Distill's hard threshold approach.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Quality report with gate results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityReport {
    /// Report ID
    pub id: String,
    /// Task ID this report is for
    pub task_id: String,
    /// Individual gate results
    pub gates: Vec<GateResult>,
    /// Overall pass/fail status
    pub passed: bool,
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Fingerprints for reproducibility
    pub fingerprints: Fingerprints,
}

impl QualityReport {
    /// Check if all gates passed
    pub fn all_gates_passed(&self) -> bool {
        self.gates.iter().all(|g| g.passed)
    }

    /// Get failed gates
    pub fn failed_gates(&self) -> Vec<&GateResult> {
        self.gates.iter().filter(|g| !g.passed).collect()
    }

    /// Get the lowest scoring gate
    pub fn lowest_gate(&self) -> Option<&GateResult> {
        self.gates.iter().min_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Individual gate result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GateResult {
    /// Gate type
    pub gate: GateType,
    /// Score achieved (0.0 to 1.0)
    pub score: f64,
    /// Threshold required to pass
    pub threshold: f64,
    /// Whether this gate passed
    pub passed: bool,
    /// Details about the gate check
    pub details: Option<String>,
}

impl GateResult {
    /// Create a new gate result
    pub fn new(gate: GateType, score: f64, threshold: f64) -> Self {
        Self {
            gate,
            score,
            threshold,
            passed: score >= threshold,
            details: None,
        }
    }

    /// Create a gate result with details
    pub fn with_details(gate: GateType, score: f64, threshold: f64, details: String) -> Self {
        Self {
            gate,
            score,
            threshold,
            passed: score >= threshold,
            details: Some(details),
        }
    }

    /// How far from the threshold (positive = passed, negative = failed)
    pub fn margin(&self) -> f64 {
        self.score - self.threshold
    }
}

/// Types of quality gates (from Distill CAWS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum GateType {
    /// Integration F1 score (threshold: >= 0.90)
    IntegrationF1,
    /// Privacy compliance rate (threshold: = 1.0)
    PrivacyCompliance,
    /// Control integration score (threshold: = 0, hard fail)
    ControlIntegration,
    /// Fixture hit rate (threshold: >= 0.95)
    FixtureHitRate,
    /// Invariant violations (threshold: = 0)
    InvariantViolations,
    /// Code coverage (threshold: >= 0.80)
    CodeCoverage,
    /// Test pass rate (threshold: = 1.0)
    TestPassRate,
    /// Compilation success (threshold: = 1.0)
    Compilation,
    /// Placeholder count (threshold: = 0)
    PlaceholderCount,
}

impl GateType {
    /// Get the default threshold for this gate type
    pub fn default_threshold(&self) -> f64 {
        match self {
            Self::IntegrationF1 => 0.90,
            Self::PrivacyCompliance => 1.0,
            Self::ControlIntegration => 0.0, // Must be exactly 0
            Self::FixtureHitRate => 0.95,
            Self::InvariantViolations => 0.0, // Must be exactly 0
            Self::CodeCoverage => 0.80,
            Self::TestPassRate => 1.0,
            Self::Compilation => 1.0,
            Self::PlaceholderCount => 0.0, // Must be exactly 0
        }
    }

    /// Check if this gate uses "must be zero" logic
    pub fn is_zero_required(&self) -> bool {
        matches!(
            self,
            Self::ControlIntegration | Self::InvariantViolations | Self::PlaceholderCount
        )
    }

    /// Check if this gate uses "must be perfect" logic
    pub fn is_perfect_required(&self) -> bool {
        matches!(
            self,
            Self::PrivacyCompliance | Self::TestPassRate | Self::Compilation
        )
    }
}

/// Gate status for display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum GateStatus {
    /// Gate passed
    Passed,
    /// Gate passed with warnings
    PassedWithWarnings,
    /// Gate failed
    Failed,
    /// Gate check was skipped
    Skipped,
}

/// Fingerprints for reproducibility (from Distill)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Fingerprints {
    /// SHA-256 of the dataset
    pub dataset_sha256: Option<String>,
    /// SHA-256 of the model
    pub model_sha256: Option<String>,
    /// SHA-256 of the tokenizer
    pub tokenizer_sha256: Option<String>,
    /// SHA-256 of the tool registry
    pub tool_registry_sha256: Option<String>,
    /// SHA-256 of the invariant set
    pub invariant_set_sha256: Option<String>,
}

impl Fingerprints {
    /// Check if all required fingerprints are present
    pub fn is_complete(&self) -> bool {
        self.dataset_sha256.is_some()
            && self.model_sha256.is_some()
            && self.tool_registry_sha256.is_some()
    }

    /// Get list of missing fingerprints
    pub fn missing(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.dataset_sha256.is_none() {
            missing.push("dataset_sha256");
        }
        if self.model_sha256.is_none() {
            missing.push("model_sha256");
        }
        if self.tool_registry_sha256.is_none() {
            missing.push("tool_registry_sha256");
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_result_creation() {
        let gate = GateResult::new(GateType::IntegrationF1, 0.92, 0.90);
        assert!(gate.passed);
        assert!((gate.margin() - 0.02).abs() < 0.001);

        let failed = GateResult::new(GateType::IntegrationF1, 0.85, 0.90);
        assert!(!failed.passed);
        assert!(failed.margin() < 0.0);
    }

    #[test]
    fn test_gate_default_thresholds() {
        assert_eq!(GateType::IntegrationF1.default_threshold(), 0.90);
        assert_eq!(GateType::PrivacyCompliance.default_threshold(), 1.0);
        assert_eq!(GateType::InvariantViolations.default_threshold(), 0.0);
    }

    #[test]
    fn test_zero_required_gates() {
        assert!(GateType::InvariantViolations.is_zero_required());
        assert!(GateType::PlaceholderCount.is_zero_required());
        assert!(!GateType::IntegrationF1.is_zero_required());
    }

    #[test]
    fn test_fingerprints_completeness() {
        let incomplete = Fingerprints {
            dataset_sha256: Some("abc".to_string()),
            model_sha256: None,
            tokenizer_sha256: None,
            tool_registry_sha256: None,
            invariant_set_sha256: None,
        };
        assert!(!incomplete.is_complete());
        assert!(incomplete.missing().contains(&"model_sha256"));

        let complete = Fingerprints {
            dataset_sha256: Some("abc".to_string()),
            model_sha256: Some("def".to_string()),
            tokenizer_sha256: Some("ghi".to_string()),
            tool_registry_sha256: Some("jkl".to_string()),
            invariant_set_sha256: Some("mno".to_string()),
        };
        assert!(complete.is_complete());
        assert!(complete.missing().is_empty());
    }
}
