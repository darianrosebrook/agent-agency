//! Worker pool manager for coordinating multiple workers
//!
//! Manages worker lifecycle, health monitoring, and coordination.

use crate::worker_errors::WorkerError;
use crate::specialized_workers::SpecializedWorker;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Worker pool manager
pub struct WorkerPoolManager {
    workers: Arc<RwLock<HashMap<String, Box<dyn SpecializedWorker + Send + Sync>>>>,
}

impl WorkerPoolManager {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_worker(&self, name: String, worker: Box<dyn SpecializedWorker + Send + Sync>) -> Result<(), WorkerError> {
        let mut workers = self.workers.write().await;
        workers.insert(name, worker);
        Ok(())
    }

    pub async fn get_worker(&self, name: &str) -> Option<&Box<dyn SpecializedWorker + Send + Sync>> {
        let workers = self.workers.read().await;
        workers.get(name)
    }

    pub async fn list_workers(&self) -> Vec<String> {
        let workers = self.workers.read().await;
        workers.keys().cloned().collect()
    }
}
