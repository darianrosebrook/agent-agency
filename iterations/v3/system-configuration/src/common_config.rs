//! Common configuration abstractions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common trait for all configuration objects
pub trait Config: Default + Clone + Send + Sync {
    /// Validate the configuration
    fn validate(&self) -> Result<(), Vec<String>>;
}

/// Common trait for configurations that can be merged
pub trait MergeableConfig {
    /// Merge another config into this one, with other taking precedence
    fn merge(&mut self, other: &Self);
}

/// Common trait for configurations that support environment overrides
pub trait EnvironmentConfig {
    /// Load configuration from environment variables
    fn from_env() -> Result<Self, Vec<String>> where Self: Sized;

    /// Override configuration with environment variables
    fn override_with_env(&mut self) -> Result<(), Vec<String>>;
}

/// Generic configuration structure with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub log_level: LogLevel,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            name: "agent-agency".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
            log_level: LogLevel::Info,
            timeout_ms: 30000,
            retry_attempts: 3,
            metadata: HashMap::new(),
        }
    }
}

impl Config for CommonConfig {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("name cannot be empty".to_string());
        }

        if self.version.is_empty() {
            errors.push("version cannot be empty".to_string());
        }

        if self.timeout_ms == 0 {
            errors.push("timeout_ms must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl MergeableConfig for CommonConfig {
    fn merge(&mut self, other: &Self) {
        if !other.name.is_empty() {
            self.name = other.name.clone();
        }
        if !other.version.is_empty() {
            self.version = other.version.clone();
        }
        self.environment = other.environment.clone();
        self.log_level = other.log_level.clone();
        if other.timeout_ms > 0 {
            self.timeout_ms = other.timeout_ms;
        }
        self.retry_attempts = other.retry_attempts;

        // Merge metadata
        for (key, value) in &other.metadata {
            self.metadata.insert(key.clone(), value.clone());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Database configuration pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>, // Should be loaded from env
    pub max_connections: u32,
    pub timeout_ms: u64,
    pub ssl_mode: SslMode,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "agent_agency".to_string(),
            username: "postgres".to_string(),
            password: None,
            max_connections: 20,
            timeout_ms: 30000,
            ssl_mode: SslMode::Prefer,
        }
    }
}

impl Config for DatabaseConfig {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.host.is_empty() {
            errors.push("host cannot be empty".to_string());
        }

        if self.database.is_empty() {
            errors.push("database cannot be empty".to_string());
        }

        if self.username.is_empty() {
            errors.push("username cannot be empty".to_string());
        }

        if self.max_connections == 0 {
            errors.push("max_connections must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

/// Circuit breaker configuration pattern - base common config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout_ms: u64,
    pub success_threshold: u32,
    pub timeout_ms: u64,
    pub max_concurrent_requests: u32,
}

/// Extended circuit breaker config for advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedCircuitBreakerConfig {
    pub base: CircuitBreakerConfig,
    pub name: Option<String>,
    pub failure_window_ms: Option<u64>,
    pub reset_timeout_ms: Option<u64>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_ms: 60000, // 1 minute
            success_threshold: 3,
            timeout_ms: 30000, // 30 seconds
            max_concurrent_requests: 10,
        }
    }
}

impl Default for ExtendedCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            base: CircuitBreakerConfig::default(),
            name: None,
            failure_window_ms: Some(60000), // 1 minute
            reset_timeout_ms: Some(60000), // 1 minute
        }
    }
}

impl Config for CircuitBreakerConfig {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.failure_threshold == 0 {
            errors.push("failure_threshold must be greater than 0".to_string());
        }

        if self.success_threshold == 0 {
            errors.push("success_threshold must be greater than 0".to_string());
        }

        if self.max_concurrent_requests == 0 {
            errors.push("max_concurrent_requests must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ExtendedCircuitBreakerConfig {
    /// Convert from a basic CircuitBreakerConfig
    pub fn from_base(config: CircuitBreakerConfig) -> Self {
        Self {
            base: config,
            name: None,
            failure_window_ms: None,
            reset_timeout_ms: None,
        }
    }

    /// Convert to basic CircuitBreakerConfig
    pub fn to_base(self) -> CircuitBreakerConfig {
        self.base
    }
}

// Conversion implementations for easier migration
impl From<CircuitBreakerConfig> for ExtendedCircuitBreakerConfig {
    fn from(config: CircuitBreakerConfig) -> Self {
        ExtendedCircuitBreakerConfig::from_base(config)
    }
}

impl From<ExtendedCircuitBreakerConfig> for CircuitBreakerConfig {
    fn from(config: ExtendedCircuitBreakerConfig) -> Self {
        config.to_base()
    }
}
