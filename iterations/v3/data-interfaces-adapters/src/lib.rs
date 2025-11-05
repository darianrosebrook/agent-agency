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

pub use research_adapter::ResearchServiceAdapter;
pub use orchestration_adapter::OrchestrationServiceAdapter;
pub use worker_adapter::WorkerServiceAdapter;
pub use progress_adapter::ProgressTrackingServiceAdapter;
pub use memory_adapter::MemoryServiceAdapter;
pub use services::ServiceContainer;


