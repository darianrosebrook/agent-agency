//! Task Executor Provider
//!
//! Provides a way to create TaskExecutor instances without circular dependencies.
//! This module acts as a bridge between orchestration and workers.

use async_trait::async_trait;
use std::sync::Arc;

use crate::task_executor::TaskExecutor;

/// Factory function type for creating TaskExecutor instances
pub type TaskExecutorFactory = fn() -> Arc<dyn TaskExecutor>;

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
}

impl Default for TaskExecutorProvider {
    fn default() -> Self {
        // PLACEHOLDER: Real factory implementation needed from agent-workers
        // This should be replaced with a proper factory that creates real TaskExecutor instances
        panic!("TaskExecutorProvider default factory not implemented - must be configured with a real factory");
    }
}
