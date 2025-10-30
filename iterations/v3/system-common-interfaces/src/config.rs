//! Configuration Interface
//!
//! Common configuration interface that provides type-safe configuration
//! loading and validation without creating dependencies on specific config
//! implementations.
//!
//! This allows system-configuration to provide concrete config implementations
//! while other crates can depend on the interface for configuration access.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Result;

/// Configuration source types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConfigSource {
    Environment,
    File,
    Database,
    Remote,
    Default,
}

/// Configuration value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

/// Configuration interface for accessing configuration values
#[async_trait]
pub trait ConfigurationInterface: Send + Sync {
    /// Get a string configuration value
    async fn get_string(&self, key: &str) -> Result<Option<String>>;

    /// Get an integer configuration value
    async fn get_int(&self, key: &str) -> Result<Option<i64>>;

    /// Get a float configuration value
    async fn get_float(&self, key: &str) -> Result<Option<f64>>;

    /// Get a boolean configuration value
    async fn get_bool(&self, key: &str) -> Result<Option<bool>>;

    /// Get a configuration value as a specific type
    async fn get_value<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>>;

    /// Get all configuration values with a prefix
    async fn get_all_with_prefix(&self, prefix: &str) -> Result<HashMap<String, ConfigValue>>;

    /// Set a configuration value
    async fn set_value(&self, key: &str, value: ConfigValue) -> Result<()>;

    /// Check if a configuration key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Watch for configuration changes
    async fn watch(&self, key: &str) -> Result<Box<dyn ConfigWatcher>>;
}

/// Configuration watcher for change notifications
#[async_trait]
pub trait ConfigWatcher: Send + Sync {
    /// Wait for the next configuration change
    async fn next_change(&mut self) -> Result<Option<ConfigChange>>;
}

/// Configuration change notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub key: String,
    pub old_value: Option<ConfigValue>,
    pub new_value: Option<ConfigValue>,
    pub source: ConfigSource,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Configuration validator interface
#[async_trait]
pub trait ConfigurationValidator: Send + Sync {
    /// Validate a configuration value
    async fn validate(&self, key: &str, value: &ConfigValue) -> Result<ValidationResult>;

    /// Get validation rules for a key
    async fn get_rules(&self, key: &str) -> Result<Option<ValidationRules>>;
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validation rules for configuration values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub value_type: ConfigValueType,
    pub required: bool,
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
    pub pattern: Option<String>,
    pub custom_validator: Option<String>,
}

/// Configuration value types for validation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConfigValueType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

/// Configuration loader interface
#[async_trait]
pub trait ConfigurationLoader: Send + Sync {
    /// Load configuration from a source
    async fn load(&self, source: ConfigSource) -> Result<HashMap<String, ConfigValue>>;

    /// Save configuration to a source
    async fn save(&self, source: ConfigSource, config: HashMap<String, ConfigValue>) -> Result<()>;

    /// List available configuration sources
    async fn list_sources(&self) -> Result<Vec<ConfigSourceInfo>>;
}

/// Information about a configuration source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSourceInfo {
    pub source_type: ConfigSource,
    pub name: String,
    pub priority: i32,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub is_readonly: bool,
}

/// Environment-specific configuration profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProfile {
    pub name: String,
    pub description: String,
    pub extends: Option<String>, // Parent profile to inherit from
    pub overrides: HashMap<String, ConfigValue>,
    pub required_keys: Vec<String>,
}

/// Configuration manager interface
#[async_trait]
pub trait ConfigurationManager: Send + Sync {
    /// Load configuration for a specific profile
    async fn load_profile(&self, profile_name: &str) -> Result<HashMap<String, ConfigValue>>;

    /// Validate configuration against profile requirements
    async fn validate_profile(&self, profile_name: &str, config: &HashMap<String, ConfigValue>) -> Result<ValidationResult>;

    /// Merge configurations from multiple sources with precedence
    async fn merge_configs(&self, configs: Vec<HashMap<String, ConfigValue>>, precedence: Vec<ConfigSource>) -> Result<HashMap<String, ConfigValue>>;

    /// Get configuration schema for validation
    async fn get_schema(&self) -> Result<ConfigSchema>;
}

/// Configuration schema for documentation and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub version: String,
    pub profiles: HashMap<String, ConfigProfile>,
    pub global_keys: HashMap<String, ValidationRules>,
    pub environment_overrides: HashMap<String, HashMap<String, ConfigValue>>,
}

/// Common configuration keys used across the system
pub mod keys {
    pub const DATABASE_URL: &str = "database.url";
    pub const DATABASE_POOL_SIZE: &str = "database.pool_size";
    pub const DATABASE_TIMEOUT_MS: &str = "database.timeout_ms";

    pub const OBSERVABILITY_METRICS_ENABLED: &str = "observability.metrics.enabled";
    pub const OBSERVABILITY_TRACING_ENABLED: &str = "observability.tracing.enabled";
    pub const OBSERVABILITY_LOG_LEVEL: &str = "observability.log_level";

    pub const HEALTH_CHECK_INTERVAL_MS: &str = "health.check_interval_ms";
    pub const HEALTH_CHECK_TIMEOUT_MS: &str = "health.check_timeout_ms";

    pub const SERVICE_NAME: &str = "service.name";
    pub const SERVICE_VERSION: &str = "service.version";
    pub const SERVICE_ENVIRONMENT: &str = "service.environment";

    pub const API_HOST: &str = "api.host";
    pub const API_PORT: &str = "api.port";
    pub const API_TIMEOUT_MS: &str = "api.timeout_ms";

    pub const CACHE_TTL_SECONDS: &str = "cache.ttl_seconds";
    pub const CACHE_MAX_SIZE: &str = "cache.max_size";

    pub const SECURITY_JWT_SECRET: &str = "security.jwt_secret";
    pub const SECURITY_SESSION_TIMEOUT_MINUTES: &str = "security.session_timeout_minutes";
    pub const SECURITY_RATE_LIMIT_REQUESTS_PER_MINUTE: &str = "security.rate_limit_requests_per_minute";
}
