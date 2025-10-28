//! Core MCP Worker Pool Implementation
//!
//! Consolidates the worker pool orchestration from workers/, parallel-workers/,
//! and worker/ into a unified MCP-based system.

use crate::worker_types::*;
use crate::mcp_integration::MCPIntegration;
use crate::execution::ToolExecutor;
use agent_mcp::{ToolRegistry, ToolExecutionRequest, ExecutionStatus, ExecutionContext, ExecutionPriority};
use agent_memory::memory_types::{TaskContext, AgentExperience, ContextualMemory};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use crate::parallel_types::{WorkerId, TaskResult};
use anyhow::{Context, Result};

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

/// Main MCP-based worker pool with shared memory system and HTTP client
pub struct MCPWorkerPool {
    config: WorkerPoolConfig,
    workers: Arc<RwLock<HashMap<WorkerId, WorkerHandle>>>,
    mcp_integration: Arc<MCPIntegration>,
    stats: Arc<RwLock<WorkerPoolStats>>,
    /// Single shared memory system - all agents access this same instance
    shared_memory_system: Arc<agent_memory::MemorySystem>,
    /// HTTP client for real service calls
    http_client: Client,
}

impl MCPWorkerPool {
    /// Create a new worker pool with an MCP tool registry and HTTP client
    pub fn new_with_registry(config: WorkerPoolConfig, tool_registry: Arc<ToolRegistry>, shared_memory: Arc<agent_memory::MemorySystem>, mcp_server_url: String) -> Self {
        Self {
            config: config.clone(),
            workers: Arc::new(RwLock::new(HashMap::new())),
            mcp_integration: Arc::new(MCPIntegration::new(tool_registry, mcp_server_url)),
            shared_memory_system: shared_memory,
            http_client: Client::new(),
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

        // Default MCP server URL - can be configured via environment
        let mcp_server_url = std::env::var("MCP_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        Self::new_with_registry(config, tool_registry, shared_memory, mcp_server_url)
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

        // Create MCP execution request
        let request = ToolExecutionRequest {
            id: uuid::Uuid::new_v4(),
            tool_id: mcp_tool.id,
            parameters: task.parameters.clone(),
            timeout_seconds: task.timeout_seconds.map(|t| t as u64),
            context: Some(ExecutionContext {
                working_directory: None,
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

        let task_result = TaskResult {
            task_id: task.id,
            status: match result.status {
                ExecutionStatus::Completed => TaskStatus::Completed,
                ExecutionStatus::Failed => TaskStatus::Failed,
                ExecutionStatus::Timeout => TaskStatus::Failed,
                _ => TaskStatus::Failed,
            },
            output: result.output.clone(),
            error_message: result.error.clone(),
            execution_time_ms: execution_time,
            tool_used: tool_id.clone(),
            quality_score: None, // Would be calculated by quality validator
        };

        // Memory integration: Store execution experience for learning
        self.store_worker_memory(&worker, &task, &task_result, &result).await;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_tasks_processed += 1;

        Ok(task_result)
    }

    /// Store worker execution experience in memory for future learning
    async fn store_worker_memory(
        &self,
        worker: &WorkerHandle,
        task: &TaskDefinition,
        task_result: &TaskResult,
        mcp_result: &agent_mcp::ToolExecutionResult,
    ) {
        // Create task context for memory storage
        let task_context = TaskContext {
            task_id: task.id.to_string(),
            task_type: task.name.clone(),
            description: task.description.clone(),
            domain: vec!["worker_execution".to_string(), task.name.clone()],
            entities: vec![worker.id.to_string(), task.required_tools.first().cloned().unwrap_or_default()],
            temporal_context: Some(agent_memory::TemporalContext {
                start_time: chrono::Utc::now() - chrono::Duration::milliseconds(task_result.execution_time_ms as i64),
                deadline: None,
                priority: agent_memory::TaskPriority::Medium,
                recurrence_pattern: None,
            }),
            metadata: std::collections::HashMap::from([
                ("worker_specialty".to_string(), serde_json::json!(worker.specialty)),
                ("tool_used".to_string(), serde_json::json!(task_result.tool_used)),
                ("execution_status".to_string(), serde_json::json!(task_result.status)),
            ]),
        };

        // Determine success and performance score
        let success = matches!(task_result.status, TaskStatus::Completed);
        let performance_score = if success {
            // Calculate performance based on execution time and tool effectiveness
            let time_score = if task_result.execution_time_ms < 1000 { 1.0 }
                           else if task_result.execution_time_ms < 5000 { 0.8 }
                           else { 0.6 };
            Some(time_score)
        } else {
            Some(0.2) // Low score for failures
        };

        // Create experience outcome
        let outcome = agent_memory::ExperienceOutcome {
            success,
            performance_score,
            learned_capabilities: vec![format!("{}_execution", task.name)],
            failure_reasons: if success { vec![] } else {
                vec![task_result.error_message.clone().unwrap_or_else(|| "Unknown failure".to_string())]
            },
            success_factors: if success {
                vec![
                    format!("tool_{}_effective", task_result.tool_used),
                    format!("worker_{:?}_skilled", worker.specialty),
                ]
            } else { vec![] },
            execution_time_ms: Some(task_result.execution_time_ms as i64),
            tokens_used: None, // MCP tools don't track tokens directly
            feedback: Some(agent_memory::AgentFeedback {
                quality_score: performance_score,
                relevance_score: Some(0.9),
                accuracy_score: Some(if success { 0.95 } else { 0.3 }),
                comments: vec![format!("Worker {} executed {}: {:?}", worker.id, task.name, task_result.status)],
                evaluator_id: Some("worker_pool_memory_system".to_string()),
            }),
        };

        // Create and store the agent experience
        let experience = AgentExperience {
            id: uuid::Uuid::new_v4(),
            agent_id: worker.id.to_string(),
            task_id: task.id.to_string(),
            context: task_context,
            input: serde_json::json!({
                "task_description": task.description,
                "tool_id": task.required_tools.first().cloned().unwrap_or_default(),
                "parameters": task.parameters,
                "required_tools": task.required_tools
            }),
            output: serde_json::json!({
                "task_result": task_result,
                "mcp_result": mcp_result
            }),
            outcome,
            memory_type: agent_memory::MemoryType::Episodic,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
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
    ) -> Vec<ContextualMemory> {
        let task_context = TaskContext {
            task_id: task.id.to_string(),
            task_type: task.name.clone(),
            description: format!("Similar to: {}", task.description),
            domain: vec![task.name.clone()],
            entities: vec![task.required_tools.first().cloned().unwrap_or_default()],
            temporal_context: Some(agent_memory::TemporalContext {
                start_time: chrono::Utc::now(),
                deadline: None,
                priority: agent_memory::TaskPriority::Medium,
                recurrence_pattern: None,
            }),
            metadata: std::collections::HashMap::from([
                ("tool_id".to_string(), serde_json::json!(task.required_tools.first().cloned().unwrap_or_default())),
                ("worker_specialty".to_string(), serde_json::json!(worker.specialty)),
            ]),
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

    /// Execute a task using real HTTP calls to external services
    pub async fn execute_task_via_http(&self, task_context: &TaskContext, service_url: &str) -> Result<TaskResult> {
        info!("Executing task via HTTP: {}", task_context.task_id);
        
        let start_time = std::time::Instant::now();
        
        // Create HTTP request payload
        let payload = serde_json::json!({
            "task_id": task_context.task_id,
            "worker_id": task_context.worker_id,
            "context": task_context,
            "timeout_ms": task_context.timeout_ms,
            "metadata": task_context.metadata
        });

        // Execute via HTTP call
        let response = self.http_client
            .post(format!("{}/api/v1/tasks/execute", service_url))
            .json(&payload)
            .timeout(std::time::Duration::from_millis(task_context.timeout_ms))
            .send()
            .await
            .context("Failed to send HTTP request to service")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Service error {}: {}", status, error_text));
        }

        let result: TaskResult = response.json().await
            .context("Failed to parse service response")?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        info!("Task execution completed in {}ms: {}", execution_time, task_context.task_id);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_tasks_processed += 1;
            stats.average_execution_time_ms = (stats.average_execution_time_ms + execution_time as f64) / 2.0;
        }

        Ok(result)
    }

    /// Health check for external services
    pub async fn health_check_service(&self, service_url: &str) -> Result<bool> {
        let url = format!("{}/health", service_url);
        
        match self.http_client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get service statistics via HTTP
    pub async fn get_service_stats(&self, service_url: &str) -> Result<HashMap<String, serde_json::Value>> {
        let url = format!("{}/api/v1/stats", service_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to get service stats")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get stats: {}", response.status()));
        }

        let stats: HashMap<String, serde_json::Value> = response.json().await
            .context("Failed to parse stats response")?;

        Ok(stats)
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
