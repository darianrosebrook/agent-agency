//! Data Interfaces Adapters
//!
//! Provides concrete implementations of service trait interfaces defined in
//! `data-interfaces`. This crate bridges the gap between interface contracts
//! and actual implementation crates, enabling dependency injection.

pub mod research_adapter;
pub mod orchestration_adapter;
pub mod worker_adapter;
pub mod progress_adapter;
pub mod memory_adapter;
pub mod services;
pub mod mcp_coreml_executor;
pub mod working_spec_converter;
pub mod unified_orchestrator_task_executor;
pub mod database_operations_adapter;

pub use research_adapter::ResearchServiceAdapter;
pub use orchestration_adapter::OrchestrationServiceAdapter;
pub use worker_adapter::WorkerServiceAdapter;
pub use progress_adapter::ProgressTrackingServiceAdapter;
pub use memory_adapter::MemoryServiceAdapter;
pub use services::ServiceContainer;
pub use mcp_coreml_executor::{create_coreml_executor, wire_coreml_executor_to_mcp_server, RealCoreMLIngestionExecutor};
pub use unified_orchestrator_task_executor::UnifiedOrchestratorTaskExecutor;
pub use database_operations_adapter::DatabaseOperationsAdapter;


