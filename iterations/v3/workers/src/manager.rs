//! Worker Pool Manager
//!
//! Manages the lifecycle of workers in the pool, including registration,
//! health checking, load balancing, and performance monitoring.

use crate::types::*;
use crate::{TaskRouter, TaskExecutor, CawsChecker, WorkerPoolConfig};
use agent_agency_council::models::TaskSpec;
use anyhow::{Context, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Main worker pool manager
#[derive(Debug)]
pub struct WorkerPoolManager {
    config: WorkerPoolConfig,
    workers: Arc<DashMap<Uuid, Worker>>,
    task_router: Arc<TaskRouter>,
    task_executor: Arc<TaskExecutor>,
    caws_checker: Arc<CawsChecker>,
    event_sender: mpsc::UnboundedSender<WorkerPoolEvent>,
    stats: Arc<RwLock<WorkerPoolStats>>,
    health_check_handle: Option<tokio::task::JoinHandle<()>>,
    pool_start_time: Instant,
}

impl WorkerPoolManager {
    /// Create a new worker pool manager
    pub fn new(config: WorkerPoolConfig) -> Self {
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            workers: Arc::new(DashMap::new()),
            task_router: Arc::new(TaskRouter::new()),
            task_executor: Arc::new(TaskExecutor::new()),
            caws_checker: Arc::new(CawsChecker::new()),
            event_sender,
            stats: Arc::new(RwLock::new(WorkerPoolStats {
                total_workers: 0,
                available_workers: 0,
                busy_workers: 0,
                unavailable_workers: 0,
                total_tasks_completed: 0,
                total_tasks_failed: 0,
                average_execution_time_ms: 0.0,
                average_quality_score: 0.0,
                average_caws_compliance: 0.0,
                pool_uptime_seconds: 0,
                last_updated: chrono::Utc::now(),
            })),
            health_check_handle: None,
            pool_start_time: Instant::now(),
        }
    }

    /// Initialize the worker pool manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing worker pool manager");

        // Start health check task
        self.start_health_check_task().await?;

        // Auto-discover workers if enabled
        if self.config.registry.auto_discover {
            self.auto_discover_workers().await?;
        }

        info!("Worker pool manager initialized with {} workers", self.workers.len());
        Ok(())
    }

    /// Register a new worker
    pub async fn register_worker(&self, registration: WorkerRegistration) -> Result<Worker> {
        let worker = Worker::new(
            registration.name,
            registration.worker_type,
            registration.model_name,
            registration.endpoint,
            registration.capabilities,
        );

        // Set metadata
        let mut worker_clone = worker.clone();
        worker_clone.metadata = registration.metadata;

        // Perform health check
        if !self.check_worker_health(&worker_clone).await? {
            return Err(anyhow::anyhow!("Worker health check failed"));
        }

        // Register worker
        self.workers.insert(worker.id, worker_clone.clone());

        // Update stats
        self.update_stats().await;

        // Send event
        let _ = self.event_sender.send(WorkerPoolEvent::WorkerRegistered { 
            worker: worker_clone.clone() 
        });

        info!("Registered worker: {} ({})", worker_clone.name, worker_clone.id);
        Ok(worker_clone)
    }

    /// Deregister a worker
    pub async fn deregister_worker(&self, worker_id: Uuid) -> Result<()> {
        if let Some((_, worker)) = self.workers.remove(&worker_id) {
            // Send event
            let _ = self.event_sender.send(WorkerPoolEvent::WorkerDeregistered { 
                worker_id: worker.id 
            });

            info!("Deregistered worker: {} ({})", worker.name, worker.id);
        } else {
            return Err(anyhow::anyhow!("Worker not found: {}", worker_id));
        }

        // Update stats
        self.update_stats().await;

        Ok(())
    }

    /// Get worker by ID
    pub async fn get_worker(&self, worker_id: Uuid) -> Option<Worker> {
        self.workers.get(&worker_id).map(|entry| entry.value().clone())
    }

    /// Get all workers
    pub async fn get_workers(&self) -> Vec<Worker> {
        self.workers.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get available workers
    pub async fn get_available_workers(&self) -> Vec<Worker> {
        self.workers.iter()
            .filter(|entry| matches!(entry.value().status, WorkerStatus::Available))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get workers by type
    pub async fn get_workers_by_type(&self, worker_type: &WorkerType) -> Vec<Worker> {
        self.workers.iter()
            .filter(|entry| &entry.value().worker_type == worker_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Route and execute a task
    pub async fn execute_task(&self, task_spec: TaskSpec) -> Result<TaskExecutionResult> {
        let task_id = task_spec.id;
        info!("Executing task: {} ({})", task_spec.title, task_id);

        // Route task to appropriate workers
        let routing_result = self.task_router.route_task(&task_spec, &self.workers).await?;
        
        if routing_result.selected_workers.is_empty() {
            return Err(anyhow::anyhow!("No suitable workers found for task"));
        }

        // Select the best worker
        let best_assignment = &routing_result.selected_workers[0];
        let worker_id = best_assignment.worker_id;

        // Update worker status to busy
        if let Some(mut worker) = self.workers.get_mut(&worker_id) {
            let old_status = worker.status.clone();
            worker.status = WorkerStatus::Busy;
            
            // Send event
            let _ = self.event_sender.send(WorkerPoolEvent::WorkerStatusChanged {
                worker_id,
                old_status,
                new_status: WorkerStatus::Busy,
            });
        }

        // Send task assignment event
        let _ = self.event_sender.send(WorkerPoolEvent::TaskAssigned {
            task_id,
            worker_id,
        });

        // Execute task
        let result = self.task_executor.execute_task(task_spec, worker_id).await?;
        let result_status = result.status.clone();

        // Update worker performance metrics
        if let Some(mut worker) = self.workers.get_mut(&worker_id) {
            worker.update_performance_metrics(&result);
            
            // Reset status based on result
            worker.status = match result_status {
                ExecutionStatus::Completed | ExecutionStatus::Partial => WorkerStatus::Available,
                ExecutionStatus::Failed | ExecutionStatus::Timeout | ExecutionStatus::Cancelled => {
                    // Keep busy if failed to allow retry logic
                    WorkerStatus::Available
                }
            };

            worker.last_heartbeat = chrono::Utc::now();
        }

        // Update stats
        self.update_stats().await;

        // Send completion event
        let event = match result.status {
            ExecutionStatus::Completed | ExecutionStatus::Partial => {
                WorkerPoolEvent::TaskCompleted { task_id, worker_id, result: result.clone() }
            }
            ExecutionStatus::Failed | ExecutionStatus::Timeout | ExecutionStatus::Cancelled => {
                WorkerPoolEvent::TaskFailed {
                    task_id,
                    worker_id,
                    error: result.error_message.as_ref().unwrap_or(&"Unknown error".to_string()).clone()
                }
            }
        };
        let _ = self.event_sender.send(event);

        info!("Completed task: {} ({})", task_id, result_status);
        Ok(result)
    }

    /// Update worker status
    pub async fn update_worker_status(&self, worker_id: Uuid, new_status: WorkerStatus) -> Result<()> {
        if let Some(mut worker) = self.workers.get_mut(&worker_id) {
            let old_status = worker.status.clone();
            worker.status = new_status.clone();
            worker.last_heartbeat = chrono::Utc::now();

            // Send event
            let _ = self.event_sender.send(WorkerPoolEvent::WorkerStatusChanged {
                worker_id,
                old_status,
                new_status,
            });

            // Update stats
            self.update_stats().await;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Worker not found: {}", worker_id))
        }
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> WorkerPoolStats {
        let mut stats = self.stats.read().await.clone();
        stats.pool_uptime_seconds = self.pool_start_time.elapsed().as_secs();
        stats
    }

    /// Check worker health
    async fn check_worker_health(&self, worker: &Worker) -> Result<bool> {
        let start_time = Instant::now();
        
        // TODO: Implement actual health check
        // For now, simulate health check
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Simulate health check result
        let is_healthy = response_time < 1000; // Healthy if response < 1s
        
        if !is_healthy {
            warn!("Worker health check failed: {} ({})", worker.name, worker.id);
        }

        Ok(is_healthy)
    }

    /// Start health check task
    async fn start_health_check_task(&mut self) -> Result<()> {
        let workers = self.workers.clone();
        let event_sender = self.event_sender.clone();
        let interval = Duration::from_millis(self.config.health_check_interval_ms);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            
            loop {
                interval.tick().await;
                
                // Check health of all workers
                for entry in workers.iter() {
                    let worker = entry.value();
                    let worker_id = worker.id;
                    
                    // TODO: Implement actual health check
                    // For now, just update heartbeat
                    if let Some(mut worker_mut) = workers.get_mut(&worker_id) {
                        worker_mut.last_heartbeat = chrono::Utc::now();
                    }
                }
            }
        });

        self.health_check_handle = Some(handle);
        Ok(())
    }

    /// Auto-discover workers from endpoints
    async fn auto_discover_workers(&self) -> Result<()> {
        info!("Auto-discovering workers from endpoints");

        for endpoint in &self.config.registry.discovery_endpoints {
            // TODO: Implement actual worker discovery
            // For now, create mock workers
            let mock_worker = WorkerRegistration {
                name: format!("Auto-discovered worker at {}", endpoint),
                worker_type: WorkerType::Generalist,
                model_name: "llama3.3:7b".to_string(),
                endpoint: endpoint.clone(),
                capabilities: WorkerCapabilities::default(),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = self.register_worker(mock_worker).await {
                warn!("Failed to register auto-discovered worker at {}: {}", endpoint, e);
            }
        }

        Ok(())
    }

    /// Update pool statistics
    async fn update_stats(&self) {
        let mut stats = self.stats.write().await;
        
        stats.total_workers = self.workers.len() as u32;
        stats.available_workers = self.workers.iter()
            .filter(|entry| matches!(entry.value().status, WorkerStatus::Available))
            .count() as u32;
        stats.busy_workers = self.workers.iter()
            .filter(|entry| matches!(entry.value().status, WorkerStatus::Busy))
            .count() as u32;
        stats.unavailable_workers = self.workers.iter()
            .filter(|entry| matches!(entry.value().status, WorkerStatus::Unavailable | WorkerStatus::Maintenance))
            .count() as u32;

        // Calculate averages
        if stats.total_workers > 0 {
            let mut total_execution_time = 0.0;
            let mut total_quality = 0.0;
            let mut total_compliance = 0.0;
            let mut worker_count = 0;

            for entry in self.workers.iter() {
                let metrics = &entry.value().performance_metrics;
                if metrics.total_tasks > 0 {
                    total_execution_time += metrics.average_execution_time_ms;
                    total_quality += metrics.average_quality_score;
                    total_compliance += metrics.average_caws_compliance;
                    worker_count += 1;
                }
            }

            if worker_count > 0 {
                stats.average_execution_time_ms = total_execution_time / worker_count as f64;
                stats.average_quality_score = total_quality / worker_count as f32;
                stats.average_caws_compliance = total_compliance / worker_count as f32;
            }
        }

        stats.last_updated = chrono::Utc::now();
    }

    /// Shutdown the worker pool manager
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down worker pool manager");

        // Cancel health check task
        if let Some(handle) = self.health_check_handle.take() {
            handle.abort();
        }

        // Deregister all workers
        let worker_ids: Vec<Uuid> = self.workers.iter().map(|entry| entry.key().clone()).collect();
        for worker_id in worker_ids {
            if let Err(e) = self.deregister_worker(worker_id).await {
                warn!("Failed to deregister worker during shutdown: {}", e);
            }
        }

        info!("Worker pool manager shutdown complete");
        Ok(())
    }
}

#[async_trait]
pub trait WorkerPoolService: Send + Sync {
    async fn register_worker(&self, registration: WorkerRegistration) -> Result<Worker>;
    async fn deregister_worker(&self, worker_id: Uuid) -> Result<()>;
    async fn get_worker(&self, worker_id: Uuid) -> Option<Worker>;
    async fn get_workers(&self) -> Vec<Worker>;
    async fn get_available_workers(&self) -> Vec<Worker>;
    async fn execute_task(&self, task_spec: TaskSpec) -> Result<TaskExecutionResult>;
    async fn update_worker_status(&self, worker_id: Uuid, status: WorkerStatus) -> Result<()>;
    async fn get_stats(&self) -> WorkerPoolStats;
}

#[async_trait]
impl WorkerPoolService for WorkerPoolManager {
    async fn register_worker(&self, registration: WorkerRegistration) -> Result<Worker> {
        self.register_worker(registration).await
    }

    async fn deregister_worker(&self, worker_id: Uuid) -> Result<()> {
        self.deregister_worker(worker_id).await
    }

    async fn get_worker(&self, worker_id: Uuid) -> Option<Worker> {
        self.get_worker(worker_id).await
    }

    async fn get_workers(&self) -> Vec<Worker> {
        self.get_workers().await
    }

    async fn get_available_workers(&self) -> Vec<Worker> {
        self.get_available_workers().await
    }

    async fn execute_task(&self, task_spec: TaskSpec) -> Result<TaskExecutionResult> {
        self.execute_task(task_spec).await
    }

    async fn update_worker_status(&self, worker_id: Uuid, status: WorkerStatus) -> Result<()> {
        self.update_worker_status(worker_id, status).await
    }

    async fn get_stats(&self) -> WorkerPoolStats {
        self.get_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_pool_manager_creation() {
        let config = WorkerPoolConfig::default();
        let manager = WorkerPoolManager::new(config);
        assert_eq!(manager.workers.len(), 0);
    }

    #[tokio::test]
    async fn test_worker_registration() {
        let config = WorkerPoolConfig::default();
        let manager = WorkerPoolManager::new(config);
        
        let registration = WorkerRegistration {
            name: "test-worker".to_string(),
            worker_type: WorkerType::Generalist,
            model_name: "llama3.3:7b".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            capabilities: WorkerCapabilities::default(),
            metadata: std::collections::HashMap::new(),
        };

        let worker = manager.register_worker(registration).await.unwrap();
        assert_eq!(worker.name, "test-worker");
        assert_eq!(manager.workers.len(), 1);
    }

    #[tokio::test]
    async fn test_worker_deregistration() {
        let config = WorkerPoolConfig::default();
        let manager = WorkerPoolManager::new(config);
        
        let registration = WorkerRegistration {
            name: "test-worker".to_string(),
            worker_type: WorkerType::Generalist,
            model_name: "llama3.3:7b".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            capabilities: WorkerCapabilities::default(),
            metadata: std::collections::HashMap::new(),
        };

        let worker = manager.register_worker(registration).await.unwrap();
        manager.deregister_worker(worker.id).await.unwrap();
        assert_eq!(manager.workers.len(), 0);
    }

    #[tokio::test]
    async fn test_get_available_workers() {
        let config = WorkerPoolConfig::default();
        let manager = WorkerPoolManager::new(config);
        
        let registration = WorkerRegistration {
            name: "test-worker".to_string(),
            worker_type: WorkerType::Generalist,
            model_name: "llama3.3:7b".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            capabilities: WorkerCapabilities::default(),
            metadata: std::collections::HashMap::new(),
        };

        manager.register_worker(registration).await.unwrap();
        
        let available = manager.get_available_workers().await;
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].name, "test-worker");
    }
}
