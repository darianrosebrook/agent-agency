//! Environment-specific configuration management

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

/// Environment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Test,
}

impl Environment {
    /// Get environment from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            "test" | "testing" => Ok(Environment::Test),
            _ => Err(anyhow!("Invalid environment: {}", s)),
        }
    }

    /// Get environment as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
            Environment::Test => "test",
        }
    }

    /// Check if environment is production
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Check if environment is development
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    /// Check if environment is staging
    pub fn is_staging(&self) -> bool {
        matches!(self, Environment::Staging)
    }

    /// Check if environment is test
    pub fn is_test(&self) -> bool {
        matches!(self, Environment::Test)
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub environment: Environment,
    pub config: HashMap<String, serde_json::Value>,
    pub overrides: HashMap<String, serde_json::Value>,
}

/// Environment configuration manager
#[derive(Debug, Clone)]
pub struct EnvironmentManager {
    current_environment: Environment,
    configs: HashMap<Environment, EnvironmentConfig>,
    default_config: HashMap<String, serde_json::Value>,
}

impl EnvironmentManager {
    /// Create a new environment manager
    pub fn new(environment: Environment) -> Self {
        Self {
            current_environment: environment,
            configs: HashMap::new(),
            default_config: HashMap::new(),
        }
    }

    /// Set the current environment
    pub fn set_environment(&mut self, environment: Environment) {
        self.current_environment = environment;
        info!("Switched to environment: {}", environment);
    }

    /// Get the current environment
    pub fn get_environment(&self) -> Environment {
        self.current_environment
    }

    /// Load environment-specific configuration
    pub fn load_environment_config(&mut self, environment: Environment, config: HashMap<String, serde_json::Value>) {
        let env_config = EnvironmentConfig {
            environment,
            config: config.clone(),
            overrides: HashMap::new(),
        };
        
        self.configs.insert(environment, env_config);
        info!("Loaded configuration for environment: {}", environment);
    }

    /// Set default configuration
    pub fn set_default_config(&mut self, config: HashMap<String, serde_json::Value>) {
        self.default_config = config;
        info!("Set default configuration");
    }

    /// Get configuration for current environment
    pub fn get_current_config(&self) -> HashMap<String, serde_json::Value> {
        let mut config = self.default_config.clone();
        
        if let Some(env_config) = self.configs.get(&self.current_environment) {
            // Merge environment-specific config
            for (key, value) in &env_config.config {
                config.insert(key.clone(), value.clone());
            }
            
            // Apply overrides
            for (key, value) in &env_config.overrides {
                config.insert(key.clone(), value.clone());
            }
        }
        
        config
    }

    /// Get configuration for specific environment
    pub fn get_environment_config(&self, environment: Environment) -> HashMap<String, serde_json::Value> {
        let mut config = self.default_config.clone();
        
        if let Some(env_config) = self.configs.get(&environment) {
            // Merge environment-specific config
            for (key, value) in &env_config.config {
                config.insert(key.clone(), value.clone());
            }
            
            // Apply overrides
            for (key, value) in &env_config.overrides {
                config.insert(key.clone(), value.clone());
            }
        }
        
        config
    }

    /// Override a configuration value for current environment
    pub fn override_config(&mut self, key: String, value: serde_json::Value) {
        if let Some(env_config) = self.configs.get_mut(&self.current_environment) {
            env_config.overrides.insert(key.clone(), value.clone());
            info!("Overridden configuration for {}: {}", self.current_environment, key);
        } else {
            // Create new environment config if it doesn't exist
            let mut env_config = EnvironmentConfig {
                environment: self.current_environment,
                config: HashMap::new(),
                overrides: HashMap::new(),
            };
            env_config.overrides.insert(key.clone(), value.clone());
            self.configs.insert(self.current_environment, env_config);
            info!("Created new environment config and overridden: {}", key);
        }
    }

    /// Remove configuration override
    pub fn remove_override(&mut self, key: &str) -> bool {
        if let Some(env_config) = self.configs.get_mut(&self.current_environment) {
            let removed = env_config.overrides.remove(key).is_some();
            if removed {
                info!("Removed configuration override for {}: {}", self.current_environment, key);
            }
            removed
        } else {
            false
        }
    }

    /// Get all available environments
    pub fn get_available_environments(&self) -> Vec<Environment> {
        self.configs.keys().cloned().collect()
    }

    /// Check if environment has configuration
    pub fn has_environment_config(&self, environment: Environment) -> bool {
        self.configs.contains_key(&environment)
    }

    /// Get environment-specific file path
    pub fn get_config_file_path(&self, base_path: &str) -> String {
        match self.current_environment {
            Environment::Development => format!("{}.dev.json", base_path),
            Environment::Staging => format!("{}.staging.json", base_path),
            Environment::Production => format!("{}.prod.json", base_path),
            Environment::Test => format!("{}.test.json", base_path),
        }
    }

    /// Get environment-specific log level
    pub fn get_log_level(&self) -> &'static str {
        match self.current_environment {
            Environment::Development => "debug",
            Environment::Staging => "info",
            Environment::Production => "warn",
            Environment::Test => "error",
        }
    }

    /// Get environment-specific database URL
    pub fn get_database_url(&self) -> String {
        let config = self.get_current_config();
        config.get("database.url")
            .and_then(|v| v.as_str())
            .unwrap_or("postgresql://localhost:5432/agent_agency")
            .to_string()
    }

    /// Get environment-specific server port
    pub fn get_server_port(&self) -> u16 {
        let config = self.get_current_config();
        config.get("server.port")
            .and_then(|v| v.as_u64())
            .unwrap_or(8080) as u16
    }

    /// Get environment-specific JWT secret
    pub fn get_jwt_secret(&self) -> String {
        let config = self.get_current_config();
        config.get("security.jwt_secret")
            .and_then(|v| v.as_str())
            .unwrap_or("change-me-in-production")
            .to_string()
    }

    /// Check if debug mode is enabled
    pub fn is_debug_enabled(&self) -> bool {
        match self.current_environment {
            Environment::Development | Environment::Test => true,
            Environment::Staging | Environment::Production => false,
        }
    }

    /// Check if hot reloading is enabled
    pub fn is_hot_reload_enabled(&self) -> bool {
        match self.current_environment {
            Environment::Development => true,
            Environment::Staging | Environment::Production | Environment::Test => false,
        }
    }

    /// Get environment-specific cache TTL
    pub fn get_cache_ttl(&self) -> u64 {
        let config = self.get_current_config();
        config.get("cache.default_ttl_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(match self.current_environment {
                Environment::Development => 60,   // 1 minute
                Environment::Staging => 300,      // 5 minutes
                Environment::Production => 3600,  // 1 hour
                Environment::Test => 0,           // No cache
            })
    }

    /// Get environment-specific metrics collection interval
    pub fn get_metrics_interval(&self) -> u64 {
        let config = self.get_current_config();
        config.get("metrics.collection_interval_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(match self.current_environment {
                Environment::Development => 30,   // 30 seconds
                Environment::Staging => 60,       // 1 minute
                Environment::Production => 300,   // 5 minutes
                Environment::Test => 10,          // 10 seconds
            })
    }
}

/// Environment detection utilities
pub mod detection {
    use super::*;

    /// Detect environment from environment variable
    pub fn detect_from_env() -> Result<Environment> {
        let env_str = std::env::var("AGENT_AGENCY_ENV")
            .or_else(|_| std::env::var("NODE_ENV"))
            .or_else(|_| std::env::var("ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());
        
        Environment::from_str(&env_str)
    }

    /// Detect environment from file
    pub fn detect_from_file(path: &str) -> Result<Environment> {
        let content = std::fs::read_to_string(path)?;
        let env_str = content.trim();
        Environment::from_str(env_str)
    }

    /// Detect environment from hostname
    pub fn detect_from_hostname() -> Result<Environment> {
        let hostname = hostname::get()?
            .to_string_lossy()
            .to_lowercase();
        
        if hostname.contains("prod") || hostname.contains("production") {
            Ok(Environment::Production)
        } else if hostname.contains("staging") || hostname.contains("stage") {
            Ok(Environment::Staging)
        } else if hostname.contains("test") || hostname.contains("testing") {
            Ok(Environment::Test)
        } else {
            Ok(Environment::Development)
        }
    }

    /// Auto-detect environment using multiple methods
    pub fn auto_detect() -> Result<Environment> {
        // Try environment variable first
        if let Ok(env) = detect_from_env() {
            return Ok(env);
        }
        
        // Try file detection
        if let Ok(env) = detect_from_file(".env") {
            return Ok(env);
        }
        
        // Fallback to hostname detection
        detect_from_hostname()
    }
}

/// Environment-specific configuration presets
pub mod presets {
    use super::*;

    /// Get development configuration preset
    pub fn development_config() -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        
        // Database
        config.insert("database.url".to_string(), serde_json::Value::String("postgresql://localhost:5432/agent_agency_dev".to_string()));
        config.insert("database.max_connections".to_string(), serde_json::Value::Number(5.into()));
        
        // Server
        config.insert("server.port".to_string(), serde_json::Value::Number(3000.into()));
        config.insert("server.host".to_string(), serde_json::Value::String("localhost".to_string()));
        
        // Logging
        config.insert("logging.level".to_string(), serde_json::Value::String("debug".to_string()));
        config.insert("logging.format".to_string(), serde_json::Value::String("pretty".to_string()));
        
        // Security
        config.insert("security.jwt_secret".to_string(), serde_json::Value::String("dev-secret-key".to_string()));
        config.insert("security.rate_limit_per_minute".to_string(), serde_json::Value::Number(1000.into()));
        
        // Cache
        config.insert("cache.default_ttl_secs".to_string(), serde_json::Value::Number(60.into()));
        
        // Metrics
        config.insert("metrics.collection_interval_secs".to_string(), serde_json::Value::Number(30.into()));
        
        config
    }

    /// Get staging configuration preset
    pub fn staging_config() -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        
        // Database
        config.insert("database.url".to_string(), serde_json::Value::String("postgresql://staging-db:5432/agent_agency_staging".to_string()));
        config.insert("database.max_connections".to_string(), serde_json::Value::Number(20.into()));
        
        // Server
        config.insert("server.port".to_string(), serde_json::Value::Number(8080.into()));
        config.insert("server.host".to_string(), serde_json::Value::String("0.0.0.0".to_string()));
        
        // Logging
        config.insert("logging.level".to_string(), serde_json::Value::String("info".to_string()));
        config.insert("logging.format".to_string(), serde_json::Value::String("json".to_string()));
        
        // Security
        config.insert("security.jwt_secret".to_string(), serde_json::Value::String("staging-secret-key".to_string()));
        config.insert("security.rate_limit_per_minute".to_string(), serde_json::Value::Number(500.into()));
        
        // Cache
        config.insert("cache.default_ttl_secs".to_string(), serde_json::Value::Number(300.into()));
        
        // Metrics
        config.insert("metrics.collection_interval_secs".to_string(), serde_json::Value::Number(60.into()));
        
        config
    }

    /// Get production configuration preset
    pub fn production_config() -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        
        // Database
        config.insert("database.url".to_string(), serde_json::Value::String("postgresql://prod-db:5432/agent_agency_prod".to_string()));
        config.insert("database.max_connections".to_string(), serde_json::Value::Number(50.into()));
        
        // Server
        config.insert("server.port".to_string(), serde_json::Value::Number(8080.into()));
        config.insert("server.host".to_string(), serde_json::Value::String("0.0.0.0".to_string()));
        
        // Logging
        config.insert("logging.level".to_string(), serde_json::Value::String("warn".to_string()));
        config.insert("logging.format".to_string(), serde_json::Value::String("json".to_string()));
        
        // Security
        config.insert("security.jwt_secret".to_string(), serde_json::Value::String("production-secret-key".to_string()));
        config.insert("security.rate_limit_per_minute".to_string(), serde_json::Value::Number(100.into()));
        
        // Cache
        config.insert("cache.default_ttl_secs".to_string(), serde_json::Value::Number(3600.into()));
        
        // Metrics
        config.insert("metrics.collection_interval_secs".to_string(), serde_json::Value::Number(300.into()));
        
        config
    }

    /// Get test configuration preset
    pub fn test_config() -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        
        // Database
        config.insert("database.url".to_string(), serde_json::Value::String("postgresql://localhost:5432/agent_agency_test".to_string()));
        config.insert("database.max_connections".to_string(), serde_json::Value::Number(2.into()));
        
        // Server
        config.insert("server.port".to_string(), serde_json::Value::Number(0.into())); // Random port
        config.insert("server.host".to_string(), serde_json::Value::String("localhost".to_string()));
        
        // Logging
        config.insert("logging.level".to_string(), serde_json::Value::String("error".to_string()));
        config.insert("logging.format".to_string(), serde_json::Value::String("json".to_string()));
        
        // Security
        config.insert("security.jwt_secret".to_string(), serde_json::Value::String("test-secret-key".to_string()));
        config.insert("security.rate_limit_per_minute".to_string(), serde_json::Value::Number(10000.into()));
        
        // Cache
        config.insert("cache.default_ttl_secs".to_string(), serde_json::Value::Number(0.into())); // No cache
        
        // Metrics
        config.insert("metrics.collection_interval_secs".to_string(), serde_json::Value::Number(10.into()));
        
        config
    }
}

/// Global environment manager instance
static ENVIRONMENT_MANAGER: once_cell::sync::OnceCell<EnvironmentManager> = once_cell::sync::OnceCell::new();

/// Initialize the global environment manager
pub fn init_environment_manager(environment: Environment) -> Result<()> {
    let mut manager = EnvironmentManager::new(environment);
    
    // Load environment-specific presets
    match environment {
        Environment::Development => manager.load_environment_config(environment, presets::development_config()),
        Environment::Staging => manager.load_environment_config(environment, presets::staging_config()),
        Environment::Production => manager.load_environment_config(environment, presets::production_config()),
        Environment::Test => manager.load_environment_config(environment, presets::test_config()),
    }
    
    ENVIRONMENT_MANAGER.set(manager)
        .map_err(|_| anyhow!("Environment manager already initialized"))?;
    
    info!("Environment manager initialized for: {}", environment);
    Ok(())
}

/// Get the global environment manager
pub fn get_environment_manager() -> Result<&'static EnvironmentManager> {
    ENVIRONMENT_MANAGER.get()
        .ok_or_else(|| anyhow!("Environment manager not initialized"))
}

/// Convenience function to get current environment
pub fn get_current_environment() -> Result<Environment> {
    let manager = get_environment_manager()?;
    Ok(manager.get_environment())
}

/// Convenience function to get current configuration
pub fn get_current_config() -> Result<HashMap<String, serde_json::Value>> {
    let manager = get_environment_manager()?;
    Ok(manager.get_current_config())
}
