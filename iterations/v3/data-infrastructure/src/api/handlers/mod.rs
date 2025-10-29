//! API handlers module
//! 
//! This module contains all API handlers organized by functionality.
//! Each submodule focuses on a specific domain to improve maintainability.

pub mod waiver_management;
pub mod slo_management;
pub mod provenance_management;
pub mod task_management;
pub mod query_management;
pub mod system_monitoring;

// Re-export all handlers for easy access
pub use waiver_management::*;
pub use slo_management::*;
pub use provenance_management::*;
pub use task_management::*;
pub use query_management::*;
pub use system_monitoring::*;
