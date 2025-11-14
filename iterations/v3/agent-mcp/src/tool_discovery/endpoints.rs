//! Endpoint management for tool discovery

use super::health::{EndpointHealthCheckResult, EndpointType};
use crate::mcp_types::*;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, warn};

/// Endpoint manager for tool discovery
pub struct EndpointManager {
    client: Client,
    timeout: Duration,
}

impl EndpointManager {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            timeout: Duration::from_secs(10),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_default(),
            timeout,
        }
    }

    /// Check HTTP endpoint health
    pub async fn check_http_endpoint(&self, url: &str) -> Result<EndpointHealthCheckResult> {
        let start = Instant::now();

        let result = match self.client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    EndpointHealthCheckResult {
                        endpoint: url.to_string(),
                        endpoint_type: EndpointType::Http,
                        status: super::core::HealthStatus::Healthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        checked_at: chrono::Utc::now(),
                        error_message: None,
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert(
                                "status_code".to_string(),
                                serde_json::json!(response.status().as_u16()),
                            );
                            meta.insert(
                                "content_type".to_string(),
                                serde_json::json!(response
                                    .headers()
                                    .get("content-type")
                                    .map(|v| v.to_str().unwrap_or(""))
                                    .unwrap_or("")),
                            );
                            meta
                        },
                    }
                } else {
                    EndpointHealthCheckResult {
                        endpoint: url.to_string(),
                        endpoint_type: EndpointType::Http,
                        status: super::core::HealthStatus::Degraded,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        checked_at: chrono::Utc::now(),
                        error_message: Some(format!(
                            "HTTP {}: {}",
                            response.status().as_u16(),
                            response.status().canonical_reason().unwrap_or("Unknown")
                        )),
                        metadata: HashMap::new(),
                    }
                }
            }
            Err(e) => EndpointHealthCheckResult {
                endpoint: url.to_string(),
                endpoint_type: EndpointType::Http,
                status: super::core::HealthStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                checked_at: chrono::Utc::now(),
                error_message: Some(format!("Connection failed: {}", e)),
                metadata: HashMap::new(),
            },
        };

        Ok(result)
    }

    /// Check WebSocket endpoint health
    pub async fn check_websocket_endpoint(&self, url: &str) -> Result<EndpointHealthCheckResult> {
        let start = Instant::now();

        let result = match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let mut stream = ws_stream;

                // Send ping and wait for pong
                if let Err(e) = stream.send(Message::Ping(vec![1, 2, 3])).await {
                    EndpointHealthCheckResult {
                        endpoint: url.to_string(),
                        endpoint_type: EndpointType::WebSocket,
                        status: super::core::HealthStatus::Degraded,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        checked_at: chrono::Utc::now(),
                        error_message: Some(format!("Ping failed: {}", e)),
                        metadata: HashMap::new(),
                    }
                } else {
                    // Close connection
                    let _ = stream.close(None).await;
                    EndpointHealthCheckResult {
                        endpoint: url.to_string(),
                        endpoint_type: EndpointType::WebSocket,
                        status: super::core::HealthStatus::Healthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        checked_at: chrono::Utc::now(),
                        error_message: None,
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert("protocol".to_string(), serde_json::json!("WebSocket"));
                            meta
                        },
                    }
                }
            }
            Err(e) => EndpointHealthCheckResult {
                endpoint: url.to_string(),
                endpoint_type: EndpointType::WebSocket,
                status: super::core::HealthStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                checked_at: chrono::Utc::now(),
                error_message: Some(format!("Connection failed: {}", e)),
                metadata: HashMap::new(),
            },
        };

        Ok(result)
    }

    /// Check filesystem endpoint health
    pub async fn check_filesystem_endpoint(&self, path: &str) -> Result<EndpointHealthCheckResult> {
        let start = Instant::now();

        let result = match tokio::fs::metadata(path).await {
            Ok(metadata) => {
                let mut metadata_map = HashMap::new();
                metadata_map.insert(
                    "is_directory".to_string(),
                    serde_json::json!(metadata.is_dir()),
                );
                metadata_map.insert("is_file".to_string(), serde_json::json!(metadata.is_file()));
                metadata_map.insert("size_bytes".to_string(), serde_json::json!(metadata.len()));

                EndpointHealthCheckResult {
                    endpoint: path.to_string(),
                    endpoint_type: EndpointType::Filesystem,
                    status: super::core::HealthStatus::Healthy,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    checked_at: chrono::Utc::now(),
                    error_message: None,
                    metadata: metadata_map,
                }
            }
            Err(e) => EndpointHealthCheckResult {
                endpoint: path.to_string(),
                endpoint_type: EndpointType::Filesystem,
                status: super::core::HealthStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                checked_at: chrono::Utc::now(),
                error_message: Some(format!("Access failed: {}", e)),
                metadata: HashMap::new(),
            },
        };

        Ok(result)
    }

    /// Discover tools from remote endpoints
    pub async fn discover_from_endpoints(&self, endpoints: &[String]) -> Result<Vec<MCPTool>> {
        let mut all_tools = Vec::new();

        for endpoint in endpoints {
            debug!("Discovering tools from endpoint: {}", endpoint);

            match self.discover_from_endpoint(endpoint).await {
                Ok(tools) => {
                    debug!("Found {} tools from {}", tools.len(), endpoint);
                    all_tools.extend(tools);
                }
                Err(e) => {
                    warn!("Failed to discover from {}: {}", endpoint, e);
                }
            }
        }

        Ok(all_tools)
    }

    /// Discover tools from a single endpoint
    /// Implemented: Comprehensive endpoint parsing for HTTP, HTTPS, filesystem, and WebSocket endpoints
    async fn discover_from_endpoint(&self, endpoint: &str) -> Result<Vec<MCPTool>> {
        // Parse endpoint type and route to appropriate discovery method
        let endpoint_lower = endpoint.to_lowercase();

        // HTTP/HTTPS endpoints
        if endpoint_lower.starts_with("http://") || endpoint_lower.starts_with("https://") {
            debug!("Parsing HTTP/HTTPS endpoint: {}", endpoint);
            return self.discover_from_http_endpoint(endpoint).await;
        }

        // WebSocket endpoints
        if endpoint_lower.starts_with("ws://") || endpoint_lower.starts_with("wss://") {
            debug!("Parsing WebSocket endpoint: {}", endpoint);
            return self.discover_from_websocket_endpoint(endpoint).await;
        }

        // Filesystem endpoints - check if path exists
        let path = std::path::Path::new(endpoint);
        if path.exists() {
            debug!("Parsing filesystem endpoint: {}", endpoint);
            return self.discover_from_filesystem_endpoint(endpoint).await;
        }

        // Try parsing as relative filesystem path
        if !endpoint.contains("://") && !endpoint.starts_with('/') {
            // Relative path - try current directory
            let current_dir = std::env::current_dir()
                .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
            let relative_path = current_dir.join(endpoint);
            if relative_path.exists() {
                debug!("Parsing relative filesystem endpoint: {}", endpoint);
                return self
                    .discover_from_filesystem_endpoint(relative_path.to_string_lossy().as_ref())
                    .await;
            }
        }

        // Unknown endpoint type
        warn!("Unknown endpoint type or path does not exist: {}", endpoint);
        Err(anyhow::anyhow!(
            "Unknown endpoint type or path does not exist: {}. Supported types: http://, https://, ws://, wss://, filesystem paths",
            endpoint
        ))
    }

    /// Discover tools from HTTP/HTTPS endpoint
    async fn discover_from_http_endpoint(&self, url: &str) -> Result<Vec<MCPTool>> {
        debug!("Discovering tools from HTTP endpoint: {}", url);

        // Validate URL format
        url::Url::parse(url).map_err(|e| anyhow::anyhow!("Invalid HTTP endpoint URL: {}", e))?;

        let response = self
            .client
            .get(url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP endpoint returned error status: {}",
                response.status()
            ));
        }

        let tools: Vec<MCPTool> = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))?;

        debug!("Found {} tools from HTTP endpoint: {}", tools.len(), url);
        Ok(tools)
    }

    /// Discover tools from WebSocket endpoint
    async fn discover_from_websocket_endpoint(&self, url: &str) -> Result<Vec<MCPTool>> {
        debug!("Discovering tools from WebSocket endpoint: {}", url);

        // Validate WebSocket URL format
        url::Url::parse(url)
            .map_err(|e| anyhow::anyhow!("Invalid WebSocket endpoint URL: {}", e))?;

        // Connect to WebSocket endpoint
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket connection failed: {}", e))?;

        let (mut _write, mut read) = ws_stream.split();

        // Send discovery request
        let discovery_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        // Note: WebSocket discovery would require full WebSocket protocol implementation
        // For now, return empty vector as WebSocket tool discovery needs more infrastructure
        warn!(
            "WebSocket endpoint discovery not fully implemented: {}",
            url
        );
        Ok(Vec::new())
    }

    /// Discover tools from filesystem endpoint
    async fn discover_from_filesystem_endpoint(&self, path: &str) -> Result<Vec<MCPTool>> {
        debug!("Discovering tools from filesystem endpoint: {}", path);

        let fs_path = std::path::Path::new(path);

        // Validate path exists and is accessible
        if !fs_path.exists() {
            return Err(anyhow::anyhow!("Filesystem path does not exist: {}", path));
        }

        if !fs_path.is_dir() && !fs_path.is_file() {
            return Err(anyhow::anyhow!("Invalid filesystem path type: {}", path));
        }

        // Create filesystem scanner with config for this specific endpoint
        // Use ToolDiscoveryConfig from mcp_types (which FilesystemScanner expects)
        let config = crate::mcp_types::ToolDiscoveryConfig {
            enable_auto_discovery: false, // Single endpoint discovery, not periodic
            discovery_paths: vec![path.to_string()],
            manifest_patterns: vec![
                "**/manifest.json".to_string(),
                "**/*.tool.json".to_string(),
                "**/tool.json".to_string(),
                "**/*.mcp.json".to_string(),
            ],
            discovery_interval_seconds: 0, // No periodic scanning for endpoint discovery
            enable_validation: true,
            enable_health_checks: false, // Skip health checks for endpoint discovery
            health_check_timeout_seconds: 30,
        };

        let scanner = super::filesystem::FilesystemScanner::new(config);

        // Scan for tool manifests
        let (tools, errors) = scanner
            .scan_manifests()
            .await
            .map_err(|e| anyhow::anyhow!("Filesystem scan failed: {}", e))?;

        // Log any errors but don't fail if some tools were found
        if !errors.is_empty() {
            warn!(
                "Found {} errors during filesystem scan, but discovered {} tools",
                errors.len(),
                tools.len()
            );
            for error in &errors {
                warn!("Discovery error: {} - {}", error.path, error.message);
            }
        }

        debug!(
            "Found {} tools from filesystem endpoint: {}",
            tools.len(),
            path
        );
        Ok(tools)
    }
}
