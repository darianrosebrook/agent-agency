//! Integration layer connecting self-prompting agent components
//!
//! This module provides the glue that connects:
//! - Model providers → Loop controller → File operations → Evaluation
//!
//! It implements the autonomous workflow described in the theory.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agent::SelfPromptingAgent;
use crate::evaluation::{EvaluationOrchestrator, EvalReport};
use crate::loop_controller::SelfPromptingLoop;
use crate::models::{ModelRegistry, ModelProvider};
use crate::sandbox::SandboxEnvironment;
use crate::types::{Task, TaskResult, ExecutionMode, SafetyMode};

/// Integrated autonomous agent that connects all components
pub struct IntegratedAutonomousAgent {
    loop_controller: SelfPromptingLoop,
    sandbox: Arc<RwLock<SandboxEnvironment>>,
    execution_mode: ExecutionMode,
}

impl IntegratedAutonomousAgent {
    /// Create a new integrated autonomous agent
    pub async fn new(
        model_registry: Arc<ModelRegistry>,
        evaluation_orchestrator: Arc<RwLock<EvaluationOrchestrator>>,
        execution_mode: ExecutionMode,
    ) -> Result<Self, IntegrationError> {
        // Initialize sandbox with appropriate safety mode
        let safety_mode = match execution_mode {
            ExecutionMode::Strict => SafetyMode::Strict,
            ExecutionMode::Auto => SafetyMode::Sandbox,
            ExecutionMode::DryRun => SafetyMode::Autonomous, // Dry run can be autonomous
        };

        let sandbox = SandboxEnvironment::new(
            std::env::temp_dir().join("agent-workspace"),
            vec!["src/".into(), "tests/".into()], // Allow list
            safety_mode,
            true, // Use git
        ).await?;

        let agent = SelfPromptingAgent::new(
            Arc::clone(&model_registry),
            Arc::clone(&evaluation_orchestrator),
        );

        Ok(Self {
            agent,
            evaluation_orchestrator,
            model_registry,
            sandbox: Arc::new(RwLock::new(sandbox)),
            execution_mode,
        })
    }

    /// Execute a task autonomously end-to-end
    pub async fn execute_task_autonomously(
        &mut self,
        task: Task,
    ) -> Result<TaskResult, IntegrationError> {
        // 1. Initialize loop with task
        let mut loop_controller = SelfPromptingLoop::new(
            task.clone(),
            self.execution_mode.clone(),
        );

        // 2. Run the autonomous loop
        loop {
            // Generate/refine with model
            let artifacts = self.agent.generate_artifacts(&loop_controller).await?;

            // Apply changes to sandbox if not dry run
            if !matches!(self.execution_mode, ExecutionMode::DryRun) {
                let mut sandbox = self.sandbox.write().await;
                for artifact in &artifacts {
                    if let Some(diff) = &artifact.unified_diff {
                        sandbox.apply_diff(diff, loop_controller.iteration()).await?;
                    }
                }
            }

            // Evaluate results
            let eval_result = {
                let mut orchestrator = self.evaluation_orchestrator.write().await;
                orchestrator.evaluate(&artifacts, &loop_controller.context()).await?
            };

            // Check if we should continue
            if self.should_stop(&eval_result, &loop_controller)? {
                return self.create_final_result(&eval_result, &loop_controller);
            }

            // Generate refinement prompt and continue
            let refinement_prompt = self.generate_refinement_prompt(&eval_result);
            loop_controller.add_refinement_context(refinement_prompt);
        }
    }

    /// Check if the loop should stop based on evaluation and satisficing logic
    fn should_stop(
        &self,
        eval_result: &EvalReport,
        loop_controller: &SelfPromptingLoop,
    ) -> Result<bool, IntegrationError> {
        // Check satisficing criteria
        if eval_result.status == crate::evaluation::EvalStatus::Pass {
            return Ok(true);
        }

        // Check iteration limits
        if loop_controller.iteration() >= loop_controller.max_iterations() {
            return Ok(true);
        }

        // Check for quality ceiling (no improvement)
        if self.detect_quality_ceiling(eval_result, loop_controller) {
            return Ok(true);
        }

        Ok(false)
    }

    /// Detect if we've hit a quality ceiling (no meaningful improvement)
    fn detect_quality_ceiling(
        &self,
        current_eval: &EvalReport,
        loop_controller: &SelfPromptingLoop,
    ) -> bool {
        const CEILING_THRESHOLD: f64 = 0.02; // 2% improvement threshold
        const CEILING_STREAK: usize = 2; // Consecutive evaluations without improvement

        if loop_controller.iteration() < CEILING_STREAK {
            return false;
        }

        let recent_scores: Vec<f64> = loop_controller
            .history()
            .iter()
            .rev()
            .take(CEILING_STREAK)
            .map(|r| r.score)
            .collect();

        let max_recent = recent_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (current_eval.score - max_recent).abs() < CEILING_THRESHOLD
    }

    /// Generate refinement prompt based on evaluation results
    fn generate_refinement_prompt(&self, eval_result: &EvalReport) -> String {
        let mut prompt = "Based on the evaluation results, please refine your approach:\n\n".to_string();

        // Add specific feedback based on failed criteria
        for criterion in &eval_result.criteria {
            if !criterion.passed {
                prompt.push_str(&format!(
                    "- {}: {}\n",
                    criterion.id,
                    criterion.notes.as_deref().unwrap_or("Needs improvement")
                ));
            }
        }

        prompt.push_str(&format!(
            "\nCurrent quality score: {:.2}%. Target: {:.2}%.\n",
            eval_result.score * 100.0,
            85.0 // From acceptance criteria
        ));

        prompt.push_str("Please improve the implementation to address these issues.");

        prompt
    }

    /// Create the final task result
    fn create_final_result(
        &self,
        final_eval: &EvalReport,
        loop_controller: &SelfPromptingLoop,
    ) -> Result<TaskResult, IntegrationError> {
        let sandbox = self.sandbox.try_read()?;

        Ok(TaskResult {
            task_id: loop_controller.task().id.clone(),
            success: final_eval.status == crate::evaluation::EvalStatus::Pass,
            iterations: loop_controller.iteration(),
            final_quality_score: final_eval.score,
            artifacts: sandbox.get_final_artifacts()?, // Get artifacts from sandbox
            evaluation_report: final_eval.clone(),
            execution_mode: self.execution_mode.clone(),
        })
    }
}

/// Errors that can occur during integration
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(#[from] crate::evaluation::EvaluationError),

    #[error("Sandbox error: {0}")]
    SandboxError(#[from] crate::sandbox::SandboxError),

    #[error("Agent error: {0}")]
    AgentError(String),

    #[error("Loop controller error: {0}")]
    LoopControllerError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_initialization() {
        // Test that the integrated agent can be created
        let model_registry = Arc::new(ModelRegistry::new());
        let evaluation_orchestrator = Arc::new(RwLock::new(EvaluationOrchestrator::new(Default::default())));

        let result = IntegratedAutonomousAgent::new(
            model_registry,
            evaluation_orchestrator,
            ExecutionMode::DryRun,
        ).await;

        assert!(result.is_ok(), "Failed to create integrated agent: {:?}", result.err());
    }
}
