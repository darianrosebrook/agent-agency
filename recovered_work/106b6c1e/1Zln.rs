//! Quality Gate Orchestrator
//!
//! Coordinates concurrent execution of quality gates with budgets,
//! tiered thresholds, and deterministic pass/fail decisions.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::time::timeout;

use crate::planning::types::{ExecutionArtifacts, RiskTier};
use super::gates::{QualityGate, QualityGateResult, GateStatus, QualityThresholds};

/// Quality gate orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateOrchestratorConfig {
    /// Maximum concurrent gates
    pub max_concurrent_gates: usize,
    /// Overall timeout for all gates (seconds)
    pub overall_timeout_seconds: u64,
    /// Individual gate timeout (seconds)
    pub gate_timeout_seconds: u64,
    /// Enable parallel execution
    pub enable_parallel: bool,
    /// Stop on first failure
    pub stop_on_first_failure: bool,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
}

/// Quality report summarizing all gate results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub task_id: uuid::Uuid,
    pub risk_tier: RiskTier,
    pub overall_status: GateStatus,
    pub overall_score: f64,
    pub gates_executed: usize,
    pub gates_passed: usize,
    pub gates_failed: usize,
    pub gates_warning: usize,
    pub gates_skipped: usize,
    pub total_duration_ms: u64,
    pub executed_at: DateTime<Utc>,
    pub gate_results: Vec<QualityGateResult>,
    pub recommendations: Vec<String>,
}

/// Quality gate orchestrator
pub struct QualityGateOrchestrator {
    config: QualityGateOrchestratorConfig,
    gates: Vec<Arc<dyn QualityGate>>,
}

impl QualityGateOrchestrator {
    pub fn new(config: QualityGateOrchestratorConfig) -> Self {
        Self {
            config,
            gates: Vec::new(),
        }
    }

    /// Add a quality gate
    pub fn add_gate(&mut self, gate: Arc<dyn QualityGate>) {
        self.gates.push(gate);
    }

    /// Execute all quality gates for the given artifacts
    pub async fn execute_quality_gates(
        &self,
        artifacts: &ExecutionArtifacts,
        working_dir: PathBuf,
        risk_tier: RiskTier,
    ) -> Result<QualityReport, QualityGateOrchestratorError> {
        let start_time = std::time::Instant::now();
        let executed_at = Utc::now();

        tracing::info!("Starting quality gate execution for task: {} (tier: {:?})",
            artifacts.task_id, risk_tier);

        // Get thresholds for risk tier
        let thresholds = self.get_thresholds_for_tier(risk_tier);

        // Filter gates that should run
        let gates_to_run: Vec<_> = self.gates.iter()
            .filter(|gate| gate.should_run(artifacts))
            .map(|gate| Arc::clone(gate))
            .collect();

        if gates_to_run.is_empty() {
            return Ok(QualityReport {
                task_id: artifacts.task_id,
                risk_tier,
                overall_status: GateStatus::Skipped,
                overall_score: 0.0,
                gates_executed: 0,
                gates_passed: 0,
                gates_failed: 0,
                gates_warning: 0,
                gates_skipped: 0,
                total_duration_ms: 0,
                executed_at,
                gate_results: vec![],
                recommendations: vec!["No quality gates applicable".to_string()],
            });
        }

        // Execute gates (parallel or sequential)
        let gate_results = if self.config.enable_parallel {
            self.execute_gates_parallel(&gates_to_run, artifacts, &working_dir, &thresholds).await?
        } else {
            self.execute_gates_sequential(&gates_to_run, artifacts, &working_dir, &thresholds).await?
        };

        let total_duration = start_time.elapsed().as_millis() as u64;

        // Calculate summary statistics
        let (overall_status, overall_score, gates_passed, gates_failed, gates_warning, gates_skipped) =
            self.calculate_summary(&gate_results);

        let recommendations = self.generate_recommendations(&gate_results, risk_tier);

        let report = QualityReport {
            task_id: artifacts.task_id,
            risk_tier,
            overall_status,
            overall_score,
            gates_executed: gate_results.len(),
            gates_passed,
            gates_failed,
            gates_warning,
            gates_skipped,
            total_duration_ms: total_duration,
            executed_at,
            gate_results,
            recommendations,
        };

        tracing::info!("Quality gate execution completed for task: {} - Status: {:?}, Score: {:.2}",
            artifacts.task_id, report.overall_status, report.overall_score);

        Ok(report)
    }

    /// Execute gates in parallel with concurrency control
    async fn execute_gates_parallel(
        &self,
        gates: &[Arc<dyn QualityGate>],
        artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<Vec<QualityGateResult>, QualityGateOrchestratorError> {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_gates));
        let mut tasks = Vec::new();

        for gate in gates {
            let gate = Arc::clone(gate);
            let artifacts = artifacts.clone();
            let working_dir = working_dir.clone();
            let thresholds = thresholds.clone();
            let semaphore = Arc::clone(&semaphore);

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                timeout(
                    Duration::from_secs(self.config.gate_timeout_seconds),
                    gate.execute(&artifacts, &working_dir, &thresholds)
                ).await
                .map_err(|_| QualityGateOrchestratorError::GateTimeout(gate.name().to_string()))?
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    tracing::error!("Gate execution error: {:?}", e);
                    // Create error result
                    results.push(QualityGateResult {
                        name: "unknown".to_string(),
                        status: GateStatus::Error,
                        score: 0.0,
                        threshold: 0.0,
                        duration_ms: 0,
                        details: serde_json::json!({"error": format!("{:?}", e)}),
                        errors: vec![format!("{:?}", e)],
                    });
                }
                Err(e) => {
                    tracing::error!("Task join error: {:?}", e);
                    results.push(QualityGateResult {
                        name: "unknown".to_string(),
                        status: GateStatus::Error,
                        score: 0.0,
                        threshold: 0.0,
                        duration_ms: 0,
                        details: serde_json::json!({"error": "task panicked"}),
                        errors: vec!["Task panicked".to_string()],
                    });
                }
            }
        }

        Ok(results)
    }

    /// Execute gates sequentially
    async fn execute_gates_sequential(
        &self,
        gates: &[Arc<dyn QualityGate>],
        artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<Vec<QualityGateResult>, QualityGateOrchestratorError> {
        let mut results = Vec::new();

        for gate in gates {
            let result = timeout(
                Duration::from_secs(self.config.gate_timeout_seconds),
                gate.execute(artifacts, working_dir, thresholds)
            ).await
            .map_err(|_| QualityGateOrchestratorError::GateTimeout(gate.name().to_string()))?;

            results.push(result);

            // Stop on first failure if configured
            if self.config.stop_on_first_failure && matches!(result.status, GateStatus::Failed | GateStatus::Error) {
                break;
            }
        }

        Ok(results)
    }

    /// Get quality thresholds for a risk tier
    fn get_thresholds_for_tier(&self, risk_tier: RiskTier) -> QualityThresholds {
        match risk_tier {
            RiskTier::Critical => QualityThresholds {
                lint_errors_max: 0,
                type_errors_max: 0,
                test_failure_max: 0,
                coverage_min: 90.0,
                mutation_score_min: 70.0,
                caws_violations_max: 0,
            },
            RiskTier::High => QualityThresholds {
                lint_errors_max: 5,
                type_errors_max: 0,
                test_failure_max: 0,
                coverage_min: 80.0,
                mutation_score_min: 50.0,
                caws_violations_max: 3,
            },
            RiskTier::Standard => QualityThresholds {
                lint_errors_max: 10,
                type_errors_max: 5,
                test_failure_max: 2,
                coverage_min: 70.0,
                mutation_score_min: 30.0,
                caws_violations_max: 5,
            },
        }
    }

    /// Calculate summary statistics from gate results
    fn calculate_summary(&self, results: &[QualityGateResult]) -> (GateStatus, f64, usize, usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut warning = 0;
        let mut skipped = 0;
        let mut total_score = 0.0;

        for result in results {
            total_score += result.score;

            match result.status {
                GateStatus::Passed => passed += 1,
                GateStatus::Failed => failed += 1,
                GateStatus::Warning => warning += 1,
                GateStatus::Skipped => skipped += 1,
                GateStatus::Error => failed += 1,
            }
        }

        let overall_score = if results.is_empty() { 0.0 } else { total_score / results.len() as f64 };

        let overall_status = if failed > 0 {
            GateStatus::Failed
        } else if warning > 0 {
            GateStatus::Warning
        } else if passed > 0 {
            GateStatus::Passed
        } else {
            GateStatus::Skipped
        };

        (overall_status, overall_score, passed, failed, warning, skipped)
    }

    /// Generate recommendations based on results
    fn generate_recommendations(&self, results: &[QualityGateResult], risk_tier: RiskTier) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for failing gates
        let failing_gates: Vec<_> = results.iter()
            .filter(|r| matches!(r.status, GateStatus::Failed | GateStatus::Error))
            .collect();

        for gate in &failing_gates {
            recommendations.push(format!(
                "Fix {} issues in {} gate (current: {}, threshold: {})",
                gate.name,
                gate.name,
                if gate.name == "coverage" || gate.name == "mutation" {
                    format!("{:.1}%", gate.score * 100.0)
                } else {
                    format!("{}", gate.score as u32)
                },
                gate.threshold
            ));
        }

        // Check for warning gates
        let warning_gates: Vec<_> = results.iter()
            .filter(|r| r.status == GateStatus::Warning)
            .collect();

        for gate in &warning_gates {
            recommendations.push(format!(
                "Address {} warnings in {} gate",
                gate.name, gate.name
            ));
        }

        // Risk tier specific recommendations
        match risk_tier {
            RiskTier::Critical => {
                if failing_gates.is_empty() && warning_gates.is_empty() {
                    recommendations.push("All critical quality gates passed - ready for production".to_string());
                }
            }
            RiskTier::High => {
                recommendations.push("Consider adding more comprehensive tests for high-risk changes".to_string());
            }
            RiskTier::Standard => {
                recommendations.push("Standard quality thresholds met - monitor in production".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("All quality gates passed successfully".to_string());
        }

        recommendations
    }

    /// Get all registered gates
    pub fn get_gates(&self) -> &[Arc<dyn QualityGate>] {
        &self.gates
    }

    /// Check if orchestrator has any gates
    pub fn has_gates(&self) -> bool {
        !self.gates.is_empty()
    }
}

/// Convenience builder for quality gate orchestrators
pub struct QualityGateOrchestratorBuilder {
    config: QualityGateOrchestratorConfig,
    gates: Vec<Arc<dyn QualityGate>>,
}

impl QualityGateOrchestratorBuilder {
    pub fn new() -> Self {
        Self {
            config: QualityGateOrchestratorConfig {
                max_concurrent_gates: 4,
                overall_timeout_seconds: 900, // 15 minutes
                gate_timeout_seconds: 300,     // 5 minutes
                enable_parallel: true,
                stop_on_first_failure: false,
                enable_detailed_logging: true,
            },
            gates: Vec::new(),
        }
    }

    pub fn config(mut self, config: QualityGateOrchestratorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn add_gate(mut self, gate: Arc<dyn QualityGate>) -> Self {
        self.gates.push(gate);
        self
    }

    pub fn add_default_javascript_gates(mut self, caws_validator: Arc<dyn crate::caws_runtime::CawsRuntimeValidator>) -> Self {
        use super::gates::*;

        self.gates.push(Arc::new(CawsComplianceGate::new(caws_validator)));
        self.gates.push(Arc::new(LintingGate::eslint()));
        self.gates.push(Arc::new(TypeCheckGate::tsc()));
        self.gates.push(Arc::new(TestingGate::jest()));
        self.gates.push(Arc::new(CoverageGate::istanbul()));
        self.gates.push(Arc::new(MutationGate::stryker()));

        self
    }

    pub fn add_default_rust_gates(mut self, caws_validator: Arc<dyn crate::caws_runtime::CawsRuntimeValidator>) -> Self {
        use super::gates::*;

        self.gates.push(Arc::new(CawsComplianceGate::new(caws_validator)));
        self.gates.push(Arc::new(LintingGate::clippy()));
        self.gates.push(Arc::new(TypeCheckGate::cargo_check()));
        self.gates.push(Arc::new(TestingGate::cargo_test()));
        // Note: Coverage and mutation testing for Rust would require additional setup
        // self.gates.push(Arc::new(CoverageGate::tarpaulin()));
        // self.gates.push(Arc::new(MutationGate::cargo_mutants()));

        self
    }

    pub fn build(self) -> QualityGateOrchestrator {
        let mut orchestrator = QualityGateOrchestrator::new(self.config);
        for gate in self.gates {
            orchestrator.add_gate(gate);
        }
        orchestrator
    }
}

impl Default for QualityGateOrchestratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub type Result<T> = std::result::Result<T, QualityGateOrchestratorError>;

#[derive(Debug, thiserror::Error)]
pub enum QualityGateOrchestratorError {
    #[error("Gate execution timed out: {0}")]
    GateTimeout(String),

    #[error("Gate execution failed: {0}")]
    GateExecutionError(String),

    #[error("Task execution failed")]
    TaskError,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
