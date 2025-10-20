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

// Specific re-exports to avoid ambiguous glob re-exports
pub use config::{AppConfig, AppMetadata, ServerConfig, TlsConfig, DatabaseConfig, SecurityConfig, MonitoringConfig, RedisConfig, PrometheusConfig, StatsDConfig, ComponentConfigs, OrchestrationConfig, CouncilConfig};
pub use environment::{Environment, EnvironmentConfig, EnvironmentManager};
pub use loader::{ConfigLoader, ConfigWatcher, ConfigSource, ConfigLoadResult, ConfigLoaderBuilder, MergeStrategy};
pub use secrets::{SecretManager, SecretMetadata, SecretValue};
pub use validation::{ValidationError, ValidationResult, validate_config};

pub use anyhow::Result;
/// Re-export commonly used types
pub use serde::{Deserialize, Serialize};
