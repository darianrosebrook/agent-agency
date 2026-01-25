//! Task Execution
//!
//! Task submission and execution within the worker pool.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use v4_tools::{ExecutionContext, ToolExecutor, ToolRegistry};
use v4_types::operators::OperatorResult;
use v4_types::OperatorType;

use crate::pool::WorkerPool;
use crate::worker::{WorkerHandle, WorkerType};

/// Task identifier
pub type TaskId = String;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Low priority - background tasks
    Low = 0,
    /// Normal priority - default
    Normal = 1,
    /// High priority - user-facing
    High = 2,
    /// Critical priority - system tasks
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Waiting in queue
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled by user or system
    Cancelled,
}

/// A task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: TaskId,
    /// Operators to execute
    pub operators: Vec<OperatorType>,
    /// Task priority
    pub priority: TaskPriority,
    /// Preferred worker type
    pub preferred_worker: Option<WorkerType>,
    /// When the task was created
    pub created_at: DateTime<Utc>,
    /// Task timeout in milliseconds
    pub timeout_ms: u64,
    /// Working directory for execution
    pub working_dir: Option<String>,
    /// Whether to run in sandbox
    pub sandboxed: bool,
}

impl Task {
    /// Create a new task
    pub fn new(id: impl Into<String>, operators: Vec<OperatorType>) -> Self {
        Self {
            id: id.into(),
            operators,
            priority: TaskPriority::default(),
            preferred_worker: None,
            created_at: Utc::now(),
            timeout_ms: 30000,
            working_dir: None,
            sandboxed: false,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set preferred worker type
    pub fn with_worker_type(mut self, worker_type: WorkerType) -> Self {
        self.preferred_worker = Some(worker_type);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Enable sandbox mode
    pub fn with_sandbox(mut self) -> Self {
        self.sandboxed = true;
        self
    }
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: TaskId,
    /// Final status
    pub status: TaskStatus,
    /// Results from each operator
    pub operator_results: Vec<OperatorResult>,
    /// Worker that executed the task
    pub worker_id: String,
    /// When execution started
    pub started_at: DateTime<Utc>,
    /// When execution completed
    pub completed_at: DateTime<Utc>,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

impl TaskResult {
    /// Check if task succeeded
    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    /// Get success count
    pub fn success_count(&self) -> usize {
        self.operator_results.iter().filter(|r| r.success).count()
    }

    /// Get failure count
    pub fn failure_count(&self) -> usize {
        self.operator_results.iter().filter(|r| !r.success).count()
    }
}

/// Task executor coordinates task execution across workers
pub struct TaskExecutor {
    pool: Arc<WorkerPool>,
    tool_executor: Arc<ToolExecutor>,
    config: TaskExecutorConfig,
}

/// Task executor configuration
#[derive(Debug, Clone)]
pub struct TaskExecutorConfig {
    /// Maximum concurrent tasks
    pub max_concurrent: usize,
    /// Default timeout in milliseconds
    pub default_timeout_ms: u64,
    /// Whether to auto-spawn workers
    pub auto_spawn_workers: bool,
}

impl Default for TaskExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            default_timeout_ms: 30000,
            auto_spawn_workers: true,
        }
    }
}

/// Task executor error
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("No worker available")]
    NoWorkerAvailable,

    #[error("Task timeout")]
    Timeout,

    #[error("Task cancelled")]
    Cancelled,

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Pool error: {0}")]
    PoolError(String),
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(pool: Arc<WorkerPool>, registry: Arc<ToolRegistry>) -> Self {
        let tool_executor = Arc::new(ToolExecutor::new(registry));
        Self {
            pool,
            tool_executor,
            config: TaskExecutorConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        pool: Arc<WorkerPool>,
        registry: Arc<ToolRegistry>,
        config: TaskExecutorConfig,
    ) -> Self {
        let tool_executor = Arc::new(ToolExecutor::new(registry));
        Self {
            pool,
            tool_executor,
            config,
        }
    }

    /// Submit a task for execution
    pub async fn submit(&self, task: Task) -> Result<TaskResult, TaskError> {
        let started_at = Utc::now();

        // Get preferred worker type
        let worker_type = task.preferred_worker.unwrap_or(if task.sandboxed {
            WorkerType::Sandboxed
        } else {
            WorkerType::Local
        });

        // Try to acquire a worker
        let worker = self.acquire_worker(worker_type).await?;

        // Mark worker as busy
        worker.start_task(&task.id);

        // Build execution context
        let context = ExecutionContext::new(&task.id)
            .with_working_dir(task.working_dir.as_deref().unwrap_or("."))
            .with_timeout(task.timeout_ms);

        let context = if task.sandboxed {
            context.with_sandbox()
        } else {
            context
        };

        // Execute operators
        let mut operator_results = Vec::new();
        let mut error = None;

        for operator in &task.operators {
            match self.tool_executor.execute(operator, &context).await {
                Ok(record) => {
                    operator_results.push(record.result);
                }
                Err(e) => {
                    error = Some(e.to_string());
                    // Create failed result for this operator
                    operator_results.push(OperatorResult {
                        operator: operator.clone(),
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                        execution_time_ms: 0,
                        content_hash: None,
                    });
                    break;
                }
            }
        }

        let completed_at = Utc::now();
        let execution_time_ms = (completed_at - started_at).num_milliseconds() as u64;

        // Determine status
        let status = if error.is_some() {
            worker.fail_task();
            TaskStatus::Failed
        } else {
            worker.complete_task(execution_time_ms);
            TaskStatus::Completed
        };

        Ok(TaskResult {
            task_id: task.id,
            status,
            operator_results,
            worker_id: worker.id.clone(),
            started_at,
            completed_at,
            execution_time_ms,
            error,
        })
    }

    /// Acquire a worker, spawning if necessary
    async fn acquire_worker(&self, worker_type: WorkerType) -> Result<WorkerHandle, TaskError> {
        // Try to get existing worker
        if let Some(worker) = self.pool.acquire(worker_type) {
            return Ok(worker);
        }

        // Try to spawn new worker if configured
        if self.config.auto_spawn_workers {
            match self.pool.spawn(worker_type) {
                Ok(worker) => return Ok(worker),
                Err(_) => {
                    // Pool at capacity, try any available worker
                    if let Some(worker) = self.pool.acquire_any(worker_type) {
                        return Ok(worker);
                    }
                }
            }
        }

        Err(TaskError::NoWorkerAvailable)
    }

    /// Get pool statistics
    pub fn pool_stats(&self) -> crate::pool::PoolStats {
        self.pool.stats()
    }

    /// Shutdown all workers
    pub fn shutdown(&self) {
        self.pool.stop_all();
    }
}

/// Task queue for managing pending tasks
pub struct TaskQueue {
    pending: std::sync::RwLock<Vec<Task>>,
    max_size: usize,
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: std::sync::RwLock::new(Vec::new()),
            max_size,
        }
    }

    /// Add a task to the queue
    pub fn enqueue(&self, task: Task) -> Result<(), TaskError> {
        let mut pending = self.pending.write().unwrap();
        if pending.len() >= self.max_size {
            return Err(TaskError::ExecutionFailed("Queue full".to_string()));
        }

        // Insert sorted by priority (highest first)
        let pos = pending
            .iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(pending.len());
        pending.insert(pos, task);
        Ok(())
    }

    /// Get the next task from the queue
    pub fn dequeue(&self) -> Option<Task> {
        let mut pending = self.pending.write().unwrap();
        if pending.is_empty() {
            None
        } else {
            Some(pending.remove(0))
        }
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.pending.read().unwrap().len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.pending.read().unwrap().is_empty()
    }

    /// Clear all pending tasks
    pub fn clear(&self) {
        self.pending.write().unwrap().clear();
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::operators::SeekOp;

    #[test]
    fn test_task_creation() {
        let operators = vec![OperatorType::Seek(SeekOp::ReadFile {
            path: "test.txt".to_string(),
        })];

        let task = Task::new("task-1", operators)
            .with_priority(TaskPriority::High)
            .with_timeout(5000)
            .with_sandbox();

        assert_eq!(task.id, "task-1");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.timeout_ms, 5000);
        assert!(task.sandboxed);
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
    }

    #[test]
    fn test_task_queue_priority() {
        let queue = TaskQueue::new(100);

        let low = Task::new("low", vec![]).with_priority(TaskPriority::Low);
        let high = Task::new("high", vec![]).with_priority(TaskPriority::High);
        let normal = Task::new("normal", vec![]).with_priority(TaskPriority::Normal);

        queue.enqueue(low).unwrap();
        queue.enqueue(normal).unwrap();
        queue.enqueue(high).unwrap();

        // Should come out in priority order
        assert_eq!(queue.dequeue().unwrap().id, "high");
        assert_eq!(queue.dequeue().unwrap().id, "normal");
        assert_eq!(queue.dequeue().unwrap().id, "low");
    }

    #[test]
    fn test_task_queue_max_size() {
        let queue = TaskQueue::new(2);

        queue.enqueue(Task::new("1", vec![])).unwrap();
        queue.enqueue(Task::new("2", vec![])).unwrap();

        let result = queue.enqueue(Task::new("3", vec![]));
        assert!(result.is_err());
    }

    #[test]
    fn test_task_result() {
        let result = TaskResult {
            task_id: "task-1".to_string(),
            status: TaskStatus::Completed,
            operator_results: vec![
                OperatorResult {
                    operator: OperatorType::Seek(SeekOp::ReadFile {
                        path: "a.txt".to_string(),
                    }),
                    success: true,
                    data: None,
                    error: None,
                    execution_time_ms: 10,
                    content_hash: None,
                },
                OperatorResult {
                    operator: OperatorType::Seek(SeekOp::ReadFile {
                        path: "b.txt".to_string(),
                    }),
                    success: false,
                    data: None,
                    error: Some("Not found".to_string()),
                    execution_time_ms: 5,
                    content_hash: None,
                },
            ],
            worker_id: "worker-1".to_string(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            execution_time_ms: 15,
            error: None,
        };

        assert!(result.is_success());
        assert_eq!(result.success_count(), 1);
        assert_eq!(result.failure_count(), 1);
    }

    #[test]
    fn test_task_executor_config() {
        let config = TaskExecutorConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.default_timeout_ms, 30000);
        assert!(config.auto_spawn_workers);
    }
}
