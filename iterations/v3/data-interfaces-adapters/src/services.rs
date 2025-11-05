//! Service Initialization for Binaries
//!
//! Provides helper functions to initialize service adapters for use in binaries.
//! This centralizes dependency injection setup.

use std::sync::Arc;
use data_interfaces::service_contracts::{
    ResearchService, OrchestrationService, WorkerService,
    ProgressTrackingService, MemoryService,
};

// Re-export adapters
pub use super::research_adapter::ResearchServiceAdapter;
pub use super::orchestration_adapter::OrchestrationServiceAdapter;
pub use super::worker_adapter::WorkerServiceAdapter;
pub use super::progress_adapter::ProgressTrackingServiceAdapter;
pub use super::memory_adapter::MemoryServiceAdapter;

/// Service container holding all initialized services
pub struct ServiceContainer {
    pub research_service: Arc<dyn ResearchService>,
    pub orchestration_service: Arc<dyn OrchestrationService>,
    pub worker_service: Arc<dyn WorkerService>,
    pub progress_service: Arc<dyn ProgressTrackingService>,
    pub memory_service: Option<Arc<dyn MemoryService>>,
}

impl ServiceContainer {
    /// Create a new service container with default adapters
    pub fn new() -> Self {
        Self {
            research_service: Arc::new(ResearchServiceAdapter::with_defaults()),
            orchestration_service: Arc::new(OrchestrationServiceAdapter::with_defaults()),
            worker_service: Arc::new(WorkerServiceAdapter::new()),
            progress_service: Arc::new(ProgressTrackingServiceAdapter::new()),
            memory_service: None, // Memory service requires database connection
        }
    }
    
    /// Create with custom services (for testing or advanced usage)
    pub fn with_services(
        research: Arc<dyn ResearchService>,
        orchestration: Arc<dyn OrchestrationService>,
        worker: Arc<dyn WorkerService>,
        progress: Arc<dyn ProgressTrackingService>,
        memory: Option<Arc<dyn MemoryService>>,
    ) -> Self {
        Self {
            research_service: research,
            orchestration_service: orchestration,
            worker_service: worker,
            progress_service: progress,
            memory_service: memory,
        }
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

