//! Error types for agent-workers
//!
//! This module re-exports error types from other modules for convenience.

pub use crate::worker_errors::{
    CommunicationError, CommunicationResult, DecompositionError, DecompositionResult,
    ParallelError, ProgressError, ProgressResult, ValidationError, WorkerError,
};
