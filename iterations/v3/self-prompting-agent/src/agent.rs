//! Main self-prompting agent coordinator

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

use crate::evaluation::EvaluationOrchestrator;
use crate::loop_controller::{SelfPromptingLoop, SelfPromptingResult, SelfPromptingEvent};
use crate::models::ModelRegistry;
use crate::sandbox::SandboxEnvironment;
use crate::types::{Task, SelfPromptingAgentError, ExecutionMode, SafetyMode};

/// Configuration for the self-prompting agent
#[derive(Debug, Clone)]
pub struct SelfPromptingAgentConfig {
    pub max_iterations: usize,
    pub enable_sandbox: bool,
    pub sandbox_path: Option<String>,
    pub enable_git_snapshots: bool,
    pub execution_mode: ExecutionMode,
    pub safety_mode: SafetyMode,
}

impl Default for SelfPromptingAgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            enable_sandbox: true,
            sandbox_path: None,
            enable_git_snapshots: true,
            execution_mode: ExecutionMode::Auto,
            safety_mode: SafetyMode::Sandbox,
        }
    }
}

/// Main self-prompting agent coordinator
pub struct SelfPromptingAgent {
    config: SelfPromptingAgentConfig,
    model_registry: Arc<ModelRegistry>,
    evaluator: Arc<EvaluationOrchestrator>,
    loop_controller: SelfPromptingLoop,
    sandbox: Option<SandboxEnvironment>,
    event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
}

impl SelfPromptingAgent {
    /// Create a new self-prompting agent
    pub async fn new(
        config: SelfPromptingAgentConfig,
        model_registry: Arc<ModelRegistry>,
        evaluator: Arc<EvaluationOrchestrator>,
    ) -> Result<Self, SelfPromptingAgentError> {
        // Create event channel
        let (event_tx, _event_rx) = mpsc::unbounded_channel();

        // Create loop controller
        let loop_controller = SelfPromptingLoop::new(
            config.max_iterations,
            event_tx.clone(),
        ).await.map_err(|e| SelfPromptingAgentError::Configuration(e.to_string()))?;

        // Create sandbox if enabled
        let sandbox = if config.enable_sandbox {
            Some(SandboxEnvironment::new(config.sandbox_path.clone())
                .map_err(|e| SelfPromptingAgentError::Sandbox(e.to_string()))?)
        } else {
            None
        };

        Ok(Self {
            config,
            model_registry,
            evaluator,
            loop_controller,
            sandbox,
            event_sender: Some(event_tx),
        })
    }

    /// Execute a task with self-prompting
    pub async fn execute_task(&self, task: Task) -> Result<SelfPromptingResult, SelfPromptingAgentError> {
        info!("Starting self-prompting execution for task: {}", task.description);

        // Validate task
        self.validate_task(&task).await?;

        // Execute the self-prompting loop
        let result = self.loop_controller.execute_task(task, self.model_registry.clone(), self.evaluator.clone()).await
            .map_err(|e| SelfPromptingAgentError::Execution(e.to_string()))?;

        info!("Self-prompting execution completed with {} iterations", result.iterations);

        Ok(result)
    }

    /// Validate task before execution
    async fn validate_task(&self, task: &Task) -> Result<(), SelfPromptingAgentError> {
        if task.description.trim().is_empty() {
            return Err(SelfPromptingAgentError::Validation("Task description cannot be empty".to_string()));
        }

        if task.description.len() > 10000 {
            return Err(SelfPromptingAgentError::Validation("Task description too long".to_string()));
        }

        // Additional validation can be added here

        Ok(())
    }

    /// Get agent status
    pub async fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "operational",
            "config": {
                "max_iterations": self.config.max_iterations,
                "execution_mode": format!("{:?}", self.config.execution_mode),
                "safety_mode": format!("{:?}", self.config.safety_mode),
                "sandbox_enabled": self.config.enable_sandbox,
                "git_snapshots": self.config.enable_git_snapshots
            },
            "capabilities": {
                "model_providers": true,
                "evaluation_framework": true,
                "sandbox_environment": self.sandbox.is_some(),
                "loop_controller": true
            }
        })
    }

    /// Shutdown the agent
    pub async fn shutdown(&self) -> Result<(), SelfPromptingAgentError> {
        info!("Shutting down self-prompting agent");

        if let Some(ref sandbox) = self.sandbox {
            sandbox.cleanup().await
                .map_err(|e| SelfPromptingAgentError::Sandbox(e.to_string()))?;
        }

        self.loop_controller.shutdown().await
            .map_err(|e| SelfPromptingAgentError::Execution(e.to_string()))?;

        Ok(())
    }
}
