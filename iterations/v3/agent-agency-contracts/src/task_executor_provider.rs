//! Task Executor Provider
//!
//! Provides a way to create TaskExecutor instances without circular dependencies.
//! This module acts as a bridge between orchestration and workers.

use std::sync::Arc;
use std::sync::OnceLock;

use crate::task_executor::TaskExecutor;

/// Factory function type for creating TaskExecutor instances
pub type TaskExecutorFactory = fn() -> Arc<dyn TaskExecutor>;

/// Global registry for default factory function
static DEFAULT_FACTORY: OnceLock<TaskExecutorFactory> = OnceLock::new();

/// Task executor provider that can be configured with a factory function
#[derive(Clone)]
pub struct TaskExecutorProvider {
    factory: TaskExecutorFactory,
}

impl TaskExecutorProvider {
    /// Create a new provider with a factory function
    pub fn new(factory: TaskExecutorFactory) -> Self {
        Self { factory }
    }

    /// Create a new TaskExecutor instance using the configured factory
    pub fn create_executor(&self) -> Arc<dyn TaskExecutor> {
        (self.factory)()
    }

    /// Set the default factory function (should be called from agent-workers)
    /// 
    /// # Example
    /// ```rust,ignore
    /// use agent_agency_contracts::task_executor_provider::TaskExecutorProvider;
    /// use agent_workers::task_executor_factory;
    /// 
    /// TaskExecutorProvider::set_default_factory(task_executor_factory());
    /// ```
    pub fn set_default_factory(factory: TaskExecutorFactory) -> Result<(), ()> {
        DEFAULT_FACTORY.set(factory).map_err(|_| ())
    }
}

impl Default for TaskExecutorProvider {
    fn default() -> Self {
        // Use registered default factory if available, otherwise require explicit configuration
        if let Some(factory) = DEFAULT_FACTORY.get() {
            Self::new(*factory)
        } else {
            // If no default factory is registered, this will panic when create_executor is called
            // This encourages explicit factory configuration via TaskExecutorProvider::new()
            Self {
                factory: || {
                    panic!(
                        "TaskExecutorProvider default factory not configured. \
                        Call TaskExecutorProvider::set_default_factory() or use \
                        TaskExecutorProvider::new() with a factory function from agent-workers."
                    )
                },
            }
        }
    }
}
