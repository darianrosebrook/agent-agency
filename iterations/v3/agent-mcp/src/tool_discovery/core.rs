//! Core tool discovery types and service

use crate::mcp_types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use chrono::{DateTime, Utc};

/// Tool discovery service - core functionality
#[derive(Debug)]
pub struct ToolDiscovery {
    /// Configuration for tool discovery
    pub(crate) config: ToolDiscoveryConfig,
    /// Discovered tools cache
    pub(crate) discovered_tools: Arc<RwLock<Vec<MCPTool>>>,
    /// Whether discovery is currently active
    pub(crate) discovery_active: Arc<RwLock<bool>>,
    /// Cancellation token for stopping discovery
    pub(crate) cancellation_token: Arc<CancellationToken>,
}

impl ToolDiscovery {
    /// Create a new tool discovery service with default configuration
    pub fn new() -> Self {
        Self::with_config(ToolDiscoveryConfig::default())
    }

    /// Create with explicit configuration
    pub fn with_config(config: ToolDiscoveryConfig) -> Self {
        Self {
            config,
            discovered_tools: Arc::new(RwLock::new(Vec::new())),
            discovery_active: Arc::new(RwLock::new(false)),
            cancellation_token: Arc::new(CancellationToken::new()),
        }
    }

    /// Initialize the tool discovery service
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing tool discovery");

        // Validate discovery paths exist
        for path in &self.config.discovery_paths {
            if !std::path::Path::new(path).exists() {
                tracing::warn!("Discovery path does not exist: {}", path);
            }
        }

        // Validate manifest patterns are reasonable
        for pattern in &self.config.manifest_patterns {
            if pattern.is_empty() {
                tracing::warn!("Empty manifest pattern found");
            }
        }

        tracing::info!("Tool discovery initialized successfully");
        Ok(())
    }

    /// Start the tool discovery service
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting tool discovery");

        let mut active = self.discovery_active.write().await;
        if *active {
            return Ok(()); // Already started
        }
        *active = true;

        tracing::info!("Tool discovery started");
        Ok(())
    }

    /// Stop the tool discovery service
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping tool discovery");

        // Cancel the cancellation token for immediate shutdown
        self.cancellation_token.cancel();

        let mut active = self.discovery_active.write().await;
        *active = false;

        // Background task loop exits when inactive flag is false or token is cancelled
        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &ToolDiscoveryConfig {
        &self.config
    }

    /// Check if discovery is currently active
    pub async fn is_active(&self) -> bool {
        *self.discovery_active.read().await
    }

    /// Get the number of currently discovered tools
    pub async fn tool_count(&self) -> usize {
        self.discovered_tools.read().await.len()
    }
}

/// Configuration for tool discovery
#[derive(Debug, Clone)]
pub struct ToolDiscoveryConfig {
    /// Paths to search for tool manifests
    pub discovery_paths: Vec<String>,
    /// Glob patterns for manifest files
    pub manifest_patterns: Vec<String>,
    /// Whether to enable recursive discovery
    pub recursive_discovery: bool,
    /// Timeout for individual tool validations
    pub validation_timeout_seconds: u64,
    /// Maximum number of tools to discover
    pub max_tools: Option<usize>,
    /// Whether to validate tools after discovery
    pub validate_tools: bool,
    /// Cache duration for discovery results
    pub cache_duration_seconds: u64,
}

impl Default for ToolDiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_paths: vec![
                "./tools".to_string(),
                "./mcp-tools".to_string(),
                "/usr/local/lib/mcp-tools".to_string(),
            ],
            manifest_patterns: vec![
                "**/tool.json".to_string(),
                "**/manifest.json".to_string(),
                "**/mcp.json".to_string(),
            ],
            recursive_discovery: true,
            validation_timeout_seconds: 30,
            max_tools: Some(1000),
            validate_tools: true,
            cache_duration_seconds: 300, // 5 minutes
        }
    }
}

/// Result of a tool discovery operation
#[derive(Debug, Clone)]
pub struct ToolDiscoveryResult {
    /// Successfully discovered tools
    pub discovered_tools: Vec<MCPTool>,
    /// Any errors encountered during discovery
    pub errors: Vec<DiscoveryError>,
    /// Time taken for discovery in milliseconds
    pub discovery_time_ms: u64,
    /// Timestamp when discovery completed
    pub discovered_at: DateTime<Utc>,
}

impl ToolDiscoveryResult {
    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            discovered_tools: Vec::new(),
            errors: Vec::new(),
            discovery_time_ms: 0,
            discovered_at: Utc::now(),
        }
    }

    /// Check if discovery was successful (no errors)
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of successfully discovered tools
    pub fn tool_count(&self) -> usize {
        self.discovered_tools.len()
    }

    /// Get the number of errors encountered
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

/// Error encountered during tool discovery
#[derive(Debug, Clone)]
pub struct DiscoveryError {
    /// Path or location where error occurred
    pub path: String,
    /// Type of discovery error
    pub error_type: DiscoveryErrorType,
    /// Human-readable error message
    pub message: String,
    /// Optional detailed error information
    pub details: Option<String>,
}

/// Types of discovery errors
#[derive(Debug, Clone)]
pub enum DiscoveryErrorType {
    /// File not found
    FileNotFound,
    /// Permission denied
    PermissionDenied,
    /// Invalid manifest format
    InvalidManifest,
    /// Network error
    NetworkError,
    /// Validation failed
    ValidationFailed,
    /// Timeout exceeded
    Timeout,
    /// Unknown error
    Unknown,
}

impl std::fmt::Display for DiscoveryErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryErrorType::FileNotFound => write!(f, "FileNotFound"),
            DiscoveryErrorType::PermissionDenied => write!(f, "PermissionDenied"),
            DiscoveryErrorType::InvalidManifest => write!(f, "InvalidManifest"),
            DiscoveryErrorType::NetworkError => write!(f, "NetworkError"),
            DiscoveryErrorType::ValidationFailed => write!(f, "ValidationFailed"),
            DiscoveryErrorType::Timeout => write!(f, "Timeout"),
            DiscoveryErrorType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Tool discovery statistics
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    /// Total discovery time in milliseconds
    pub total_time_ms: u64,
    /// Number of paths scanned
    pub paths_scanned: usize,
    /// Number of manifest files found
    pub manifests_found: usize,
    /// Number of tools successfully discovered
    pub tools_discovered: usize,
    /// Number of validation errors
    pub validation_errors: usize,
    /// Number of network errors
    pub network_errors: usize,
    /// Timestamp when stats were collected
    pub collected_at: DateTime<Utc>,
}

impl Default for DiscoveryStats {
    fn default() -> Self {
        Self {
            total_time_ms: 0,
            paths_scanned: 0,
            manifests_found: 0,
            tools_discovered: 0,
            validation_errors: 0,
            network_errors: 0,
            collected_at: Utc::now(),
        }
    }
}
