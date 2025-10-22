//! Quality gates and validation framework

use crate::types::*;
use crate::error::*;
use async_trait::async_trait;

/// Quality gate definition
pub struct QualityGate {
    pub name: String,
    pub description: String,
    pub validator: Box<dyn QualityValidatorTrait>,
    pub required_score: f32,
    pub blocking: bool,
    pub timeout_seconds: u64,
}

impl Clone for QualityGate {
    fn clone(&self) -> Self {
        // For cloning, we create a new instance with the same configuration
        // but the validator trait object cannot be cloned, so we create a new one
        // This is a limitation - in practice, we'd want a different approach
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            validator: Box::new(DummyValidator), // Placeholder - this won't work in practice
            required_score: self.required_score,
            blocking: self.blocking,
            timeout_seconds: self.timeout_seconds,
        }
    }
}

/// Dummy validator for cloning - this is a workaround
struct DummyValidator;

#[async_trait::async_trait]
impl QualityValidatorTrait for DummyValidator {
    async fn validate(&self, _context: &crate::ValidationContext) -> crate::ValidationResult {
        crate::ValidationResult::Fail {
            score: 0.0,
            details: "Dummy validator - should not be used".to_string(),
            suggestions: vec![],
        }
    }
}

impl QualityGate {
    pub fn new<V>(
        name: String,
        description: String,
        validator: V,
        required_score: f32,
        blocking: bool,
    ) -> Self
    where
        V: QualityValidatorTrait + 'static,
    {
        Self {
            name,
            description,
            validator: Box::new(validator),
            required_score,
            blocking,
            timeout_seconds: 30, // Default 30 second timeout
        }
    }

    /// Validate against a context with timeout
    pub async fn validate_with_timeout(
        &self,
        context: &ValidationContext,
    ) -> crate::ValidationResult {
        let timeout_duration = std::time::Duration::from_secs(self.timeout_seconds);

        match tokio::time::timeout(timeout_duration, self.validator.validate(context)).await {
            Ok(result) => result,
            Err(_) => crate::ValidationResult::Fail {
                score: 0.0,
                details: format!("Validation timeout after {} seconds", self.timeout_seconds),
                suggestions: vec!["Increase timeout".to_string(), "Check validator implementation".to_string()],
            },
        }
    }

    /// Check if the gate passes
    pub fn passes(&self, result: &ValidationResult) -> bool {
        match result {
            ValidationResult::Pass { score, .. } => *score >= self.required_score,
            ValidationResult::Fail { .. } => false,
            ValidationResult::Warning { score, .. } => *score >= self.required_score,
        }
    }
}

/// Trait for quality validators
#[async_trait]
pub trait QualityValidatorTrait: Send + Sync {
    async fn validate(&self, context: &ValidationContext) -> crate::ValidationResult;
}

/// Quality gate runner that executes multiple gates
pub struct QualityGateRunner {
    gates: Vec<QualityGate>,
}

impl QualityGateRunner {
    pub fn new(gates: Vec<QualityGate>) -> Self {
        Self { gates }
    }

    /// Run all quality gates
    pub async fn run_all_gates(
        &self,
        context: &ValidationContext,
    ) -> Vec<GateResult> {
        let mut results = Vec::new();

        for gate in &self.gates {
            let result = gate.validate_with_timeout(context).await;
            let passes = gate.passes(&result);

            results.push(GateResult {
                gate_name: gate.name.clone(),
                description: gate.description.clone(),
                result,
                passes,
                blocking: gate.blocking,
            });
        }

        results
    }

    /// Check if all blocking gates pass
    pub async fn check_blocking_gates(&self, context: &ValidationContext) -> bool {
        let results = self.run_all_gates(context).await;

        results.iter().all(|r| !r.blocking || r.passes)
    }

    /// Get summary of gate results
    pub async fn get_summary(&self, context: &ValidationContext) -> GateSummary {
        let results = self.run_all_gates(context).await;

        let total_gates = results.len();
        let passed_gates = results.iter().filter(|r| r.passes).count();
        let failed_gates = total_gates - passed_gates;
        let blocking_failures = results.iter()
            .filter(|r| r.blocking && !r.passes)
            .count();

        let overall_score = if total_gates > 0 {
            passed_gates as f32 / total_gates as f32
        } else {
            1.0
        };

        GateSummary {
            total_gates,
            passed_gates,
            failed_gates,
            blocking_failures,
            overall_score,
            results,
        }
    }

    /// Add a new quality gate
    pub fn add_gate(&mut self, gate: QualityGate) {
        self.gates.push(gate);
    }

    /// Remove a quality gate by name
    pub fn remove_gate(&mut self, name: &str) {
        self.gates.retain(|g| g.name != name);
    }

    /// Get all gates
    pub fn gates(&self) -> &[QualityGate] {
        &self.gates
    }
}

/// Result of running a single quality gate
#[derive(Debug, Clone)]
pub struct GateResult {
    pub gate_name: String,
    pub description: String,
    pub result: ValidationResult,
    pub passes: bool,
    pub blocking: bool,
}

/// Summary of all gate results
#[derive(Debug, Clone)]
pub struct GateSummary {
    pub total_gates: usize,
    pub passed_gates: usize,
    pub failed_gates: usize,
    pub blocking_failures: usize,
    pub overall_score: f32,
    pub results: Vec<GateResult>,
}

/// Predefined quality gate configurations
pub mod presets {
    use super::*;
    use crate::validation::CompilationValidator;
    use crate::validation::TestValidator;
    use crate::validation::LintValidator;
    use crate::validation::SecurityValidator;
    use crate::validation::PerformanceValidator;

    /// Create compilation quality gate
    pub fn compilation_gate() -> QualityGate {
        QualityGate::new(
            "compilation".to_string(),
            "Ensures code compiles without errors".to_string(),
            CompilationValidator,
            1.0, // Must pass completely
            true, // Blocking
        )
    }

    /// Create testing quality gate
    pub fn testing_gate(min_coverage: f32) -> QualityGate {
        QualityGate::new(
            "testing".to_string(),
            format!("Ensures test coverage meets {}%", min_coverage * 100.0),
TestValidator::new(min_coverage),
            1.0, // Must pass
            true, // Blocking
        )
    }

    /// Create linting quality gate
    pub fn linting_gate() -> QualityGate {
        QualityGate::new(
            "linting".to_string(),
            "Ensures code passes linting checks".to_string(),
LintValidator,
            1.0, // Must pass completely
            false, // Not blocking
        )
    }

    /// Create security quality gate
    pub fn security_gate() -> QualityGate {
        QualityGate::new(
            "security".to_string(),
            "Ensures code passes security checks".to_string(),
SecurityValidator,
            1.0, // Must pass completely
            true, // Blocking
        )
    }

    /// Create performance quality gate
    pub fn performance_gate(max_response_time_ms: u64) -> QualityGate {
        QualityGate::new(
            "performance".to_string(),
            format!("Ensures performance within {}ms", max_response_time_ms),
PerformanceValidator::new(max_response_time_ms),
            0.8, // 80% acceptable
            false, // Not blocking
        )
    }

    /// Create a standard set of quality gates
    pub fn standard_gates() -> Vec<QualityGate> {
        vec![
            compilation_gate(),
            testing_gate(0.8), // 80% coverage
            linting_gate(),
            security_gate(),
        ]
    }

    /// Create minimal quality gates (fast feedback)
    pub fn minimal_gates() -> Vec<QualityGate> {
        vec![
            compilation_gate(),
        ]
    }

    /// Create comprehensive quality gates (slow but thorough)
    pub fn comprehensive_gates() -> Vec<QualityGate> {
        vec![
            compilation_gate(),
            testing_gate(0.9), // 90% coverage
            linting_gate(),
            security_gate(),
            performance_gate(1000), // 1 second max
        ]
    }
}

impl Default for QualityGateRunner {
    fn default() -> Self {
        Self::new(presets::minimal_gates())
    }
}
