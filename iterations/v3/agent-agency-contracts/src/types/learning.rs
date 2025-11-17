//! Learning algorithm types and configurations
//!
//! This module contains types for reinforcement learning, optimization,
//! and other learning algorithms used throughout the agent system.
//!
//! @author @darianrosebrook

#[cfg(feature = "serde")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Types of learning algorithms supported
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LearningAlgorithmType {
    ReinforcementLearning,
    SupervisedLearning,
    UnsupervisedLearning,
    TransferLearning,
    DeepReinforcementLearning,
    EnsembleLearning,
    MetaLearning,
    OnlineLearning,
}

/// Configuration for learning algorithms
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct AlgorithmConfig {
    /// Learning rate (alpha)
    pub learning_rate: f64,
    /// Discount factor (gamma)
    pub discount_factor: f64,
    /// Initial exploration rate (epsilon)
    pub exploration_rate: f64,
    /// Minimum exploration rate
    pub min_exploration_rate: f64,
    /// Exploration decay rate
    pub exploration_decay: f64,
    /// Maximum training iterations
    pub max_iterations: usize,
    /// Maximum training episodes
    pub max_episodes: usize,
    /// Convergence threshold
    pub convergence_threshold: f64,
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.1,
            min_exploration_rate: 0.01,
            exploration_decay: 0.995,
            max_iterations: 1000,
            max_episodes: 1000,
            convergence_threshold: 0.001,
        }
    }
}

/// Errors that can occur during learning operations
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearningError {
    Algorithm(String),
    TrainingData(String),
    Model(String),
    Optimization(String),
    Configuration(String),
}

impl std::fmt::Display for LearningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LearningError::Algorithm(msg) => write!(f, "Algorithm error: {}", msg),
            LearningError::TrainingData(msg) => write!(f, "Training data error: {}", msg),
            LearningError::Model(msg) => write!(f, "Model error: {}", msg),
            LearningError::Optimization(msg) => write!(f, "Optimization error: {}", msg),
            LearningError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learning_error_display_all_variants() {
        // Mutation test: Display should return actual formatted strings, not empty
        let algorithm = LearningError::Algorithm("test".to_string()).to_string();
        let training_data = LearningError::TrainingData("test".to_string()).to_string();
        let model = LearningError::Model("test".to_string()).to_string();
        let optimization = LearningError::Optimization("test".to_string()).to_string();
        let configuration = LearningError::Configuration("test".to_string()).to_string();
        
        assert_eq!(algorithm, "Algorithm error: test");
        assert_eq!(training_data, "Training data error: test");
        assert_eq!(model, "Model error: test");
        assert_eq!(optimization, "Optimization error: test");
        assert_eq!(configuration, "Configuration error: test");
        
        // Ensure Display doesn't return empty (would happen if fmt returns Ok(Default::default()))
        assert!(!algorithm.is_empty(), "Display should not return empty string");
        assert!(!training_data.is_empty(), "Display should not return empty string");
        assert!(!model.is_empty(), "Display should not return empty string");
        assert!(!optimization.is_empty(), "Display should not return empty string");
        assert!(!configuration.is_empty(), "Display should not return empty string");
    }

    #[test]
    fn learning_error_display_with_empty_message() {
        assert_eq!(
            LearningError::Algorithm("".to_string()).to_string(),
            "Algorithm error: "
        );
    }
}

/// Result type for learning operations
pub type LearningResult<T> = Result<T, LearningError>;
