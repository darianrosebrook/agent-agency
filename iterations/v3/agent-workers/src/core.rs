//! Core MCP Worker Pool Implementation
//!
//! Consolidates the worker pool orchestration from workers/, parallel-workers/,
//! and worker/ into a unified MCP-based system.

use crate::types::*;
use crate::mcp_integration::MCPToolRegistry;
use crate::execution::ToolExecutor;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Configuration for the MCP worker pool
#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub max_workers: usize,
    pub worker_timeout_seconds: u64,
    pub enable_quality_validation: bool,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            max_workers: 10,
            worker_timeout_seconds: 300,
            enable_quality_validation: true,
        }
    }
}

/// Handle to a worker instance
#[derive(Debug, Clone)]
pub struct WorkerHandle {
    pub id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub capabilities: WorkerCapabilities,
}

/// Main MCP-based worker pool
pub struct MCPWorkerPool {
    config: WorkerPoolConfig,
    workers: Arc<RwLock<HashMap<WorkerId, WorkerHandle>>>,
    tool_registry: Arc<MCPToolRegistry>,
    tool_executor: Arc<ToolExecutor>,
    stats: Arc<RwLock<WorkerPoolStats>>,
}

impl MCPWorkerPool {
    /// Create a new worker pool with default configuration
    pub fn new(config: WorkerPoolConfig) -> Self {
        Self {
            config: config.clone(),
            workers: Arc::new(RwLock::new(HashMap::new())),
            tool_registry: Arc::new(MCPToolRegistry::new()),
            tool_executor: Arc::new(ToolExecutor::new()),
            stats: Arc::new(RwLock::new(WorkerPoolStats {
                total_workers: 0,
                active_workers: 0,
                idle_workers: 0,
                unhealthy_workers: 0,
                total_tasks_processed: 0,
                tasks_in_progress: 0,
                average_queue_time_ms: 0.0,
                average_execution_time_ms: 0.0,
            })),
        }
    }

    /// Create a worker pool with a custom MCP tool registry
    pub fn with_tools(config: WorkerPoolConfig, tools: MCPToolRegistry) -> Self {
        let mut pool = Self::new(config);
        pool.tool_registry = Arc::new(tools);
        pool
    }

    /// Register a new worker with the pool
    pub async fn register_worker(&self, specialty: WorkerSpecialty, capabilities: WorkerCapabilities) -> WorkerHandle {
        let worker_id = WorkerId::new_v4();
        let handle = WorkerHandle {
            id: worker_id,
            specialty: specialty.clone(),
            capabilities,
        };

        let mut workers = self.workers.write().await;
        workers.insert(worker_id, handle.clone());

        let mut stats = self.stats.write().await;
        stats.total_workers += 1;

        info!("Registered worker {} with specialty {:?}", worker_id, specialty);
        handle
    }

    /// Execute a task using MCP tools
    pub async fn execute_task(&self, task: TaskDefinition) -> Result<TaskResult, WorkerError> {
        let start_time = std::time::Instant::now();

        // Find suitable worker
        let worker = self.find_suitable_worker(&task).await?;

        // Validate task requirements
        self.validate_task_requirements(&task).await?;

        // Execute using MCP tools
        let context = TaskContext {
            task_id: task.id,
            worker_id: worker.id,
            tool_id: task.required_tools.first().cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            parameters: task.parameters.clone(),
            execution_timeout: std::time::Duration::from_secs(
                task.timeout_seconds.unwrap_or(self.config.worker_timeout_seconds)
            ),
        };

        let result = self.tool_executor.execute_tool(context).await
            .map_err(|e| WorkerError::ToolExecutionError(e.to_string()))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        let task_result = TaskResult {
            task_id: task.id,
            status: if result.success { TaskStatus::Completed } else { TaskStatus::Failed },
            output: result.output,
            error_message: result.error_message,
            execution_time_ms: execution_time,
            tool_used: result.tool_id,
            quality_score: Some(0.8), // Placeholder quality score
        };

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_tasks_processed += 1;

        Ok(task_result)
    }

    /// Find a suitable worker for the given task
    async fn find_suitable_worker(&self, task: &TaskDefinition) -> Result<WorkerHandle, WorkerError> {
        let workers = self.workers.read().await;

        // Find workers that can handle required tools
        for worker in workers.values() {
            if self.worker_can_handle_task(worker, task).await {
                return Ok(worker.clone());
            }
        }

        Err(WorkerError::NoSuitableWorker)
    }

    /// Check if a worker can handle a given task
    async fn worker_can_handle_task(&self, worker: &WorkerHandle, task: &TaskDefinition) -> bool {
        // Check if worker has required specialties
        match &task.name {
            name if name.contains("react") || name.contains("component") =>
                worker.specialty == WorkerSpecialty::ReactComponent,
            name if name.contains("file") || name.contains("edit") =>
                worker.specialty == WorkerSpecialty::FileEditing,
            name if name.contains("research") || name.contains("search") =>
                worker.specialty == WorkerSpecialty::Research,
            _ => worker.specialty == WorkerSpecialty::General,
        }
    }

    /// Validate task requirements before execution
    async fn validate_task_requirements(&self, task: &TaskDefinition) -> Result<(), WorkerError> {
        // Check if required tools are available
        for tool_id in &task.required_tools {
            if !self.tool_registry.has_tool(tool_id).await {
                return Err(WorkerError::ToolNotAvailable(tool_id.clone()));
            }
        }

        Ok(())
    }

    /// Get current pool statistics
    pub async fn get_stats(&self) -> WorkerPoolStats {
        self.stats.read().await.clone()
    }

    /// Health check for the worker pool
    pub async fn health_check(&self) -> WorkerHealth {
        let stats = self.stats.read().await;
        let workers = self.workers.read().await;

        let unhealthy_count = workers.values()
            .filter(|w| matches!(w.capabilities.health_status, WorkerHealth::Unhealthy | WorkerHealth::Offline))
            .count();

        if unhealthy_count > stats.total_workers / 2 {
            WorkerHealth::Unhealthy
        } else if unhealthy_count > 0 {
            WorkerHealth::Degraded
        } else {
            WorkerHealth::Healthy
        }
    }
}

/// Error types for worker operations
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("No suitable worker available for task")]
    NoSuitableWorker,

    #[error("Required tool not available: {0}")]
    ToolNotAvailable(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionError(String),
}

/// Create a new worker pool with default configuration
pub fn create_worker_pool() -> MCPWorkerPool {
    MCPWorkerPool::new(WorkerPoolConfig::default())
}
