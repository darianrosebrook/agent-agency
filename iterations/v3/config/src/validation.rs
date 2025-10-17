//! Configuration validation and schema enforcement

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use validator::{Validate, ValidationError as ValidatorError, ValidationErrors};

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Configuration validator
#[derive(Debug, Clone)]
pub struct ConfigValidator {
    rules: HashMap<String, ValidationRule>,
    strict_mode: bool,
}

/// Validation rule for a configuration field
#[derive(Debug)]
pub struct ValidationRule {
    pub field_name: String,
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub custom_validator: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
}

/// Database configuration validation
#[derive(Debug, Clone, Validate)]
pub struct DatabaseConfigValidation {
    #[validate(length(min = 1, message = "Database URL cannot be empty"))]
    pub url: String,
    
    #[validate(range(min = 1, max = 100, message = "Connection pool size must be between 1 and 100"))]
    pub max_connections: u32,
    
    #[validate(range(min = 1, max = 3600, message = "Connection timeout must be between 1 and 3600 seconds"))]
    pub connection_timeout_secs: u64,
    
    #[validate(range(min = 1, max = 3600, message = "Idle timeout must be between 1 and 3600 seconds"))]
    pub idle_timeout_secs: u64,
}

/// Server configuration validation
#[derive(Debug, Clone, Validate)]
pub struct ServerConfigValidation {
    #[validate(range(min = 1, max = 65535, message = "Port must be between 1 and 65535"))]
    pub port: u16,
    
    #[validate(length(min = 1, message = "Host cannot be empty"))]
    pub host: String,
    
    #[validate(range(min = 1, max = 3600, message = "Request timeout must be between 1 and 3600 seconds"))]
    pub request_timeout_secs: u64,
    
    #[validate(range(min = 1, max = 1000, message = "Max request size must be between 1 and 1000 MB"))]
    pub max_request_size_mb: u64,
}

/// Agent configuration validation
#[derive(Debug, Clone, Validate)]
pub struct AgentConfigValidation {
    #[validate(range(min = 1, max = 100, message = "Max concurrent tasks must be between 1 and 100"))]
    pub max_concurrent_tasks: u32,
    
    #[validate(range(min = 1, max = 3600, message = "Task timeout must be between 1 and 3600 seconds"))]
    pub task_timeout_secs: u64,
    
    #[validate(range(min = 1, max = 100, message = "Retry attempts must be between 1 and 100"))]
    pub max_retry_attempts: u32,
    
    #[validate(range(min = 1, max = 60, message = "Retry delay must be between 1 and 60 seconds"))]
    pub retry_delay_secs: u64,
}

/// Logging configuration validation
#[derive(Debug, Clone, Validate)]
pub struct LoggingConfigValidation {
    #[validate(custom = "validate_log_level")]
    pub level: String,
    
    #[validate(length(min = 1, message = "Log format cannot be empty"))]
    pub format: String,
    
    #[validate(range(min = 1, max = 100, message = "Max file size must be between 1 and 100 MB"))]
    pub max_file_size_mb: u64,
    
    #[validate(range(min = 1, max = 30, message = "Max files must be between 1 and 30"))]
    pub max_files: u32,
}

/// Metrics configuration validation
#[derive(Debug, Clone, Validate)]
pub struct MetricsConfigValidation {
    #[validate(range(min = 1, max = 3600, message = "Collection interval must be between 1 and 3600 seconds"))]
    pub collection_interval_secs: u64,
    
    #[validate(range(min = 1, max = 1000, message = "Retention days must be between 1 and 1000"))]
    pub retention_days: u32,
    
    #[validate(range(min = 1, max = 100, message = "Batch size must be between 1 and 100"))]
    pub batch_size: u32,
}

/// Security configuration validation
#[derive(Debug, Clone, Validate)]
pub struct SecurityConfigValidation {
    #[validate(length(min = 32, message = "JWT secret must be at least 32 characters"))]
    pub jwt_secret: String,
    
    #[validate(range(min = 300, max = 86400, message = "JWT expiry must be between 300 and 86400 seconds"))]
    pub jwt_expiry_secs: u64,
    
    #[validate(range(min = 1, max = 100, message = "Rate limit must be between 1 and 100 requests per minute"))]
    pub rate_limit_per_minute: u32,
    
    #[validate(range(min = 1, max = 1000, message = "Max login attempts must be between 1 and 1000"))]
    pub max_login_attempts: u32,
}

/// Cache configuration validation
#[derive(Debug, Clone, Validate)]
pub struct CacheConfigValidation {
    #[validate(range(min = 1, max = 3600, message = "TTL must be between 1 and 3600 seconds"))]
    pub default_ttl_secs: u64,
    
    #[validate(range(min = 1, max = 1000, message = "Max size must be between 1 and 1000 MB"))]
    pub max_size_mb: u64,
    
    #[validate(range(min = 1, max = 100, message = "Eviction threshold must be between 1 and 100 percent"))]
    pub eviction_threshold_percent: u8,
}

/// Resource configuration validation
#[derive(Debug, Clone, Validate)]
pub struct ResourceConfigValidation {
    #[validate(range(min = 1, max = 100, message = "CPU limit must be between 1 and 100 percent"))]
    pub cpu_limit_percent: u8,
    
    #[validate(range(min = 1, max = 100, message = "Memory limit must be between 1 and 100 percent"))]
    pub memory_limit_percent: u8,
    
    #[validate(range(min = 1, max = 1000, message = "Disk limit must be between 1 and 1000 GB"))]
    pub disk_limit_gb: u64,
    
    #[validate(range(min = 1, max = 100, message = "Network limit must be between 1 and 100 Mbps"))]
    pub network_limit_mbps: u32,
}

/// Tracing configuration validation
#[derive(Debug, Clone, Validate)]
pub struct TracingConfigValidation {
    #[validate(custom = "validate_trace_level")]
    pub level: String,
    
    #[validate(range(min = 1, max = 100, message = "Sampling rate must be between 1 and 100 percent"))]
    pub sampling_rate_percent: u8,
    
    #[validate(range(min = 1, max = 3600, message = "Export interval must be between 1 and 3600 seconds"))]
    pub export_interval_secs: u64,
}

/// Deployment configuration validation
#[derive(Debug, Clone, Validate)]
pub struct DeploymentConfigValidation {
    #[validate(custom = "validate_environment")]
    pub environment: String,
    
    #[validate(length(min = 1, message = "Version cannot be empty"))]
    pub version: String,
    
    #[validate(custom = "validate_region")]
    pub region: String,
    
    #[validate(range(min = 1, max = 10, message = "Replicas must be between 1 and 10"))]
    pub replicas: u32,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new(strict_mode: bool) -> Self {
        Self {
            rules: HashMap::new(),
            strict_mode,
        }
    }

    /// Add a validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.insert(rule.field_name.clone(), rule);
    }

    /// Validate a configuration value
    pub fn validate_field(&self, field_name: &str, value: &str) -> Result<()> {
        if let Some(rule) = self.rules.get(field_name) {
            if rule.required && value.is_empty() {
                return Err(anyhow!("Field '{}' is required", field_name));
            }

            if let Some(min_len) = rule.min_length {
                if value.len() < min_len {
                    return Err(anyhow!("Field '{}' must be at least {} characters", field_name, min_len));
                }
            }

            if let Some(max_len) = rule.max_length {
                if value.len() > max_len {
                    return Err(anyhow!("Field '{}' must be at most {} characters", field_name, max_len));
                }
            }

            if let Some(pattern) = &rule.pattern {
                let regex = regex::Regex::new(pattern)
                    .map_err(|e| anyhow!("Invalid regex pattern for field '{}': {}", field_name, e))?;
                if !regex.is_match(value) {
                    return Err(anyhow!("Field '{}' does not match required pattern", field_name));
                }
            }

            if let Some(validator) = &rule.custom_validator {
                validator(value)?;
            }
        }

        Ok(())
    }

    /// Validate a complete configuration
    pub fn validate_config<T: Validate>(&self, config: &T) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Use the validator crate for automatic validation
        if let Err(validation_errors) = config.validate() {
            for (field, field_errors) in validation_errors.field_errors() {
                for error in field_errors {
                    errors.push(ValidationError {
                        field: field.to_string(),
                        message: error.message.clone().unwrap_or_else(|| "Validation failed".to_string().into()),
                        code: error.code.clone(),
                    });
                }
            }
        }

        // Add custom validation logic here if needed
        if self.strict_mode && !errors.is_empty() {
            warnings.push("Strict mode enabled - all validation errors must be resolved".to_string());
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Custom validation functions
fn validate_log_level(level: &str) -> Result<(), ValidationError> {
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&level.to_lowercase().as_str()) {
        return Err(ValidationError::new("invalid_log_level"));
    }
    Ok(())
}

fn validate_trace_level(level: &str) -> Result<(), ValidationError> {
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&level.to_lowercase().as_str()) {
        return Err(ValidationError::new("invalid_trace_level"));
    }
    Ok(())
}

fn validate_environment(env: &str) -> Result<(), ValidationError> {
    let valid_envs = ["development", "staging", "production", "test"];
    if !valid_envs.contains(&env.to_lowercase().as_str()) {
        return Err(ValidationError::new("invalid_environment"));
    }
    Ok(())
}

fn validate_region(region: &str) -> Result<(), ValidationError> {
    let valid_regions = ["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"];
    if !valid_regions.contains(&region.to_lowercase().as_str()) {
        return Err(ValidationError::new("invalid_region"));
    }
    Ok(())
}

/// Validation error with additional context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: std::borrow::Cow<'static, str>,
    pub code: std::borrow::Cow<'static, str>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation error in field '{}': {}", self.field, self.message.as_ref())
    }
}

impl std::error::Error for ValidationError {}

/// Configuration validation utilities
pub mod utils {
    use super::*;

    /// Validate a database URL format
    pub fn validate_database_url(url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(anyhow!("Database URL cannot be empty"));
        }

        if !url.starts_with("postgresql://") && !url.starts_with("mysql://") && !url.starts_with("sqlite://") {
            return Err(anyhow!("Database URL must start with postgresql://, mysql://, or sqlite://"));
        }

        Ok(())
    }

    /// Validate a JWT secret strength
    pub fn validate_jwt_secret(secret: &str) -> Result<()> {
        if secret.len() < 32 {
            return Err(anyhow!("JWT secret must be at least 32 characters long"));
        }

        if secret.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(anyhow!("JWT secret should contain special characters for better security"));
        }

        Ok(())
    }

    /// Validate a port number
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            return Err(anyhow!("Port cannot be 0"));
        }

        if port < 1024 && port != 80 && port != 443 {
            return Err(anyhow!("Port {} requires root privileges", port));
        }

        Ok(())
    }

    /// Validate a file path
    pub fn validate_file_path(path: &str) -> Result<()> {
        if path.is_empty() {
            return Err(anyhow!("File path cannot be empty"));
        }

        if path.contains("..") {
            return Err(anyhow!("File path cannot contain '..' for security reasons"));
        }

        Ok(())
    }
}
