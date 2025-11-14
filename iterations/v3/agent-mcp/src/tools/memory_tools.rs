//! Memory System MCP Tools
//!
//! Provides MCP tools for memory operations including search, store, and retrieve.
//! These tools allow agents to interact with their memory system through MCP.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::mcp_types::*;

/// Create all memory-related MCP tools
pub fn create_memory_tools() -> Vec<MCPTool> {
    vec![
        create_memory_search_tool(),
        create_memory_store_tool(),
        create_memory_retrieve_tool(),
    ]
}

/// Create memory search tool
pub fn create_memory_search_tool() -> MCPTool {
    MCPTool {
        id: Uuid::new_v4(),
        name: "memory_search".to_string(),
        description: "Search the agent's memory system for relevant experiences and knowledge"
            .to_string(),
        version: "1.0.0".to_string(),
        author: "Agent Agency".to_string(),
        tool_type: ToolType::Utility,
        capabilities: vec![ToolCapability::DatabaseAccess],
        parameters: ToolParameters {
            required: vec![ParameterDefinition {
                name: "query".to_string(),
                parameter_type: ParameterType::String,
                description: "Search query to find relevant memories".to_string(),
                default_value: None,
                validation_rules: vec![],
            }],
            optional: vec![ParameterDefinition {
                name: "limit".to_string(),
                parameter_type: ParameterType::Integer,
                description: "Maximum number of results to return".to_string(),
                default_value: Some(serde_json::Value::Number(10.into())),
                validation_rules: vec![],
            }],
            constraints: vec![],
        },
        output_schema: serde_json::json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "memory_id": {"type": "string"},
                    "content": {"type": "string"},
                    "relevance_score": {"type": "number"}
                }
            }
        }),
        manifest: ToolManifest {
            name: "memory_search".to_string(),
            version: "1.0.0".to_string(),
            description: "Memory search tool".to_string(),
            author: "Agent Agency".to_string(),
            tool_type: ToolType::Utility,
            entry_point: "memory_search".to_string(),
            dependencies: vec![],
            capabilities: vec![ToolCapability::DatabaseAccess],
            parameters: ToolParameters {
                required: vec![],
                optional: vec![],
                constraints: vec![],
            },
            output_schema: serde_json::json!({
                "type": "object"
            }),
            endpoint: Some("/tools/memory_search".to_string()),
            caws_compliance: None,
            metadata: std::collections::HashMap::new(),
            configuration_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "default": 10}
                },
                "required": ["query"]
            }),
        },
        caws_compliance: CawsComplianceStatus::Compliant,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: std::collections::HashMap::new(),
        endpoint: "/tools/memory_search".to_string(),
    }
}

/// Create memory store tool
pub fn create_memory_store_tool() -> MCPTool {
    MCPTool {
        id: Uuid::new_v4(),
        name: "memory_store".to_string(),
        description: "Store a new experience in the agent's memory system".to_string(),
        version: "1.0.0".to_string(),
        author: "Agent Agency".to_string(),
        tool_type: ToolType::Utility,
        capabilities: vec![ToolCapability::DatabaseAccess],
        parameters: ToolParameters {
            required: vec![
                ParameterDefinition {
                    name: "agent_id".to_string(),
                    parameter_type: ParameterType::String,
                    description: "ID of the agent that had this experience".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "task_type".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Type/category of the task".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "input".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Input/context that led to this experience".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "output".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Output/result of the experience".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "outcome".to_string(),
                    parameter_type: ParameterType::JSON,
                    description: "Outcome metadata (success, performance, etc.)".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "memory_type".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Type of memory ('episodic', 'semantic', 'procedural', 'working')"
                        .to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
            ],
            optional: vec![],
            constraints: vec![],
        },
        manifest: ToolManifest {
            name: "memory_store".to_string(),
            version: "1.0.0".to_string(),
            description: "Memory store tool".to_string(),
            author: "Agent Agency".to_string(),
            tool_type: ToolType::Utility,
            entry_point: "memory_store".to_string(),
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
                    "memory_id": {"type": "string"},
                    "stored_at": {"type": "string"}
                }
            }),
            endpoint: Some("/tools/memory_store".to_string()),
            caws_compliance: None,
            metadata: std::collections::HashMap::new(),
            configuration_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string"},
                    "task_type": {"type": "string"},
                    "input": {"type": "string"},
                    "output": {"type": "string"},
                    "outcome": {"type": "object"},
                    "memory_type": {"type": "string", "enum": ["episodic", "semantic", "procedural", "working"]}
                },
                "required": ["agent_id", "task_type", "input", "output", "outcome", "memory_type"]
            }),
        },
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "memory_id": {"type": "string"},
                "stored_at": {"type": "string"}
            }
        }),
        endpoint: "/tools/memory_store".to_string(),
        caws_compliance: CawsComplianceStatus::Compliant,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert(
                "category".to_string(),
                serde_json::Value::String("memory".to_string()),
            );
            map.insert(
                "tags".to_string(),
                serde_json::json!(["store", "experience", "learning"]),
            );
            map
        },
    }
}

/// Create memory retrieve tool
pub fn create_memory_retrieve_tool() -> MCPTool {
    MCPTool {
        id: Uuid::new_v4(),
        name: "memory_retrieve".to_string(),
        description: "Retrieve a specific memory by ID".to_string(),
        version: "1.0.0".to_string(),
        author: "Agent Agency".to_string(),
        tool_type: ToolType::Utility,
        capabilities: vec![ToolCapability::DatabaseAccess],
        parameters: ToolParameters {
            required: vec![ParameterDefinition {
                name: "memory_id".to_string(),
                parameter_type: ParameterType::String,
                description: "UUID of the memory to retrieve".to_string(),
                default_value: None,
                validation_rules: vec![],
            }],
            optional: vec![],
            constraints: vec![],
        },
        manifest: ToolManifest {
            name: "memory_retrieve".to_string(),
            version: "1.0.0".to_string(),
            description: "Memory retrieve tool".to_string(),
            author: "Agent Agency".to_string(),
            tool_type: ToolType::Utility,
            entry_point: "memory_retrieve".to_string(),
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
                    "memory_id": {"type": "string"},
                    "content": {"type": "string"}
                }
            }),
            endpoint: Some("/tools/memory_retrieve".to_string()),
            caws_compliance: None,
            metadata: std::collections::HashMap::new(),
            configuration_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "memory_id": {"type": "string", "format": "uuid"}
                },
                "required": ["memory_id"]
            }),
        },
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "memory_id": {"type": "string"},
                "content": {"type": "string"}
            }
        }),
        endpoint: "/tools/memory_retrieve".to_string(),
        caws_compliance: CawsComplianceStatus::Compliant,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert(
                "category".to_string(),
                serde_json::Value::String("memory".to_string()),
            );
            map.insert(
                "tags".to_string(),
                serde_json::json!(["retrieve", "lookup", "recall"]),
            );
            map
        },
    }
}
