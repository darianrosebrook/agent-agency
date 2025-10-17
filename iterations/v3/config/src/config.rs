//! Core configuration structures and management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use validator::Validate;
use anyhow::Result;
use tracing::{info, warn, error};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppConfig {
    /// Application metadata
    pub app: AppMetadata,
    
    /// Server configuration
    pub server: ServerConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    
    /// Component-specific configurations
    pub components: ComponentConfigs,
    
    /// Environment-specific overrides
    pub environment: EnvironmentConfig,
}

/// Application metadata
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppMetadata {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub debug: bool,
    pub log_level: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: u32,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub ca_path: Option<PathBuf>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub ssl_mode: String,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub encryption_key: String,
    pub session_timeout_minutes: u64,
    pub rate_limit_requests_per_minute: u32,
    pub cors_origins: Vec<String>,
    pub enable_audit_logging: bool,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub health_check_enabled: bool,
    pub health_check_port: u16,
    pub log_level: String,
    pub structured_logging: bool,
    pub prometheus_endpoint: Option<String>,
}

/// Component-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ComponentConfigs {
    pub orchestration: OrchestrationConfig,
    pub council: CouncilConfig,
    pub research: ResearchConfig,
    pub workers: WorkersConfig,
    pub provenance: ProvenanceConfig,
    pub apple_silicon: AppleSiliconConfig,
}

/// Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OrchestrationConfig {
    pub max_concurrent_tasks: u32,
    pub task_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
    pub enable_circuit_breaker: bool,
}

/// Council configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CouncilConfig {
    pub judge_timeout_seconds: u64,
    pub max_debate_rounds: u32,
    pub consensus_threshold: f32,
    pub enable_learning: bool,
    pub evidence_cache_size: usize,
}

/// Research configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResearchConfig {
    pub max_search_results: u32,
    pub search_timeout_seconds: u64,
    pub enable_semantic_search: bool,
    pub enable_web_search: bool,
    pub confidence_threshold: f32,
}

/// Workers configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WorkersConfig {
    pub max_workers: u32,
    pub worker_timeout_seconds: u64,
    pub enable_auto_scaling: bool,
    pub min_workers: u32,
}

/// Provenance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProvenanceConfig {
    pub enable_git_integration: bool,
    pub git_repo_path: Option<PathBuf>,
    pub enable_chain_tracking: bool,
    pub max_chain_length: usize,
}

/// Apple Silicon configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppleSiliconConfig {
    pub enable_ane_optimization: bool,
    pub enable_gpu_acceleration: bool,
    pub thermal_management: bool,
    pub max_cpu_cores: u32,
    pub max_memory_gb: u32,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EnvironmentConfig {
    pub development: Option<EnvironmentOverrides>,
    pub staging: Option<EnvironmentOverrides>,
    pub production: Option<EnvironmentOverrides>,
}

/// Environment-specific overrides
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EnvironmentOverrides {
    pub debug: Option<bool>,
    pub log_level: Option<String>,
    pub database_url: Option<String>,
    pub server_port: Option<u16>,
    pub enable_metrics: Option<bool>,
}

impl AppConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self {
            app: AppMetadata {
                name: "agent-agency-v3".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: "development".to_string(),
                debug: true,
                log_level: "info".to_string(),
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
                max_connections: 1000,
                timeout_seconds: 30,
                tls: None,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/agent_agency".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout_seconds: 30,
                idle_timeout_seconds: 600,
                max_lifetime_seconds: 3600,
                ssl_mode: "prefer".to_string(),
            },
            security: SecurityConfig {
                jwt_secret: "default-secret-change-in-production".to_string(),
                encryption_key: "default-encryption-key-change-in-production".to_string(),
                session_timeout_minutes: 60,
                rate_limit_requests_per_minute: 100,
                cors_origins: vec!["http://localhost:3000".to_string()],
                enable_audit_logging: true,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                health_check_enabled: true,
                health_check_port: 8081,
                log_level: "info".to_string(),
                structured_logging: true,
                prometheus_endpoint: Some("http://localhost:9090/metrics".to_string()),
            },
            components: ComponentConfigs {
                orchestration: OrchestrationConfig {
                    max_concurrent_tasks: 100,
                    task_timeout_seconds: 300,
                    retry_attempts: 3,
                    retry_delay_seconds: 5,
                    enable_circuit_breaker: true,
                },
                council: CouncilConfig {
                    judge_timeout_seconds: 30,
                    max_debate_rounds: 3,
                    consensus_threshold: 0.6,
                    enable_learning: true,
                    evidence_cache_size: 1000,
                },
                research: ResearchConfig {
                    max_search_results: 50,
                    search_timeout_seconds: 10,
                    enable_semantic_search: true,
                    enable_web_search: false,
                    confidence_threshold: 0.7,
                },
                workers: WorkersConfig {
                    max_workers: 10,
                    worker_timeout_seconds: 60,
                    enable_auto_scaling: true,
                    min_workers: 2,
                },
                provenance: ProvenanceConfig {
                    enable_git_integration: true,
                    git_repo_path: Some(PathBuf::from(".")),
                    enable_chain_tracking: true,
                    max_chain_length: 1000,
                },
                apple_silicon: AppleSiliconConfig {
                    enable_ane_optimization: true,
                    enable_gpu_acceleration: true,
                    thermal_management: true,
                    max_cpu_cores: 8,
                    max_memory_gb: 16,
                },
            },
            environment: EnvironmentConfig {
                development: Some(EnvironmentOverrides {
                    debug: Some(true),
                    log_level: Some("debug".to_string()),
                    database_url: Some("postgresql://localhost:5432/agent_agency_dev".to_string()),
                    server_port: Some(8080),
                    enable_metrics: Some(true),
                }),
                staging: Some(EnvironmentOverrides {
                    debug: Some(false),
                    log_level: Some("info".to_string()),
                    database_url: Some("postgresql://staging-db:5432/agent_agency_staging".to_string()),
                    server_port: Some(8080),
                    enable_metrics: Some(true),
                }),
                production: Some(EnvironmentOverrides {
                    debug: Some(false),
                    log_level: Some("warn".to_string()),
                    database_url: None, // Must be provided via environment variable
                    server_port: Some(80),
                    enable_metrics: Some(true),
                }),
            },
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Basic validation logic
        
        // Additional custom validations
        if self.security.jwt_secret == "default-secret-change-in-production" && 
           self.app.environment == "production" {
            return Err(anyhow::anyhow!("JWT secret must be changed in production"));
        }
        
        if self.security.encryption_key == "default-encryption-key-change-in-production" && 
           self.app.environment == "production" {
            return Err(anyhow::anyhow!("Encryption key must be changed in production"));
        }
        
        if self.database.url.contains("localhost") && self.app.environment == "production" {
            warn!("Using localhost database in production environment");
        }
        
        info!("Configuration validation passed");
        Ok(())
    }

    /// Apply environment-specific overrides
    pub fn apply_environment_overrides(&mut self) -> Result<()> {
        let overrides = match self.app.environment.as_str() {
            "development" => &self.environment.development,
            "staging" => &self.environment.staging,
            "production" => &self.environment.production,
            _ => {
                warn!("Unknown environment: {}, using defaults", self.app.environment);
                return Ok(());
            }
        };

        if let Some(overrides) = overrides {
            if let Some(debug) = overrides.debug {
                self.app.debug = debug;
            }
            if let Some(log_level) = &overrides.log_level {
                self.app.log_level = log_level.clone();
                self.monitoring.log_level = log_level.clone();
            }
            if let Some(database_url) = &overrides.database_url {
                self.database.url = database_url.clone();
            }
            if let Some(server_port) = overrides.server_port {
                self.server.port = server_port;
            }
            if let Some(enable_metrics) = overrides.enable_metrics {
                self.monitoring.metrics_enabled = enable_metrics;
            }
            
            info!("Applied environment overrides for: {}", self.app.environment);
        }

        Ok(())
    }

    /// Get configuration for a specific component
    pub fn get_component_config<T>(&self, component_name: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let config_value = match component_name {
            "orchestration" => serde_json::to_value(&self.components.orchestration)?,
            "council" => serde_json::to_value(&self.components.council)?,
            "research" => serde_json::to_value(&self.components.research)?,
            "workers" => serde_json::to_value(&self.components.workers)?,
            "provenance" => serde_json::to_value(&self.components.provenance)?,
            "apple_silicon" => serde_json::to_value(&self.components.apple_silicon)?,
            _ => return Err(anyhow::anyhow!("Unknown component: {}", component_name)),
        };

        let config: T = serde_json::from_value(config_value)?;
        Ok(config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}
