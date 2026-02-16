//! Worker Pool
//!
//! Manages a pool of workers for concurrent task execution.

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::worker::{WorkerHandle, WorkerId, WorkerInfo, WorkerState, WorkerType};

/// Worker pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of local workers
    pub max_local_workers: usize,
    /// Maximum number of sandboxed workers
    pub max_sandboxed_workers: usize,
    /// Worker idle timeout in seconds (after which workers are removed)
    pub idle_timeout_seconds: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_local_workers: 4,
            max_sandboxed_workers: 2,
            idle_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Worker pool manages a collection of workers
pub struct WorkerPool {
    workers: RwLock<HashMap<WorkerId, WorkerHandle>>,
    config: PoolConfig,
    next_worker_id: std::sync::atomic::AtomicU64,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new() -> Self {
        Self {
            workers: RwLock::new(HashMap::new()),
            config: PoolConfig::default(),
            next_worker_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Create with custom config
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            workers: RwLock::new(HashMap::new()),
            config,
            next_worker_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Get the next worker ID
    fn next_id(&self) -> String {
        let id = self.next_worker_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("worker-{}", id)
    }

    /// Spawn a new worker
    pub fn spawn(&self, worker_type: WorkerType) -> Result<WorkerHandle, PoolError> {
        let workers = self.workers.read().unwrap();

        // Check capacity
        let type_count = workers
            .values()
            .filter(|w| w.worker_type == worker_type && w.state().is_active())
            .count();

        let max = match &worker_type {
            WorkerType::Local => self.config.max_local_workers,
            WorkerType::Sandboxed => self.config.max_sandboxed_workers,
            WorkerType::GpuAccelerated => 1,
            WorkerType::Remote { .. } => self.config.max_local_workers, // Same limit as local
            WorkerType::ManualReview => 1,
        };

        if type_count >= max {
            return Err(PoolError::CapacityExceeded(format!(
                "Maximum {} workers of type {:?} reached",
                max, worker_type
            )));
        }

        drop(workers);

        // Create new worker
        let id = self.next_id();
        let handle = WorkerHandle::new(&id, worker_type);

        self.workers
            .write()
            .unwrap()
            .insert(id.clone(), handle.clone());

        Ok(handle)
    }

    /// Get an available worker of the specified type
    pub fn acquire(&self, worker_type: &WorkerType) -> Option<WorkerHandle> {
        let workers = self.workers.read().unwrap();

        workers
            .values()
            .find(|w| &w.worker_type == worker_type && w.is_available())
            .cloned()
    }

    /// Get an available worker of any type, preferring the specified type
    pub fn acquire_any(&self, preferred: &WorkerType) -> Option<WorkerHandle> {
        // Try preferred first
        if let Some(worker) = self.acquire(preferred) {
            return Some(worker);
        }

        // Fall back to any available worker
        let workers = self.workers.read().unwrap();
        workers.values().find(|w| w.is_available()).cloned()
    }

    /// Get a specific worker by ID
    pub fn get(&self, id: &str) -> Option<WorkerHandle> {
        self.workers.read().unwrap().get(id).cloned()
    }

    /// Remove a worker from the pool
    pub fn remove(&self, id: &str) -> bool {
        self.workers.write().unwrap().remove(id).is_some()
    }

    /// Stop all workers
    pub fn stop_all(&self) {
        let workers = self.workers.read().unwrap();
        for worker in workers.values() {
            worker.set_state(WorkerState::ShuttingDown);
        }
    }

    /// Get all worker info
    pub fn list(&self) -> Vec<WorkerInfo> {
        self.workers
            .read()
            .unwrap()
            .values()
            .map(|w| w.info())
            .collect()
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let workers = self.workers.read().unwrap();

        let mut stats = PoolStats {
            total_workers: workers.len(),
            ..Default::default()
        };

        for worker in workers.values() {
            match worker.state() {
                WorkerState::Idle => stats.idle_workers += 1,
                WorkerState::Busy => stats.busy_workers += 1,
                _ => {}
            }
            stats.total_tasks_completed += worker.stats.completed() as usize;
            stats.total_tasks_failed += worker.stats.failed() as usize;
        }

        stats
    }

    /// Clean up idle workers that have exceeded the timeout
    pub fn cleanup_idle(&self) -> usize {
        let timeout = chrono::Duration::seconds(self.config.idle_timeout_seconds as i64);
        let cutoff = Utc::now() - timeout;

        let mut to_remove = Vec::new();

        {
            let workers = self.workers.read().unwrap();
            for (id, worker) in workers.iter() {
                if worker.state() == WorkerState::Idle {
                    let info = worker.info();
                    if let Some(last_idle) = info.last_idle_at {
                        if last_idle < cutoff {
                            to_remove.push(id.clone());
                        }
                    }
                }
            }
        }

        let mut workers = self.workers.write().unwrap();
        for id in &to_remove {
            if let Some(worker) = workers.get(id) {
                worker.set_state(WorkerState::Stopped);
            }
            workers.remove(id);
        }

        to_remove.len()
    }

    /// Get worker count by type
    pub fn count_by_type(&self, worker_type: &WorkerType) -> usize {
        self.workers
            .read()
            .unwrap()
            .values()
            .filter(|w| &w.worker_type == worker_type && w.state().is_active())
            .count()
    }

    /// Get total worker count
    pub fn count(&self) -> usize {
        self.workers.read().unwrap().len()
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total workers in pool
    pub total_workers: usize,
    /// Idle workers
    pub idle_workers: usize,
    /// Busy workers
    pub busy_workers: usize,
    /// Total tasks completed across all workers
    pub total_tasks_completed: usize,
    /// Total tasks failed across all workers
    pub total_tasks_failed: usize,
}

/// Pool error
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),

    #[error("Worker not found: {0}")]
    WorkerNotFound(String),

    #[error("Worker unavailable: {0}")]
    WorkerUnavailable(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_spawn() {
        let pool = WorkerPool::new();

        let worker = pool.spawn(WorkerType::Local).unwrap();
        assert!(worker.is_available());
        assert_eq!(pool.count(), 1);
    }

    #[test]
    fn test_pool_capacity() {
        let config = PoolConfig {
            max_local_workers: 2,
            max_sandboxed_workers: 1,
            idle_timeout_seconds: 60,
        };
        let pool = WorkerPool::with_config(config);

        pool.spawn(WorkerType::Local).unwrap();
        pool.spawn(WorkerType::Local).unwrap();

        // Third should fail
        let result = pool.spawn(WorkerType::Local);
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_acquire() {
        let pool = WorkerPool::new();

        pool.spawn(WorkerType::Local).unwrap();
        pool.spawn(WorkerType::Sandboxed).unwrap();

        let local = pool.acquire(&WorkerType::Local);
        assert!(local.is_some());

        let sandboxed = pool.acquire(&WorkerType::Sandboxed);
        assert!(sandboxed.is_some());
    }

    #[test]
    fn test_pool_acquire_busy() {
        let pool = WorkerPool::new();

        let worker = pool.spawn(WorkerType::Local).unwrap();
        worker.start_task("task-1");

        // Should not find available worker
        let acquired = pool.acquire(&WorkerType::Local);
        assert!(acquired.is_none());
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
    fn test_pool_list() {
        let pool = WorkerPool::new();

        pool.spawn(WorkerType::Local).unwrap();
        pool.spawn(WorkerType::Sandboxed).unwrap();

        let list = pool.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_pool_remove() {
        let pool = WorkerPool::new();

        let worker = pool.spawn(WorkerType::Local).unwrap();
        let id = worker.id.clone();

        assert!(pool.remove(&id));
        assert_eq!(pool.count(), 0);
    }
}
