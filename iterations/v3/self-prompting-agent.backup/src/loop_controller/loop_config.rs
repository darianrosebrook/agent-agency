//! Configuration management for the self-prompting loop controller

use std::sync::Arc;
use crate::types::EvalStatus;
use crate::evaluation::{EvaluationOrchestrator, SatisficingEvaluator};
use crate::models::ModelRegistry;
use crate::prompting::PromptingStrategy;
use crate::stubs::{WorkspaceFactory, AllowList, Budgets};

/// Configuration for the self-prompting loop controller
#[derive(Debug, Clone)]
pub struct LoopControllerConfig {
    /// Maximum number of iterations to perform
    pub max_iterations: usize,
    /// Execution mode (strict/auto/dry-run)
    pub execution_mode: super::types::ExecutionMode,
    /// File operation allow-list
    pub allow_list: AllowList,
    /// Change budget constraints
    pub budgets: Budgets,
    /// Overload threshold for context monitoring (0.0-1.0)
    pub context_overload_threshold: f64,
    /// User approval callback for strict mode
    pub user_approval_callback: Option<Arc<dyn Fn(&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>>,
}

impl Default for LoopControllerConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            execution_mode: super::types::ExecutionMode::Auto,
            allow_list: vec![
                "src/**/*.rs".to_string(),
                "src/**/*.ts".to_string(),
                "tests/**/*.rs".to_string(),
            ],
            budgets: Budgets {
                max_files: 10,
                max_loc: 500,
            },
            context_overload_threshold: 0.8,
            user_approval_callback: None,
        }
    }
}

/// Builder for creating a LoopControllerConfig
pub struct LoopControllerConfigBuilder {
    config: LoopControllerConfig,
}

impl LoopControllerConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: LoopControllerConfig::default(),
        }
    }

    /// Set the maximum number of iterations
    pub fn max_iterations(mut self, max_iterations: usize) -> Self {
        self.config.max_iterations = max_iterations;
        self
    }

    /// Set the execution mode
    pub fn execution_mode(mut self, mode: super::types::ExecutionMode) -> Self {
        self.config.execution_mode = mode;
        self
    }

    /// Set the file operation allow-list
    pub fn allow_list(mut self, allow_list: AllowList) -> Self {
        self.config.allow_list = allow_list;
        self
    }

    /// Set the change budgets
    pub fn budgets(mut self, budgets: Budgets) -> Self {
        self.config.budgets = budgets;
        self
    }

    /// Set the context overload threshold
    pub fn context_overload_threshold(mut self, threshold: f64) -> Self {
        self.config.context_overload_threshold = threshold;
        self
    }

    /// Set the user approval callback
    pub fn user_approval_callback(
        mut self,
        callback: Arc<dyn Fn(&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>
    ) -> Self {
        self.config.user_approval_callback = Some(callback);
        self
    }

    /// Build the configuration
    pub fn build(self) -> LoopControllerConfig {
        self.config
    }
}

impl LoopControllerConfig {
    /// Create a builder for this configuration
    pub fn builder() -> LoopControllerConfigBuilder {
        LoopControllerConfigBuilder::new()
    }
}
