//! MCP server interface
//!
//! Provides a clean interface layer for MCP (Model Context Protocol) server functionality,
//! bridging the sophisticated MCP integration with the rest of the Agent Agency system.

use agent_agency_mcp::{
    MCPServer as InnerMCPServer,
    MCPConfig,
    ServerConfig,
    ToolDiscoveryConfig,
    CawsIntegrationConfig,
    ToolRegistryConfig,
    PerformanceConfig,
    ValidationStrictness,
    MCPServerStatus,
    ToolExecutionRequest,
    ToolExecutionResult,
    MCPTool,
    ToolDiscoveryResult,
    ToolRegistryStats,
    CawsComplianceResult,
    server::CircuitBreakerStats,
    server::AuthRateLimitStats,
    MCPConnection,
};
use agent_agency_database::DatabaseClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for the MCP interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Tool discovery configuration
    pub tool_discovery: ToolDiscoveryConfig,
    /// CAWS integration configuration
    pub caws_integration: CawsIntegrationConfig,
    /// Tool registry configuration
    pub tool_registry: ToolRegistryConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                server_name: "agent-agency-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                host: "127.0.0.1".to_string(),
                port: 8080,
                enable_tls: false,
                enable_http: true,
                enable_websocket: true,
                max_connections: 100,
                connection_timeout_ms: 300_000,
                enable_compression: false,
                log_level: "info".to_string(),
                auth_api_key: None,
                requests_per_minute: Some(100),
            },
            tool_discovery: ToolDiscoveryConfig {
                enable_auto_discovery: true,
                discovery_paths: vec!["./tools".to_string(), "./extensions".to_string()],
                manifest_patterns: vec!["**/tool.json".to_string(), "**/manifest.toml".to_string()],
                discovery_interval_seconds: 60,
                enable_validation: true,
                enable_health_checks: true,
                health_check_timeout_seconds: 10,
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

/// Main MCP server interface
pub struct McpServer {
    /// Inner MCP server implementation
    inner: Arc<RwLock<InnerMCPServer>>,
    /// Tool discovery service
    tool_discovery: Arc<ToolDiscovery>,
    /// Tool registry service
    tool_registry: Arc<ToolRegistry>,
    /// CAWS integration service
    caws_integration: Arc<CawsIntegration>,
    /// Configuration
    config: McpConfig,
}

impl McpServer {
    /// Create a new MCP server instance
    pub async fn new(config: McpConfig, db_client: Arc<DatabaseClient>) -> Result<Self> {
        // Convert interface config to MCP integration config
        let inner_config = MCPConfig {
            server: config.server.clone(),
            tool_discovery: config.tool_discovery.clone(),
            caws_integration: config.caws_integration.clone(),
            tool_registry: config.tool_registry.clone(),
            performance: config.performance.clone(),
        };

        // Create inner MCP server
        let inner = InnerMCPServer::new(inner_config, db_client)
            .context("Failed to create MCP server")?;

        // Create service components
        let tool_discovery = Arc::new(ToolDiscovery::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        let caws_integration = Arc::new(CawsIntegration::new());

        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
            tool_discovery,
            tool_registry,
            caws_integration,
            config,
        })
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        tracing::info!(
            "Starting MCP interface server on {}:{}",
            self.config.server.host,
            self.config.server.port
        );

        // Initialize components
        self.tool_discovery.initialize().await?;
        self.tool_registry.initialize().await?;
        self.caws_integration.initialize().await?;

        // Start auto-discovery if enabled
        if self.config.tool_discovery.enable_auto_discovery {
            self.tool_discovery.start_auto_discovery().await?;
        }

        // Start the inner MCP server
        let inner = self.inner.read().await;
        inner.start().await?;

        tracing::info!("MCP interface server started successfully");
        Ok(())
    }

    /// Stop the MCP server
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping MCP interface server");

        // Stop auto-discovery
        self.tool_discovery.stop().await?;

        // Stop the inner server
        let inner = self.inner.read().await;
        inner.stop().await?;

        tracing::info!("MCP interface server stopped successfully");
        Ok(())
    }

    /// Get server status
    pub async fn status(&self) -> Result<MCPServerStatus> {
        let inner = self.inner.read().await;
        Ok(inner.get_status())
    }

    /// Execute a tool through the MCP server
    pub async fn execute_tool(&self, request: ToolExecutionRequest) -> Result<ToolExecutionResult> {
        let inner = self.inner.read().await;
        inner.execute_tool(request).await
    }

    /// Register a tool with the MCP server
    pub async fn register_tool(&self, tool: MCPTool) -> Result<()> {
        // Validate tool before registration
        if self.config.tool_discovery.enable_validation {
            let validation_result = self.tool_discovery.validate_tool(&tool).await?;
            if !validation_result.is_valid {
                return Err(anyhow::anyhow!(
                    "Tool validation failed: {:?}",
                    validation_result.errors
                ));
            }
        }

        // Check CAWS compliance if enabled
        if self.config.caws_integration.enable_caws_checking {
            let compliance_result = self.caws_integration.validate_tool(&tool).await?;
            if !compliance_result.is_compliant {
                return Err(anyhow::anyhow!(
                    "CAWS compliance check failed: {:?}",
                    compliance_result.violations
                ));
            }
        }

        // Register with MCP server
        let inner = self.inner.read().await;
        inner.test_register_tool(tool).await
    }

    /// Discover tools from configured paths
    pub async fn discover_tools(&self) -> Result<ToolDiscoveryResult> {
        self.tool_discovery.discover_tools().await
    }

    /// Get all registered tools
    pub async fn get_registered_tools(&self) -> Result<Vec<MCPTool>> {
        let inner = self.inner.read().await;
        Ok(inner.get_registry_stats().most_used_tools)
    }

    /// Get tool registry statistics
    pub async fn get_statistics(&self) -> Result<ToolRegistryStats> {
        let inner = self.inner.read().await;
        Ok(inner.get_registry_stats().await)
    }

    /// Validate a tool against CAWS rules
    pub async fn validate_tool_caws(&self, tool: &MCPTool) -> Result<CawsComplianceResult> {
        self.caws_integration.validate_tool(tool).await
    }

    /// Get authentication rate limiting statistics
    pub async fn get_auth_rate_limit_stats(&self) -> Result<Option<AuthRateLimitStats>> {
        let inner = self.inner.read().await;
        Ok(inner.get_auth_rate_limit_stats().await)
    }

    /// Get circuit breaker statistics
    pub async fn get_circuit_breaker_stats(&self) -> Result<HashMap<String, CircuitBreakerStats>> {
        let inner = self.inner.read().await;
        Ok(inner.get_circuit_breaker_stats().await)
    }

    /// Get API rate limiting statistics
    pub async fn get_api_rate_limit_stats(&self) -> Result<Option<HashMap<String, (u32, u32)>>> {
        let inner = self.inner.read().await;
        Ok(inner.get_api_rate_limit_stats().await)
    }

    /// Get active connections
    pub async fn get_active_connections(&self) -> Result<Vec<MCPConnection>> {
        let inner = self.inner.read().await;
        Ok(inner.get_connections().await)
    }
}

impl Drop for McpServer {
    fn drop(&mut self) {
        // Note: In a real implementation, we'd want to gracefully shutdown
        // but Drop doesn't allow async operations
        tracing::debug!("MCP server interface dropped");
    }
}

/// Builder pattern for MCP server configuration
pub struct McpServerBuilder {
    config: McpConfig,
    db_client: Option<Arc<DatabaseClient>>,
}

impl McpServerBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: McpConfig::default(),
            db_client: None,
        }
    }

    /// Set the server configuration
    pub fn with_config(mut self, config: McpConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the database client
    pub fn with_database_client(mut self, db_client: Arc<DatabaseClient>) -> Self {
        self.db_client = Some(db_client);
        self
    }

    /// Set server host and port
    pub fn with_address(mut self, host: impl Into<String>, port: u16) -> Self {
        self.config.server.host = host.into();
        self.config.server.port = port;
        self
    }

    /// Enable or disable auto tool discovery
    pub fn with_auto_discovery(mut self, enabled: bool) -> Self {
        self.config.tool_discovery.enable_auto_discovery = enabled;
        self
    }

    /// Set tool discovery paths
    pub fn with_discovery_paths(mut self, paths: Vec<String>) -> Self {
        self.config.tool_discovery.discovery_paths = paths;
        self
    }

    /// Enable or disable CAWS checking
    pub fn with_caws_checking(mut self, enabled: bool) -> Self {
        self.config.caws_integration.enable_caws_checking = enabled;
        self
    }

    /// Set CAWS rulebook path
    pub fn with_caws_rulebook(mut self, path: impl Into<String>) -> Self {
        self.config.caws_integration.caws_rulebook_path = path.into();
        self
    }

    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.server.auth_api_key = Some(api_key.into());
        self
    }

    /// Build the MCP server
    pub async fn build(self) -> Result<McpServer> {
        let db_client = self.db_client
            .ok_or_else(|| anyhow::anyhow!("Database client is required"))?;

        McpServer::new(self.config, db_client).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_mcp_server_builder() {
        let db_client = Arc::new(DatabaseClient::new());

        let server = McpServerBuilder::new()
            .with_address("127.0.0.1", 9090)
            .with_auto_discovery(true)
            .with_caws_checking(true)
            .with_database_client(db_client)
            .build()
            .await;

        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_default_config() {
        let config = McpConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "127.0.0.1");
        assert!(config.tool_discovery.enable_auto_discovery);
    }
}
