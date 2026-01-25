//! CAWS Gate Evaluation
//!
//! Implements the Coding-Agent Working Spec gate evaluation system.
//! Gates are hard thresholds that must be met before code proceeds.

use serde::{Deserialize, Serialize};
use v4_types::verdict::{Fingerprints, GateResult, GateType};

/// Result of evaluating all CAWS gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateVerdict {
    /// All gate results
    pub gates: Vec<GateResult>,
    /// Whether all gates passed
    pub all_passed: bool,
    /// Whether any hard gates failed (blocks everything)
    pub hard_failure: bool,
    /// Summary message
    pub summary: String,
    /// Timestamp of evaluation
    #[serde(with = "chrono::serde::ts_seconds")]
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
    /// Fingerprints used for this evaluation
    pub fingerprints: Fingerprints,
}

impl GateVerdict {
    /// Get all failed gates
    pub fn failed_gates(&self) -> Vec<&GateResult> {
        self.gates.iter().filter(|g| !g.passed).collect()
    }

    /// Get gates that barely passed (within 5% of threshold)
    pub fn marginal_gates(&self) -> Vec<&GateResult> {
        self.gates
            .iter()
            .filter(|g| g.passed && g.margin() < 0.05)
            .collect()
    }

    /// Check if a specific gate type passed
    pub fn gate_passed(&self, gate_type: GateType) -> bool {
        self.gates
            .iter()
            .find(|g| g.gate == gate_type)
            .map(|g| g.passed)
            .unwrap_or(false)
    }
}

/// A single gate evaluation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEvaluation {
    /// The gate type being evaluated
    pub gate_type: GateType,
    /// Raw metric value (before normalization)
    pub raw_value: f64,
    /// Normalized score (0.0 to 1.0)
    pub score: f64,
    /// Threshold for this gate
    pub threshold: f64,
    /// Whether this is a hard gate (failure blocks everything)
    pub is_hard_gate: bool,
    /// Additional context
    pub context: Option<String>,
}

impl GateEvaluation {
    /// Create a new gate evaluation
    pub fn new(gate_type: GateType, raw_value: f64) -> Self {
        let threshold = gate_type.default_threshold();
        let score = Self::normalize_score(gate_type, raw_value);
        let is_hard_gate = gate_type.is_zero_required() || gate_type.is_perfect_required();

        Self {
            gate_type,
            raw_value,
            score,
            threshold,
            is_hard_gate,
            context: None,
        }
    }

    /// Add context to this evaluation
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Normalize the raw value to a 0.0-1.0 score
    fn normalize_score(gate_type: GateType, raw_value: f64) -> f64 {
        match gate_type {
            // For "must be zero" gates, invert: 0 violations = 1.0 score
            GateType::InvariantViolations | GateType::PlaceholderCount | GateType::ControlIntegration => {
                if raw_value <= 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            // For percentage gates, value is already normalized
            _ => raw_value.clamp(0.0, 1.0),
        }
    }

    /// Check if this gate passes
    pub fn passes(&self) -> bool {
        if self.gate_type.is_zero_required() {
            // For zero-required gates, raw value must be exactly 0
            self.raw_value <= 0.0
        } else {
            self.score >= self.threshold
        }
    }

    /// Convert to a GateResult
    pub fn to_result(&self) -> GateResult {
        GateResult {
            gate: self.gate_type,
            score: self.score,
            threshold: self.threshold,
            passed: self.passes(),
            details: self.context.clone(),
        }
    }
}

/// CAWS Gate Evaluator
pub struct CAWSEvaluator {
    /// Custom thresholds (overrides defaults)
    custom_thresholds: std::collections::HashMap<GateType, f64>,
    /// Whether to fail on first hard gate failure
    fail_fast: bool,
}

impl CAWSEvaluator {
    /// Create a new evaluator with default thresholds
    pub fn new() -> Self {
        Self {
            custom_thresholds: std::collections::HashMap::new(),
            fail_fast: false,
        }
    }

    /// Enable fail-fast mode
    pub fn fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }

    /// Set a custom threshold for a gate type
    pub fn with_threshold(mut self, gate: GateType, threshold: f64) -> Self {
        self.custom_thresholds.insert(gate, threshold);
        self
    }

    /// Get the threshold for a gate type
    fn get_threshold(&self, gate: GateType) -> f64 {
        self.custom_thresholds
            .get(&gate)
            .copied()
            .unwrap_or_else(|| gate.default_threshold())
    }

    /// Evaluate a set of metrics against CAWS gates
    pub fn evaluate(&self, metrics: &CAWSMetrics, fingerprints: Fingerprints) -> GateVerdict {
        let mut gates = Vec::new();
        let mut hard_failure = false;

        // Evaluate each gate
        let evaluations = [
            GateEvaluation::new(GateType::IntegrationF1, metrics.integration_f1),
            GateEvaluation::new(GateType::PrivacyCompliance, metrics.privacy_compliance),
            GateEvaluation::new(GateType::ControlIntegration, metrics.control_violations as f64),
            GateEvaluation::new(GateType::FixtureHitRate, metrics.fixture_hit_rate),
            GateEvaluation::new(GateType::InvariantViolations, metrics.invariant_violations as f64),
            GateEvaluation::new(GateType::CodeCoverage, metrics.code_coverage),
            GateEvaluation::new(GateType::TestPassRate, metrics.test_pass_rate),
            GateEvaluation::new(GateType::Compilation, if metrics.compiles { 1.0 } else { 0.0 }),
            GateEvaluation::new(GateType::PlaceholderCount, metrics.placeholder_count as f64),
        ];

        for eval in evaluations {
            let mut result = eval.to_result();
            // Apply custom threshold if set
            result.threshold = self.get_threshold(eval.gate_type);
            // For zero-required gates, raw value must be exactly 0
            // For other gates, score must meet threshold
            result.passed = if eval.gate_type.is_zero_required() {
                eval.raw_value <= 0.0
            } else {
                eval.score >= result.threshold
            };

            if !result.passed && eval.is_hard_gate {
                hard_failure = true;
                if self.fail_fast {
                    gates.push(result);
                    break;
                }
            }
            gates.push(result);
        }

        let all_passed = gates.iter().all(|g| g.passed);
        let failed_count = gates.iter().filter(|g| !g.passed).count();

        let summary = if all_passed {
            "All CAWS gates passed".to_string()
        } else if hard_failure {
            format!("HARD FAILURE: {} gates failed including critical gates", failed_count)
        } else {
            format!("{} gates failed (soft failures only)", failed_count)
        };

        GateVerdict {
            gates,
            all_passed,
            hard_failure,
            summary,
            evaluated_at: chrono::Utc::now(),
            fingerprints,
        }
    }
}

impl Default for CAWSEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics input for CAWS gate evaluation
#[derive(Debug, Clone, Default)]
pub struct CAWSMetrics {
    /// Integration F1 score (0.0 to 1.0)
    pub integration_f1: f64,
    /// Privacy compliance rate (0.0 to 1.0)
    pub privacy_compliance: f64,
    /// Number of control integration violations
    pub control_violations: u32,
    /// Fixture hit rate (0.0 to 1.0)
    pub fixture_hit_rate: f64,
    /// Number of invariant violations
    pub invariant_violations: u32,
    /// Code coverage (0.0 to 1.0)
    pub code_coverage: f64,
    /// Test pass rate (0.0 to 1.0)
    pub test_pass_rate: f64,
    /// Whether code compiles
    pub compiles: bool,
    /// Number of placeholders in code
    pub placeholder_count: u32,
}

impl CAWSMetrics {
    /// Create a perfect metrics set (all gates pass)
    pub fn perfect() -> Self {
        Self {
            integration_f1: 0.95,
            privacy_compliance: 1.0,
            control_violations: 0,
            fixture_hit_rate: 0.98,
            invariant_violations: 0,
            code_coverage: 0.85,
            test_pass_rate: 1.0,
            compiles: true,
            placeholder_count: 0,
        }
    }

    /// Create a failing metrics set
    pub fn failing() -> Self {
        Self {
            integration_f1: 0.5,
            privacy_compliance: 0.8,
            control_violations: 3,
            fixture_hit_rate: 0.7,
            invariant_violations: 5,
            code_coverage: 0.4,
            test_pass_rate: 0.6,
            compiles: false,
            placeholder_count: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_fingerprints() -> Fingerprints {
        Fingerprints {
            dataset_sha256: None,
            model_sha256: None,
            tokenizer_sha256: None,
            tool_registry_sha256: None,
            invariant_set_sha256: None,
        }
    }

    #[test]
    fn test_perfect_metrics_pass() {
        let evaluator = CAWSEvaluator::new();
        let verdict = evaluator.evaluate(&CAWSMetrics::perfect(), empty_fingerprints());

        assert!(verdict.all_passed);
        assert!(!verdict.hard_failure);
        assert!(verdict.failed_gates().is_empty());
    }

    #[test]
    fn test_failing_metrics_fail() {
        let evaluator = CAWSEvaluator::new();
        let verdict = evaluator.evaluate(&CAWSMetrics::failing(), empty_fingerprints());

        assert!(!verdict.all_passed);
        assert!(verdict.hard_failure); // Compilation fails, which is a hard gate
        assert!(!verdict.failed_gates().is_empty());
    }

    #[test]
    fn test_zero_required_gates() {
        let evaluator = CAWSEvaluator::new();

        // Zero violations should pass
        let mut metrics = CAWSMetrics::perfect();
        metrics.invariant_violations = 0;
        let verdict = evaluator.evaluate(&metrics, empty_fingerprints());
        assert!(verdict.gate_passed(GateType::InvariantViolations));

        // Any violations should fail
        metrics.invariant_violations = 1;
        let verdict = evaluator.evaluate(&metrics, empty_fingerprints());
        assert!(!verdict.gate_passed(GateType::InvariantViolations));
    }

    #[test]
    fn test_custom_threshold() {
        let evaluator = CAWSEvaluator::new()
            .with_threshold(GateType::CodeCoverage, 0.95); // Stricter than default 0.80

        let metrics = CAWSMetrics::perfect(); // Has 0.85 coverage
        let verdict = evaluator.evaluate(&metrics, empty_fingerprints());

        // Should fail with stricter threshold
        assert!(!verdict.gate_passed(GateType::CodeCoverage));
    }

    #[test]
    fn test_gate_evaluation_normalization() {
        // Regular percentage gates
        let eval = GateEvaluation::new(GateType::IntegrationF1, 0.92);
        assert!((eval.score - 0.92).abs() < 0.001);
        assert!(eval.passes());

        // Zero-required gates (violations count)
        let eval = GateEvaluation::new(GateType::InvariantViolations, 0.0);
        assert!((eval.score - 1.0).abs() < 0.001); // Normalized to 1.0
        assert!(eval.passes());

        let eval = GateEvaluation::new(GateType::InvariantViolations, 5.0);
        assert!((eval.score - 0.0).abs() < 0.001); // Normalized to 0.0
        assert!(!eval.passes());
    }

    #[test]
    fn test_marginal_gates() {
        let evaluator = CAWSEvaluator::new();

        let mut metrics = CAWSMetrics::perfect();
        metrics.integration_f1 = 0.91; // Just above 0.90 threshold
        metrics.code_coverage = 0.81;  // Just above 0.80 threshold

        let verdict = evaluator.evaluate(&metrics, empty_fingerprints());
        let marginal = verdict.marginal_gates();

        // Both should be flagged as marginal
        assert!(marginal.len() >= 2);
    }

    #[test]
    fn test_fail_fast_mode() {
        let evaluator = CAWSEvaluator::new().fail_fast();
        let verdict = evaluator.evaluate(&CAWSMetrics::failing(), empty_fingerprints());

        // Should stop after first hard failure
        assert!(verdict.hard_failure);
        // In fail-fast mode, we should have fewer results
        assert!(verdict.gates.len() < 9);
    }
}
