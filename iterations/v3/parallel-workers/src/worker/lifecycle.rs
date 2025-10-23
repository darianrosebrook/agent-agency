//! Worker lifecycle management

use crate::types::*;
use crate::error::{WorkerError, WorkerExecutionResult};
use crate::worker::specialization::*;
use crate::progress::WorkerStatus;
use std::sync::Arc;
use parking_lot::RwLock;

/// Manages the lifecycle of parallel workers
pub struct WorkerManager {
    factory: WorkerFactory,
    active_workers: Arc<RwLock<std::collections::HashMap<WorkerId, WorkerHandle>>>,
    worker_pool: Arc<dyn WorkerPool>, // Integration point with existing workers
}

impl WorkerManager {
    pub fn new(worker_pool: Arc<dyn WorkerPool>) -> Self {
        Self {
            factory: WorkerFactory::new(),
            active_workers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            worker_pool,
        }
    }

    /// Spawn a worker for a specific subtask
    pub async fn spawn_worker(&mut self, subtask: SubTask) -> WorkerExecutionResult<WorkerId> {
        let worker_id = WorkerId::new();

        // Find an appropriate specialized worker
        let worker = self.factory.find_worker(&subtask.specialty)
            .ok_or_else(|| WorkerError::NoSpecializedWorkerAvailable {
                specialty: subtask.specialty.clone(),
            })?;

        // Create isolated worker environment (move out of self to avoid lifetime issues)
        let workspace_root = std::env::current_dir()
            .map_err(|e| WorkerError::ExecutionFailed {
                worker_id: worker_id.clone(),
                message: format!("Failed to get workspace root: {}", e),
            })?;
        let isolated_env = std::collections::HashMap::new(); // TODO: Implement proper env isolation

        // Create isolated worker context
        let context = WorkerContext {
            subtask: subtask.clone(),
            workspace_root,
            isolated_env,
            communication_channel: tokio::sync::mpsc::unbounded_channel().0, // Will be replaced
        };

        // TODO: Implement proper async worker spawning
        // For now, just create the handle and return - actual execution will be synchronous
        // tokio::spawn(async move {
        //     let result = worker_clone.execute_subtask(context_clone).await;
        //     // TODO: Send result back through communication channel
        // });

        // Create worker handle
        let handle = WorkerHandle {
            id: worker_id.clone(),
            subtask_id: subtask.id,
            start_time: chrono::Utc::now(),
        };

        // Track the active worker
        self.active_workers.write().insert(worker_id.clone(), handle);

        Ok(worker_id)
    }

    /// Spawn multiple workers in parallel
    pub async fn spawn_workers(&mut self, subtasks: Vec<SubTask>) -> WorkerExecutionResult<Vec<WorkerHandle>> {
        let mut handles = Vec::new();
        let mut errors = Vec::new();

        // Spawn workers sequentially for now (to avoid lifetime issues)
        for subtask in subtasks {
            match self.spawn_worker(subtask).await {
                Ok(worker_id) => {
                    if let Some(handle) = self.active_workers.read().get(&worker_id).cloned() {
                        handles.push(handle);
                    } else {
                        errors.push(WorkerError::WorkerNotFound { worker_id });
                    }
                }
                Err(e) => errors.push(e),
            }
        }

        // If we had some successes but also errors, return partial success
        // If all failed, return the first error
        if handles.is_empty() && !errors.is_empty() {
            return Err(errors.into_iter().next().unwrap());
        }

        Ok(handles)
    }

    /// Get the status of a worker
    pub async fn get_worker_status(&self, worker_id: &WorkerId) -> Option<WorkerStatus> {
        let active_workers = self.active_workers.read();
        active_workers.get(worker_id)?;

        // TODO: Implement proper status tracking without join handles
        Some(WorkerStatus::Running)
    }

    /// Cancel a running worker
    pub async fn cancel_worker(&self, worker_id: &WorkerId) -> WorkerExecutionResult<()> {
        let mut active_workers = self.active_workers.write();

        if active_workers.contains_key(worker_id) {
            // TODO: Implement proper cancellation without join handles
            // Remove from active workers
            active_workers.remove(worker_id);

            Ok(())
        } else {
            Err(WorkerError::WorkerNotFound {
                worker_id: worker_id.clone(),
            })
        }
    }

    /// Wait for a worker to complete
    pub async fn wait_for_worker(&self, _worker_id: &WorkerId) -> WorkerExecutionResult<WorkerResult> {
        // TODO: Implement proper join handle tracking to avoid lifetime issues
        Err(WorkerError::ExecutionFailed {
            worker_id: _worker_id.clone(),
            message: "Join handle tracking not yet implemented".to_string(),
        })
    }

    /// Wait for all workers to complete
    pub async fn wait_for_all_workers(&self) -> WorkerExecutionResult<Vec<WorkerResult>> {
        let worker_ids: Vec<_> = {
            let active_workers = self.active_workers.read();
            active_workers.keys().cloned().collect()
        };

        let mut results = Vec::new();

        for worker_id in worker_ids {
            match self.wait_for_worker(&worker_id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Log error but continue with other workers
                    tracing::error!("Worker {} failed: {:?}", worker_id.0, e);
                }
            }
        }

        Ok(results)
    }

    /// Get all active worker IDs
    pub fn active_worker_ids(&self) -> Vec<WorkerId> {
        self.active_workers.read().keys().cloned().collect()
    }

    /// Get the number of active workers
    pub fn active_worker_count(&self) -> usize {
        self.active_workers.read().len()
    }

    /// Check if a worker is active
    pub fn is_worker_active(&self, worker_id: &WorkerId) -> bool {
        self.active_workers.read().contains_key(worker_id)
    }

    /// Get worker statistics
    pub fn get_statistics(&self) -> WorkerStatistics {
        let active_workers = self.active_workers.read();
        let total_workers = active_workers.len();

        let mut running = 0;
        let completed = 0;
        let failed = 0;

        // Note: This is a simplified check - in practice we'd need to poll the join handles
        // For now, assume all are running
        running = total_workers;

        WorkerStatistics {
            total_workers,
            running_workers: running,
            completed_workers: completed,
            failed_workers: failed,
        }
    }

    /// Create an isolated environment for a worker
    fn create_isolated_env(&self, subtask: &SubTask) -> WorkerExecutionResult<std::collections::HashMap<String, String>> {
        let mut env = std::collections::HashMap::new();

        // Set basic environment variables
        env.insert("WORKER_SUBTASK_ID".to_string(), subtask.id.0.clone());
        env.insert("WORKER_SPECIALTY".to_string(), format!("{:?}", subtask.specialty));
        env.insert("WORKER_TIME_BUDGET_SECS".to_string(),
                  subtask.scope.time_budget.as_secs().to_string());

        // Add subtask-specific environment
        match &subtask.specialty {
            WorkerSpecialty::CompilationErrors { error_codes } => {
                env.insert("COMPILATION_ERROR_CODES".to_string(),
                          error_codes.join(","));
            }
            WorkerSpecialty::Testing { frameworks } => {
                env.insert("TEST_FRAMEWORKS".to_string(),
                          frameworks.join(","));
            }
            _ => {} // Other specialties don't need special env vars
        }

        Ok(env)
    }
}

/// Worker pool trait for integration with existing worker infrastructure
#[async_trait::async_trait]
pub trait WorkerPool: Send + Sync {
    /// Get a specialized worker by specialty
    async fn get_specialized_worker(&self, specialty: WorkerSpecialty) -> WorkerExecutionResult<Arc<dyn SpecializedWorker>>;

    /// Return a worker to the pool
    async fn return_worker(&self, worker: Arc<dyn SpecializedWorker>) -> WorkerExecutionResult<()>;

    /// Get pool statistics
    fn get_pool_stats(&self) -> WorkerPoolStats;
}

/// Statistics for worker pool
#[derive(Debug, Clone)]
pub struct WorkerPoolStats {
    pub total_workers: usize,
    pub available_workers: usize,
    pub busy_workers: usize,
    pub specializations: std::collections::HashMap<String, usize>,
}

/// Statistics for worker manager
#[derive(Debug, Clone)]
pub struct WorkerStatistics {
    pub total_workers: usize,
    pub running_workers: usize,
    pub completed_workers: usize,
    pub failed_workers: usize,
}

/// Worker pool implementation using the existing worker infrastructure
pub struct DefaultWorkerPool {
    factory: WorkerFactory,
}

impl Default for DefaultWorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultWorkerPool {
    pub fn new() -> Self {
        Self {
            factory: WorkerFactory::new(),
        }
    }
}

#[async_trait::async_trait]
impl WorkerPool for DefaultWorkerPool {
    async fn get_specialized_worker(&self, specialty: WorkerSpecialty) -> WorkerExecutionResult<Arc<dyn SpecializedWorker>> {
        // For now, create new workers on demand
        // In practice, this would pool and reuse workers
        match specialty {
            WorkerSpecialty::CompilationErrors { .. } => {
                Ok(Arc::new(CompilationSpecialist::new()))
            }
            WorkerSpecialty::Refactoring { .. } => {
                Ok(Arc::new(RefactoringSpecialist::new()))
            }
            WorkerSpecialty::Testing { .. } => {
                Ok(Arc::new(TestingSpecialist::new()))
            }
            WorkerSpecialty::Documentation { .. } => {
                Ok(Arc::new(DocumentationSpecialist::new()))
            }
            WorkerSpecialty::TypeSystem { .. } => {
                // TODO: Implement TypeSystem specialist
                Err(WorkerError::NoSpecializedWorkerAvailable { specialty })
            }
            WorkerSpecialty::AsyncPatterns { .. } => {
                // TODO: Implement AsyncPatterns specialist
                Err(WorkerError::NoSpecializedWorkerAvailable { specialty })
            }
            WorkerSpecialty::Custom { .. } => {
                // For custom specialties, try to find a matching built-in worker
                if let Some(worker) = self.factory.find_worker(&specialty) {
                    // Clone the worker reference - in practice this would be Arc
                    match specialty {
                        WorkerSpecialty::CompilationErrors { .. } => {
                            Ok(Arc::new(CompilationSpecialist::new()))
                        }
                        _ => Err(WorkerError::NoSpecializedWorkerAvailable { specialty }),
                    }
                } else {
                    Err(WorkerError::NoSpecializedWorkerAvailable { specialty })
                }
            }
        }
    }

    async fn return_worker(&self, _worker: Arc<dyn SpecializedWorker>) -> WorkerExecutionResult<()> {
        // For now, just drop the worker
        // In practice, this would return it to a pool
        Ok(())
    }

    fn get_pool_stats(&self) -> WorkerPoolStats {
        WorkerPoolStats {
            total_workers: 4, // Number of built-in worker types
            available_workers: 4,
            busy_workers: 0,
            specializations: [
                ("compilation".to_string(), 1),
                ("refactoring".to_string(), 1),
                ("testing".to_string(), 1),
                ("documentation".to_string(), 1),
            ].into_iter().collect(),
        }
    }
}

impl Default for WorkerManager {
    fn default() -> Self {
        Self::new(Arc::new(DefaultWorkerPool::new()))
    }
}
