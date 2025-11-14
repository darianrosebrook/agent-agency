//! Enterprise memory management system for Rust applications
//!
//! This module has been successfully refactored from a monolithic 4954-line file
//! into 11 focused submodules for improved maintainability and performance.
//!
//! Provides comprehensive memory monitoring, object pooling, leak detection,
//! and garbage collection optimization for production workloads.

pub mod allocation;
pub mod allocator;
pub mod cache;
pub mod compaction;
pub mod integration;
pub mod leaks;
pub mod manager;
pub mod metrics;
pub mod monitor;
pub mod pool;
pub mod resources;
pub mod types;

// Global registry for orphaned objects that couldn't be returned to pools
// Used as a fallback when tokio runtime is unavailable
#[cfg(not(target_arch = "wasm32"))]
pub static ORPHANED_OBJECTS: std::sync::Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>> =
    std::sync::Mutex::new(Vec::new());

// All functionality has been moved to focused submodules
// Re-export all public APIs for backward compatibility
pub use allocation::{AllocationRecord, AllocationSite, AllocationSiteTracker};
pub use manager::{MemoryManagementConfig, MemoryManager};
pub use metrics::MemoryPressure;
pub use resources::{HandleInfo, HandleType, ResourceHandle};
pub use types::MemoryLimitConfig;
pub use types::ObjectRef;
pub use types::TaskAllocationStats;
