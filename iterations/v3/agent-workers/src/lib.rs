//! Agent Workers - Unified MCP-Based Task Execution System
//!
//! A comprehensive worker orchestration system that consolidates:
//! - **agent-workers**: MCP-based task execution
//! - **workers**: Specialized worker types and routing
//! - **parallel-workers**: Parallel task decomposition and coordination
//! - **worker**: CLI application interface
//!
//! ## Architecture
//!
//! The unified system provides:
//! - **MCP Tool Integration**: Tool-based execution instead of hardcoded logic
//! - **Parallel Processing**: Task decomposition and coordinated execution
//! - **Specialized Workers**: Domain-specific execution capabilities
//! - **Quality Gates**: CAWS compliance and validation throughout
//! - **Intelligent Routing**: Capability-based task distribution
//!
//! ## Key Components
//!
//! - **MCPWorkerPool**: Main orchestration with MCP tool integration
//! - **ParallelCoordinator**: Parallel task decomposition and execution
//! - **Specialized Workers**: Compilation, refactoring, testing, documentation
//! - **TaskRouter**: Intelligent routing based on capabilities
//! - **QualityValidator**: CAWS compliance checking

#![allow(warnings)]
#![allow(dead_code)]

// Core MCP-based worker modules
pub mod core;
pub mod execution;
pub mod mcp_integration;
pub mod services;
pub mod worker_types;

// Re-export commonly used types
pub use worker_types::{
    Artifact, ArtifactType, CawsSpec, Clock, ExecutionOutcome, LearningMode, Priority, Progress,
    QualityRequirements, SeverityLevel, SubTaskId, SystemClock, TaskDefinition, TaskId,
    TaskPriority, TaskScope, TaskStatus, ToolId, UuidGenerator, ValidationContext,
    ValidationResult, ValidationRuleType, WorkerHealth, WorkerId, WorkerMessage, WorkerProgress,
};

// Re-export types from parallel_types
pub use parallel_types::{
    ComplexTask, CoordinationStrategy, DecompositionStrategy, Dependency, ErrorGroup,
    ParallelError, ParallelExecutionPlan, ParallelResult, RefactoringOperation, SubTask,
    SubTaskStatus, SubtaskScores, TaskAnalysis, TaskDependency, TaskPattern, TaskResult,
    WorkerBreakdown, WorkerMetrics, WorkerResult, WorkerSpecialty,
};

// Consolidated from workers/ crate
pub mod autonomous_executor;
pub mod caws_checker;
pub mod executor;
pub mod manager;
pub mod multimodal_scheduler;
pub mod router;
pub mod specialized_workers;

// Consolidated from parallel-workers/ crate
pub mod communication;
pub mod coordinator;
pub mod decomposition;
pub mod error;
pub mod learning;
pub mod metrics;
pub mod parallel_types;
pub mod progress;
pub mod validation;
pub mod worker;
pub mod worker_errors;
pub mod workers_common;

// Refactored modules for better organization
pub mod bridges;
pub mod execution_stats;
pub mod learning_system;

// Consolidated from worker/ crate (CLI interface)
pub mod cli;

// Re-export main types from core agent-workers
pub use agent_mcp::ToolRegistry;
pub use core::{MCPWorkerPool, WorkerHandle, WorkerPoolConfig};
pub use execution::{ExecutionResult, ToolExecutor};
pub use worker_types::*;

// Re-export types from consolidated workers crate
pub use autonomous_executor::{
    AutonomousExecutor, AutonomousExecutorConfig, ExecutionResult as AutoExecutionResult,
};
pub use caws_checker::CawsChecker;
pub use executor::TaskExecutor;
pub use manager::WorkerPoolManager;
pub use multimodal_scheduler::{MultimodalJob, MultimodalJobScheduler, MultimodalSchedulerConfig};
pub use router::TaskRouter;
pub use specialized_workers::{
    CompilationSpecialist, DocumentationSpecialist, RefactoringSpecialist, TestingSpecialist,
};

// Re-export types from consolidated parallel-workers crate
pub use communication::hub::CommunicationHub;
pub use coordinator::{ParallelCoordinator, ParallelCoordinatorConfig};
pub use decomposition::DecompositionEngine;
pub use parallel_types::DependencyType;
pub use progress::WorkerProgressTracker;
pub use validation::{QualityGate, QualityValidatorTrait};
pub use worker::WorkerManager;
pub use worker_errors::*;

// Re-export contract traits for compatibility
pub use agent_agency_contracts::task_executor::{
    TaskExecutionResult, TaskExecutor as TaskExecutorTrait,
};

// Factory functions (async due to memory initialization)

/// Create a new MCP-based worker pool with default configuration
pub async fn new_worker_pool() -> MCPWorkerPool {
    MCPWorkerPool::new(WorkerPoolConfig::default()).await
}

/// Create a worker pool with custom MCP tool registry and shared memory
pub async fn new_worker_pool_with_registry(
    tool_registry: std::sync::Arc<agent_mcp::ToolRegistry>,
) -> MCPWorkerPool {
    // Initialize shared memory system for the worker pool
    let memory_config = agent_memory::MemoryConfig::default();
    let shared_memory = std::sync::Arc::new(
        agent_memory::MemorySystem::init(memory_config)
            .await
            .unwrap(),
    );

    MCPWorkerPool::new_with_registry(WorkerPoolConfig::default(), tool_registry, shared_memory)
}

/// Create a ToolRegistry with real FileOperationsService
///
/// This creates a fully functional ToolRegistry with all file editing tools
/// registered and ready to use. The FileOperationsService is created from
/// data-infrastructure and uses the provided repository path.
pub async fn create_tool_registry_with_file_ops(
    repo_path: std::path::PathBuf,
) -> anyhow::Result<std::sync::Arc<agent_mcp::ToolRegistry>> {
    use agent_mcp::ToolRegistry;
    use data_infrastructure::file_operations_service::create_file_operations_service;

    let file_ops = create_file_operations_service(repo_path);
    let tool_registry = std::sync::Arc::new(ToolRegistry::with_file_ops(file_ops));

    // Initialize the tool registry to register all available tools
    tool_registry
        .initialize()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize tool registry: {}", e))?;

    Ok(tool_registry)
}

/// Create a parallel coordinator for complex task decomposition
pub fn new_parallel_coordinator() -> ParallelCoordinator {
    let config = ParallelCoordinatorConfig::default();
    ParallelCoordinator::new(config)
}

/// Create a parallel coordinator with custom configuration
pub fn new_parallel_coordinator_with_config(
    config: ParallelCoordinatorConfig,
) -> ParallelCoordinator {
    ParallelCoordinator::new(config)
}

/// Create a task executor that implements the TaskExecutor trait
pub fn create_task_executor() -> std::sync::Arc<dyn TaskExecutorTrait> {
    // Create default database client for task executor
    let db_config = data_infrastructure::DatabaseConfig::default();
    let db_client = std::sync::Arc::new(tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            data_infrastructure::DatabaseClient::new(db_config)
                .await
                .unwrap()
        })
    }));
    std::sync::Arc::new(executor::TaskExecutor::new(db_client))
}

/// Create a factory function for TaskExecutorProvider
pub fn task_executor_factory() -> agent_agency_contracts::task_executor_provider::TaskExecutorFactory
{
    create_task_executor
}
