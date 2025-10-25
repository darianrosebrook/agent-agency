//! Modular production observability system
//!
//! This module provides decomposed observability functionality,
//! organized by domain responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod metrics;
pub mod health;
pub mod logging;
pub mod quantiles;

// Re-export all types for convenient access
pub use core::*;
pub use metrics::*;
pub use health::*;
pub use logging::*;
pub use quantiles::*;

// Re-export common types that might be used externally
pub use core::{ObservabilityConfig, HealthStatus, HealthCheckResult, LogEntry, LogLevel};
pub use metrics::{MetricType, MetricValue, MetricDataPoint, MetricsCollector};
pub use quantiles::{QuantileConfig, QuantileAlgorithm, QuantileEstimator};
