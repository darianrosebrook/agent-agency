//! Bridge between parallel workers and orchestration quality gates
//!
//! This module provides integration points that allow parallel workers
//! to leverage the orchestration layer's quality gate system for validation.

use crate::error::{ParallelError, ParallelResult};
use crate::types::{TaskId, QualityRequirements, ValidationResult as ParallelValidationResult};
use async_trait::async_trait;
use std::sync::Arc;

// Stub types for orchestration integration (replace with actual imports when available)
#[derive(Debug, Clone)]
pub struct QualityGateResult {
    pub name: String,
    pub status: GateStatus,
    pub score: f64,
    pub threshold: f64,
    pub duration_ms: u64,
    pub details: serde_json::Value,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
    Error,
}

#[derive(Debug, Clone)]
pub struct QualityThresholds {
    pub lint_errors_max: u32,
    pub type_errors_max: u32,
    pub test_failure_max: u32,
    pub coverage_min: f64,
    pub mutation_score_min: f64,
    pub caws_violations_max: u32,
}

#[derive(Debug, Clone)]
pub struct ExecutionArtifacts {
    pub test_results: Option<serde_json::Value>,
    pub coverage_report: Option<serde_json::Value>,
    pub lint_report: Option<serde_json::Value>,
    pub type_check_report: Option<serde_json::Value>,
    pub mutation_report: Option<serde_json::Value>,
    pub provenance_record: Option<serde_json::Value>,
}

/// Bridge to orchestration quality gates
#[derive(Clone)]
pub struct OrchestrationQualityBridge {
    /// Handle to orchestration quality gate system
    orchestration_handle: Arc<dyn OrchestrationQualityHandle>,
}

/// Handle for accessing orchestration quality gate functionality
#[async_trait]
pub trait OrchestrationQualityHandle: Send + Sync {
    /// Run quality gates against execution artifacts
    async fn run_quality_gates(
        &self,
        artifacts: &ExecutionArtifacts,
        thresholds: &QualityThresholds,
    ) -> Result<Vec<QualityGateResult>, OrchestrationQualityError>;

    /// Check if quality gates are satisfied
    async fn check_quality_satisfied(
        &self,
        results: &[QualityGateResult],
        thresholds: &QualityThresholds,
    ) -> Result<bool, OrchestrationQualityError>;

    /// Get quality thresholds for a task
    async fn get_quality_thresholds(
        &self,
        task_id: &TaskId,
    ) -> Result<QualityThresholds, OrchestrationQualityError>;
}

/// Error type for orchestration quality bridge operations
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationQualityError {
    #[error("Quality gate execution failed: {message}")]
    GateExecutionFailed { message: String },

    #[error("Thresholds not found for task: {task_id}")]
    ThresholdsNotFound { task_id: String },

    #[error("Invalid quality configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Orchestration integration error: {message}")]
    IntegrationError { message: String },
}

impl OrchestrationQualityBridge {
    /// Create a new quality bridge
    pub fn new(handle: Arc<dyn OrchestrationQualityHandle>) -> Self {
        Self {
            orchestration_handle: handle,
        }
    }

    /// Run orchestration quality gates for parallel worker results
    pub async fn validate_with_orchestration_gates(
        &self,
        task_id: &TaskId,
        artifacts: &ExecutionArtifacts,
        quality_requirements: &QualityRequirements,
    ) -> ParallelResult<ParallelValidationResult> {
        // Get quality thresholds from orchestration
        let thresholds = self.orchestration_handle
            .get_quality_thresholds(task_id)
            .await
            .map_err(|e| ParallelError::Validation {
                message: format!("Failed to get quality thresholds: {}", e),
                source: None,
            })?;

        // Run quality gates
        let gate_results = self.orchestration_handle
            .run_quality_gates(artifacts, &thresholds)
            .await
            .map_err(|e| ParallelError::Validation {
                message: format!("Failed to run quality gates: {}", e),
                source: None,
            })?;

        // Check if all gates passed
        let all_passed = self.orchestration_handle
            .check_quality_satisfied(&gate_results, &thresholds)
            .await
            .map_err(|e| ParallelError::Validation {
                message: format!("Failed to check quality satisfaction: {}", e),
                source: None,
            })?;

        // Convert to parallel validation result
        if all_passed {
            Ok(ParallelValidationResult::Pass {
                score: 1.0,
                details: "All orchestration quality gates passed".to_string(),
            })
        } else {
            let failed_gates: Vec<_> = gate_results
                .iter()
                .filter(|r| r.status == GateStatus::Failed)
                .map(|r| r.name.clone())
                .collect();

            Ok(ParallelValidationResult::Fail {
                score: calculate_failure_score(&gate_results),
                details: format!("Failed quality gates: {:?}", failed_gates),
                suggestions: vec!["Review failed quality gates and fix issues".to_string()],
            })
        }
    }

    /// Get quality thresholds for parallel worker validation
    pub async fn get_quality_thresholds(
        &self,
        task_id: &TaskId,
    ) -> ParallelResult<QualityThresholds> {
        self.orchestration_handle
            .get_quality_thresholds(task_id)
            .await
            .map_err(|e| ParallelError::Validation {
                message: format!("Failed to get quality thresholds: {}", e),
                source: None,
            })
    }
}

/// Calculate failure score based on gate results
fn calculate_failure_score(gate_results: &[QualityGateResult]) -> f32 {
    let total_gates = gate_results.len() as f32;
    let passed_gates = gate_results
        .iter()
        .filter(|r| r.status == GateStatus::Passed)
        .count() as f32;

    if total_gates == 0.0 {
        0.0
    } else {
        passed_gates / total_gates
    }
}

/// Stub implementation for testing (replace with actual orchestration integration)
pub struct StubOrchestrationQualityHandle;

#[async_trait]
impl OrchestrationQualityHandle for StubOrchestrationQualityHandle {
    async fn run_quality_gates(
        &self,
        _artifacts: &ExecutionArtifacts,
        _thresholds: &QualityThresholds,
    ) -> Result<Vec<QualityGateResult>, OrchestrationQualityError> {
        // TODO: Implement actual orchestration quality gate execution
        // For now, return a basic passing result
        Ok(vec![
            QualityGateResult {
                name: "compilation".to_string(),
                status: GateStatus::Passed,
                score: 1.0,
                threshold: 0.8,
                duration_ms: 100,
                details: serde_json::json!({"message": "Compilation successful"}),
                errors: vec![],
            },
            QualityGateResult {
                name: "testing".to_string(),
                status: GateStatus::Passed,
                score: 0.95,
                threshold: 0.8,
                duration_ms: 500,
                details: serde_json::json!({"tests_passed": 95, "total_tests": 100}),
                errors: vec![],
            },
        ])
    }

    async fn check_quality_satisfied(
        &self,
        results: &[QualityGateResult],
        _thresholds: &QualityThresholds,
    ) -> Result<bool, OrchestrationQualityError> {
        Ok(results.iter().all(|r| r.status == GateStatus::Passed))
    }

    async fn get_quality_thresholds(
        &self,
        _task_id: &TaskId,
    ) -> Result<QualityThresholds, OrchestrationQualityError> {
        // Return default thresholds
        Ok(QualityThresholds {
            lint_errors_max: 10,
            type_errors_max: 0,
            test_failure_max: 5,
            coverage_min: 0.8,
            mutation_score_min: 0.5,
            caws_violations_max: 5,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestration_quality_bridge() {
        let stub_handle = Arc::new(StubOrchestrationQualityHandle);
        let bridge = OrchestrationQualityBridge::new(stub_handle);

        let task_id = TaskId::new();
        let artifacts = ExecutionArtifacts::default();
        let quality_reqs = QualityRequirements::default();

        let result = bridge.validate_with_orchestration_gates(
            &task_id,
            &artifacts,
            &quality_reqs,
        ).await;

        assert!(result.is_ok());
        match result.unwrap() {
            ParallelValidationResult::Pass { score, .. } => {
                assert_eq!(score, 1.0);
            }
            _ => panic!("Expected Pass result"),
        }
    }
}
