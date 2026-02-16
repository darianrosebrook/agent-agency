//! A2A HTTP Server
//!
//! Axum-based server implementing the A2A protocol over HTTP.
//! Supports JSON-RPC POST, Agent Card discovery, and SSE streaming.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Router,
};
use futures::stream;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handler::{A2AHandler, AgentHandler};
use crate::types::{JsonRpcRequest, JsonRpcResponse, SendMessageRequest};

/// A2A server configuration.
#[derive(Debug, Clone)]
pub struct A2AServerConfig {
    /// Server host address.
    pub host: String,
    /// Server port.
    pub port: u16,
}

impl Default for A2AServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
        }
    }
}

impl A2AServerConfig {
    /// Set the port.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the host.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Get the socket address.
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

/// Shared state for the A2A server.
#[derive(Clone)]
struct AppState {
    handler: Arc<A2AHandler>,
    agent: Arc<dyn AgentHandler>,
}

/// A2A Protocol server.
pub struct A2AServer {
    config: A2AServerConfig,
    handler: Arc<A2AHandler>,
    agent: Arc<dyn AgentHandler>,
}

impl A2AServer {
    /// Create a new A2A server from an agent implementation.
    pub fn new(agent: Arc<dyn AgentHandler>, config: A2AServerConfig) -> Self {
        let handler = Arc::new(A2AHandler::new(agent.clone()));
        Self {
            config,
            handler,
            agent,
        }
    }

    /// Create with default config.
    pub fn with_agent(agent: Arc<dyn AgentHandler>) -> Self {
        Self::new(agent, A2AServerConfig::default())
    }

    /// Build the Axum router.
    pub fn router(&self) -> Router {
        let state = AppState {
            handler: self.handler.clone(),
            agent: self.agent.clone(),
        };

        Router::new()
            // A2A JSON-RPC endpoint
            .route("/", post(handle_rpc))
            // Agent Card discovery (well-known)
            .route("/.well-known/agent-card.json", get(agent_card))
            // SSE streaming endpoint
            .route("/stream", post(handle_stream))
            // Health check
            .route("/health", get(health_check))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    }

    /// Start the server.
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let addr = self.config.socket_addr();
        let card = self.agent.agent_card();
        let router = self.router();

        tracing::info!(
            name = %card.name,
            version = %card.version,
            address = %addr,
            skills = card.skills.len(),
            "Starting A2A server"
        );

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await
    }

    /// Get the configured address.
    pub fn address(&self) -> SocketAddr {
        self.config.socket_addr()
    }
}

/// Handle JSON-RPC POST requests.
async fn handle_rpc(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    tracing::debug!(method = %request.method, "A2A request");
    let response = state.handler.handle(request).await;
    Json(response)
}

/// Serve the Agent Card at the well-known URL.
async fn agent_card(State(state): State<AppState>) -> impl IntoResponse {
    let card = state.agent.agent_card();
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        serde_json::to_string(card).unwrap_or_default(),
    )
}

/// Handle streaming requests via SSE.
async fn handle_stream(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let id = request.id.clone();

    // Parse the send-message request from params
    let send_request: Result<SendMessageRequest, _> = request
        .params
        .ok_or_else(|| crate::error::A2AError::MissingParameter("params".to_string()))
        .and_then(|p| {
            serde_json::from_value(p)
                .map_err(|e| crate::error::A2AError::InvalidParameter(e.to_string()))
        });

    match send_request {
        Ok(req) => {
            match state.agent.handle_message_stream(req).await {
                Ok(events) => {
                    let sse_events = events.into_iter().map(move |event| {
                        let response = JsonRpcResponse::success(
                            id.clone(),
                            serde_json::to_value(&event).unwrap_or_default(),
                        );
                        Ok::<_, std::convert::Infallible>(
                            Event::default()
                                .data(serde_json::to_string(&response).unwrap_or_default()),
                        )
                    });
                    Sse::new(stream::iter(sse_events)).into_response()
                }
                Err(err) => {
                    let response = JsonRpcResponse::error(
                        id,
                        crate::types::JsonRpcError {
                            code: err.error_code(),
                            message: err.to_string(),
                            data: None,
                        },
                    );
                    (
                        StatusCode::OK,
                        Json(response),
                    )
                        .into_response()
                }
            }
        }
        Err(err) => {
            let response = JsonRpcResponse::error(
                id,
                crate::types::JsonRpcError {
                    code: err.error_code(),
                    message: err.to_string(),
                    data: None,
                },
            );
            (StatusCode::OK, Json(response)).into_response()
        }
    }
}

/// Health check endpoint.
async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::EchoAgent;
    use crate::types::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn create_server() -> A2AServer {
        let card = AgentCard {
            name: "test-agent".to_string(),
            url: "http://localhost:3001".to_string(),
            version: "0.1.0".to_string(),
            skills: vec![AgentSkill {
                id: "echo".to_string(),
                name: "Echo".to_string(),
                description: "Echoes input".to_string(),
                tags: None,
                examples: None,
                input_modes: None,
                output_modes: None,
            }],
            description: None,
            documentation_url: None,
            provider: None,
            capabilities: Some(AgentCapabilities {
                a2a_version: Some("0.3".to_string()),
                streaming: Some(true),
                push_notifications: Some(false),
                state_transition_history: None,
            }),
            security_schemes: None,
            security: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            supports_authenticated_extended_card: None,
        };

        let agent = Arc::new(EchoAgent::new(card));
        A2AServer::with_agent(agent)
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
    async fn test_agent_card_discovery() {
        let server = create_server();
        let router = server.router();

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/.well-known/agent-card.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let card: AgentCard = serde_json::from_slice(&body).unwrap();
        assert_eq!(card.name, "test-agent");
        assert_eq!(card.skills.len(), 1);
    }

    #[tokio::test]
    async fn test_message_send() {
        let server = create_server();
        let router = server.router();

        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "message/send",
            "params": {
                "message": {
                    "messageId": "msg-1",
                    "role": "user",
                    "parts": [{"kind": "text", "text": "Hello agent"}],
                    "kind": "message"
                }
            }
        });

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&rpc_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let rpc_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
        assert!(rpc_response.error.is_none());

        let task: Task = serde_json::from_value(rpc_response.result.unwrap()).unwrap();
        assert_eq!(task.status.state, TaskState::Completed);
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let server = create_server();
        let router = server.router();

        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "nonexistent/method"
        });

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&rpc_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let rpc_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
        assert!(rpc_response.error.is_some());
    }

    #[test]
    fn test_config_defaults() {
        let config = A2AServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3001);
    }

    #[test]
    fn test_config_builder() {
        let config = A2AServerConfig::default()
            .with_host("0.0.0.0")
            .with_port(8080);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }
}
