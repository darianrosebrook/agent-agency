//! Telemetry and performance monitoring
//!
//! Comprehensive performance monitoring and metrics collection
//! for acceleration backends and model inference operations.

pub mod telemetry;
pub mod enhanced_telemetry;

// Re-export main types for convenience
pub use telemetry::{TelemetryCollector, CoreMLMetrics, FailureMode};
pub use enhanced_telemetry::*;
