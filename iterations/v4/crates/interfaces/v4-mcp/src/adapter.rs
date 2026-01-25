//! Tool Adapter
//!
//! Converts between v4-tools types and MCP protocol types.

use std::sync::Arc;

use v4_tools::{ToolCapability, ToolCategory, ToolMetadata, ToolRegistry};
use v4_types::operators::{OperatorResult, SeekOp};
use v4_types::OperatorType;

use crate::error::MCPError;
use crate::types::{CallToolParams, CallToolResult, MCPTool, ToolContent};

/// Adapts v4-tools to MCP protocol
pub struct ToolAdapter {
    registry: Arc<ToolRegistry>,
}

impl ToolAdapter {
    /// Create a new adapter
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    /// Convert a v4 tool to MCP tool definition
    pub fn to_mcp_tool(metadata: &ToolMetadata) -> MCPTool {
        MCPTool {
            name: metadata.id.clone(),
            description: Some(metadata.description.clone()),
            input_schema: Self::build_input_schema(metadata),
        }
    }

    /// Build JSON Schema for tool input based on tool metadata
    fn build_input_schema(metadata: &ToolMetadata) -> serde_json::Value {
        // Build schema based on tool category and capabilities
        match metadata.category {
            ToolCategory::FileSystem => Self::file_system_schema(&metadata.id),
            ToolCategory::CodeAnalysis => Self::code_analysis_schema(&metadata.id),
            ToolCategory::Memory => Self::memory_schema(&metadata.id),
            ToolCategory::Web => Self::web_schema(&metadata.id),
            ToolCategory::Git => Self::git_schema(&metadata.id),
            ToolCategory::Control => Self::control_schema(&metadata.id),
            ToolCategory::Validation => Self::validation_schema(&metadata.id),
        }
    }

    fn file_system_schema(tool_id: &str) -> serde_json::Value {
        match tool_id {
            "file_read" => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
            "directory_list" => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list"
                    }
                },
                "required": ["path"]
            }),
            _ => Self::generic_schema(),
        }
    }

    fn code_analysis_schema(tool_id: &str) -> serde_json::Value {
        match tool_id {
            "code_search" => serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (regex)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    }
                },
                "required": ["pattern"]
            }),
            _ => Self::generic_schema(),
        }
    }

    fn memory_schema(tool_id: &str) -> serde_json::Value {
        match tool_id {
            "memory_query" => serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Query string"
                    },
                    "max_hops": {
                        "type": "integer",
                        "description": "Maximum graph hops",
                        "minimum": 1,
                        "default": 3
                    }
                },
                "required": ["query"]
            }),
            _ => Self::generic_schema(),
        }
    }

    fn web_schema(tool_id: &str) -> serde_json::Value {
        match tool_id {
            "web_search" => serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
            "web_fetch" => serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to fetch"
                    }
                },
                "required": ["url"]
            }),
            _ => Self::generic_schema(),
        }
    }

    fn git_schema(tool_id: &str) -> serde_json::Value {
        match tool_id {
            "git_status" => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository path (optional)"
                    }
                }
            }),
            _ => Self::generic_schema(),
        }
    }

    fn control_schema(_tool_id: &str) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Control action to perform"
                }
            },
            "required": ["action"]
        })
    }

    fn validation_schema(_tool_id: &str) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "schema_id": {
                    "type": "string",
                    "description": "Schema ID to validate against"
                },
                "data": {
                    "type": "string",
                    "description": "Data to validate"
                }
            },
            "required": ["schema_id", "data"]
        })
    }

    fn generic_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        })
    }

    /// List all available tools as MCP tools
    pub fn list_tools(&self) -> Vec<MCPTool> {
        self.registry
            .list()
            .iter()
            .map(Self::to_mcp_tool)
            .collect()
    }

    /// Convert MCP call params to v4 operator type
    pub fn to_operator(&self, params: &CallToolParams) -> Result<OperatorType, MCPError> {
        // Get tool metadata to determine how to construct the operator
        let metadata = self
            .registry
            .get_metadata(&params.name)
            .ok_or_else(|| MCPError::ToolNotFound(params.name.clone()))?;

        // Build operator based on tool category
        match metadata.category {
            ToolCategory::FileSystem => self.file_system_operator(params),
            ToolCategory::CodeAnalysis => self.code_analysis_operator(params),
            ToolCategory::Memory => self.memory_operator(params),
            ToolCategory::Web => self.web_operator(params),
            ToolCategory::Git => self.git_operator(params),
            _ => Err(MCPError::UnsupportedTool(params.name.clone())),
        }
    }

    fn file_system_operator(&self, params: &CallToolParams) -> Result<OperatorType, MCPError> {
        match params.name.as_str() {
            "file_read" => {
                let path = params
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MCPError::MissingParameter("path".to_string()))?
                    .to_string();

                Ok(OperatorType::Seek(SeekOp::ReadFile { path }))
            }
            "directory_list" => {
                let path = params
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MCPError::MissingParameter("path".to_string()))?
                    .to_string();

                Ok(OperatorType::Seek(SeekOp::ListDirectory { path }))
            }
            _ => Err(MCPError::UnsupportedTool(params.name.clone())),
        }
    }

    fn code_analysis_operator(&self, params: &CallToolParams) -> Result<OperatorType, MCPError> {
        match params.name.as_str() {
            "code_search" => {
                let pattern = params
                    .arguments
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MCPError::MissingParameter("pattern".to_string()))?
                    .to_string();

                let path = params
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(OperatorType::Seek(SeekOp::SearchCode { pattern, path }))
            }
            _ => Err(MCPError::UnsupportedTool(params.name.clone())),
        }
    }

    fn memory_operator(&self, params: &CallToolParams) -> Result<OperatorType, MCPError> {
        match params.name.as_str() {
            "memory_query" => {
                let query = params
                    .arguments
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MCPError::MissingParameter("query".to_string()))?
                    .to_string();

                let max_hops = params
                    .arguments
                    .get("max_hops")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .unwrap_or(3);

                Ok(OperatorType::Seek(SeekOp::QueryMemory { query, max_hops }))
            }
            _ => Err(MCPError::UnsupportedTool(params.name.clone())),
        }
    }

    fn web_operator(&self, params: &CallToolParams) -> Result<OperatorType, MCPError> {
        match params.name.as_str() {
            "web_search" => {
                let query = params
                    .arguments
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MCPError::MissingParameter("query".to_string()))?
                    .to_string();

                Ok(OperatorType::Seek(SeekOp::WebSearch { query }))
            }
            "web_fetch" => {
                let url = params
                    .arguments
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MCPError::MissingParameter("url".to_string()))?
                    .to_string();

                Ok(OperatorType::Seek(SeekOp::WebFetch { url }))
            }
            _ => Err(MCPError::UnsupportedTool(params.name.clone())),
        }
    }

    fn git_operator(&self, params: &CallToolParams) -> Result<OperatorType, MCPError> {
        match params.name.as_str() {
            "git_status" => {
                let path = params
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(OperatorType::Seek(SeekOp::GitStatus { path }))
            }
            _ => Err(MCPError::UnsupportedTool(params.name.clone())),
        }
    }

    /// Convert v4 operator result to MCP call result
    pub fn to_call_result(result: &OperatorResult) -> CallToolResult {
        if result.success {
            let output = result
                .data
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Success".to_string());

            CallToolResult {
                content: vec![ToolContent::text(output)],
                is_error: false,
            }
        } else {
            let error = result
                .error
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error".to_string());

            CallToolResult {
                content: vec![ToolContent::error(error)],
                is_error: true,
            }
        }
    }

    /// Execute a tool through MCP
    pub async fn execute(&self, params: CallToolParams) -> Result<CallToolResult, MCPError> {
        // Get the tool
        let tool = self
            .registry
            .get(&params.name)
            .ok_or_else(|| MCPError::ToolNotFound(params.name.clone()))?;

        // Convert to operator
        let operator = self.to_operator(&params)?;

        // Execute
        let result = tool
            .execute(&operator)
            .await
            .map_err(|e| MCPError::ExecutionFailed(e.to_string()))?;

        Ok(Self::to_call_result(&result))
    }

    /// Get tool capabilities for governance checks
    pub fn get_capabilities(&self, tool_name: &str) -> Option<Vec<ToolCapability>> {
        self.registry
            .get_metadata(tool_name)
            .map(|m| m.capabilities)
    }

    /// Check if a tool requires sandbox execution
    pub fn requires_sandbox(&self, tool_name: &str) -> bool {
        self.registry
            .get_metadata(tool_name)
            .map(|m| m.requires_sandbox)
            .unwrap_or(true) // Default to requiring sandbox for unknown tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use v4_tools::{Tool, ToolError};

    // Mock tool for testing
    struct MockFileTool;

    #[async_trait]
    impl Tool for MockFileTool {
        fn id(&self) -> &str {
            "file_read"
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                id: "file_read".to_string(),
                name: "File Read".to_string(),
                description: "Read contents of a file".to_string(),
                version: "1.0.0".to_string(),
                category: ToolCategory::FileSystem,
                capabilities: vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess],
                supported_operators: vec!["S".to_string()],
                requires_sandbox: false,
            }
        }

        async fn execute(&self, _operator: &OperatorType) -> Result<OperatorResult, ToolError> {
            Ok(OperatorResult {
                operator: OperatorType::Seek(SeekOp::ReadFile {
                    path: "test.txt".to_string(),
                }),
                success: true,
                data: Some(serde_json::json!("file contents")),
                error: None,
                execution_time_ms: 10,
                content_hash: Some("abc123".to_string()),
            })
        }

        fn can_execute(&self, operator: &OperatorType) -> bool {
            matches!(operator, OperatorType::Seek(SeekOp::ReadFile { .. }))
        }
    }

    #[test]
    fn test_to_mcp_tool() {
        let metadata = ToolMetadata {
            id: "test_tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::ReadOnly],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        };

        let mcp_tool = ToolAdapter::to_mcp_tool(&metadata);
        assert_eq!(mcp_tool.name, "test_tool");
        assert_eq!(mcp_tool.description, Some("A test tool".to_string()));
    }

    #[test]
    fn test_list_tools() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockFileTool));

        let adapter = ToolAdapter::new(registry);
        let tools = adapter.list_tools();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "file_read");
    }

    #[test]
    fn test_to_operator_file_read() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockFileTool));

        let adapter = ToolAdapter::new(registry);

        let params = CallToolParams {
            name: "file_read".to_string(),
            arguments: {
                let mut map = HashMap::new();
                map.insert("path".to_string(), serde_json::json!("/tmp/test.txt"));
                map
            },
        };

        let operator = adapter.to_operator(&params).unwrap();
        match operator {
            OperatorType::Seek(SeekOp::ReadFile { path }) => {
                assert_eq!(path, "/tmp/test.txt");
            }
            _ => panic!("Expected ReadFile operator"),
        }
    }

    #[test]
    fn test_to_operator_missing_param() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockFileTool));

        let adapter = ToolAdapter::new(registry);

        let params = CallToolParams {
            name: "file_read".to_string(),
            arguments: HashMap::new(), // Missing "path"
        };

        let result = adapter.to_operator(&params);
        assert!(matches!(result, Err(MCPError::MissingParameter(_))));
    }

    #[test]
    fn test_to_call_result_success() {
        let result = OperatorResult {
            operator: OperatorType::Seek(SeekOp::ReadFile {
                path: "test.txt".to_string(),
            }),
            success: true,
            data: Some(serde_json::json!("hello")),
            error: None,
            execution_time_ms: 5,
            content_hash: Some("abc".to_string()),
        };

        let call_result = ToolAdapter::to_call_result(&result);
        assert!(!call_result.is_error);
        assert_eq!(call_result.content.len(), 1);
    }

    #[test]
    fn test_to_call_result_failure() {
        let result = OperatorResult {
            operator: OperatorType::Seek(SeekOp::ReadFile {
                path: "test.txt".to_string(),
            }),
            success: false,
            data: None,
            error: Some("something went wrong".to_string()),
            execution_time_ms: 5,
            content_hash: None,
        };

        let call_result = ToolAdapter::to_call_result(&result);
        assert!(call_result.is_error);
    }

    #[tokio::test]
    async fn test_execute_tool() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockFileTool));

        let adapter = ToolAdapter::new(registry);

        let params = CallToolParams {
            name: "file_read".to_string(),
            arguments: {
                let mut map = HashMap::new();
                map.insert("path".to_string(), serde_json::json!("/tmp/test.txt"));
                map
            },
        };

        let result = adapter.execute(params).await.unwrap();
        assert!(!result.is_error);
    }

    #[test]
    fn test_requires_sandbox() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockFileTool));

        let adapter = ToolAdapter::new(registry);

        // Known tool
        assert!(!adapter.requires_sandbox("file_read"));

        // Unknown tool defaults to requiring sandbox
        assert!(adapter.requires_sandbox("unknown_tool"));
    }
}
