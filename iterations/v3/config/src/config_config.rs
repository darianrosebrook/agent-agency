//! Core configuration structures and management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};
use validator::Validate;

use super::environment::secure_loader;
use agent_memory::MemoryConfig;

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
    #[validate(custom(function = "tls_validation::validate_cert_file"))]
    pub cert_path: PathBuf,
    #[validate(custom(function = "tls_validation::validate_key_file"))]
    pub key_path: PathBuf,
    #[validate(custom(function = "tls_validation::validate_ca_file"))]
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
    #[validate(length(min = 32, message = "JWT secret must be at least 32 characters"))]
    pub jwt_secret: String,
    #[validate(length(min = 32, message = "Encryption key must be at least 32 characters"))]
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
    /// Redis configuration for metrics caching
    pub redis: Option<RedisConfig>,
    /// Prometheus configuration for metrics collection
    pub prometheus: Option<PrometheusConfig>,
    /// StatsD configuration for metrics aggregation
    pub statsd: Option<StatsDConfig>,
}

/// Redis configuration for observability backends
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub database: u8,
    pub pool_size: usize,
    pub connection_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
}

/// Prometheus configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PrometheusConfig {
    pub endpoint: String,
    pub push_interval_seconds: u64,
    pub job_name: String,
    pub instance: String,
}

/// StatsD configuration for metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StatsDConfig {
    pub host: String,
    pub port: u16,
    pub prefix: String,
    pub flush_interval_seconds: u64,
}

/// Component-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ComponentConfigs {
    pub orchestration: OrchestrationConfig,
    pub council: CouncilConfig,
    pub research: ResearchConfig,
    pub workers: WorkersConfig,
    pub parallel_workers: ParallelWorkersConfig,
    pub provenance: ProvenanceConfig,
    pub apple_silicon: AppleSiliconConfig,
    /// Core memory system configuration - every agent has memory
    pub memory: MemoryConfig,
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

/// Parallel workers configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ParallelWorkersConfig {
    pub enabled: bool,
    pub complexity_threshold: f32,
    pub max_concurrent_workers: u32,
    pub max_subtasks_per_task: u32,
    pub coordination: ParallelCoordinationConfig,
    pub decomposition: ParallelDecompositionConfig,
    pub quality_gates: QualityGatesConfig,
    pub learning: LearningConfig,
}

/// Parallel coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ParallelCoordinationConfig {
    pub message_buffer_size: usize,
    pub progress_update_interval_ms: u64,
    pub worker_heartbeat_timeout_ms: u64,
}

/// Parallel decomposition configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ParallelDecompositionConfig {
    pub strategies: Vec<String>,
    pub max_workers_per_task: u32,
    pub dependency_analysis_enabled: bool,
}

/// Quality gates configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct QualityGatesConfig {
    pub compilation: QualityGateSettings,
    pub testing: QualityGateSettings,
    pub linting: QualityGateSettings,
    pub security: QualityGateSettings,
}

/// Individual quality gate settings
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct QualityGateSettings {
    pub enabled: bool,
    pub blocking: bool,
    pub min_score: f32,
}

/// Learning system configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LearningConfig {
    pub enabled: bool,
    pub metrics_collection: MetricsCollectionConfig,
    pub pattern_analysis: PatternAnalysisConfig,
    pub adaptive_selection: AdaptiveSelectionConfig,
    pub config_optimization: ConfigOptimizationConfig,
    pub persistence: LearningPersistenceConfig,
    pub reward_system: RewardSystemConfig,
    pub drift_detection: DriftDetectionConfig,
    pub failure_taxonomy: FailureTaxonomyConfig,
    pub council_integration: CouncilIntegrationConfig,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MetricsCollectionConfig {
    pub enabled: bool,
    pub collection_interval_ms: u64,
    pub max_records_buffer: usize,
    pub retention_days: u32,
}

/// Pattern analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PatternAnalysisConfig {
    pub enabled: bool,
    pub min_pattern_confidence: f32,
    pub max_patterns_per_type: usize,
    pub pattern_update_interval_ms: u64,
}

/// Adaptive worker selection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AdaptiveSelectionConfig {
    pub enabled: bool,
    pub selection_strategy: String,
    pub fairness_alpha: f32,
    pub exploration_rate: f32,
    pub max_workers_per_task: usize,
}

/// Configuration optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ConfigOptimizationConfig {
    pub enabled: bool,
    pub optimization_interval_ms: u64,
    pub min_improvement_threshold: f32,
    pub max_configurations_per_pattern: usize,
}

/// Learning persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LearningPersistenceConfig {
    pub enabled: bool,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub connection_pool_size: u32,
}

/// Reward system configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RewardSystemConfig {
    pub quality_weight: f32,
    pub latency_weight: f32,
    pub token_cost_weight: f32,
    pub rework_rate_weight: f32,
    pub baseline_update_interval_ms: u64,
}

/// Drift detection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DriftDetectionConfig {
    pub enabled: bool,
    pub cusum_threshold: f32,
    pub adwin_delta: f64,
    pub min_samples_before_detection: usize,
    pub drift_freeze_duration_ms: u64,
}

/// Failure taxonomy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FailureTaxonomyConfig {
    pub enabled: bool,
    pub max_categories: usize,
    pub min_confidence_threshold: f32,
    pub rca_max_depth: usize,
}

/// Council integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CouncilIntegrationConfig {
    pub enabled: bool,
    pub signal_batch_size: usize,
    pub signal_flush_interval_ms: u64,
    pub max_signal_age_ms: u64,
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
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| panic!("DATABASE_URL environment variable is required")),
                max_connections: 20,
                min_connections: 5,
                connection_timeout_seconds: 30,
                idle_timeout_seconds: 600,
                max_lifetime_seconds: 3600,
                ssl_mode: "prefer".to_string(),
            },
            security: SecurityConfig {
                jwt_secret: secure_loader::load_secure_var("JWT_SECRET")
                    .unwrap_or_else(|_| panic!("JWT_SECRET environment variable is required and must meet security requirements")),
                encryption_key: secure_loader::load_secure_var("ENCRYPTION_KEY")
                    .unwrap_or_else(|_| panic!("ENCRYPTION_KEY environment variable is required and must meet security requirements")),
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
                redis: Some(RedisConfig {
                    host: "localhost".to_string(),
                    port: 6379,
                    password: None,
                    database: 0,
                    pool_size: 10,
                    connection_timeout_seconds: 5,
                    command_timeout_seconds: 3,
                }),
                prometheus: Some(PrometheusConfig {
                    endpoint: "http://localhost:9090".to_string(),
                    push_interval_seconds: 15,
                    job_name: "agent-agency".to_string(),
                    instance: "default".to_string(),
                }),
                statsd: Some(StatsDConfig {
                    host: "localhost".to_string(),
                    port: 8125,
                    prefix: "agent_agency".to_string(),
                    flush_interval_seconds: 10,
                }),
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
                parallel_workers: ParallelWorkersConfig {
                    enabled: true,
                    complexity_threshold: 0.6,
                    max_concurrent_workers: 8,
                    max_subtasks_per_task: 20,
                    coordination: ParallelCoordinationConfig {
                        message_buffer_size: 1000,
                        progress_update_interval_ms: 500,
                        worker_heartbeat_timeout_ms: 30000,
                    },
                    decomposition: ParallelDecompositionConfig {
                        strategies: vec![
                            "compilation_errors".to_string(),
                            "refactoring".to_string(),
                            "testing".to_string(),
                            "documentation".to_string(),
                        ],
                        max_workers_per_task: 8,
                        dependency_analysis_enabled: true,
                    },
                    quality_gates: QualityGatesConfig {
                        compilation: QualityGateSettings {
                            enabled: true,
                            blocking: true,
                            min_score: 1.0,
                        },
                        testing: QualityGateSettings {
                            enabled: true,
                            blocking: true,
                            min_score: 0.8,
                        },
                        linting: QualityGateSettings {
                            enabled: true,
                            blocking: false,
                            min_score: 0.9,
                        },
                        security: QualityGateSettings {
                            enabled: true,
                            blocking: true,
                            min_score: 0.95,
                        },
                    },
                   learning: LearningConfig {
                       enabled: true,
                       metrics_collection: MetricsCollectionConfig {
                           enabled: true,
                           collection_interval_ms: 1000,
                           max_records_buffer: 10000,
                           retention_days: 30,
                       },
                       pattern_analysis: PatternAnalysisConfig {
                           enabled: true,
                           min_pattern_confidence: 0.7,
                           max_patterns_per_type: 100,
                           pattern_update_interval_ms: 5000,
                       },
                       adaptive_selection: AdaptiveSelectionConfig {
                           enabled: true,
                           selection_strategy: "weighted_round_robin".to_string(),
                           fairness_alpha: 0.5,
                           exploration_rate: 0.1,
                           max_workers_per_task: 10,
                       },
                       config_optimization: ConfigOptimizationConfig {
                           enabled: true,
                           optimization_interval_ms: 30000,
                           min_improvement_threshold: 0.01,
                           max_configurations_per_pattern: 100,
                       },
                       persistence: LearningPersistenceConfig {
                           enabled: true,
                           batch_size: 100,
                           flush_interval_ms: 300000,
                           connection_pool_size: 10,
                       },
                       reward_system: RewardSystemConfig {
                           quality_weight: 0.4,
                           latency_weight: 0.3,
                           token_cost_weight: 0.2,
                           rework_rate_weight: 0.1,
                           baseline_update_interval_ms: 60000,
                       },
                       drift_detection: DriftDetectionConfig {
                           enabled: true,
                           cusum_threshold: 0.1,
                           adwin_delta: 0.002,
                           min_samples_before_detection: 100,
                           drift_freeze_duration_ms: 300000,
                       },
                       failure_taxonomy: FailureTaxonomyConfig {
                           enabled: true,
                           max_categories: 50,
                           min_confidence_threshold: 0.8,
                           rca_max_depth: 5,
                       },
                       council_integration: CouncilIntegrationConfig {
                           enabled: true,
                           signal_batch_size: 100,
                           signal_flush_interval_ms: 10000,
                           max_signal_age_ms: 300000,
                       },
                   },
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
                memory: MemoryConfig::default(),
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
                    database_url: Some(
                        "postgresql://staging-db:5432/agent_agency_staging".to_string(),
                    ),
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
    pub fn validate_config(&self) -> Result<()> {
        // Use the validator crate for automatic validation
        if let Err(validation_errors) = self.validate() {
            for (field, field_errors) in validation_errors.field_errors() {
                for error in field_errors {
                    return Err(anyhow::anyhow!(
                        "Configuration validation failed for field '{}': {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Validation error".into())
                    ));
                }
            }
        }

        // Additional custom validations
        if self.database.url.contains("localhost") && self.app.environment == "production" {
            warn!("Using localhost database in production environment");
        }

        // Production database URL validation
        if self.app.environment == "production" {
            if self.database.url.contains("localhost") || self.database.url.contains("127.0.0.1") {
                return Err(anyhow::anyhow!("Production database URL cannot use localhost or 127.0.0.1"));
            }
            if !self.database.url.contains("sslmode=require") && !self.database.url.contains("ssl=true") {
                warn!("Production database connection should use SSL/TLS");
            }
        }

        // Security validations for production
        if self.app.environment == "production" {
            // Security secrets validation
            if self.security.jwt_secret.len() < 32 {
                return Err(anyhow::anyhow!("JWT secret must be at least 32 characters in production"));
            }
            if self.security.encryption_key.len() < 32 {
                return Err(anyhow::anyhow!("Encryption key must be at least 32 characters in production"));
            }
            if !self.security.jwt_secret.chars().any(|c| !c.is_alphanumeric()) {
                return Err(anyhow::anyhow!("JWT secret should contain special characters in production"));
            }
            if !self.security.encryption_key.chars().any(|c| !c.is_alphanumeric()) {
                return Err(anyhow::anyhow!("Encryption key should contain special characters in production"));
            }

            // HTTPS/TLS enforcement
            if self.server.tls.is_none() {
                return Err(anyhow::anyhow!("TLS configuration is required in production (HTTPS must be enabled)"));
            }

            // Validate TLS configuration
            if let Some(tls_config) = &self.server.tls {
                if !tls_config.cert_path.exists() {
                    return Err(anyhow::anyhow!("TLS certificate file does not exist: {:?}", tls_config.cert_path));
                }
                if !tls_config.key_path.exists() {
                    return Err(anyhow::anyhow!("TLS private key file does not exist: {:?}", tls_config.key_path));
                }
                if let Some(ca_path) = &tls_config.ca_path {
                    if !ca_path.exists() {
                        return Err(anyhow::anyhow!("TLS CA certificate file does not exist: {:?}", ca_path));
                    }
                }
            }

            // Additional security checks for production
            if self.server.port == 80 {
                warn!("Using port 80 in production - consider using 443 for HTTPS");
            } else if self.server.port != 443 && self.server.tls.is_some() {
                warn!("TLS is configured but not using standard HTTPS port 443");
            }
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
                warn!(
                    "Unknown environment: {}, using defaults",
                    self.app.environment
                );
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

            info!(
                "Applied environment overrides for: {}",
                self.app.environment
            );
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

    /// Get a masked version of the configuration for logging/debugging
    pub fn get_masked_config(&self) -> Result<AppConfig> {
        let mut masked = self.clone();

        // Mask JWT secret
        masked.security.jwt_secret = secure_loader::mask_sensitive_value(&self.security.jwt_secret);

        // Mask encryption key
        masked.security.encryption_key = secure_loader::mask_sensitive_value(&self.security.encryption_key);

        // Mask database URL password if present
        masked.database.url = secure_loader::mask_database_url(&self.database.url);

        Ok(masked)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom validation functions for TLS configuration
mod tls_validation {
    use super::*;
    use validator::ValidationError;

    pub fn validate_cert_file(path: &PathBuf) -> Result<(), ValidationError> {
        if path.to_string_lossy().is_empty() {
            return Err(ValidationError::new("Certificate path cannot be empty"));
        }

        // Check for common certificate file extensions
        let path_str = path.to_string_lossy().to_lowercase();
        if !path_str.ends_with(".pem") && !path_str.ends_with(".crt") && !path_str.ends_with(".cer") {
            return Err(ValidationError::new("Certificate file should have .pem, .crt, or .cer extension"));
        }

        Ok(())
    }

    pub fn validate_key_file(path: &PathBuf) -> Result<(), ValidationError> {
        if path.to_string_lossy().is_empty() {
            return Err(ValidationError::new("Private key path cannot be empty"));
        }

        // Check for common key file extensions
        let path_str = path.to_string_lossy().to_lowercase();
        if !path_str.ends_with(".pem") && !path_str.ends_with(".key") {
            return Err(ValidationError::new("Private key file should have .pem or .key extension"));
        }

        Ok(())
    }

    pub fn validate_ca_file(path: &&PathBuf) -> Result<(), ValidationError> {
        if path.to_string_lossy().is_empty() {
            return Err(ValidationError::new("CA certificate path cannot be empty"));
        }

        // Check for common CA file extensions
        let path_str = path.to_string_lossy().to_lowercase();
        if !path_str.ends_with(".pem") && !path_str.ends_with(".crt") && !path_str.ends_with(".cer") {
            return Err(ValidationError::new("CA certificate file should have .pem, .crt, or .cer extension"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_url_masking() {
        // Test database URL with password
        let url = "postgresql://user:secretpassword@host:5432/database";
        let masked = secure_loader::mask_database_url(url);
        assert_eq!(masked, "postgresql://user:****@host:5432/database");

        // Test database URL without password
        let url_no_pass = "postgresql://user@host:5432/database";
        let masked_no_pass = secure_loader::mask_database_url(url_no_pass);
        assert_eq!(masked_no_pass, url_no_pass);

        // Test non-database URL
        let regular_url = "https://example.com";
        let masked_regular = secure_loader::mask_database_url(regular_url);
        assert_eq!(masked_regular, secure_loader::mask_sensitive_value(regular_url));
    }

    #[test]
    fn test_sensitive_value_masking() {
        // Test long value
        let long_value = "this_is_a_very_long_secret_key_that_should_be_masked";
        let masked = secure_loader::mask_sensitive_value(long_value);
        assert_eq!(masked, "this_****asked");

        // Test short value
        let short_value = "abc";
        let masked_short = secure_loader::mask_sensitive_value(short_value);
        assert_eq!(masked_short, "***");
    }

    #[test]
    fn test_masked_config() {
        let config = AppConfig::new();

        // The config should fail validation because secrets aren't set, but masking should still work
        let masked_result = config.get_masked_config();
        assert!(masked_result.is_ok());

        let masked = masked_result.unwrap();
        assert_ne!(masked.security.jwt_secret, config.security.jwt_secret);
        assert!(masked.security.jwt_secret.contains("****"));
        assert_ne!(masked.security.encryption_key, config.security.encryption_key);
        assert!(masked.security.encryption_key.contains("****"));
    }

    #[test]
    fn test_tls_validation() {
        // Test valid certificate paths
        assert!(tls_validation::validate_cert_file(&PathBuf::from("cert.pem")).is_ok());
        assert!(tls_validation::validate_cert_file(&PathBuf::from("cert.crt")).is_ok());
        assert!(tls_validation::validate_cert_file(&PathBuf::from("cert.cer")).is_ok());

        // Test invalid certificate extensions
        assert!(tls_validation::validate_cert_file(&PathBuf::from("cert.txt")).is_err());

        // Test empty path
        assert!(tls_validation::validate_cert_file(&PathBuf::from("")).is_err());

        // Test valid key paths
        assert!(tls_validation::validate_key_file(&PathBuf::from("key.pem")).is_ok());
        assert!(tls_validation::validate_key_file(&PathBuf::from("key.key")).is_ok());

        // Test invalid key extensions
        assert!(tls_validation::validate_key_file(&PathBuf::from("key.txt")).is_err());
    }

    #[test]
    fn test_https_enforcement_in_production() {
        let mut config = AppConfig::new();
        config.app.environment = "production".to_string();

        // Should fail validation without TLS in production
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("TLS configuration is required"));

        // Add TLS config (but with non-existent files)
        config.server.tls = Some(TlsConfig {
            cert_path: PathBuf::from("nonexistent.pem"),
            key_path: PathBuf::from("nonexistent.key"),
            ca_path: None,
        });

        // Should fail because files don't exist
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
