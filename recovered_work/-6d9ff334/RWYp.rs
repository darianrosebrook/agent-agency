//! Learning Bridge for converting self-prompting results to reflexive learning signals

use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::evaluation::{EvalReport};
use crate::types::EvalStatus;
use crate::types::{TaskResult, StopReason};

/// Learning bridge that converts self-prompting execution to reflexive learning signals
pub struct LearningBridge {
    reflexive_learning_system: Arc<dyn ReflexiveLearningSystem>,
}

/// Learning signals that can be processed by the reflexive learning system
#[derive(Debug, Clone)]
pub enum LearningSignal {
    /// Quality improvement trajectory over iterations
    QualityTrajectory {
        task_id: Uuid,
        improvement: f64,
        iterations: usize,
        converged: bool,
        task_type: String,
        timestamp: DateTime<Utc>,
    },

    /// Model performance on specific task types
    ModelPerformance {
        model_id: String,
        task_type: String,
        success: bool,
        score: f64,
        iterations: usize,
        execution_time_ms: u64,
        stop_reason: StopReason,
        timestamp: DateTime<Utc>,
    },

    /// Iteration efficiency patterns
    IterationEfficiency {
        task_type: String,
        iterations: usize,
        quality: f64,
        time_per_iteration: f64,
        converged_quickly: bool,
        timestamp: DateTime<Utc>,
    },

    /// Satisficing behavior effectiveness
    SatisficingEffectiveness {
        stopped_early: bool,
        quality_delta: f64,
        iterations_saved: usize,
        task_type: String,
        timestamp: DateTime<Utc>,
    },

    /// Evaluation pattern insights
    EvaluationPattern {
        task_type: String,
        common_failures: Vec<String>,
        average_score: f64,
        evaluation_count: usize,
        timestamp: DateTime<Utc>,
    },

    /// Convert to reflexive learning SelfPromptingSignal
    SelfPrompting(crate::types::SelfPromptingSignal),
}

/// Trait for reflexive learning system integration
#[async_trait::async_trait]
pub trait ReflexiveLearningSystem: Send + Sync {
    /// Process learning signals from self-prompting execution
    async fn process_signals(&self, signals: Vec<LearningSignal>) -> Result<LearningUpdate, LearningError>;

    /// Update model preferences based on performance data
    async fn update_model_preferences(&self, model_id: &str, task_type: &str, score: f64) -> Result<(), LearningError>;

    /// Adjust satisficing thresholds based on learning
    async fn tune_satisficing_thresholds(&self, feedback: SatisficingFeedback) -> Result<(), LearningError>;
}

/// Learning update result
#[derive(Debug, Clone)]
pub struct LearningUpdate {
    pub signals_processed: usize,
    pub insights_generated: Vec<String>,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Satisficing feedback for tuning
#[derive(Debug, Clone)]
pub enum SatisficingFeedback {
    TooConservative,    // Stopped too early, could have continued
    TooAggressive,      // Continued too long, wasted iterations
    MaxIterationsTooLow, // Hit max iterations but could have benefited from more
    MaxIterationsTooHigh, // Rarely hit max iterations
}

/// Learning errors
#[derive(Debug, thiserror::Error)]
pub enum LearningError {
    #[error("Processing failed: {0}")]
    ProcessingError(String),

    #[error("Invalid signal data: {0}")]
    InvalidData(String),

    #[error("System unavailable: {0}")]
    SystemUnavailable(String),
}

impl LearningBridge {
    /// Create a new learning bridge
    pub fn new(reflexive_learning_system: Arc<dyn ReflexiveLearningSystem>) -> Self {
        Self {
            reflexive_learning_system,
        }
    }

    /// Process a task result and generate learning signals
    pub async fn process_task_result(
        &self,
        task_result: &TaskResult,
        eval_history: &[EvalReport],
    ) -> Result<LearningUpdate, LearningError> {
        // Generate learning signals from the execution
        let signals = self.extract_learning_signals(task_result, eval_history)?;

        // Send to reflexive learning system
        self.reflexive_learning_system.process_signals(signals).await
    }

    /// Extract learning signals from task execution
    fn extract_learning_signals(
        &self,
        task_result: &TaskResult,
        eval_history: &[EvalReport],
    ) -> Result<Vec<LearningSignal>, LearningError> {
        let mut signals = Vec::new();

        // 1. Quality trajectory signal
        if eval_history.len() > 1 {
            let improvement = self.calculate_quality_improvement(eval_history)?;
            signals.push(LearningSignal::QualityTrajectory {
                task_id: task_result.task_id,
                improvement,
                iterations: eval_history.len(),
                converged: matches!(task_result.final_report.status, EvalStatus::Pass),
                task_type: format!("{:?}", task_result.task_type),
                timestamp: Utc::now(),
            });
        }

        // 2. Model performance signal
        signals.push(LearningSignal::ModelPerformance {
            model_id: task_result.model_used.clone(),
            task_type: format!("{:?}", task_result.task_type),
            success: matches!(task_result.final_report.status, EvalStatus::Pass),
            score: task_result.final_report.score,
            iterations: task_result.iterations,
            execution_time_ms: task_result.total_time_ms,
            stop_reason: task_result.stop_reason.unwrap_or(StopReason::Unknown),
            timestamp: Utc::now(),
        });

        // 3. Iteration efficiency signal
        if task_result.iterations > 0 {
            let time_per_iteration = task_result.total_time_ms as f64 / task_result.iterations as f64;
            let converged_quickly = task_result.iterations <= 2 && matches!(task_result.final_report.status, EvalStatus::Pass);

            signals.push(LearningSignal::IterationEfficiency {
                task_type: format!("{:?}", task_result.task_type),
                iterations: task_result.iterations,
                quality: task_result.final_report.score,
                time_per_iteration,
                converged_quickly,
                timestamp: Utc::now(),
            });
        }

        // 4. Satisficing effectiveness signal
        if let Some(stop_reason) = task_result.stop_reason {
            let stopped_early = matches!(stop_reason, StopReason::Satisficed) && task_result.iterations < 3;
            let quality_delta = if eval_history.len() > 1 {
                self.calculate_quality_improvement(eval_history)?
            } else {
                0.0
            };

            // Estimate iterations that might have been saved
            let iterations_saved = if stopped_early { 2.max(task_result.iterations) - task_result.iterations } else { 0 };

            signals.push(LearningSignal::SatisficingEffectiveness {
                stopped_early,
                quality_delta,
                iterations_saved,
                task_type: format!("{:?}", task_result.task_type),
                timestamp: Utc::now(),
            });
        }

        // 5. Evaluation pattern signal
        let common_failures = self.extract_common_failures(eval_history);
        let average_score = eval_history.iter().map(|r| r.score).sum::<f64>() / eval_history.len() as f64;

        signals.push(LearningSignal::EvaluationPattern {
            task_type: format!("{:?}", task_result.task_type),
            common_failures,
            average_score,
            evaluation_count: eval_history.len(),
            timestamp: Utc::now(),
        });

        // 6. Self-prompting signals for reflexive learning system
        signals.push(LearningSignal::SelfPrompting(crate::types::SelfPromptingSignal::IterationEfficiency {
            iterations: task_result.iterations,
            quality: task_result.final_report.score,
            time: task_result.total_time_ms as f64 / task_result.iterations as f64,
        }));

        signals.push(LearningSignal::SelfPrompting(crate::types::SelfPromptingSignal::ModelPerformance {
            model_id: task_result.model_used.clone(),
            task_type: format!("{:?}", task_result.task_type),
            score: task_result.final_report.score,
        }));

        // Calculate satisficing effectiveness
        let stopped_early = matches!(task_result.stop_reason, Some(crate::types::StopReason::Satisficed));
        let quality_delta = if eval_history.len() > 1 {
            self.calculate_quality_improvement(eval_history)?
        } else {
            0.0
        };
        let iterations_saved = if stopped_early { 2.max(task_result.iterations) - task_result.iterations } else { 0 };

        signals.push(LearningSignal::SelfPrompting(crate::types::SelfPromptingSignal::SatisficingEffectiveness {
            stopped_early,
            quality_delta,
            iterations_saved,
        }));

        Ok(signals)
    }

    /// Calculate quality improvement over iterations
    fn calculate_quality_improvement(&self, history: &[EvalReport]) -> Result<f64, LearningError> {
        if history.len() < 2 {
            return Ok(0.0);
        }

        let first_score = history.first().ok_or_else(|| LearningError::InvalidData("Empty history".to_string()))?.score;
        let last_score = history.last().ok_or_else(|| LearningError::InvalidData("Empty history".to_string()))?.score;

        Ok(last_score - first_score)
    }

    /// Extract common failure patterns from evaluation history
    fn extract_common_failures(&self, history: &[EvalReport]) -> Vec<String> {
        let mut failure_counts = std::collections::HashMap::new();

        for report in history {
            for criterion in &report.criteria {
                if !criterion.passed {
                    *failure_counts.entry(criterion.id.clone()).or_insert(0) += 1;
                }
            }
        }

        // Return failures that occurred in more than 50% of evaluations
        let threshold = (history.len() / 2).max(1);

        failure_counts.into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(failure, _)| failure)
            .collect()
    }

    /// Update model preferences based on recent performance
    pub async fn update_model_preferences(
        &self,
        model_id: &str,
        task_type: &str,
        score: f64,
    ) -> Result<(), LearningError> {
        self.reflexive_learning_system.update_model_preferences(model_id, task_type, score).await
    }

    /// Provide satisficing feedback for tuning
    pub async fn provide_satisficing_feedback(
        &self,
        feedback: SatisficingFeedback,
    ) -> Result<(), LearningError> {
        self.reflexive_learning_system.tune_satisficing_thresholds(feedback).await
    }
}
