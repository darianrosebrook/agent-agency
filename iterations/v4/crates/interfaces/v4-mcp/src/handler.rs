//! MCP Request Handler
//!
//! Handles JSON-RPC requests according to the MCP protocol.

use std::sync::Arc;

use crate::adapter::ToolAdapter;
use crate::error::MCPError;
use crate::types::{
    CallToolParams, InitializeParams, InitializeResult, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse, ListToolsResult, MCP_VERSION,
    ServerCapabilities, ServerInfo,
};

/// MCP request handler
pub struct MCPHandler {
    adapter: Arc<ToolAdapter>,
    server_name: String,
    server_version: String,
    initialized: std::sync::atomic::AtomicBool,
}

impl MCPHandler {
    /// Create a new handler
    pub fn new(adapter: Arc<ToolAdapter>) -> Self {
        Self {
            adapter,
            server_name: "v4-mcp".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Create with custom server info
    pub fn with_server_info(
        adapter: Arc<ToolAdapter>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            adapter,
            server_name: name.into(),
            server_version: version.into(),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Handle a JSON-RPC request
    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        match self.dispatch(request).await {
            Ok(result) => JsonRpcResponse::success(id, result),
            Err(err) => JsonRpcResponse::error(
                id,
                JsonRpcError {
                    code: err.error_code(),
                    message: err.to_string(),
                    data: None,
                },
            ),
        }
    }

    /// Dispatch request to appropriate method handler
    async fn dispatch(&self, request: JsonRpcRequest) -> Result<serde_json::Value, MCPError> {
        // Check if initialized for methods that require it
        if !self.is_initialized() && !self.is_init_method(&request.method) {
            return Err(MCPError::ProtocolError(
                "Server not initialized. Call initialize first.".to_string(),
            ));
        }

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "initialized" => self.handle_initialized().await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            "ping" => self.handle_ping().await,
            method => Err(MCPError::ProtocolError(format!(
                "Unknown method: {}",
                method
            ))),
        }
    }

    fn is_init_method(&self, method: &str) -> bool {
        matches!(method, "initialize" | "initialized" | "ping")
    }

    fn is_initialized(&self) -> bool {
        self.initialized
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Handle initialize request
    async fn handle_initialize(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, MCPError> {
        let _init_params: InitializeParams = params
            .map(|p| serde_json::from_value(p))
            .transpose()
            .map_err(|e| MCPError::InvalidParameter(e.to_string()))?
            .unwrap_or_else(|| InitializeParams {
                protocol_version: MCP_VERSION.to_string(),
                capabilities: Default::default(),
                client_info: crate::types::ClientInfo {
                    name: "unknown".to_string(),
                    version: "0.0.0".to_string(),
                },
            });

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: ServerInfo {
                name: self.server_name.clone(),
                version: self.server_version.clone(),
            },
        };

        Ok(serde_json::to_value(result)?)
    }

    /// Handle initialized notification
    async fn handle_initialized(&self) -> Result<serde_json::Value, MCPError> {
        self.initialized
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(serde_json::json!({}))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<serde_json::Value, MCPError> {
        let tools = self.adapter.list_tools();

        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };

        Ok(serde_json::to_value(result)?)
    }

    /// Handle tools/call request
    async fn handle_tools_call(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, MCPError> {
        let call_params: CallToolParams = params
            .ok_or_else(|| MCPError::MissingParameter("params".to_string()))
            .and_then(|p| {
                serde_json::from_value(p).map_err(|e| MCPError::InvalidParameter(e.to_string()))
            })?;

        let result = self.adapter.execute(call_params).await?;

        Ok(serde_json::to_value(result)?)
    }

    /// Handle ping request
    async fn handle_ping(&self) -> Result<serde_json::Value, MCPError> {
        Ok(serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_tools::{Tool, ToolCapability, ToolCategory, ToolMetadata, ToolRegistry};
    use v4_types::operators::OperatorResult;
    use async_trait::async_trait;

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn id(&self) -> &str {
            "test_tool"
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                id: "test_tool".to_string(),
                name: "Test Tool".to_string(),
                description: "A test tool".to_string(),
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
                data: Some(serde_json::json!("test output")),
                error: None,
                execution_time_ms: 5,
                content_hash: Some("abc123".to_string()),
            })
        }

        fn can_execute(&self, _operator: &v4_types::OperatorType) -> bool {
            true
        }
    }

    fn create_handler() -> MCPHandler {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockTool));
        let adapter = Arc::new(ToolAdapter::new(registry));
        MCPHandler::new(adapter)
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let handler = create_handler();

        let request = JsonRpcRequest::new("ping", None);
        let response = handler.handle(request).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let handler = create_handler();

        let params = serde_json::json!({
            "protocolVersion": MCP_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let request = JsonRpcRequest::new("initialize", Some(params));
        let response = handler.handle(request).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], MCP_VERSION);
        assert_eq!(result["serverInfo"]["name"], "v4-mcp");
    }

    #[tokio::test]
    async fn test_handle_tools_list_requires_init() {
        let handler = create_handler();

        // Without initialization
        let request = JsonRpcRequest::new("tools/list", None);
        let response = handler.handle(request).await;

        // Should fail because not initialized
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_handle_tools_list_after_init() {
        let handler = create_handler();

        // Initialize first
        let init_request = JsonRpcRequest::new("initialize", None);
        handler.handle(init_request).await;

        let notify_request = JsonRpcRequest::new("initialized", None);
        handler.handle(notify_request).await;

        // Now list tools
        let list_request = JsonRpcRequest::new("tools/list", None);
        let response = handler.handle(list_request).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "test_tool");
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let handler = create_handler();

        // Initialize first
        handler.handle(JsonRpcRequest::new("initialize", None)).await;
        handler.handle(JsonRpcRequest::new("initialized", None)).await;

        let request = JsonRpcRequest::new("unknown/method", None);
        let response = handler.handle(request).await;

        assert!(response.error.is_some());
        assert!(response.error.unwrap().message.contains("Unknown method"));
    }

    #[tokio::test]
    async fn test_custom_server_info() {
        let registry = Arc::new(ToolRegistry::new());
        let adapter = Arc::new(ToolAdapter::new(registry));
        let handler = MCPHandler::with_server_info(adapter, "custom-server", "2.0.0");

        let request = JsonRpcRequest::new("initialize", None);
        let response = handler.handle(request).await;

        let result = response.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "custom-server");
        assert_eq!(result["serverInfo"]["version"], "2.0.0");
    }
}
