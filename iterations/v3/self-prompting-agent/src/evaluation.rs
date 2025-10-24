//! Evaluation framework for self-prompting agent
//!
//! Provides comprehensive evaluation of task results against quality criteria.

use std::sync::Arc;
use async_trait::async_trait;

use crate::types::{TaskResult, EvalReport, EvalStatus, SelfPromptingAgentError};

/// Evaluation orchestrator
pub struct EvaluationOrchestrator {
    evaluators: Vec<Box<dyn Evaluator>>,
}

/// Evaluation result
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub score: f64,
    pub status: EvalStatus,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Evaluator trait for pluggable evaluation strategies
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// Evaluate a task result
    async fn evaluate(&self, result: &TaskResult) -> Result<EvaluationResult, SelfPromptingAgentError>;

    /// Get evaluator name
    fn name(&self) -> &str;

    /// Get evaluator priority (higher = run first)
    fn priority(&self) -> i32 { 0 }
}

impl EvaluationOrchestrator {
    /// Create a new evaluation orchestrator
    pub fn new() -> Self {
        Self {
            evaluators: Vec::new(),
        }
    }

    /// Add an evaluator
    pub fn add_evaluator(&mut self, evaluator: Box<dyn Evaluator>) {
        self.evaluators.push(evaluator);
        self.evaluators.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Evaluate a task result
    pub async fn evaluate_result(&self, result: &TaskResult) -> Result<EvaluationResult, SelfPromptingAgentError> {
        if self.evaluators.is_empty() {
            // Default evaluation
            return Ok(EvaluationResult {
                score: result.final_report.score,
                status: result.final_report.status.clone(),
                issues: result.final_report.failed_criteria.clone(),
                recommendations: vec!["Add more evaluators for comprehensive evaluation".to_string()],
            });
        }

        let mut combined_score = 0.0;
        let mut all_issues = Vec::new();
        let mut all_recommendations = Vec::new();
        let mut status = EvalStatus::Pass;

        for evaluator in &self.evaluators {
            let eval_result = evaluator.evaluate(result).await?;

            combined_score += eval_result.score;
            all_issues.extend(eval_result.issues);
            all_recommendations.extend(eval_result.recommendations);

            // Use the worst status
            if matches!(eval_result.status, EvalStatus::Fail) || matches!(status, EvalStatus::Partial) {
                status = EvalStatus::Fail;
            } else if matches!(eval_result.status, EvalStatus::Partial) {
                status = EvalStatus::Partial;
            }
        }

        let average_score = combined_score / self.evaluators.len() as f64;

        Ok(EvaluationResult {
            score: average_score,
            status,
            issues: all_issues,
            recommendations: all_recommendations,
        })
    }
}

/// Basic code quality evaluator
pub struct CodeQualityEvaluator;

#[async_trait]
impl Evaluator for CodeQualityEvaluator {
    async fn evaluate(&self, result: &TaskResult) -> Result<EvaluationResult, SelfPromptingAgentError> {
        let mut score: f64 = 0.5; // Base score
        let mut issues = Vec::new();

        // Check artifacts
        for artifact in &result.artifacts {
            match artifact.artifact_type {
                crate::types::ArtifactType::Code => {
                    let content = &artifact.content;

                    // Basic code quality checks
                    if content.contains("TODO") || content.contains("FIXME") {
                        issues.push("Code contains TODO/FIXME comments".to_string());
                        score -= 0.1;
                    }

                    if content.contains("println!") {
                        issues.push("Code contains debug prints".to_string());
                        score -= 0.05;
                    }

                    if content.lines().count() > 100 {
                        issues.push("Code file is very long (>100 lines)".to_string());
                        score -= 0.05;
                    }

                    // Reward good practices
                    if content.contains("///") || content.contains("//!") {
                        score += 0.05; // Documentation
                    }

                    if content.contains("Result<") || content.contains("Option<") {
                        score += 0.05; // Error handling
                    }
                }
                _ => {}
            }
        }

        score = score.max(0.0).min(1.0);

        Ok(EvaluationResult {
            score,
            status: if score >= 0.7 { EvalStatus::Pass } else { EvalStatus::Partial },
            issues,
            recommendations: vec![
                "Add comprehensive error handling".to_string(),
                "Include documentation comments".to_string(),
                "Remove debug prints before production".to_string(),
            ],
        })
    }

    fn name(&self) -> &str {
        "Code Quality Evaluator"
    }

    fn priority(&self) -> i32 {
        10 // High priority
    }
}

/// Performance evaluator
pub struct PerformanceEvaluator;

#[async_trait]
impl Evaluator for PerformanceEvaluator {
    async fn evaluate(&self, result: &TaskResult) -> Result<EvaluationResult, SelfPromptingAgentError> {
        let execution_time = result.execution_time_ms as f64;

        let score: f64 = if execution_time < 100.0 {
            1.0 // Excellent
        } else if execution_time < 500.0 {
            0.8 // Good
        } else if execution_time < 2000.0 {
            0.6 // Acceptable
        } else {
            0.3 // Poor
        };

        let issues = if execution_time > 5000.0 {
            vec!["Execution time is very high (>5s)".to_string()]
        } else if execution_time > 2000.0 {
            vec!["Execution time is high (>2s)".to_string()]
        } else {
            vec![]
        };

        Ok(EvaluationResult {
            score,
            status: if score >= 0.6 { EvalStatus::Pass } else { EvalStatus::Partial },
            issues,
            recommendations: vec![
                "Optimize algorithm complexity".to_string(),
                "Consider caching for repeated operations".to_string(),
                "Profile performance bottlenecks".to_string(),
            ],
        })
    }

    fn name(&self) -> &str {
        "Performance Evaluator"
    }

    fn priority(&self) -> i32 {
        5
    }
}
