//! Configuration loading and hot-reloading

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Configuration loader with hot-reloading support
pub struct ConfigLoader {
    config_path: String,
    config: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    watchers: Arc<RwLock<Vec<ConfigWatcher>>>,
    reload_interval: Duration,
    last_modified: Arc<RwLock<Option<std::time::SystemTime>>>,
}

/// Configuration watcher for change notifications
pub struct ConfigWatcher {
    pub id: Uuid,
    pub callback: Arc<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<()> + Send + Sync>,
}

/// Configuration source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    File(String),
    Environment,
    Default,
    Override(String),
}

/// Configuration loading result
#[derive(Debug, Clone)]
pub struct ConfigLoadResult {
    pub source: ConfigSource,
    pub config: HashMap<String, serde_json::Value>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Configuration loader builder
pub struct ConfigLoaderBuilder {
    config_path: Option<String>,
    reload_interval: Option<Duration>,
    auto_reload: bool,
    validate_on_load: bool,
    merge_strategy: MergeStrategy,
}

/// Configuration merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Override values (later sources override earlier ones)
    Override,
    /// Merge objects recursively
    Merge,
    /// Replace entire configuration
    Replace,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
            config: Arc::new(RwLock::new(HashMap::new())),
            watchers: Arc::new(RwLock::new(Vec::new())),
            reload_interval: Duration::from_secs(30),
            last_modified: Arc::new(RwLock::new(None)),
        }
    }

    /// Load configuration from all sources
    pub async fn load(&self) -> Result<ConfigLoadResult> {
        let mut config = HashMap::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Load from file first
        if let Err(e) = self.load_from_file(&mut config).await {
            errors.push(format!("Failed to load from file: {}", e));
        }

        // Load from environment variables
        if let Err(e) = self.load_from_env(&mut config).await {
            errors.push(format!("Failed to load from environment: {}", e));
        }

        // Load from defaults
        if let Err(e) = self.load_defaults(&mut config).await {
            errors.push(format!("Failed to load defaults: {}", e));
        }

        // Store the loaded configuration
        {
            let mut current_config = self.config.write().await;
            *current_config = config.clone();
        }

        // Notify watchers
        self.notify_watchers(&config).await;

        Ok(ConfigLoadResult {
            source: ConfigSource::File(self.config_path.clone()),
            config,
            errors,
            warnings,
        })
    }

    /// Load configuration from file
    async fn load_from_file(&self, config: &mut HashMap<String, serde_json::Value>) -> Result<()> {
        let path = Path::new(&self.config_path);
        
        if !path.exists() {
            return Err(anyhow!("Configuration file does not exist: {}", self.config_path));
        }

        let content = fs::read_to_string(&path).await?;
        let file_config: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
        
        // Merge with existing config
        for (key, value) in file_config {
            config.insert(key, value);
        }

        // Update last modified time
        let metadata = fs::metadata(&path).await?;
        let modified = metadata.modified()?;
        {
            let mut last_modified = self.last_modified.write().await;
            *last_modified = Some(modified);
        }

        info!("Loaded configuration from file: {}", self.config_path);
        Ok(())
    }

    /// Load configuration from environment variables
    async fn load_from_env(&self, config: &mut HashMap<String, serde_json::Value>) -> Result<()> {
        let env_prefix = "AGENT_AGENCY_";
        let mut loaded_count = 0;

        for (key, value) in std::env::vars() {
            if key.starts_with(env_prefix) {
                let config_key = key.strip_prefix(env_prefix).unwrap_or(&key).to_lowercase();
                
                // Try to parse as JSON first, fallback to string
                let parsed_value = if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&value) {
                    json_value
                } else {
                    serde_json::Value::String(value)
                };

                config.insert(config_key, parsed_value);
                loaded_count += 1;
            }
        }

        if loaded_count > 0 {
            info!("Loaded {} configuration values from environment variables", loaded_count);
        }

        Ok(())
    }

    /// Load default configuration values
    async fn load_defaults(&self, config: &mut HashMap<String, serde_json::Value>) -> Result<()> {
        let defaults = self.get_default_config();
        
        for (key, value) in defaults {
            if !config.contains_key(&key) {
                config.insert(key, value);
            }
        }

        Ok(())
    }

    /// Get default configuration values
    fn get_default_config(&self) -> HashMap<String, serde_json::Value> {
        let mut defaults = HashMap::new();
        
        // Database defaults
        defaults.insert("database.url".to_string(), serde_json::Value::String("postgresql://localhost:5432/agent_agency".to_string()));
        defaults.insert("database.max_connections".to_string(), serde_json::Value::Number(10.into()));
        defaults.insert("database.connection_timeout_secs".to_string(), serde_json::Value::Number(30.into()));
        defaults.insert("database.idle_timeout_secs".to_string(), serde_json::Value::Number(300.into()));

        // Server defaults
        defaults.insert("server.port".to_string(), serde_json::Value::Number(8080.into()));
        defaults.insert("server.host".to_string(), serde_json::Value::String("0.0.0.0".to_string()));
        defaults.insert("server.request_timeout_secs".to_string(), serde_json::Value::Number(30.into()));
        defaults.insert("server.max_request_size_mb".to_string(), serde_json::Value::Number(10.into()));

        // Agent defaults
        defaults.insert("agent.max_concurrent_tasks".to_string(), serde_json::Value::Number(10.into()));
        defaults.insert("agent.task_timeout_secs".to_string(), serde_json::Value::Number(300.into()));
        defaults.insert("agent.max_retry_attempts".to_string(), serde_json::Value::Number(3.into()));
        defaults.insert("agent.retry_delay_secs".to_string(), serde_json::Value::Number(5.into()));

        // Logging defaults
        defaults.insert("logging.level".to_string(), serde_json::Value::String("info".to_string()));
        defaults.insert("logging.format".to_string(), serde_json::Value::String("json".to_string()));
        defaults.insert("logging.max_file_size_mb".to_string(), serde_json::Value::Number(100.into()));
        defaults.insert("logging.max_files".to_string(), serde_json::Value::Number(10.into()));

        // Metrics defaults
        defaults.insert("metrics.collection_interval_secs".to_string(), serde_json::Value::Number(60.into()));
        defaults.insert("metrics.retention_days".to_string(), serde_json::Value::Number(30.into()));
        defaults.insert("metrics.batch_size".to_string(), serde_json::Value::Number(1000.into()));

        // Security defaults
        defaults.insert("security.jwt_secret".to_string(), serde_json::Value::String("change-me-in-production".to_string()));
        defaults.insert("security.jwt_expiry_secs".to_string(), serde_json::Value::Number(3600.into()));
        defaults.insert("security.rate_limit_per_minute".to_string(), serde_json::Value::Number(100.into()));
        defaults.insert("security.max_login_attempts".to_string(), serde_json::Value::Number(5.into()));

        // Cache defaults
        defaults.insert("cache.default_ttl_secs".to_string(), serde_json::Value::Number(300.into()));
        defaults.insert("cache.max_size_mb".to_string(), serde_json::Value::Number(100.into()));
        defaults.insert("cache.eviction_threshold_percent".to_string(), serde_json::Value::Number(80.into()));

        // Resource defaults
        defaults.insert("resources.cpu_limit_percent".to_string(), serde_json::Value::Number(80.into()));
        defaults.insert("resources.memory_limit_percent".to_string(), serde_json::Value::Number(80.into()));
        defaults.insert("resources.disk_limit_gb".to_string(), serde_json::Value::Number(10.into()));
        defaults.insert("resources.network_limit_mbps".to_string(), serde_json::Value::Number(100.into()));

        // Tracing defaults
        defaults.insert("tracing.level".to_string(), serde_json::Value::String("info".to_string()));
        defaults.insert("tracing.sampling_rate_percent".to_string(), serde_json::Value::Number(10.into()));
        defaults.insert("tracing.export_interval_secs".to_string(), serde_json::Value::Number(60.into()));

        // Deployment defaults
        defaults.insert("deployment.environment".to_string(), serde_json::Value::String("development".to_string()));
        defaults.insert("deployment.version".to_string(), serde_json::Value::String("1.0.0".to_string()));
        defaults.insert("deployment.region".to_string(), serde_json::Value::String("us-east-1".to_string()));
        defaults.insert("deployment.replicas".to_string(), serde_json::Value::Number(1.into()));

        defaults
    }

    /// Start hot-reloading
    pub async fn start_hot_reload(&self) -> Result<()> {
        let mut interval = interval(self.reload_interval);
        let config_path = self.config_path.clone();
        let config = self.config.clone();
        let last_modified = self.last_modified.clone();
        let watchers = self.watchers.clone();

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::check_and_reload(&config_path, &config, &last_modified, &watchers).await {
                    error!("Hot reload failed: {}", e);
                }
            }
        });

        info!("Started hot-reloading for configuration file: {}", self.config_path);
        Ok(())
    }

    /// Check for file changes and reload if necessary
    async fn check_and_reload(
        config_path: &str,
        config: &Arc<RwLock<HashMap<String, serde_json::Value>>>,
        last_modified: &Arc<RwLock<Option<std::time::SystemTime>>>,
        watchers: &Arc<RwLock<Vec<ConfigWatcher>>>,
    ) -> Result<()> {
        let path = Path::new(config_path);
        
        if !path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(path).await?;
        let modified = metadata.modified()?;
        
        let should_reload = {
            let last_modified_guard = last_modified.read().await;
            last_modified_guard.map_or(true, |last| modified > last)
        };

        if should_reload {
            debug!("Configuration file changed, reloading...");
            
            let content = fs::read_to_string(path).await?;
            let new_config: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
            
            {
                let mut config_guard = config.write().await;
                *config_guard = new_config.clone();
            }
            
            {
                let mut last_modified_guard = last_modified.write().await;
                *last_modified_guard = Some(modified);
            }
            
            // Notify watchers
            let watchers_guard = watchers.read().await;
            for watcher in watchers_guard.iter() {
                if let Err(e) = (watcher.callback)(&new_config) {
                    error!("Configuration watcher callback failed: {}", e);
                }
            }
            
            info!("Configuration reloaded successfully");
        }

        Ok(())
    }

    /// Add a configuration watcher
    pub async fn add_watcher<F>(&self, callback: F) -> Uuid
    where
        F: Fn(&HashMap<String, serde_json::Value>) -> Result<()> + Send + Sync + 'static,
    {
        let watcher = ConfigWatcher {
            id: Uuid::new_v4(),
            callback: Arc::new(callback),
        };
        
        let id = watcher.id;
        let mut watchers = self.watchers.write().await;
        watchers.push(watcher);
        
        info!("Added configuration watcher: {}", id);
        id
    }

    /// Remove a configuration watcher
    pub async fn remove_watcher(&self, id: Uuid) -> bool {
        let mut watchers = self.watchers.write().await;
        let initial_len = watchers.len();
        watchers.retain(|w| w.id != id);
        let removed = watchers.len() < initial_len;
        
        if removed {
            info!("Removed configuration watcher: {}", id);
        }
        
        removed
    }

    /// Get current configuration
    pub async fn get_config(&self) -> HashMap<String, serde_json::Value> {
        let config = self.config.read().await;
        config.clone()
    }

    /// Get a specific configuration value
    pub async fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        let config = self.config.read().await;
        config.get(key).cloned()
    }

    /// Set a configuration value
    pub async fn set_value(&self, key: String, value: serde_json::Value) {
        let mut config = self.config.write().await;
        config.insert(key, value);
    }

    /// Notify all watchers of configuration changes
    async fn notify_watchers(&self, config: &HashMap<String, serde_json::Value>) {
        let watchers = self.watchers.read().await;
        for watcher in watchers.iter() {
            if let Err(e) = (watcher.callback)(config) {
                error!("Configuration watcher callback failed: {}", e);
            }
        }
    }
}

impl ConfigLoaderBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config_path: None,
            reload_interval: None,
            auto_reload: false,
            validate_on_load: false,
            merge_strategy: MergeStrategy::Override,
        }
    }

    /// Set the configuration file path
    pub fn config_path(mut self, path: &str) -> Self {
        self.config_path = Some(path.to_string());
        self
    }

    /// Set the reload interval
    pub fn reload_interval(mut self, interval: Duration) -> Self {
        self.reload_interval = Some(interval);
        self
    }

    /// Enable or disable auto-reload
    pub fn auto_reload(mut self, enabled: bool) -> Self {
        self.auto_reload = enabled;
        self
    }

    /// Enable or disable validation on load
    pub fn validate_on_load(mut self, enabled: bool) -> Self {
        self.validate_on_load = enabled;
        self
    }

    /// Set the merge strategy
    pub fn merge_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }

    /// Build the configuration loader
    pub fn build(self) -> Result<ConfigLoader> {
        let config_path = self.config_path
            .ok_or_else(|| anyhow!("Configuration path is required"))?;

        let mut loader = ConfigLoader::new(&config_path);
        
        if let Some(interval) = self.reload_interval {
            loader.reload_interval = interval;
        }

        Ok(loader)
    }
}

/// Global configuration loader instance
static CONFIG_LOADER: once_cell::sync::OnceCell<Arc<ConfigLoader>> = once_cell::sync::OnceCell::new();

/// Initialize the global configuration loader
pub async fn init_config_loader(config_path: &str) -> Result<Arc<ConfigLoader>> {
    let loader = Arc::new(ConfigLoader::new(config_path));
    loader.load().await?;
    
    if CONFIG_LOADER.set(loader.clone()).is_err() {
        return Err(anyhow!("Configuration loader already initialized"));
    }
    
    info!("Configuration loader initialized with path: {}", config_path);
    Ok(loader)
}

/// Get the global configuration loader
pub fn get_config_loader() -> Result<Arc<ConfigLoader>> {
    CONFIG_LOADER.get()
        .cloned()
        .ok_or_else(|| anyhow!("Configuration loader not initialized"))
}

/// Convenience function to get a configuration value
pub async fn get_config_value(key: &str) -> Result<Option<serde_json::Value>> {
    let loader = get_config_loader()?;
    Ok(loader.get_value(key).await)
}

/// Convenience function to set a configuration value
pub async fn set_config_value(key: String, value: serde_json::Value) -> Result<()> {
    let loader = get_config_loader()?;
    loader.set_value(key, value).await;
    Ok(())
}
