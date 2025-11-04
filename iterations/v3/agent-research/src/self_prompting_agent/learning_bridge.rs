//! Learning bridge for connecting to reflexive learning systems
//!
//! Bridges the gap between self-prompting agent and external learning algorithms.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use crate::self_prompting_agent::prompting_types::SelfPromptingAgentError;
use crate::learning_service::create_learning_service;
// PLACEHOLDER: system_common_interfaces crate doesn't exist
// Using a stub trait implementation instead
// TODO: Replace with actual system_common_interfaces when available
#[async_trait::async_trait]
pub trait LearningService: Send + Sync {
    async fn learn_from_execution(&self, context: &LearningContext, performance: &TaskPerformance) -> Result<LearningInsights, String>;
    async fn get_optimization_recommendations(&self, context: &LearningContext, goal: OptimizationGoal) -> Result<Vec<OptimizationRecommendation>, String>;
    async fn update_model(&self, experiences: Vec<Experience>) -> Result<(), String>;
    async fn get_statistics(&self) -> Result<LearningStatistics, String>;
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningContext {
    pub task_id: String,
    pub state: String,
    pub system_metrics: SystemMetrics,
    pub available_actions: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskPerformance {
    pub success_rate: f64,
    pub avg_execution_time: std::time::Duration,
    pub quality_score: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub available_models: Vec<String>,
    pub active_tasks: usize,
    pub queue_depth: usize,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningInsights {
    pub patterns: Vec<Pattern>,
    pub improvements: Vec<Improvement>,
    pub recommendations: Vec<OptimizationRecommendation>,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub frequency: f64,
    pub impact: f64,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum PatternType {
    ResourceBottleneck,
    ModelInefficiency,
    ComplexityMismatch,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Improvement {
    pub improvement_type: ImprovementType,
    pub expected_benefit: f64,
    pub difficulty: Difficulty,
    pub description: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ImprovementType {
    ResourceAllocation,
    ModelSelection,
    AlgorithmOptimization,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Difficulty {
    Easy,
    Moderate,
    Hard,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OptimizationRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub expected_improvement: f64,
    pub confidence: f64,
    pub priority: Priority,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RecommendationType {
    AdjustResources,
    ExecutionStrategy,
    ChangeModel,
    TuneParameters,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Priority {
    Low,
    Medium,
    High,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum OptimizationGoal {
    MinimizeTime,
    MinimizeResources,
    MaximizeQuality,
    Balanced,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Experience {
    pub state: String,
    pub action: String,
    pub reward: f64,
    pub next_state: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningStatistics {
    pub total_experiences: usize,
    pub total_patterns: usize,
    pub total_improvements: usize,
    pub average_confidence: f64,
    pub learning_rate: f64,
    pub top_recommendations: Vec<String>,
}

/// Learning bridge coordinator
pub struct LearningBridge {
    /// Learning service instance
    learning_service: Arc<dyn LearningService>,
}

impl LearningBridge {
    /// Create a new learning bridge
    pub fn new() -> Self {
        Self {
            learning_service: create_learning_service(),
        }
    }

    /// Process a learning signal
    pub async fn process_signal(&self, signal: LearningSignal) -> Result<(), SelfPromptingAgentError> {
        tracing::info!("Processing learning signal: {:?}", signal.signal_type);
        
        // Convert LearningSignal to LearningContext and TaskPerformance
        let context = LearningContext {
            task_id: signal.context.clone(),
            state: format!("signal_{}_{}", signal.signal_type, signal.value),
            system_metrics: SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                available_models: vec![],
                active_tasks: 0,
                queue_depth: 0,
            },
            available_actions: vec![],
        };

        let performance = TaskPerformance {
            success_rate: signal.value.max(0.0).min(1.0),
            avg_execution_time: std::time::Duration::from_secs(0),
            quality_score: signal.value,
        };

        // Forward to learning service
        match self.learning_service.learn_from_execution(&context, &performance).await {
            Ok(insights) => {
                tracing::debug!("Learning insights generated: {} patterns, {} recommendations", 
                    insights.patterns.len(), insights.recommendations.len());
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Failed to process learning signal: {}", e);
                Err(SelfPromptingAgentError::Learning(format!("Learning service error: {}", e)))
            }
        }
    }

    /// Get learning recommendations
    pub async fn get_recommendations(&self, context: &str) -> Result<Vec<String>, SelfPromptingAgentError> {
        let learning_context = LearningContext {
            task_id: context.to_string(),
            state: context.to_string(),
            system_metrics: SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                available_models: vec![],
                active_tasks: 0,
                queue_depth: 0,
            },
            available_actions: vec![],
        };

        // Query learning system for optimization recommendations
        match self.learning_service
            .get_optimization_recommendations(&learning_context, OptimizationGoal::Balanced)
            .await
        {
            Ok(recommendations) => {
                Ok(recommendations
                    .iter()
                    .map(|r| r.description.clone())
                    .collect())
            }
            Err(e) => {
                tracing::warn!("Failed to get learning recommendations: {}", e);
                // Fallback to default recommendations
                Ok(vec![
                    "Consider using more specific prompts".to_string(),
                    "Try breaking complex tasks into smaller steps".to_string(),
                ])
            }
        }
    }
}

/// Learning signal for RL feedback

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningSignal {
    pub signal_type: String,
    pub value: f64,
    pub context: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Reflexive learning system integration
pub struct ReflexiveLearningSystem {
    /// Learning service instance
    learning_service: Arc<dyn LearningService>,
}

impl ReflexiveLearningSystem {
    /// Create a new reflexive learning system
    pub fn new() -> Self {
        Self {
            learning_service: create_learning_service(),
        }
    }

    /// Process learning signal
    pub async fn process_signal(&self, signal: LearningSignal) -> Result<(), SelfPromptingAgentError> {
        // Delegate to learning bridge for processing
        let bridge = LearningBridge::new();
        bridge.process_signal(signal).await
    }

    /// Generate insights from learning data
    pub async fn generate_insights(&self) -> Result<Vec<String>, SelfPromptingAgentError> {
        // Get learning statistics and convert to insights
        match self.learning_service.get_statistics().await {
            Ok(stats) => {
                let mut insights = Vec::new();
                
                if stats.total_experiences > 0 {
                    insights.push(format!("Learning system has processed {} experiences", stats.total_experiences));
                }
                
                if stats.total_patterns > 0 {
                    insights.push(format!("Identified {} patterns from execution data", stats.total_patterns));
                }
                
                if stats.total_improvements > 0 {
                    insights.push(format!("Generated {} improvement recommendations", stats.total_improvements));
                }
                
                if stats.average_confidence > 0.0 {
                    insights.push(format!("Average confidence: {:.2}", stats.average_confidence));
                }
                
                if insights.is_empty() {
                    insights.push("Learning system operational".to_string());
                }
                
                Ok(insights)
            }
            Err(e) => {
                tracing::warn!("Failed to get learning statistics: {}", e);
                Err(SelfPromptingAgentError::Learning(format!("Failed to generate insights: {}", e)))
            }
        }
    }
}
