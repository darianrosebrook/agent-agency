//! Error types for agent-workers
//!
//! This module re-exports error types from other modules for convenience.

pub use crate::worker_errors::{
    ParallelError, DecompositionError, ValidationError, CommunicationError, ProgressError, WorkerError,
    DecompositionResult, CommunicationResult, ProgressResult
};
