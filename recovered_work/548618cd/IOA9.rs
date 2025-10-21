//! Error types for the observability crate

use std::fmt;

/// Main error type for observability operations
#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    #[error("Analytics error: {0}")]
    AnalyticsError(String),

    #[error("Metrics error: {0}")]
    MetricsError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Dashboard error: {0}")]
    DashboardError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

// Note: From implementation removed to avoid conflicts
