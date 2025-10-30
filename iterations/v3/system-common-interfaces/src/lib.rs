//! System Common Interfaces
//!
//! This crate provides common interfaces and types that are shared across multiple
//! system crates without creating circular dependencies. All interfaces are designed
//! to be dependency-injection friendly, allowing concrete implementations to be
//! provided at runtime.
//!
//! ## Architecture
//!
//! This crate breaks circular dependencies by providing:
//!
//! - **Trait-based interfaces**: Allow dependency injection of implementations
//! - **Common data types**: Shared without implementation details
//! - **Abstracted services**: Database, observability, health checks
//! - **Configuration types**: Shared configuration structures
//!
//! ## Usage Pattern
//!
//! ```rust
//! use system_common_interfaces::{DatabaseInterface, ObservabilityInterface};
//!
//! struct MyService<D: DatabaseInterface, O: ObservabilityInterface> {
//!     database: D,
//!     observability: O,
//! }
//! ```
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod database;
pub mod observability;
pub mod health;
pub mod config;
pub mod types;

pub use database::*;
pub use observability::*;
pub use health::*;
pub use config::*;
pub use types::*;

/// Common result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Service lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ServiceState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

/// Common service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub service_name: String,
    pub service_version: String,
    pub instance_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub environment: String,
}

/// Common error types that can be shared across services
#[derive(thiserror::Error, Debug)]
pub enum SystemError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Common pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order_by: Option<String>,
    pub order_direction: Option<OrderDirection>,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// Common filter structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParams {
    pub filters: HashMap<String, serde_json::Value>,
}

/// Combined pagination and filter parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub pagination: Option<PaginationParams>,
    pub filters: Option<FilterParams>,
}

/// Generic response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub metadata: Option<ResponseMetadata>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub total_count: Option<u64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub has_more: Option<bool>,
}

/// Common audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub service: String,
    pub action: String,
    pub resource: String,
    pub user_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
}
