//! Feedback Loop for Continuous Improvement
//!
//! Manages the continuous learning and improvement cycle by collecting
//! feedback from refinement iterations and updating strategies.

use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::coordinator::{RefinementDecision, RefinementStrategy};
use super::strategy::{StrategyExecutionResult, ActionType};

/// Feedback loop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackLoopConfig {
    /// Enable learning from feedback
    pub enable_learning: bool,
    /// Minimum feedback samples for learning
    pub min_feedback_samples: usize,
    /// Learning rate for strategy adaptation
    pub learning_rate: f64,
    /// Feedback retention period (days)
    pub retention_days: u64,
    /// Enable predictive analytics
    pub enable_prediction: bool,
}

/// Quality feedback data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFeedback {
    pub task_id: uuid::Uuid,
    pub iteration: usize,
    pub timestamp: DateTime<Utc>,
    pub quality_before: f64,
    pub quality_after: f64,
    pub strategy_used: RefinementStrategy,
    pub actions_taken: Vec<String>,
    pub time_spent: u64,
    pub success: bool,
    pub issues_resolved: Vec<String>,
    pub remaining_issues: Vec<String>,
    pub user_feedback: Option<String>,
}

/// Learning metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub strategy_effectiveness: HashMap<String, StrategyEffectiveness>,
    pub action_success_rates: HashMap<String, ActionSuccessRate>,
    pub time_predictions: HashMap<String, TimePrediction>,
    pub quality_improvements: Vec<QualityImprovementTrend>,
    pub total_feedback_samples: usize,
    pub last_updated: DateTime<Utc>,
}

/// Strategy effectiveness data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEffectiveness {
    pub strategy_name: String,
    pub times_used: usize,
    pub average_quality_improvement: f64,
    pub average_time_spent: u64,
    pub success_rate: f64,
    pub confidence_score: f64,
}

/// Action success rate data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSuccessRate {
    pub action_type: String,
    pub times_executed: usize,
    pub success_rate: f64,
    pub average_time: u64,
    pub quality_impact: f64,
}

/// Time prediction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePrediction {
    pub strategy_name: String,
    pub average_time: u64,
    pub standard_deviation: u64,
    pub confidence_interval: (u64, u64),
    pub samples: usize,
}

/// Quality improvement trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityImprovementTrend {
    pub timestamp: DateTime<Utc>,
    pub quality_score: f64,
    pub strategy_used: String,
    pub improvement_amount: f64,
}

/// Feedback loop for continuous improvement
pub struct FeedbackLoop {
    config: FeedbackLoopConfig,
    feedback_data: Arc<RwLock<Vec<QualityFeedback>>>,
    learning_metrics: Arc<RwLock<LearningMetrics>>,
}

impl FeedbackLoop {
    pub fn new(config: FeedbackLoopConfig) -> Self {
        Self {
            config,
            feedback_data: Arc::new(RwLock::new(Vec::new())),
            learning_metrics: Arc::new(RwLock::new(LearningMetrics {
                strategy_effectiveness: HashMap::new(),
                action_success_rates: HashMap::new(),
                time_predictions: HashMap::new(),
                quality_improvements: Vec::new(),
                total_feedback_samples: 0,
                last_updated: Utc::now(),
            })),
        }
    }

    /// Record feedback from a refinement iteration
    pub async fn record_feedback(
        &self,
        decision: &RefinementDecision,
        execution_result: &StrategyExecutionResult,
        quality_before: f64,
        quality_after: f64,
        user_feedback: Option<String>,
    ) -> Result<(), FeedbackLoopError> {
        let feedback = QualityFeedback {
            task_id: decision.task_id,
            iteration: decision.iteration,
            timestamp: Utc::now(),
            quality_before,
            quality_after,
            strategy_used: execution_result.strategy.clone(),
            actions_taken: execution_result.actions_taken.iter()
                .map(|a| a.description.clone())
                .collect(),
            time_spent: execution_result.time_spent,
            success: execution_result.success,
            issues_resolved: execution_result.issues_resolved.clone(),
            remaining_issues: execution_result.remaining_issues.clone(),
            user_feedback,
        };

        // Store feedback
        {
            let mut feedback_data = self.feedback_data.write().await;
            feedback_data.push(feedback.clone());
        }

        // Update learning metrics if enabled
        if self.config.enable_learning {
            self.update_learning_metrics(&feedback, execution_result).await?;
        }

        tracing::debug!("Recorded feedback for task {} iteration {}", decision.task_id, decision.iteration);
        Ok(())
    }

    /// Get recommendations based on learning metrics
    pub async fn get_recommendations(
        &self,
        current_quality: f64,
        available_strategies: &[RefinementStrategy],
    ) -> Result<Vec<StrategyRecommendation>, FeedbackLoopError> {
        if !self.config.enable_learning || !self.config.enable_prediction {
            return Ok(Vec::new());
        }

        let metrics = self.learning_metrics.read().await;
        let mut recommendations = Vec::new();

        for strategy in available_strategies {
            if let Some(effectiveness) = metrics.strategy_effectiveness.get(&self.strategy_key(strategy)) {
                let predicted_improvement = effectiveness.average_quality_improvement;
                let predicted_time = effectiveness.average_time_spent;
                let confidence = effectiveness.confidence_score;

                // Calculate cost-benefit score
                let cost_benefit = if predicted_time > 0 {
                    (predicted_improvement / current_quality) / (predicted_time as f64 / 60.0) // per minute
                } else {
                    0.0
                };

                recommendations.push(StrategyRecommendation {
                    strategy: strategy.clone(),
                    predicted_quality_improvement: predicted_improvement,
                    predicted_time_cost: predicted_time,
                    confidence_score: confidence,
                    cost_benefit_score: cost_benefit,
                    reasoning: format!(
                        "Based on {} similar refinements: {:.1}% quality improvement, {} min average time",
                        effectiveness.times_used,
                        predicted_improvement * 100.0,
                        predicted_time
                    ),
                });
            }
        }

        // Sort by cost-benefit score
        recommendations.sort_by(|a, b| b.cost_benefit_score.partial_cmp(&a.cost_benefit_score).unwrap());

        Ok(recommendations)
    }

    /// Get learning metrics summary
    pub async fn get_metrics_summary(&self) -> Result<LearningMetrics, FeedbackLoopError> {
        Ok(self.learning_metrics.read().await.clone())
    }

    /// Predict outcome for a potential refinement
    pub async fn predict_outcome(
        &self,
        strategy: &RefinementStrategy,
        current_quality: f64,
    ) -> Result<OutcomePrediction, FeedbackLoopError> {
        if !self.config.enable_learning || !self.config.enable_prediction {
            return Ok(OutcomePrediction {
                strategy: strategy.clone(),
                predicted_quality: current_quality,
                predicted_time: 0,
                confidence: 0.0,
                based_on_samples: 0,
            });
        }

        let metrics = self.learning_metrics.read().await;

        if let Some(effectiveness) = metrics.strategy_effectiveness.get(&self.strategy_key(strategy)) {
            let predicted_improvement = effectiveness.average_quality_improvement;
            let predicted_quality = (current_quality + predicted_improvement).min(1.0);

            Ok(OutcomePrediction {
                strategy: strategy.clone(),
                predicted_quality,
                predicted_time: effectiveness.average_time_spent,
                confidence: effectiveness.confidence_score,
                based_on_samples: effectiveness.times_used,
            })
        } else {
            Ok(OutcomePrediction {
                strategy: strategy.clone(),
                predicted_quality: current_quality,
                predicted_time: 0,
                confidence: 0.0,
                based_on_samples: 0,
            })
        }
    }

    /// Clean up old feedback data
    pub async fn cleanup_old_feedback(&self) -> Result<usize, FeedbackLoopError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        let mut feedback_data = self.feedback_data.write().await;
        let initial_count = feedback_data.len();

        feedback_data.retain(|feedback| feedback.timestamp > cutoff_date);

        let removed_count = initial_count - feedback_data.len();

        if removed_count > 0 {
            tracing::info!("Cleaned up {} old feedback records", removed_count);
        }

        Ok(removed_count)
    }

    /// Update learning metrics from new feedback
    async fn update_learning_metrics(
        &self,
        feedback: &QualityFeedback,
        execution_result: &StrategyExecutionResult,
    ) -> Result<(), FeedbackLoopError> {
        let mut metrics = self.learning_metrics.write().await;

        let strategy_key = self.strategy_key(&feedback.strategy_used);

        // Update strategy effectiveness
        let effectiveness = metrics.strategy_effectiveness
            .entry(strategy_key.clone())
            .or_insert(StrategyEffectiveness {
                strategy_name: strategy_key.clone(),
                times_used: 0,
                average_quality_improvement: 0.0,
                average_time_spent: 0,
                success_rate: 0.0,
                confidence_score: 0.0,
            });

        let improvement = feedback.quality_after - feedback.quality_before;
        effectiveness.times_used += 1;

        // Update running averages
        let n = effectiveness.times_used as f64;
        effectiveness.average_quality_improvement =
            (effectiveness.average_quality_improvement * (n - 1.0) + improvement) / n;
        effectiveness.average_time_spent =
            ((effectiveness.average_time_spent as f64 * (n - 1.0) + feedback.time_spent as f64) / n) as u64;

        // Update success rate
        let success_contribution = if feedback.success { 1.0 } else { 0.0 };
        effectiveness.success_rate =
            (effectiveness.success_rate * (n - 1.0) + success_contribution) / n;

        // Calculate confidence based on sample size and consistency
        effectiveness.confidence_score = (n / (n + 10.0)).min(0.95); // Approaches 95% with more samples

        // Update action success rates
        for action in &execution_result.actions_taken {
            let action_key = format!("{:?}", action.action_type);
            let action_rate = metrics.action_success_rates
                .entry(action_key.clone())
                .or_insert(ActionSuccessRate {
                    action_type: action_key,
                    times_executed: 0,
                    success_rate: 0.0,
                    average_time: 0,
                    quality_impact: 0.0,
                });

            action_rate.times_executed += 1;
            let m = action_rate.times_executed as f64;

            // Update success rate
            action_rate.success_rate =
                (action_rate.success_rate * (m - 1.0) + if feedback.success { 1.0 } else { 0.0 }) / m;

            // Update average time and quality impact
            action_rate.average_time =
                ((action_rate.average_time as f64 * (m - 1.0) + action.estimated_effort as f64) / m) as u64;
            action_rate.quality_impact =
                (action_rate.quality_impact * (m - 1.0) + improvement) / m;
        }

        // Update time predictions
        let time_pred = metrics.time_predictions
            .entry(strategy_key)
            .or_insert(TimePrediction {
                strategy_name: strategy_key.clone(),
                average_time: 0,
                standard_deviation: 0,
                confidence_interval: (0, 0),
                samples: 0,
            });

        time_pred.samples += 1;
        let k = time_pred.samples as f64;

        // Update running statistics
        let old_avg = time_pred.average_time as f64;
        time_pred.average_time =
            ((old_avg * (k - 1.0) + feedback.time_spent as f64) / k) as u64;

        // Simplified standard deviation calculation
        let variance = ((feedback.time_spent as f64 - time_pred.average_time as f64).powi(2) / k).sqrt() as u64;
        time_pred.standard_deviation = variance;

        // 95% confidence interval
        let margin = (1.96 * variance as f64 / k.sqrt()) as u64;
        time_pred.confidence_interval = (
            time_pred.average_time.saturating_sub(margin),
            time_pred.average_time + margin,
        );

        // Record quality improvement trend
        metrics.quality_improvements.push(QualityImprovementTrend {
            timestamp: feedback.timestamp,
            quality_score: feedback.quality_after,
            strategy_used: strategy_key,
            improvement_amount: improvement,
        });

        metrics.total_feedback_samples += 1;
        metrics.last_updated = Utc::now();

        Ok(())
    }

    /// Generate strategy key for metrics
    fn strategy_key(&self, strategy: &RefinementStrategy) -> String {
        match strategy {
            RefinementStrategy::TargetedFixes(gates) => format!("targeted_{}", gates.join("_")),
            RefinementStrategy::QualityOverhaul => "quality_overhaul".to_string(),
            RefinementStrategy::RefactorArchitecture => "architecture_refactor".to_string(),
            RefinementStrategy::EnhanceTesting => "testing_enhancement".to_string(),
            RefinementStrategy::PerformanceOptimization => "performance_opt".to_string(),
            RefinementStrategy::SecurityEnhancement => "security_enhancement".to_string(),
            RefinementStrategy::DocumentationFocus => "documentation".to_string(),
            RefinementStrategy::CouncilSpecified(_) => "council_specified".to_string(),
        }
    }
}

/// Strategy recommendation based on learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecommendation {
    pub strategy: RefinementStrategy,
    pub predicted_quality_improvement: f64,
    pub predicted_time_cost: u64,
    pub confidence_score: f64,
    pub cost_benefit_score: f64,
    pub reasoning: String,
}

/// Outcome prediction for a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomePrediction {
    pub strategy: RefinementStrategy,
    pub predicted_quality: f64,
    pub predicted_time: u64,
    pub confidence: f64,
    pub based_on_samples: usize,
}

pub type Result<T> = std::result::Result<T, FeedbackLoopError>;

#[derive(Debug, thiserror::Error)]
pub enum FeedbackLoopError {
    #[error("Feedback recording failed: {0}")]
    RecordingError(String),

    #[error("Metrics update failed: {0}")]
    MetricsError(String),

    #[error("Prediction failed: {0}")]
    PredictionError(String),

    #[error("Insufficient data for analysis")]
    InsufficientData,
}
