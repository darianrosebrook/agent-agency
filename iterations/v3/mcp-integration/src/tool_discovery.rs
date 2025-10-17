//! Tool Discovery
//!
//! Discovers and validates MCP tools from filesystem and remote sources.

use crate::types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use glob;

/// Tool discovery service
#[derive(Debug)]
pub struct ToolDiscovery {
    pub(crate) config: ToolDiscoveryConfig,
    pub(crate) discovered_tools: Arc<RwLock<Vec<MCPTool>>>,
    pub(crate) discovery_active: Arc<RwLock<bool>>,
}

impl ToolDiscovery {
    /// Create a new tool discovery service
    pub fn new() -> Self { Self::with_config(ToolDiscoveryConfig::default()) }

    /// Create with explicit config (useful for tests)
    pub fn with_config(config: ToolDiscoveryConfig) -> Self {
        Self {
            config,
            discovered_tools: Arc::new(RwLock::new(Vec::new())),
            discovery_active: Arc::new(RwLock::new(false)),
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
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval as u64));
            loop {
                ticker.tick().await;
                // stop if deactivated
                if !this.is_active().await { break; }
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
        
        {
            let mut active = self.discovery_active.write().await;
            *active = false;
        }

        // Background task loop exits when inactive flag is false
        Ok(())
    }

    /// Discover tools from configured paths
    pub async fn discover_tools(&self) -> Result<ToolDiscoveryResult> {
        info!("Discovering tools from paths: {:?}", self.config.discovery_paths);

        let start_time = std::time::Instant::now();
        let mut discovered_tools = Vec::new();
        let mut errors = Vec::new();

        // Scan filesystem paths for manifests and parse them
        let mut set = std::collections::HashSet::new();
        for base in &self.config.discovery_paths {
            if !std::path::Path::new(base).exists() { continue; }
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
                    let manifest: Result<crate::types::ToolManifest, _> = serde_json::from_str(&content);
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
                            errors.push(DiscoveryError{
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

        info!("Tool discovery completed: {} tools, {} errors", 
            result.discovered_tools.len(), result.errors.len());

        Ok(result)
    }

    /// Validate a discovered tool
    pub async fn validate_tool(&self, tool: &MCPTool) -> Result<ValidationResult> {
        info!("Validating tool: {}", tool.name);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        if tool.name.trim().is_empty() { errors.push("name is required".into()); }
        if tool.version.trim().is_empty() { warnings.push("version missing".into()); }
        if tool.parameters.required.iter().any(|p| p.name.trim().is_empty()) {
            errors.push("parameter with empty name".into());
        }
        Ok(ValidationResult { is_valid: errors.is_empty(), errors, warnings })
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
    fn clone_for_task(&self) -> Self { Self { config: self.config.clone(), discovered_tools: self.discovered_tools.clone(), discovery_active: self.discovery_active.clone() } }
    async fn is_active(&self) -> bool { *self.discovery_active.read().await }
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
        }
    }
}
