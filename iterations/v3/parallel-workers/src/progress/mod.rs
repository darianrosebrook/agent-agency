//! Progress tracking and aggregation across parallel workers

pub mod tracker;
pub mod aggregator;
pub mod synthesizer;

pub use tracker::*;
pub use aggregator::*;
pub use synthesizer::*;

// Re-export types from types module that are used in progress tracking
pub use crate::types::{Progress, WorkerProgress};
