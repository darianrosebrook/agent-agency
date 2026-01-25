//! Pre-execution CAWS Gate Enforcement
//!
//! Provides gate checking before execution authorization.

use serde::{Deserialize, Serialize};
use v4_governance::gates::{CAWSEvaluator, CAWSMetrics, GateVerdict};
use v4_types::verdict::{Fingerprints, GateType};

/// Gate check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheckResult {
    /// The gate verdict
    pub verdict: GateVerdict,
    /// Whether pre-execution can proceed
    pub can_proceed: bool,
    /// Blocking issues that prevent execution
    pub blocking_issues: Vec<String>,
    /// Warnings that don't block execution
    pub warnings: Vec<String>,
    /// Timestamp of check
    #[serde(with = "chrono::serde::ts_seconds")]
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

impl GateCheckResult {
    /// Check if all gates passed
    pub fn all_passed(&self) -> bool {
        self.verdict.all_passed
    }

    /// Check if there's a hard failure
    pub fn has_hard_failure(&self) -> bool {
        self.verdict.hard_failure
    }

    /// Get list of failed gates
    pub fn failed_gates(&self) -> Vec<GateType> {
        self.verdict
            .gates
            .iter()
            .filter(|g| !g.passed)
            .map(|g| g.gate)
            .collect()
    }

    /// Get list of passed gates
    pub fn passed_gates(&self) -> Vec<GateType> {
        self.verdict
            .gates
            .iter()
            .filter(|g| g.passed)
            .map(|g| g.gate)
            .collect()
    }
}

/// Pre-execution gate checker
pub struct GateChecker {
    /// CAWS evaluator
    evaluator: CAWSEvaluator,
    /// Whether to block on any gate failure
    strict_mode: bool,
}

impl GateChecker {
    /// Create a new gate checker
    pub fn new() -> Self {
        Self {
            evaluator: CAWSEvaluator::new(),
            strict_mode: false,
        }
    }

    /// Enable strict mode (any failure blocks)
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Enable fail-fast mode
    pub fn fail_fast(mut self) -> Self {
        self.evaluator = self.evaluator.fail_fast();
        self
    }

    /// Set a custom threshold for a gate
    pub fn with_threshold(mut self, gate: GateType, threshold: f64) -> Self {
        self.evaluator = self.evaluator.with_threshold(gate, threshold);
        self
    }

    /// Check gates with provided metrics
    pub fn check(&self, metrics: &CAWSMetrics, fingerprints: Fingerprints) -> GateCheckResult {
        let verdict = self.evaluator.evaluate(metrics, fingerprints);

        let mut blocking_issues = Vec::new();
        let mut warnings = Vec::new();

        for gate in &verdict.gates {
            if !gate.passed {
                let message = format!(
                    "{:?}: score {:.2} below threshold {:.2}",
                    gate.gate, gate.score, gate.threshold
                );

                // Hard gates always block
                if gate.gate.is_zero_required() || gate.gate.is_perfect_required() {
                    blocking_issues.push(message);
                } else if self.strict_mode {
                    blocking_issues.push(message);
                } else {
                    warnings.push(message);
                }
            }
        }

        // Can proceed if no blocking issues (or all passed)
        let can_proceed = blocking_issues.is_empty();

        GateCheckResult {
            verdict,
            can_proceed,
            blocking_issues,
            warnings,
            checked_at: chrono::Utc::now(),
        }
    }

    /// Check gates with default (perfect) metrics
    pub fn check_default(&self) -> GateCheckResult {
        self.check(&CAWSMetrics::perfect(), default_fingerprints())
    }

    /// Create metrics from execution evidence
    pub fn metrics_from_evidence(evidence: &ExecutionEvidence) -> CAWSMetrics {
        CAWSMetrics {
            integration_f1: evidence.integration_f1.unwrap_or(0.95),
            privacy_compliance: if evidence.privacy_violations == 0 {
                1.0
            } else {
                0.0
            },
            control_violations: evidence.control_violations,
            fixture_hit_rate: evidence.fixture_hit_rate.unwrap_or(0.95),
            invariant_violations: evidence.invariant_violations,
            code_coverage: evidence.code_coverage.unwrap_or(0.80),
            test_pass_rate: evidence.test_pass_rate.unwrap_or(1.0),
            compiles: evidence.compiles,
            placeholder_count: evidence.placeholder_count,
        }
    }
}

impl Default for GateChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default fingerprints
pub fn default_fingerprints() -> Fingerprints {
    Fingerprints {
        dataset_sha256: None,
        model_sha256: None,
        tokenizer_sha256: None,
        tool_registry_sha256: None,
        invariant_set_sha256: None,
    }
}

/// Evidence for gate checking
#[derive(Debug, Clone, Default)]
pub struct ExecutionEvidence {
    /// Integration F1 score
    pub integration_f1: Option<f64>,
    /// Number of privacy violations
    pub privacy_violations: u32,
    /// Number of control violations
    pub control_violations: u32,
    /// Fixture hit rate
    pub fixture_hit_rate: Option<f64>,
    /// Number of invariant violations
    pub invariant_violations: u32,
    /// Code coverage
    pub code_coverage: Option<f64>,
    /// Test pass rate
    pub test_pass_rate: Option<f64>,
    /// Whether code compiles
    pub compiles: bool,
    /// Number of placeholders
    pub placeholder_count: u32,
}

impl ExecutionEvidence {
    /// Create perfect evidence (all gates pass)
    pub fn perfect() -> Self {
        Self {
            integration_f1: Some(0.95),
            privacy_violations: 0,
            control_violations: 0,
            fixture_hit_rate: Some(0.98),
            invariant_violations: 0,
            code_coverage: Some(0.85),
            test_pass_rate: Some(1.0),
            compiles: true,
            placeholder_count: 0,
        }
    }

    /// Create failing evidence
    pub fn failing() -> Self {
        Self {
            integration_f1: Some(0.5),
            privacy_violations: 1,
            control_violations: 2,
            fixture_hit_rate: Some(0.6),
            invariant_violations: 3,
            code_coverage: Some(0.4),
            test_pass_rate: Some(0.5),
            compiles: false,
            placeholder_count: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_check_perfect() {
        let checker = GateChecker::new();
        let result = checker.check(&CAWSMetrics::perfect(), default_fingerprints());

        assert!(result.all_passed());
        assert!(result.can_proceed);
        assert!(result.blocking_issues.is_empty());
    }

    #[test]
    fn test_gate_check_failing() {
        let checker = GateChecker::new();
        let result = checker.check(&CAWSMetrics::failing(), default_fingerprints());

        assert!(!result.all_passed());
        assert!(!result.can_proceed);
        assert!(!result.blocking_issues.is_empty());
    }

    #[test]
    fn test_strict_mode() {
        let checker = GateChecker::new().strict();

        // Create metrics that fail a soft gate
        let mut metrics = CAWSMetrics::perfect();
        metrics.code_coverage = 0.75; // Below 0.80 threshold

        let result = checker.check(&metrics, default_fingerprints());

        // In strict mode, soft failures also block
        assert!(!result.can_proceed);
    }

    #[test]
    fn test_non_strict_soft_failure() {
        let checker = GateChecker::new(); // Not strict

        // Create metrics that fail a soft gate
        let mut metrics = CAWSMetrics::perfect();
        metrics.code_coverage = 0.75; // Below 0.80 threshold

        let result = checker.check(&metrics, default_fingerprints());

        // In non-strict mode, soft failures become warnings
        assert!(result.can_proceed);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_hard_gate_always_blocks() {
        let checker = GateChecker::new(); // Not strict

        // Create metrics that fail a hard gate
        let mut metrics = CAWSMetrics::perfect();
        metrics.compiles = false;

        let result = checker.check(&metrics, default_fingerprints());

        // Hard gates always block
        assert!(!result.can_proceed);
        assert!(result.has_hard_failure());
    }

    #[test]
    fn test_evidence_conversion() {
        let evidence = ExecutionEvidence::perfect();
        let metrics = GateChecker::metrics_from_evidence(&evidence);

        assert!(metrics.compiles);
        assert_eq!(metrics.invariant_violations, 0);
        assert!(metrics.test_pass_rate >= 0.99);
    }

    #[test]
    fn test_passed_and_failed_gates() {
        let checker = GateChecker::new();

        let mut metrics = CAWSMetrics::perfect();
        metrics.compiles = false; // Fail one gate

        let result = checker.check(&metrics, default_fingerprints());

        assert!(result.failed_gates().contains(&GateType::Compilation));
        assert!(result.passed_gates().contains(&GateType::IntegrationF1));
    }
}
