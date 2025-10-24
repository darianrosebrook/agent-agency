//! MCP Integration for Workers
//!
//! Provides the bridge between workers and MCP tools, enabling
//! tool discovery, registration, and execution.

use crate::types::*;
use agent_mcp::{MCPServer, ToolExecutionRequest, ToolExecutionResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Registry for MCP tools available to workers
pub struct MCPToolRegistry {
    tools: Arc<RwLock<HashMap<ToolId, ToolMetadata>>>,
    mcp_server: Option<Arc<MCPServer>>,
}

impl MCPToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            mcp_server: None,
        }
    }

    /// Create a registry with an MCP server connection
    pub fn with_server(mcp_server: Arc<MCPServer>) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            mcp_server: Some(mcp_server),
        }
    }

    /// Register a tool in the registry
    pub async fn register_tool(&self, metadata: ToolMetadata) {
        let mut tools = self.tools.write().await;
        tools.insert(metadata.id.clone(), metadata.clone());
        info!("Registered MCP tool: {} ({})", metadata.name, metadata.id);
    }

    /// Check if a tool is available
    pub async fn has_tool(&self, tool_id: &ToolId) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(tool_id)
    }

    /// Get tool metadata
    pub async fn get_tool(&self, tool_id: &ToolId) -> Option<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.get(tool_id).cloned()
    }

    /// Discover tools from MCP server
    pub async fn discover_tools(&self) -> Result<Vec<ToolMetadata>, MCPIntegrationError> {
        if let Some(server) = &self.mcp_server {
            // In a real implementation, this would query the MCP server
            // For now, return mock tools
            Ok(vec![
                ToolMetadata {
                    id: "react-generator".to_string(),
                    name: "React Component Generator".to_string(),
                    description: "Generates React components with TypeScript and SCSS modules".to_string(),
                    version: "1.0.0".to_string(),
                    capabilities: vec!["react".to_string(), "typescript".to_string(), "scss".to_string()],
                    parameters: HashMap::new(),
                },
                ToolMetadata {
                    id: "file-editor".to_string(),
                    name: "File Editor".to_string(),
                    description: "Edits files with context-aware changes".to_string(),
                    version: "1.0.0".to_string(),
                    capabilities: vec!["file-editing".to_string(), "context-aware".to_string()],
                    parameters: HashMap::new(),
                },
                ToolMetadata {
                    id: "research-assistant".to_string(),
                    name: "Research Assistant".to_string(),
                    description: "Gathers and synthesizes research information".to_string(),
                    version: "1.0.0".to_string(),
                    capabilities: vec!["research".to_string(), "synthesis".to_string()],
                    parameters: HashMap::new(),
                },
            ])
        } else {
            Err(MCPIntegrationError::NoMCPServer)
        }
    }

    /// Get all available tools
    pub async fn get_available_tools(&self) -> Vec<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    /// Get tools by capability
    pub async fn get_tools_by_capability(&self, capability: &str) -> Vec<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.values()
            .filter(|tool| tool.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }
}

/// Tool capabilities and requirements
#[derive(Debug, Clone)]
pub struct ToolCapabilities {
    pub supported_tasks: Vec<String>,
    pub input_formats: Vec<String>,
    pub output_formats: Vec<String>,
    pub performance_characteristics: ToolPerformance,
}

/// Performance characteristics of a tool
#[derive(Debug, Clone)]
pub struct ToolPerformance {
    pub average_execution_time_ms: f64,
    pub success_rate: f64,
    pub resource_usage: ResourceUsage,
}

/// Resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub network_bytes: u64,
}

/// Errors from MCP integration
#[derive(Debug, thiserror::Error)]
pub enum MCPIntegrationError {
    #[error("No MCP server configured")]
    NoMCPServer,

    #[error("Tool discovery failed: {0}")]
    ToolDiscoveryFailed(String),

    #[error("Tool registration failed: {0}")]
    ToolRegistrationFailed(String),

    #[error("MCP protocol error: {0}")]
    MCPProtocolError(String),
}
