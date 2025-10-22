//! Worker management and execution

pub mod specialization;
pub mod lifecycle;
pub mod context;

pub use specialization::*;
pub use lifecycle::*;
pub use context::*;

// Re-export types from types module that are used in worker management
pub use crate::types::WorkerHandle;
