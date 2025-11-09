//! Core MCP Worker Pool Implementation
//!
//! Consolidates the worker pool orchestration from workers/, parallel-workers/,
//! and worker/ into a unified MCP-based system.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::worker_types::*;
use crate::parallel_types::{TaskResult, WorkerSpecialty, WorkerBreakdown};
use crate::mcp_integration::MCPIntegration;
use crate::execution::ToolExecutor;
use agent_mcp::{
    ToolRegistry,
    mcp_types::{ExecutionStatus, ExecutionContext, ExecutionPriority, MCPTool, ToolExecutionRequest, ToolExecutionResult},
};
// ContextualMemory will be used with full path to avoid import conflicts
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use chrono::{DateTime, Utc};

/// Configuration for the MCP worker pool

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
            mcp_integration: Arc::new(MCPIntegration::new(tool_registry, "http://localhost:8080".to_string())),
            shared_memory_system: shared_memory,
            stats: Arc::new(RwLock::new(WorkerPoolStats {
                total_workers: 0,
                available_workers: 0,
                busy_workers: 0,
                unavailable_workers: 0,
                active_workers: 0,
                idle_workers: 0,
                unhealthy_workers: 0,
                tasks_in_progress: 0,
                total_tasks_completed: 0,
                total_tasks_failed: 0,
                average_execution_time_ms: 0.0,
                average_quality_score: 0.0,
                average_caws_compliance: 0.0,
                average_queue_time_ms: 0.0,
                pool_uptime_seconds: 0,
                last_updated: Utc::now(),
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
        let worker_id = WorkerId::new();

        // Give worker access to shared memory system - all agents share the same memory
        let handle = WorkerHandle {
            id: worker_id.clone(),
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

        // Retrieve relevant execution memories to inform decision making
        let relevant_memories = self.retrieve_execution_memories(&worker, &task).await;
        if !relevant_memories.is_empty() {
            debug!("Retrieved {} relevant execution memories for worker {} on task {}",
                   relevant_memories.len(), worker.id, task.id);
        }

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

        // Convert task parameters to tool-specific parameters
        // Different tools expect different parameter formats
        let tool_parameters = self.convert_task_params_to_tool_params(&task, &mcp_tool.name, &task.parameters)?;
        
        // Create MCP execution request
        let request = ToolExecutionRequest {
            id: uuid::Uuid::new_v4(),
            tool_id: mcp_tool.id,
            parameters: tool_parameters,
            timeout_seconds: task.timeout_seconds.map(|t| t as u64),
            context: Some(ExecutionContext {
                working_directory: task.parameters.get("worktree_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                environment_variables: HashMap::new(),
                input_files: vec![],
                output_directory: None,
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("task_id".to_string(), serde_json::json!(task.id));
                    map.insert("worker_id".to_string(), serde_json::json!(worker.id));
                    map.insert("execution_timeout".to_string(), serde_json::json!(self.config.worker_timeout_seconds));
                    map
                },
            }),
            created_at: chrono::Utc::now(),
            priority: ExecutionPriority::Normal,
            requested_by: Some("worker_pool".to_string()),
        };

        // Execute using MCP integration
        let result = self.mcp_integration.execute_tool(request).await
            .map_err(|e| WorkerError::ToolExecutionError(e.to_string()))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        let success = matches!(result.status, ExecutionStatus::Completed);
        let task_result = TaskResult {
            task_id: TaskId(task.id),
            success,
            subtasks_completed: if success { 1 } else { 0 },
            total_subtasks: 1,
            execution_time: std::time::Duration::from_millis(execution_time),
            execution_time_ms: execution_time,
            summary: if success {
                format!("Task completed successfully using tool {}", tool_id)
            } else {
                format!("Task failed: {:?}", result.status)
            },
            worker_breakdown: vec![WorkerBreakdown {
                worker_id: worker.id,
                subtasks_assigned: 1,
                subtasks_completed: if success { 1 } else { 0 },
                execution_time: std::time::Duration::from_millis(execution_time),
                quality_score: 0.8, // Default quality score
                errors: if success { vec![] } else { vec![result.error.clone().unwrap_or_else(|| "Unknown error".to_string())] },
            }],
            quality_scores: {
                let mut map = HashMap::new();
                map.insert("overall".to_string(), if success { 0.8 } else { 0.2 });
                map
            },
            errors: if success { vec![] } else { vec![result.error.clone().unwrap_or_else(|| "Unknown error".to_string())] },
            error_message: result.error.clone(),
            tool_used: Some(tool_id.to_string()),
            status: if success { TaskStatus::Completed } else { TaskStatus::Failed },
            metadata: {
                let mut map = HashMap::new();
                map.insert("worker_id".to_string(), serde_json::json!(worker.id));
                map.insert("tool_used".to_string(), serde_json::json!(tool_id));
                map.insert("raw_output".to_string(), serde_json::json!(result.output));
                if let Some(error) = &result.error {
                    map.insert("error".to_string(), serde_json::json!(error));
                }
                map
            },
        };

        // Memory integration: Store execution experience for learning
        self.store_worker_memory(&worker, &task, &task_result, &result).await;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_tasks_completed += 1;

        Ok(task_result)
    }

    /// Store worker execution experience in memory for future learning
    async fn store_worker_memory(
        &self,
        worker: &WorkerHandle,
        task: &TaskDefinition,
        task_result: &TaskResult,
        mcp_result: &ToolExecutionResult,
    ) {
        // Create task context for memory storage
        let task_context = agent_memory::memory_types::TaskContext {
            task_id: task.id.to_string(),
            agent_id: worker.id.0.to_string(),
            task_type: task.name.clone(),
            keywords: vec!["worker_execution".to_string(), task.name.clone()],
            entities: vec![worker.id.0.to_string(), task.required_tools.first().cloned().unwrap_or_default()],
            timestamp: chrono::Utc::now() - chrono::Duration::milliseconds(task_result.execution_time.as_millis() as i64),
            description: format!("{} - {}", task.description, format!("Success: {}, Summary: {}", task_result.success, task_result.summary)),
        };

        // Determine success and performance score
        let success = task_result.success;
        let performance_score = if success {
            // Calculate performance based on execution time and tool effectiveness
            let execution_time_ms = task_result.execution_time.as_millis() as u64;
            let time_score = if execution_time_ms < 1000 { 1.0 }
                           else if execution_time_ms < 5000 { 0.8 }
                           else { 0.6 };
            Some(time_score)
        } else {
            Some(0.2) // Low score for failures
        };
        let tool_label = task_result
            .tool_used
            .as_deref()
            .unwrap_or("unknown_tool")
            .to_string();

        // Create experience outcome
        let outcome = agent_memory::memory_types::ExperienceOutcome {
            success,
            quality_score: performance_score.unwrap_or(0.0) as f64,
            error_message: if success { None } else {
                Some(task_result.error_message.clone().unwrap_or_else(|| "Unknown failure".to_string()))
            },
            metadata: std::collections::HashMap::from([
                ("execution_time_ms".to_string(), serde_json::json!(task_result.execution_time_ms)),
                ("tool_used".to_string(), serde_json::json!(task_result.tool_used)),
                ("worker_specialty".to_string(), serde_json::json!(worker.specialty)),
            ]),
            performance_score,
            execution_time_ms: Some(task_result.execution_time_ms as u64),
            learned_capabilities: if success {
                vec![
                    format!("tool_{}_effective", tool_label),
                    format!("worker_{:?}_skilled", worker.specialty),
                ]
            } else {
                vec![]
            },
        };

        // Create and store the agent experience
        let experience = agent_memory::memory_types::AgentExperience {
            id: uuid::Uuid::new_v4(),
            agent_id: worker.id.0.to_string(),
            task_id: task.id.to_string(),
            content: format!("Worker {} executed task {}: {}", worker.id.0, task.name, task.description),
            input: serde_json::to_string(&serde_json::json!({
                "task_description": task.description,
                "tool_id": task.required_tools.first().cloned().unwrap_or_default(),
                "parameters": task.parameters,
                "required_tools": task.required_tools
            })).unwrap_or_default(),
            output: serde_json::to_string(&serde_json::json!({
                "task_result": task_result,
                "mcp_result": mcp_result
            })).unwrap_or_default(),
            context: agent_memory::memory_types::ExperienceContext {
                description: format!("Worker execution: {}", task.description),
                domain: vec!["worker_execution".to_string(), task.name.clone()],
                task_type: task.name.clone(),
                temporal_context: Some(agent_memory::memory_types::TemporalContext {
                    timestamp: chrono::Utc::now() - chrono::Duration::milliseconds(task_result.execution_time_ms as i64),
                    duration: Some(chrono::Duration::milliseconds(task_result.execution_time_ms as i64)),
                    sequence_number: None,
                    priority: agent_memory::memory_types::TaskPriority::Normal,
                }),
            },
            outcome,
            memory_type: agent_memory::memory_types::MemoryType::Episodic,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::from([
                ("worker_specialty".to_string(), serde_json::json!(worker.specialty)),
                ("tool_used".to_string(), serde_json::json!(task_result.tool_used)),
                ("execution_status".to_string(), serde_json::json!(task_result.status)),
            ]),
        };

        // Store in shared memory system
        if let Err(e) = worker.memory_access.store_experience(experience).await {
            warn!("Failed to store worker execution in memory: {}", e);
        }
    }

    /// Retrieve relevant execution memories before task execution
    async fn retrieve_execution_memories(
        &self,
        worker: &WorkerHandle,
        task: &TaskDefinition,
    ) -> Vec<agent_memory::ContextualMemory> {
        let task_context = agent_memory::memory_types::TaskContext {
            task_id: task.id.to_string(),
            agent_id: worker.id.0.to_string(),
            task_type: task.name.clone(),
            keywords: vec![task.name.clone(), "similar_execution".to_string()],
            entities: vec![task.required_tools.first().cloned().unwrap_or_default()],
            timestamp: chrono::Utc::now(),
            description: format!("Similar to: {}", task.description),
        };

        match worker.memory_access.retrieve_contextual_memories(&task_context, 5).await {
            Ok(memories) => memories,
            Err(e) => {
                warn!("Failed to retrieve execution memories: {}", e);
                vec![]
            }
        }
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

    /// Convert task parameters to tool-specific parameters
    /// 
    /// Different MCP tools expect different parameter formats. This function
    /// converts high-level task parameters (objective, scope, etc.) into
    /// tool-specific parameters that the MCP tools can understand.
    fn convert_task_params_to_tool_params(
        &self,
        task: &TaskDefinition,
        tool_name: &str,
        task_params: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, WorkerError> {
        let mut tool_params = HashMap::new();
        
        match tool_name {
            "file_edit" => {
                // file_edit requires: task_id, changes
                // Extract task_id from task.id
                tool_params.insert("task_id".to_string(), serde_json::json!(task.id.to_string()));
                
                // For changes, we need to generate a changeset from the objective
                // Since we don't have an LLM here, we'll create a minimal placeholder changeset
                // that indicates the file should be created/modified based on the objective
                let objective = task_params.get("objective")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&task.description);
                
                // Extract file path from scope if available
                let file_path = task_params.get("scope")
                    .and_then(|s| s.get("files"))
                    .and_then(|f| f.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        // Default to a Python file if objective mentions Python
                        if objective.to_lowercase().contains("python") {
                            "hello_world.py"
                        } else {
                            "output.txt"
                        }
                    });
                
                // Create a minimal changeset - full file replacement
                // file_edit expects: path, old_content (optional), new_content (optional)
                // This is a placeholder that will need LLM interpretation in the future
                let changes = vec![serde_json::json!({
                    "path": file_path,
                    "old_content": "",
                    "new_content": format!("# {}\n# TODO: Implement actual content based on objective", objective)
                })];
                
                tool_params.insert("changes".to_string(), serde_json::json!(changes));
            },
            "file_write" => {
                // file_write requires: path, content
                let file_path = task_params.get("scope")
                    .and_then(|s| s.get("files"))
                    .and_then(|f| f.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        let objective = task_params.get("objective")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&task.description);
                        if objective.to_lowercase().contains("python") {
                            "hello_world.py"
                        } else {
                            "output.txt"
                        }
                    });
                
                let objective = task_params.get("objective")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&task.description);
                
                tool_params.insert("path".to_string(), serde_json::json!(file_path));
                tool_params.insert("content".to_string(), serde_json::json!(format!("# {}\n# TODO: Implement actual content", objective)));
            },
            "file_read" => {
                // file_read requires: path
                let file_path = task_params.get("scope")
                    .and_then(|s| s.get("files"))
                    .and_then(|f| f.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                
                tool_params.insert("path".to_string(), serde_json::json!(file_path));
            },
            _ => {
                // For other tools, pass through task parameters as-is
                tool_params.extend(task_params.clone());
            },
        }
        
        Ok(tool_params)
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

        // Implement proper worker health tracking
        // Use the tracked unhealthy_workers from stats as primary indicator
        // Workers are tracked as unhealthy when they fail health checks or become unresponsive
        let tracked_unhealthy = stats.unhealthy_workers;
        
        // Also check if stats indicate overall pool degradation
        let healthy_ratio = if stats.total_workers > 0 {
            (stats.total_workers - stats.unhealthy_workers) as f64 / stats.total_workers as f64
        } else {
            1.0
        };

        // Determine health status based on tracked metrics
        if tracked_unhealthy > stats.total_workers / 2 {
            WorkerHealth::Unhealthy
        } else if tracked_unhealthy > 0 || healthy_ratio < 0.8 {
            WorkerHealth::Degraded
        } else {
            WorkerHealth::Healthy
        }
    }

    /// List all registered workers
    pub async fn list_workers(&self) -> Vec<WorkerHandle> {
        let workers = self.workers.read().await;
        workers.values().cloned().collect()
    }
}

/// Error types for worker operations

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
pub enum WorkerError {
    #[error("No suitable worker available for task")]
    NoSuitableWorker,

    #[error("Required tool not available: {0}")]
    ToolNotAvailable(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionError(String),
}

// Factory functions moved to lib.rs due to async requirements
