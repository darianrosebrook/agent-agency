//! MCP Server
//!
//! Main MCP server implementation for handling tool requests and responses.

use crate::types::*;
use crate::{CawsIntegration, ToolDiscovery, ToolRegistry};
use anyhow::{anyhow, bail, Result};
use jsonrpc_core::{Error as JsonRpcError, IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_ws_server::ServerBuilder as WsServerBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::info;

/// Handle used to shutdown the HTTP server gracefully.
#[derive(Debug)]
pub struct HttpServerHandle {
    join_handle: JoinHandle<()>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl HttpServerHandle {
    /// Gracefully shutdown the HTTP server.
    pub async fn shutdown(mut self) -> Result<()> {
        info!("Shutting down HTTP server");

        if let Some(tx) = self.shutdown_tx.take() {
            // Ignore error if thread has already exited.
            let _ = tx.send(());
        }

        self.join_handle
            .await
            .map_err(|err| anyhow!("HTTP server task failed: {}", err))?;

        info!("HTTP server shutdown complete");
        Ok(())
    }
}

/// Main MCP server
#[derive(Debug)]
pub struct MCPServer {
    config: MCPConfig,
    tool_registry: Arc<ToolRegistry>,
    tool_discovery: Arc<ToolDiscovery>,
    caws_integration: Arc<CawsIntegration>,
    status: Arc<RwLock<MCPServerStatus>>,
    connections: Arc<RwLock<Vec<MCPConnection>>>,
    http_handle: Arc<RwLock<Option<HttpServerHandle>>>,
    ws_handle: Arc<RwLock<Option<HttpServerHandle>>>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new(config: MCPConfig) -> Self {
        Self {
            config,
            tool_registry: Arc::new(ToolRegistry::new()),
            tool_discovery: Arc::new(ToolDiscovery::new()),
            caws_integration: Arc::new(CawsIntegration::new()),
            status: Arc::new(RwLock::new(MCPServerStatus::Starting)),
            connections: Arc::new(RwLock::new(Vec::new())),
            http_handle: Arc::new(RwLock::new(None)),
            ws_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        info!(
            server_name = %self.config.server.server_name,
            version = %self.config.server.version,
            host = %self.config.server.host,
            port = %self.config.server.port,
            "Starting MCP server"
        );

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Starting;
        }

        // Initialize components
        self.tool_discovery.initialize().await?;
        self.tool_registry.initialize().await?;
        self.caws_integration.initialize().await?;

        // Start discovery process
        if self.config.tool_discovery.enable_auto_discovery {
            self.tool_discovery.start_auto_discovery().await?;
        }

        // Start server listeners
        self.start_http_server().await?;
        self.start_websocket_server().await?;

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Running;
        }

        info!(
            server_name = %self.config.server.server_name,
            status = "running",
            "MCP server started successfully"
        );
        Ok(())
    }

    /// Spawn the MCP HTTP server and return a readiness receiver plus handle.
    async fn spawn_http_server(&self) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        if !self.config.server.enable_http {
            bail!("HTTP disabled");
        }

        let (ready_tx, ready_rx) = oneshot::channel();
        let (stop_tx, stop_rx) = oneshot::channel();

        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let registry = self.tool_registry.clone();
        let caws = self.caws_integration.clone();
        let registry_for_stats = self.tool_registry.clone();
        let version_payload = Arc::new(serde_json::json!({
            "name": self.config.server.server_name.clone(),
            "version": self.config.server.version.clone(),
        }));

        let handle = tokio::task::spawn_blocking(move || {
            let io = Self::build_io_handler(
                registry.clone(),
                registry_for_stats.clone(),
                caws.clone(),
                version_payload.clone(),
            );
            let server = ServerBuilder::new(io)
                .threads(1)
                .start_http(&addr.parse().expect("valid addr"))
                .expect("start http");
            let _ = ready_tx.send(());
            let _ = stop_rx.blocking_recv();
            server.close();
        });

        let http_handle = HttpServerHandle {
            join_handle: handle,
            shutdown_tx: Some(stop_tx),
        };

        Ok((ready_rx, http_handle))
    }

    fn build_io_handler(
        registry: Arc<ToolRegistry>,
        registry_stats: Arc<ToolRegistry>,
        caws: Arc<CawsIntegration>,
        version_payload: Arc<serde_json::Value>,
    ) -> IoHandler<()> {
        let mut io = IoHandler::default();

        io.add_sync_method("health", move |_| Ok(Value::String("ok".into())));

        let registry_for_tools = registry.clone();
        io.add_method("tools", move |_| {
            let registry_for_tools = registry_for_tools.clone();
            async move { Ok(serde_json::to_value(&registry_for_tools.get_all_tools().await).unwrap()) }
        });

        let registry_for_stats = registry_stats.clone();
        io.add_method("stats", move |_| {
            let registry_for_stats = registry_for_stats.clone();
            async move {
                let stats = registry_for_stats.get_statistics().await;
                Ok(serde_json::to_value(&stats).unwrap())
            }
        });

        let version_payload = version_payload.clone();
        io.add_sync_method("version", move |_| Ok(version_payload.as_ref().clone()));

        let caws_validate = caws.clone();
        io.add_method("validate", move |params: Params| {
            let caws_validate = caws_validate.clone();
            async move {
                let v: Value = params.parse().unwrap_or(Value::Null);
                let tool: crate::types::MCPTool =
                    serde_json::from_value(v).map_err(|e| JsonRpcError {
                        code: jsonrpc_core::ErrorCode::InvalidParams,
                        message: "Invalid tool format".to_string(),
                        data: Some(serde_json::Value::String(e.to_string())),
                    })?;
                let res = caws_validate
                    .validate_tool(&tool)
                    .await
                    .map_err(|e| JsonRpcError {
                        code: jsonrpc_core::ErrorCode::InternalError,
                        message: "Tool validation failed".to_string(),
                        data: Some(serde_json::Value::String(e.to_string())),
                    })?;
                Ok(serde_json::to_value(&res).unwrap())
            }
        });

        io
    }

    /// Start the MCP HTTP server and return a readiness receiver and structured handle for tests.
    pub async fn start_http_with_readiness(
        &self,
    ) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        self.spawn_http_server().await
    }

    pub async fn start_ws_with_readiness(
        &self,
    ) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        self.spawn_websocket_server().await
    }

    async fn spawn_websocket_server(&self) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        if !self.config.server.enable_websocket {
            bail!("WebSocket disabled");
        }

        let (ready_tx, ready_rx) = oneshot::channel();
        let (stop_tx, stop_rx) = oneshot::channel();

        let port = self.config.server.port + 1;
        let addr: SocketAddr = format!("{}:{}", self.config.server.host, port).parse()?;
        let registry = self.tool_registry.clone();
        let registry_stats = self.tool_registry.clone();
        let caws = self.caws_integration.clone();
        let version_payload = Arc::new(serde_json::json!({
            "name": self.config.server.server_name.clone(),
            "version": self.config.server.version.clone(),
        }));

        let handle = tokio::task::spawn_blocking(move || {
            let io = MCPServer::build_io_handler(
                registry.clone(),
                registry_stats.clone(),
                caws.clone(),
                version_payload.clone(),
            );

            let server = WsServerBuilder::new(io)
                .start(&addr)
                .expect("start websocket server");
            let close_handle = server.close_handle();
            let _ = ready_tx.send(());
            let _ = stop_rx.blocking_recv();
            close_handle.close();
            let _ = server.wait();
        });

        let ws_handle = HttpServerHandle {
            join_handle: handle,
            shutdown_tx: Some(stop_tx),
        };

        Ok((ready_rx, ws_handle))
    }

    /// Stop the MCP server
    pub async fn stop(&self) -> Result<()> {
        info!(
            server_name = %self.config.server.server_name,
            "Stopping MCP server"
        );

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Stopping;
        }

        // Stop components
        self.tool_discovery.stop().await?;
        self.tool_registry.shutdown().await?;
        self.caws_integration.shutdown().await?;

        if let Some(handle) = self.http_handle.write().await.take() {
            handle.shutdown().await?;
        }
        if let Some(handle) = self.ws_handle.write().await.take() {
            handle.shutdown().await?;
        }

        // Close all connections
        {
            let mut connections = self.connections.write().await;
            connections.clear();
        }

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Stopped;
        }

        info!(
            server_name = %self.config.server.server_name,
            status = "stopped",
            "MCP server stopped successfully"
        );
        Ok(())
    }

    /// Get server status
    pub async fn get_status(&self) -> MCPServerStatus {
        let status = self.status.read().await;
        status.clone()
    }

    /// Get active connections
    pub async fn get_connections(&self) -> Vec<MCPConnection> {
        let connections = self.connections.read().await;
        connections.clone()
    }

    /// Execute a tool
    pub async fn execute_tool(&self, request: ToolExecutionRequest) -> Result<ToolExecutionResult> {
        info!(
            "Executing tool: {} (request: {})",
            request.tool_id, request.id
        );

        // Get tool from registry
        let tool = self
            .tool_registry
            .get_tool(request.tool_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", request.tool_id))?;

        // Check CAWS compliance if enabled
        let _caws_result = if self.config.caws_integration.enable_caws_checking {
            Some(
                self.caws_integration
                    .validate_tool_execution(&tool, &request)
                    .await?,
            )
        } else {
            None
        };

        // Execute tool
        let result = self.tool_registry.execute_tool(request.clone()).await?;

        // Update tool usage statistics
        self.tool_registry
            .update_tool_usage(request.tool_id)
            .await?;

        info!(
            "Tool execution completed: {} (status: {:?})",
            request.tool_id, result.status
        );
        Ok(result)
    }

    // Test helper: register a tool directly in registry
    #[cfg(test)]
    pub async fn execute_tool_registry_register(&self, tool: MCPTool) {
        let _ = self.tool_registry.register_tool(tool).await;
    }

    /// Discover and register tools
    pub async fn discover_tools(&self) -> Result<ToolDiscoveryResult> {
        info!("Starting tool discovery");

        let result = self.tool_discovery.discover_tools().await?;

        // Register discovered tools
        for tool in &result.discovered_tools {
            self.tool_registry.register_tool(tool.clone()).await?;
        }

        info!(
            "Tool discovery completed: {} tools discovered",
            result.discovered_tools.len()
        );
        Ok(result)
    }

    /// Get tool registry statistics
    pub async fn get_registry_stats(&self) -> ToolRegistryStats {
        self.tool_registry.get_statistics().await
    }

    /// Test-only: register tool via server
    #[cfg(test)]
    pub async fn test_register_tool(&self, tool: MCPTool) -> Result<()> {
        self.tool_registry.register_tool(tool).await
    }

    /// Register tool for testing purposes (feature-gated for test utilities)
    #[cfg(feature = "test-utils")]
    pub async fn register_tool_for_testing(&self, tool: MCPTool) -> Result<()> {
        info!("Registering tool for testing: {}", tool.name);
        self.tool_registry.register_tool(tool).await
    }

    /// Start HTTP server
    async fn start_http_server(&self) -> Result<()> {
        if !self.config.server.enable_http {
            return Ok(());
        }

        info!("Starting HTTP server on port {}", self.config.server.port);

        let (ready, handle) = self.spawn_http_server().await?;

        match timeout(Duration::from_secs(3), ready).await {
            Ok(Ok(())) => {
                let mut slot = self.http_handle.write().await;
                *slot = Some(handle);
                Ok(())
            }
            Ok(Err(_)) => {
                handle.shutdown().await?;
                bail!("HTTP server task ended before readiness");
            }
            Err(_) => {
                handle.shutdown().await?;
                bail!("HTTP server failed to become ready in time");
            }
        }
    }

    /// Start WebSocket server
    async fn start_websocket_server(&self) -> Result<()> {
        if !self.config.server.enable_websocket {
            return Ok(());
        }

        info!(
            "Starting WebSocket server on port {}",
            self.config.server.port + 1
        );

        let (ready, handle) = self.spawn_websocket_server().await?;

        match timeout(Duration::from_secs(3), ready).await {
            Ok(Ok(())) => {
                let mut slot = self.ws_handle.write().await;
                *slot = Some(handle);
                Ok(())
            }
            Ok(Err(_)) => {
                handle.shutdown().await?;
                bail!("WebSocket server task ended before readiness");
            }
            Err(_) => {
                handle.shutdown().await?;
                bail!("WebSocket server failed to become ready in time");
            }
        }
    }
}
