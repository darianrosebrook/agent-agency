//! Learning and Optimization Interface
//!
//! Defines interfaces for reinforcement learning, optimization algorithms,
//! and self-improvement capabilities across the agent system.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Re-export learning types from contracts for backward compatibility
pub use agent_agency_contracts::types::learning::{
    AlgorithmConfig, LearningError, LearningResult,
};

/// Q-table for Q-learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QTable {
    /// State-action value table
    table: HashMap<String, HashMap<String, f64>>,
}

impl QTable {
    /// Create a new empty Q-table
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Get Q-value for state-action pair
    pub fn get(&self, state: &str, action: &str) -> f64 {
        self.table
            .get(state)
            .and_then(|actions| actions.get(action))
            .copied()
            .unwrap_or(0.0)
    }

    /// Set Q-value for state-action pair
    pub fn set(&mut self, state: &str, action: &str, value: f64) {
        self.table
            .entry(state.to_string())
            .or_insert_with(HashMap::new)
            .insert(action.to_string(), value);
    }

    /// Get the best action for a state
    pub fn get_best_action(&self, state: &str) -> Option<String> {
        self.table
            .get(state)?
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(action, _)| action.clone())
    }

    /// Get all actions for a state
    pub fn get_actions(&self, state: &str) -> Vec<String> {
        self.table
            .get(state)
            .map(|actions| actions.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Reinforcement learning algorithm trait
#[async_trait]
pub trait ReinforcementLearningAlgorithm: Send + Sync {
    /// Update the algorithm with a new experience
    async fn update(
        &mut self,
        state: &str,
        action: &str,
        reward: f64,
        next_state: &str,
    ) -> LearningResult<()>;

    /// Select an action for the given state
    async fn select_action(
        &mut self,
        state: &str,
        available_actions: &[String],
    ) -> LearningResult<String>;

    /// Get the algorithm's configuration
    fn config(&self) -> &AlgorithmConfig;

    /// Get algorithm statistics
    fn statistics(&self) -> AlgorithmStatistics;
}

/// Statistics for a learning algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmStatistics {
    /// Total number of updates performed
    pub total_updates: usize,
    /// Current exploration rate
    pub current_exploration_rate: f64,
    /// Average reward per episode
    pub average_reward: f64,
    /// Best action-value found
    pub best_q_value: f64,
    /// Number of states learned
    pub states_learned: usize,
    /// Number of actions learned
    pub actions_learned: usize,
    /// Convergence status
    pub converged: bool,
}

/// Experience tuple for reinforcement learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Current state
    pub state: String,
    /// Action taken
    pub action: String,
    /// Reward received
    pub reward: f64,
    /// Next state
    pub next_state: String,
    /// Episode number
    pub episode: usize,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Learning context for decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContext {
    /// Current task identifier
    pub task_id: String,
    /// Current state representation
    pub state: String,
    /// Available actions
    pub available_actions: Vec<String>,
    /// Historical performance data
    pub historical_performance: Vec<TaskPerformance>,
    /// Current system metrics
    pub system_metrics: SystemMetrics,
}

/// Task performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPerformance {
    /// Task identifier
    pub task_id: String,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
    /// Average execution time
    pub avg_execution_time: std::time::Duration,
    /// Quality score (0.0-1.0)
    pub quality_score: f64,
    /// Resource usage
    pub resource_usage: ResourceUsage,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System metrics for learning context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Available models
    pub available_models: Vec<String>,
    /// Active task count
    pub active_tasks: usize,
    /// Queue depth
    pub queue_depth: usize,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU time used
    pub cpu_time: std::time::Duration,
    /// Memory peak usage in bytes
    pub memory_peak: u64,
    /// I/O operations performed
    pub io_operations: u64,
    /// Network bytes transferred
    pub network_bytes: u64,
}

/// Optimization goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationGoal {
    /// Minimize execution time
    MinimizeTime,
    /// Minimize resource usage
    MinimizeResources,
    /// Maximize quality
    MaximizeQuality,
    /// Balance all factors
    Balanced,
}

/// Learning service interface for self-improvement
#[async_trait]
pub trait LearningService: Send + Sync + std::fmt::Debug {
    /// Learn from task execution results
    async fn learn_from_execution(
        &self,
        context: &LearningContext,
        performance: &TaskPerformance,
    ) -> LearningResult<LearningInsights>;

    /// Get optimization recommendations
    async fn get_optimization_recommendations(
        &self,
        context: &LearningContext,
        goal: OptimizationGoal,
    ) -> LearningResult<Vec<OptimizationRecommendation>>;

    /// Update learning model with new experiences
    async fn update_model(&self, experiences: Vec<Experience>) -> LearningResult<()>;

    /// Get learning statistics
    async fn get_statistics(&self) -> LearningResult<LearningStatistics>;
}

/// Insights gained from learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningInsights {
    /// Key patterns identified
    pub patterns: Vec<Pattern>,
    /// Performance improvements found
    pub improvements: Vec<Improvement>,
    /// Recommendations for future tasks
    pub recommendations: Vec<OptimizationRecommendation>,
    /// Confidence in insights (0.0-1.0)
    pub confidence: f64,
}

/// Identified pattern in task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern type
    pub pattern_type: PatternType,
    /// Description of the pattern
    pub description: String,
    /// Frequency of occurrence
    pub frequency: f64,
    /// Impact on performance
    pub impact: f64,
}

/// Types of patterns that can be identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Resource bottleneck
    ResourceBottleneck,
    /// Model selection inefficiency
    ModelInefficiency,
    /// Task complexity mismatch
    ComplexityMismatch,
    /// Timing optimization opportunity
    TimingOptimization,
    /// Quality-resource tradeoff
    QualityTradeoff,
}

/// Performance improvement identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    /// Type of improvement
    pub improvement_type: ImprovementType,
    /// Expected benefit (0.0-1.0)
    pub expected_benefit: f64,
    /// Implementation difficulty
    pub difficulty: Difficulty,
    /// Description
    pub description: String,
}

/// Types of improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementType {
    /// Algorithm optimization
    AlgorithmOptimization,
    /// Resource allocation
    ResourceAllocation,
    /// Model selection
    ModelSelection,
    /// Caching strategy
    CachingStrategy,
    /// Parallelization
    Parallelization,
}

/// Implementation difficulty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    /// Easy to implement
    Easy,
    /// Moderate effort
    Moderate,
    /// Significant effort required
    Hard,
    /// Major architectural changes
    Complex,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Description of the recommendation
    pub description: String,
    /// Expected improvement
    pub expected_improvement: f64,
    /// Confidence in recommendation
    pub confidence: f64,
    /// Implementation priority
    pub priority: Priority,
}

/// Types of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Change model selection
    ChangeModel,
    /// Modify resource allocation
    AdjustResources,
    /// Update algorithm parameters
    TuneParameters,
    /// Implement caching
    AddCaching,
    /// Change execution strategy
    ExecutionStrategy,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    /// High priority - implement immediately
    High,
    /// Medium priority - implement soon
    Medium,
    /// Low priority - implement when convenient
    Low,
}

/// Overall learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatistics {
    /// Total experiences processed
    pub total_experiences: usize,
    /// Total patterns identified
    pub total_patterns: usize,
    /// Total improvements found
    pub total_improvements: usize,
    /// Average learning confidence
    pub average_confidence: f64,
    /// Learning rate over time
    pub learning_rate: f64,
    /// Most effective recommendations
    pub top_recommendations: Vec<OptimizationRecommendation>,
}
