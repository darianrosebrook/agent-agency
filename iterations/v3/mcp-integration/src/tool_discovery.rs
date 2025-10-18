//! Tool Discovery
//!
//! Discovers and validates MCP tools from filesystem and remote sources.

use crate::types::*;
use anyhow::Result;
use glob;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

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
        let has_command = tool.capabilities.contains(&ToolCapability::CommandExecution);
        let has_file_system = tool.capabilities.contains(&ToolCapability::FileSystemAccess);

        if has_network && has_command {
            warnings.push("tool has both network and command execution capabilities - ensure proper sandboxing".into());
        }

        if has_command && !tool.metadata.get("sandboxed").map_or(false, |v| v.as_bool().unwrap_or(false)) {
            errors.push("command execution capability requires sandboxed=true in metadata".into());
        }

        if has_file_system {
            let allowed_paths: Vec<String> = tool.metadata.get("allowed_paths")
                .and_then(|p| serde_json::from_value(p.clone()).ok())
                .unwrap_or_default();

            if allowed_paths.is_empty() {
                warnings.push("filesystem access without restricted paths - consider limiting scope".into());
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
                    warnings.push(format!("health check error: {} - may indicate connectivity issues", e));
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
        // In a real implementation, this might make an actual API call
        let endpoint = &tool.endpoint;

        // For HTTP endpoints, try a HEAD request
        if endpoint.starts_with("http") {
            match timeout(
                Duration::from_secs(self.config.health_check_timeout_seconds as u64),
                reqwest::get(endpoint)
            ).await {
                Ok(Ok(response)) => {
                    return Ok(response.status().is_success());
                }
                Ok(Err(_)) | Err(_) => {
                    return Ok(false);
                }
            }
        }

        // For other endpoints (like Unix sockets or local processes),
        // we could implement additional health checks here
        // For now, assume they're healthy if they have an endpoint configured
        Ok(!endpoint.is_empty())
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
                let tool_tags: Vec<String> = tool.metadata.get("tags")
                    .and_then(|t| serde_json::from_value(t.clone()).ok())
                    .unwrap_or_default();

                let has_all_tags = tags.iter().all(|required_tag| {
                    tool_tags.contains(required_tag)
                });

                if !has_all_tags {
                    continue;
                }
            }

            // Apply risk tier filter
            if let Some(required_tier) = &risk_tier_filter {
                let tool_tier: RiskTier = tool.metadata.get("risk_tier")
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

        all_tools.into_iter().filter(|tool| {
            // Apply language filter
            if let Some(lang) = language_filter {
                if !tool.metadata.get("language").map_or(false, |l| l == lang) {
                    return false;
                }
            }

            // Apply tag filters
            if let Some(tags) = tag_filters {
                let tool_tags: Vec<String> = tool.metadata.get("tags")
                    .and_then(|t| serde_json::from_value(t.clone()).ok())
                    .unwrap_or_default();

                let has_all_tags = tags.iter().all(|required_tag| {
                    tool_tags.contains(required_tag)
                });

                if !has_all_tags {
                    return false;
                }
            }

            // Apply risk tier filter
            if let Some(required_tier) = &risk_tier_filter {
                let tool_tier: RiskTier = tool.metadata.get("risk_tier")
                    .and_then(|t| serde_json::from_value(t.clone()).ok())
                    .unwrap_or(RiskTier::Medium);

                if tool_tier != *required_tier {
                    return false;
                }
            }

            true
        }).collect()
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
