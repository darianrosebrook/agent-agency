//! HTTP Server
//!
//! Axum server configuration and startup.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::handlers::{
    api_info, get_chain_of_thought, get_council_decisions, get_task, get_worker_actions, health,
    metrics, not_found, probe_llm, submit_task,
};
use crate::service::{ApiService, ServiceConfig};
use v4_arbiter::Arbiter;
use v4_inference::{InferenceConfig, InferenceService};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Request timeout
    pub request_timeout: Duration,
    /// Enable CORS
    pub enable_cors: bool,
    /// Service configuration
    pub service_config: ServiceConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            request_timeout: Duration::from_secs(30),
            enable_cors: true,
            service_config: ServiceConfig::default(),
        }
    }
}

impl ServerConfig {
    /// Create a config for local development
    pub fn local_dev() -> Self {
        Self::default()
    }

    /// Create a config for production
    pub fn production() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            request_timeout: Duration::from_secs(60),
            enable_cors: false, // More restrictive in production
            ..Default::default()
        }
    }

    /// Get socket address
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

/// Build the application router
pub fn build_router(service: Arc<ApiService>, enable_cors: bool) -> Router {
    let mut app = Router::new()
        // API v1 routes
        .route("/api/v1", get(api_info))
        .route("/api/v1/tasks", post(submit_task))
        .route("/api/v1/tasks/:id", get(get_task))
        .route("/api/v1/tasks/:id/chain-of-thought", get(get_chain_of_thought))
        .route("/api/v1/tasks/:id/council-decisions", get(get_council_decisions))
        .route("/api/v1/tasks/:id/worker-actions", get(get_worker_actions))
        .route("/api/v1/probe", post(probe_llm))
        // System routes
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        // Fallback
        .fallback(not_found)
        .with_state(service);

    // Add middleware
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http());

    app = app.layer(middleware);

    // Add CORS if enabled
    if enable_cors {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        app = app.layer(cors);
    }

    app
}

/// Server instance
pub struct Server {
    config: ServerConfig,
    service: Arc<ApiService>,
}

impl Server {
    /// Create a new server with default arbiter and mock inference
    pub fn new(config: ServerConfig) -> Self {
        let arbiter = Arc::new(Arbiter::new());
        let inference = Arc::new(InferenceService::new(InferenceConfig::mock()));
        let service = Arc::new(ApiService::new(
            arbiter,
            inference,
            config.service_config.clone(),
        ));

        Self { config, service }
    }

    /// Create with custom arbiter
    pub fn with_arbiter(config: ServerConfig, arbiter: Arc<Arbiter>) -> Self {
        let inference = Arc::new(InferenceService::new(InferenceConfig::mock()));
        let service = Arc::new(ApiService::new(
            arbiter,
            inference,
            config.service_config.clone(),
        ));
        Self { config, service }
    }

    /// Create with custom inference config
    pub fn with_inference(config: ServerConfig, inference_config: InferenceConfig) -> Self {
        let arbiter = Arc::new(Arbiter::new());
        let inference = Arc::new(InferenceService::new(inference_config));
        let service = Arc::new(ApiService::new(
            arbiter,
            inference,
            config.service_config.clone(),
        ));
        Self { config, service }
    }

    /// Create with custom service
    pub fn with_service(config: ServerConfig, service: Arc<ApiService>) -> Self {
        Self { config, service }
    }

    /// Get the service reference
    pub fn service(&self) -> &Arc<ApiService> {
        &self.service
    }

    /// Get the router (useful for testing)
    pub fn router(&self) -> Router {
        build_router(self.service.clone(), self.config.enable_cors)
    }

    /// Run the server
    pub async fn run(self) -> Result<(), std::io::Error> {
        let addr = self.config.socket_addr();
        let router = self.router();

        tracing::info!(%addr, "Starting V4 API server");

        let listener = tokio::net::TcpListener::bind(addr).await?;

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        tracing::info!("Server shut down gracefully");
        Ok(())
    }

    /// Run the server and return when ready
    /// Useful for testing
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.run().await
    }
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down");
        }
        () = terminate => {
            tracing::info!("Received terminate signal, shutting down");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert!(config.enable_cors);
    }

    #[test]
    fn test_server_config_production() {
        let config = ServerConfig::production();
        assert_eq!(config.host, "0.0.0.0");
        assert!(!config.enable_cors);
    }

    #[test]
    fn test_socket_addr() {
        let config = ServerConfig::default();
        let addr = config.socket_addr();
        assert_eq!(addr.port(), 8080);
    }

    #[tokio::test]
    async fn test_router_health() {
        let service = Arc::new(ApiService::with_defaults());
        let router = build_router(service, true);

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
    async fn test_router_api_info() {
        let service = Arc::new(ApiService::with_defaults());
        let router = build_router(service, true);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_router_404() {
        let service = Arc::new(ApiService::with_defaults());
        let router = build_router(service, true);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_server_creation() {
        let config = ServerConfig::default();
        let server = Server::new(config);
        assert!(Arc::strong_count(server.service()) >= 1);
    }
}
