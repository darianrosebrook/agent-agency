//! Endpoint management for tool discovery

use crate::mcp_types::*;
use super::health::{EndpointType, EndpointHealthCheckResult};
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
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
                            meta.insert("status_code".to_string(), serde_json::json!(response.status().as_u16()));
                            meta.insert("content_type".to_string(), serde_json::json!(response.headers()
                                .get("content-type")
                                .map(|v| v.to_str().unwrap_or(""))
                                .unwrap_or("")));
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
                        error_message: Some(format!("HTTP {}: {}", response.status().as_u16(),
                                                   response.status().canonical_reason().unwrap_or("Unknown"))),
                        metadata: HashMap::new(),
                    }
                }
            }
            Err(e) => {
                EndpointHealthCheckResult {
                    endpoint: url.to_string(),
                    endpoint_type: EndpointType::Http,
                    status: super::core::HealthStatus::Unhealthy,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    checked_at: chrono::Utc::now(),
                    error_message: Some(format!("Connection failed: {}", e)),
                    metadata: HashMap::new(),
                }
            }
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
            Err(e) => {
                EndpointHealthCheckResult {
                    endpoint: url.to_string(),
                    endpoint_type: EndpointType::WebSocket,
                    status: super::core::HealthStatus::Unhealthy,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    checked_at: chrono::Utc::now(),
                    error_message: Some(format!("Connection failed: {}", e)),
                    metadata: HashMap::new(),
                }
            }
        };

        Ok(result)
    }

    /// Check filesystem endpoint health
    pub async fn check_filesystem_endpoint(&self, path: &str) -> Result<EndpointHealthCheckResult> {
        let start = Instant::now();

        let result = match tokio::fs::metadata(path).await {
            Ok(metadata) => {
                let mut metadata_map = HashMap::new();
                metadata_map.insert("is_directory".to_string(), serde_json::json!(metadata.is_dir()));
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
            Err(e) => {
                EndpointHealthCheckResult {
                    endpoint: path.to_string(),
                    endpoint_type: EndpointType::Filesystem,
                    status: super::core::HealthStatus::Unhealthy,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    checked_at: chrono::Utc::now(),
                    error_message: Some(format!("Access failed: {}", e)),
                    metadata: HashMap::new(),
                }
            }
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
    async fn discover_from_endpoint(&self, endpoint: &str) -> Result<Vec<MCPTool>> {
        // This is a simplified implementation
        // Real implementation would parse different endpoint types

        if endpoint.starts_with("http") {
            self.discover_from_http_endpoint(endpoint).await
        } else if std::path::Path::new(endpoint).exists() {
            // Filesystem endpoint
            Ok(Vec::new()) // Simplified
        } else {
            Ok(Vec::new())
        }
    }

    /// Discover tools from HTTP endpoint
    async fn discover_from_http_endpoint(&self, url: &str) -> Result<Vec<MCPTool>> {
        let response = self.client.get(url).send().await?;
        let tools: Vec<MCPTool> = response.json().await?;
        Ok(tools)
    }
}
