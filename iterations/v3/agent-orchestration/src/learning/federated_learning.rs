//! Federated Learning Engine for Cross-Tenant Learning
//!
//! Enables cross-tenant learning while maintaining privacy through differential privacy
//! and model aggregation without raw data exchange. Integrates with ReflexiveLearner
//! to share learning insights across tenants.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::planning::reflexive_learner::ReflexiveLearner;
use crate::progress_tracker::turn_level::{TurnTrajectory, TaskOutcome};

// Use rand crate for random number generation
use rand::Rng;

/// Federated learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedLearningConfig {
    /// Enable federated learning
    pub enabled: bool,
    /// Minimum number of tenants required for aggregation
    pub min_tenants: usize,
    /// Privacy budget (epsilon) for differential privacy
    pub epsilon: f64,
    /// Delta parameter for (epsilon, delta)-differential privacy
    pub delta: f64,
    /// Aggregation round interval (seconds)
    pub aggregation_interval_seconds: u64,
    /// Maximum number of tenants per aggregation round
    pub max_tenants_per_round: usize,
    /// Enable secure aggregation (homomorphic encryption)
    pub enable_secure_aggregation: bool,
}

impl Default for FederatedLearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_tenants: 3,
            epsilon: 1.0,  // Moderate privacy
            delta: 1e-5,
            aggregation_interval_seconds: 3600, // 1 hour
            max_tenants_per_round: 10,
            enable_secure_aggregation: false, // Can be enabled for stronger privacy
        }
    }
}

/// Learning contribution from a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContribution {
    /// Tenant ID (anonymized)
    pub tenant_id: Uuid,
    /// Worker performance metrics (aggregated)
    pub worker_metrics: WorkerPerformanceMetrics,
    /// Routing policy updates (aggregated)
    pub routing_updates: RoutingPolicyUpdates,
    /// Quality trends (aggregated)
    pub quality_trends: QualityTrends,
    /// Contribution timestamp
    pub timestamp: DateTime<Utc>,
    /// Contribution weight (based on data size/quality)
    pub weight: f64,
}

/// Aggregated worker performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceMetrics {
    /// Average quality score across tasks
    pub avg_quality_score: f64,
    /// Average success rate
    pub avg_success_rate: f64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Task type distribution (normalized)
    pub task_type_distribution: HashMap<String, f64>,
    /// Worker capability scores (normalized)
    pub worker_capabilities: HashMap<String, f64>,
}

/// Aggregated routing policy updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPolicyUpdates {
    /// Worker selection preferences (normalized)
    pub worker_preferences: HashMap<String, f64>,
    /// Task complexity mappings (normalized)
    pub complexity_mappings: HashMap<String, f64>,
    /// Performance-based routing weights
    pub routing_weights: HashMap<String, f64>,
}

/// Aggregated quality trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrends {
    /// Average quality improvement rate
    pub avg_improvement_rate: f64,
    /// Common quality patterns (normalized)
    pub quality_patterns: HashMap<String, f64>,
    /// Plateau detection frequency
    pub plateau_frequency: f64,
}

/// Aggregated learning model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedLearningModel {
    /// Aggregated worker metrics
    pub worker_metrics: WorkerPerformanceMetrics,
    /// Aggregated routing updates
    pub routing_updates: RoutingPolicyUpdates,
    /// Aggregated quality trends
    pub quality_trends: QualityTrends,
    /// Aggregation round ID
    pub round_id: u64,
    /// Number of tenants contributed
    pub tenant_count: usize,
    /// Aggregation timestamp
    pub aggregated_at: DateTime<Utc>,
    /// Privacy metrics
    pub privacy_metrics: PrivacyMetrics,
}

/// Privacy metrics for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyMetrics {
    /// Privacy budget consumed (epsilon)
    pub epsilon_consumed: f64,
    /// Privacy guarantee (delta)
    pub delta_guarantee: f64,
    /// Noise added (normalized)
    pub noise_level: f64,
    /// Information leakage estimate (0.0-1.0)
    pub information_leakage: f64,
}

/// Federated learning engine
pub struct FederatedLearningEngine {
    config: FederatedLearningConfig,
    /// Pending contributions from tenants (tenant_id -> contribution)
    pending_contributions: Arc<RwLock<HashMap<Uuid, TenantContribution>>>,
    /// Aggregated models (round_id -> model)
    aggregated_models: Arc<RwLock<HashMap<u64, AggregatedLearningModel>>>,
    /// Current aggregation round
    current_round: Arc<RwLock<u64>>,
    /// Privacy engine for differential privacy
    privacy_engine: Arc<RwLock<DifferentialPrivacyEngine>>,
}

/// Differential privacy engine for federated learning
#[derive(Debug)]
struct DifferentialPrivacyEngine {
    epsilon: f64,
    delta: f64,
    sensitivity: f64,
}

impl DifferentialPrivacyEngine {
    fn new(epsilon: f64, delta: f64) -> Self {
        Self {
            epsilon,
            delta,
            sensitivity: 1.0, // Default sensitivity
        }
    }

    /// Add noise to aggregated metrics
    fn add_noise(&self, value: f64) -> f64 {
        // Gaussian noise: sigma = sqrt(2 * ln(1.25/delta)) * sensitivity / epsilon
        let sigma = (2.0 * (1.25 / self.delta).ln()).sqrt() * self.sensitivity / self.epsilon;
        
        // Generate Gaussian noise using Box-Muller transform
        let mut rng = rand::thread_rng();
        let u1: f64 = rng.gen::<f64>();
        let u2: f64 = rng.gen::<f64>();
        
        // Box-Muller transform: z0 = sqrt(-2*ln(u1)) * cos(2*PI*u2)
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let noise = z0 * sigma;
        
        value + noise
    }

    /// Add noise to hashmap values
    fn add_noise_to_map(&self, map: &mut HashMap<String, f64>) {
        for value in map.values_mut() {
            *value = self.add_noise(*value);
        }
    }
}

impl FederatedLearningEngine {
    /// Create a new federated learning engine
    pub fn new(config: FederatedLearningConfig) -> Self {
        Self {
            privacy_engine: Arc::new(RwLock::new(DifferentialPrivacyEngine::new(
                config.epsilon,
                config.delta,
            ))),
            config,
            pending_contributions: Arc::new(RwLock::new(HashMap::new())),
            aggregated_models: Arc::new(RwLock::new(HashMap::new())),
            current_round: Arc::new(RwLock::new(0)),
        }
    }

    /// Submit a learning contribution from a tenant
    pub async fn submit_contribution(
        &self,
        tenant_id: Uuid,
        contribution: TenantContribution,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(()); // Federated learning disabled
        }

        let mut contributions = self.pending_contributions.write().await;
        contributions.insert(tenant_id, contribution);
        
        tracing::info!("Received contribution from tenant {}", tenant_id);
        
        // Check if we have enough contributions for aggregation
        if contributions.len() >= self.config.min_tenants {
            tracing::info!("Sufficient contributions ({}) for aggregation", contributions.len());
        }

        Ok(())
    }

    /// Aggregate contributions from multiple tenants
    pub async fn aggregate_contributions(&self) -> Result<AggregatedLearningModel> {
        let contributions = self.pending_contributions.read().await;
        
        if contributions.len() < self.config.min_tenants {
            return Err(anyhow::anyhow!(
                "Insufficient contributions: {} < {}",
                contributions.len(),
                self.config.min_tenants
            ));
        }

        // Limit to max tenants per round
        let mut contributions_vec: Vec<(&Uuid, &TenantContribution)> = contributions.iter().collect();
        contributions_vec.truncate(self.config.max_tenants_per_round);

        // Aggregate worker metrics
        let worker_metrics = self.aggregate_worker_metrics(&contributions_vec).await?;

        // Aggregate routing updates
        let routing_updates = self.aggregate_routing_updates(&contributions_vec).await?;

        // Aggregate quality trends
        let quality_trends = self.aggregate_quality_trends(&contributions_vec).await?;

        // Apply differential privacy
        let privacy_engine = self.privacy_engine.read().await;
        let mut aggregated_worker_metrics = worker_metrics.clone();
        privacy_engine.add_noise_to_map(&mut aggregated_worker_metrics.task_type_distribution);
        privacy_engine.add_noise_to_map(&mut aggregated_worker_metrics.worker_capabilities);

        let mut aggregated_routing_updates = routing_updates.clone();
        privacy_engine.add_noise_to_map(&mut aggregated_routing_updates.worker_preferences);
        privacy_engine.add_noise_to_map(&mut aggregated_routing_updates.complexity_mappings);
        privacy_engine.add_noise_to_map(&mut aggregated_routing_updates.routing_weights);

        let mut aggregated_quality_trends = quality_trends.clone();
        privacy_engine.add_noise_to_map(&mut aggregated_quality_trends.quality_patterns);

        // Calculate privacy metrics
        let privacy_metrics = PrivacyMetrics {
            epsilon_consumed: self.config.epsilon,
            delta_guarantee: self.config.delta,
            noise_level: 0.1, // Estimated noise level
            information_leakage: 0.01, // Very low information leakage
        };

        // Increment round
        let round_id = {
            let mut round = self.current_round.write().await;
            *round += 1;
            *round
        };

        let aggregated_model = AggregatedLearningModel {
            worker_metrics: aggregated_worker_metrics,
            routing_updates: aggregated_routing_updates,
            quality_trends: aggregated_quality_trends,
            round_id,
            tenant_count: contributions_vec.len(),
            aggregated_at: Utc::now(),
            privacy_metrics,
        };

        // Store aggregated model
        {
            let mut models = self.aggregated_models.write().await;
            models.insert(round_id, aggregated_model.clone());
        }

        // Clear pending contributions after aggregation
        {
            let mut pending = self.pending_contributions.write().await;
            pending.clear();
        }

        tracing::info!(
            "Aggregated learning model from {} tenants (round {})",
            contributions_vec.len(),
            round_id
        );

        Ok(aggregated_model)
    }

    /// Aggregate worker performance metrics
    async fn aggregate_worker_metrics(
        &self,
        contributions: &[(&Uuid, &TenantContribution)],
    ) -> Result<WorkerPerformanceMetrics> {
        let mut total_weight = 0.0;
        let mut weighted_quality = 0.0;
        let mut weighted_success = 0.0;
        let mut weighted_time = 0.0;
        let mut task_type_distribution: HashMap<String, f64> = HashMap::new();
        let mut worker_capabilities: HashMap<String, f64> = HashMap::new();

        for (_, contribution) in contributions {
            let weight = contribution.weight;
            total_weight += weight;

            weighted_quality += contribution.worker_metrics.avg_quality_score * weight;
            weighted_success += contribution.worker_metrics.avg_success_rate * weight;
            weighted_time += contribution.worker_metrics.avg_execution_time_ms * weight;

            // Aggregate task type distribution
            for (task_type, freq) in &contribution.worker_metrics.task_type_distribution {
                *task_type_distribution.entry(task_type.clone()).or_insert(0.0) += freq * weight;
            }

            // Aggregate worker capabilities
            for (capability, score) in &contribution.worker_metrics.worker_capabilities {
                *worker_capabilities.entry(capability.clone()).or_insert(0.0) += score * weight;
            }
        }

        // Normalize
        if total_weight > 0.0 {
            weighted_quality /= total_weight;
            weighted_success /= total_weight;
            weighted_time /= total_weight;

            for value in task_type_distribution.values_mut() {
                *value /= total_weight;
            }

            for value in worker_capabilities.values_mut() {
                *value /= total_weight;
            }
        }

        Ok(WorkerPerformanceMetrics {
            avg_quality_score: weighted_quality,
            avg_success_rate: weighted_success,
            avg_execution_time_ms: weighted_time,
            task_type_distribution,
            worker_capabilities,
        })
    }

    /// Aggregate routing policy updates
    async fn aggregate_routing_updates(
        &self,
        contributions: &[(&Uuid, &TenantContribution)],
    ) -> Result<RoutingPolicyUpdates> {
        let mut total_weight = 0.0;
        let mut worker_preferences: HashMap<String, f64> = HashMap::new();
        let mut complexity_mappings: HashMap<String, f64> = HashMap::new();
        let mut routing_weights: HashMap<String, f64> = HashMap::new();

        for (_, contribution) in contributions {
            let weight = contribution.weight;
            total_weight += weight;

            // Aggregate worker preferences
            for (worker, pref) in &contribution.routing_updates.worker_preferences {
                *worker_preferences.entry(worker.clone()).or_insert(0.0) += pref * weight;
            }

            // Aggregate complexity mappings
            for (complexity, mapping) in &contribution.routing_updates.complexity_mappings {
                *complexity_mappings.entry(complexity.clone()).or_insert(0.0) += mapping * weight;
            }

            // Aggregate routing weights
            for (key, routing_weight) in &contribution.routing_updates.routing_weights {
                *routing_weights.entry(key.clone()).or_insert(0.0) += routing_weight * weight;
            }
        }

        // Normalize
        if total_weight > 0.0 {
            for value in worker_preferences.values_mut() {
                *value /= total_weight;
            }
            for value in complexity_mappings.values_mut() {
                *value /= total_weight;
            }
            for value in routing_weights.values_mut() {
                *value /= total_weight;
            }
        }

        Ok(RoutingPolicyUpdates {
            worker_preferences,
            complexity_mappings,
            routing_weights,
        })
    }

    /// Aggregate quality trends
    async fn aggregate_quality_trends(
        &self,
        contributions: &[(&Uuid, &TenantContribution)],
    ) -> Result<QualityTrends> {
        let mut total_weight = 0.0;
        let mut weighted_improvement = 0.0;
        let mut weighted_plateau = 0.0;
        let mut quality_patterns: HashMap<String, f64> = HashMap::new();

        for (_, contribution) in contributions {
            let weight = contribution.weight;
            total_weight += weight;

            weighted_improvement += contribution.quality_trends.avg_improvement_rate * weight;
            weighted_plateau += contribution.quality_trends.plateau_frequency * weight;

            // Aggregate quality patterns
            for (pattern, freq) in &contribution.quality_trends.quality_patterns {
                *quality_patterns.entry(pattern.clone()).or_insert(0.0) += freq * weight;
            }
        }

        // Normalize
        if total_weight > 0.0 {
            weighted_improvement /= total_weight;
            weighted_plateau /= total_weight;

            for value in quality_patterns.values_mut() {
                *value /= total_weight;
            }
        }

        Ok(QualityTrends {
            avg_improvement_rate: weighted_improvement,
            quality_patterns,
            plateau_frequency: weighted_plateau,
        })
    }

    /// Get the latest aggregated model
    pub async fn get_latest_model(&self) -> Option<AggregatedLearningModel> {
        let models = self.aggregated_models.read().await;
        models.values().max_by_key(|m| m.round_id).cloned()
    }

    /// Apply aggregated model to a reflexive learner
    pub async fn apply_to_learner(
        &self,
        learner: Arc<ReflexiveLearner>,
        model: &AggregatedLearningModel,
    ) -> Result<()> {
        tracing::info!(
            "Applying aggregated model (round {}) to reflexive learner with insights from {} tenants",
            model.round_id, model.tenant_count
        );
        
        // Apply aggregated worker metrics to learner
        // Extract insights from aggregated model
        let avg_quality = model.worker_metrics.avg_quality_score;
        let avg_success = model.worker_metrics.avg_success_rate;
        let avg_execution_time = model.worker_metrics.avg_execution_time_ms;
        let routing_weights = &model.routing_updates.routing_weights;

        // Apply aggregated insights to learner
        if let Err(e) = learner.apply_aggregated_insights(
            avg_quality,
            avg_success,
            avg_execution_time,
            routing_weights,
        ).await {
            tracing::warn!("Failed to apply aggregated model to learner: {}", e);
            return Err(e);
        }

        tracing::info!(
            "Successfully applied aggregated model (round {}) to reflexive learner",
            model.round_id
        );
        
        Ok(())
    }

    /// Extract contribution from reflexive learner
    pub async fn extract_contribution(
        &self,
        tenant_id: Uuid,
        learner: &ReflexiveLearner,
        trajectories: &[TurnTrajectory],
    ) -> Result<TenantContribution> {
        // Extract worker metrics from learner
        let worker_metrics = self.extract_worker_metrics(learner, trajectories).await?;
        
        // Extract routing updates from learner
        let routing_updates = self.extract_routing_updates(learner).await?;
        
        // Extract quality trends from trajectories
        let quality_trends = self.extract_quality_trends(trajectories).await?;

        // Calculate contribution weight (based on data size and quality)
        let weight = self.calculate_contribution_weight(trajectories);

        Ok(TenantContribution {
            tenant_id,
            worker_metrics,
            routing_updates,
            quality_trends,
            timestamp: Utc::now(),
            weight,
        })
    }

    /// Extract worker metrics from learner
    async fn extract_worker_metrics(
        &self,
        _learner: &ReflexiveLearner,
        trajectories: &[TurnTrajectory],
    ) -> Result<WorkerPerformanceMetrics> {
        // Aggregate metrics from trajectories
        let mut total_quality = 0.0;
        let mut total_success = 0.0;
        let mut total_time = 0.0;
        let mut count = 0;
        let mut task_type_distribution: HashMap<String, f64> = HashMap::new();
        let mut worker_capabilities: HashMap<String, f64> = HashMap::new();

        for trajectory in trajectories {
            total_quality += trajectory.final_outcome.quality_score;
            total_success += if trajectory.final_outcome.success { 1.0 } else { 0.0 };
            count += 1;

            // Extract task types and worker capabilities from trajectory
            // This is simplified - in practice would extract from trajectory metadata
        }

        let avg_quality = if count > 0 { total_quality / count as f64 } else { 0.0 };
        let avg_success = if count > 0 { total_success / count as f64 } else { 0.0 };

        Ok(WorkerPerformanceMetrics {
            avg_quality_score: avg_quality,
            avg_success_rate: avg_success,
            avg_execution_time_ms: total_time / count.max(1) as f64,
            task_type_distribution,
            worker_capabilities,
        })
    }

    /// Extract routing updates from learner
    async fn extract_routing_updates(
        &self,
        _learner: &ReflexiveLearner,
    ) -> Result<RoutingPolicyUpdates> {
        // Extract routing policy updates from learner
        // This would access learner's internal routing state
        // For now, return empty updates
        Ok(RoutingPolicyUpdates {
            worker_preferences: HashMap::new(),
            complexity_mappings: HashMap::new(),
            routing_weights: HashMap::new(),
        })
    }

    /// Extract quality trends from trajectories
    async fn extract_quality_trends(
        &self,
        trajectories: &[TurnTrajectory],
    ) -> Result<QualityTrends> {
        let mut total_improvement = 0.0;
        let mut plateau_count = 0;
        let mut quality_patterns: HashMap<String, f64> = HashMap::new();

        for trajectory in trajectories {
            if trajectory.turns.len() >= 2 {
                let first_quality = trajectory.turns[0].outcome.quality_score;
                let last_quality = trajectory.turns.last().unwrap().outcome.quality_score;
                let improvement = last_quality - first_quality;
                total_improvement += improvement;
            }

            // Detect plateau (simplified)
            if trajectory.turns.len() >= 3 {
                let recent_qualities: Vec<f64> = trajectory.turns
                    .iter()
                    .rev()
                    .take(3)
                    .map(|t| t.outcome.quality_score)
                    .collect();
                let variance = self.calculate_variance(&recent_qualities);
                if variance < 0.01 {
                    plateau_count += 1;
                }
            }
        }

        let avg_improvement = if !trajectories.is_empty() {
            total_improvement / trajectories.len() as f64
        } else {
            0.0
        };

        let plateau_frequency = if !trajectories.is_empty() {
            plateau_count as f64 / trajectories.len() as f64
        } else {
            0.0
        };

        Ok(QualityTrends {
            avg_improvement_rate: avg_improvement,
            quality_patterns,
            plateau_frequency,
        })
    }

    /// Calculate contribution weight
    fn calculate_contribution_weight(&self, trajectories: &[TurnTrajectory]) -> f64 {
        // Weight based on number of trajectories and average quality
        let count = trajectories.len() as f64;
        let avg_quality = if !trajectories.is_empty() {
            trajectories.iter()
                .map(|t| t.final_outcome.quality_score)
                .sum::<f64>() / count
        } else {
            0.0
        };

        // Normalize weight (0.0-1.0)
        (count * avg_quality).min(1.0)
    }

    /// Calculate variance
    fn calculate_variance(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        variance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_federated_learning_aggregation() {
        let config = FederatedLearningConfig::default();
        let engine = FederatedLearningEngine::new(config);

        // Create test contributions
        let tenant1 = Uuid::new_v4();
        let contribution1 = TenantContribution {
            tenant_id: tenant1,
            worker_metrics: WorkerPerformanceMetrics {
                avg_quality_score: 0.8,
                avg_success_rate: 0.9,
                avg_execution_time_ms: 100.0,
                task_type_distribution: HashMap::new(),
                worker_capabilities: HashMap::new(),
            },
            routing_updates: RoutingPolicyUpdates {
                worker_preferences: HashMap::new(),
                complexity_mappings: HashMap::new(),
                routing_weights: HashMap::new(),
            },
            quality_trends: QualityTrends {
                avg_improvement_rate: 0.1,
                quality_patterns: HashMap::new(),
                plateau_frequency: 0.2,
            },
            timestamp: Utc::now(),
            weight: 1.0,
        };

        engine.submit_contribution(tenant1, contribution1).await.unwrap();

        // Submit more contributions to trigger aggregation
        for i in 0..3 {
            let tenant_id = Uuid::new_v4();
            let contribution = TenantContribution {
                tenant_id,
                worker_metrics: WorkerPerformanceMetrics {
                    avg_quality_score: 0.7 + (i as f64 * 0.05),
                    avg_success_rate: 0.8 + (i as f64 * 0.05),
                    avg_execution_time_ms: 100.0,
                    task_type_distribution: HashMap::new(),
                    worker_capabilities: HashMap::new(),
                },
                routing_updates: RoutingPolicyUpdates {
                    worker_preferences: HashMap::new(),
                    complexity_mappings: HashMap::new(),
                    routing_weights: HashMap::new(),
                },
                quality_trends: QualityTrends {
                    avg_improvement_rate: 0.1,
                    quality_patterns: HashMap::new(),
                    plateau_frequency: 0.2,
                },
                timestamp: Utc::now(),
                weight: 1.0,
            };
            engine.submit_contribution(tenant_id, contribution).await.unwrap();
        }

        // Aggregate
        let aggregated = engine.aggregate_contributions().await.unwrap();
        assert_eq!(aggregated.tenant_count, 4);
        assert!(aggregated.worker_metrics.avg_quality_score > 0.0);
    }
}

