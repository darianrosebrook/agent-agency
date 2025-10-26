//! Predictive Learning System for V3
//!
//! This module implements V3's superior learning capabilities that surpass V2's
//! reactive learning with proactive performance prediction, strategy optimization,
//! resource prediction, outcome prediction, and meta-learning acceleration.

pub mod performance;
pub mod strategy;
pub mod resource;
pub mod outcome;
pub mod learning_accelerator;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::council_types::{LearningInsights, TaskOutcome};
use crate::predictive_learning::performance::PerformancePrediction;
use crate::predictive_learning::strategy::StrategyOptimization;
use crate::predictive_learning::resource::ResourcePrediction;
use crate::predictive_learning::outcome::OutcomePrediction;
use crate::predictive_learning::learning_accelerator::LearningAcceleration;

use performance::PerformancePredictor;
use strategy::StrategyOptimizer;
use resource::ResourcePredictor;
use outcome::OutcomePredictor;
use learning_accelerator::LearningAccelerator;

/// Predictive Learning System that surpasses V2's reactive learning
#[derive(Debug)]
pub struct PredictiveLearningSystem {
    performance_predictor: Arc<PerformancePredictor>,
    strategy_optimizer: Arc<StrategyOptimizer>,
    resource_predictor: Arc<ResourcePredictor>,
    outcome_predictor: Arc<OutcomePredictor>,
    learning_accelerator: Arc<LearningAccelerator>,
    historical_data: Arc<RwLock<HashMap<String, LearningHistory>>>,
}

/// Learning history for tracking progress
#[derive(Debug, Clone)]
pub struct LearningHistory {
    pub task_id: uuid::Uuid,
    pub performance_history: Vec<performance::PerformanceSnapshot>,
    pub strategy_history: Vec<strategy::StrategySnapshot>,
    pub resource_history: Vec<resource::ResourceSnapshot>,
    pub outcome_history: Vec<outcome::OutcomeSnapshot>,
    pub learning_events: Vec<learning_accelerator::LearningEvent>,
}

/// Learning event for tracking learning progress
#[derive(Debug, Clone)]
pub struct LearningEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: LearningEventType,
    pub description: String,
    pub impact_score: f64,
}

/// Type of learning event
#[derive(Debug, Clone)]
pub enum LearningEventType {
    OutcomeAchieved,
    StrategyLearned,
    PatternDiscovered,
    ImprovementApplied,
}

impl PredictiveLearningSystem {
    /// Create a new Predictive Learning System
    pub fn new() -> Self {
        Self {
            performance_predictor: Arc::new(PerformancePredictor::new()),
            strategy_optimizer: Arc::new(StrategyOptimizer::new()),
            resource_predictor: Arc::new(ResourcePredictor::new()),
            outcome_predictor: Arc::new(OutcomePredictor::new()),
            learning_accelerator: Arc::new(LearningAccelerator::new()),
            historical_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// V3's superior learning capabilities
    pub async fn learn_and_predict(&self, task_outcome: &TaskOutcome) -> anyhow::Result<LearningInsights> {
        info!(
            "Starting predictive learning analysis for task: {}",
            task_outcome.task_id
        );

        // 1. Predict future performance (V2: no prediction)
        let performance_prediction = self
            .performance_predictor
            .predict_future(task_outcome)
            .await?;

        // 2. Optimize strategies proactively (V2: reactive optimization)
        let strategy_optimization = self
            .strategy_optimizer
            .optimize_strategies(task_outcome)
            .await?;

        // 3. Predict resource needs (V2: no resource prediction)
        let resource_prediction = self.resource_predictor.predict_needs(task_outcome).await?;

        // 4. Predict task outcomes (V2: no outcome prediction)
        let outcome_prediction = self
            .outcome_predictor
            .predict_outcomes(task_outcome)
            .await?;

        // 5. Accelerate learning through meta-learning (V2: no meta-learning)
        let learning_acceleration = self
            .learning_accelerator
            .accelerate_learning(task_outcome)
            .await?;

        // Update historical data
        self.update_learning_history(task_outcome).await?;

        let insights = LearningInsights {
            performance_prediction,
            strategy_optimization,
            resource_prediction,
            outcome_prediction,
            learning_acceleration,
        };

        info!(
            "Completed predictive learning analysis for task: {}",
            task_outcome.task_id
        );
        Ok(insights)
    }

    /// Update learning history with new task outcome
    async fn update_learning_history(&self, task_outcome: &TaskOutcome) -> anyhow::Result<()> {
        let mut history = self.historical_data.write().await;

        let entry = history
            .entry(task_outcome.task_id.to_string())
            .or_insert_with(|| LearningHistory {
                task_id: task_outcome.task_id,
                performance_history: Vec::new(),
                strategy_history: Vec::new(),
                resource_history: Vec::new(),
                outcome_history: Vec::new(),
                learning_events: Vec::new(),
            });

        // Add performance snapshot
        entry.performance_history.push(performance::PerformanceSnapshot {
            timestamp: task_outcome.timestamp,
            performance_score: task_outcome.performance_score,
            metrics: task_outcome.resource_usage.clone(),
            context: format!("Task outcome: {:?}", task_outcome.outcome_type),
        });

        // Add outcome snapshot
        entry.outcome_history.push(outcome::OutcomeSnapshot {
            timestamp: task_outcome.timestamp,
            outcome_type: task_outcome.outcome_type.clone(),
            success_score: task_outcome.performance_score,
            duration_ms: task_outcome.duration_ms,
        });

        // Add learning event
        entry.learning_events.push(learning_accelerator::LearningEvent {
            timestamp: task_outcome.timestamp,
            event_type: learning_accelerator::LearningEventType::OutcomeAchieved,
            description: format!(
                "Task completed with {:?} outcome",
                task_outcome.outcome_type
            ),
            impact_score: task_outcome.performance_score,
        });

        Ok(())
    }
}
