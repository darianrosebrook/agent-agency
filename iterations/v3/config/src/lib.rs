//! Agent Agency V3 - Centralized Configuration Management
//!
//! Provides secure, validated configuration management with support for:
//! - Environment-based configuration
//! - Secrets management with encryption
//! - Configuration validation and hot-reloading
//! - Multi-environment support (dev, staging, production)

pub mod config;
pub mod secrets;
pub mod validation;
pub mod loader;
pub mod environment;

#[cfg(test)]
mod tests;

pub use config::*;
pub use secrets::*;
pub use validation::*;
pub use loader::*;
pub use environment::*;

/// Re-export commonly used types
pub use serde::{Deserialize, Serialize};
pub use anyhow::Result;
