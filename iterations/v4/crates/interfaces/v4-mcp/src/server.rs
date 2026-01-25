//! MCP HTTP Server
//!
//! Axum-based HTTP server implementing the MCP protocol over HTTP POST.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::adapter::ToolAdapter;
use crate::handler::MCPHandler;
use crate::types::{JsonRpcRequest, JsonRpcResponse};

/// MCP Server configuration
#[derive(Debug, Clone)]
pub struct MCPServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Server name for MCP protocol
    pub server_name: String,
    /// Server version
    pub server_version: String,
}

impl Default for MCPServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            server_name: "v4-mcp".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl MCPServerConfig {
    /// Create config with custom port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Create config with custom host
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Create config with custom server info
    pub fn with_server_info(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.server_name = name.into();
        self.server_version = version.into();
        self
    }

    /// Get socket address
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

/// Shared state for the server
#[derive(Clone)]
pub struct AppState {
    handler: Arc<MCPHandler>,
}

/// MCP Server
pub struct MCPServer {
    config: MCPServerConfig,
    handler: Arc<MCPHandler>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new(adapter: Arc<ToolAdapter>, config: MCPServerConfig) -> Self {
        let handler = Arc::new(MCPHandler::with_server_info(
            adapter,
            config.server_name.clone(),
            config.server_version.clone(),
        ));

        Self { config, handler }
    }

    /// Create with default config
    pub fn with_adapter(adapter: Arc<ToolAdapter>) -> Self {
        Self::new(adapter, MCPServerConfig::default())
    }

    /// Build the router
    pub fn router(&self) -> Router {
        let state = AppState {
            handler: self.handler.clone(),
        };

        Router::new()
            .route("/", post(handle_rpc))
            .route("/mcp", post(handle_rpc))
            .route("/health", get(health_check))
            .route("/info", get(server_info))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    }

    /// Start the server
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let addr = self.config.socket_addr();
        let router = self.router();

        tracing::info!(
            name = %self.config.server_name,
            version = %self.config.server_version,
            address = %addr,
            "Starting MCP server"
        );

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await
    }

    /// Get the configured address
    pub fn address(&self) -> SocketAddr {
        self.config.socket_addr()
    }
}

/// Handle JSON-RPC request
async fn handle_rpc(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    tracing::debug!(method = %request.method, "Handling MCP request");
    let response = state.handler.handle(request).await;
    Json(response)
}

/// Health check endpoint
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Server info endpoint
async fn server_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Initialize to get server info
    let init_request = crate::types::JsonRpcRequest::new("initialize", None);
    let response = state.handler.handle(init_request).await;

    if let Some(result) = response.result {
        Json(result)
    } else {
        Json(serde_json::json!({
            "error": "Failed to get server info"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use v4_tools::{Tool, ToolCapability, ToolCategory, ToolMetadata, ToolRegistry};
    use v4_types::operators::OperatorResult;
    use async_trait::async_trait;

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn id(&self) -> &str {
            "mock_tool"
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                id: "mock_tool".to_string(),
                name: "Mock Tool".to_string(),
                description: "A mock tool".to_string(),
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
                data: Some(serde_json::json!("mock output")),
                error: None,
                execution_time_ms: 5,
                content_hash: Some("abc".to_string()),
            })
        }

        fn can_execute(&self, _operator: &v4_types::OperatorType) -> bool {
            true
        }
    }

    fn create_server() -> MCPServer {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockTool));
        let adapter = Arc::new(ToolAdapter::new(registry));
        MCPServer::with_adapter(adapter)
    }

    #[tokio::test]
    async fn test_health_check() {
        let server = create_server();
        let router = server.router();

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rpc_ping() {
        let server = create_server();
        let router = server.router();

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        });

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/mcp")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rpc_initialize() {
        let server = create_server();
        let router = server.router();

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test",
                    "version": "1.0.0"
                }
            }
        });

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_config_default() {
        let config = MCPServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_builder() {
        let config = MCPServerConfig::default()
            .with_host("0.0.0.0")
            .with_port(8080)
            .with_server_info("custom", "1.0.0");

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.server_name, "custom");
    }

    #[test]
    fn test_socket_addr() {
        let config = MCPServerConfig::default().with_port(9000);
        let addr = config.socket_addr();
        assert_eq!(addr.port(), 9000);
    }
}
