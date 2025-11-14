//! API handlers module
//!
//! This module contains all API handlers organized by functionality.
//! Each submodule focuses on a specific domain to improve maintainability.

#[cfg(feature = "orchestration")]
pub mod auth_handlers;
#[cfg(feature = "orchestration")]
pub mod chat_handlers;
#[cfg(feature = "orchestration")]
pub mod provenance_management;
#[cfg(feature = "orchestration")]
pub mod query_management;
#[cfg(feature = "orchestration")]
pub mod query_performance;
#[cfg(feature = "orchestration")]
pub mod slo_management;
#[cfg(feature = "orchestration")]
pub mod system_monitoring;
#[cfg(feature = "orchestration")]
pub mod task_management;
#[cfg(feature = "orchestration")]
pub mod waiver_management;

// Re-export all handlers for easy access
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use auth_handlers::*;
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use chat_handlers::*;
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use provenance_management::*;
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use query_management::*;
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use slo_management::*;
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use system_monitoring::*;
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use task_management::*;
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use waiver_management::*;
