//! Main self-prompting agent coordinator

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::self_prompting_agent::evaluation::EvaluationOrchestrator;
use crate::self_prompting_agent::learning_bridge::LearningBridge;
use crate::self_prompting_agent::loop_controller::{
    SelfPromptingEvent, SelfPromptingLoop, SelfPromptingResult,
};
use crate::self_prompting_agent::models::ModelRegistry;
use crate::self_prompting_agent::prompting_types::{
    AutonomousMode, SafetyMode, SelfPromptingAgentError, Task,
};
use crate::self_prompting_agent::rl_signals::RLTrainer;
use crate::self_prompting_agent::sandbox::SandboxEnvironment;

/// Configuration for the self-prompting agent

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfPromptingAgentConfig {
    pub max_iterations: usize,
    pub enable_sandbox: bool,
    pub sandbox_path: Option<String>,
    pub enable_git_snapshots: bool,
    pub execution_mode: AutonomousMode,
    pub safety_mode: SafetyMode,
    pub enable_learning: bool,
    pub enable_rl: bool,
}

impl Default for SelfPromptingAgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            enable_sandbox: true,
            sandbox_path: None,
            enable_git_snapshots: true,
            execution_mode: AutonomousMode::Auto,
            safety_mode: SafetyMode::Sandbox,
            enable_learning: true,
            enable_rl: false,
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
    learning_bridge: Option<Arc<LearningBridge>>,
    rl_trainer: Option<Arc<RLTrainer>>,
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
        let loop_controller = SelfPromptingLoop::new(config.max_iterations, event_tx.clone())
            .await
            .map_err(|e| SelfPromptingAgentError::Configuration(e.to_string()))?;

        // Create sandbox if enabled
        let sandbox = if config.enable_sandbox {
            Some(
                SandboxEnvironment::new(config.sandbox_path.clone())
                    .map_err(|e| SelfPromptingAgentError::Sandbox(e.to_string()))?,
            )
        } else {
            None
        };

        // Initialize learning bridge if enabled
        let learning_bridge = if config.enable_learning {
            Some(Arc::new(LearningBridge::new()))
        } else {
            None
        };

        // Initialize RL trainer if enabled
        let rl_trainer = if config.enable_rl {
            Some(Arc::new(RLTrainer::new(0.1, 0.9))) // learning_rate, discount_factor
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
            learning_bridge,
            rl_trainer,
        })
    }

    /// Execute a task with self-prompting
    pub async fn execute_task(
        &self,
        task: Task,
    ) -> Result<SelfPromptingResult, SelfPromptingAgentError> {
        info!(
            "Starting self-prompting execution for task: {}",
            task.description
        );

        // Validate task
        self.validate_task(&task).await?;

        // Get learning recommendations before execution if learning is enabled
        let mut task_with_recommendations = task;
        if let Some(ref learning_bridge) = self.learning_bridge {
            match learning_bridge
                .get_recommendations(&format!(
                    "{:?}_code_fixing",
                    task_with_recommendations.task_type
                ))
                .await
            {
                Ok(recommendations) => {
                    if !recommendations.is_empty() {
                        info!("Learning system recommendations: {:?}", recommendations);
                        // Add recommendations to task refinement_context
                        task_with_recommendations.refinement_context.extend(
                            recommendations
                                .iter()
                                .map(|r| format!("Learning insight: {}", r)),
                        );
                    }
                }
                Err(e) => warn!("Failed to get learning recommendations: {}", e),
            }
        }

        // Execute the self-prompting loop with learning bridge and RL trainer
        let result = self
            .loop_controller
            .execute_task(
                task_with_recommendations,
                self.model_registry.clone(),
                self.evaluator.clone(),
                self.learning_bridge.clone(),
                self.rl_trainer.clone(),
            )
            .await
            .map_err(|e| SelfPromptingAgentError::Execution(e.to_string()))?;

        info!(
            "Self-prompting execution completed with {} iterations",
            result.iterations
        );

        Ok(result)
    }

    /// Validate task before execution
    async fn validate_task(&self, task: &Task) -> Result<(), SelfPromptingAgentError> {
        if task.description.trim().is_empty() {
            return Err(SelfPromptingAgentError::Validation(
                "Task description cannot be empty".to_string(),
            ));
        }

        if task.description.len() > 10000 {
            return Err(SelfPromptingAgentError::Validation(
                "Task description too long".to_string(),
            ));
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
                "loop_controller": true,
                "learning_enabled": self.learning_bridge.is_some(),
                "rl_enabled": self.rl_trainer.is_some()
            }
        })
    }

    /// Get learning bridge reference
    pub fn learning_bridge(&self) -> Option<&Arc<LearningBridge>> {
        self.learning_bridge.as_ref()
    }

    /// Get RL trainer reference
    pub fn rl_trainer(&self) -> Option<&Arc<RLTrainer>> {
        self.rl_trainer.as_ref()
    }

    /// Shutdown the agent
    pub async fn shutdown(&self) -> Result<(), SelfPromptingAgentError> {
        info!("Shutting down self-prompting agent");

        if let Some(ref sandbox) = self.sandbox {
            sandbox
                .cleanup()
                .await
                .map_err(|e| SelfPromptingAgentError::Sandbox(e.to_string()))?;
        }

        self.loop_controller
            .shutdown()
            .await
            .map_err(|e| SelfPromptingAgentError::Execution(e.to_string()))?;

        Ok(())
    }
}
