//! Main self-prompting agent coordinator

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};

use crate::evaluation::EvaluationOrchestrator;
use crate::loop_controller::{SelfPromptingLoop, SelfPromptingResult, SelfPromptingEvent};
use crate::models::ModelRegistry;
use crate::sandbox::SandboxEnvironment;
use crate::types::Task;

/// Configuration for the self-prompting agent
#[derive(Debug, Clone)]
pub struct SelfPromptingAgentConfig {
    pub max_iterations: usize,
    pub enable_sandbox: bool,
    pub sandbox_path: Option<String>,
    pub enable_git_snapshots: bool,
}

impl Default for SelfPromptingAgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            enable_sandbox: true,
            sandbox_path: None,
            enable_git_snapshots: true,
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
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Create loop controller
        let loop_controller = SelfPromptingLoop::with_config(
            model_registry.clone(),
            evaluator.clone(),
            config.max_iterations,
            Some(event_sender.clone()),
        );

        // Create sandbox if enabled
        let sandbox = if config.enable_sandbox {
            let sandbox_path = config.sandbox_path
                .clone()
                .unwrap_or_else(|| "./.agent-sandbox".to_string());

            Some(SandboxEnvironment::new(
                std::path::PathBuf::from(sandbox_path),
                // TODO: Implement path-based security sandboxing
                // - Define allowed path patterns and restrictions
                // - Implement path validation and sanitization
                // - Add configurable security policies per task
                // - Support path-based access control lists
                // - Implement path traversal attack prevention
                // - Add security audit logging for path access
                vec![], // PLACEHOLDER: Allowing all paths for now
                crate::sandbox::SafetyMode::Sandbox,
                config.enable_git_snapshots,
            ).await?)
        } else {
            None
        };

        // Start event handler
        tokio::spawn(Self::handle_events(event_receiver));

        Ok(Self {
            config,
            model_registry,
            evaluator,
            loop_controller,
            sandbox,
            event_sender: Some(event_sender),
        })
    }

    /// Execute a task using the self-prompting agent
    pub async fn execute_task(&self, task: Task) -> Result<SelfPromptingResult, SelfPromptingAgentError> {
        info!("Self-prompting agent executing task: {}", task.description);

        match &self.sandbox {
            Some(sandbox) => {
                // Execute with sandbox environment
                let mut sandbox_clone = sandbox.clone(); // Clone for mutability
                self.loop_controller.execute_with_sandbox(task, &mut sandbox_clone).await
                    .map_err(SelfPromptingAgentError::LoopError)
            }
            None => {
                // Execute without sandbox
                self.loop_controller.execute_task(task).await
                    .map_err(SelfPromptingAgentError::LoopError)
            }
        }
    }

    /// Get available models
    pub fn available_models(&self) -> Vec<String> {
        self.model_registry.list_providers()
    }

    /// Hot-swap a model
    pub async fn hot_swap_model(
        &mut self,
        id: &str,
        new_provider: Box<dyn crate::models::ModelProvider>,
    ) -> Result<(), SelfPromptingAgentError> {
        self.model_registry.hot_swap_provider(id, new_provider)
            .map_err(|e| SelfPromptingAgentError::ModelError(e.to_string()))?;

        info!("Hot-swapped model: {}", id);
        Ok(())
    }

    /// Register a new model provider
    pub async fn register_model(
        &mut self,
        id: String,
        provider: Box<dyn crate::models::ModelProvider>,
    ) -> Result<(), SelfPromptingAgentError> {
        self.model_registry.register_provider(id.clone(), provider)
            .map_err(|e| SelfPromptingAgentError::ModelError(e.to_string()))?;

        info!("Registered new model: {}", id);
        Ok(())
    }

    /// Get agent status
    pub fn status(&self) -> AgentStatus {
        AgentStatus {
            models_available: self.available_models().len(),
            sandbox_enabled: self.sandbox.is_some(),
            max_iterations: self.config.max_iterations,
        }
    }

    /// Handle events from the self-prompting loop
    async fn handle_events(mut receiver: mpsc::UnboundedReceiver<SelfPromptingEvent>) {
        while let Some(event) = receiver.recv().await {
            match event {
                SelfPromptingEvent::IterationStarted { task_id, iteration, .. } => {
                    info!("Task {}: Started iteration {}", task_id, iteration);
                }
                SelfPromptingEvent::EvaluationCompleted { task_id, iteration, score, status, .. } => {
                    info!("Task {}: Iteration {} completed with score {:.2} ({:?})",
                        task_id, iteration, score, status);
                }
                SelfPromptingEvent::ModelSwapped { task_id, old_model, new_model, reason, .. } => {
                    info!("Task {}: Swapped model from {} to {} ({})",
                        task_id, old_model, new_model, reason);
                }
                SelfPromptingEvent::LoopCompleted { task_id, total_iterations, final_score, stop_reason, .. } => {
                    info!("Task {}: Completed after {} iterations with final score {:.2} (reason: {:?})",
                        task_id, total_iterations, final_score, stop_reason);
                }
            }
        }
    }
}

/// Agent status information
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentStatus {
    pub models_available: usize,
    pub sandbox_enabled: bool,
    pub max_iterations: usize,
}

/// Errors from self-prompting agent operations
#[derive(Debug, thiserror::Error)]
pub enum SelfPromptingAgentError {
    #[error("Sandbox initialization failed: {0}")]
    SandboxError(#[from] crate::sandbox::SandboxError),

    #[error("Model registry error: {0}")]
    ModelError(String),

    #[error("Loop execution error: {0}")]
    LoopError(#[from] crate::loop_controller::SelfPromptingError),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
