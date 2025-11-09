//! Auto-Tuner Framework
//!
//! Implements Bayesian optimization for continuous parameter tuning to optimize
//! runtime performance while maintaining CAWS compliance.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use tracing::info;
use chrono::Utc;

/// Optimization parameter space
#[derive(Debug, Clone)]
pub struct ParameterSpace {
    /// Parameter name
    pub name: String,
    
    /// Minimum value
    pub min: f64,
    
    /// Maximum value
    pub max: f64,
    
    /// Current value
    pub current: f64,
    
    /// Step size for exploration
    pub step_size: f64,
}

/// Optimization objective
#[derive(Debug, Clone)]
pub struct OptimizationObjective {
    /// Objective name
    pub name: String,
    
    /// Target value (for minimization, this is the minimum acceptable)
    pub target: f64,
    
    /// Weight in multi-objective optimization (0.0 - 1.0)
    pub weight: f64,
    
    /// Whether to minimize (true) or maximize (false)
    pub minimize: bool,
}

/// Performance measurement
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    /// Measurement timestamp
    pub timestamp: chrono::DateTime<Utc>,
    
    /// Parameter values used
    pub parameters: HashMap<String, f64>,
    
    /// Objective values achieved
    pub objectives: HashMap<String, f64>,
    
    /// CAWS compliance score (0.0 - 1.0)
    pub caws_compliance: f64,
    
    /// Overall performance score
    pub performance_score: f64,
}

/// Bayesian optimization configuration
#[derive(Debug, Clone)]
pub struct BayesianOptimizationConfig {
    /// Number of random samples before optimization
    pub initial_samples: usize,
    
    /// Maximum number of optimization iterations
    pub max_iterations: usize,
    
    /// Exploration-exploitation balance (0.0 = pure exploitation, 1.0 = pure exploration)
    pub exploration_weight: f64,
    
    /// Convergence threshold (stop if improvement < threshold)
    pub convergence_threshold: f64,
}

impl Default for BayesianOptimizationConfig {
    fn default() -> Self {
        Self {
            initial_samples: 10,
            max_iterations: 100,
            exploration_weight: 0.3,
            convergence_threshold: 0.01,
        }
    }
}

/// Auto-tuner for continuous parameter optimization
pub struct AutoTuner {
    /// Parameter spaces to optimize
    parameter_spaces: Vec<ParameterSpace>,
    
    /// Optimization objectives
    objectives: Vec<OptimizationObjective>,
    
    /// Bayesian optimization configuration
    config: BayesianOptimizationConfig,
    
    /// Performance measurement history
    measurements: Arc<tokio::sync::RwLock<Vec<PerformanceMeasurement>>>,
    
    /// Best parameters found so far
    best_parameters: Arc<tokio::sync::RwLock<HashMap<String, f64>>>,
    
    /// Best performance score achieved
    best_score: Arc<tokio::sync::RwLock<f64>>,
}

impl AutoTuner {
    /// Create a new auto-tuner
    pub fn new(
        parameter_spaces: Vec<ParameterSpace>,
        objectives: Vec<OptimizationObjective>,
        config: BayesianOptimizationConfig,
    ) -> Self {
        Self {
            parameter_spaces,
            objectives,
            config,
            measurements: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            best_parameters: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            best_score: Arc::new(tokio::sync::RwLock::new(0.0)),
        }
    }

    /// Suggest next parameter values using Bayesian optimization
    pub async fn suggest_next_parameters(&self) -> Result<HashMap<String, f64>> {
        let measurements = self.measurements.read().await;

        // If we don't have enough samples, use random exploration
        if measurements.len() < self.config.initial_samples {
            return Ok(self.random_sample());
        }

        // Use acquisition function to balance exploration and exploitation
        let suggested = self.acquisition_function(&measurements).await?;

        Ok(suggested)
    }

    /// Record performance measurement
    pub async fn record_measurement(&self, measurement: PerformanceMeasurement) -> Result<()> {
        let mut measurements = self.measurements.write().await;
        measurements.push(measurement.clone());

        // Update best parameters if this is better
        let current_score = measurement.performance_score;
        let mut best_score = self.best_score.write().await;
        
        if current_score > *best_score {
            *best_score = current_score;
            let mut best_params = self.best_parameters.write().await;
            *best_params = measurement.parameters.clone();
            
            info!(
                "New best performance score: {:.4} with parameters: {:?}",
                current_score, measurement.parameters
            );
        }

        Ok(())
    }

    /// Get best parameters found so far
    pub async fn get_best_parameters(&self) -> HashMap<String, f64> {
        self.best_parameters.read().await.clone()
    }

    /// Get best performance score
    pub async fn get_best_score(&self) -> f64 {
        *self.best_score.read().await
    }

    /// Random sample for initial exploration
    fn random_sample(&self) -> HashMap<String, f64> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut params = HashMap::new();

        for space in &self.parameter_spaces {
            let value = rng.gen_range(space.min..=space.max);
            params.insert(space.name.clone(), value);
        }

        params
    }

    /// Acquisition function for Bayesian optimization
    async fn acquisition_function(
        &self,
        measurements: &[PerformanceMeasurement],
    ) -> Result<HashMap<String, f64>> {
        // TODO: Implement proper Upper Confidence Bound (UCB) acquisition function with Gaussian Process
        //       Currently uses basic UCB; should use Gaussian Process model for accurate acquisition function.
        
        // Calculate mean and variance for each parameter
        let mut parameter_stats: HashMap<String, (f64, f64)> = HashMap::new();

        for space in &self.parameter_spaces {
            let values: Vec<f64> = measurements
                .iter()
                .filter_map(|m| m.parameters.get(&space.name).copied())
                .collect();

            if !values.is_empty() {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values
                    .iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>()
                    / values.len() as f64;
                parameter_stats.insert(space.name.clone(), (mean, variance));
            }
        }

        // Generate next sample using UCB
        
        let rng = rand::thread_rng();
        let mut suggested = HashMap::new();

        for space in &self.parameter_spaces {
            let (mean, variance) = parameter_stats
                .get(&space.name)
                .copied()
                .unwrap_or((space.current, 1.0));

            // UCB: mean + exploration_weight * sqrt(variance)
            let exploration_term = self.config.exploration_weight * variance.sqrt();
            let ucb_value = mean + exploration_term;

            // Clamp to parameter bounds
            let value = ucb_value.max(space.min).min(space.max);
            suggested.insert(space.name.clone(), value);
        }

        Ok(suggested)
    }

    /// Check if optimization has converged
    pub async fn has_converged(&self) -> bool {
        let measurements = self.measurements.read().await;

        if measurements.len() < self.config.initial_samples {
            return false;
        }

        // Check if recent improvements are below threshold
        let recent_count = 10.min(measurements.len());
        let recent_scores: Vec<f64> = measurements
            .iter()
            .rev()
            .take(recent_count)
            .map(|m| m.performance_score)
            .collect();

        if recent_scores.len() < 2 {
            return false;
        }

        let max_score = recent_scores.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_score = recent_scores.iter().fold(f64::MAX, |a, &b| a.min(b));
        let improvement = max_score - min_score;

        improvement < self.config.convergence_threshold
    }

    /// Calculate multi-objective performance score
    pub fn calculate_performance_score(
        &self,
        objectives: &HashMap<String, f64>,
        caws_compliance: f64,
    ) -> f64 {
        // Weighted sum of objectives
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for objective in &self.objectives {
            if let Some(&value) = objectives.get(&objective.name) {
                let normalized_value = if objective.minimize {
                    // For minimization, invert the value
                    objective.target / value.max(0.001) // Avoid division by zero
                } else {
                    // For maximization, use value directly
                    value / objective.target.max(0.001)
                };

                weighted_sum += normalized_value * objective.weight;
                total_weight += objective.weight;
            }
        }

        // Include CAWS compliance as a mandatory constraint
        // If CAWS compliance is below threshold, heavily penalize
        let caws_penalty = if caws_compliance < 0.8 {
            0.5 // Heavy penalty for non-compliance
        } else {
            1.0 // No penalty for compliance
        };

        if total_weight > 0.0 {
            (weighted_sum / total_weight) * caws_penalty
        } else {
            caws_compliance // Fallback to CAWS compliance only
        }
    }

    /// Get optimization statistics
    pub async fn get_statistics(&self) -> OptimizationStatistics {
        let measurements = self.measurements.read().await;
        let best_score = *self.best_score.read().await;

        OptimizationStatistics {
            total_measurements: measurements.len(),
            best_score,
            convergence_status: self.has_converged().await,
        }
    }
}

/// Optimization statistics
#[derive(Debug, Clone)]
pub struct OptimizationStatistics {
    pub total_measurements: usize,
    pub best_score: f64,
    pub convergence_status: bool,
}

