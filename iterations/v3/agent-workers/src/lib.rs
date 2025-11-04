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
    WorkerMessage, WorkerProgress, Progress, ValidationResult, ValidationContext,
    Artifact, ArtifactType, WorkerHealth, SeverityLevel, TaskPriority,
    WorkerSpecialty, TaskDefinition, TaskStatus, ExecutionOutcome, LearningMode,
    Priority, WorkerBreakdown, QualityRequirements, ToolId, ValidationRuleType,
    SystemClock, UuidGenerator, Clock, CawsSpec
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
pub mod coordinator;
pub mod decomposition;
pub mod communication;
pub mod progress;
pub mod validation;
pub mod metrics;
pub mod learning;
pub mod worker_errors;
pub mod parallel_types;
pub mod error;
pub mod worker;

// Refactored modules for better organization
pub mod learning_system;
pub mod bridges;
pub mod execution_stats;

// Consolidated from worker/ crate (CLI interface)
pub mod cli;

// Re-export main types from core agent-workers
pub use core::{MCPWorkerPool, WorkerPoolConfig, WorkerHandle};
pub use execution::{ToolExecutor, ExecutionResult};
pub use agent_mcp::ToolRegistry;
pub use worker_types::*;

// Re-export types from consolidated workers crate
pub use autonomous_executor::{AutonomousExecutor, AutonomousExecutorConfig, ExecutionResult as AutoExecutionResult};
pub use caws_checker::CawsChecker;
pub use executor::TaskExecutor;
pub use manager::WorkerPoolManager;
pub use multimodal_scheduler::{MultimodalJobScheduler, MultimodalSchedulerConfig, MultimodalJob};
pub use router::TaskRouter;
pub use specialized_workers::{CompilationSpecialist, RefactoringSpecialist, TestingSpecialist, DocumentationSpecialist};

// Re-export types from consolidated parallel-workers crate
pub use coordinator::{ParallelCoordinator, ParallelCoordinatorConfig};
pub use decomposition::{DecompositionEngine, TaskAnalysis, TaskPattern};
pub use communication::hub::CommunicationHub;
pub use progress::{WorkerProgressTracker};
pub use validation::{QualityValidatorTrait, QualityGate};
pub use worker_errors::*;

// Re-export contract traits for compatibility
pub use agent_agency_contracts::task_executor::{TaskExecutor as TaskExecutorTrait, TaskExecutionResult};

// Factory functions (async due to memory initialization)

/// Create a new MCP-based worker pool with default configuration
pub async fn new_worker_pool() -> MCPWorkerPool {
    MCPWorkerPool::new(WorkerPoolConfig::default()).await
}

/// Create a worker pool with custom MCP tool registry and shared memory
pub async fn new_worker_pool_with_registry(tool_registry: std::sync::Arc<agent_mcp::ToolRegistry>) -> MCPWorkerPool {
    // Initialize shared memory system for the worker pool
    let memory_config = agent_memory::MemoryConfig::default();
    let shared_memory = std::sync::Arc::new(agent_memory::MemorySystem::init(memory_config).await.unwrap());

    MCPWorkerPool::new_with_registry(WorkerPoolConfig::default(), tool_registry, shared_memory)
}

/// Create a parallel coordinator for complex task decomposition
pub fn new_parallel_coordinator() -> ParallelCoordinator {
    // TODO: Implement coordinator creation
    todo!("Implement coordinator creation")
}

/// Create a parallel coordinator with custom configuration
pub fn new_parallel_coordinator_with_config(_config: ParallelCoordinatorConfig) -> ParallelCoordinator {
    // TODO: Implement coordinator creation with config
    todo!("Implement coordinator creation with config")
}

/// Create a task executor that implements the TaskExecutor trait
pub fn create_task_executor() -> std::sync::Arc<dyn TaskExecutorTrait> {
    std::sync::Arc::new(executor::TaskExecutor::new())
}

/// Create a factory function for TaskExecutorProvider
pub fn task_executor_factory() -> agent_agency_contracts::task_executor_provider::TaskExecutorFactory {
    create_task_executor
}
