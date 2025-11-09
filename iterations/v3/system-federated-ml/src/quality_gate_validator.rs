//! Quality Gate Validator for LLM Parameter Optimization
//!
//! Implements trust region validation, quality floor checks, and pre-deployment
//! validation for safe parameter optimization.

use schemars::JsonSchema;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bandit_policy::ParameterSet;
use crate::reward::{OptimizationConstraints, BaselineMetrics};

/// Validation result for parameter proposals
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Serialize, Deserialize)]
pub enum ValidationResult {
    Approved {
        quality_delta: f64,
        latency_delta: i64,
        token_delta: f64,
        confidence_score: f64,
    },
    Rejected {
        reason: String,
        severity: ValidationSeverity,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

/// Quality gate validator with trust regions and pre-deployment checks
pub struct QualityGateValidator {
    /// Baseline quality metrics per task type
    baseline_quality: Arc<RwLock<HashMap<String, BaselineMetrics>>>,
    /// Quality threshold for approval
    quality_threshold: f64,
    /// Compliance validator
    compliance_validator: Arc<dyn ComplianceValidator>,
}

/// Trait for compliance validation
#[async_trait::async_trait]
pub trait ComplianceValidator: Send + Sync {
    async fn validate_parameters(&self, parameters: &ParameterSet) -> Result<ComplianceValidationResult>;
}

/// Compliance validation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceValidationResult {
    pub passed: bool,
    pub violations: Vec<String>,
    pub score: f64, // 0.0 to 1.0
}

/// Placeholder compliance validator that panics
/// 
/// This is a placeholder indicating that a real ComplianceValidator implementation
/// must be provided via QualityGateValidator::with_compliance_validator().
/// Real CAWS compliance validator using policy enforcement tools
pub struct RealComplianceValidator {
    policy_tools: std::sync::Arc<crate::policy_enforcement::PolicyEnforcementTools>,
}

impl RealComplianceValidator {
    /// Create a new real compliance validator
    pub fn new() -> Self {
        Self {
            policy_tools: std::sync::Arc::new(crate::policy_enforcement::PolicyEnforcementTools::new()),
        }
    }
}

#[async_trait::async_trait]
impl ComplianceValidator for RealComplianceValidator {
    async fn validate_parameters(&self, parameters: &ParameterSet) -> Result<ComplianceValidationResult> {
        use crate::policy_enforcement::{TaskDescriptor, WorkingSpec, AcceptanceCriterion};

        // Convert parameter set to task descriptor for CAWS validation
        let task_descriptor = TaskDescriptor {
            id: parameters.execution_id.clone(),
            task_type: "federated_execution".to_string(),
            description: format!("Federated ML execution with {} parameters", parameters.parameters.len()),
            priority: agent_agency_contracts::task_executor::TaskPriority::High,
            metadata: serde_json::json!({
                "parameters": parameters.parameters,
                "execution_mode": parameters.execution_mode,
                "quality_gates": parameters.quality_gates
            }),
        };

        // Create a basic working spec for validation
        let working_spec = WorkingSpec {
            id: format!("spec-{}", parameters.execution_id),
            title: "Federated ML Execution".to_string(),
            risk_tier: 2, // Medium risk for ML execution
            mode: "feature".to_string(),
            change_budget: crate::ChangeBudget {
                max_files: 10,
                max_loc: 1000,
                max_days: 1,
            },
            blast_radius: crate::BlastRadius {
                modules: vec!["federated-ml".to_string()],
                data_migration: false,
                external_apis: false,
            },
            operational_rollback_slo: "5m".to_string(),
            scope: crate::Scope {
                in_paths: vec!["src/federated-ml/".to_string()],
                out_paths: vec!["src/other/".to_string()],
            },
            invariants: vec![
                "Data privacy must be maintained".to_string(),
                "Model accuracy requirements met".to_string(),
            ],
            acceptance: vec![AcceptanceCriterion {
                id: "A1".to_string(),
                given: "Valid federated execution parameters".to_string(),
                when: "Parameters are validated".to_string(),
                then: "CAWS compliance is confirmed".to_string(),
            }],
            non_functional: crate::NonFunctionalRequirements {
                performance: crate::PerformanceRequirements {
                    response_time_ms: 5000,
                    throughput_per_second: 10.0,
                    availability_percent: 99.9,
                },
                security: vec![
                    "Input validation".to_string(),
                    "Data encryption".to_string(),
                ],
                accessibility: vec![],
                compliance: vec![
                    "CAWS policy compliance".to_string(),
                    "Data privacy regulations".to_string(),
                ],
            },
            contracts: vec![],
        };

        // Validate against CAWS policies using real policy enforcement
        let validation_result = self.policy_tools.validate_task_against_caws(&task_descriptor, &working_spec).await?;

        // Convert to compliance validation result
        let compliance_result = ComplianceValidationResult {
            compliant: validation_result.passed,
            violations: validation_result.violations,
            recommendations: vec![], // Policy tools don't provide recommendations yet
            risk_score: 0.0, // Default risk score - could be calculated based on violations
            compliance_score: validation_result.compliance_score,
        };

        Ok(compliance_result)
    }
}

impl QualityGateValidator {
    /// Create a new quality gate validator with real CAWS compliance validation
    pub fn new(quality_threshold: f64) -> Self {
        Self {
            baseline_quality: Arc::new(RwLock::new(HashMap::new())),
            quality_threshold,
            compliance_validator: Arc::new(RealComplianceValidator::new()),
        }
    }

    pub fn with_compliance_validator(
        quality_threshold: f64,
        compliance_validator: Arc<dyn ComplianceValidator>,
    ) -> Self {
        Self {
            baseline_quality: Arc::new(RwLock::new(HashMap::new())),
            quality_threshold,
            compliance_validator,
        }
    }

    /// Set baseline metrics for a task type
    pub async fn set_baseline(&self, task_type: String, baseline: BaselineMetrics) {
        let mut baselines = self.baseline_quality.write().await;
        baselines.insert(task_type, baseline);
    }

    /// Get baseline metrics for a task type
    pub async fn get_baseline(&self, task_type: &str) -> Result<BaselineMetrics> {
        let baselines = self.baseline_quality.read().await;
        baselines.get(task_type)
            .cloned()
            .ok_or_else(|| anyhow!("No baseline found for task type: {}", task_type))
    }

    /// Validate parameters are within trust region and constraints
    pub async fn validate_pre_deployment(
        &self,
        task_type: &str,
        proposed: &ParameterSet,
        constraints: &OptimizationConstraints,
    ) -> Result<ValidationResult> {
        let baseline = self.get_baseline(task_type).await?;
        
        // 1. Trust region check
        let temp_delta = (proposed.temperature - baseline.temperature).abs();
        if temp_delta > constraints.max_delta_temperature {
            return Ok(ValidationResult::Rejected {
                reason: format!(
                    "Temperature delta {:.3} exceeds trust region {:.3}",
                    temp_delta, constraints.max_delta_temperature
                ),
                severity: ValidationSeverity::Error,
            });
        }
        
        let tokens_delta = (proposed.max_tokens as i64 - baseline.max_tokens as i64).abs();
        if tokens_delta > constraints.max_delta_max_tokens as i64 {
            return Ok(ValidationResult::Rejected {
                reason: format!(
                    "Token delta {} exceeds trust region {}",
                    tokens_delta, constraints.max_delta_max_tokens
                ),
                severity: ValidationSeverity::Error,
            });
        }
        
        // 2. Quality floor check (expected quality ≥ baseline - threshold)
        let expected_quality = self.estimate_quality(proposed, &baseline).await?;
        if expected_quality < baseline.avg_quality - self.quality_threshold {
            return Ok(ValidationResult::Rejected {
                reason: format!(
                    "Expected quality {:.3} below acceptable threshold {:.3}",
                    expected_quality, baseline.avg_quality - self.quality_threshold
                ),
                severity: ValidationSeverity::Warning,
            });
        }
        
        // 3. Hard constraint checks
        if proposed.max_tokens > constraints.max_tokens {
            return Ok(ValidationResult::Rejected {
                reason: format!(
                    "Token limit {} exceeds constraint {}",
                    proposed.max_tokens, constraints.max_tokens
                ),
                severity: ValidationSeverity::Error,
            });
        }
        
        // 4. CAWS compliance
        if constraints.require_caws_compliance {
            let compliance = self.compliance_validator
                .validate_parameters(proposed)
                .await?;
            if !compliance.passed {
                return Ok(ValidationResult::Rejected {
                    reason: format!("CAWS compliance failed: {:?}", compliance.violations),
                    severity: ValidationSeverity::Error,
                });
            }
        }
        
        // Calculate deltas for approved parameters
        let quality_delta = expected_quality - baseline.avg_quality;
        let latency_delta = self.estimate_latency_delta(proposed, &baseline) as i64;
        let token_delta = proposed.max_tokens as f64 - baseline.avg_tokens;
        
        Ok(ValidationResult::Approved {
            quality_delta,
            latency_delta,
            token_delta,
            confidence_score: self.calculate_confidence_score(proposed, &baseline),
        })
    }

    /// Estimate quality for proposed parameters
    async fn estimate_quality(&self, params: &ParameterSet, baseline: &BaselineMetrics) -> Result<f64> {
        // TODO: Implement quality estimation using trained model or historical data
        //       Currently uses basic similarity-based estimation; should implement quality estimation using trained model or historical data for accurate predictions.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Quality estimation uses trained model or historical data
        // - Predictions are accurate
        // - Model integration works correctly
        // - Performance is acceptable
        //
        // DEPENDENCIES:
        // - Trained quality model (Required)
        // - Historical data infrastructure (Required)
        // - Model inference utilities (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (ML model integration feature)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: ML model integration expertise
        let temp_similarity = 1.0 - (params.temperature - baseline.temperature).abs() / 2.0; // Temporary: similarity-based until model integration
        let token_similarity = 1.0 - (params.max_tokens as f64 - baseline.avg_tokens).abs() / baseline.avg_tokens;
        
        // Weighted combination of similarities
        let estimated_quality = baseline.avg_quality * (0.7 * temp_similarity + 0.3 * token_similarity);
        
        Ok(estimated_quality.max(0.0).min(1.0))
    }

    /// Estimate latency delta for proposed parameters
    fn estimate_latency_delta(&self, params: &ParameterSet, baseline: &BaselineMetrics) -> f64 {
        // TODO: Implement comprehensive latency estimation
        //       Currently uses basic estimation; should implement comprehensive latency estimation considering all parameter effects and historical patterns.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Latency estimation considers all parameter effects
        // - Historical patterns are incorporated
        // - Estimation is accurate
        // - Performance is acceptable
        //
        // DEPENDENCIES:
        // - Historical latency data (Required)
        // - Parameter effect analysis (Required)
        // - Latency modeling utilities (Required)
        //
        // ESTIMATED EFFORT: 5-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (performance estimation feature)
        // - Change Budget: ~120 LOC
        // - Reviewer Requirements: Performance modeling expertise
        let temp_factor = (params.temperature - baseline.temperature) * 100.0; // Temporary: basic until comprehensive estimation
        let token_factor = (params.max_tokens as f64 - baseline.avg_tokens) * 0.1;
        
        temp_factor + token_factor
    }

    /// Calculate confidence score for parameter proposal
    fn calculate_confidence_score(&self, params: &ParameterSet, baseline: &BaselineMetrics) -> f64 {
        // Confidence based on how close parameters are to known good baselines
        let temp_distance = (params.temperature - baseline.temperature).abs();
        let token_distance = (params.max_tokens as f64 - baseline.avg_tokens).abs() / baseline.avg_tokens;
        
        // Closer to baseline = higher confidence
        let temp_confidence = (1.0 - temp_distance / 2.0).max(0.0);
        let token_confidence = (1.0 - token_distance).max(0.0);
        
        (temp_confidence + token_confidence) / 2.0
    }

    /// Validate parameters against historical performance
    pub async fn validate_against_history(
        &self,
        task_type: &str,
        params: &ParameterSet,
    ) -> Result<ValidationResult> {
        let baseline = self.get_baseline(task_type).await?;
        
        // Check if parameters are within historical performance bounds
        let quality_estimate = self.estimate_quality(params, &baseline).await?;
        
        if quality_estimate < baseline.avg_quality * 0.8 {
            return Ok(ValidationResult::Rejected {
                reason: format!(
                    "Estimated quality {:.3} significantly below baseline {:.3}",
                    quality_estimate, baseline.avg_quality
                ),
                severity: ValidationSeverity::Warning,
            });
        }
        
        Ok(ValidationResult::Approved {
            quality_delta: quality_estimate - baseline.avg_quality,
            latency_delta: self.estimate_latency_delta(params, &baseline) as i64,
            token_delta: params.max_tokens as f64 - baseline.avg_tokens,
            confidence_score: self.calculate_confidence_score(params, &baseline),
        })
    }

    /// Get validation statistics for a task type
    pub async fn get_validation_stats(&self, task_type: &str) -> Result<ValidationStats> {
        let baseline = self.get_baseline(task_type).await?;
        
        Ok(ValidationStats {
            task_type: task_type.to_string(),
            baseline_quality: baseline.avg_quality,
            baseline_latency: baseline.avg_latency,
            baseline_tokens: baseline.avg_tokens,
            quality_threshold: self.quality_threshold,
            validation_count: 0, // Would be tracked in real implementation
        })
    }
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationStats {
    pub task_type: String,
    pub baseline_quality: f64,
    pub baseline_latency: u64,
    pub baseline_tokens: f64,
    pub quality_threshold: f64,
    pub validation_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_quality_gate_validation() {
        let validator = QualityGateValidator::new(0.1);
        
        let baseline = BaselineMetrics {
            avg_quality: 0.8,
            avg_latency: 1000,
            avg_tokens: 500.0,
            temperature: 0.7,
            max_tokens: 1000,
        };
        
        validator.set_baseline("test_task".to_string(), baseline).await;
        
        let constraints = OptimizationConstraints::default();
        
        let params = ParameterSet {
            temperature: 0.75, // Within trust region
            max_tokens: 800,    // Within trust region
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: vec![],
            seed: None,
            origin: "test".to_string(),
            policy_version: "1.0.0".to_string(),
            created_at: Utc::now(),
        };
        
        let result = validator.validate_pre_deployment("test_task", &params, &constraints).await.unwrap();
        
        match result {
            ValidationResult::Approved { quality_delta, .. } => {
                assert!(quality_delta >= -0.1, "Quality delta should be reasonable");
            }
            ValidationResult::Rejected { .. } => {
                panic!("Valid parameters should be approved");
            }
        }
    }

    #[tokio::test]
    async fn test_trust_region_violation() {
        let validator = QualityGateValidator::new(0.1);
        
        let baseline = BaselineMetrics {
            avg_quality: 0.8,
            avg_latency: 1000,
            avg_tokens: 500.0,
            temperature: 0.7,
            max_tokens: 1000,
        };
        
        validator.set_baseline("test_task".to_string(), baseline).await;
        
        let constraints = OptimizationConstraints {
            max_delta_temperature: 0.1, // Very restrictive
            max_delta_max_tokens: 50,
            ..Default::default()
        };
        
        let params = ParameterSet {
            temperature: 1.0, // Way outside trust region
            max_tokens: 1000,
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: vec![],
            seed: None,
            origin: "test".to_string(),
            policy_version: "1.0.0".to_string(),
            created_at: Utc::now(),
        };
        
        let result = validator.validate_pre_deployment("test_task", &params, &constraints).await.unwrap();
        
        match result {
            ValidationResult::Rejected { reason, .. } => {
                assert!(reason.contains("Temperature delta"), "Should reject due to temperature delta");
            }
            ValidationResult::Approved { .. } => {
                panic!("Parameters outside trust region should be rejected");
            }
        }
    }
}
