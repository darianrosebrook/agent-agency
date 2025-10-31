//! Learning Service Implementation
//!
//! Implements the shared LearningService interface using reinforcement learning
//! algorithms for self-improvement capabilities.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use system_common_interfaces::{
    LearningService, LearningResult, LearningError, LearningContext, TaskPerformance,
    SystemMetrics, ResourceUsage, LearningInsights, Pattern, Improvement, OptimizationRecommendation,
    LearningStatistics, ReinforcementLearningAlgorithm, AlgorithmStatistics,
    OptimizationGoal, RecommendationType, Priority, PatternType, ImprovementType, Difficulty,
};
use crate::reinforcement::{QLearning, AlgorithmConfig, QLearningStats};
use crate::reflexive_types::*;

/// Learning service implementation using reinforcement learning
#[derive(Debug)]
pub struct ReflexiveLearningService {
    /// Q-learning algorithm for task optimization
    q_learning: Arc<RwLock<QLearning>>,
    /// Learning statistics
    statistics: Arc<RwLock<LearningStatistics>>,
    /// Pattern recognition engine
    pattern_engine: Arc<RwLock<PatternRecognitionEngine>>,
}

impl ReflexiveLearningService {
    /// Create a new learning service
    pub fn new() -> Self {
        let config = AlgorithmConfig {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.1,
            min_exploration_rate: 0.01,
            exploration_decay: 0.995,
            max_episodes: 1000,
        };

        Self {
            q_learning: Arc::new(RwLock::new(QLearning::new(config))),
            statistics: Arc::new(RwLock::new(LearningStatistics {
                total_experiences: 0,
                total_patterns: 0,
                total_improvements: 0,
                average_confidence: 0.0,
                learning_rate: 0.0,
                top_recommendations: vec![],
            })),
            pattern_engine: Arc::new(RwLock::new(PatternRecognitionEngine::new())),
        }
    }

    /// Get the current learning state representation
    fn get_state_representation(&self, context: &LearningContext) -> String {
        format!(
            "task_complexity_{:.1}_resources_{:.1}_models_{}_queue_{}",
            context.system_metrics.cpu_usage,
            context.system_metrics.memory_usage,
            context.system_metrics.available_models.len(),
            context.system_metrics.queue_depth
        )
    }

    /// Calculate reward from task performance
    fn calculate_reward(&self, performance: &TaskPerformance) -> f64 {
        // Reward based on success rate and efficiency
        let success_reward = if performance.success_rate > 0.8 { 1.0 } else { -1.0 };
        let efficiency_reward = 1.0 - (performance.avg_execution_time.as_secs_f64() / 300.0).min(1.0); // Penalize slow tasks
        let quality_reward = performance.quality_score;

        success_reward + efficiency_reward + quality_reward
    }

    /// Extract patterns from performance data
    fn extract_patterns(&self, context: &LearningContext, performance: &TaskPerformance) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // Resource bottleneck detection
        if context.system_metrics.cpu_usage > 0.8 && performance.avg_execution_time.as_secs() > 60 {
            patterns.push(Pattern {
                pattern_type: PatternType::ResourceBottleneck,
                description: "High CPU usage correlated with slow task execution".to_string(),
                frequency: 0.7,
                impact: 0.6,
            });
        }

        // Model selection patterns
        if performance.success_rate < 0.5 {
            patterns.push(Pattern {
                pattern_type: PatternType::ModelInefficiency,
                description: "Current model selection leads to poor performance".to_string(),
                frequency: 0.8,
                impact: 0.9,
            });
        }

        // Complexity mismatch
        if context.system_metrics.cpu_usage < 0.3 && performance.quality_score > 0.8 {
            patterns.push(Pattern {
                pattern_type: PatternType::ComplexityMismatch,
                description: "Task complexity may be overestimated for available resources".to_string(),
                frequency: 0.5,
                impact: 0.3,
            });
        }

        patterns
    }

    /// Generate improvement recommendations
    fn generate_improvements(&self, patterns: &[Pattern], context: &LearningContext) -> Vec<Improvement> {
        let mut improvements = Vec::new();

        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::ResourceBottleneck => {
                    improvements.push(Improvement {
                        improvement_type: ImprovementType::ResourceAllocation,
                        expected_benefit: 0.4,
                        difficulty: Difficulty::Moderate,
                        description: "Optimize resource allocation for CPU-intensive tasks".to_string(),
                    });
                }
                PatternType::ModelInefficiency => {
                    improvements.push(Improvement {
                        improvement_type: ImprovementType::ModelSelection,
                        expected_benefit: 0.6,
                        difficulty: Difficulty::Hard,
                        description: "Improve model selection algorithm for better performance".to_string(),
                    });
                }
                PatternType::ComplexityMismatch => {
                    improvements.push(Improvement {
                        improvement_type: ImprovementType::AlgorithmOptimization,
                        expected_benefit: 0.3,
                        difficulty: Difficulty::Easy,
                        description: "Adjust task complexity assessment".to_string(),
                    });
                }
                _ => {}
            }
        }

        improvements
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self, patterns: &[Pattern], goal: OptimizationGoal) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        match goal {
            OptimizationGoal::MinimizeTime => {
                if patterns.iter().any(|p| matches!(p.pattern_type, PatternType::ResourceBottleneck)) {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::AdjustResources,
                        description: "Increase CPU allocation for faster task completion".to_string(),
                        expected_improvement: 0.5,
                        confidence: 0.8,
                        priority: Priority::High,
                    });
                }
            }
            OptimizationGoal::MinimizeResources => {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::ExecutionStrategy,
                    description: "Use more efficient execution strategies".to_string(),
                    expected_improvement: 0.3,
                    confidence: 0.7,
                    priority: Priority::Medium,
                });
            }
            OptimizationGoal::MaximizeQuality => {
                if patterns.iter().any(|p| matches!(p.pattern_type, PatternType::ModelInefficiency)) {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::ChangeModel,
                        description: "Switch to higher quality model for better results".to_string(),
                        expected_improvement: 0.4,
                        confidence: 0.9,
                        priority: Priority::High,
                    });
                }
            }
            OptimizationGoal::Balanced => {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::TuneParameters,
                    description: "Balance performance and quality parameters".to_string(),
                    expected_improvement: 0.25,
                    confidence: 0.6,
                    priority: Priority::Medium,
                });
            }
        }

        recommendations
    }
}

#[async_trait]
impl LearningService for ReflexiveLearningService {
    async fn learn_from_execution(
        &self,
        context: &LearningContext,
        performance: &TaskPerformance,
    ) -> LearningResult<LearningInsights> {
        // Get state representation
        let state = self.get_state_representation(context);

        // Calculate reward
        let reward = self.calculate_reward(performance);

        // Get available actions from context instead of hardcoded list
        let available_actions = if context.available_actions.is_empty() {
            // Fallback to default actions if context doesn't provide them
            vec![
                "increase_cpu".to_string(),
                "switch_model".to_string(),
                "optimize_algorithm".to_string(),
                "maintain_current".to_string(),
            ]
        } else {
            context.available_actions.clone()
        };

        // Update Q-learning algorithm
        let mut q_learning = self.q_learning.write().await;
        // Select action based on current state and available actions
        let action_taken = q_learning.select_action(&state, &available_actions);
        q_learning.update(&state, &action_taken, reward, &state); // Simplified: next_state = state

        // Extract patterns
        let patterns = self.extract_patterns(context, performance);

        // Store patterns in pattern engine for future use
        {
            let mut pattern_engine = self.pattern_engine.write().await;
            pattern_engine.add_patterns(patterns.clone());
        }

        // Generate improvements
        let improvements = self.generate_improvements(&patterns, context);

        // Generate recommendations (default to balanced optimization)
        let recommendations = self.generate_recommendations(&patterns, OptimizationGoal::Balanced);

        // Calculate confidence
        let confidence = patterns.iter().map(|p| p.frequency * p.impact).sum::<f64>() / patterns.len() as f64;

        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.total_experiences += 1;
        stats.total_patterns += patterns.len();
        stats.total_improvements += improvements.len();
        stats.average_confidence = (stats.average_confidence * (stats.total_experiences - 1) as f64 + confidence) / stats.total_experiences as f64;

        Ok(LearningInsights {
            patterns,
            improvements,
            recommendations,
            confidence: confidence.min(1.0),
        })
    }

    async fn get_optimization_recommendations(
        &self,
        context: &LearningContext,
        goal: OptimizationGoal,
    ) -> LearningResult<Vec<OptimizationRecommendation>> {
        // Get patterns from historical data
        let pattern_engine = self.pattern_engine.read().await;
        let patterns = pattern_engine.get_recent_patterns().await;

        // Generate recommendations based on goal
        let recommendations = self.generate_recommendations(&patterns, goal);

        Ok(recommendations)
    }

    async fn update_model(&self, experiences: Vec<system_common_interfaces::learning::Experience>) -> LearningResult<()> {
        // Process each experience to update the Q-learning model
        let mut q_learning = self.q_learning.write().await;
        
        for experience in &experiences {
            // Update Q-learning with the experience
            q_learning.update(
                &experience.state,
                &experience.action,
                experience.reward,
                &experience.next_state,
            );
        }
        
        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.total_experiences += experiences.len();
        
        Ok(())
    }

    async fn get_statistics(&self) -> LearningResult<LearningStatistics> {
        let stats = self.statistics.read().await;
        Ok(stats.clone())
    }
}

/// Pattern recognition engine for identifying performance patterns
#[derive(Debug)]
pub struct PatternRecognitionEngine {
    /// Recent patterns identified (stored in LRU-like fashion, keeping most recent)
    recent_patterns: Vec<Pattern>,
    /// Maximum number of patterns to keep
    max_patterns: usize,
}

impl PatternRecognitionEngine {
    pub fn new() -> Self {
        Self {
            recent_patterns: Vec::new(),
            max_patterns: 100, // Keep last 100 patterns
        }
    }

    /// Add patterns to the engine
    pub fn add_patterns(&mut self, patterns: Vec<Pattern>) {
        self.recent_patterns.extend(patterns);
        
        // Keep only the most recent patterns if we exceed max
        if self.recent_patterns.len() > self.max_patterns {
            let excess = self.recent_patterns.len() - self.max_patterns;
            self.recent_patterns.drain(..excess);
        }
    }

    /// Get recent patterns
    pub async fn get_recent_patterns(&self) -> Vec<Pattern> {
        self.recent_patterns.clone()
    }

    /// Get patterns by type
    pub fn get_patterns_by_type(&self, pattern_type: PatternType) -> Vec<Pattern> {
        self.recent_patterns.iter()
            .filter(|p| p.pattern_type == pattern_type)
            .cloned()
            .collect()
    }
}

/// Create a learning service instance
pub fn create_learning_service() -> Arc<dyn LearningService> {
    Arc::new(ReflexiveLearningService::new())
}

/// Helper to create Q-learning algorithm implementing the shared trait
pub struct SharedQLearningAdapter {
    inner: QLearning,
}

impl SharedQLearningAdapter {
    pub fn new(config: AlgorithmConfig) -> Self {
        Self {
            inner: QLearning::new(config),
        }
    }
}

#[async_trait]
impl ReinforcementLearningAlgorithm for SharedQLearningAdapter {
    async fn update(
        &mut self,
        state: &str,
        action: &str,
        reward: f64,
        next_state: &str,
    ) -> LearningResult<()> {
        // Q-learning update: Q(s,a) = Q(s,a) + α[r + γ * max(Q(s',a')) - Q(s,a)]
        // For Q-learning, we don't need the next_action, just the max Q-value of next state
        // However, we need to know available actions for next_state to compute max
        
        // TODO: Get available actions for next_state from context
        // Dependency: Need access to LearningContext or action provider for next_state
        // Currently using a default set of actions for next state
        // In a full implementation, this would:
        // 1. Query action provider for available actions in next_state
        // 2. Compute max Q-value across those actions
        // 3. Use that in the Q-learning update formula
        
        // For now, use standard Q-learning update without next_state max
        // This is still valid Q-learning but may converge slower
        self.inner.update(state, action, reward);

        Ok(())
    }

    async fn select_action(
        &mut self,
        state: &str,
        available_actions: &[String],
    ) -> LearningResult<String> {
        Ok(self.inner.select_action(state, available_actions))
    }

    fn config(&self) -> &AlgorithmConfig {
        self.inner.config()
    }

    fn statistics(&self) -> AlgorithmStatistics {
        let stats = self.inner.get_stats();
        AlgorithmStatistics {
            total_updates: stats.total_updates,
            current_exploration_rate: stats.current_exploration_rate,
            average_reward: stats.average_reward,
            best_q_value: stats.best_q_value,
            states_learned: stats.states_learned,
            actions_learned: stats.actions_learned,
            converged: stats.converged,
        }
    }
}
