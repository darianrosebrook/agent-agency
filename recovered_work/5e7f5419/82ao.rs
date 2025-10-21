//! Evaluation framework for self-prompting agent

pub mod code_evaluator;
pub mod text_evaluator;
pub mod token_evaluator;
pub mod caws_evaluator;
pub mod satisficing;
pub mod flakiness;

pub use code_evaluator::CodeEvaluator;
pub use text_evaluator::TextEvaluator;
pub use token_evaluator::TokenEvaluator;
pub use caws_evaluator::CawsEvaluator;
pub use satisficing::{SatisficingEvaluator, SatisficingDecision};
pub use flakiness::{FlakinessHardener, FlakinessConfig, FailureBucket, FailureCategory, HardenedEvaluationResult, RefinementPromptGenerator};

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::types::{Artifact, EvalStatus, StopReason, Task};

/// Evaluation criterion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCriterion {
    pub id: String,
    pub description: String,
    pub weight: f64, // 0..1, sum should be ≤ 1.0
    pub passed: bool,
    pub score: f64, // 0..1
    pub notes: Option<String>,
}

/// Evaluation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub task_id: String,
    pub artifact_paths: Vec<String>,
    pub status: EvalStatus,
    pub score: f64, // 0..1 weighted average
    pub thresholds_met: Vec<String>,
    pub thresholds_missed: Vec<String>,
    pub criteria: Vec<EvalCriterion>,
    pub iterations: usize,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub elapsed_ms: Option<u64>,
    pub stop_reason: Option<StopReason>,
    pub next_actions: Vec<String>,
    pub logs: Vec<String>,
    pub seed: Option<u64>,
    pub tool_versions: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Context for evaluation
#[derive(Debug, Clone)]
pub struct EvalContext {
    pub task: Task,
    pub iteration: usize,
    pub previous_reports: Vec<EvalReport>,
    pub config: EvaluationConfig,
}

/// Evaluation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub min_score: f64,
    pub mandatory_gates: Vec<String>,
    pub max_iterations: usize,
    pub min_improvement_threshold: f64,
    pub quality_ceiling_budget: usize,
}

/// Evaluation orchestrator
pub struct EvaluationOrchestrator {
    config: EvaluationConfig,
    history: Vec<EvalReport>,
    evaluators: Vec<Box<dyn Evaluator>>,
}

impl EvaluationOrchestrator {
    /// Create a new evaluation orchestrator
    pub fn new(config: EvaluationConfig) -> Self {
        let evaluators = vec![
            Box::new(CodeEvaluator::new()) as Box<dyn Evaluator>,
            Box::new(TextEvaluator::new()),
            Box::new(TokenEvaluator::new()),
            Box::new(CawsEvaluator::new()),
        ];

        Self {
            config,
            history: Vec::new(),
            evaluators,
        }
    }

    /// Run evaluation on artifacts
    pub async fn evaluate(&mut self, artifacts: &[Artifact], context: &EvalContext) -> Result<EvalReport, EvaluationError> {
        let start_time = std::time::Instant::now();
        let mut all_criteria = Vec::new();
        let mut logs = Vec::new();

        // Run all evaluators
        for evaluator in &self.evaluators {
            if evaluator.applies_to(&context.task.task_type) {
                match evaluator.evaluate(artifacts, context).await {
                    Ok(criteria) => {
                        all_criteria.extend(criteria);
                        logs.push(format!("{}: {} criteria", evaluator.evaluator_type(), criteria.len()));
                    }
                    Err(e) => {
                        logs.push(format!("{} evaluation failed: {}", evaluator.evaluator_type(), e));
                    }
                }
            }
        }

        // Calculate weighted score
        let (score, thresholds_met, thresholds_missed) = self.calculate_score(&all_criteria);

        // Determine status
        let status = self.determine_status(score, &thresholds_met, &thresholds_missed);

        // Determine stop reason if this is a final evaluation
        let stop_reason = self.determine_stop_reason(&context, score);

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        let report = EvalReport {
            task_id: context.task.id.to_string(),
            artifact_paths: artifacts.iter().map(|a| a.file_path.clone()).collect(),
            status,
            score,
            thresholds_met,
            thresholds_missed,
            criteria: all_criteria,
            iterations: context.iteration,
            prompt_tokens: None, // TODO: track from model
            completion_tokens: None,
            elapsed_ms: Some(elapsed_ms),
            stop_reason,
            next_actions: self.generate_next_actions(&context),
            logs,
            seed: None,
            tool_versions: HashMap::new(), // TODO: populate
            timestamp: Utc::now(),
        };

        // Add to history
        self.history.push(report.clone());

        Ok(report)
    }

    /// Get evaluation history
    pub fn history(&self) -> &[EvalReport] {
        &self.history
    }

    /// Calculate weighted score from criteria
    fn calculate_score(&self, criteria: &[EvalCriterion]) -> (f64, Vec<String>, Vec<String>) {
        let total_weight: f64 = criteria.iter().map(|c| c.weight).sum();
        let weighted_score: f64 = criteria.iter()
            .map(|c| c.score * c.weight)
            .sum();

        let score = if total_weight > 0.0 { weighted_score / total_weight } else { 0.0 };

        let thresholds_met = criteria.iter()
            .filter(|c| c.passed)
            .map(|c| c.id.clone())
            .collect();

        let thresholds_missed = criteria.iter()
            .filter(|c| !c.passed)
            .map(|c| c.id.clone())
            .collect();

        (score, thresholds_met, thresholds_missed)
    }

    /// Determine evaluation status
    fn determine_status(&self, score: f64, thresholds_met: &[String], thresholds_missed: &[String]) -> EvalStatus {
        // Check mandatory gates first
        for gate in &self.config.mandatory_gates {
            if thresholds_missed.contains(gate) {
                return EvalStatus::Fail;
            }
        }

        if score >= self.config.min_score {
            EvalStatus::Pass
        } else {
            EvalStatus::Iterate
        }
    }

    /// Determine stop reason based on context
    fn determine_stop_reason(&self, context: &EvalContext, score: f64) -> Option<StopReason> {
        // Check if we've reached max iterations
        if context.iteration >= self.config.max_iterations {
            return Some(StopReason::MaxIterations);
        }

        // Check if satisficed
        if score >= self.config.min_score {
            return Some(StopReason::Satisficed);
        }

        // Check for quality ceiling (no improvement streak)
        if context.previous_reports.len() >= self.config.quality_ceiling_budget {
            let recent_scores: Vec<f64> = context.previous_reports
                .iter()
                .rev()
                .take(self.config.quality_ceiling_budget)
                .map(|r| r.score)
                .collect();

            let max_recent = recent_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min_improvement = self.config.min_improvement_threshold;

            if (score - max_recent).abs() < min_improvement {
                return Some(StopReason::QualityCeiling);
            }
        }

        None
    }

    /// Generate next actions based on evaluation results
    fn generate_next_actions(&self, _context: &EvalContext) -> Vec<String> {
        // TODO: Implement based on failed criteria
        vec!["Address failed evaluation criteria".to_string()]
    }
}

/// Evaluator trait
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// Evaluate artifacts and return criteria
    async fn evaluate(&self, artifacts: &[Artifact], context: &EvalContext) -> Result<Vec<EvalCriterion>, EvaluationError>;

    /// Check if this evaluator applies to the given task type
    fn applies_to(&self, task_type: &crate::types::TaskType) -> bool;

    /// Get evaluator type name
    fn evaluator_type(&self) -> &'static str;
}

/// Evaluation errors
#[derive(Debug, thiserror::Error)]
pub enum EvaluationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Command execution error: {0}")]
    CommandError(String),

    #[error("Artifact evaluation failed: {0}")]
    ArtifactError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
