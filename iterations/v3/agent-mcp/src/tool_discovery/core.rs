//! Core tool discovery types and service

use schemars::JsonSchema;
use crate::mcp_types::*;
use crate::tool_discovery::validation::ValidationResult;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use glob::glob;
use uuid::Uuid;

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

    /// Start automatic tool discovery
    pub async fn start_auto_discovery(&self) -> Result<()> {
        tracing::info!("Starting automatic tool discovery");
        
        // Set discovery as active
        {
            let mut active = self.discovery_active.write().await;
            *active = true;
        }
        
        // Start discovery loop
        let config = self.config.clone();
        let tools = self.discovered_tools.clone();
        let active = self.discovery_active.clone();
        let token = self.cancellation_token.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(config.cache_duration_seconds)
            );
            
            loop {
                if token.is_cancelled() {
                    break;
                }
                
                interval.tick().await;
                
                // Check if discovery is still active
                {
                    let is_active = active.read().await;
                    if !*is_active {
                        break;
                    }
                }
                
                // Perform discovery
                match Self::perform_discovery(&config).await {
                    Ok(new_tools) => {
                        let mut tools_guard = tools.write().await;
                        *tools_guard = new_tools;
                        tracing::debug!("Discovered {} tools", tools_guard.len());
                    }
                    Err(e) => {
                        tracing::error!("Tool discovery failed: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Discover tools from configured paths
    pub async fn discover_tools(&self) -> Result<ToolDiscoveryResult> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        
        match Self::perform_discovery(&self.config).await {
            Ok(tools) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Update the cache
                {
                    let mut tools_guard = self.discovered_tools.write().await;
                    *tools_guard = tools.clone();
                }
                
                Ok(ToolDiscoveryResult {
                    discovered_tools: tools,
                    discovery_time_ms: execution_time_ms,
                    discovered_at: Utc::now(),
                    errors,
                })
            }
            Err(e) => {
                errors.push(DiscoveryError {
                    path: "unknown".to_string(),
                    error_type: DiscoveryErrorType::Unknown,
                    message: e.to_string(),
                    details: None,
                });
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                
                Ok(ToolDiscoveryResult {
                    discovered_tools: Vec::new(),
                    discovery_time_ms: execution_time_ms,
                    discovered_at: Utc::now(),
                    errors,
                })
            }
        }
    }
    
    /// Perform the actual discovery process
    async fn perform_discovery(config: &ToolDiscoveryConfig) -> Result<Vec<MCPTool>> {
        let mut tools = Vec::new();
        
        for path in &config.discovery_paths {
            if let Ok(path_tools) = Self::discover_tools_in_path(path, config).await {
                tools.extend(path_tools);
            }
        }
        
        Ok(tools)
    }
    
    /// Discover tools in a specific path
    async fn discover_tools_in_path(path: &str, config: &ToolDiscoveryConfig) -> Result<Vec<MCPTool>> {
        let mut tools = Vec::new();
        
        tracing::debug!("Discovering tools in path: {}", path);
        
        // Check if path exists
        let path_buf = std::path::PathBuf::from(path);
        if !path_buf.exists() {
            tracing::warn!("Discovery path does not exist: {}", path);
            return Ok(tools);
        }
        
        // Search for manifest files matching patterns
        for pattern in &config.manifest_patterns {
            let glob_pattern = if config.recursive_discovery {
                format!("{}/**/{}", path, pattern.trim_start_matches("**/"))
            } else {
                format!("{}/{}", path, pattern.trim_start_matches("**/"))
            };
            
            tracing::debug!("Searching for tools with pattern: {}", glob_pattern);
            
            // Use glob to find matching files
            match glob::glob(&glob_pattern) {
                Ok(paths) => {
                    for entry in paths {
                        match entry {
                            Ok(manifest_path) => {
                                if let Some(tool) = Self::load_tool_from_manifest(&manifest_path, config).await {
                                    tools.push(tool);
                                    
                                    // Check max_tools limit
                                    if let Some(max) = config.max_tools {
                                        if tools.len() >= max {
                                            tracing::info!("Reached max_tools limit ({})", max);
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Error reading glob entry: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Invalid glob pattern {}: {}", glob_pattern, e);
                }
            }
        }
        
        tracing::info!("Discovered {} tools in path: {}", tools.len(), path);
        Ok(tools)
    }
    
    /// Load a tool from a manifest file
    async fn load_tool_from_manifest(manifest_path: &std::path::Path, config: &ToolDiscoveryConfig) -> Option<MCPTool> {
        tracing::debug!("Loading tool from manifest: {}", manifest_path.display());
        
        // Read manifest file
        let contents = match tokio::fs::read_to_string(manifest_path).await {
            Ok(contents) => contents,
            Err(e) => {
                tracing::warn!("Failed to read manifest file {}: {}", manifest_path.display(), e);
                return None;
            }
        };
        
        // Parse JSON manifest
        let manifest: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(value) => value,
            Err(e) => {
                tracing::warn!("Failed to parse manifest JSON {}: {}", manifest_path.display(), e);
                return None;
            }
        };
        
        // Extract tool information from manifest
        let tool_name = manifest.get("name")
            .or_else(|| manifest.get("tool_name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                manifest_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown_tool")
                    .to_string()
            });
        
        let tool_description = manifest.get("description")
            .or_else(|| manifest.get("tool_description"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Tool discovered from {}", manifest_path.display()));
        
        let tool_version = manifest.get("version")
            .or_else(|| manifest.get("tool_version"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "1.0.0".to_string());
        
        let tool_author = manifest.get("author")
            .or_else(|| manifest.get("tool_author"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Extract tool type
        let tool_type = manifest.get("type")
            .or_else(|| manifest.get("tool_type"))
            .and_then(|v| v.as_str())
            .and_then(|s| match s.to_lowercase().as_str() {
                "code_generation" | "codegeneration" => Some(ToolType::CodeGeneration),
                "code_analysis" | "codeanalysis" => Some(ToolType::CodeAnalysis),
                "testing" => Some(ToolType::Testing),
                "documentation" => Some(ToolType::Documentation),
                "build" => Some(ToolType::Build),
                "deployment" => Some(ToolType::Deployment),
                "monitoring" => Some(ToolType::Monitoring),
                "utility" => Some(ToolType::Utility),
                custom => Some(ToolType::Custom(custom.to_string())),
            })
            .unwrap_or(ToolType::Utility);
        
        // Extract capabilities
        let capabilities: Vec<ToolCapability> = manifest.get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        v.as_str().and_then(|s| {
                            // Try to parse as ToolCapability
                            Some(ToolCapability::from_str(s))
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        // Extract parameters schema
        let parameters = manifest.get("parameters")
            .or_else(|| manifest.get("schema"))
            .or_else(|| manifest.get("input_schema"))
            .and_then(|v| {
                serde_json::from_value::<ToolParameters>(v.clone()).ok()
            })
            .unwrap_or_else(|| ToolParameters {
                required: vec![],
                optional: vec![],
                constraints: vec![],
            });
        
        // Extract output schema
        let output_schema = manifest.get("output_schema")
            .or_else(|| manifest.get("output"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({
                "type": "object"
            }));
        
        // Extract metadata
        let metadata = manifest.get("metadata")
            .and_then(|v| serde_json::from_value::<HashMap<String, serde_json::Value>>(v.clone()).ok())
            .unwrap_or_default();

        // Extract endpoint
        let endpoint = manifest.get("endpoint")
            .or_else(|| manifest.get("url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("local://{}", tool_name));

        // Create tool manifest
        let tool_manifest = ToolManifest {
            name: tool_name.clone(),
            version: tool_version.clone(),
            description: tool_description.clone(),
            author: tool_author.clone(),
            tool_type: tool_type.clone(),
            entry_point: endpoint.clone(),
            dependencies: vec![], // TODO: parse dependencies
            capabilities: capabilities.clone(),
            parameters: parameters.clone(),
            output_schema: output_schema.clone(),
            endpoint: Some(endpoint.clone()),
            caws_compliance: None, // TODO: parse caws compliance
            metadata: metadata.clone(),
            configuration_schema: serde_json::json!({}), // TODO: parse config schema
        };
        
        // Create MCPTool instance
        let tool = MCPTool {
            id: uuid::Uuid::new_v4(),
            name: tool_name.clone(),
            description: tool_description.clone(),
            version: tool_version.clone(),
            author: tool_author.clone(),
            tool_type: tool_type.clone(),
            capabilities,
            parameters,
            output_schema,
            endpoint,
            manifest: tool_manifest,
            caws_compliance: CawsComplianceStatus::Unknown, // Will be validated later
            registration_time: Utc::now(),
            last_updated: Utc::now(),
            usage_count: 0,
            metadata,
        };
        
        // Validate tool if configured
        if config.validate_tools {
            // TODO: Integrate with validation module when available
            tracing::debug!("Tool validation requested for: {}", tool_name);
        }
        
        Some(tool)
    }
}

/// Configuration for tool discovery
#[derive(Debug, Clone, JsonSchema)]
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
#[derive(Debug, Clone, JsonSchema)]
pub struct ToolDiscoveryResult {
    /// Successfully discovered tools
    pub discovered_tools: Vec<MCPTool>,
    /// Any errors encountered during discovery
    pub errors: Vec<DiscoveryError>,
    /// Time taken for discovery in milliseconds
    pub discovery_time_ms: u64,
    /// Timestamp when discovery completed
    #[schemars(with = "String")]
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

impl ToolDiscovery {
    /// Validate a tool using basic validation
    pub async fn validate_tool(&self, tool: &MCPTool) -> Result<ValidationResult> {
        use crate::tool_discovery::validation::{BasicToolValidator, ToolValidator};
        
        let validator = BasicToolValidator::new();
        validator.validate_tool(tool).await
    }

    /// Shutdown the tool discovery service
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down tool discovery");
        
        // Cancel the cancellation token
        self.cancellation_token.cancel();
        
        // Set discovery as inactive
        {
            let mut active = self.discovery_active.write().await;
            *active = false;
        }
        
        tracing::info!("Tool discovery shutdown complete");
        Ok(())
    }
}

/// Error encountered during tool discovery
#[derive(Debug, Clone, JsonSchema)]
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
#[derive(Debug, Clone, JsonSchema)]
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
#[derive(Debug, Clone, JsonSchema)]
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
    #[schemars(with = "String")]
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

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

impl Default for HealthCheckResult {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            message: "Health check not performed".to_string(),
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }
}
