//! Ensemble learning algorithms for reflexive learning

use schemars::JsonSchema;
use crate::reflexive_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ensemble learning orchestrator

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LearningAlgorithms {
    algorithms: HashMap<LearningAlgorithmType, Box<dyn LearningAlgorithm>>,
}

impl LearningAlgorithms {
    /// Create a new ensemble learning system
    pub fn new() -> Self {
        Self {
            algorithms: HashMap::new(),
        }
    }

    /// Register a learning algorithm
    pub fn register_algorithm<A: LearningAlgorithm + 'static>(
        &mut self,
        algorithm_type: LearningAlgorithmType,
        algorithm: A,
    ) -> Result<(), String> {
        if self.algorithms.contains_key(&algorithm_type) {
            return Err(format!("Algorithm type {:?} already registered", algorithm_type));
        }

        self.algorithms.insert(algorithm_type, Box::new(algorithm));
        Ok(())
    }

    /// Get a registered algorithm
    pub fn get_algorithm(&self, algorithm_type: &LearningAlgorithmType) -> Option<&dyn LearningAlgorithm> {
        self.algorithms.get(algorithm_type).map(|a| a.as_ref())
    }

    /// Get all registered algorithm types
    pub fn get_registered_types(&self) -> Vec<LearningAlgorithmType> {
        self.algorithms.keys().cloned().collect()
    }
}

/// Trait for learning algorithms
pub trait LearningAlgorithm: Send + Sync {
    /// Get the algorithm type
    fn algorithm_type(&self) -> LearningAlgorithmType;

    /// Train on learning data
    fn train(&mut self, data: &[LearningDataPoint]) -> Result<(), String>;

    /// Make predictions
    fn predict(&self, input: &LearningInput) -> Result<LearningOutput, String>;

    /// Get algorithm performance metrics
    fn get_performance_metrics(&self) -> AlgorithmPerformance;

    /// Update algorithm based on feedback
    fn update_from_feedback(&mut self, feedback: &LearningFeedback) -> Result<(), String>;
}

/// Learning data point
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningDataPoint {
    pub input: LearningInput,
    pub expected_output: LearningOutput,
    pub context: LearningContext,
}

/// Learning input
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LearningInput {
    TaskPrediction {
        task_type: TaskType,
        complexity: TaskComplexity,
        historical_performance: Option<HistoricalPerformance>,
    },
    QualityAssessment {
        code_sample: String,
        requirements: Vec<String>,
    },
    ResourceEstimation {
        task_type: TaskType,
        complexity: TaskComplexity,
        constraints: Vec<String>,
    },
}

/// Learning output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LearningOutput {
    TaskPrediction {
        success_probability: f64,
        estimated_quality: f64,
        recommended_strategy: LearningStrategy,
    },
    QualityScore {
        overall_score: f64,
        component_scores: HashMap<String, f64>,
        recommendations: Vec<String>,
    },
    ResourceEstimate {
        cpu_hours: f64,
        memory_mb: u64,
        time_estimate_minutes: u64,
        confidence: f64,
    },
}

/// Learning context
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningContext {
    pub domain: String,
    pub technology_stack: Vec<String>,
    pub time_pressure: bool,
    pub quality_requirements: Vec<String>,
}

/// Learning feedback for algorithm improvement
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningFeedback {
    pub input: LearningInput,
    pub predicted_output: LearningOutput,
    pub actual_outcome: TaskOutcome,
    pub performance_delta: f64,
    pub lessons_learned: Vec<String>,
}

/// Ensemble analytics and component tracking

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnsembleAnalytics {
    component_stats: Vec<EnsembleComponentStatistics>,
    overall_performance: HashMap<String, f64>,
}

impl EnsembleAnalytics {
    pub fn new() -> Self {
        Self {
            component_stats: Vec::new(),
            overall_performance: HashMap::new(),
        }
    }

    /// Add component statistics
    pub fn add_component_stats(&mut self, stats: EnsembleComponentStatistics) {
        self.component_stats.push(stats);
    }

    /// Calculate ensemble diversity score
    pub fn calculate_diversity_score(&self) -> f64 {
        if self.component_stats.len() < 2 {
            return 0.0;
        }

        // Simple diversity calculation based on performance variance
        let accuracies: Vec<f64> = self.component_stats.iter()
            .map(|s| s.accuracy)
            .collect();

        let mean_accuracy = accuracies.iter().sum::<f64>() / accuracies.len() as f64;
        let variance = accuracies.iter()
            .map(|acc| (acc - mean_accuracy).powi(2))
            .sum::<f64>() / accuracies.len() as f64;

        // Higher variance = more diverse (normalized)
        (variance * 100.0).min(1.0)
    }

    /// Calculate ensemble stability score
    pub fn calculate_stability_score(&self) -> f64 {
        // Simplified stability calculation
        // In a real implementation, this would track prediction consistency over time
        if self.component_stats.is_empty() {
            0.0
        } else {
            let avg_f1 = self.component_stats.iter()
                .map(|s| s.f1_score)
                .sum::<f64>() / self.component_stats.len() as f64;
            avg_f1
        }
    }

    /// Get component contributions for ensemble prediction
    pub fn get_component_contributions(&self, prediction_context: &str) -> Vec<ComponentContribution> {
        self.component_stats.iter()
            .enumerate()
            .map(|(i, stats)| {
                let weight = if stats.accuracy > 0.8 {
                    0.3 // High accuracy gets higher weight
                } else if stats.accuracy > 0.6 {
                    0.2 // Medium accuracy
                } else {
                    0.1 // Low accuracy
                };

                ComponentContribution {
                    component_id: stats.component_id.clone(),
                    weight,
                    confidence: stats.accuracy,
                    prediction: serde_json::Value::String(format!("prediction_from_{}", stats.component_id)),
                }
            })
            .collect()
    }

    /// Generate ensemble analytics summary
    pub fn generate_analytics(&self) -> EnsembleAnalytics {
        let mut analytics = EnsembleAnalytics::new();
        analytics.overall_performance.insert("diversity_score".to_string(), self.calculate_diversity_score());
        analytics.overall_performance.insert("stability_score".to_string(), self.calculate_stability_score());
        analytics.overall_performance.insert("component_count".to_string(), self.component_stats.len() as f64);
        analytics
    }
}

/// Problem characteristics analysis

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProblemCharacteristicsAnalyzer {
    characteristics_cache: HashMap<String, ProblemCharacteristics>,
}

impl ProblemCharacteristicsAnalyzer {
    pub fn new() -> Self {
        Self {
            characteristics_cache: HashMap::new(),
        }
    }

    /// Analyze problem characteristics
    pub fn analyze(&mut self, problem_id: &str, data: &LearningDataPoint) -> ProblemCharacteristics {
        if let Some(cached) = self.characteristics_cache.get(problem_id) {
            return cached.clone();
        }

        let characteristics = self.compute_characteristics(data);
        self.characteristics_cache.insert(problem_id.to_string(), characteristics.clone());
        characteristics
    }

    fn compute_characteristics(&self, data: &LearningDataPoint) -> ProblemCharacteristics {
        // Simplified characteristic computation
        // In a real implementation, this would analyze the actual data

        let feature_count = match &data.input {
            LearningInput::TaskPrediction { .. } => 10,
            LearningInput::QualityAssessment { code_sample, .. } => code_sample.len() / 100,
            LearningInput::ResourceEstimation { .. } => 8,
        };

        let sample_count = 1; // Single data point
        let has_missing_values = false; // Assume complete data
        let is_regression = matches!(data.expected_output, LearningOutput::TaskPrediction { .. });

        let estimated_complexity = match &data.input {
            LearningInput::TaskPrediction { complexity, .. } => match complexity {
                TaskComplexity::Simple => 0.2,
                TaskComplexity::Moderate => 0.5,
                TaskComplexity::Complex => 0.8,
                TaskComplexity::Critical => 0.95,
            },
            _ => 0.5,
        };

        ProblemCharacteristics {
            feature_count,
            sample_count,
            class_count: if is_regression { None } else { Some(2) }, // Binary classification
            has_missing_values,
            is_regression,
            estimated_complexity,
        }
    }
}

/// Algorithm performance tracker

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AlgorithmPerformanceTracker {
    performance_history: HashMap<LearningAlgorithmType, Vec<AlgorithmPerformance>>,
}

impl AlgorithmPerformanceTracker {
    pub fn new() -> Self {
        Self {
            performance_history: HashMap::new(),
        }
    }

    /// Record algorithm performance
    pub fn record_performance(&mut self, performance: AlgorithmPerformance) {
        self.performance_history
            .entry(performance.algorithm_type)
            .or_insert_with(Vec::new)
            .push(performance);
    }

    /// Get recent performance for algorithm type
    pub fn get_recent_performance(&self, algorithm_type: &LearningAlgorithmType, count: usize) -> Vec<&AlgorithmPerformance> {
        if let Some(history) = self.performance_history.get(algorithm_type) {
            history.iter().rev().take(count).collect()
        } else {
            Vec::new()
        }
    }

    /// Get average performance for algorithm type
    pub fn get_average_performance(&self, algorithm_type: &LearningAlgorithmType) -> Option<AlgorithmPerformance> {
        let performances = self.performance_history.get(algorithm_type)?;
        if performances.is_empty() {
            return None;
        }

        let avg_accuracy = performances.iter().map(|p| p.accuracy).sum::<f64>() / performances.len() as f64;
        let avg_training_time = performances.iter().map(|p| p.training_time_ms).sum::<u64>() / performances.len() as u64;
        let avg_prediction_time = performances.iter().map(|p| p.prediction_time_ms).sum::<u64>() / performances.len() as u64;

        Some(AlgorithmPerformance {
            algorithm_type: algorithm_type.clone(),
            accuracy: avg_accuracy,
            training_time_ms: avg_training_time,
            prediction_time_ms: avg_prediction_time,
            memory_usage_mb: 0.0, // Would need to track this
            convergence_iterations: performances.iter().map(|p| p.convergence_iterations).max().unwrap_or(0),
            measured_at: chrono::Utc::now(),
        })
    }
}
