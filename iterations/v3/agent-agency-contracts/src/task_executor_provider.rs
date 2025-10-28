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
        // Default factory - returns a basic implementation
        // In practice, this should be replaced with a real factory from agent-workers
        Self::new(|| {
            // For now, return a simple implementation that can be replaced
            // The actual factory should be provided by agent-workers::create_task_executor()
            Arc::new(BasicTaskExecutor::new())
        })
    }
}

/// Basic TaskExecutor implementation for default cases
/// This should be replaced with the real implementation from agent-workers
#[derive(Debug)]
struct BasicTaskExecutor;

impl BasicTaskExecutor {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TaskExecutor for BasicTaskExecutor {
    async fn execute_task(
        &self,
        task_spec: crate::task_executor::TaskSpec,
        worker_id: uuid::Uuid,
    ) -> Result<crate::task_executor::TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // Basic implementation - in practice this should delegate to agent-workers
        Ok(crate::task_executor::TaskExecutionResult {
            execution_id: uuid::Uuid::new_v4(),
            task_id: task_spec.id,
            success: true,
            output: format!("Task '{}' executed successfully (basic implementation)", task_spec.title),
            errors: vec![],
            metadata: std::collections::HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: chrono::Utc::now(),
            duration_ms: 100,
            worker_id: Some(worker_id),
        })
    }

    async fn execute_task_with_circuit_breaker(
        &self,
        task_spec: crate::task_executor::TaskSpec,
        worker_id: uuid::Uuid,
        _circuit_breaker_enabled: bool,
    ) -> Result<crate::task_executor::TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        self.execute_task(task_spec, worker_id).await
    }

    async fn health_check(&self) -> Result<crate::task_executor::TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>> {
        Ok(crate::task_executor::TaskExecutorHealth {
            status: crate::task_executor::HealthStatus::Healthy,
            last_execution_time: Some(chrono::Utc::now()),
            active_tasks: 0,
            queued_tasks: 0,
            total_executions: 0,
            success_rate: 1.0,
        })
    }

    async fn get_execution_stats(&self) -> Result<crate::task_executor::TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>> {
        Ok(crate::task_executor::TaskExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
            median_execution_time_ms: 0.0,
            p95_execution_time_ms: 0.0,
            p99_execution_time_ms: 0.0,
        })
    }

    async fn cancel_task_execution(&self, _task_id: uuid::Uuid, _worker_id: uuid::Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Basic implementation - just return success
        Ok(())
    }
}
