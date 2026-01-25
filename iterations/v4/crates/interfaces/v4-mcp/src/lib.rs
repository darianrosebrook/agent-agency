//! v4-mcp - Model Context Protocol Server
//!
//! This crate provides an MCP-compliant server for Agent Agency V4,
//! exposing v4-tools through the standard Model Context Protocol.
//!
//! ## Overview
//!
//! The Model Context Protocol (MCP) is a standard for LLM tool integration.
//! This crate wraps the v4-tools abstraction to expose tools via MCP's
//! JSON-RPC interface.
//!
//! ## Features
//!
//! - **Tool exposure**: All v4-tools are accessible through MCP
//! - **JSON-RPC 2.0**: Standard protocol compliance
//! - **HTTP transport**: Simple HTTP POST interface
//! - **Governance integration**: CAWS gates can be enforced on tool calls
//!
//! ## Usage
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use v4_mcp::{MCPServer, MCPServerConfig, ToolAdapter};
//! use v4_tools::ToolRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create tool registry and register tools
//!     let registry = Arc::new(ToolRegistry::new());
//!     // registry.register(Arc::new(YourTool));
//!
//!     // Create MCP adapter and server
//!     let adapter = Arc::new(ToolAdapter::new(registry));
//!     let config = MCPServerConfig::default().with_port(3000);
//!     let server = MCPServer::new(adapter, config);
//!
//!     // Start serving
//!     server.serve().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Protocol
//!
//! The server implements these MCP methods:
//!
//! | Method | Description |
//! |--------|-------------|
//! | `initialize` | Initialize connection with client capabilities |
//! | `initialized` | Notification that initialization is complete |
//! | `tools/list` | List all available tools |
//! | `tools/call` | Execute a tool with arguments |
//! | `ping` | Health check |
//!
//! ## Architecture
//!
//! ```text
//! MCP Client (Claude, etc.)
//!     │
//!     │ JSON-RPC over HTTP POST
//!     ▼
//! ┌─────────────────┐
//! │   MCPServer     │  HTTP server (Axum)
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │   MCPHandler    │  Protocol dispatch
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │   ToolAdapter   │  Convert MCP ↔ v4 types
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │   v4-tools      │  Execute operators
//! └─────────────────┘
//! ```

pub mod adapter;
pub mod error;
pub mod handler;
pub mod server;
pub mod types;

// Re-exports
pub use adapter::ToolAdapter;
pub use error::MCPError;
pub use handler::MCPHandler;
pub use server::{MCPServer, MCPServerConfig};
pub use types::{
    CallToolParams, CallToolResult, InitializeParams, InitializeResult, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse, ListToolsResult, MCPTool, MCP_VERSION, ServerCapabilities,
    ServerInfo, ToolContent,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use v4_tools::{Tool, ToolCapability, ToolCategory, ToolMetadata, ToolRegistry};
    use v4_types::operators::OperatorResult;
    use async_trait::async_trait;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn id(&self) -> &str {
            "test"
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "Test tool".to_string(),
                version: "1.0.0".to_string(),
                category: ToolCategory::FileSystem,
                capabilities: vec![ToolCapability::ReadOnly],
                supported_operators: vec!["S".to_string()],
                requires_sandbox: false,
            }
        }

        async fn execute(
            &self,
            operator: &v4_types::OperatorType,
        ) -> Result<OperatorResult, v4_tools::ToolError> {
            Ok(OperatorResult {
                operator: operator.clone(),
                success: true,
                data: Some(serde_json::json!("ok")),
                error: None,
                execution_time_ms: 5,
                content_hash: Some("x".to_string()),
            })
        }

        fn can_execute(&self, _operator: &v4_types::OperatorType) -> bool {
            true
        }
    }

    #[test]
    fn test_crate_exports() {
        // Verify key types are accessible
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(TestTool));

        let _adapter = ToolAdapter::new(registry.clone());
        let _config = MCPServerConfig::default();
    }

    #[tokio::test]
    async fn test_end_to_end_flow() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(TestTool));

        let adapter = Arc::new(ToolAdapter::new(registry));
        let handler = MCPHandler::new(adapter);

        // Initialize
        let init_req = JsonRpcRequest::new("initialize", None);
        let init_resp = handler.handle(init_req).await;
        assert!(init_resp.error.is_none());

        // Confirm initialized
        let notify_req = JsonRpcRequest::new("initialized", None);
        handler.handle(notify_req).await;

        // List tools
        let list_req = JsonRpcRequest::new("tools/list", None);
        let list_resp = handler.handle(list_req).await;
        assert!(list_resp.error.is_none());

        let result = list_resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "test");
    }
}
