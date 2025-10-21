//! Integration layer connecting self-prompting agent components
//!
//! This module provides the glue that connects:
//! - Model providers → Loop controller → File operations → Evaluation
//!
//! It implements the autonomous workflow described in the theory.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::evaluation::EvaluationOrchestrator;
use crate::loop_controller::SelfPromptingLoop;
use crate::models::{ModelRegistry, OllamaProvider};
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
        evaluation_orchestrator: Arc<RwLock<SimpleEvaluationOrchestrator>>,
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

        // Configure loop controller with execution mode
        let loop_controller = SelfPromptingLoop::with_config(
            Arc::clone(&model_registry),
            Arc::clone(&evaluation_orchestrator),
            execution_mode.clone(),
            5, // max iterations
        );

        Ok(Self {
            loop_controller,
            sandbox: Arc::new(RwLock::new(sandbox)),
            execution_mode,
        })
    }

    /// Execute a task autonomously end-to-end
    pub async fn execute_task_autonomously(
        &self,
        task: Task,
    ) -> Result<TaskResult, IntegrationError> {
        // Use the existing SelfPromptingLoop.execute_task method
        let result = self.loop_controller.execute_task(task).await
            .map_err(|e| IntegrationError::LoopControllerError(e.to_string()))?;

        // Convert the SelfPromptingResult to our TaskResult format
        match result.task_result {
            crate::loop_controller::TaskResult::Completed(task_result) => {
                Ok(task_result)
            }
            crate::loop_controller::TaskResult::Failed(reason) => {
                Err(IntegrationError::AgentError(format!("Task failed: {}", reason)))
            }
        }
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
