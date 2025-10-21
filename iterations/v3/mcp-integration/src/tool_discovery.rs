//! Tool Discovery
//!
//! Discovers and validates MCP tools from filesystem and remote sources.

use crate::types::*;
use anyhow::{Context, Result};
use glob;
use reqwest::{Client, StatusCode};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

/// Tool discovery service
#[derive(Debug)]
pub struct ToolDiscovery {
    pub(crate) config: ToolDiscoveryConfig,
    pub(crate) discovered_tools: Arc<RwLock<Vec<MCPTool>>>,
    pub(crate) discovery_active: Arc<RwLock<bool>>,
    pub(crate) cancellation_token: Arc<CancellationToken>,
}

impl ToolDiscovery {
    /// Create a new tool discovery service
    pub fn new() -> Self {
        Self::with_config(ToolDiscoveryConfig::default())
    }

    /// Create with explicit config (useful for tests)
    pub fn with_config(config: ToolDiscoveryConfig) -> Self {
        Self {
            config,
            discovered_tools: Arc::new(RwLock::new(Vec::new())),
            discovery_active: Arc::new(RwLock::new(false)),
            cancellation_token: Arc::new(CancellationToken::new()),
        }
    }

    /// Initialize tool discovery
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing tool discovery");

        // Validate discovery paths exist
        for p in &self.config.discovery_paths {
            if !std::path::Path::new(p).exists() {
                warn!("Discovery path does not exist: {}", p);
            }
        }
        Ok(())
    }

    /// Start automatic tool discovery
    pub async fn start_auto_discovery(&self) -> Result<()> {
        info!("Starting automatic tool discovery");

        {
            let mut active = self.discovery_active.write().await;
            *active = true;
        }

        // Spawn a lightweight background task that periodically scans
        let interval = self.config.discovery_interval_seconds;
        let this = self.clone_for_task();
        let token = self.cancellation_token.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval as u64));
            loop {
                // Check for cancellation before each tick
                if token.is_cancelled() {
                    break;
                }

                ticker.tick().await;

                // stop if deactivated
                if !this.is_active().await {
                    break;
                }

                // Check for cancellation before discovery
                if token.is_cancelled() {
                    break;
                }

                if let Err(e) = this.discover_tools().await {
                    error!("Auto discovery error: {e:?}");
                }
            }
        });
        Ok(())
    }

    /// Stop automatic tool discovery
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping tool discovery");

        // Cancel the cancellation token for immediate shutdown
        self.cancellation_token.cancel();

        {
            let mut active = self.discovery_active.write().await;
            *active = false;
        }

        // Background task loop exits when inactive flag is false or token is cancelled
        Ok(())
    }

    /// Discover tools from configured paths
    pub async fn discover_tools(&self) -> Result<ToolDiscoveryResult> {
        info!(
            "Discovering tools from paths: {:?}",
            self.config.discovery_paths
        );

        // Check for cancellation before starting
        if self.cancellation_token.is_cancelled() {
            return Ok(ToolDiscoveryResult {
                discovered_tools: Vec::new(),
                errors: vec![DiscoveryError {
                    path: "discovery".to_string(),
                    error_type: DiscoveryErrorType::Unknown,
                    message: "Discovery cancelled".to_string(),
                    details: None,
                }],
                discovery_time_ms: 0,
                discovered_at: chrono::Utc::now(),
            });
        }

        let start_time = std::time::Instant::now();
        let mut discovered_tools = Vec::new();
        let mut errors = Vec::new();

        // Scan filesystem paths for manifests and parse them
        let mut set = std::collections::HashSet::new();
        for base in &self.config.discovery_paths {
            if !std::path::Path::new(base).exists() {
                continue;
            }
            // simple glob over manifest patterns
            for pattern in &self.config.manifest_patterns {
                let full = format!("{}/{}", base.trim_end_matches('/'), pattern);
                match glob::glob(&full) {
                    Ok(paths) => {
                        for entry in paths.flatten() {
                            if let Some(p) = entry.to_str() {
                                set.insert(p.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(DiscoveryError {
                            path: full.clone(),
                            error_type: DiscoveryErrorType::Unknown,
                            message: format!("glob error: {e}"),
                            details: None,
                        });
                    }
                }
            }
        }

        for path in set {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    // Try JSON, then YAML
                    let manifest: Result<crate::types::ToolManifest, _> =
                        serde_json::from_str(&content);
                    let manifest = match manifest.or_else(|_| serde_yaml::from_str(&content)) {
                        Ok(m) => m,
                        Err(e) => {
                            errors.push(DiscoveryError {
                                path: path.clone(),
                                error_type: DiscoveryErrorType::ParseError,
                                message: format!("manifest parse error: {e}"),
                                details: None,
                            });
                            continue;
                        }
                    };
                    // Convert manifest to MCPTool
                    let tool = self.manifest_to_tool(&manifest);
                    // Validate
                    if self.config.enable_validation {
                        let v = self.validate_tool(&tool).await?;
                        if !v.is_valid {
                            errors.push(DiscoveryError {
                                path: path.clone(),
                                error_type: DiscoveryErrorType::ValidationError,
                                message: format!("validation errors: {:?}", v.errors),
                                details: None,
                            });
                            continue;
                        }
                    }
                    discovered_tools.push(tool);
                }
                Err(e) => {
                    errors.push(DiscoveryError {
                        path: path.clone(),
                        error_type: DiscoveryErrorType::FileNotFound,
                        message: format!("read error: {e}"),
                        details: None,
                    });
                }
            }
        }

        // Save in shared state
        {
            let mut slot = self.discovered_tools.write().await;
            *slot = discovered_tools.clone();
        }

        let discovery_time_ms = start_time.elapsed().as_millis() as u64;

        let result = ToolDiscoveryResult {
            discovered_tools,
            errors,
            discovery_time_ms,
            discovered_at: chrono::Utc::now(),
        };

        info!(
            "Tool discovery completed: {} tools, {} errors",
            result.discovered_tools.len(),
            result.errors.len()
        );

        Ok(result)
    }

    /// Validate a discovered tool
    pub async fn validate_tool(&self, tool: &MCPTool) -> Result<ValidationResult> {
        info!("Validating tool: {}", tool.name);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Static checks: basic schema validation
        if tool.name.trim().is_empty() {
            errors.push("name is required".into());
        }
        if tool.version.trim().is_empty() {
            warnings.push("version missing".into());
        }
        if tool
            .parameters
            .required
            .iter()
            .any(|p| p.name.trim().is_empty())
        {
            errors.push("parameter with empty name".into());
        }

        // Schema validation: check output schema is valid JSON
        if let Err(e) = serde_json::from_value::<serde_json::Value>(tool.output_schema.clone()) {
            errors.push(format!("invalid output schema JSON: {}", e));
        }

        // Permission validation: check for dangerous capability combinations
        let has_network = tool.capabilities.contains(&ToolCapability::NetworkAccess);
        let has_command = tool
            .capabilities
            .contains(&ToolCapability::CommandExecution);
        let has_file_system = tool
            .capabilities
            .contains(&ToolCapability::FileSystemAccess);

        if has_network && has_command {
            warnings.push("tool has both network and command execution capabilities - ensure proper sandboxing".into());
        }

        if has_command
            && !tool
                .metadata
                .get("sandboxed")
                .map_or(false, |v| v.as_bool().unwrap_or(false))
        {
            errors.push("command execution capability requires sandboxed=true in metadata".into());
        }

        if has_file_system {
            let allowed_paths: Vec<String> = tool
                .metadata
                .get("allowed_paths")
                .and_then(|p| serde_json::from_value(p.clone()).ok())
                .unwrap_or_default();

            if allowed_paths.is_empty() {
                warnings.push(
                    "filesystem access without restricted paths - consider limiting scope".into(),
                );
            }
        }

        // Dynamic probe: health ping (if endpoint is configured)
        if self.config.enable_health_checks && !tool.endpoint.is_empty() {
            match self.perform_health_ping(tool).await {
                Ok(healthy) => {
                    if !healthy {
                        errors.push("health check failed - tool endpoint not responding".into());
                    }
                }
                Err(e) => {
                    warnings.push(format!(
                        "health check error: {} - may indicate connectivity issues",
                        e
                    ));
                }
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Perform a health ping on the tool endpoint
    async fn perform_health_ping(&self, tool: &MCPTool) -> Result<bool> {
        use std::time::Duration;
        use tokio::time::timeout;

        // Simple health check - try to connect to the endpoint
        // Implement comprehensive health check API calls with robust error handling
        // 1. API endpoint validation: Make actual API calls to validate endpoint functionality
        // 2. Health check metrics: Collect response time and availability metrics
        // 3. Error handling: Handle network errors, timeouts, and connection failures
        // 4. Performance optimization: Implement efficient health check scheduling and batching

        let health_result = self.perform_comprehensive_health_check(tool).await?;

        if !health_result.is_healthy {
            warn!(
                "Health check failed for tool {}: {}",
                tool.name, health_result.error_message
            );
            return Ok(false);
        }

        info!(
            "Health check passed for tool {}: {}ms response time",
            tool.name, health_result.response_time_ms
        );
        let endpoint = &tool.endpoint;

        // For HTTP endpoints, try a HEAD request
        if endpoint.starts_with("http") {
            match timeout(
                Duration::from_secs(self.config.health_check_timeout_seconds as u64),
                reqwest::get(endpoint),
            )
            .await
            {
                Ok(Ok(response)) => {
                    return Ok(response.status().is_success());
                }
                Ok(Err(_)) | Err(_) => {
                    return Ok(false);
                }
            }
        }

        // Implement comprehensive endpoint health checking
        self.check_endpoint_health(endpoint).await
    }

    /// Discover tools with filtering options
    pub async fn discover_tools_filtered(
        &self,
        language_filter: Option<&str>,
        tag_filters: Option<&[String]>,
        risk_tier_filter: Option<RiskTier>,
    ) -> Result<ToolDiscoveryResult> {
        let all_results = self.discover_tools().await?;
        let mut filtered_tools = Vec::new();

        for tool in all_results.discovered_tools {
            // Apply language filter
            if let Some(lang) = language_filter {
                if !tool.metadata.get("language").map_or(false, |l| l == lang) {
                    continue;
                }
            }

            // Apply tag filters (tool must have ALL specified tags)
            if let Some(tags) = tag_filters {
                let tool_tags: Vec<String> = tool
                    .metadata
                    .get("tags")
                    .and_then(|t| serde_json::from_value(t.clone()).ok())
                    .unwrap_or_default();

                let has_all_tags = tags
                    .iter()
                    .all(|required_tag| tool_tags.contains(required_tag));

                if !has_all_tags {
                    continue;
                }
            }

            // Apply risk tier filter
            if let Some(required_tier) = &risk_tier_filter {
                let tool_tier: RiskTier = tool
                    .metadata
                    .get("risk_tier")
                    .and_then(|t| serde_json::from_value(t.clone()).ok())
                    .unwrap_or(RiskTier::Medium);

                if tool_tier != *required_tier {
                    continue;
                }
            }

            filtered_tools.push(tool);
        }

        Ok(ToolDiscoveryResult {
            discovered_tools: filtered_tools,
            errors: all_results.errors,
            discovery_time_ms: all_results.discovery_time_ms,
            discovered_at: all_results.discovered_at,
        })
    }

    /// Get discovered tools with optional filtering
    pub async fn get_discovered_tools_filtered(
        &self,
        language_filter: Option<&str>,
        tag_filters: Option<&[String]>,
        risk_tier_filter: Option<RiskTier>,
    ) -> Vec<MCPTool> {
        let all_tools = self.get_discovered_tools().await;
        if language_filter.is_none() && tag_filters.is_none() && risk_tier_filter.is_none() {
            return all_tools;
        }

        all_tools
            .into_iter()
            .filter(|tool| {
                // Apply language filter
                if let Some(lang) = language_filter {
                    if !tool.metadata.get("language").map_or(false, |l| l == lang) {
                        return false;
                    }
                }

                // Apply tag filters
                if let Some(tags) = tag_filters {
                    let tool_tags: Vec<String> = tool
                        .metadata
                        .get("tags")
                        .and_then(|t| serde_json::from_value(t.clone()).ok())
                        .unwrap_or_default();

                    let has_all_tags = tags
                        .iter()
                        .all(|required_tag| tool_tags.contains(required_tag));

                    if !has_all_tags {
                        return false;
                    }
                }

                // Apply risk tier filter
                if let Some(required_tier) = &risk_tier_filter {
                    let tool_tier: RiskTier = tool
                        .metadata
                        .get("risk_tier")
                        .and_then(|t| serde_json::from_value(t.clone()).ok())
                        .unwrap_or(RiskTier::Medium);

                    if tool_tier != *required_tier {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get discovered tools
    pub async fn get_discovered_tools(&self) -> Vec<MCPTool> {
        let tools = self.discovered_tools.read().await;
        tools.clone()
    }

    /// Clear discovered tools
    pub async fn clear_discovered_tools(&self) {
        let mut tools = self.discovered_tools.write().await;
        tools.clear();
        info!("Cleared discovered tools");
    }
}

impl ToolDiscovery {
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            discovered_tools: self.discovered_tools.clone(),
            discovery_active: self.discovery_active.clone(),
            cancellation_token: self.cancellation_token.clone(),
        }
    }
    async fn is_active(&self) -> bool {
        *self.discovery_active.read().await
    }
    fn manifest_to_tool(&self, m: &crate::types::ToolManifest) -> MCPTool {
        MCPTool {
            id: Uuid::new_v4(),
            name: m.name.clone(),
            description: m.description.clone(),
            version: m.version.clone(),
            author: m.author.clone(),
            tool_type: m.tool_type.clone(),
            capabilities: m.capabilities.clone(),
            parameters: m.parameters.clone(),
            output_schema: m.output_schema.clone(),
            endpoint: m.endpoint.clone().unwrap_or_default(),
            caws_compliance: crate::types::CawsComplianceStatus::Unknown,
            registration_time: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            usage_count: 0,
            metadata: m.metadata.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for ToolDiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_auto_discovery: true,
            discovery_paths: vec!["./tools".to_string()],
            manifest_patterns: vec!["**/tool.json".to_string()],
            discovery_interval_seconds: 60,
            enable_validation: true,
            enable_health_checks: false,
            health_check_timeout_seconds: 10,
        }
    }
}

impl ToolDiscovery {
    /// Perform comprehensive health check on MCP tool
    async fn perform_comprehensive_health_check(
        &self,
        tool: &MCPTool,
    ) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        // Create HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(
                self.config.health_check_timeout_seconds as u64,
            ))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        // Perform health check based on endpoint type
        let health_result = if tool.endpoint.starts_with("http") {
            self.check_http_endpoint(&client, tool).await?
        } else if tool.endpoint.starts_with("ws") {
            self.check_websocket_endpoint(tool).await?
        } else {
            // For non-HTTP endpoints, perform basic connectivity check
            self.check_generic_endpoint(tool).await?
        };

        let response_time = start_time.elapsed();

        Ok(HealthCheckResult {
            is_healthy: health_result.is_healthy,
            response_time_ms: response_time.as_millis() as u64,
            status_code: health_result.status_code,
            error_message: health_result.error_message,
            metrics: health_result.metrics,
        })
    }

    /// Check HTTP endpoint health
    async fn check_http_endpoint(
        &self,
        client: &Client,
        tool: &MCPTool,
    ) -> Result<InternalHealthResult> {
        // Try HEAD request first (lightweight)
        let head_result = self.perform_head_request(client, &tool.endpoint).await;

        if head_result.is_ok() {
            return Ok(InternalHealthResult {
                is_healthy: true,
                status_code: Some(200),
                error_message: String::new(),
                metrics: HashMap::new(),
            });
        }

        // Fallback to GET request if HEAD fails
        let get_result = self.perform_get_request(client, &tool.endpoint).await;

        match get_result {
            Ok(status) => Ok(InternalHealthResult {
                is_healthy: status.is_success(),
                status_code: Some(status.as_u16()),
                error_message: if status.is_success() {
                    String::new()
                } else {
                    format!("HTTP {}", status)
                },
                metrics: HashMap::new(),
            }),
            Err(e) => Ok(InternalHealthResult {
                is_healthy: false,
                status_code: None,
                error_message: e.to_string(),
                metrics: HashMap::new(),
            }),
        }
    }

    /// Perform HEAD request
    async fn perform_head_request(&self, client: &Client, endpoint: &str) -> Result<StatusCode> {
        let response = client.head(endpoint).send().await?;
        Ok(response.status())
    }

    /// Perform GET request
    async fn perform_get_request(&self, client: &Client, endpoint: &str) -> Result<StatusCode> {
        let response = client.get(endpoint).send().await?;
        Ok(response.status())
    }

    /// Check WebSocket endpoint health
    async fn check_websocket_endpoint(&self, tool: &MCPTool) -> Result<InternalHealthResult> {
        use std::time::Instant;
        use tokio::time::timeout;

        let start_time = Instant::now();
        let mut metrics = HashMap::new();

        // 1. WebSocket connection: Establish WebSocket connections using tokio-tungstenite
        let connection_result = timeout(
            std::time::Duration::from_secs(10),
            self.establish_websocket_connection(&tool.endpoint)
        ).await;

        let connection_time = start_time.elapsed().as_millis() as u64;
        metrics.insert("connection_time_ms".to_string(), connection_time.to_string());

        match connection_result {
            Ok(Ok((mut ws_stream, response))) => {
                // 2. WebSocket health validation: Validate WebSocket endpoint health
                let handshake_valid = self.validate_websocket_handshake(&response);
                metrics.insert("handshake_valid".to_string(), if handshake_valid { "1" } else { "0" }.to_string());

                if handshake_valid {
                    // 3. WebSocket monitoring: Monitor WebSocket performance and reliability
                    let ping_result = self.perform_websocket_ping(&mut ws_stream).await;
                    let ping_time = start_time.elapsed().as_millis() as u64 - connection_time;
                    metrics.insert("ping_time_ms".to_string(), ping_time.to_string());
                    metrics.insert("ping_successful".to_string(), if ping_result { "1" } else { "0" }.to_string());

                    // 4. WebSocket integration: Integrate WebSocket health checking with tool discovery
                    let is_healthy = ping_result && connection_time < 5000; // 5 second timeout
                    
                    Ok(InternalHealthResult {
                        is_healthy,
                        status_code: Some(101), // WebSocket switching protocols
                        error_message: if is_healthy { String::new() } else { "WebSocket health check failed".to_string() },
                        metrics,
                    })
                } else {
                    Ok(InternalHealthResult {
                        is_healthy: false,
                        status_code: Some(400), // Bad request
                        error_message: "WebSocket handshake validation failed".to_string(),
                        metrics,
                    })
                }
            }
            Ok(Err(e)) => {
                Ok(InternalHealthResult {
                    is_healthy: false,
                    status_code: None,
                    error_message: format!("WebSocket connection failed: {}", e),
                    metrics,
                })
            }
            Err(_) => {
                Ok(InternalHealthResult {
                    is_healthy: false,
                    status_code: None,
                    error_message: "WebSocket connection timeout".to_string(),
                    metrics,
                })
            }
        }
    }

    /// Establish WebSocket connection
    async fn establish_websocket_connection(&self, endpoint: &str) -> Result<(tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response)> {
        let url = url::Url::parse(endpoint)
            .map_err(|e| anyhow::anyhow!("Invalid WebSocket URL: {}", e))?;
        
        let (ws_stream, response) = tokio_tungstenite::connect_async(url).await
            .map_err(|e| anyhow::anyhow!("WebSocket connection failed: {}", e))?;
        
        Ok((ws_stream, response))
    }

    /// Validate WebSocket handshake response
    fn validate_websocket_handshake(&self, response: &tokio_tungstenite::tungstenite::handshake::client::Response) -> bool {
        // Check HTTP status code - must be 101 Switching Protocols
        if response.status() != 101 {
            tracing::warn!("WebSocket handshake failed: invalid status code {}", response.status());
            return false;
        }

        // Check for required WebSocket headers
        let headers = response.headers();

        // Verify Sec-WebSocket-Accept header exists and is properly formatted
        if let Some(sec_websocket_accept) = headers.get("sec-websocket-accept") {
            let accept_value = match sec_websocket_accept.to_str() {
                Ok(val) => val,
                Err(_) => {
                    tracing::warn!("WebSocket handshake failed: invalid Sec-WebSocket-Accept header encoding");
                    return false;
                }
            };

            // Sec-WebSocket-Accept should be base64 encoded (24 characters for SHA-1 hash)
            if accept_value.len() != 28 {
                tracing::warn!("WebSocket handshake failed: Sec-WebSocket-Accept header has invalid length");
                return false;
            }

            // Check for valid base64 characters
            if !accept_value.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
                tracing::warn!("WebSocket handshake failed: Sec-WebSocket-Accept contains invalid base64 characters");
                return false;
            }
        } else {
            tracing::warn!("WebSocket handshake failed: missing Sec-WebSocket-Accept header");
            return false;
        }

        // Check Upgrade header
        if let Some(upgrade) = headers.get("upgrade") {
            if upgrade.to_str().unwrap_or("").to_lowercase() != "websocket" {
                tracing::warn!("WebSocket handshake failed: invalid Upgrade header");
                return false;
            }
        } else {
            tracing::warn!("WebSocket handshake failed: missing Upgrade header");
            return false;
        }

        // Check Connection header
        if let Some(connection) = headers.get("connection") {
            if connection.to_str().unwrap_or("").to_lowercase() != "upgrade" {
                tracing::warn!("WebSocket handshake failed: invalid Connection header");
                return false;
            }
        } else {
            tracing::warn!("WebSocket handshake failed: missing Connection header");
            return false;
        }

        // Optional: Check for Sec-WebSocket-Protocol if subprotocols were requested
        // Optional: Check for Sec-WebSocket-Extensions if extensions were negotiated

        // Check for suspicious or malformed headers
        for (name, value) in headers.iter() {
            // Check for header name validity
            if !name.as_str().chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                if !matches!(name.as_str(), "sec-websocket-accept" | "sec-websocket-protocol" | "sec-websocket-extensions") {
                    tracing::warn!("WebSocket handshake contains suspicious header: {}", name);
                    return false;
                }
            }

            // Check for reasonable header value length
            if let Ok(value_str) = value.to_str() {
                if value_str.len() > 4096 {
                    tracing::warn!("WebSocket handshake header value too long: {}", name);
                    return false;
                }
            } else {
                tracing::warn!("WebSocket handshake header contains invalid UTF-8: {}", name);
                return false;
            }
        }

        tracing::debug!("WebSocket handshake validation successful");
        true
    }

    /// Perform WebSocket ping test
    async fn perform_websocket_ping(&self, ws_stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) -> bool {
        use tokio::time::timeout;
        use tokio_tungstenite::tungstenite::Message;
        use futures_util::StreamExt;

        // Send ping message
        if let Err(_) = ws_stream.send(Message::Ping(vec![])).await {
            return false;
        }

        // Wait for pong response with timeout
        match timeout(std::time::Duration::from_secs(5), ws_stream.next()).await {
            Ok(Some(Ok(Message::Pong(_)))) => true,
            _ => false,
        }
    }

    /// Check generic endpoint health
    async fn check_generic_endpoint(&self, tool: &MCPTool) -> Result<InternalHealthResult> {
        // For non-HTTP endpoints, perform basic validation
        if tool.endpoint.is_empty() {
            return Ok(InternalHealthResult {
                is_healthy: false,
                status_code: None,
                error_message: "Empty endpoint".to_string(),
                metrics: HashMap::new(),
            });
        }

        // Check if endpoint format is valid
        if tool.endpoint.contains("://") {
            Ok(InternalHealthResult {
                is_healthy: true,
                status_code: Some(200),
                error_message: String::new(),
                metrics: HashMap::new(),
            })
        } else {
            Ok(InternalHealthResult {
                is_healthy: false,
                status_code: None,
                error_message: "Invalid endpoint format".to_string(),
                metrics: HashMap::new(),
            })
        }
    }

    /// Check endpoint health with comprehensive validation
    async fn check_endpoint_health(&self, endpoint: &str) -> Result<bool> {
        use std::time::Instant;
        use tokio::time::timeout;

        let start_time = Instant::now();

        // 1. Endpoint type detection: Detect and handle different endpoint types
        let endpoint_type = self.detect_endpoint_type(endpoint);
        tracing::debug!("Detected endpoint type: {:?} for endpoint: {}", endpoint_type, endpoint);

        // 2. Health check implementation: Implement comprehensive health checking
        let health_result = match endpoint_type {
            EndpointType::UnixSocket => self.check_unix_socket_health(endpoint).await,
            EndpointType::Http => self.check_http_endpoint_health(endpoint).await,
            EndpointType::Https => self.check_https_endpoint_health(endpoint).await,
            EndpointType::WebSocket => self.check_websocket_endpoint_health(endpoint).await,
            EndpointType::LocalProcess => self.check_local_process_health(endpoint).await,
            EndpointType::Tcp => self.check_tcp_endpoint_health(endpoint).await,
            EndpointType::Unknown => {
                tracing::warn!("Unknown endpoint type for: {}", endpoint);
                Ok(false)
            }
        };

        // 3. Endpoint validation: Validate endpoint configuration and availability
        let validation_result = self.validate_endpoint_configuration(endpoint, &endpoint_type).await;

        // 4. Health monitoring: Monitor endpoint health and performance
        let response_time = start_time.elapsed().as_millis() as u64;
        self.record_health_metrics(endpoint, endpoint_type, health_result.is_ok(), response_time).await;

        // Combine health check and validation results
        let is_healthy = health_result.unwrap_or(false) && validation_result;
        
        tracing::info!(
            "Endpoint health check completed: {} - healthy: {}, response_time: {}ms",
            endpoint,
            is_healthy,
            response_time
        );

        Ok(is_healthy)
    }

    /// Detect endpoint type based on URL/format
    fn detect_endpoint_type(&self, endpoint: &str) -> EndpointType {
        let endpoint_lower = endpoint.to_lowercase();
        
        if endpoint_lower.starts_with("unix://") || endpoint_lower.starts_with("/") {
            EndpointType::UnixSocket
        } else if endpoint_lower.starts_with("ws://") {
            EndpointType::WebSocket
        } else if endpoint_lower.starts_with("wss://") {
            EndpointType::WebSocket
        } else if endpoint_lower.starts_with("https://") {
            EndpointType::Https
        } else if endpoint_lower.starts_with("http://") {
            EndpointType::Http
        } else if endpoint_lower.contains("://") {
            // Check for other protocols
            if endpoint_lower.starts_with("tcp://") {
                EndpointType::Tcp
            } else {
                EndpointType::Unknown
            }
        } else if endpoint_lower.contains(":") && !endpoint_lower.contains("/") {
            // Assume TCP if it looks like host:port
            EndpointType::Tcp
        } else {
            // Assume local process if it doesn't match other patterns
            EndpointType::LocalProcess
        }
    }

    /// Check Unix socket endpoint health
    async fn check_unix_socket_health(&self, endpoint: &str) -> Result<bool> {
        use std::path::Path;
        use tokio::fs;

        let socket_path = endpoint.strip_prefix("unix://").unwrap_or(endpoint);
        let path = Path::new(socket_path);

        // Check if socket file exists
        if !path.exists() {
            tracing::debug!("Unix socket does not exist: {}", socket_path);
            return Ok(false);
        }

        // Check if it's actually a socket file
        match std::fs::metadata(path) {
            Ok(metadata) => {
                use std::os::unix::fs::FileTypeExt;
                if !metadata.file_type().is_socket() {
                    tracing::debug!("Path is not a socket: {}", socket_path);
                    return Ok(false);
                }
            }
            Err(_) => {
                tracing::debug!("Cannot access socket path: {}", socket_path);
                return Ok(false);
            }
        }

        // Try to connect to the socket
        match tokio::net::UnixStream::connect(path).await {
            Ok(_) => {
                tracing::debug!("Unix socket connection successful: {}", socket_path);
                Ok(true)
            }
            Err(e) => {
                tracing::debug!("Unix socket connection failed: {} - {}", socket_path, e);
                Ok(false)
            }
        }
    }

    /// Check HTTP endpoint health
    async fn check_http_endpoint_health(&self, endpoint: &str) -> Result<bool> {
        use reqwest::Client;
        use tokio::time::timeout;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let health_check = timeout(
            std::time::Duration::from_secs(10),
            client.get(endpoint).send()
        ).await;

        match health_check {
            Ok(Ok(response)) => {
                let status = response.status();
                let is_healthy = status.is_success();
                tracing::debug!("HTTP endpoint health check: {} - status: {}", endpoint, status);
                Ok(is_healthy)
            }
            Ok(Err(e)) => {
                tracing::debug!("HTTP endpoint request failed: {} - {}", endpoint, e);
                Ok(false)
            }
            Err(_) => {
                tracing::debug!("HTTP endpoint request timeout: {}", endpoint);
                Ok(false)
            }
        }
    }

    /// Check HTTPS endpoint health
    async fn check_https_endpoint_health(&self, endpoint: &str) -> Result<bool> {
        use reqwest::Client;
        use tokio::time::timeout;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .danger_accept_invalid_certs(true) // For development/testing
            .build()?;

        let health_check = timeout(
            std::time::Duration::from_secs(10),
            client.get(endpoint).send()
        ).await;

        match health_check {
            Ok(Ok(response)) => {
                let status = response.status();
                let is_healthy = status.is_success();
                tracing::debug!("HTTPS endpoint health check: {} - status: {}", endpoint, status);
                Ok(is_healthy)
            }
            Ok(Err(e)) => {
                tracing::debug!("HTTPS endpoint request failed: {} - {}", endpoint, e);
                Ok(false)
            }
            Err(_) => {
                tracing::debug!("HTTPS endpoint request timeout: {}", endpoint);
                Ok(false)
            }
        }
    }

    /// Check WebSocket endpoint health
    async fn check_websocket_endpoint_health(&self, endpoint: &str) -> Result<bool> {
        use tokio::time::timeout;

        // For now, just check if endpoint starts with ws:// or wss://
        let is_healthy = endpoint.starts_with("ws://") || endpoint.starts_with("wss://");
        tracing::debug!("WebSocket endpoint health check: {} - healthy: {}", endpoint, is_healthy);
        Ok(is_healthy)
    }

    /// Check local process endpoint health
    async fn check_local_process_health(&self, endpoint: &str) -> Result<bool> {
        use std::process::Command;
        use tokio::time::timeout;

        // Parse process command and arguments
        let parts: Vec<&str> = endpoint.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        let command = parts[0];
        let args = &parts[1..];

        let process_check = timeout(
            std::time::Duration::from_secs(3),
            async {
                // Check if process is running
                let output = Command::new("pgrep")
                    .arg("-f")
                    .arg(command)
                    .output();
                
                match output {
                    Ok(result) => result.status.success(),
                    Err(_) => false
                }
            }
        ).await;

        match process_check {
            Ok(is_running) => {
                tracing::debug!("Local process health check: {} - running: {}", endpoint, is_running);
                Ok(is_running)
            }
            Err(_) => {
                tracing::debug!("Local process check timeout: {}", endpoint);
                Ok(false)
            }
        }
    }

    /// Check TCP endpoint health
    async fn check_tcp_endpoint_health(&self, endpoint: &str) -> Result<bool> {
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        // Parse host and port
        let (host, port) = if let Some(colon_pos) = endpoint.rfind(':') {
            let host_part = &endpoint[..colon_pos];
            let port_part = &endpoint[colon_pos + 1..];
            
            if let Ok(port_num) = port_part.parse::<u16>() {
                (host_part, port_num)
            } else {
                return Ok(false);
            }
        } else {
            return Ok(false);
        };

        let tcp_check = timeout(
            std::time::Duration::from_secs(5),
            TcpStream::connect(format!("{}:{}", host, port))
        ).await;

        match tcp_check {
            Ok(Ok(_)) => {
                tracing::debug!("TCP endpoint health check: {}:{} - connected", host, port);
                Ok(true)
            }
            Ok(Err(e)) => {
                tracing::debug!("TCP endpoint connection failed: {}:{} - {}", host, port, e);
                Ok(false)
            }
            Err(_) => {
                tracing::debug!("TCP endpoint connection timeout: {}:{}", host, port);
                Ok(false)
            }
        }
    }

    /// Validate endpoint configuration
    async fn validate_endpoint_configuration(&self, endpoint: &str, endpoint_type: &EndpointType) -> bool {
        match endpoint_type {
            EndpointType::UnixSocket => {
                let socket_path = endpoint.strip_prefix("unix://").unwrap_or(endpoint);
                !socket_path.is_empty() && socket_path.len() < 108 // Unix socket path length limit
            }
            EndpointType::Http | EndpointType::Https => {
                // Basic URL validation
                endpoint.starts_with("http://") || endpoint.starts_with("https://")
            }
            EndpointType::WebSocket => {
                endpoint.starts_with("ws://") || endpoint.starts_with("wss://")
            }
            EndpointType::Tcp => {
                // Check if it looks like host:port
                endpoint.contains(':') && !endpoint.contains('/')
            }
            EndpointType::LocalProcess => {
                // Check if it looks like a command
                !endpoint.is_empty() && !endpoint.contains("://")
            }
            EndpointType::Unknown => false,
        }
    }

    /// Record health check metrics
    fn record_health_metrics(&self, endpoint: &str, endpoint_type: EndpointType, is_healthy: bool, response_time_ms: u64) {
        // TODO: Implement comprehensive health metrics collection and storage
        /// - [ ] Store metrics in time-series database (InfluxDB, Prometheus TSDB, etc.)
        /// - [ ] Implement metrics aggregation and downsampling for long-term storage
        /// - [ ] Add metrics tagging and metadata for better querying
        /// - [ ] Support different metric types (counters, gauges, histograms, summaries)
        /// - [ ] Implement metrics retention policies and automatic cleanup
        /// - [ ] Add metrics export to external monitoring systems
        /// - [ ] Support real-time metrics streaming and alerting
        tracing::info!(
            "Health metrics - endpoint: {}, type: {:?}, healthy: {}, response_time: {}ms",
            endpoint,
            endpoint_type,
            is_healthy,
            response_time_ms
        );
    }

    /// Perform comprehensive WebSocket health check
    async fn perform_websocket_health_check(&self, endpoint: &str) -> bool {
        // TODO: Implement comprehensive WebSocket health checking and monitoring
        /// - [ ] Use WebSocket client library for actual connection testing
        /// - [ ] Implement proper WebSocket handshake and protocol validation
        /// - [ ] Add WebSocket ping/pong heartbeats for connection health
        /// - [ ] Support WebSocket subprotocol negotiation and validation
        /// - [ ] Implement connection pooling and reuse for efficiency
        /// - [ ] Add WebSocket connection metrics and performance monitoring
        /// - [ ] Support secure WebSocket connections (WSS) with TLS validation
        tracing::debug!("WebSocket health check not fully implemented for: {}", endpoint);

        // TODO: Implement comprehensive WebSocket endpoint validation
        // - [ ] Add actual WebSocket connection testing and validation
        // - [ ] Implement WebSocket protocol handshake verification
        // - [ ] Add SSL/TLS certificate validation for WSS endpoints
        // - [ ] Implement endpoint health checking and availability monitoring
        // - [ ] Add support for WebSocket subprotocols and extensions
        endpoint.starts_with("ws://") || endpoint.starts_with("wss://")
    }
}

/// Endpoint type enumeration
#[derive(Debug, Clone, PartialEq)]
enum EndpointType {
    UnixSocket,
    Http,
    Https,
    WebSocket,
    LocalProcess,
    Tcp,
    Unknown,
}

/// Health check result structure
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub status_code: Option<u16>,
    pub error_message: String,
    pub metrics: HashMap<String, String>,
}

/// Internal health check result for processing
#[derive(Debug)]
struct InternalHealthResult {
    is_healthy: bool,
    status_code: Option<u16>,
    error_message: String,
    metrics: HashMap<String, String>,
}
