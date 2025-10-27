//! Prompting types for MCP integration
//!
//! This module contains types and structures used for prompting and MCP tool interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub inputs: Vec<ToolInput>,
    pub outputs: Vec<ToolOutput>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool input specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub input_type: InputType,
    pub default_value: Option<serde_json::Value>,
}

/// Tool output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub name: String,
    pub description: String,
    pub output_type: OutputType,
}

/// Input type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    File,
    Directory,
}

/// Output type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    File,
    Directory,
    Error,
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRequest {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: Option<ExecutionContext>,
    pub timeout_ms: Option<u64>,
}

/// Execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDiscoveryResult {
    pub tools: Vec<MCPTool>,
    pub discovery_time_ms: u64,
    pub errors: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistryStats {
    pub total_tools: u32,
    pub active_tools: u32,
    pub failed_tools: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_execution_time_ms: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub total_executions: u64,
}

/// CAWS compliance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsComplianceResult {
    pub compliant: bool,
    pub violations: Vec<ComplianceViolation>,
    pub score: f64,
    pub recommendations: Vec<String>,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
}

/// Violation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// MCP connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConnection {
    pub connection_id: String,
    pub client_id: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub connection_type: ConnectionType,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Connection type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    WebSocket,
    HTTP,
    GRPC,
}

/// MCP server status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerStatus {
    pub server_name: String,
    pub version: String,
    pub status: ServerStatus,
    pub uptime_seconds: u64,
    pub active_connections: u32,
    pub total_requests: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub performance_metrics: PerformanceMetrics,
}

/// Server status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Running,
    Starting,
    Stopping,
    Stopped,
    Error,
}

impl Default for ToolExecutionResult {
    fn default() -> Self {
        Self {
            success: false,
            output: None,
            error: None,
            execution_time_ms: 0,
            metadata: HashMap::new(),
        }
    }
}

impl Default for ToolRegistryStats {
    fn default() -> Self {
        Self {
            total_tools: 0,
            active_tools: 0,
            failed_tools: 0,
            last_updated: chrono::Utc::now(),
            performance_metrics: PerformanceMetrics {
                average_execution_time_ms: 0.0,
                success_rate: 0.0,
                error_rate: 0.0,
                total_executions: 0,
            },
        }
    }
}

impl Default for CawsComplianceResult {
    fn default() -> Self {
        Self {
            compliant: true,
            violations: Vec::new(),
            score: 1.0,
            recommendations: Vec::new(),
        }
    }
}

impl Default for MCPServerStatus {
    fn default() -> Self {
        Self {
            server_name: "agent-agency-mcp".to_string(),
            version: "0.1.0".to_string(),
            status: ServerStatus::Stopped,
            uptime_seconds: 0,
            active_connections: 0,
            total_requests: 0,
            error_count: 0,
            last_error: None,
            performance_metrics: PerformanceMetrics {
                average_execution_time_ms: 0.0,
                success_rate: 0.0,
                error_rate: 0.0,
                total_executions: 0,
            },
        }
    }
}
