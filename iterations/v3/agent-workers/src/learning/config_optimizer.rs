//! Configuration optimizer for learning optimal settings

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;

use crate::learning::types::*;
use crate::learning::PatternAnalyzer;
use crate::worker_types::{ExecutionOutcome, LearningMode};

/// Optimizes configuration parameters based on execution history
pub struct ConfigurationOptimizer {
    optimization_history: Arc<tokio::sync::RwLock<Vec<OptimizationEvent>>>,
    optimal_configs: Arc<tokio::sync::RwLock<Vec<OptimalConfig>>>,
    pattern_analyzer: Arc<PatternAnalyzer>,
}

impl ConfigurationOptimizer {
    pub fn new(pattern_analyzer: Arc<PatternAnalyzer>) -> Self {
        Self {
            optimization_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            optimal_configs: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            pattern_analyzer,
        }
    }

    /// Generate configuration recommendations based on task characteristics
    pub async fn generate_recommendations(
        &self,
        task_characteristics: &HashMap<String, serde_json::Value>,
    ) -> Result<ConfigurationRecommendations, Box<dyn std::error::Error + Send + Sync>> {
        let mut recommendations = ConfigurationRecommendations {
            worker_selection: None,
            task_decomposition: None,
            resource_allocation: None,
            quality_thresholds: None,
            confidence: 0.0,
            reasoning: String::new(),
            recommended_config: None,
            confidence_score: 0.0,
            expected_improvement: None,
            optimization_reason: None,
        };

        // Get optimal configurations
        let optimal_configs = self.optimal_configs.read().await;
        
        // Find best matching configuration
        let mut best_config = None;
        let mut best_match_score = 0.0;

        for config in optimal_configs.iter() {
            let match_score = self.calculate_config_match_score(task_characteristics, &config.conditions);
            if match_score > best_match_score {
                best_match_score = match_score;
                best_config = Some(config.clone());
            }
        }

        if let Some(config) = best_config {
            recommendations.confidence = config.confidence;
            recommendations.reasoning = format!("Based on {} similar executions", config.conditions.get("min_executions").unwrap_or(&serde_json::Value::Number(0.into())));

            // Generate specific recommendations based on config type
            match config.config_type {
                ConfigType::WorkerSelection => {
                    recommendations.worker_selection = Some(WorkerSelectionRecommendation {
                        preferred_workers: vec![], // Would be populated from config parameters
                        worker_weights: HashMap::new(),
                        reasoning: "Optimized worker selection based on historical performance".to_string(),
                    });
                }
                ConfigType::TaskDecomposition => {
                    recommendations.task_decomposition = Some(TaskDecompositionRecommendation {
                        suggested_subtasks: 3, // Default value
                        decomposition_strategy: "Parallel decomposition".to_string(),
                        reasoning: "Optimal decomposition strategy based on task complexity".to_string(),
                    });
                }
                ConfigType::ResourceAllocation => {
                    recommendations.resource_allocation = Some(ResourceAllocationRecommendation {
                        cpu_allocation: 0.8,
                        memory_allocation: 0.6,
                        timeout_ms: 300000, // 5 minutes
                        reasoning: "Resource allocation optimized for similar tasks".to_string(),
                    });
                }
                ConfigType::QualityThresholds => {
                    recommendations.quality_thresholds = Some(QualityThresholdRecommendation {
                        min_quality_score: config.performance_metrics.quality_score,
                        max_rework_rate: 0.1,
                        reasoning: "Quality thresholds based on historical success patterns".to_string(),
                    });
                }
                _ => {
                    // Other config types not yet implemented
                }
            }
        } else {
            // No matching configuration found, provide default recommendations
            recommendations.confidence = 0.5;
            recommendations.reasoning = "No historical data available, using default recommendations".to_string();
            
            recommendations.worker_selection = Some(WorkerSelectionRecommendation {
                preferred_workers: vec![],
                worker_weights: HashMap::new(),
                reasoning: "Default worker selection strategy".to_string(),
            });
        }

        Ok(recommendations)
    }

    /// Apply a configuration and track its performance
    pub async fn apply_configuration(
        &self,
        config: OptimalConfig,
        performance_metrics: PerformanceMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = OptimizationEvent {
            id: Uuid::new_v4(),
            event_type: OptimizationEventType::ConfigApplied,
            config_id: config.id,
            performance_delta: performance_metrics,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            config_before: None,
            config_after: None,
            performance_improvement: None,
            optimization_type: None,
        };

        let mut history = self.optimization_history.write().await;
        history.push(event);

        Ok(())
    }

    /// Get optimization history
    pub async fn get_optimization_history(&self) -> Vec<OptimizationEvent> {
        let history = self.optimization_history.read().await;
        history.clone()
    }

    /// Calculate configuration match score
    fn calculate_config_match_score(
        &self,
        task_characteristics: &HashMap<String, serde_json::Value>,
        config_conditions: &HashMap<String, serde_json::Value>,
    ) -> f64 {
        let mut matches = 0;
        let mut total_conditions = config_conditions.len();

        for (key, config_value) in config_conditions {
            if let Some(task_value) = task_characteristics.get(key) {
                if task_value == config_value {
                    matches += 1;
                }
            }
        }

        if total_conditions == 0 {
            0.0
        } else {
            matches as f64 / total_conditions as f64
        }
    }

    /// Learn from execution results
    pub async fn learn_from_execution(
        &self,
        config_id: Uuid,
        success: bool,
        performance_metrics: PerformanceMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_type = if success {
            OptimizationEventType::PerformanceImproved
        } else {
            OptimizationEventType::PerformanceDegraded
        };

        let event = OptimizationEvent {
            id: Uuid::new_v4(),
            event_type,
            config_id,
            performance_delta: performance_metrics,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            config_before: None,
            config_after: None,
            performance_improvement: None,
            optimization_type: None,
        };

        let mut history = self.optimization_history.write().await;
        history.push(event);

        Ok(())
    }

    /// Get optimal configurations
    pub async fn get_optimal_configs(&self) -> Vec<OptimalConfig> {
        let configs = self.optimal_configs.read().await;
        configs.clone()
    }

    /// Add optimal configuration
    pub async fn add_optimal_config(&self, config: OptimalConfig) -> Result<()> {
        let mut configs = self.optimal_configs.write().await;
        configs.push(config);
        Ok(())
    }
}
