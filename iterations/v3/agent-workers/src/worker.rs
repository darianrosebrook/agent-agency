//! Worker management types
//!
//! This module provides worker management functionality for the coordinator.

use crate::parallel_types::{WorkerId, WorkerSpecialty, SubTask, TaskId, SubTaskId};
use crate::WorkerCapabilities;
use crate::worker_types::{Worker, WorkerStatus, WorkerPerformanceMetrics, Artifact};
use crate::error::ParallelError;
use crate::parallel_types::ParallelResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// Manager for worker instances
pub struct WorkerManager {
    workers: Arc<RwLock<HashMap<WorkerId, Worker>>>,
}

impl WorkerManager {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_worker(&self, worker: Worker) -> Result<(), String> {
        let mut workers = self.workers.write().await;
        workers.insert(WorkerId(worker.id), worker);
        Ok(())
    }

    pub async fn get_worker(&self, worker_id: &WorkerId) -> Option<Worker> {
        let workers = self.workers.read().await;
        workers.get(worker_id).cloned()
    }

    pub async fn list_available_workers(&self) -> Vec<WorkerId> {
        let workers = self.workers.read().await;
        workers
            .iter()
            .filter(|(_, worker)| worker.status == WorkerStatus::Available)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub async fn assign_worker(&self, worker_id: &WorkerId) -> Result<(), String> {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.status = WorkerStatus::Busy;
            Ok(())
        } else {
            Err("Worker not found".to_string())
        }
    }

    pub async fn release_worker(&self, worker_id: &WorkerId) -> Result<(), String> {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.status = WorkerStatus::Available;
            Ok(())
        } else {
            Err("Worker not found".to_string())
        }
    }

    /// Execute a subtask with a specific worker
    pub async fn execute_subtask(
        &self,
        subtask: SubTask,
        worker_id: WorkerId,
    ) -> ParallelResult<SubTaskExecutionResult> {
        // Get the worker
        let worker = self.get_worker(&worker_id).await
            .ok_or_else(|| ParallelError::Coordination {
                message: format!("Worker {} not found", worker_id),
                source: None,
            })?;

        // Assign the worker to the subtask
        self.assign_worker(&worker_id).await?;

        // Execute the subtask using the worker's capabilities
        // PLACEHOLDER: Real execution logic would go here
        // For now, simulate execution
        let start_time = std::time::Instant::now();
        
        // Simulate task execution
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        let execution_time = start_time.elapsed();
        
        // Release the worker
        let _ = self.release_worker(&worker_id).await;

        // Create execution result
        Ok(SubTaskExecutionResult {
            task_id: subtask.parent_task_id,
            subtask_id: subtask.id,
            success: true,
            quality_score: 0.8, // Default quality score
            artifacts: vec![],
            errors: vec![],
        })
    }
}

/// Result from executing a subtask
#[derive(Debug, Clone)]
pub struct SubTaskExecutionResult {
    pub task_id: TaskId,
    pub subtask_id: SubTaskId,
    pub success: bool,
    pub quality_score: f64,
    pub artifacts: Vec<Artifact>,
    pub errors: Vec<String>,
}

/// Default worker pool implementation
pub struct DefaultWorkerPool {
    manager: WorkerManager,
}

impl DefaultWorkerPool {
    pub fn new() -> Self {
        Self {
            manager: WorkerManager::new(),
        }
    }

    pub fn manager(&self) -> &WorkerManager {
        &self.manager
    }
}

impl Default for DefaultWorkerPool {
    fn default() -> Self {
        Self::new()
    }
}
