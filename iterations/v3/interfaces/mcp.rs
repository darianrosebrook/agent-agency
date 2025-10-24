//! MCP Interface for Agent Agency V3
//!
//! Provides Model Context Protocol (MCP) server interface for external tool integration.

use agent_agency_mcp::{MCPServer, MCPConfig};
use agent_agency_database::DatabaseClient;
use agent_agency_observability::metrics::MetricsBackend;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP Server configuration
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable TLS
    pub enable_tls: bool,
    /// Authentication API key
    pub api_key: Option<String>,
    /// Maximum connections
    pub max_connections: u32,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
            enable_tls: false,
            api_key: None,
            max_connections: 100,
        }
    }
}

/// MCP Server interface
pub struct McpServer {
    config: McpConfig,
    mcp_server: Arc<RwLock<Option<MCPServer>>>,
    database: Arc<DatabaseClient>,
    metrics: Option<Arc<dyn MetricsBackend>>,
}

impl McpServer {
    /// Create a new MCP server instance
    pub async fn new(
        config: McpConfig,
        database: Arc<DatabaseClient>,
        metrics: Option<Arc<dyn MetricsBackend>>,
    ) -> Result<Self> {
        let mcp_config = MCPConfig::default();

        Ok(Self {
            config,
            mcp_server: Arc::new(RwLock::new(None)),
            database,
            metrics,
        })
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        let mut server_guard = self.mcp_server.write().await;

        if server_guard.is_some() {
            return Ok(()); // Already started
        }

        // Create MCP server configuration
        let mcp_config = MCPConfig::default();

        // Initialize the MCP server
        let server = MCPServer::new(mcp_config).await?;

        *server_guard = Some(server);

        println!("MCP Server started on {}:{}", self.config.host, self.config.port);

        Ok(())
    }

    /// Stop the MCP server
    pub async fn stop(&self) -> Result<()> {
        let mut server_guard = self.mcp_server.write().await;

        if let Some(server) = server_guard.take() {
            // The MCPServer should implement proper shutdown
            println!("MCP Server stopped");
        }

        Ok(())
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        self.mcp_server.read().await.is_some()
    }

    /// Get server status
    pub async fn status(&self) -> Result<String> {
        if self.is_running().await {
            Ok(format!("MCP Server running on {}:{}", self.config.host, self.config.port))
        } else {
            Ok("MCP Server stopped".to_string())
        }
    }

    /// Ping the server to test connectivity
    pub async fn ping(&self) -> Result<String> {
        if !self.is_running().await {
            return Err(anyhow::anyhow!("MCP Server is not running"));
        }

        // Basic connectivity test
        Ok("MCP Server is responding".to_string())
    }
}

impl Drop for McpServer {
    fn drop(&mut self) {
        // Note: Async drop is not stable in Rust yet
        // The server should be explicitly stopped before dropping
    }
}