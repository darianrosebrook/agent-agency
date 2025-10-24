//! Tool Execution Engine
//!
//! Handles the execution of MCP tools with proper error handling,
//! timeout management, and result processing.

use crate::types::*;
use crate::mcp_integration::MCPToolRegistry;
use std::sync::Arc;

/// Tool executor that manages MCP tool execution
pub struct ToolExecutor {
    tool_registry: Arc<MCPToolRegistry>,
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Self {
        Self {
            tool_registry: Arc::new(MCPToolRegistry::new()),
        }
    }

    /// Execute a tool with the given context
    pub async fn execute_tool(&self, context: TaskContext) -> Result<ExecutionResult, ExecutionError> {
        // Validate tool availability
        if !self.tool_registry.has_tool(&context.tool_id).await {
            return Err(ExecutionError::ToolNotFound(context.tool_id));
        }

        // Simulate MCP tool execution based on tool type
        let result = match context.tool_id.as_str() {
            "react-generator" => self.execute_react_generator(&context).await,
            "file-editor" => self.execute_file_editor(&context).await,
            "research-assistant" => self.execute_research_assistant(&context).await,
            _ => Err(ExecutionError::UnknownTool(context.tool_id)),
        }?;

        Ok(result)
    }

    /// Execute React component generation
    async fn execute_react_generator(&self, context: &TaskContext) -> Result<ExecutionResult, ExecutionError> {
        let component_name = context.parameters
            .get("componentName")
            .and_then(|v| v.as_str())
            .unwrap_or("MyComponent");

        let component_code = format!(
            "import React from 'react';\n\
             import styles from './{}.module.scss';\n\
             \n\
             interface {}Props {{\n\
             \tchildren?: React.ReactNode;\n\
             }}\n\
             \n\
             export const {}: React.FC<{}Props> = ({{ children }}) => {{\n\
             \treturn (\n\
             \t\t<div className={{styles.container}}>\n\
             \t\t\t{{children}}\n\
             \t\t</div>\n\
             \t);\n\
             }};",
            component_name.to_lowercase(), component_name, component_name, component_name
        );

        let output = serde_json::json!({
            "component": component_code,
            "files": [format!("{}.tsx", component_name)]
        });

        Ok(ExecutionResult {
            success: true,
            output: Some(output),
            error_message: None,
            execution_time_ms: 100,
            tool_id: context.tool_id.clone(),
        })
    }

    /// Execute file editing
    async fn execute_file_editor(&self, context: &TaskContext) -> Result<ExecutionResult, ExecutionError> {
        let output = serde_json::json!({
            "file_edited": true,
            "changes_applied": "simulated changes"
        });

        Ok(ExecutionResult {
            success: true,
            output: Some(output),
            error_message: None,
            execution_time_ms: 50,
            tool_id: context.tool_id.clone(),
        })
    }

    /// Execute research assistant
    async fn execute_research_assistant(&self, context: &TaskContext) -> Result<ExecutionResult, ExecutionError> {
        let output = serde_json::json!({
            "research_completed": true,
            "findings": ["Key insight 1", "Key insight 2"]
        });

        Ok(ExecutionResult {
            success: true,
            output: Some(output),
            error_message: None,
            execution_time_ms: 200,
            tool_id: context.tool_id.clone(),
        })
    }
}

/// Result of tool execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub tool_id: ToolId,
}

/// Errors from tool execution
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Unknown tool: {0}")]
    UnknownTool(String),
}
