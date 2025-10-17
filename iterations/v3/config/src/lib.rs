//! Agent Agency V3 - Centralized Configuration Management
//!
//! Provides secure, validated configuration management with support for:
//! - Environment-based configuration
//! - Secrets management with encryption
//! - Configuration validation and hot-reloading
//! - Multi-environment support (dev, staging, production)

pub mod config;
pub mod environment;
pub mod loader;
pub mod secrets;
pub mod validation;

#[cfg(test)]
mod tests;

pub use config::*;
pub use environment::*;
pub use loader::*;
pub use secrets::*;
pub use validation::*;

pub use anyhow::Result;
/// Re-export commonly used types
pub use serde::{Deserialize, Serialize};
