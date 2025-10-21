/// Update validation for federated learning
///
/// Validates model updates for quality, consistency, and security
/// before inclusion in federated aggregation.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Update validator for model contributions
pub struct UpdateValidator {
    quality_thresholds: QualityThresholds,
    security_validator: SecurityValidator,
}

/// Quality thresholds for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum gradient norm
    pub min_gradient_norm: f32,
    /// Maximum gradient norm
    pub max_gradient_norm: f32,
    /// Minimum stability score
    pub min_stability_score: f32,
    /// Maximum update magnitude
    pub max_update_magnitude: f32,
    /// Minimum training samples
    pub min_training_samples: usize,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the update passed validation
    pub is_valid: bool,
    /// Validation score (0.0-1.0)
    pub score: f32,
    /// Validation issues found
    pub issues: Vec<ValidationIssue>,
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// Individual validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue type
    pub issue_type: String,
    /// Severity level
    pub severity: Severity,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggestion: String,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl UpdateValidator {
    /// Create a new update validator
    pub fn new() -> Self {
        Self {
            quality_thresholds: QualityThresholds {
                min_gradient_norm: 0.001,
                max_gradient_norm: 100.0,
                min_stability_score: 0.5,
                max_update_magnitude: 1.0,
                min_training_samples: 100,
            },
            security_validator: SecurityValidator::new(),
        }
    }

    /// Validate a model update
    pub async fn validate_update(&self, update: &ModelUpdate) -> Result<ValidationResult> {
        debug!("Validating update from participant {}", update.participant_id);

        let mut issues = Vec::new();
        let mut score = 1.0; // Start with perfect score
        let mut recommendations = Vec::new();

        // Basic validation
        self.validate_basic_properties(update, &mut issues, &mut score, &mut recommendations)?;

        // Quality validation
        self.validate_quality_metrics(update, &mut issues, &mut score, &mut recommendations)?;

        // Security validation
        self.validate_security(update, &mut issues, &mut score, &mut recommendations).await?;

        // Consistency validation
        self.validate_consistency(update, &mut issues, &mut score, &mut recommendations)?;

        let is_valid = issues.iter().all(|issue| issue.severity != Severity::Critical);

        // Adjust final score based on issues
        for issue in &issues {
            match issue.severity {
                Severity::Low => score *= 0.95,
                Severity::Medium => score *= 0.85,
                Severity::High => score *= 0.7,
                Severity::Critical => score *= 0.5,
            }
        }

        Ok(ValidationResult {
            is_valid,
            score: score.max(0.0),
            issues,
            recommendations,
        })
    }

    /// Validate basic update properties
    fn validate_basic_properties(
        &self,
        update: &ModelUpdate,
        issues: &mut Vec<ValidationIssue>,
        score: &mut f32,
        recommendations: &mut Vec<String>,
    ) -> Result<()> {
        // Check if parameters exist
        if update.parameters.is_empty() {
            issues.push(ValidationIssue {
                issue_type: "empty_parameters".to_string(),
                severity: Severity::Critical,
                description: "Update contains no model parameters".to_string(),
                suggestion: "Ensure model training completed successfully".to_string(),
            });
            return Ok(());
        }

        // Check training samples
        if update.metadata.training_samples < self.quality_thresholds.min_training_samples {
            issues.push(ValidationIssue {
                issue_type: "insufficient_training_data".to_string(),
                severity: Severity::High,
                description: format!("Only {} training samples, minimum required is {}",
                                   update.metadata.training_samples,
                                   self.quality_thresholds.min_training_samples),
                suggestion: "Train with more data samples".to_string(),
            });
            recommendations.push("Increase training dataset size".to_string());
        }

        // Check for reasonable loss values
        if update.metadata.final_loss.is_nan() || update.metadata.final_loss.is_infinite() {
            issues.push(ValidationIssue {
                issue_type: "invalid_loss".to_string(),
                severity: Severity::Critical,
                description: "Final loss is NaN or infinite".to_string(),
                suggestion: "Check training stability and numerical issues".to_string(),
            });
        } else if update.metadata.final_loss > 10.0 {
            issues.push(ValidationIssue {
                issue_type: "high_loss".to_string(),
                severity: Severity::Medium,
                description: format!("Final loss is very high: {:.3}", update.metadata.final_loss),
                suggestion: "Review training hyperparameters".to_string(),
            });
            recommendations.push("Tune learning rate or training duration".to_string());
        }

        Ok(())
    }

    /// Validate quality metrics
    fn validate_quality_metrics(
        &self,
        update: &ModelUpdate,
        issues: &mut Vec<ValidationIssue>,
        score: &mut f32,
        recommendations: &mut Vec<String>,
    ) -> Result<()> {
        let metrics = match &update.quality_metrics {
            Some(m) => m,
            None => {
                issues.push(ValidationIssue {
                    issue_type: "missing_metrics".to_string(),
                    severity: Severity::Medium,
                    description: "Quality metrics not computed".to_string(),
                    suggestion: "Compute quality metrics before submission".to_string(),
                });
                recommendations.push("Add quality metrics computation".to_string());
                return Ok(());
            }
        };

        // Validate gradient norm
        if metrics.gradient_norm < self.quality_thresholds.min_gradient_norm {
            issues.push(ValidationIssue {
                issue_type: "low_gradient_norm".to_string(),
                severity: Severity::Medium,
                description: format!("Gradient norm too low: {:.6}", metrics.gradient_norm),
                suggestion: "Check for vanishing gradients or training issues".to_string(),
            });
        } else if metrics.gradient_norm > self.quality_thresholds.max_gradient_norm {
            issues.push(ValidationIssue {
                issue_type: "high_gradient_norm".to_string(),
                severity: Severity::High,
                description: format!("Gradient norm too high: {:.3}", metrics.gradient_norm),
                suggestion: "Apply gradient clipping".to_string(),
            });
            recommendations.push("Implement gradient clipping".to_string());
        }

        // Validate stability score
        if metrics.stability_score < self.quality_thresholds.min_stability_score {
            issues.push(ValidationIssue {
                issue_type: "low_stability".to_string(),
                severity: Severity::High,
                description: format!("Stability score too low: {:.3}", metrics.stability_score),
                suggestion: "Improve training stability".to_string(),
            });
            recommendations.push("Use learning rate scheduling".to_string());
        }

        // Validate update magnitude
        if metrics.update_magnitude > self.quality_thresholds.max_update_magnitude {
            issues.push(ValidationIssue {
                issue_type: "large_update".to_string(),
                severity: Severity::Medium,
                description: format!("Update magnitude too large: {:.3}", metrics.update_magnitude),
                suggestion: "Reduce learning rate or apply update clipping".to_string(),
            });
            recommendations.push("Apply update magnitude clipping".to_string());
        }

        Ok(())
    }

    /// Validate security properties
    async fn validate_security(
        &self,
        update: &ModelUpdate,
        issues: &mut Vec<ValidationIssue>,
        score: &mut f32,
        recommendations: &mut Vec<String>,
    ) -> Result<()> {
        // Check for potential poisoning attempts
        let parameter_data = serde_json::to_vec(&update.parameters)?;
        let violations = self.security_validator.check_security_violations(&parameter_data).await?;

        for violation in violations {
            let severity = match violation.severity {
                crate::security::Severity::Low => Severity::Low,
                crate::security::Severity::Medium => Severity::Medium,
                crate::security::Severity::High => Severity::High,
                crate::security::Severity::Critical => Severity::Critical,
            };

            issues.push(ValidationIssue {
                issue_type: format!("security_{}", violation.violation_type),
                severity,
                description: violation.description,
                suggestion: "Review update for potential security issues".to_string(),
            });
        }

        Ok(())
    }

    /// Validate update consistency
    fn validate_consistency(
        &self,
        update: &ModelUpdate,
        issues: &mut Vec<ValidationIssue>,
        score: &mut f32,
        recommendations: &mut Vec<String>,
    ) -> Result<()> {
        // Check parameter dimensions are reasonable
        for (layer_idx, layer) in update.parameters.iter().enumerate() {
            if layer.is_empty() {
                issues.push(ValidationIssue {
                    issue_type: "empty_layer".to_string(),
                    severity: Severity::Critical,
                    description: format!("Layer {} has no parameters", layer_idx),
                    suggestion: "Check model architecture".to_string(),
                });
            } else if layer.len() > 10_000_000 {
                issues.push(ValidationIssue {
                    issue_type: "excessive_parameters".to_string(),
                    severity: Severity::Medium,
                    description: format!("Layer {} has {} parameters, seems excessive", layer_idx, layer.len()),
                    suggestion: "Verify model architecture is correct".to_string(),
                });
            }
        }

        // Check for NaN or infinite values
        let mut nan_count = 0;
        let mut inf_count = 0;

        for layer in &update.parameters {
            for &param in layer {
                if param.is_nan() {
                    nan_count += 1;
                } else if param.is_infinite() {
                    inf_count += 1;
                }
            }
        }

        if nan_count > 0 {
            issues.push(ValidationIssue {
                issue_type: "nan_values".to_string(),
                severity: Severity::Critical,
                description: format!("Found {} NaN values in parameters", nan_count),
                suggestion: "Check for numerical instability in training".to_string(),
            });
        }

        if inf_count > 0 {
            issues.push(ValidationIssue {
                issue_type: "infinite_values".to_string(),
                severity: Severity::Critical,
                description: format!("Found {} infinite values in parameters", inf_count),
                suggestion: "Apply gradient clipping or check loss function".to_string(),
            });
        }

        Ok(())
    }

    /// Update validation thresholds based on federation history
    pub fn update_thresholds(&mut self, new_thresholds: QualityThresholds) {
        self.quality_thresholds = new_thresholds;
    }

    /// Get current validation statistics
    pub fn get_validation_stats(&self) -> ValidationStatistics {
        ValidationStatistics {
            total_validations: 0, // Would track in practice
            pass_rate: 0.0,
            common_issues: Vec::new(),
        }
    }
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatistics {
    pub total_validations: usize,
    pub pass_rate: f32,
    pub common_issues: Vec<(String, usize)>,
}

// Placeholder types for dependencies that will be implemented in other modules
use crate::model_updates::ModelUpdate;
use crate::security::{SecurityValidator, Severity};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_updates::ModelUpdate;

    #[test]
    fn test_basic_validation() {
        let validator = UpdateValidator::new();

        // Create a minimal valid update
        let mut update = ModelUpdate::new(
            "test_participant".to_string(),
            1,
            vec![vec![0.1, 0.2, 0.3]],
            1000,
            5,
            0.01,
            0.5,
        );

        update.compute_quality_metrics().unwrap();

        // This should pass basic validation
        let result = tokio::runtime::Runtime::new().unwrap()
            .block_on(validator.validate_update(&update))
            .unwrap();

        assert!(result.is_valid);
        assert!(result.score > 0.8);
    }

    #[test]
    fn test_insufficient_training_data() {
        let validator = UpdateValidator::new();

        let update = ModelUpdate::new(
            "test_participant".to_string(),
            1,
            vec![vec![0.1, 0.2, 0.3]],
            10, // Too few samples
            5,
            0.01,
            0.5,
        );

        let result = tokio::runtime::Runtime::new().unwrap()
            .block_on(validator.validate_update(&update))
            .unwrap();

        assert!(!result.issues.is_empty());
        assert!(result.issues.iter().any(|issue| issue.issue_type == "insufficient_training_data"));
    }
}


