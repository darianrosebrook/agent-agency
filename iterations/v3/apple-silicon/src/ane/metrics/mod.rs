//! Performance metrics and monitoring
//!
//! This module provides EWMA-based performance tracking and monitoring
//! for ANE operations with adaptive metrics and outlier detection.

pub mod ewma;

// Re-export commonly used types
pub use ewma::{
    Ewma, PerformanceTracker, PerformanceSummary, MetricAlphas, MetricCounts,
};
