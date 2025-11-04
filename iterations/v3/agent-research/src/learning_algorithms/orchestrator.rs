//! Learning orchestrator for coordinating algorithm selection and execution

use schemars::JsonSchema;
use crate::reflexive_types::{LearningAlgorithmType, LearningDataPoint, LearningInput, LearningOutput, LearningFeedback, ProblemCharacteristics, AlgorithmPerformance, LearningSystemHealth, EnsembleAnalytics as TypesEnsembleAnalytics, EnsembleComponentStatistics, ComponentContribution, AlgorithmPerformanceTracker, LearningStrategy};
use super::reinforcement::*;
use super::supervised::*;
use super::unsupervised::*;
use super::ensemble::{LearningAlgorithms, LearningAlgorithm, EnsembleAnalytics, ProblemCharacteristicsAnalyzer};
use chrono::{DateTime, Utc};
use tracing::{info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Learning orchestrator that coordinates algorithm selection and execution

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LearningOrchestratorr {
    /// Available learning algorithms
    algorithms: HashMap<LearningAlgorithmType, Box<dyn LearningAlgorithm>>,
    /// Performance tracker
    performance_tracker: AlgorithmPerformanceTracker,
    /// Problem characteristics analyzer
    characteristics_analyzer: ProblemCharacteristicsAnalyzer,
    /// Ensemble analytics
    ensemble_analytics: EnsembleAnalytics,
    /// Learning system health monitor
    health_monitor: LearningSystemHealth,
}

impl LearningOrchestrator {
    /// Create a new learning orchestrator
    pub fn new() -> Self {
        Self {
            algorithms: HashMap::new(),
            performance_tracker: AlgorithmPerformanceTracker::new(),
            characteristics_analyzer: ProblemCharacteristicsAnalyzer::new(),
            ensemble_analytics: EnsembleAnalytics::new(),
            health_monitor: LearningSystemHealth::new(),
        }
    }

    /// Register a learning algorithm
    pub fn register_algorithm<A: LearningAlgorithm + 'static>(
        &mut self,
        algorithm: A,
    ) -> Result<(), String> {
        let algorithm_type = algorithm.algorithm_type();
        if self.algorithms.contains_key(&algorithm_type) {
            return Err(format!("Algorithm type {:?} already registered", algorithm_type));
        }

        self.algorithms.insert(algorithm_type, Box::new(algorithm));
        Ok(())
    }

    /// Select the best algorithm for a given problem
    pub fn select_algorithm(&self, problem: &ProblemCharacteristics) -> Option<LearningAlgorithmType> {
        // Simple algorithm selection based on problem characteristics
        // In a real implementation, this would use ML to select the best algorithm

        if problem.is_regression {
            Some(LearningAlgorithmType::SupervisedLearning)
        } else if problem.sample_count > 1000 {
            Some(LearningAlgorithmType::EnsembleLearning)
        } else if problem.estimated_complexity > 0.7 {
            Some(LearningAlgorithmType::DeepReinforcementLearning)
        } else {
            Some(LearningAlgorithmType::ReinforcementLearning)
        }
    }

    /// Execute learning on a data point
    pub async fn execute_learning(
        &mut self,
        data_point: &LearningDataPoint,
    ) -> Result<LearningOutput, String> {
        let problem_id = format!("{:?}_{}", data_point.input, data_point.expected_output);
        let characteristics = self.characteristics_analyzer.analyze(&problem_id, data_point);

        let algorithm_type = self.select_algorithm(&characteristics)
            .ok_or_else(|| "No suitable algorithm found".to_string())?;

        let algorithm = self.algorithms.get_mut(&algorithm_type)
            .ok_or_else(|| format!("Algorithm {:?} not registered", algorithm_type))?;

        // Train the algorithm (simplified - in practice would batch)
        algorithm.train(&[data_point.clone()])?;

        // Make prediction
        let prediction = algorithm.predict(&data_point.input)?;

        // Record performance
        let performance = algorithm.get_performance_metrics();
        self.performance_tracker.record_performance(performance);

        Ok(prediction)
    }

    /// Get system health status
    pub fn get_system_health(&self) -> &LearningSystemHealth {
        &self.health_monitor
    }

    /// Get performance metrics for all algorithms
    pub fn get_performance_metrics(&self) -> HashMap<LearningAlgorithmType, Option<AlgorithmPerformance>> {
        self.algorithms.keys()
            .map(|algorithm_type| {
                let performance = self.performance_tracker.get_average_performance(algorithm_type);
                (algorithm_type.clone(), performance)
            })
            .collect()
    }

    /// Get ensemble analytics
    pub fn get_ensemble_analytics(&self) -> &EnsembleAnalytics {
        &self.ensemble_analytics
    }

    /// Update algorithms based on feedback
    pub fn process_feedback(&mut self, feedback: &LearningFeedback) -> Result<(), String> {
        // Find the algorithm that made the prediction
        for algorithm in self.algorithms.values_mut() {
            algorithm.update_from_feedback(feedback)?;
        }

        Ok(())
    }
}

impl Default for LearningOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning system health monitor

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningSystemHealthh {
    pub algorithm_count: usize,
    pub total_training_sessions: u64,
    pub average_performance: f64,
    pub system_uptime_seconds: u64,
    pub memory_usage_mb: f64,
    ##[schemars(with = "String")]

    pub last_health_check: DateTime<Utc>,
}

impl LearningSystemHealth {
    pub fn new() -> Self {
        Self {
            algorithm_count: 0,
            total_training_sessions: 0,
            average_performance: 0.0,
            system_uptime_seconds: 0,
            memory_usage_mb: 0.0,
            last_health_check: chrono::Utc::now(),
        }
    }

    /// Check if the system is healthy
    pub fn is_healthy(&self) -> bool {
        self.algorithm_count > 0 &&
        self.average_performance > 0.5 &&
        self.memory_usage_mb < 1000.0 // Less than 1GB
    }

    /// Get health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let mut score = 0.0;

        // Algorithm availability (30%)
        if self.algorithm_count > 0 {
            score += 0.3 * (self.algorithm_count as f64 / 5.0).min(1.0);
        }

        // Performance (40%)
        score += 0.4 * self.average_performance;

        // Memory usage (20%) - lower is better
        let memory_score = if self.memory_usage_mb < 500.0 {
            1.0
        } else if self.memory_usage_mb < 1000.0 {
            0.5
        } else {
            0.0
        };
        score += 0.2 * memory_score;

        // Training activity (10%)
        let training_score = (self.total_training_sessions as f64 / 100.0).min(1.0);
        score += 0.1 * training_score;

        score.min(1.0)
    }

    /// Update health metrics
    pub fn update_metrics(&mut self, algorithm_count: usize, performance_tracker: &AlgorithmPerformanceTracker) {
        self.algorithm_count = algorithm_count;
        self.total_training_sessions += 1;
        self.last_health_check = chrono::Utc::now();

        // Calculate average performance across all algorithms
        let mut total_performance = 0.0;
        let mut count = 0;

        for algorithm_type in [
            LearningAlgorithmType::ReinforcementLearning,
            LearningAlgorithmType::SupervisedLearning,
            LearningAlgorithmType::UnsupervisedLearning,
            LearningAlgorithmType::EnsembleLearning,
        ].iter() {
            if let Some(performance) = performance_tracker.get_average_performance(algorithm_type) {
                total_performance += performance.accuracy;
                count += 1;
            }
        }

        if count > 0 {
            self.average_performance = total_performance / count as f64;
        }
    }
}

/// Meta-learning coordinator for algorithm improvement

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MetaLearningCoordinator {
    orchestrator: Arc<RwLock<LearningOrchestrator>>,
    meta_algorithms: HashMap<String, Box<dyn MetaLearningAlgorithm>>,
}

impl MetaLearningCoordinator {
    pub fn new(orchestrator: Arc<RwLock<LearningOrchestrator>>) -> Self {
        Self {
            orchestrator,
            meta_algorithms: HashMap::new(),
        }
    }

    /// Learn from algorithm performance to improve selection
    pub async fn learn_from_performance(&mut self, performance_data: &[AlgorithmPerformance]) -> Result<(), String> {
        // Meta-learning: analyze which algorithms work best for different problem types
        // This would train a meta-model to predict algorithm performance

        for performance in performance_data {
            // Update meta-learning model
            // This is a placeholder - real implementation would train a model
            info!("Meta-learning update: {:?} achieved {:.3} accuracy",
                  performance.algorithm_type, performance.accuracy);
        }

        Ok(())
    }

    /// Predict the best algorithm for a new problem
    pub async fn predict_best_algorithm(&self, problem: &ProblemCharacteristics) -> Result<LearningAlgorithmType, String> {
        // Use meta-learning model to predict best algorithm
        // This is a placeholder - real implementation would use trained model

        // Simple rule-based fallback
        if problem.is_regression {
            Ok(LearningAlgorithmType::SupervisedLearning)
        } else if problem.estimated_complexity > 0.8 {
            Ok(LearningAlgorithmType::DeepReinforcementLearning)
        } else {
            Ok(LearningAlgorithmType::ReinforcementLearning)
        }
    }

    /// Adapt algorithm parameters based on meta-learning insights
    pub async fn adapt_algorithm_parameters(&mut self, algorithm_type: &LearningAlgorithmType, performance_history: &[AlgorithmPerformance]) -> Result<(), String> {
        // Analyze performance history to suggest parameter adjustments
        // This is a placeholder - real implementation would use optimization algorithms

        let avg_performance = performance_history.iter()
            .map(|p| p.accuracy)
            .sum::<f64>() / performance_history.len() as f64;

        if avg_performance < 0.6 {
            info!("Meta-learning suggests parameter tuning for {:?} (current performance: {:.3})",
                  algorithm_type, avg_performance);
        }

        Ok(())
    }
}

/// Trait for meta-learning algorithms
pub trait MetaLearningAlgorithm: Send + Sync {
    fn name(&self) -> &str;
    fn learn(&mut self, performance_data: &[AlgorithmPerformance]) -> Result<(), String>;
    fn predict(&self, problem: &ProblemCharacteristics) -> Result<LearningAlgorithmType, String>;
}
