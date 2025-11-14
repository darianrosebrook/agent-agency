//! Telemetry and performance monitoring
//!
//! Comprehensive performance monitoring and metrics collection
//! for acceleration backends and model inference operations.

pub mod enhanced_telemetry;
pub mod telemetry;

// Re-export main types for convenience
pub use enhanced_telemetry::*;
pub use telemetry::{CoreMLMetrics, FailureMode, TelemetryCollector};
