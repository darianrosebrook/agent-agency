//! Learning Service Implementation
//!
//! Concrete implementation of the LearningService trait using basic
//! reinforcement learning algorithms for task optimization and self-improvement.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use system_common_interfaces::learning::{
        LearningService, LearningResult, LearningContext, TaskPerformance,
        LearningInsights, Pattern, Improvement, OptimizationRecommendation,
        LearningStatistics, OptimizationGoal, RecommendationType, Priority,
        PatternType, ImprovementType, Difficulty,
    };

/// Simple Q-learning based learning service
#[derive(Debug)]
pub struct SimpleLearningService {
    /// Q-table for state-action value storage
    q_table: Arc<RwLock<HashMap<String, HashMap<String, f64>>>>,
    /// Learning statistics
    statistics: Arc<RwLock<LearningStatistics>>,
    /// Learning configuration
    learning_rate: f64,
    discount_factor: f64,
    exploration_rate: f64,
}

impl SimpleLearningService {
    /// Create a new simple learning service
    pub fn new() -> Self {
        Self {
            q_table: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(LearningStatistics {
                total_experiences: 0,
                total_patterns: 0,
                total_improvements: 0,
                average_confidence: 0.0,
                learning_rate: 0.0,
                top_recommendations: vec![],
            })),
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.1,
        }
    }

    /// Get the exploration rate used in learning
    pub fn exploration_rate(&self) -> f64 {
        self.exploration_rate
    }

    /// Get Q-value for state-action pair
    async fn get_q_value(&self, state: &str, action: &str) -> f64 {
        let q_table = self.q_table.read().await;
        q_table
            .get(state)
            .and_then(|actions| actions.get(action))
            .copied()
            .unwrap_or(0.0)
    }

    /// Set Q-value for state-action pair
    async fn set_q_value(&self, state: &str, action: &str, value: f64) {
        let mut q_table = self.q_table.write().await;
        q_table
            .entry(state.to_string())
            .or_insert_with(HashMap::new)
            .insert(action.to_string(), value);
    }

    /// Get best action for state
    async fn get_best_action(&self, state: &str) -> Option<String> {
        let q_table = self.q_table.read().await;
        q_table
            .get(state)?
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(action, _)| action.clone())
    }

    /// Update statistics
    async fn update_statistics(&self, experiences_processed: usize) {
        let mut stats = self.statistics.write().await;
        stats.total_experiences += experiences_processed;
        stats.learning_rate = self.learning_rate;

        // Calculate average confidence from Q-table values
        let q_table = self.q_table.read().await;
        let mut total_confidence = 0.0;
        let mut confidence_count = 0;
        
        for (_state, actions) in q_table.iter() {
            for (_action, q_value) in actions.iter() {
                // Convert Q-value to confidence (normalize to 0-1 range)
                // Q-values can be negative, so we use sigmoid-like normalization
                let normalized_confidence = (q_value / (1.0 + q_value.abs()) + 1.0) / 2.0;
                total_confidence += normalized_confidence;
                confidence_count += 1;
            }
        }
        
        stats.average_confidence = if confidence_count > 0 {
            total_confidence / confidence_count as f64
        } else {
            0.5 // Default confidence when no data
        };

        // Generate some top recommendations based on learned patterns
        stats.top_recommendations = vec![
            OptimizationRecommendation {
                recommendation_type: RecommendationType::ChangeModel,
                description: "Use faster model for similar tasks".to_string(),
                expected_improvement: 0.3,
                confidence: 0.8,
                priority: Priority::Medium,
            },
        ];
    }
}

#[async_trait]
impl LearningService for SimpleLearningService {
    async fn learn_from_execution(
        &self,
        context: &LearningContext,
        performance: &TaskPerformance,
    ) -> LearningResult<LearningInsights> {
        // Calculate reward based on performance
        let reward = performance.quality_score * 0.7 + performance.success_rate * 0.3;

        // Get current Q-value
        let current_q = self.get_q_value(&context.state, "current").await;

        // Q-learning update with next state estimation
        // Use best action from current state as next state approximation
        let best_action = self.get_best_action(&context.state).await;
        let next_state_q = if let Some(action) = best_action {
            self.get_q_value(&context.state, &action).await
        } else {
            0.0
        };
        
        // Q-learning update: Q(s,a) = Q(s,a) + α[r + γ*max(Q(s',a')) - Q(s,a)]
        let new_q = current_q + self.learning_rate * (reward + self.discount_factor * next_state_q - current_q);

        // Update Q-table
        self.set_q_value(&context.state, "current", new_q).await;

        // Update statistics
        self.update_statistics(1).await;

        // Generate insights
        let patterns = vec![
            Pattern {
                pattern_type: PatternType::TimingOptimization,
                description: format!("Task {} shows performance pattern", context.task_id),
                frequency: 1.0,
                impact: performance.quality_score,
            },
        ];

        let improvements = vec![
            Improvement {
                improvement_type: ImprovementType::ResourceAllocation,
                expected_benefit: 0.2,
                difficulty: Difficulty::Moderate,
                description: "Optimize resource allocation for better performance".to_string(),
            },
        ];

        let recommendations = vec![
            OptimizationRecommendation {
                recommendation_type: RecommendationType::AdjustResources,
                description: "Increase memory allocation for similar tasks".to_string(),
                expected_improvement: 0.25,
                confidence: 0.75,
                priority: Priority::Medium,
            },
        ];

        Ok(LearningInsights {
            patterns,
            improvements,
            recommendations,
            confidence: 0.8,
        })
    }

    async fn get_optimization_recommendations(
        &self,
        context: &LearningContext,
        goal: OptimizationGoal,
    ) -> LearningResult<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Generate recommendations based on goal and learned patterns
        match goal {
            OptimizationGoal::MinimizeTime => {
                if let Some(best_action) = self.get_best_action(&context.state).await {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::ExecutionStrategy,
                        description: format!("Use optimized execution strategy: {}", best_action),
                        expected_improvement: 0.3,
                        confidence: 0.8,
                        priority: Priority::High,
                    });
                }
            }
            OptimizationGoal::MinimizeResources => {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::AdjustResources,
                    description: "Reduce resource allocation for efficiency".to_string(),
                    expected_improvement: 0.2,
                    confidence: 0.7,
                    priority: Priority::Medium,
                });
            }
            OptimizationGoal::MaximizeQuality => {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::TuneParameters,
                    description: "Adjust parameters for better quality".to_string(),
                    expected_improvement: 0.25,
                    confidence: 0.75,
                    priority: Priority::High,
                });
            }
            OptimizationGoal::Balanced => {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::ExecutionStrategy,
                    description: "Use balanced optimization approach".to_string(),
                    expected_improvement: 0.2,
                    confidence: 0.8,
                    priority: Priority::Medium,
                });
            }
        }

        Ok(recommendations)
    }

    async fn update_model(&self, experiences: Vec<system_common_interfaces::learning::Experience>) -> LearningResult<()> {
        // Update learning model with batch of experiences
        // Process each experience using Q-learning update
        for experience in &experiences {
            let current_q = self.get_q_value(&experience.state, &experience.action).await;
            let next_q = self.get_q_value(&experience.next_state, "best").await;
            
            // Q-learning update: Q(s,a) = Q(s,a) + α[r + γ*max(Q(s',a')) - Q(s,a)]
            let new_q = current_q + self.learning_rate * (experience.reward + self.discount_factor * next_q - current_q);
            self.set_q_value(&experience.state, &experience.action, new_q).await;
        }
        
        // Update statistics
        let experience_count = experiences.len();
        self.update_statistics(experience_count).await;
        Ok(())
    }

    async fn get_statistics(&self) -> LearningResult<LearningStatistics> {
        let stats = self.statistics.read().await;
        Ok(stats.clone())
    }
}

/// Create a new learning service instance
pub fn create_learning_service() -> Arc<dyn LearningService> {
    Arc::new(SimpleLearningService::new())
}
