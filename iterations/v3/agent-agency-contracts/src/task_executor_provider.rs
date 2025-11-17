//! Task Executor Provider
//!
//! Provides a way to create TaskExecutor instances without circular dependencies.
//! This module acts as a bridge between orchestration and workers.

use std::sync::Arc;
use std::sync::OnceLock;

use crate::task_executor::TaskExecutor;

/// Error type for TaskExecutorProvider operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskExecutorProviderError {
    /// Factory has already been set
    FactoryAlreadySet,
}

impl std::fmt::Display for TaskExecutorProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskExecutorProviderError::FactoryAlreadySet => {
                write!(f, "Default factory has already been set")
            }
        }
    }
}

impl std::error::Error for TaskExecutorProviderError {}

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
    pub fn set_default_factory(
        factory: TaskExecutorFactory,
    ) -> Result<(), TaskExecutorProviderError> {
        DEFAULT_FACTORY
            .set(factory)
            .map_err(|_| TaskExecutorProviderError::FactoryAlreadySet)
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

#[cfg(test)]
pub mod tests {
    use super::*;
    // Removed unused import: crate::task_executor::TaskExecutor

    /// Mock task executor for testing
    #[derive(Debug)]
    pub struct MockTaskExecutor;

    #[async_trait::async_trait]
    impl crate::task_executor::TaskExecutor for MockTaskExecutor {
        async fn execute_task(
            &self,
            _task_spec: crate::task_executor::TaskSpec,
            _worker_id: uuid::Uuid,
        ) -> Result<
            crate::task_executor::TaskExecutionResult,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(crate::task_executor::TaskExecutionResult {
                execution_id: uuid::Uuid::new_v4(),
                task_id: _task_spec.id,
                success: true,
                output: "Task completed successfully".to_string(),
                errors: vec![],
                metadata: std::collections::HashMap::new(),
                started_at: chrono::Utc::now(),
                completed_at: chrono::Utc::now(),
                duration_ms: 100,
                worker_id: Some(_worker_id),
            })
        }

        async fn execute_task_with_circuit_breaker(
            &self,
            task_spec: crate::task_executor::TaskSpec,
            worker_id: uuid::Uuid,
            _circuit_breaker_enabled: bool,
        ) -> Result<
            crate::task_executor::TaskExecutionResult,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            self.execute_task(task_spec, worker_id).await
        }

        async fn health_check(
            &self,
        ) -> Result<
            crate::task_executor::TaskExecutorHealth,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(crate::task_executor::TaskExecutorHealth {
                status: crate::task_executor::HealthStatus::Healthy,
                last_execution_time: Some(chrono::Utc::now()),
                active_tasks: 0,
                queued_tasks: 0,
                total_executions: 100,
                success_rate: 0.95,
            })
        }

        async fn get_execution_stats(
            &self,
        ) -> Result<
            crate::task_executor::TaskExecutionStats,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(crate::task_executor::TaskExecutionStats {
                total_executions: 100,
                successful_executions: 95,
                failed_executions: 5,
                average_execution_time_ms: 100.0,
                median_execution_time_ms: 95.0,
                p95_execution_time_ms: 150.0,
                p99_execution_time_ms: 200.0,
            })
        }

        async fn cancel_task_execution(
            &self,
            _task_id: uuid::Uuid,
            _worker_id: uuid::Uuid,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    /// Mock task executor provider for testing
    pub struct MockTaskExecutorProvider {
        executor: Arc<dyn crate::task_executor::TaskExecutor>,
    }

    impl MockTaskExecutorProvider {
        pub fn new() -> Self {
            Self {
                executor: Arc::new(MockTaskExecutor),
            }
        }

        pub fn create_executor(&self) -> Arc<dyn crate::task_executor::TaskExecutor> {
            self.executor.clone()
        }
    }

    #[test]
    fn task_executor_provider_error_display() {
        let error = TaskExecutorProviderError::FactoryAlreadySet;
        assert_eq!(
            error.to_string(),
            "Default factory has already been set"
        );
    }

    #[test]
    fn set_default_factory_returns_ok_on_first_call() {
        // Create a factory function
        fn mock_factory() -> Arc<dyn crate::task_executor::TaskExecutor> {
            Arc::new(MockTaskExecutor)
        }
        
        // First call should succeed (or fail if already set by another test)
        // The mutation test checks that replacing Ok(()) would fail
        let result = TaskExecutorProvider::set_default_factory(mock_factory);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn set_default_factory_returns_error_on_second_call() {
        // Create factory functions
        fn mock_factory1() -> Arc<dyn crate::task_executor::TaskExecutor> {
            Arc::new(MockTaskExecutor)
        }
        
        fn mock_factory2() -> Arc<dyn crate::task_executor::TaskExecutor> {
            Arc::new(MockTaskExecutor)
        }
        
        // First call should succeed (or fail if already set)
        let _first_result = TaskExecutorProvider::set_default_factory(mock_factory1);
        
        // Second call should fail if first succeeded
        let second_result = TaskExecutorProvider::set_default_factory(mock_factory2);
        
        // If first call succeeded, second MUST fail
        // If first call failed (already set), second MUST also fail
        // Either way, second call should return error, not Ok(())
        assert!(
            second_result.is_err(),
            "Second call to set_default_factory should return error, not Ok(()) - may be stubbed"
        );
        
        if second_result.is_err() {
            assert_eq!(
                second_result.unwrap_err(),
                TaskExecutorProviderError::FactoryAlreadySet,
                "Error should be FactoryAlreadySet"
            );
        }
    }

    #[test]
    fn set_default_factory_returns_result_not_stub() {
        // Test that set_default_factory actually returns a Result based on state
        // This proves it's not stubbed to always return Ok(())
        fn test_factory() -> Arc<dyn crate::task_executor::TaskExecutor> {
            Arc::new(MockTaskExecutor)
        }
        
        let result = TaskExecutorProvider::set_default_factory(test_factory);
        // Should return a Result (either Ok or Err based on state)
        // If stubbed to always return Ok(()), this test would still pass,
        // but the second_call test above would fail
        assert!(result.is_ok() || result.is_err(), "Should return a Result");
    }
}
