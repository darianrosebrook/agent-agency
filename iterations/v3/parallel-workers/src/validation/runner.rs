//! Quality validation runner with parallel execution

use crate::types::*;
use crate::error::*;
use super::gates::*;

/// Validation runner that executes quality gates in parallel
pub struct ValidationRunner {
    runner: QualityGateRunner,
    max_parallel_validations: usize,
}

impl ValidationRunner {
    pub fn new(max_parallel_validations: usize) -> Self {
        Self {
            runner: QualityGateRunner::default(),
            max_parallel_validations,
        }
    }

    /// Run all validations in parallel
    pub async fn run_parallel(
        &self,
        context: &ValidationContext,
    ) -> ValidationResult<ValidationReport> {
        let gates = self.runner.gates().to_vec();
        let mut handles = Vec::new();

        // Create chunks for parallel execution
        let chunks = gates.chunks(self.max_parallel_validations);

        for chunk in chunks {
            let chunk_context = context.clone();
            let chunk_gates = chunk.to_vec();

            let handle = tokio::spawn(async move {
                let mut results = Vec::new();

                for gate in chunk_gates {
                    let result = gate.validate_with_timeout(&chunk_context).await?;
                    let passes = gate.passes(&result);

                    results.push(GateResult {
                        gate_name: gate.name.clone(),
                        description: gate.description.clone(),
                        result,
                        passes,
                        blocking: gate.blocking,
                    });
                }

                Ok::<Vec<GateResult>, ValidationError>(results)
            });

            handles.push(handle);
        }

        // Collect results
        let mut all_results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(results)) => all_results.extend(results),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(ValidationError::ExternalToolFailure {
                    tool_name: "tokio".to_string(),
                    message: format!("Task join error: {}", e),
                }),
            }
        }

        // Generate report
        let report = self.generate_report(all_results, context)?;

        Ok(report)
    }

    /// Run validations sequentially (fallback for debugging)
    pub async fn run_sequential(
        &self,
        context: &ValidationContext,
    ) -> ValidationResult<ValidationReport> {
        let summary = self.runner.get_summary(context).await?;
        let report = ValidationReport {
            context: context.clone(),
            summary,
            execution_time: std::time::Duration::from_secs(0), // Would need timing
            recommendations: self.generate_recommendations(&summary.results),
        };

        Ok(report)
    }

    /// Validate that all blocking gates pass
    pub async fn validate_blocking_gates(
        &self,
        context: &ValidationContext,
    ) -> ValidationResult<bool> {
        self.runner.check_blocking_gates(context).await
    }

    /// Generate detailed validation report
    fn generate_report(
        &self,
        results: Vec<GateResult>,
        context: &ValidationContext,
    ) -> ValidationResult<ValidationReport> {
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

        let summary = GateSummary {
            total_gates,
            passed_gates,
            failed_gates,
            blocking_failures,
            overall_score,
            results,
        };

        let recommendations = self.generate_recommendations(&summary.results);

        Ok(ValidationReport {
            context: context.clone(),
            summary,
            execution_time: std::time::Duration::from_secs(0), // TODO: Add timing
            recommendations,
        })
    }

    /// Generate recommendations based on validation results
    fn generate_recommendations(&self, results: &[GateResult]) -> Vec<String> {
        let mut recommendations = Vec::new();

        for result in results {
            if !result.passes {
                match result.result {
                    ValidationResult::Fail { ref suggestions, .. } |
                    ValidationResult::Warning { ref suggestions, .. } => {
                        recommendations.extend(suggestions.iter().cloned());
                    }
                    _ => {}
                }

                // Add gate-specific recommendations
                match result.gate_name.as_str() {
                    "compilation" => {
                        recommendations.push("Fix compilation errors before proceeding".to_string());
                    }
                    "testing" => {
                        recommendations.push("Add or fix failing tests".to_string());
                        recommendations.push("Consider increasing test coverage".to_string());
                    }
                    "linting" => {
                        recommendations.push("Address code quality issues".to_string());
                    }
                    "security" => {
                        recommendations.push("Address security vulnerabilities immediately".to_string());
                    }
                    "performance" => {
                        recommendations.push("Optimize performance bottlenecks".to_string());
                    }
                    _ => {}
                }
            }
        }

        // Remove duplicates and sort
        recommendations.sort();
        recommendations.dedup();

        recommendations
    }

    /// Add a custom quality gate
    pub fn add_gate(&mut self, gate: QualityGate) {
        self.runner.add_gate(gate);
    }

    /// Set standard gates based on tier
    pub fn set_standard_gates(&mut self, tier: ValidationTier) {
        let gates = match tier {
            ValidationTier::Minimal => super::gates::presets::minimal_gates(),
            ValidationTier::Standard => super::gates::presets::standard_gates(),
            ValidationTier::Comprehensive => super::gates::presets::comprehensive_gates(),
        };

        self.runner = QualityGateRunner::new(gates);
    }
}

/// Validation tiers for different quality requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationTier {
    /// Fast feedback - compilation only
    Minimal,
    /// Balanced - compilation, testing, linting, security
    Standard,
    /// Thorough - all gates including performance
    Comprehensive,
}

/// Comprehensive validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub context: ValidationContext,
    pub summary: GateSummary,
    pub execution_time: std::time::Duration,
    pub recommendations: Vec<String>,
}

impl ValidationReport {
    /// Check if validation passed (all blocking gates)
    pub fn passed(&self) -> bool {
        self.summary.blocking_failures == 0
    }

    /// Get overall quality score
    pub fn quality_score(&self) -> f32 {
        self.summary.overall_score
    }

    /// Get failed gates
    pub fn failed_gates(&self) -> Vec<&GateResult> {
        self.summary.results.iter()
            .filter(|r| !r.passes)
            .collect()
    }

    /// Get blocking failures
    pub fn blocking_failures(&self) -> Vec<&GateResult> {
        self.summary.results.iter()
            .filter(|r| r.blocking && !r.passes)
            .collect()
    }

    /// Generate human-readable summary
    pub fn summary_text(&self) -> String {
        format!(
            "Validation Report: {}/{} gates passed ({:.1}%)\nBlocking failures: {}\nRecommendations:\n{}",
            self.summary.passed_gates,
            self.summary.total_gates,
            self.quality_score() * 100.0,
            self.summary.blocking_failures,
            if self.recommendations.is_empty() {
                "  - No specific recommendations".to_string()
            } else {
                self.recommendations.iter()
                    .map(|r| format!("  - {}", r))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    }
}

/// Validation pipeline for sequential gate execution
pub struct ValidationPipeline {
    gates: Vec<QualityGate>,
}

impl ValidationPipeline {
    pub fn new(gates: Vec<QualityGate>) -> Self {
        Self { gates }
    }

    /// Run gates in sequence, stopping on blocking failures
    pub async fn run_with_early_exit(
        &self,
        context: &ValidationContext,
    ) -> ValidationResult<ValidationReport> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        for gate in &self.gates {
            let result = gate.validate_with_timeout(context).await?;
            let passes = gate.passes(&result);

            results.push(GateResult {
                gate_name: gate.name.clone(),
                description: gate.description.clone(),
                result,
                passes,
                blocking: gate.blocking,
            });

            // Stop on blocking failure
            if gate.blocking && !passes {
                break;
            }
        }

        let execution_time = start_time.elapsed();
        let summary = create_summary_from_results(results);
        let recommendations = generate_recommendations_from_results(&summary.results);

        Ok(ValidationReport {
            context: context.clone(),
            summary,
            execution_time,
            recommendations,
        })
    }
}

/// Helper function to create summary from results
fn create_summary_from_results(results: Vec<GateResult>) -> GateSummary {
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

/// Helper function to generate recommendations
fn generate_recommendations_from_results(results: &[GateResult]) -> Vec<String> {
    let mut recommendations = Vec::new();

    for result in results {
        if !result.passes {
            match &result.result {
                ValidationResult::Fail { suggestions, .. } |
                ValidationResult::Warning { suggestions, .. } => {
                    recommendations.extend(suggestions.iter().cloned());
                }
                _ => {}
            }
        }
    }

    recommendations.sort();
    recommendations.dedup();
    recommendations
}

impl Default for ValidationRunner {
    fn default() -> Self {
        Self::new(4) // Run 4 validations in parallel by default
    }
}
