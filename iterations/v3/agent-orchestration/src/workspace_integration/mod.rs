//! Workspace Integration Module
//!
//! Bridges between system-resilience workspace state manager and other agent crates
//! @author @darianrosebrook

#[cfg(feature = "data-processing")]
pub mod file_watcher_bridge;

#[cfg(feature = "memory")]
pub mod embedding_service_adapter;

#[cfg(feature = "data-processing")]
pub use file_watcher_bridge::FileWatcherBridge;

#[cfg(feature = "memory")]
pub use embedding_service_adapter::EmbeddingServiceAdapter;

pub mod unified_workspace_setup;
pub use unified_workspace_setup::UnifiedWorkspaceSetupConfig;

#[cfg(all(feature = "data-processing", feature = "memory"))]
pub use unified_workspace_setup::setup_unified_workspace;

