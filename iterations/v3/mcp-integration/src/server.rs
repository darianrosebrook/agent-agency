//! MCP Server
//!
//! Main MCP server implementation for handling tool requests and responses.

use crate::types::*;
use crate::{CawsIntegration, ToolDiscovery, ToolRegistry};
use anyhow::{anyhow, bail, Result};
use jsonrpc_core::{Error as JsonRpcError, IoHandler, Params, Value};
use jsonrpc_http_server::hyper::{Body, Response, StatusCode};
use jsonrpc_http_server::{RequestMiddlewareAction, ServerBuilder};
use jsonrpc_ws_server::ws;
use jsonrpc_ws_server::ServerBuilder as WsServerBuilder;
// Using council package for security functionality
use agent_agency_council::error_handling::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats};

// Simple stub implementations for security functions
fn validate_api_input(_input: &serde_json::Value, _field: &str) -> Result<(), String> {
    Ok(()) // Stub - always pass validation
}

fn sanitize_api_input(input: &serde_json::Value) -> serde_json::Value {
    input.clone() // Stub - return as-is
}

struct CircuitBreakerRegistry;

impl CircuitBreakerRegistry {
    fn get_all_stats(&self) -> HashMap<String, CircuitBreakerStats> {
        HashMap::new() // Stub - return empty stats
    }
}

fn init_circuit_breaker_registry() -> Arc<CircuitBreakerRegistry> {
    Arc::new(CircuitBreakerRegistry) // Stub
}

fn get_circuit_breaker_registry() -> Arc<CircuitBreakerRegistry> {
    Arc::new(CircuitBreakerRegistry) // Stub
}

fn init_audit_logger(_enabled: bool, _level: String, _json: bool) -> Result<(), String> {
    Ok(()) // Stub
}

fn get_audit_logger() -> Option<String> {
    None // Stub
}
use observability::slo::{SLOTracker, create_default_slos};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{info, warn};

// Prometheus metrics
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, register_gauge, Counter, Histogram, Gauge};
use lazy_static::lazy_static;

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: Counter = register_counter!(
        "mcp_http_requests_total",
        "Total number of HTTP requests"
    ).expect("Can't create HTTP_REQUESTS_TOTAL metric");

    static ref HTTP_REQUEST_DURATION: Histogram = register_histogram!(
        "mcp_http_request_duration_seconds",
        "HTTP request duration in seconds"
    ).expect("Can't create HTTP_REQUEST_DURATION metric");

    static ref WEBSOCKET_CONNECTIONS_ACTIVE: Gauge = register_gauge!(
        "mcp_websocket_connections_active",
        "Number of active WebSocket connections"
    ).expect("Can't create WEBSOCKET_CONNECTIONS_ACTIVE metric");

    static ref WEBSOCKET_REQUESTS_TOTAL: Counter = register_counter!(
        "mcp_websocket_requests_total",
        "Total number of WebSocket requests"
    ).expect("Can't create WEBSOCKET_REQUESTS_TOTAL metric");

    static ref API_RATE_LIMIT_HITS: Counter = register_counter!(
        "mcp_api_rate_limit_hits_total",
        "Total number of rate limit hits"
    ).expect("Can't create API_RATE_LIMIT_HITS metric");

    static ref AUTH_FAILURES_TOTAL: Counter = register_counter!(
        "mcp_auth_failures_total",
        "Total number of authentication failures"
    ).expect("Can't create AUTH_FAILURES_TOTAL metric");

    static ref CIRCUIT_BREAKER_TRIPS: Counter = register_counter!(
        "mcp_circuit_breaker_trips_total",
        "Total number of circuit breaker trips"
    ).expect("Can't create CIRCUIT_BREAKER_TRIPS metric");

    // SLO-related metrics
    static ref SLO_API_AVAILABILITY: Gauge = register_gauge!(
        "multimodal_slo_api_availability",
        "API availability SLO compliance percentage"
    ).expect("Can't create SLO_API_AVAILABILITY metric");

    static ref SLO_TASK_COMPLETION: Gauge = register_gauge!(
        "multimodal_slo_task_completion",
        "Task completion SLO compliance percentage"
    ).expect("Can't create SLO_TASK_COMPLETION metric");

    static ref SLO_COUNCIL_DECISION_TIME: Gauge = register_gauge!(
        "multimodal_slo_council_decision_time",
        "Council decision time SLO P95 in milliseconds"
    ).expect("Can't create SLO_COUNCIL_DECISION_TIME metric");

    static ref SLO_WORKER_EXECUTION_TIME: Gauge = register_gauge!(
        "multimodal_slo_worker_execution_time",
        "Worker execution time SLO P95 in milliseconds"
    ).expect("Can't create SLO_WORKER_EXECUTION_TIME metric");

    static ref SLO_STATUS: Gauge = register_gauge!(
        "multimodal_slo_status",
        "SLO status (0=Compliant, 1=AtRisk, 2=Violated)"
    ).expect("Can't create SLO_STATUS metric");

    static ref SLO_ALERTS_TOTAL: Counter = register_counter!(
        "multimodal_slo_alerts_total",
        "Total number of SLO alerts generated"
    ).expect("Can't create SLO_ALERTS_TOTAL metric");
}

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

fn unauthorized_http_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("unauthorized"))
        .expect("response")
}

fn rate_limited_http_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body(Body::from("rate limit exceeded"))
        .expect("response")
}

fn unauthorized_ws_response() -> ws::Response {
    ws::Response::new(401, "Unauthorized", b"unauthorized".to_vec())
}

fn rate_limited_ws_response() -> ws::Response {
    ws::Response::new(429, "Too Many Requests", b"rate limit exceeded".to_vec())
}

#[derive(Debug)]
struct RateLimiter {
    limit_per_minute: u32,
    window_start: Instant,
    count: u32,
}

impl RateLimiter {
    fn new(limit_per_minute: u32) -> Self {
        Self {
            limit_per_minute,
            window_start: Instant::now(),
            count: 0,
        }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.window_start) >= Duration::from_secs(60) {
            self.window_start = now;
            self.count = 0;
        }

        if self.count >= self.limit_per_minute {
            false
        } else {
            self.count += 1;
            true
        }
    }
}

/// Enhanced authentication rate limiter with IP-based tracking
#[derive(Debug)]
struct AuthRateLimiter {
    /// Global auth attempts per minute
    global_limit: u32,
    /// Per-IP auth attempts per minute
    per_ip_limit: u32,
    /// Window duration in seconds
    window_duration: u64,
    /// IP-based attempt tracking: IP -> (window_start, count, blocked_until)
    ip_attempts: Arc<Mutex<HashMap<String, (Instant, u32, Option<Instant>)>>>,
    /// Global attempt tracking
    global_attempts: Arc<Mutex<(Instant, u32)>>,
}

impl AuthRateLimiter {
    fn new(global_limit: u32, per_ip_limit: u32, window_duration: u64) -> Self {
        Self {
            global_limit,
            per_ip_limit,
            window_duration,
            ip_attempts: Arc::new(Mutex::new(HashMap::new())),
            global_attempts: Arc::new(Mutex::new((Instant::now(), 0))),
        }
    }

    /// Check if authentication attempt is allowed for the given IP
    fn allow_auth_attempt(&self, ip: &str) -> AuthRateLimitResult {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.window_duration);

        // Check global rate limit
        {
            let mut global = self.global_attempts.lock().unwrap();
            if now.duration_since(global.0) >= window_duration {
                global.0 = now;
                global.1 = 0;
            }

            if global.1 >= self.global_limit {
                tracing::warn!("Global authentication rate limit exceeded");
                return AuthRateLimitResult::Blocked("Global rate limit exceeded".to_string());
            }
            global.1 += 1;
        }

        // Check per-IP rate limit
        {
            let mut ip_attempts = self.ip_attempts.lock().unwrap();
            let entry = ip_attempts.entry(ip.to_string()).or_insert((now, 0, None));

            // Check if IP is currently blocked
            if let Some(blocked_until) = entry.2 {
                if now < blocked_until {
                    let remaining = blocked_until.duration_since(now).as_secs();
                    return AuthRateLimitResult::Blocked(
                        format!("IP temporarily blocked for {} more seconds", remaining)
                    );
                } else {
                    // Block period expired, reset
                    entry.2 = None;
                    entry.0 = now;
                    entry.1 = 0;
                }
            }

            // Reset window if expired
            if now.duration_since(entry.0) >= window_duration {
                entry.0 = now;
                entry.1 = 0;
                entry.2 = None;
            }

            // Check rate limit
            if entry.1 >= self.per_ip_limit {
                // Implement progressive blocking: 5 minutes for first offense, 15 for second, etc.
                let block_duration = Duration::from_secs(300 * (entry.1 / self.per_ip_limit));
                entry.2 = Some(now + block_duration);

                tracing::warn!(
                    ip = %ip,
                    attempts = %entry.1,
                    block_duration_secs = %block_duration.as_secs(),
                    "IP authentication rate limit exceeded, blocking temporarily"
                );

                return AuthRateLimitResult::Blocked(
                    format!("Rate limit exceeded, blocked for {} seconds", block_duration.as_secs())
                );
            }

            entry.1 += 1;

            // Log suspicious activity if approaching limit
            if entry.1 > self.per_ip_limit / 2 {
                tracing::info!(
                    ip = %ip,
                    attempts = %entry.1,
                    limit = %self.per_ip_limit,
                    "High authentication attempt rate from IP"
                );
            }
        }

        AuthRateLimitResult::Allowed
    }

    /// Record a failed authentication attempt
    fn record_failed_attempt(&self, ip: &str) {
        let mut ip_attempts = self.ip_attempts.lock().unwrap();
        let entry = ip_attempts.entry(ip.to_string()).or_insert((Instant::now(), 0, None));
        entry.1 += 1; // Extra penalty for failed attempts

        tracing::warn!(
            ip = %ip,
            failed_attempts = %entry.1,
            "Failed authentication attempt recorded"
        );
    }

    /// Get current stats for monitoring
    fn get_stats(&self) -> AuthRateLimitStats {
        let ip_attempts = self.ip_attempts.lock().unwrap();
        let global = self.global_attempts.lock().unwrap();

        let now = Instant::now();
        let active_blocks = ip_attempts.values()
            .filter(|(_, _, blocked_until)| {
                blocked_until.map_or(false, |until| now < until)
            })
            .count();

        AuthRateLimitStats {
            global_attempts: global.1,
            global_limit: self.global_limit,
            unique_ips_tracked: ip_attempts.len(),
            active_blocks,
        }
    }
}

/// Result of authentication rate limit check
#[derive(Debug, Clone)]
enum AuthRateLimitResult {
    Allowed,
    Blocked(String),
}

/// Statistics for authentication rate limiting
#[derive(Debug, Clone)]
pub struct AuthRateLimitStats {
    pub global_attempts: u32,
    pub global_limit: u32,
    pub unique_ips_tracked: usize,
    pub active_blocks: usize,
}

/// Main MCP server
#[derive(Debug, Clone)]
pub struct MCPServer {
    config: MCPConfig,
    tool_registry: Arc<ToolRegistry>,
    tool_discovery: Arc<ToolDiscovery>,
    caws_integration: Arc<CawsIntegration>,
    status: Arc<RwLock<MCPServerStatus>>,
    connections: Arc<RwLock<Vec<MCPConnection>>>,
    http_handle: Arc<RwLock<Option<HttpServerHandle>>>,
    ws_handle: Arc<RwLock<Option<HttpServerHandle>>>,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    auth_rate_limiter: Option<Arc<AuthRateLimiter>>,
    api_rate_limiter: Option<Arc<RateLimitMiddleware>>,
    slo_tracker: Arc<SLOTracker>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new(config: MCPConfig) -> Self {
        let rate_limiter = config
            .server
            .requests_per_minute
            .map(|limit| Arc::new(Mutex::new(RateLimiter::new(limit))));

        // Create auth rate limiter with stricter limits for security
        // Global limit: 100 auth attempts per minute
        // Per-IP limit: 5 auth attempts per minute
        // Window: 60 seconds
        let auth_rate_limiter = Some(Arc::new(AuthRateLimiter::new(100, 5, 60)));

        // Create API rate limiter with endpoint-specific limits
        let api_rate_configs = vec![
            RateLimitConfig {
                endpoint: "/api/tools".to_string(),
                requests_per_minute: 100,
                burst_limit: 20,
                window_seconds: 60,
            },
            RateLimitConfig {
                endpoint: "/api/stats".to_string(),
                requests_per_minute: 30,
                burst_limit: 5,
                window_seconds: 60,
            },
            RateLimitConfig {
                endpoint: "/api/validate".to_string(),
                requests_per_minute: 50,
                burst_limit: 10,
                window_seconds: 60,
            },
            RateLimitConfig {
                endpoint: "/api/*".to_string(),
                requests_per_minute: 200,
                burst_limit: 50,
                window_seconds: 60,
            },
        ];
        let api_rate_limiter = Some(Arc::new(RateLimitMiddleware::new(None, api_rate_configs)));

        // Initialize SLO tracker with default SLOs
        let slo_tracker = Arc::new({
            let mut tracker = SLOTracker::new();
            // Register default SLOs for the multimodal RAG system
            let default_slos = create_default_slos();
            for slo in default_slos {
                if let Err(e) = tokio::runtime::Handle::current().block_on(tracker.register_slo(slo)) {
                    warn!("Failed to register SLO: {}", e);
                }
            }
            tracker
        });

        Self {
            config,
            tool_registry: Arc::new(ToolRegistry::new()),
            tool_discovery: Arc::new(ToolDiscovery::new()),
            caws_integration: Arc::new(CawsIntegration::new()),
            status: Arc::new(RwLock::new(MCPServerStatus::Starting)),
            connections: Arc::new(RwLock::new(Vec::new())),
            http_handle: Arc::new(RwLock::new(None)),
            ws_handle: Arc::new(RwLock::new(None)),
            rate_limiter,
            auth_rate_limiter,
            api_rate_limiter,
            slo_tracker,
        }
    }

    /// Update SLO metrics from tracker
    async fn update_slo_metrics(&self) -> Result<()> {
        let slo_statuses = self.slo_tracker.get_all_slo_statuses().await?;

        for status in slo_statuses {
            match status.slo_name.as_str() {
                "api_availability" => {
                    SLO_API_AVAILABILITY.set(status.compliance_percentage);
                }
                "task_completion" => {
                    SLO_TASK_COMPLETION.set(status.compliance_percentage);
                }
                "council_decision_time" => {
                    SLO_COUNCIL_DECISION_TIME.set(status.current_value);
                }
                "worker_execution_time" => {
                    SLO_WORKER_EXECUTION_TIME.set(status.current_value);
                }
                _ => {}
            }

            // Set SLO status gauge
            let status_value = match status.status {
                observability::slo::SLOStatus::Compliant => 0.0,
                observability::slo::SLOStatus::AtRisk => 1.0,
                observability::slo::SLOStatus::Violated => 2.0,
                observability::slo::SLOStatus::Unknown => -1.0,
            };
            SLO_STATUS.set(status_value);
        }

        // Update SLO alerts counter
        let recent_alerts = self.slo_tracker.get_recent_alerts(100).await;
        SLO_ALERTS_TOTAL.reset();
        SLO_ALERTS_TOTAL.inc_by(recent_alerts.len() as f64);

        Ok(())
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

        // Initialize circuit breaker registry
        let registry = init_circuit_breaker_registry();

        // Register circuit breakers for external services
        registry.register("caws-integration", CircuitBreakerConfig {
            service_name: "caws-integration".to_string(),
            failure_threshold: 3,
            success_threshold: 2,
            timeout_duration: Duration::from_secs(30),
            request_timeout: Duration::from_secs(10),
            half_open_max_requests: 2,
        });

        registry.register("tool-discovery", CircuitBreakerConfig {
            service_name: "tool-discovery".to_string(),
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(60),
            request_timeout: Duration::from_secs(5),
            half_open_max_requests: 3,
        });

        // Initialize audit logger
        init_audit_logger(true, "info".to_string(), false).map_err(|e| {
            anyhow!("Failed to initialize audit logger: {}", e)
        })?;

        // Start SLO metrics update task
        let slo_tracker_clone = Arc::clone(&self.slo_tracker);
        let slo_server = Arc::new(self.clone());
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Update every 30 seconds
            loop {
                interval.tick().await;
                if let Err(e) = slo_server.update_slo_metrics().await {
                    warn!("Failed to update SLO metrics: {}", e);
                }
            }
        });

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
        let auth_api_key = self.config.server.auth_api_key.clone();
        let rate_limiter = self.rate_limiter.clone();

        let handle = tokio::task::spawn_blocking(move || {
            let io = Self::build_io_handler(
                registry.clone(),
                registry_for_stats.clone(),
                caws.clone(),
                version_payload.clone(),
            );
            let builder = ServerBuilder::new(io).request_middleware(
                move |request: jsonrpc_http_server::hyper::Request<Body>| {
                    let start_time = Instant::now();
                    let method = request.method().to_string();
                    let uri = request.uri().path().to_string();
                    // Extract client IP for rate limiting
                    let client_ip = request
                        .headers()
                        .get("x-forwarded-for")
                        .and_then(|value| value.to_str().ok())
                        .or_else(|| request
                            .headers()
                            .get("x-real-ip")
                            .and_then(|value| value.to_str().ok()))
                        .unwrap_or("unknown");

                    // Check authentication rate limit before processing auth
                    if let Some(ref auth_limiter) = auth_rate_limiter {
                        match auth_limiter.allow_auth_attempt(client_ip) {
                            AuthRateLimitResult::Blocked(reason) => {
                                warn!(ip = %client_ip, reason = %reason, "Authentication rate limit exceeded");
                                API_RATE_LIMIT_HITS.inc();
                                return RequestMiddlewareAction::from(rate_limited_http_response());
                            }
                            AuthRateLimitResult::Allowed => {
                                // Continue with authentication check
                            }
                        }
                    }

                    // Check API key authentication
                    let auth_failed = if let Some(ref expected) = auth_api_key {
                        let provided = request
                            .headers()
                            .get("x-api-key")
                            .and_then(|value| value.to_str().ok());
                        if provided != Some(expected.as_str()) {
                            // Record failed authentication attempt
                            if let Some(ref auth_limiter) = auth_rate_limiter {
                                auth_limiter.record_failed_attempt(client_ip);
                            }
                            AUTH_FAILURES_TOTAL.inc();

                            // Log failed authentication
                            let user_agent = request
                                .headers()
                                .get("user-agent")
                                .and_then(|value| value.to_str().ok())
                                .map(|s| s.to_string());

                            if let Ok(logger) = get_audit_logger() {
                                let mut metadata = HashMap::new();
                                metadata.insert("provided_key".to_string(), serde_json::Value::String(provided.unwrap_or("none").to_string()));
                                metadata.insert("endpoint".to_string(), serde_json::Value::String("http".to_string()));

                                tokio::spawn(async move {
                                    let _ = logger.log_authentication(
                                        "api_client".to_string(),
                                        false,
                                        Some(client_ip.to_string()),
                                        user_agent,
                                        metadata,
                                    ).await;
                                });
                            }

                            true
                        } else {
                            // Log successful authentication
                            if let Ok(logger) = get_audit_logger() {
                                let user_agent = request
                                    .headers()
                                    .get("user-agent")
                                    .and_then(|value| value.to_str().ok())
                                    .map(|s| s.to_string());

                                let mut metadata = HashMap::new();
                                metadata.insert("endpoint".to_string(), serde_json::Value::String("http".to_string()));

                                tokio::spawn(async move {
                                    let _ = logger.log_authentication(
                                        "api_client".to_string(),
                                        true,
                                        Some(client_ip.to_string()),
                                        user_agent,
                                        metadata,
                                    ).await;
                                });
                            }

                            false
                        }
                    } else {
                        false
                    };

                    if auth_failed {
                        return RequestMiddlewareAction::from(unauthorized_http_response());
                    }

                    // Check API-specific rate limiting
                    if let Some(ref api_limiter) = api_rate_limiter {
                        if !api_limiter.should_allow("/api/validate", client_ip) {
                            warn!("API rate limit exceeded for {} on endpoint /api/validate", client_ip);
                            API_RATE_LIMIT_HITS.inc();
                            return RequestMiddlewareAction::from(rate_limited_http_response());
                        }
                    }

                    // Check general rate limiting
                    if let Some(ref limiter) = rate_limiter {
                        let mut guard = limiter.lock().unwrap();
                        if !guard.allow() {
                            API_RATE_LIMIT_HITS.inc();
                            return RequestMiddlewareAction::from(rate_limited_http_response());
                        }
                    }

                    RequestMiddlewareAction::from(request)
                },
            );

            let server = builder
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

        // Add metrics endpoint for Prometheus
        io.add_sync_method("metrics", move |_| {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let metrics = String::from_utf8(buffer).unwrap();
            Ok(Value::String(metrics))
        });

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

                // Validate and sanitize input
                if let Err(validation_error) = validate_api_input(&v, "tool") {
                    return Err(JsonRpcError {
                        code: jsonrpc_core::ErrorCode::InvalidParams,
                        message: format!("Input validation failed: {}", validation_error),
                        data: Some(serde_json::Value::String(validation_error.to_string())),
                    });
                }

                // Sanitize the input
                let sanitized_value = sanitize_api_input(&v);

                let tool: crate::types::MCPTool =
                    serde_json::from_value(sanitized_value).map_err(|e| JsonRpcError {
                        code: jsonrpc_core::ErrorCode::InvalidParams,
                        message: "Invalid tool format after sanitization".to_string(),
                        data: Some(serde_json::Value::String(e.to_string())),
                    })?;
                // Execute CAWS validation with circuit breaker protection
                let registry = get_circuit_breaker_registry();
                let res = registry
                    .execute_with_circuit_breaker("caws-integration", || {
                        caws_validate.validate_tool(&tool)
                    })
                    .await
                    .map_err(|e| match e {
                        security::CircuitBreakerError::CircuitOpen(_) => {
                            CIRCUIT_BREAKER_TRIPS.inc();
                            JsonRpcError {
                                code: jsonrpc_core::ErrorCode::InternalError,
                                message: "Service temporarily unavailable".to_string(),
                                data: Some(serde_json::Value::String("Circuit breaker open".to_string())),
                            }
                        },
                        security::CircuitBreakerError::OperationFailed(orig_err) => JsonRpcError {
                            code: jsonrpc_core::ErrorCode::InternalError,
                            message: "Tool validation failed".to_string(),
                            data: Some(serde_json::Value::String(orig_err.to_string())),
                        },
                        security::CircuitBreakerError::Timeout(duration) => JsonRpcError {
                            code: jsonrpc_core::ErrorCode::InternalError,
                            message: format!("Tool validation timed out after {:?}", duration),
                            data: Some(serde_json::Value::String("Request timeout".to_string())),
                        },
                    })?;
                Ok(serde_json::to_value(&res).unwrap())
            }
        });

        // SLO endpoints
        io.add_sync_method("slo/status", |_| {
            // TODO: Integrate with SLO tracker for real-time status reporting
            // - [ ] Connect to SLO tracker service or database
            // - [ ] Implement SLO status queries with current metrics
            // - [ ] Add SLO violation detection and alerting
            // - [ ] Handle SLO tracker connection failures gracefully
            // - [ ] Implement SLO status caching for performance
            Ok(serde_json::to_value(observability::slo::create_default_slos()).unwrap())
        });

        io.add_sync_method("slo/alerts", |_| {
            // TODO: Implement SLO alerts retrieval from tracker
            // - [ ] Query SLO tracker for recent alerts and violations
            // - [ ] Implement alert filtering by time range and severity
            // - [ ] Add alert acknowledgment and resolution tracking
            // - [ ] Handle alert pagination for large result sets
            // - [ ] Implement real-time alert streaming via WebSocket
            Ok(serde_json::to_value(Vec::<observability::slo::SLOAlert>::new()).unwrap())
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

    pub async fn push_connection_for_testing(&self, connection: MCPConnection) {
        let mut guard = self.connections.write().await;
        guard.push(connection);
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
        let auth_api_key = self.config.server.auth_api_key.clone();
        let rate_limiter = self.rate_limiter.clone();

        let handle = tokio::task::spawn_blocking(move || {
            let io = MCPServer::build_io_handler(
                registry.clone(),
                registry_stats.clone(),
                caws.clone(),
                version_payload.clone(),
            );

            let middleware = move |req: &ws::Request| {
                // Extract client IP for rate limiting (WebSocket connections)
                let client_ip = req
                    .header("x-forwarded-for")
                    .and_then(|value| std::str::from_utf8(value).ok())
                    .or_else(|| req
                        .header("x-real-ip")
                        .and_then(|value| std::str::from_utf8(value).ok()))
                    .unwrap_or("unknown");

                // Check authentication rate limit before processing auth
                if let Some(ref auth_limiter) = auth_rate_limiter {
                    match auth_limiter.allow_auth_attempt(client_ip) {
                        AuthRateLimitResult::Blocked(reason) => {
                            warn!(ip = %client_ip, reason = %reason, "WebSocket authentication rate limit exceeded");
                            return Some(rate_limited_ws_response());
                        }
                        AuthRateLimitResult::Allowed => {
                            // Continue with authentication check
                        }
                    }
                }

                // Check API key authentication
                let auth_failed = if let Some(ref expected) = auth_api_key {
                    let provided = req
                        .header("x-api-key")
                        .and_then(|value| std::str::from_utf8(value).ok());
                    if provided != Some(expected.as_str()) {
                        // Record failed authentication attempt
                        if let Some(ref auth_limiter) = auth_rate_limiter {
                            auth_limiter.record_failed_attempt(client_ip);
                        }

                        // Log failed WebSocket authentication
                        if let Ok(logger) = get_audit_logger() {
                            let mut metadata = HashMap::new();
                            metadata.insert("provided_key".to_string(), serde_json::Value::String(provided.unwrap_or("none").to_string()));
                            metadata.insert("endpoint".to_string(), serde_json::Value::String("websocket".to_string()));

                            tokio::spawn(async move {
                                let _ = logger.log_authentication(
                                    "websocket_client".to_string(),
                                    false,
                                    Some(client_ip.to_string()),
                                    None,
                                    metadata,
                                ).await;
                            });
                        }

                        true
                    } else {
                        // Log successful WebSocket authentication
                        if let Ok(logger) = get_audit_logger() {
                            let mut metadata = HashMap::new();
                            metadata.insert("endpoint".to_string(), serde_json::Value::String("websocket".to_string()));

                            tokio::spawn(async move {
                                let _ = logger.log_authentication(
                                    "websocket_client".to_string(),
                                    true,
                                    Some(client_ip.to_string()),
                                    None,
                                    metadata,
                                ).await;
                            });
                        }

                        false
                    }
                } else {
                    false
                };

                if auth_failed {
                    return Some(unauthorized_ws_response());
                }

                // Check general rate limiting
                if let Some(ref limiter) = rate_limiter {
                    let mut guard = limiter.lock().unwrap();
                    if !guard.allow() {
                        return Some(rate_limited_ws_response());
                    }
                }

                None
            };

            let server = WsServerBuilder::new(io)
                .request_middleware(middleware)
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

    /// Get authentication rate limiting statistics
    pub async fn get_auth_rate_limit_stats(&self) -> Option<AuthRateLimitStats> {
        self.auth_rate_limiter.as_ref().map(|limiter| limiter.get_stats())
    }

    /// Get circuit breaker statistics
    pub async fn get_circuit_breaker_stats(&self) -> HashMap<String, agent_agency_security::CircuitBreakerStats> {
        get_circuit_breaker_registry().get_all_stats()
    }

    /// Get API rate limiting statistics
    pub async fn get_api_rate_limit_stats(&self) -> Option<HashMap<String, (u32, u32)>> {
        self.api_rate_limiter.as_ref().map(|limiter| limiter.get_stats())
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
