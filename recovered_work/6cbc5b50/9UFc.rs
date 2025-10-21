/// Model update handling for federated learning
///
/// Manages the creation, validation, and aggregation of model updates
/// from federation participants.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Model update from a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdate {
    /// Participant identifier
    pub participant_id: String,
    /// Round identifier
    pub round_id: u64,
    /// Model parameters (layer-wise)
    pub parameters: Vec<Vec<f32>>,
    /// Update metadata
    pub metadata: UpdateMetadata,
    /// Quality metrics
    pub quality_metrics: Option<UpdateQualityMetrics>,
}

/// Metadata for a model update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadata {
    /// Training data size used for this update
    pub training_samples: usize,
    /// Training epochs performed
    pub epochs_trained: u32,
    /// Learning rate used
    pub learning_rate: f32,
    /// Loss value after training
    pub final_loss: f32,
    /// Timestamp of update creation
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Model version/hash
    pub model_version: String,
}

/// Quality metrics for the update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQualityMetrics {
    /// Gradient norm
    pub gradient_norm: f32,
    /// Parameter update magnitude
    pub update_magnitude: f32,
    /// Training stability score
    pub stability_score: f32,
    /// Generalization estimate
    pub generalization_score: f32,
}

/// Update aggregator for combining model updates
pub struct UpdateAggregator {
    validation_engine: UpdateValidator,
    quality_thresholds: QualityThresholds,
}

/// Quality thresholds for accepting updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub min_gradient_norm: f32,
    pub max_gradient_norm: f32,
    pub min_stability_score: f32,
    pub max_update_magnitude: f32,
}

impl ModelUpdate {
    /// Create a new model update
    pub fn new(
        participant_id: String,
        round_id: u64,
        parameters: Vec<Vec<f32>>,
        training_samples: usize,
        epochs_trained: u32,
        learning_rate: f32,
        final_loss: f32,
    ) -> Self {
        let metadata = UpdateMetadata {
            training_samples,
            epochs_trained,
            learning_rate,
            final_loss,
            created_at: chrono::Utc::now(),
            model_version: format!("round_{}", round_id),
        };

        Self {
            participant_id,
            round_id,
            parameters,
            metadata,
            quality_metrics: None,
        }
    }

    /// Compute quality metrics for this update
    pub fn compute_quality_metrics(&mut self) -> Result<()> {
        let gradient_norm = self.compute_gradient_norm();
        let update_magnitude = self.compute_update_magnitude();
        let stability_score = self.compute_stability_score();
        let generalization_score = self.estimate_generalization();

        self.quality_metrics = Some(UpdateQualityMetrics {
            gradient_norm,
            update_magnitude,
            stability_score,
            generalization_score,
        });

        Ok(())
    }

    /// Compute the norm of parameter gradients/updates
    fn compute_gradient_norm(&self) -> f32 {
        let mut sum_squares = 0.0;

        for layer in &self.parameters {
            for param in layer {
                sum_squares += param * param;
            }
        }

        sum_squares.sqrt()
    }

    /// Compute the magnitude of parameter updates
    fn compute_update_magnitude(&self) -> f32 {
        let total_params: usize = self.parameters.iter().map(|layer| layer.len()).sum();
        let sum_abs: f32 = self.parameters.iter()
            .flatten()
            .map(|param| param.abs())
            .sum();

        sum_abs / total_params as f32
    }

    /// Compute training stability score based on loss and other metrics
    fn compute_stability_score(&self) -> f32 {
        // Simple stability score based on loss (lower loss = more stable)
        // In practice, this would use more sophisticated metrics
        let loss_score = 1.0 / (1.0 + self.metadata.final_loss);
        let epoch_score = (self.metadata.epochs_trained as f32 / 10.0).min(1.0);

        (loss_score + epoch_score) / 2.0
    }

    /// Estimate generalization capability
    fn estimate_generalization(&self) -> f32 {
        // Simple estimation based on training samples and loss
        // In practice, this would use validation metrics
        let sample_score = (self.metadata.training_samples as f32 / 1000.0).min(1.0);
        let loss_penalty = self.metadata.final_loss.min(1.0);

        sample_score * (1.0 - loss_penalty)
    }

    /// Validate the update against quality thresholds
    pub fn validate_quality(&self, thresholds: &QualityThresholds) -> Result<ValidationResult> {
        let metrics = self.quality_metrics.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Quality metrics not computed"))?;

        let mut issues = Vec::new();
        let mut score = 0.0;

        // Check gradient norm
        if metrics.gradient_norm < thresholds.min_gradient_norm {
            issues.push("Gradient norm too low".to_string());
        } else if metrics.gradient_norm > thresholds.max_gradient_norm {
            issues.push("Gradient norm too high".to_string());
        } else {
            score += 0.25;
        }

        // Check stability
        if metrics.stability_score < thresholds.min_stability_score {
            issues.push("Stability score too low".to_string());
        } else {
            score += 0.25;
        }

        // Check update magnitude
        if metrics.update_magnitude > thresholds.max_update_magnitude {
            issues.push("Update magnitude too high".to_string());
        } else {
            score += 0.25;
        }

        // Check generalization
        if metrics.generalization_score < 0.5 {
            issues.push("Poor generalization estimate".to_string());
        } else {
            score += 0.25;
        }

        let is_valid = issues.is_empty();

        Ok(ValidationResult {
            is_valid,
            score,
            issues,
        })
    }

    /// Get the size of this update in bytes
    pub fn size_bytes(&self) -> usize {
        let param_size = self.parameters.iter()
            .map(|layer| layer.len() * std::mem::size_of::<f32>())
            .sum::<usize>();

        param_size + std::mem::size_of::<Self>()
    }
}

impl UpdateAggregator {
    /// Create a new update aggregator
    pub fn new(quality_thresholds: QualityThresholds) -> Self {
        Self {
            validation_engine: UpdateValidator::new(),
            quality_thresholds,
        }
    }

    /// Aggregate multiple model updates
    pub async fn aggregate_updates(&self, updates: Vec<ModelUpdate>) -> Result<ModelUpdate> {
        if updates.is_empty() {
            return Err(anyhow::anyhow!("No updates to aggregate"));
        }

        info!("Aggregating {} model updates", updates.len());

        // Validate all updates
        let mut valid_updates = Vec::new();
        for update in &updates {
            let validation = self.validation_engine.validate_update(update).await?;
            if validation.is_valid {
                valid_updates.push(update);
            } else {
                debug!("Rejected update from {}: {:?}", update.participant_id, validation.issues);
            }
        }

        if valid_updates.is_empty() {
            return Err(anyhow::anyhow!("No valid updates to aggregate"));
        }

        // Perform federated averaging
        let aggregated_params = self.perform_federated_averaging(&valid_updates)?;

        // Create aggregated update
        let mut aggregated_update = ModelUpdate::new(
            "federation_coordinator".to_string(),
            updates[0].round_id,
            aggregated_params,
            valid_updates.iter().map(|u| u.metadata.training_samples).sum(),
            valid_updates.iter().map(|u| u.metadata.epochs_trained).max().unwrap_or(1),
            valid_updates.iter().map(|u| u.metadata.learning_rate).sum::<f32>() / valid_updates.len() as f32,
            valid_updates.iter().map(|u| u.metadata.final_loss).sum::<f32>() / valid_updates.len() as f32,
        );

        // Compute quality metrics for aggregated update
        aggregated_update.compute_quality_metrics()?;

        Ok(aggregated_update)
    }

    /// Perform federated averaging of parameters
    fn perform_federated_averaging(&self, updates: &[&ModelUpdate]) -> Result<Vec<Vec<f32>>> {
        if updates.is_empty() {
            return Err(anyhow::anyhow!("No updates for averaging"));
        }

        let num_updates = updates.len() as f32;
        let mut aggregated = updates[0].parameters.clone();

        // Sum all parameter updates
        for update in updates.iter().skip(1) {
            for (layer_idx, layer) in update.parameters.iter().enumerate() {
                if layer_idx < aggregated.len() {
                    for (param_idx, param) in layer.iter().enumerate() {
                        if param_idx < aggregated[layer_idx].len() {
                            aggregated[layer_idx][param_idx] += param;
                        }
                    }
                }
            }
        }

        // Average the parameters
        for layer in &mut aggregated {
            for param in layer {
                *param /= num_updates;
            }
        }

        Ok(aggregated)
    }

    /// Get aggregation statistics
    pub fn get_aggregation_stats(&self, updates: &[ModelUpdate]) -> AggregationStats {
        let total_updates = updates.len();
        let valid_updates = updates.iter()
            .filter(|u| u.quality_metrics.is_some())
            .count();

        let avg_quality_score = if valid_updates > 0 {
            updates.iter()
                .filter_map(|u| u.quality_metrics.as_ref())
                .map(|qm| (qm.stability_score + qm.generalization_score) / 2.0)
                .sum::<f32>() / valid_updates as f32
        } else {
            0.0
        };

        AggregationStats {
            total_updates,
            valid_updates,
            rejected_updates: total_updates - valid_updates,
            average_quality_score: avg_quality_score,
        }
    }
}

/// Result of validating an update
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub score: f32,
    pub issues: Vec<String>,
}

/// Statistics from aggregation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationStats {
    pub total_updates: usize,
    pub valid_updates: usize,
    pub rejected_updates: usize,
    pub average_quality_score: f32,
}

// Placeholder for the UpdateValidator that will be implemented in validation.rs
#[derive(Debug)]
pub struct UpdateValidator;

impl UpdateValidator {
    pub fn new() -> Self {
        Self
    }

    pub async fn validate_update(&self, update: &ModelUpdate) -> Result<ValidationResult> {
        // Basic validation - in practice this would be more comprehensive
        let is_valid = !update.parameters.is_empty() &&
                      update.metadata.training_samples > 0;

        let score = if is_valid { 1.0 } else { 0.0 };
        let issues = if is_valid {
            Vec::new()
        } else {
            vec!["Invalid update parameters".to_string()]
        };

        Ok(ValidationResult { is_valid, score, issues })
    }
}


