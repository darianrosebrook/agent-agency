//! Tool Registry - Central Registry for CAWS Tooling
//!
//! Manages registration, discovery, and metadata for all CAWS tools
//! in the ecosystem, providing unified access and governance.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Central tool registry
#[derive(Debug)]
pub struct ToolRegistry {
    /// Registered tools by name
    tools: Arc<RwLock<HashMap<String, RegisteredTool>>>,
    /// Tool categories for organization
    categories: Arc<RwLock<HashMap<String, Vec<String>>>>, // category -> tool_names
    /// Tool health monitoring
    health_monitor: Arc<RwLock<HashMap<String, ToolHealth>>>,
}

/// A registered tool in the ecosystem
#[derive(Debug, Clone)]
pub struct RegisteredTool {
    /// Tool metadata
    pub metadata: ToolMetadata,
    /// Tool implementation (boxed trait object)
    pub implementation: Arc<dyn Tool + Send + Sync>,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Usage count
    pub usage_count: u64,
}

/// Tool metadata for discovery and documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool name (unique identifier)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Tool version
    pub version: String,
    /// Tool category
    pub category: ToolCategory,
    /// Input schema (JSON Schema)
    pub input_schema: Option<serde_json::Value>,
    /// Output schema (JSON Schema)
    pub output_schema: Option<serde_json::Value>,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Tool capabilities
    pub capabilities: Vec<String>,
    /// Cost estimation (relative units)
    pub cost_estimate: Option<f64>,
    /// Timeout recommendation (ms)
    pub timeout_ms: Option<u64>,
    /// Author/maintainer
    pub author: String,
    /// License
    pub license: String,
}

/// Tool categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    /// Policy enforcement and compliance
    Policy,
    /// Conflict resolution and debate
    ConflictResolution,
    /// Evidence collection and verification
    EvidenceCollection,
    /// Governance and audit
    Governance,
    /// Quality gates and validation
    QualityGate,
    /// Reasoning and inference
    Reasoning,
    /// Workflow and orchestration
    Workflow,
}

/// Tool registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistration {
    /// Tool metadata
    pub metadata: ToolMetadata,
    /// Registration options
    pub options: RegistrationOptions,
}

/// Registration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationOptions {
    /// Enable health monitoring
    pub enable_health_monitoring: bool,
    /// Enable usage tracking
    pub enable_usage_tracking: bool,
    /// Override existing registration
    pub override_existing: bool,
    /// Auto-discover capabilities
    pub auto_discover: bool,
}

/// Tool health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolHealth {
    /// Tool is healthy
    pub healthy: bool,
    /// Last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Response time (ms)
    pub response_time_ms: Option<u64>,
    /// Error count
    pub error_count: u32,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
    /// Health check details
    pub details: Option<String>,
}

/// Tool trait for unified execution interface
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Get tool metadata
    fn metadata(&self) -> &ToolMetadata;

    /// Execute the tool
    async fn execute(&self, parameters: serde_json::Value, context: Option<&str>) -> Result<serde_json::Value>;

    /// Validate tool parameters
    async fn validate_parameters(&self, parameters: &serde_json::Value) -> Result<()> {
        // Default implementation - validate against schema if available
        if let Some(schema) = &self.metadata().input_schema {
            let compiled = jsonschema::JSONSchema::compile(schema)
                .map_err(|e| anyhow::anyhow!("Invalid schema: {}", e))?;

            compiled.validate(parameters)
                .map_err(|e| anyhow::anyhow!("Parameter validation failed: {:?}", e))?;
        }

        Ok(())
    }

    /// Get tool health status
    async fn health_check(&self) -> Result<ToolHealth> {
        // Default implementation - assume healthy
        Ok(ToolHealth {
            healthy: true,
            last_check: chrono::Utc::now(),
            response_time_ms: Some(0),
            error_count: 0,
            success_rate: 1.0,
            details: Some("Default health check".to_string()),
        })
    }

    /// Get tool capabilities
    fn capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            health_monitor: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool in the registry
    pub async fn register_tool<T: Tool + 'static>(&self, tool: Arc<T>) -> Result<()> {
        let metadata = tool.metadata().clone();
        let tool_name = metadata.name.clone();

        info!("Registering tool: {} (category: {:?})", tool_name, metadata.category);

        // Check if tool already exists
        {
            let tools = self.tools.read().await;
            if tools.contains_key(&tool_name) {
                return Err(anyhow::anyhow!("Tool '{}' already registered", tool_name));
            }
        }

        // Create registered tool
        let registered = RegisteredTool {
            metadata: metadata.clone(),
            implementation: tool,
            registered_at: chrono::Utc::now(),
            last_used: None,
            usage_count: 0,
        };

        // Add to registry
        {
            let mut tools = self.tools.write().await;
            tools.insert(tool_name.clone(), registered);
        }

        // Add to category
        {
            let mut categories = self.categories.write().await;
            let category_key = format!("{:?}", metadata.category);
            categories.entry(category_key).or_insert_with(Vec::new).push(tool_name.clone());
        }

        debug!("Successfully registered tool: {}", tool_name);
        Ok(())
    }

    /// Unregister a tool from the registry
    pub async fn unregister_tool(&self, tool_name: &str) -> Result<()> {
        info!("Unregistering tool: {}", tool_name);

        // Remove from tools
        let removed = {
            let mut tools = self.tools.write().await;
            tools.remove(tool_name)
        };

        if removed.is_none() {
            return Err(anyhow::anyhow!("Tool '{}' not found", tool_name));
        }

        // Remove from categories
        {
            let mut categories = self.categories.write().await;
            for tools_list in categories.values_mut() {
                tools_list.retain(|name| name != tool_name);
            }
            // Remove empty categories
            categories.retain(|_, tools| !tools.is_empty());
        }

        // Remove from health monitor
        {
            let mut health = self.health_monitor.write().await;
            health.remove(tool_name);
        }

        debug!("Successfully unregistered tool: {}", tool_name);
        Ok(())
    }

    /// Get a registered tool by name
    pub async fn get_tool(&self, tool_name: &str) -> Option<RegisteredTool> {
        let tools = self.tools.read().await;
        tools.get(tool_name).cloned()
    }

    /// Get all registered tools
    pub async fn get_all_tools(&self) -> HashMap<String, RegisteredTool> {
        self.tools.read().await.clone()
    }

    /// Get tools by category
    pub async fn get_tools_by_category(&self, category: ToolCategory) -> Vec<RegisteredTool> {
        let categories = self.categories.read().await;
        let tools = self.tools.read().await;

        let category_key = format!("{:?}", category);
        if let Some(tool_names) = categories.get(&category_key) {
            tool_names.iter()
                .filter_map(|name| tools.get(name).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get tool count
    pub async fn get_tool_count(&self) -> usize {
        self.tools.read().await.len()
    }

    /// Get active tool count (healthy tools)
    pub async fn get_active_tool_count(&self) -> usize {
        let health = self.health_monitor.read().await;
        health.values().filter(|h| h.healthy).count()
    }

    /// Search tools by capability
    pub async fn search_by_capability(&self, capability: &str) -> Vec<RegisteredTool> {
        let tools = self.tools.read().await;
        tools.values()
            .filter(|tool| tool.metadata.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }

    /// Update tool usage statistics
    pub async fn update_tool_usage(&self, tool_name: &str) -> Result<()> {
        let mut tools = self.tools.write().await;

        if let Some(tool) = tools.get_mut(tool_name) {
            tool.last_used = Some(chrono::Utc::now());
            tool.usage_count += 1;
        } else {
            return Err(anyhow::anyhow!("Tool '{}' not found", tool_name));
        }

        Ok(())
    }

    /// Get tool health status
    pub async fn get_tool_health(&self, tool_name: &str) -> Option<ToolHealth> {
        let health = self.health_monitor.read().await;
        health.get(tool_name).cloned()
    }

    /// Update tool health status
    pub async fn update_tool_health(&self, tool_name: &str, health: ToolHealth) -> Result<()> {
        let mut health_monitor = self.health_monitor.write().await;
        health_monitor.insert(tool_name.to_string(), health);
        Ok(())
    }

    /// Run health checks for all tools
    pub async fn run_health_checks(&self) -> Result<HashMap<String, ToolHealth>> {
        info!("Running health checks for all tools");

        let tools = self.tools.read().await.clone();
        let mut results = HashMap::new();

        for (tool_name, registered_tool) in tools {
            let health_result = registered_tool.implementation.health_check().await;

            let health = match health_result {
                Ok(health) => health,
                Err(e) => {
                    warn!("Health check failed for tool {}: {}", tool_name, e);
                    ToolHealth {
                        healthy: false,
                        last_check: chrono::Utc::now(),
                        response_time_ms: None,
                        error_count: 1,
                        success_rate: 0.0,
                        details: Some(format!("Health check error: {}", e)),
                    }
                }
            };

            // Update health monitor
            let mut health_monitor = self.health_monitor.write().await;
            health_monitor.insert(tool_name.clone(), health.clone());

            results.insert(tool_name, health);
        }

        info!("Completed health checks for {} tools", results.len());
        Ok(results)
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        let tools = self.tools.read().await;
        let categories = self.categories.read().await;
        let health = self.health_monitor.read().await;

        let total_tools = tools.len();
        let healthy_tools = health.values().filter(|h| h.healthy).count();
        let total_usage = tools.values().map(|t| t.usage_count).sum::<u64>();

        RegistryStatistics {
            total_tools,
            healthy_tools,
            categories_count: categories.len(),
            total_usage,
            average_success_rate: if !health.is_empty() {
                health.values().map(|h| h.success_rate).sum::<f64>() / health.len() as f64
            } else {
                1.0
            },
        }
    }

    /// Export registry as JSON
    pub async fn export_registry(&self) -> Result<serde_json::Value> {
        let tools = self.tools.read().await;
        let categories = self.categories.read().await;
        let health = self.health_monitor.read().await;

        let export = RegistryExport {
            tools: tools.iter().map(|(name, tool)| (name.clone(), tool.metadata.clone())).collect(),
            categories: categories.clone(),
            health: health.clone(),
            exported_at: chrono::Utc::now(),
        };

        serde_json::to_value(export).context("Failed to serialize registry export")
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    /// Total number of registered tools
    pub total_tools: usize,
    /// Number of healthy tools
    pub healthy_tools: usize,
    /// Number of categories
    pub categories_count: usize,
    /// Total tool usage count
    pub total_usage: u64,
    /// Average success rate across all tools
    pub average_success_rate: f64,
}

/// Registry export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryExport {
    /// Tool metadata by name
    pub tools: HashMap<String, ToolMetadata>,
    /// Categories mapping
    pub categories: HashMap<String, Vec<String>>,
    /// Health status by tool name
    pub health: HashMap<String, ToolHealth>,
    /// Export timestamp
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}


