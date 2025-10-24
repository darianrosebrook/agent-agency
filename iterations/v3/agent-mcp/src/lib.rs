#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Agent Agency V3 - MCP Integration
//!
//! Provides Model Context Protocol (MCP) server integration for CAWS tool discovery,
//! modular extension, and seamless integration with external tools and services.

pub mod caws_integration;
pub mod server;
pub mod tool_discovery;
pub mod tool_registry;
pub mod tools;
pub mod types;

pub use caws_integration::CawsIntegration;
pub use server::{MCPServer, AuthRateLimitStats};
pub use agent_agency_council::error_handling::CircuitBreakerStats;
pub use tool_discovery::ToolDiscovery;
pub use tool_registry::ToolRegistry;
pub use tools::*;
pub use types::{
    MCPTool,
    ToolExecutionRequest,
    ToolExecutionResult,
    ToolDiscoveryResult,
    ToolRegistryStats,
    CawsComplianceResult,
    MCPConnection,
    MCPServerStatus,
};

/// MCP integration configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Tool discovery configuration
    pub tool_discovery: ToolDiscoveryConfig,
    /// CAWS integration configuration
    pub caws_integration: CawsIntegrationConfig,
    /// Tool registry configuration
    pub tool_registry: ToolRegistryConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    pub server_name: String,
    pub version: String,
    pub host: String,
    pub port: u16,
    pub enable_tls: bool,
    pub enable_http: bool,
    pub enable_websocket: bool,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
    pub enable_compression: bool,
    pub log_level: String,
    pub auth_api_key: Option<String>,
    pub requests_per_minute: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDiscoveryConfig {
    /// Enable automatic tool discovery
    pub enable_auto_discovery: bool,
    /// Tool discovery paths
    pub discovery_paths: Vec<String>,
    /// Tool manifest file patterns
    pub manifest_patterns: Vec<String>,
    /// Discovery interval in seconds
    pub discovery_interval_seconds: u64,
    /// Enable tool validation
    pub enable_validation: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CawsIntegrationConfig {
    /// Enable CAWS compliance checking
    pub enable_caws_checking: bool,
    /// CAWS rulebook path
    pub caws_rulebook_path: String,
    /// Enable provenance tracking
    pub enable_provenance: bool,
    /// Enable quality gates
    pub enable_quality_gates: bool,
    /// CAWS validation strictness
    pub validation_strictness: ValidationStrictness,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolRegistryConfig {
    /// Enable tool registration
    pub enable_registration: bool,
    /// Tool registry storage path
    pub registry_path: String,
    /// Enable tool versioning
    pub enable_versioning: bool,
    /// Maximum tool versions to keep
    pub max_versions: u32,
    /// Enable tool metadata indexing
    pub enable_indexing: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent tool executions
    pub max_concurrent_executions: u32,
    /// Tool execution timeout in seconds
    pub execution_timeout_seconds: u64,
    /// Enable execution caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationStrictness {
    /// Strict validation - all rules must pass
    Strict,
    /// Moderate validation - critical rules must pass
    Moderate,
    /// Lenient validation - warnings only
    Lenient,
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                server_name: "agent-agency-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                host: "127.0.0.1".to_string(),
                port: 8080,
                enable_tls: false,
                enable_websocket: true,
                enable_http: true,
                max_connections: 100,
                connection_timeout_ms: 300_000,
                enable_compression: false,
                log_level: "info".to_string(),
                auth_api_key: None,
                requests_per_minute: None,
            },
            tool_discovery: ToolDiscoveryConfig {
                enable_auto_discovery: true,
                discovery_paths: vec!["./tools".to_string(), "./extensions".to_string()],
                manifest_patterns: vec!["**/tool.json".to_string(), "**/manifest.toml".to_string()],
                discovery_interval_seconds: 60,
                enable_validation: true,
            },
            caws_integration: CawsIntegrationConfig {
                enable_caws_checking: true,
                caws_rulebook_path: "./caws".to_string(),
                enable_provenance: true,
                enable_quality_gates: true,
                validation_strictness: ValidationStrictness::Moderate,
            },
            tool_registry: ToolRegistryConfig {
                enable_registration: true,
                registry_path: "./registry".to_string(),
                enable_versioning: true,
                max_versions: 10,
                enable_indexing: true,
            },
            performance: PerformanceConfig {
                max_concurrent_executions: 20,
                execution_timeout_seconds: 30,
                enable_caching: true,
                cache_ttl_seconds: 3600,
                enable_monitoring: true,
            },
        }
    }
}
