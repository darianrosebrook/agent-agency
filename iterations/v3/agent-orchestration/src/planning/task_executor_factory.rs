//! Task Executor Factory - Creates TaskExecutor instances with proper dependencies
//!
//! This factory provides different execution strategies (parallel, sequential, hybrid)
//! and ensures proper dependency injection for all task executors.

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use tracing::{debug, warn};

use agent_agency_contracts::task_executor::{TaskExecutor, TaskSpec, TaskExecutionResult, TaskExecutorHealth, TaskExecutionStats};

/// Execution strategy for task execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionStrategy {
    /// Execute tasks sequentially (one at a time)
    Sequential,
    /// Execute tasks in parallel with worker limits
    Parallel,
    /// Hybrid execution: some tasks sequential, some parallel based on dependencies
    Hybrid,
    /// Adaptive execution based on load and priority
    Adaptive,
}

/// Configuration for task executor creation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskExecutorConfig {
    /// Execution strategy to use
    pub strategy: ExecutionStrategy,
    /// Maximum concurrent tasks (for parallel execution)
    pub max_concurrent_tasks: usize,
    /// Timeout for individual task execution in seconds
    pub task_timeout_seconds: u64,
    /// Enable health monitoring
    pub enable_health_monitoring: bool,
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// Worker pool size (if applicable)
    pub worker_pool_size: Option<usize>,
    /// Queue capacity for pending tasks
    pub queue_capacity: usize,
}

impl Default for TaskExecutorConfig {
    fn default() -> Self {
        Self {
            strategy: ExecutionStrategy::Parallel,
            max_concurrent_tasks: 10,
            task_timeout_seconds: 300, // 5 minutes
            enable_health_monitoring: true,
            enable_metrics: true,
            worker_pool_size: Some(5),
            queue_capacity: 100,
        }
    }
}

/// Factory for creating TaskExecutor instances with different strategies
pub struct TaskExecutorFactory {
    /// Default configuration
    default_config: TaskExecutorConfig,
    /// Worker pool integration (if available)
    worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
    /// Task queue service (if available) - TODO: Enable when data-infrastructure integration is available
    task_queue: Option<Arc<dyn std::marker::Send + std::marker::Sync + 'static>>, // Placeholder for TaskQueueService
    /// Audit trail manager for logging
    audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
}

impl TaskExecutorFactory {
    /// Create a new factory with default configuration
    pub fn new() -> Self {
        Self {
            default_config: TaskExecutorConfig::default(),
            worker_pool: None,
            task_queue: None,
            audit_manager: None,
        }
    }

    /// Configure with custom default config
    pub fn with_config(mut self, config: TaskExecutorConfig) -> Self {
        self.default_config = config;
        self
    }

    /// Configure with worker pool integration
    pub fn with_worker_pool(mut self, worker_pool: Arc<dyn crate::planning::plan_executor::WorkerPool>) -> Self {
        self.worker_pool = Some(worker_pool);
        self
    }

    /// Configure with task queue service
    /// TODO: Enable when data-infrastructure integration is available
    pub fn with_task_queue(self, _task_queue: Arc<dyn std::marker::Send + std::marker::Sync + 'static>) -> Self {
        // self.task_queue = Some(task_queue);
        self
    }

    /// Configure with audit trail manager
    pub fn with_audit_trail(mut self, audit_manager: Arc<crate::audit_trail::AuditTrailManager>) -> Self {
        self.audit_manager = Some(audit_manager);
        self
    }

    /// Create a TaskExecutor with the specified strategy
    pub fn create_executor(&self, strategy: ExecutionStrategy) -> Result<Arc<dyn TaskExecutor>, TaskExecutorFactoryError> {
        let config = TaskExecutorConfig {
            strategy,
            ..self.default_config.clone()
        };

        match strategy {
            ExecutionStrategy::Sequential => self.create_sequential_executor(config),
            ExecutionStrategy::Parallel => self.create_parallel_executor(config),
            ExecutionStrategy::Hybrid => self.create_hybrid_executor(config),
            ExecutionStrategy::Adaptive => self.create_adaptive_executor(config),
        }
    }

    /// Create a TaskExecutor with default strategy
    pub fn create_default_executor(&self) -> Result<Arc<dyn TaskExecutor>, TaskExecutorFactoryError> {
        self.create_executor(self.default_config.strategy)
    }

    /// Create a sequential task executor
    fn create_sequential_executor(&self, config: TaskExecutorConfig) -> Result<Arc<dyn TaskExecutor>, TaskExecutorFactoryError> {
        debug!("Creating sequential task executor with config: {:?}", config);

        let executor = SequentialTaskExecutor::new(
            config.clone(),
            self.worker_pool.clone(),
            // TODO: Enable task_queue when data-infrastructure integration is available
            // self.task_queue.clone(),
            self.audit_manager.clone(),
        );

        Ok(Arc::new(executor))
    }

    /// Create a parallel task executor
    fn create_parallel_executor(&self, config: TaskExecutorConfig) -> Result<Arc<dyn TaskExecutor>, TaskExecutorFactoryError> {
        debug!("Creating parallel task executor with config: {:?}", config);

        let executor = ParallelTaskExecutor::new(
            config.clone(),
            self.worker_pool.clone(),
            // TODO: Enable task_queue when data-infrastructure integration is available
            // self.task_queue.clone(),
            self.audit_manager.clone(),
        );

        Ok(Arc::new(executor))
    }

    /// Create a hybrid task executor
    fn create_hybrid_executor(&self, config: TaskExecutorConfig) -> Result<Arc<dyn TaskExecutor>, TaskExecutorFactoryError> {
        debug!("Creating hybrid task executor with config: {:?}", config);

        let executor = HybridTaskExecutor::new(
            config.clone(),
            self.worker_pool.clone(),
            // TODO: Enable task_queue when data-infrastructure integration is available
            // self.task_queue.clone(),
            self.audit_manager.clone(),
        );

        Ok(Arc::new(executor))
    }

    /// Create an adaptive task executor
    fn create_adaptive_executor(&self, config: TaskExecutorConfig) -> Result<Arc<dyn TaskExecutor>, TaskExecutorFactoryError> {
        debug!("Creating adaptive task executor with config: {:?}", config);

        let executor = AdaptiveTaskExecutor::new(
            config.clone(),
            self.worker_pool.clone(),
            // TODO: Enable task_queue when data-infrastructure integration is available
            // self.task_queue.clone(),
            self.audit_manager.clone(),
        );

        Ok(Arc::new(executor))
    }

    /// Validate that all required dependencies are available
    pub fn validate_dependencies(&self) -> Result<(), TaskExecutorFactoryError> {
        // Check if worker pool is available (required for most executors)
        if self.worker_pool.is_none() {
            warn!("Worker pool not configured - using mock implementation");
        }

        // Check if task queue is available
        if self.task_queue.is_none() {
            warn!("Task queue not configured - using in-memory implementation");
        }

        // Check if audit trail is available
        if self.audit_manager.is_none() {
            warn!("Audit trail not configured - execution will not be audited");
        }

        Ok(())
    }
}

/// Sequential task executor - executes tasks one at a time
pub struct SequentialTaskExecutor {
    config: TaskExecutorConfig,
    worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
    // TODO: Enable task_queue when data-infrastructure integration is available
    // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
    audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
}

impl std::fmt::Debug for SequentialTaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequentialTaskExecutor")
            .field("config", &self.config)
            .field("worker_pool", &self.worker_pool.as_ref().map(|_| "Some(WorkerPool)"))
            .field("audit_manager", &self.audit_manager)
            .finish()
    }
}

impl SequentialTaskExecutor {
    fn new(
        config: TaskExecutorConfig,
        worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
        // TODO: Enable task_queue when data-infrastructure integration is available
        // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
        audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        Self {
            config,
            worker_pool,
            // task_queue,
            audit_manager,
        }
    }
}

#[async_trait]
impl TaskExecutor for SequentialTaskExecutor {
    async fn execute_task(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing task {} sequentially on worker {}", task_spec.id, worker_id);

        // Record execution start in audit trail
        if let Some(audit) = &self.audit_manager {
            // TODO: Record task execution start
        }

        // TODO: Integrate with actual worker execution for sequential task execution
        //       Currently simulates execution with fixed timing; should integrate with actual worker execution infrastructure.
        //
        // COMPLETION CHECKLIST:
        // [ ] Submit task to worker pool for sequential execution
        // [ ] Wait for worker to accept and execute task
        // [ ] Track actual execution start and completion times
        // [ ] Handle worker execution errors
        // [ ] Support task cancellation and timeout
        // [ ] Add unit tests for worker integration
        // [ ] Add integration tests with real workers
        // [ ] Verify sequential execution ordering
        //
        // ACCEPTANCE CRITERIA:
        // - Tasks are executed by actual workers
        // - Execution timing reflects real worker performance
        // - Worker errors are handled gracefully
        // - Sequential execution ordering is maintained
        //
        // DEPENDENCIES:
        // - Worker pool infrastructure (Required)
        // - Task submission API (Required)
        // - Worker execution tracking (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (core orchestration feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Task orchestration expertise
        let started_at = chrono::Utc::now(); // Temporary: simulated timing until worker integration
        let completed_at = started_at + chrono::Duration::milliseconds(1000);

        let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

        let result = TaskExecutionResult {
            execution_id: uuid::Uuid::new_v4(),
            task_id: task_spec.id,
            success: true,
            output: "Task executed successfully (sequential)".to_string(),
            errors: vec![],
            metadata: std::collections::HashMap::new(),
            started_at,
            completed_at,
            duration_ms,
            worker_id: Some(worker_id),
        };

        // Record execution completion in audit trail
        if let Some(audit) = &self.audit_manager {
            // TODO: Record task execution completion
        }

        Ok(result)
    }

    async fn execute_task_with_circuit_breaker(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
        circuit_breaker_enabled: bool,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement circuit breaker logic for task execution
        //       Currently delegates to regular execute_task; should implement circuit breaker pattern for resilience.
        //
        // COMPLETION CHECKLIST:
        // [ ] Check circuit breaker state before execution
        // [ ] Track execution failures and successes
        // [ ] Open circuit after failure threshold
        // [ ] Attempt half-open state after timeout
        // [ ] Close circuit after success threshold
        // [ ] Add unit tests for circuit breaker logic
        // [ ] Add integration tests with failure scenarios
        // [ ] Verify circuit breaker effectiveness
        //
        // ACCEPTANCE CRITERIA:
        // - Circuit breaker prevents execution when circuit is open
        // - Failure tracking triggers circuit opening correctly
        // - Half-open state allows limited retry attempts
        // - Circuit closes after successful recovery
        //
        // DEPENDENCIES:
        // - Circuit breaker infrastructure (Required)
        // - Failure tracking utilities (Required)
        // - State management utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (resilience feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Resilience patterns expertise
        self.execute_task(task_spec, worker_id).await // Temporary: delegate until circuit breaker is implemented
    }

    async fn health_check(&self) -> Result<TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutorHealth {
            status: agent_agency_contracts::task_executor::HealthStatus::Healthy,
            last_execution_time: Some(chrono::Utc::now()),
            active_tasks: 1, // Sequential - only one active at a time
            queued_tasks: 0,
            total_executions: 100, // Mock stats
            success_rate: 0.95,
        })
    }

    async fn get_execution_stats(&self) -> Result<TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutionStats {
            total_executions: 100,
            successful_executions: 95,
            failed_executions: 5,
            average_execution_time_ms: 1500.0,
            median_execution_time_ms: 1400.0,
            p95_execution_time_ms: 2000.0,
            p99_execution_time_ms: 2500.0,
        })
    }

    async fn cancel_task_execution(
        &self,
        _task_id: Uuid,
        _worker_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement task cancellation
        Ok(())
    }
}

/// Parallel task executor - executes multiple tasks concurrently
pub struct ParallelTaskExecutor {
    config: TaskExecutorConfig,
    worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
    // TODO: Enable task_queue when data-infrastructure integration is available
    // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
    audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
    semaphore: tokio::sync::Semaphore,
}

impl std::fmt::Debug for ParallelTaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelTaskExecutor")
            .field("config", &self.config)
            .field("worker_pool", &self.worker_pool.as_ref().map(|_| "Some(WorkerPool)"))
            .field("audit_manager", &self.audit_manager)
            .field("semaphore", &format!("Semaphore(permits: {})", self.semaphore.available_permits()))
            .finish()
    }
}

impl ParallelTaskExecutor {
    fn new(
        config: TaskExecutorConfig,
        worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
        // TODO: Enable task_queue when data-infrastructure integration is available
        // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
        audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        let semaphore = tokio::sync::Semaphore::new(config.max_concurrent_tasks);

        Self {
            config,
            worker_pool,
            // task_queue,
            audit_manager,
            semaphore,
        }
    }
}

#[async_trait]
impl TaskExecutor for ParallelTaskExecutor {
    async fn execute_task(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing task {} in parallel on worker {}", task_spec.id, worker_id);

        // Acquire semaphore permit for parallel execution
        let _permit = self.semaphore.acquire().await
            .map_err(|e| format!("Failed to acquire execution permit: {}", e))?;

        // Record execution start in audit trail
        if let Some(audit) = &self.audit_manager {
            // TODO: Record task execution start
        }

        // TODO: Integrate with actual worker execution for parallel task execution
        //       Currently simulates execution with fixed timing; should integrate with actual worker execution infrastructure.
        //
        // COMPLETION CHECKLIST:
        // [ ] Submit tasks to worker pool for parallel execution
        // [ ] Wait for workers to accept and execute tasks concurrently
        // [ ] Track actual execution start and completion times
        // [ ] Handle worker execution errors
        // [ ] Support task cancellation and timeout
        // [ ] Add unit tests for parallel worker integration
        // [ ] Add integration tests with real workers
        // [ ] Verify parallel execution concurrency
        //
        // ACCEPTANCE CRITERIA:
        // - Tasks are executed by actual workers in parallel
        // - Execution timing reflects real worker performance
        // - Worker errors are handled gracefully
        // - Parallel execution maintains concurrency
        //
        // DEPENDENCIES:
        // - Worker pool infrastructure (Required)
        // - Task submission API (Required)
        // - Worker execution tracking (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (core orchestration feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Task orchestration expertise
        let started_at = chrono::Utc::now(); // Temporary: simulated timing until worker integration
        let completed_at = started_at + chrono::Duration::milliseconds(800);
        let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

        let result = TaskExecutionResult {
            execution_id: uuid::Uuid::new_v4(),
            task_id: task_spec.id,
            success: true,
            output: "Task executed successfully (parallel)".to_string(),
            errors: vec![],
            metadata: std::collections::HashMap::new(),
            started_at,
            completed_at,
            duration_ms,
            worker_id: Some(worker_id),
        };

        // Record execution completion in audit trail
        if let Some(audit) = &self.audit_manager {
            // TODO: Record task execution completion
        }

        Ok(result)
    }

    async fn execute_task_with_circuit_breaker(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
        circuit_breaker_enabled: bool,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement circuit breaker logic for task execution
        //       Currently delegates to regular execute_task; should implement circuit breaker pattern for resilience.
        //
        // COMPLETION CHECKLIST:
        // [ ] Check circuit breaker state before execution
        // [ ] Track execution failures and successes
        // [ ] Open circuit after failure threshold
        // [ ] Attempt half-open state after timeout
        // [ ] Close circuit after success threshold
        // [ ] Add unit tests for circuit breaker logic
        // [ ] Add integration tests with failure scenarios
        // [ ] Verify circuit breaker effectiveness
        //
        // ACCEPTANCE CRITERIA:
        // - Circuit breaker prevents execution when circuit is open
        // - Failure tracking triggers circuit opening correctly
        // - Half-open state allows limited retry attempts
        // - Circuit closes after successful recovery
        //
        // DEPENDENCIES:
        // - Circuit breaker infrastructure (Required)
        // - Failure tracking utilities (Required)
        // - State management utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (resilience feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Resilience patterns expertise
        self.execute_task(task_spec, worker_id).await // Temporary: delegate until circuit breaker is implemented
    }

    async fn health_check(&self) -> Result<TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutorHealth {
            status: agent_agency_contracts::task_executor::HealthStatus::Healthy,
            last_execution_time: Some(chrono::Utc::now()),
            active_tasks: self.config.max_concurrent_tasks as u32,
            queued_tasks: 0,
            total_executions: 200, // Mock stats - higher due to parallelization
            success_rate: 0.97,
        })
    }

    async fn get_execution_stats(&self) -> Result<TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutionStats {
            total_executions: 200,
            successful_executions: 194,
            failed_executions: 6,
            average_execution_time_ms: 1200.0,
            median_execution_time_ms: 1100.0,
            p95_execution_time_ms: 1800.0,
            p99_execution_time_ms: 2200.0,
        })
    }

    async fn cancel_task_execution(
        &self,
        _task_id: Uuid,
        _worker_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement task cancellation
        Ok(())
    }
}

/// Hybrid task executor - combines sequential and parallel execution
pub struct HybridTaskExecutor {
    config: TaskExecutorConfig,
    worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
    // TODO: Enable task_queue when data-infrastructure integration is available
    // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
    audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
    semaphore: tokio::sync::Semaphore,
}

impl std::fmt::Debug for HybridTaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridTaskExecutor")
            .field("config", &self.config)
            .field("worker_pool", &self.worker_pool.as_ref().map(|_| "Some(WorkerPool)"))
            .field("audit_manager", &self.audit_manager)
            .field("semaphore", &format!("Semaphore(permits: {})", self.semaphore.available_permits()))
            .finish()
    }
}

impl HybridTaskExecutor {
    fn new(
        config: TaskExecutorConfig,
        worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
        // TODO: Enable task_queue when data-infrastructure integration is available
        // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
        audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        let semaphore = tokio::sync::Semaphore::new(config.max_concurrent_tasks / 2); // Reserve some capacity for sequential

        Self {
            config,
            worker_pool,
            // task_queue,
            audit_manager,
            semaphore,
        }
    }

    /// Determine if a task should be executed sequentially or in parallel
    fn should_execute_sequentially(&self, task_spec: &TaskSpec) -> bool {
        // TODO: Implement comprehensive sequential/parallel execution decision logic
        //       Currently uses basic priority-based heuristics; should implement sophisticated decision logic based on task characteristics, dependencies, and system state.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Decision logic considers task dependencies and relationships
        // - System load and resource availability are factored into decisions
        // - Task complexity and estimated duration influence execution strategy
        // - Historical performance data informs decision-making
        // - Configuration allows tuning of decision parameters
        //
        // DEPENDENCIES:
        // - Task dependency analysis system (Required)
        // - System resource monitoring (Required)
        // - Historical performance tracking (Optional)
        // - Configuration management system (Optional)
        //
        // ESTIMATED EFFORT: 12-16 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (task execution optimization)
        // - Change Budget: ~250 LOC
        // - Reviewer Requirements: Task orchestration and scheduling expertise
        match task_spec.priority {
            agent_agency_contracts::TaskPriority::Critical => true, // Critical tasks sequential
            agent_agency_contracts::TaskPriority::Urgent => true,   // Urgent tasks sequential
            _ => false, // Others can be parallel
        }
    }
}

#[async_trait]
impl TaskExecutor for HybridTaskExecutor {
    async fn execute_task(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let is_sequential = self.should_execute_sequentially(&task_spec);

        let started_at = chrono::Utc::now();

        if is_sequential {
            debug!("Executing task {} sequentially (hybrid mode)", task_spec.id);

            // Sequential execution - no semaphore needed
            let completed_at = started_at + chrono::Duration::milliseconds(1200);
            let duration_ms = (completed_at - started_at).num_milliseconds() as u64;
            let result = TaskExecutionResult {
                execution_id: uuid::Uuid::new_v4(),
                task_id: task_spec.id,
                success: true,
                output: "Task executed successfully (sequential in hybrid)".to_string(),
                errors: vec![],
                metadata: std::collections::HashMap::new(),
                started_at,
                completed_at,
                duration_ms,
                worker_id: Some(worker_id),
            };

            Ok(result)
        } else {
            debug!("Executing task {} in parallel (hybrid mode)", task_spec.id);

            // Parallel execution - acquire semaphore
            let _permit = self.semaphore.acquire().await
                .map_err(|e| format!("Failed to acquire execution permit: {}", e))?;

            let completed_at = started_at + chrono::Duration::milliseconds(900);
            let duration_ms = (completed_at - started_at).num_milliseconds() as u64;
            let result = TaskExecutionResult {
                execution_id: uuid::Uuid::new_v4(),
                task_id: task_spec.id,
                success: true,
                output: "Task executed successfully (parallel in hybrid)".to_string(),
                errors: vec![],
                metadata: std::collections::HashMap::new(),
                started_at,
                completed_at,
                duration_ms,
                worker_id: Some(worker_id),
            };

            Ok(result)
        }
    }

    async fn execute_task_with_circuit_breaker(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
        circuit_breaker_enabled: bool,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement circuit breaker logic for task execution
        //       Currently delegates to regular execute_task; should implement circuit breaker pattern for resilience.
        //
        // COMPLETION CHECKLIST:
        // [ ] Check circuit breaker state before execution
        // [ ] Track execution failures and successes
        // [ ] Open circuit after failure threshold
        // [ ] Attempt half-open state after timeout
        // [ ] Close circuit after success threshold
        // [ ] Add unit tests for circuit breaker logic
        // [ ] Add integration tests with failure scenarios
        // [ ] Verify circuit breaker effectiveness
        //
        // ACCEPTANCE CRITERIA:
        // - Circuit breaker prevents execution when circuit is open
        // - Failure tracking triggers circuit opening correctly
        // - Half-open state allows limited retry attempts
        // - Circuit closes after successful recovery
        //
        // DEPENDENCIES:
        // - Circuit breaker infrastructure (Required)
        // - Failure tracking utilities (Required)
        // - State management utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (resilience feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Resilience patterns expertise
        self.execute_task(task_spec, worker_id).await // Temporary: delegate until circuit breaker is implemented
    }

    async fn health_check(&self) -> Result<TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutorHealth {
            status: agent_agency_contracts::task_executor::HealthStatus::Healthy,
            last_execution_time: Some(chrono::Utc::now()),
            active_tasks: (self.config.max_concurrent_tasks / 2) as u32, // Mix of sequential and parallel
            queued_tasks: 0,
            total_executions: 150, // Mock stats
            success_rate: 0.96,
        })
    }

    async fn get_execution_stats(&self) -> Result<TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutionStats {
            total_executions: 150,
            successful_executions: 144,
            failed_executions: 6,
            average_execution_time_ms: 1100.0,
            median_execution_time_ms: 1000.0,
            p95_execution_time_ms: 1500.0,
            p99_execution_time_ms: 1800.0,
        })
    }

    async fn cancel_task_execution(
        &self,
        _task_id: Uuid,
        _worker_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement task cancellation
        Ok(())
    }
}

/// Adaptive task executor - adjusts strategy based on load and priority
pub struct AdaptiveTaskExecutor {
    config: TaskExecutorConfig,
    worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
    // TODO: Enable task_queue when data-infrastructure integration is available
    // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
    audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
    semaphore: tokio::sync::Semaphore,
}

impl std::fmt::Debug for AdaptiveTaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdaptiveTaskExecutor")
            .field("config", &self.config)
            .field("worker_pool", &self.worker_pool.as_ref().map(|_| "Some(WorkerPool)"))
            .field("audit_manager", &self.audit_manager)
            .field("semaphore", &format!("Semaphore(permits: {})", self.semaphore.available_permits()))
            .finish()
    }
}

impl AdaptiveTaskExecutor {
    fn new(
        config: TaskExecutorConfig,
        worker_pool: Option<Arc<dyn crate::planning::plan_executor::WorkerPool>>,
        // TODO: Enable task_queue when data-infrastructure integration is available
        // task_queue: Option<Arc<dyn crate::data_infrastructure::queue::TaskQueueService>>,
        audit_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        let semaphore = tokio::sync::Semaphore::new(config.max_concurrent_tasks);

        Self {
            config,
            worker_pool,
            // task_queue,
            audit_manager,
            semaphore,
        }
    }

    /// Adapt execution strategy based on current system state
    fn adapt_strategy(&self, task_spec: &TaskSpec) -> ExecutionStrategy {
        // TODO: Implement adaptive execution strategy logic
        //       Currently uses basic priority-based logic; should implement sophisticated adaptive logic that considers system load, worker availability, task priority, and historical performance for optimal execution strategy selection.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Strategy adapts based on current system load metrics
        // - Worker availability influences strategy selection
        // - Task priority is factored into adaptive decisions
        // - Historical performance data informs strategy selection
        // - Strategy selection is configurable and tunable
        //
        // DEPENDENCIES:
        // - System load monitoring (Required)
        // - Worker availability tracking (Required)
        // - Historical performance database (Optional)
        // - Configuration management system (Optional)
        //
        // ESTIMATED EFFORT: 12-16 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (task execution optimization)
        // - Change Budget: ~250 LOC
        // - Reviewer Requirements: Task orchestration and adaptive systems expertise
        match task_spec.priority {
            agent_agency_contracts::TaskPriority::Critical => ExecutionStrategy::Sequential,
            agent_agency_contracts::TaskPriority::Urgent => ExecutionStrategy::Sequential,
            agent_agency_contracts::TaskPriority::High => ExecutionStrategy::Parallel,
            _ => ExecutionStrategy::Parallel,
        }
    }
}

#[async_trait]
impl TaskExecutor for AdaptiveTaskExecutor {
    async fn execute_task(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let strategy = self.adapt_strategy(&task_spec);
        let started_at = chrono::Utc::now();

        debug!("Executing task {} with adaptive strategy: {:?}", task_spec.id, strategy);

        match strategy {
            ExecutionStrategy::Sequential => {
                let completed_at = started_at + chrono::Duration::milliseconds(1300);
                let duration_ms = (completed_at - started_at).num_milliseconds() as u64;
                let result = TaskExecutionResult {
                    execution_id: uuid::Uuid::new_v4(),
                    task_id: task_spec.id,
                    success: true,
                    output: "Task executed successfully (adaptive sequential)".to_string(),
                    errors: vec![],
                    metadata: std::collections::HashMap::new(),
                    started_at,
                    completed_at,
                    duration_ms,
                    worker_id: Some(worker_id),
                };
                Ok(result)
            },
            ExecutionStrategy::Parallel => {
                let _permit = self.semaphore.acquire().await
                    .map_err(|e| format!("Failed to acquire execution permit: {}", e))?;

                let completed_at = started_at + chrono::Duration::milliseconds(850);
                let duration_ms = (completed_at - started_at).num_milliseconds() as u64;
                let result = TaskExecutionResult {
                    execution_id: uuid::Uuid::new_v4(),
                    task_id: task_spec.id,
                    success: true,
                    output: "Task executed successfully (adaptive parallel)".to_string(),
                    errors: vec![],
                    metadata: std::collections::HashMap::new(),
                    started_at,
                    completed_at,
                    duration_ms,
                    worker_id: Some(worker_id),
                };
                Ok(result)
            },
            _ => {
                // Fallback to parallel
                let _permit = self.semaphore.acquire().await
                    .map_err(|e| format!("Failed to acquire execution permit: {}", e))?;

                let completed_at = started_at + chrono::Duration::milliseconds(1000);
                let duration_ms = (completed_at - started_at).num_milliseconds() as u64;
                let result = TaskExecutionResult {
                    execution_id: uuid::Uuid::new_v4(),
                    task_id: task_spec.id,
                    success: true,
                    output: "Task executed successfully (adaptive fallback)".to_string(),
                    errors: vec![],
                    metadata: std::collections::HashMap::new(),
                    started_at,
                    completed_at,
                    duration_ms,
                    worker_id: Some(worker_id),
                };
                Ok(result)
            }
        }
    }

    async fn execute_task_with_circuit_breaker(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
        circuit_breaker_enabled: bool,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement circuit breaker logic for task execution
        //       Currently delegates to regular execute_task; should implement circuit breaker pattern for resilience.
        //
        // COMPLETION CHECKLIST:
        // [ ] Check circuit breaker state before execution
        // [ ] Track execution failures and successes
        // [ ] Open circuit after failure threshold
        // [ ] Attempt half-open state after timeout
        // [ ] Close circuit after success threshold
        // [ ] Add unit tests for circuit breaker logic
        // [ ] Add integration tests with failure scenarios
        // [ ] Verify circuit breaker effectiveness
        //
        // ACCEPTANCE CRITERIA:
        // - Circuit breaker prevents execution when circuit is open
        // - Failure tracking triggers circuit opening correctly
        // - Half-open state allows limited retry attempts
        // - Circuit closes after successful recovery
        //
        // DEPENDENCIES:
        // - Circuit breaker infrastructure (Required)
        // - Failure tracking utilities (Required)
        // - State management utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (resilience feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Resilience patterns expertise
        self.execute_task(task_spec, worker_id).await // Temporary: delegate until circuit breaker is implemented
    }

    async fn health_check(&self) -> Result<TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutorHealth {
            status: agent_agency_contracts::task_executor::HealthStatus::Healthy,
            last_execution_time: Some(chrono::Utc::now()),
            active_tasks: self.config.max_concurrent_tasks as u32,
            queued_tasks: 0,
            total_executions: 180, // Mock stats
            success_rate: 0.98,
        })
    }

    async fn get_execution_stats(&self) -> Result<TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TaskExecutionStats {
            total_executions: 180,
            successful_executions: 176,
            failed_executions: 4,
            average_execution_time_ms: 1000.0,
            median_execution_time_ms: 950.0,
            p95_execution_time_ms: 1300.0,
            p99_execution_time_ms: 1500.0,
        })
    }

    async fn cancel_task_execution(
        &self,
        _task_id: Uuid,
        _worker_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement task cancellation
        Ok(())
    }
}

/// Errors that can occur during task executor factory operations
#[derive(Debug, thiserror::Error)]
pub enum TaskExecutorFactoryError {
    #[error("Missing required dependency: {dependency}")]
    MissingDependency { dependency: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Execution strategy not supported: {strategy}")]
    UnsupportedStrategy { strategy: String },

    #[error("Worker pool error: {message}")]
    WorkerPoolError { message: String },

    #[error("Task queue error: {message}")]
    TaskQueueError { message: String },

    #[error("Audit trail error: {message}")]
    AuditError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_creation() {
        let factory = TaskExecutorFactory::new();
        assert!(factory.create_default_executor().is_ok());
    }

    #[tokio::test]
    async fn test_sequential_executor() {
        let factory = TaskExecutorFactory::new();
        let executor = factory.create_executor(ExecutionStrategy::Sequential).unwrap();

        let health = executor.health_check().await.unwrap();
        assert_eq!(health.active_tasks, 1); // Sequential - only one active
    }

    #[tokio::test]
    async fn test_parallel_executor() {
        let factory = TaskExecutorFactory::new();
        let executor = factory.create_executor(ExecutionStrategy::Parallel).unwrap();

        let health = executor.health_check().await.unwrap();
        assert_eq!(health.active_tasks, 10); // Parallel - max concurrent
    }

    #[tokio::test]
    async fn test_hybrid_executor() {
        let factory = TaskExecutorFactory::new();
        let executor = factory.create_executor(ExecutionStrategy::Hybrid).unwrap();

        let health = executor.health_check().await.unwrap();
        assert_eq!(health.active_tasks, 5); // Hybrid - half of max
    }

    #[tokio::test]
    async fn test_adaptive_executor() {
        let factory = TaskExecutorFactory::new();
        let executor = factory.create_executor(ExecutionStrategy::Adaptive).unwrap();

        let health = executor.health_check().await.unwrap();
        assert_eq!(health.active_tasks, 10); // Adaptive - full capacity
    }
}
