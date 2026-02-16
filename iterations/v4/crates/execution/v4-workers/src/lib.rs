//! V4 Workers
//!
//! Worker pool and task execution for the Agent Agency V4 system.
//!
//! This crate provides:
//! - **Worker Management**: Worker handles with state tracking and statistics
//! - **Worker Pool**: Pool management with capacity limits and cleanup
//! - **Task Execution**: Task submission, prioritization, and coordination
//!
//! # Architecture
//!
//! ```text
//! TaskExecutor
//!     │
//!     ├── WorkerPool
//!     │       │
//!     │       ├── WorkerHandle (Local)
//!     │       ├── WorkerHandle (Sandboxed)
//!     │       └── WorkerHandle (GpuAccelerated)
//!     │
//!     └── ToolExecutor (from v4-tools)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use v4_workers::{WorkerPool, TaskExecutor, Task, TaskPriority};
//! use v4_tools::ToolRegistry;
//! use std::sync::Arc;
//!
//! // Create pool and executor
//! let pool = Arc::new(WorkerPool::new());
//! let registry = Arc::new(ToolRegistry::new());
//! let executor = TaskExecutor::new(pool, registry);
//!
//! // Create and submit task
//! let task = Task::new("task-1", vec![/* operators */])
//!     .with_priority(TaskPriority::High);
//!
//! let result = executor.submit(task).await?;
//! ```

mod pool;
mod task;
mod worker;

pub mod agent_loop;
pub mod planner;
pub mod scope_guard;
pub mod worktree;

pub use pool::{PoolConfig, PoolError, PoolStats, WorkerPool};
pub use task::{
    Task, TaskError, TaskExecutor, TaskExecutorConfig, TaskId, TaskPriority, TaskQueue, TaskResult,
    TaskStatus,
};
pub use worker::{WorkerHandle, WorkerId, WorkerInfo, WorkerState, WorkerStats, WorkerType};

// Phase 5: Agentic Loop
pub use agent_loop::{
    AgentContext, AgentError, AgentLoop, AgentResult, Attempt, TerminationReason,
};
pub use planner::{
    Approval, ApprovalError, ApprovalGate, AutoApprovalGate, LlmPlanner, PlanError, Planner,
    RulePlanner, StdinApprovalGate,
};
pub use scope_guard::{FileLock, LockError, LockMode, LockSet, LockStats, ScopeGuard};
pub use worktree::{
    MergeResult, WorktreeConfig, WorktreeError, WorktreeInfo, WorktreeManager, WorktreeStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use v4_tools::ToolRegistry;

    #[test]
    fn test_worker_pool_integration() {
        let pool = WorkerPool::new();

        // Spawn workers
        let worker1 = pool.spawn(WorkerType::Local).unwrap();
        let worker2 = pool.spawn(WorkerType::Sandboxed).unwrap();

        assert_eq!(pool.count(), 2);
        assert_eq!(pool.count_by_type(&WorkerType::Local), 1);
        assert_eq!(pool.count_by_type(&WorkerType::Sandboxed), 1);

        // Acquire worker
        let acquired = pool.acquire(&WorkerType::Local);
        assert!(acquired.is_some());

        // List workers
        let list = pool.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_worker_state_transitions() {
        let pool = WorkerPool::new();
        let worker = pool.spawn(WorkerType::Local).unwrap();

        assert_eq!(worker.state(), WorkerState::Idle);
        assert!(worker.is_available());

        worker.start_task("task-1");
        assert_eq!(worker.state(), WorkerState::Busy);
        assert!(!worker.is_available());
        assert_eq!(worker.current_task(), Some("task-1".to_string()));

        worker.complete_task(100);
        assert_eq!(worker.state(), WorkerState::Idle);
        assert!(worker.is_available());
        assert_eq!(worker.stats.completed(), 1);
    }

    #[test]
    fn test_task_queue_operations() {
        let queue = TaskQueue::new(100);

        // Enqueue tasks with different priorities
        let critical =
            Task::new("critical", vec![]).with_priority(TaskPriority::Critical);
        let normal = Task::new("normal", vec![]).with_priority(TaskPriority::Normal);

        queue.enqueue(normal).unwrap();
        queue.enqueue(critical).unwrap();

        assert_eq!(queue.len(), 2);

        // Critical should come first
        let first = queue.dequeue().unwrap();
        assert_eq!(first.id, "critical");

        let second = queue.dequeue().unwrap();
        assert_eq!(second.id, "normal");

        assert!(queue.is_empty());
    }

    #[test]
    fn test_pool_stats() {
        let pool = WorkerPool::new();

        let worker1 = pool.spawn(WorkerType::Local).unwrap();
        let worker2 = pool.spawn(WorkerType::Local).unwrap();

        worker1.start_task("task-1");

        let stats = pool.stats();
        assert_eq!(stats.total_workers, 2);
        assert_eq!(stats.idle_workers, 1);
        assert_eq!(stats.busy_workers, 1);
    }

    #[test]
    fn test_pool_capacity_limits() {
        let config = PoolConfig {
            max_local_workers: 2,
            max_sandboxed_workers: 1,
            idle_timeout_seconds: 60,
        };
        let pool = WorkerPool::with_config(config);

        // Fill local capacity
        pool.spawn(WorkerType::Local).unwrap();
        pool.spawn(WorkerType::Local).unwrap();

        // Third should fail
        let result = pool.spawn(WorkerType::Local);
        assert!(result.is_err());

        // Sandboxed should still work
        let sandboxed = pool.spawn(WorkerType::Sandboxed);
        assert!(sandboxed.is_ok());
    }

    #[tokio::test]
    async fn test_task_executor_creation() {
        let pool = Arc::new(WorkerPool::new());
        let registry = Arc::new(ToolRegistry::new());

        let executor = TaskExecutor::new(pool.clone(), registry);

        // Verify pool is connected
        let stats = executor.pool_stats();
        assert_eq!(stats.total_workers, 0);
    }

    #[test]
    fn test_task_builder() {
        let task = Task::new("test", vec![])
            .with_priority(TaskPriority::High)
            .with_timeout(5000)
            .with_working_dir("/tmp")
            .with_sandbox()
            .with_worker_type(WorkerType::Sandboxed);

        assert_eq!(task.id, "test");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.timeout_ms, 5000);
        assert_eq!(task.working_dir, Some("/tmp".to_string()));
        assert!(task.sandboxed);
        assert_eq!(task.preferred_worker, Some(WorkerType::Sandboxed));
    }
}
