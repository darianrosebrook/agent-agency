//! MCP Integration for Workers
//!
//! Provides the bridge between workers and the agent-mcp crate's tool registry.
//! Services register their capabilities as MCP tools that workers can discover and use.

use agent_mcp::{ToolRegistry, MCPTool, ToolExecutionRequest, ToolExecutionResult, types::{ToolType, ToolCapability, ToolParameters, ParameterDefinition, ParameterConstraint, ToolManifest, CawsComplianceConfig, CawsComplianceStatus}};
use std::sync::Arc;

/// MCP integration layer for workers
pub struct MCPIntegration {
    tool_registry: Arc<ToolRegistry>,
}

impl MCPIntegration {
    /// Create new MCP integration with a tool registry
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self { tool_registry }
    }

    /// Get access to the underlying tool registry
    pub fn registry(&self) -> Arc<ToolRegistry> {
        Arc::clone(&self.tool_registry)
    }

    /// Register a service's tools with the MCP registry
    pub async fn register_service_tools(&self, service_name: &str, tools: Vec<MCPTool>) -> Result<(), MCPIntegrationError> {
        for tool in tools {
            self.tool_registry.register_tool(tool)
                .await
                .map_err(|e| MCPIntegrationError::ToolRegistrationFailed(format!("{}: {}", service_name, e)))?;
        }
        Ok(())
    }

    /// Execute a tool using the MCP registry
    pub async fn execute_tool(&self, request: ToolExecutionRequest) -> Result<ToolExecutionResult, MCPIntegrationError> {
        self.tool_registry.execute_tool(request)
            .await
            .map_err(|e| MCPIntegrationError::ExecutionFailed(e.to_string()))
    }

    /// Get all available tools
    pub async fn list_tools(&self) -> Vec<MCPTool> {
        self.tool_registry.get_all_tools().await
    }

    /// Check if a tool is available
    pub async fn has_tool(&self, tool_id: uuid::Uuid) -> bool {
        self.tool_registry.get_tool(tool_id).await.is_some()
    }
}

/// Errors from MCP integration
#[derive(Debug, thiserror::Error)]
pub enum MCPIntegrationError {
    #[error("Tool registration failed: {0}")]
    ToolRegistrationFailed(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Service not available: {0}")]
    ServiceUnavailable(String),
}

/// Helper function to create a tool definition with proper schema
pub fn create_tool_definition(
    name: &str,
    description: &str,
    tool_type: ToolType,
    capabilities: Vec<ToolCapability>,
    required_params: Vec<ParameterDefinition>,
    optional_params: Vec<ParameterDefinition>,
) -> MCPTool {
    use chrono::Utc;
    use std::collections::HashMap;

    MCPTool {
        id: uuid::Uuid::new_v4(),
        name: name.to_string(),
        description: description.to_string(),
        version: "1.0.0".to_string(),
        author: "agent-agency".to_string(),
        tool_type,
        capabilities,
        parameters: ToolParameters {
            required: required_params,
            optional: optional_params,
            constraints: vec![], // Can be extended later
        },
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {"type": "object"},
                "execution_time_ms": {"type": "number"}
            }
        }),
        endpoint: format!("/tools/{}", name),
        manifest: ToolManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: description.to_string(),
            author: "Agent Workers".to_string(),
            tool_type: ToolType::Utility,
            entry_point: name.to_string(),
            dependencies: vec![],
            capabilities: vec![ToolCapability::DatabaseAccess],
            parameters: ToolParameters {
                required: vec![],
                optional: vec![],
                constraints: vec![],
            },
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "result": {"type": "object"},
                    "execution_time_ms": {"type": "number"}
                }
            }),
            endpoint: Some(format!("/tools/{}", name)),
            caws_compliance: Some(CawsComplianceConfig {
                required_rules: vec!["memory_access".to_string()],
                optional_rules: vec![],
                strict_mode: false,
                custom_validations: vec![],
            }),
            metadata: HashMap::new(),
            configuration_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        caws_compliance: CawsComplianceStatus::Compliant,
        registration_time: Utc::now(),
        last_updated: Utc::now(),
        usage_count: 0,
        metadata: HashMap::new(),
    }
}

/// Helper to create a parameter definition
pub fn create_parameter(
    name: &str,
    description: &str,
    param_type: &str,
    required: bool,
    default_value: Option<serde_json::Value>,
) -> ParameterDefinition {
    use agent_mcp::types::ParameterType;

    let parameter_type = match param_type {
        "string" => ParameterType::String,
        "integer" => ParameterType::Integer,
        "float" => ParameterType::Float,
        "boolean" => ParameterType::Boolean,
        _ => ParameterType::String,
    };

    ParameterDefinition {
        name: name.to_string(),
        parameter_type,
        description: description.to_string(),
        default_value,
        validation_rules: vec![], // Can be extended
    }
}
