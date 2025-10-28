//! Worker management types
//!
//! This module provides worker management functionality for the coordinator.

use crate::parallel_types::{WorkerId, WorkerSpecialty};
use crate::WorkerCapabilities;
use crate::worker_types::{Worker, WorkerStatus, WorkerPerformanceMetrics};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
