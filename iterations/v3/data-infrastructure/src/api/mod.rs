//! Decomposed REST API Interface
//!
//! This module provides a clean, modular REST API interface that has been
//! decomposed from the original monolithic api.rs file into focused modules:
//!
//! - `api_types.rs`: All request/response structs and configuration types
//! - `api_errors.rs`: API error types and Axum response conversions
//! - `middleware.rs`: Authentication and request processing middleware
//! - `server.rs`: Main RestApi server struct and business logic methods
//! - `handlers/`: Modular HTTP endpoint handler functions
//!
//! The public interface remains compatible with the original monolithic version.

pub mod api_types;
pub mod api_errors;
pub mod middleware;
pub mod server;
#[cfg(feature = "orchestration")]
pub mod handlers;
pub mod types;
pub mod transform;
pub mod health;

// Re-export public types and functions for backward compatibility
pub use api_types::*;
pub use api_errors::{ApiError, Result};
pub use middleware::*;

// Conditionally re-export server types if orchestration feature is enabled
#[cfg(feature = "orchestration")]
pub use server::{RestApi, ApiState};

// Re-export commonly used handler functions (only available with orchestration feature)
#[cfg(feature = "orchestration")]
#[allow(ambiguous_glob_reexports)]
pub use handlers::*;
