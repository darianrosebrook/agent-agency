//! MCP Server
//!
//! Main MCP server implementation for handling tool requests and responses.

use crate::types::*;
use crate::{ToolRegistry, ToolDiscovery, CawsIntegration};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use jsonrpc_core::{IoHandler, Params, Value, Error as JsonRpcError, Result as JsonRpcResult};
use jsonrpc_http_server::{ServerBuilder};
use tokio::task::JoinHandle;

/// Main MCP server
#[derive(Debug)]
pub struct MCPServer {
    config: MCPConfig,
    tool_registry: Arc<ToolRegistry>,
    tool_discovery: Arc<ToolDiscovery>,
    caws_integration: Arc<CawsIntegration>,
    status: Arc<RwLock<MCPServerStatus>>,
    connections: Arc<RwLock<Vec<MCPConnection>>>,
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
        }
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        info!("Starting MCP server on {}:{}", self.config.server.host, self.config.server.port);

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

        info!("MCP server started successfully");
        Ok(())
    }

    /// Start the MCP HTTP server and return a readiness receiver and join handle for tests
    pub async fn start_http_with_readiness(&self) -> Result<(tokio::sync::oneshot::Receiver<()>, JoinHandle<()>)> {
        if !self.config.server.enable_http { anyhow::bail!("HTTP disabled"); }
        let (tx, rx) = tokio::sync::oneshot::channel();
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let registry = self.tool_registry.clone();
        let caws = self.caws_integration.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let mut io = IoHandler::default();
            io.add_sync_method("health", move |_| Ok(Value::String("ok".into())));
            let registry_list = registry.clone();
            io.add_method("tools", move |_| {
                let registry_list = registry_list.clone();
                async move { Ok(serde_json::to_value(&registry_list.get_all_tools().await).unwrap()) }
            });
            let caws_validate = caws.clone();
            io.add_method("validate", move |params: Params| {
                let caws_validate = caws_validate.clone();
                async move {
                    let v: Value = params.parse().unwrap_or(Value::Null);
                    let tool: crate::types::MCPTool = serde_json::from_value(v)
                        .map_err(|_| JsonRpcError::invalid_params("invalid tool"))?;
                    let res = caws_validate.validate_tool(&tool).await
                        .map_err(|_| JsonRpcError::internal_error())?;
                    Ok(serde_json::to_value(&res).unwrap())
                }
            });
            let server = ServerBuilder::new(io)
                .threads(1)
                .start_http(&addr.parse().expect("valid addr"))
                .expect("start http");
            let _ = tx.send(());
            server.wait();
        });
        Ok((rx, handle))
    }

    /// Stop the MCP server
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping MCP server");

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Stopping;
        }

        // Stop components
        self.tool_discovery.stop().await?;
        self.tool_registry.shutdown().await?;
        self.caws_integration.shutdown().await?;

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

        info!("MCP server stopped successfully");
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
        info!("Executing tool: {} (request: {})", request.tool_id, request.id);

        // Get tool from registry
        let tool = self.tool_registry.get_tool(request.tool_id).await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", request.tool_id))?;

        // Check CAWS compliance if enabled
        let caws_result = if self.config.caws_integration.enable_caws_checking {
            Some(self.caws_integration.validate_tool_execution(&tool, &request).await?)
        } else {
            None
        };

        // Execute tool
        let result = self.tool_registry.execute_tool(request.clone()).await?;

        // Update tool usage statistics
        self.tool_registry.update_tool_usage(request.tool_id).await?;

        info!("Tool execution completed: {} (status: {:?})", request.tool_id, result.status);
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

        info!("Tool discovery completed: {} tools discovered", result.discovered_tools.len());
        Ok(result)
    }

    /// Get tool registry statistics
    pub async fn get_registry_stats(&self) -> ToolRegistryStats {
        self.tool_registry.get_statistics().await
    }

    /// Test-only: register tool via server
    #[cfg(test)]
    pub async fn test_register_tool(&self, tool: MCPTool) -> Result<()> { self.tool_registry.register_tool(tool).await }

    /// Start HTTP server
    async fn start_http_server(&self) -> Result<()> {
        if !self.config.server.enable_http {
            return Ok(());
        }

        info!("Starting HTTP server on port {}", self.config.server.port);

        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let registry = self.tool_registry.clone();
        let _discovery = self.tool_discovery.clone();
        let caws = self.caws_integration.clone();

        // Readiness channel
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<()>(1);

        tokio::task::spawn_blocking(move || {
            let mut io = IoHandler::default();

            // /health equivalent
            io.add_sync_method("health", move |_| {
                Ok(Value::String("ok".into()))
            });

            // /tools - list tools
            let registry_list = registry.clone();
            io.add_method("tools", move |_| {
                let registry_list = registry_list.clone();
                async move {
                    let tools = registry_list.get_all_tools().await;
                    Ok(serde_json::to_value(&tools).unwrap())
                }
            });

            // /validate - CAWS validate a pseudo tool manifest (expects an MCPTool)
            let caws_validate = caws.clone();
            io.add_method("validate", move |params: Params| {
                let caws_validate = caws_validate.clone();
                async move {
                    let v: Value = params.parse().unwrap_or(Value::Null);
                    let tool: crate::types::MCPTool = serde_json::from_value(v)
                        .map_err(|e| JsonRpcError::invalid_params(format!("invalid tool: {e}")))?;
                    let res = caws_validate.validate_tool(&tool).await
                        .map_err(|e| JsonRpcError::internal_error())?;
                    Ok(serde_json::to_value(&res).unwrap())
                }
            });

            let server = ServerBuilder::new(io)
                .threads(1)
                .start_http(&addr.parse().expect("valid addr"))
                .expect("start http");
            // signal readiness
            let _ = ready_tx.send(());
            // Keep server running until process end in this background thread
            server.wait();
        });
        // Wait for readiness (with timeout safeguard)
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(3) {
            if let Ok(_) = ready_rx.try_recv() { return Ok(()); }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        anyhow::bail!("HTTP server failed to become ready in time")
    }

    /// Start WebSocket server
    async fn start_websocket_server(&self) -> Result<()> {
        if !self.config.server.enable_websocket {
            return Ok(());
        }

        info!("Starting WebSocket server on port {}", self.config.server.port + 1);
        // Minimal stub: not implementing bidirectional streaming in this step
        Ok(())
    }
}
