//! Core MCP Worker Pool Implementation
//!
//! Consolidates the worker pool orchestration from workers/, parallel-workers/,
//! and worker/ into a unified MCP-based system.

use crate::types::*;
use crate::mcp_integration::MCPIntegration;
use crate::execution::ToolExecutor;
use agent_mcp::{ToolRegistry, ToolExecutionRequest};
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

/// Handle to a worker instance with access to shared memory system
#[derive(Debug, Clone)]
pub struct WorkerHandle {
    pub id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub capabilities: WorkerCapabilities,
    /// Access to shared memory system - all agents use the same memory
    pub memory_access: std::sync::Arc<agent_memory::MemorySystem>,
}

/// Main MCP-based worker pool with shared memory system
pub struct MCPWorkerPool {
    config: WorkerPoolConfig,
    workers: Arc<RwLock<HashMap<WorkerId, WorkerHandle>>>,
    mcp_integration: Arc<MCPIntegration>,
    stats: Arc<RwLock<WorkerPoolStats>>,
    /// Single shared memory system - all agents access this same instance
    shared_memory_system: Arc<agent_memory::MemorySystem>,
}

impl MCPWorkerPool {
    /// Create a new worker pool with an MCP tool registry
    pub fn new_with_registry(config: WorkerPoolConfig, tool_registry: Arc<ToolRegistry>, shared_memory: Arc<agent_memory::MemorySystem>) -> Self {
        Self {
            config: config.clone(),
            workers: Arc::new(RwLock::new(HashMap::new())),
            mcp_integration: Arc::new(MCPIntegration::new(tool_registry)),
            shared_memory_system: shared_memory,
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

    /// Create a worker pool with a new MCP tool registry and shared memory
    pub async fn new(config: WorkerPoolConfig) -> Self {
        let tool_registry = Arc::new(ToolRegistry::new());
        tool_registry.initialize().await.unwrap(); // Initialize the registry

        // Initialize shared memory system - single instance for all agents
        let memory_config = agent_memory::MemoryConfig::default();
        let shared_memory = Arc::new(agent_memory::MemorySystem::init(memory_config).await.unwrap());

        Self::new_with_registry(config, tool_registry, shared_memory)
    }

    /// Get access to the MCP integration layer
    pub fn mcp_integration(&self) -> Arc<MCPIntegration> {
        Arc::clone(&self.mcp_integration)
    }

    /// Register a new worker with the pool (gives access to shared memory system)
    pub async fn register_worker(&self, specialty: WorkerSpecialty, capabilities: WorkerCapabilities) -> Result<WorkerHandle, Box<dyn std::error::Error + Send + Sync>> {
        let worker_id = WorkerId::new_v4();

        // Give worker access to shared memory system - all agents share the same memory
        let handle = WorkerHandle {
            id: worker_id,
            specialty: specialty.clone(),
            capabilities,
            memory_access: Arc::clone(&self.shared_memory_system),
        };

        let mut workers = self.workers.write().await;
        workers.insert(worker_id, handle.clone());

        let mut stats = self.stats.write().await;
        stats.total_workers += 1;

        info!("Registered worker {} with specialty {:?} and memory system", worker_id, specialty);
        Ok(handle)
    }

    /// Execute a task using MCP tools
    pub async fn execute_task(&self, task: TaskDefinition) -> Result<TaskResult, WorkerError> {
        let start_time = std::time::Instant::now();

        // Find suitable worker
        let worker = self.find_suitable_worker(&task).await?;

        // Validate task requirements
        self.validate_task_requirements(&task).await?;

        // Get the primary tool for this task
        let tool_id = task.required_tools.first()
            .ok_or_else(|| WorkerError::ToolNotAvailable("No tools specified".to_string()))?;

        // Find the tool in the MCP registry
        let available_tools = self.mcp_integration.list_tools().await;
        let mcp_tool = available_tools.iter()
            .find(|t| t.name == *tool_id)
            .ok_or_else(|| WorkerError::ToolNotAvailable(tool_id.clone()))?;

        // Create MCP execution request
        let request = ToolExecutionRequest {
            id: uuid::Uuid::new_v4(),
            tool_id: mcp_tool.id,
            parameters: task.parameters.clone(),
            timeout_seconds: task.timeout_seconds.map(|t| t as u64),
            context: Some(serde_json::json!({
                "task_id": task.id,
                "worker_id": worker.id,
                "execution_timeout": self.config.worker_timeout_seconds
            })),
        };

        // Execute using MCP integration
        let result = self.mcp_integration.execute_tool(request).await
            .map_err(|e| WorkerError::ToolExecutionError(e.to_string()))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        let task_result = TaskResult {
            task_id: task.id,
            status: match result.status {
                agent_mcp::ExecutionStatus::Completed => TaskStatus::Completed,
                agent_mcp::ExecutionStatus::Failed => TaskStatus::Failed,
                agent_mcp::ExecutionStatus::Timeout => TaskStatus::Failed,
                _ => TaskStatus::Failed,
            },
            output: result.output,
            error_message: result.error,
            execution_time_ms: execution_time,
            tool_used: tool_id.clone(),
            quality_score: None, // Would be calculated by quality validator
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
            name if name.contains("file") || name.contains("write") || name.contains("read") =>
                worker.specialty == WorkerSpecialty::FileEditing,
            name if name.contains("research") || name.contains("search") =>
                worker.specialty == WorkerSpecialty::Research,
            name if name.contains("code") || name.contains("generate") =>
                worker.specialty == WorkerSpecialty::CodeGeneration,
            _ => worker.specialty == WorkerSpecialty::General,
        }
    }

    /// Validate task requirements before execution
    async fn validate_task_requirements(&self, task: &TaskDefinition) -> Result<(), WorkerError> {
        // Check if required tools are available in MCP registry
        let available_tools = self.mcp_integration.list_tools().await;
        let available_tool_names: std::collections::HashSet<_> = available_tools.iter()
            .map(|t| t.name.as_str())
            .collect();

        for tool_id in &task.required_tools {
            if !available_tool_names.contains(tool_id.as_str()) {
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

// Factory functions moved to lib.rs due to async requirements
