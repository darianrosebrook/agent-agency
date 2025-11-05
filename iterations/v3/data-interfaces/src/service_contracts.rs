//! Service Interface Contracts
//!
//! Defines trait interfaces for agent services that can be injected into
//! data-interfaces layer. This enables dependency injection and removes
//! direct dependencies on implementation crates.

use async_trait::async_trait;
use agent_agency_contracts::{
    TaskRequest, TaskResponse, WorkingSpec, TaskExecutionResult,
    TaskSpec, TaskRequirements, TaskContext as ContractsTaskContext,
};
use std::sync::Arc;
use uuid::Uuid;

/// Research service for task planning and execution
#[async_trait]
pub trait ResearchService: Send + Sync {
    /// Execute a task request and return response
    async fn execute_task(&self, request: TaskRequest) -> Result<TaskResponse, ServiceError>;
    
    /// Generate a working specification from a task request
    async fn generate_working_spec(&self, request: &TaskRequest) -> Result<WorkingSpec, ServiceError>;
    
    /// Refine a working specification based on validation issues
    async fn refine_working_spec(
        &self,
        spec: &mut WorkingSpec,
        validation_issues: &[agent_agency_contracts::types::validation::ValidationIssue],
    ) -> Result<(), ServiceError>;
}

/// Orchestration service for task orchestration and coordination
#[async_trait]
pub trait OrchestrationService: Send + Sync {
    /// Orchestrate a task execution
    async fn orchestrate_task(
        &self,
        spec: WorkingSpec,
        context: ContractsTaskContext,
    ) -> Result<TaskExecutionResult, ServiceError>;
    
    /// Get task execution status
    async fn get_task_status(&self, task_id: &uuid::Uuid) -> Result<TaskStatus, ServiceError>;
    
    /// Pause a running task
    async fn pause_task(&self, task_id: &uuid::Uuid) -> Result<(), ServiceError>;
    
    /// Resume a paused task
    async fn resume_task(&self, task_id: &uuid::Uuid) -> Result<(), ServiceError>;
    
    /// Cancel a task
    async fn cancel_task(&self, task_id: &uuid::Uuid) -> Result<(), ServiceError>;
}

/// Worker service for worker pool management
#[async_trait]
pub trait WorkerService: Send + Sync {
    /// Execute a worker task
    async fn execute_worker_task(
        &self,
        spec: TaskSpec,
        requirements: TaskRequirements,
    ) -> Result<TaskExecutionResult, ServiceError>;
    
    /// Get worker pool status
    async fn get_worker_status(&self) -> Result<WorkerPoolStatus, ServiceError>;
    
    /// Register a worker
    async fn register_worker(
        &self,
        registration: WorkerRegistration,
    ) -> Result<(), ServiceError>;
}

/// Progress tracking service
#[async_trait]
pub trait ProgressTrackingService: Send + Sync {
    /// Track task progress
    async fn track_progress(
        &self,
        task_id: &uuid::Uuid,
        progress: ProgressUpdate,
    ) -> Result<(), ServiceError>;
    
    /// Get progress for a task
    async fn get_progress(&self, task_id: &uuid::Uuid) -> Result<ProgressInfo, ServiceError>;
    
    /// Subscribe to progress updates
    async fn subscribe_progress(
        &self,
        task_id: &uuid::Uuid,
    ) -> Result<ProgressStream, ServiceError>;
}

/// Memory service for agent memory operations
#[async_trait]
pub trait MemoryService: Send + Sync {
    /// Store memory
    async fn store_memory(
        &self,
        memory_type: agent_agency_contracts::types::memory::MemoryType,
        content: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<agent_agency_contracts::types::memory::MemoryId, ServiceError>;
    
    /// Retrieve memory
    async fn retrieve_memory(
        &self,
        memory_id: &agent_agency_contracts::types::memory::MemoryId,
    ) -> Result<MemoryContent, ServiceError>;
    
    /// Query memories
    async fn query_memories(
        &self,
        query: MemoryQuery,
    ) -> Result<Vec<MemoryContent>, ServiceError>;
}

// Type definitions for service responses

/// Task execution status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskStatus {
    pub task_id: uuid::Uuid,
    pub status: TaskStatusEnum,
    pub progress_percent: Option<u8>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TaskStatusEnum {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Worker pool status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerPoolStatus {
    pub total_workers: usize,
    pub active_workers: usize,
    pub idle_workers: usize,
    pub health_status: String,
}

/// Worker registration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerRegistration {
    pub worker_id: uuid::Uuid,
    pub capabilities: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Progress update
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressUpdate {
    pub task_id: uuid::Uuid,
    pub progress_percent: u8,
    pub status_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Progress information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressInfo {
    pub task_id: uuid::Uuid,
    pub progress_percent: u8,
    pub current_stage: String,
    pub status_message: Option<String>,
}

/// Progress stream (simplified - in real implementation would be a stream)
pub type ProgressStream = tokio::sync::mpsc::UnboundedReceiver<ProgressInfo>;

/// Memory content
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryContent {
    pub memory_id: agent_agency_contracts::types::memory::MemoryId,
    pub memory_type: agent_agency_contracts::types::memory::MemoryType,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Memory query
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryQuery {
    pub memory_type: Option<agent_agency_contracts::types::memory::MemoryType>,
    pub query_text: Option<String>,
    pub limit: Option<usize>,
}

/// Service error type
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    
    #[error("Task not found: {0}")]
    TaskNotFound(uuid::Uuid),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Service error: {0}")]
    Internal(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

