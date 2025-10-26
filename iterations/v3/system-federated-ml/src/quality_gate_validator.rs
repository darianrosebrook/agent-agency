//! Quality Gate Validator for LLM Parameter Optimization
//!
//! Implements trust region validation, quality floor checks, and pre-deployment
//! validation for safe parameter optimization.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bandit_policy::ParameterSet;
use crate::reward::{OptimizationConstraints, BaselineMetrics};

/// Validation result for parameter proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceValidationResult {
    pub passed: bool,
    pub violations: Vec<String>,
    pub score: f64, // 0.0 to 1.0
}

/// Mock compliance validator for testing
pub struct MockComplianceValidator;

#[async_trait::async_trait]
impl ComplianceValidator for MockComplianceValidator {
    async fn validate_parameters(&self, _parameters: &ParameterSet) -> Result<ComplianceValidationResult> {
        Ok(ComplianceValidationResult {
            passed: true,
            violations: vec![],
            score: 0.95,
        })
    }
}

impl QualityGateValidator {
    pub fn new(quality_threshold: f64) -> Self {
        Self {
            baseline_quality: Arc::new(RwLock::new(HashMap::new())),
            quality_threshold,
            compliance_validator: Arc::new(MockComplianceValidator),
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
        // Simplified quality estimation based on parameter similarity to baseline
        // In practice, this would use a trained model or historical data
        
        let temp_similarity = 1.0 - (params.temperature - baseline.temperature).abs() / 2.0;
        let token_similarity = 1.0 - (params.max_tokens as f64 - baseline.avg_tokens).abs() / baseline.avg_tokens;
        
        // Weighted combination of similarities
        let estimated_quality = baseline.avg_quality * (0.7 * temp_similarity + 0.3 * token_similarity);
        
        Ok(estimated_quality.max(0.0).min(1.0))
    }

    /// Estimate latency delta for proposed parameters
    fn estimate_latency_delta(&self, params: &ParameterSet, baseline: &BaselineMetrics) -> f64 {
        // Simplified latency estimation
        // Higher temperature and more tokens generally increase latency
        let temp_factor = (params.temperature - baseline.temperature) * 100.0;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
