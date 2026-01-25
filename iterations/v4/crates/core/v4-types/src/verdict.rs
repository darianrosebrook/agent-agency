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

    fn make_gate_result(gate: GateType, score: f64, threshold: f64) -> GateResult {
        GateResult::new(gate, score, threshold)
    }

    fn make_quality_report(gates: Vec<GateResult>) -> QualityReport {
        QualityReport {
            id: "report-1".to_string(),
            task_id: "task-1".to_string(),
            gates,
            passed: true,
            overall_score: 0.9,
            timestamp: chrono::Utc::now(),
            fingerprints: Fingerprints {
                dataset_sha256: Some("abc".to_string()),
                model_sha256: Some("def".to_string()),
                tokenizer_sha256: Some("ghi".to_string()),
                tool_registry_sha256: Some("jkl".to_string()),
                invariant_set_sha256: Some("mno".to_string()),
            },
        }
    }

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
    fn test_gate_result_with_details() {
        let gate = GateResult::with_details(
            GateType::IntegrationF1,
            0.92,
            0.90,
            "F1 score is good".to_string(),
        );
        assert!(gate.passed);
        assert_eq!(gate.details, Some("F1 score is good".to_string()));
    }

    #[test]
    fn test_gate_result_boundary() {
        // Exactly at threshold should pass (>= not >)
        let at_threshold = GateResult::new(GateType::IntegrationF1, 0.90, 0.90);
        assert!(at_threshold.passed);

        // Just below should fail
        let below = GateResult::new(GateType::IntegrationF1, 0.899, 0.90);
        assert!(!below.passed);
    }

    #[test]
    fn test_gate_with_details_boundary() {
        // Test that with_details also uses >= for passed check
        let at_threshold = GateResult::with_details(
            GateType::IntegrationF1,
            0.90,
            0.90,
            "At boundary".to_string(),
        );
        assert!(at_threshold.passed);

        let below = GateResult::with_details(
            GateType::IntegrationF1,
            0.899,
            0.90,
            "Below boundary".to_string(),
        );
        assert!(!below.passed);
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
        assert!(GateType::ControlIntegration.is_zero_required());
        assert!(!GateType::IntegrationF1.is_zero_required());
        assert!(!GateType::PrivacyCompliance.is_zero_required());
    }

    #[test]
    fn test_perfect_required_gates() {
        // Gates that must be 1.0
        assert!(GateType::PrivacyCompliance.is_perfect_required());
        assert!(GateType::TestPassRate.is_perfect_required());
        assert!(GateType::Compilation.is_perfect_required());

        // Gates that don't require perfection
        assert!(!GateType::IntegrationF1.is_perfect_required());
        assert!(!GateType::CodeCoverage.is_perfect_required());
        assert!(!GateType::FixtureHitRate.is_perfect_required());
        assert!(!GateType::InvariantViolations.is_perfect_required());
    }

    #[test]
    fn test_quality_report_all_gates_passed() {
        // All passing
        let all_pass = make_quality_report(vec![
            make_gate_result(GateType::IntegrationF1, 0.95, 0.90),
            make_gate_result(GateType::CodeCoverage, 0.85, 0.80),
        ]);
        assert!(all_pass.all_gates_passed());

        // One failing
        let one_fails = make_quality_report(vec![
            make_gate_result(GateType::IntegrationF1, 0.95, 0.90),
            make_gate_result(GateType::CodeCoverage, 0.75, 0.80), // Fails
        ]);
        assert!(!one_fails.all_gates_passed());

        // Empty gates (edge case)
        let empty = make_quality_report(vec![]);
        assert!(empty.all_gates_passed()); // vacuously true
    }

    #[test]
    fn test_quality_report_failed_gates() {
        let report = make_quality_report(vec![
            make_gate_result(GateType::IntegrationF1, 0.95, 0.90), // Pass
            make_gate_result(GateType::CodeCoverage, 0.75, 0.80), // Fail
            make_gate_result(GateType::TestPassRate, 0.90, 1.0),  // Fail
        ]);

        let failed = report.failed_gates();
        assert_eq!(failed.len(), 2);

        // Check that the correct gates are in the list
        assert!(failed.iter().any(|g| g.gate == GateType::CodeCoverage));
        assert!(failed.iter().any(|g| g.gate == GateType::TestPassRate));
    }

    #[test]
    fn test_quality_report_failed_gates_empty_when_all_pass() {
        let report = make_quality_report(vec![
            make_gate_result(GateType::IntegrationF1, 0.95, 0.90),
            make_gate_result(GateType::CodeCoverage, 0.85, 0.80),
        ]);

        let failed = report.failed_gates();
        assert!(failed.is_empty());
    }

    #[test]
    fn test_quality_report_lowest_gate() {
        let report = make_quality_report(vec![
            make_gate_result(GateType::IntegrationF1, 0.95, 0.90),
            make_gate_result(GateType::CodeCoverage, 0.75, 0.80), // Lowest
            make_gate_result(GateType::TestPassRate, 0.90, 1.0),
        ]);

        let lowest = report.lowest_gate();
        assert!(lowest.is_some());
        assert_eq!(lowest.unwrap().gate, GateType::CodeCoverage);
        assert!((lowest.unwrap().score - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_quality_report_lowest_gate_empty() {
        let report = make_quality_report(vec![]);
        assert!(report.lowest_gate().is_none());
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

    #[test]
    fn test_fingerprints_is_complete_requires_all() {
        // Missing dataset
        let no_dataset = Fingerprints {
            dataset_sha256: None,
            model_sha256: Some("def".to_string()),
            tokenizer_sha256: Some("ghi".to_string()),
            tool_registry_sha256: Some("jkl".to_string()),
            invariant_set_sha256: Some("mno".to_string()),
        };
        assert!(!no_dataset.is_complete());

        // Missing model
        let no_model = Fingerprints {
            dataset_sha256: Some("abc".to_string()),
            model_sha256: None,
            tokenizer_sha256: Some("ghi".to_string()),
            tool_registry_sha256: Some("jkl".to_string()),
            invariant_set_sha256: Some("mno".to_string()),
        };
        assert!(!no_model.is_complete());

        // Missing tool_registry
        let no_registry = Fingerprints {
            dataset_sha256: Some("abc".to_string()),
            model_sha256: Some("def".to_string()),
            tokenizer_sha256: Some("ghi".to_string()),
            tool_registry_sha256: None,
            invariant_set_sha256: Some("mno".to_string()),
        };
        assert!(!no_registry.is_complete());

        // Only required fields (tokenizer and invariant_set not required)
        let minimal_complete = Fingerprints {
            dataset_sha256: Some("abc".to_string()),
            model_sha256: Some("def".to_string()),
            tokenizer_sha256: None,
            tool_registry_sha256: Some("jkl".to_string()),
            invariant_set_sha256: None,
        };
        assert!(minimal_complete.is_complete());
    }
}
