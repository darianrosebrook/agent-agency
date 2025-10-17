//! MCP integration types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub tool_type: ToolType,
    pub capabilities: Vec<ToolCapability>,
    pub parameters: ToolParameters,
    pub output_schema: serde_json::Value,
    pub caws_compliance: CawsComplianceStatus,
    pub registration_time: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub usage_count: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    /// Code generation tool
    CodeGeneration,
    /// Code analysis tool
    CodeAnalysis,
    /// Testing tool
    Testing,
    /// Documentation tool
    Documentation,
    /// Build tool
    Build,
    /// Deployment tool
    Deployment,
    /// Monitoring tool
    Monitoring,
    /// Utility tool
    Utility,
    /// Custom tool
    Custom(String),
}

/// Tool capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCapability {
    /// Can read files
    FileRead,
    /// Can write files
    FileWrite,
    /// Can execute commands
    CommandExecution,
    /// Can make network requests
    NetworkAccess,
    /// Can access databases
    DatabaseAccess,
    /// Can process images
    ImageProcessing,
    /// Can process text
    TextProcessing,
    /// Can generate code
    CodeGeneration,
    /// Can analyze code
    CodeAnalysis,
    /// Can run tests
    TestExecution,
    /// Can generate documentation
    DocumentationGeneration,
}

/// Tool parameters schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    pub required: Vec<ParameterDefinition>,
    pub optional: Vec<ParameterDefinition>,
    pub constraints: Vec<ParameterConstraint>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub parameter_type: ParameterType,
    pub description: String,
    pub default_value: Option<serde_json::Value>,
    pub validation_rules: Vec<ValidationRule>,
}

/// Parameter types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    File,
    Directory,
    URL,
    JSON,
}

/// Parameter constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraint {
    pub parameter_name: String,
    pub constraint_type: ConstraintType,
    pub value: serde_json::Value,
    pub message: Option<String>,
}

/// Constraint types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    MinLength,
    MaxLength,
    MinValue,
    MaxValue,
    Pattern,
    Required,
    Unique,
    Custom(String),
}

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub error_message: String,
}

/// Validation rule types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    NotEmpty,
    RegexMatch,
    RangeCheck,
    TypeCheck,
    Custom(String),
}

/// CAWS compliance status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CawsComplianceStatus {
    /// Tool is CAWS compliant
    Compliant,
    /// Tool has minor violations
    MinorViolations(Vec<CawsViolation>),
    /// Tool has major violations
    MajorViolations(Vec<CawsViolation>),
    /// Tool compliance is unknown
    Unknown,
    /// Tool is not CAWS compliant
    NonCompliant(Vec<CawsViolation>),
}

/// CAWS violation details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CawsViolation {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub suggestion: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub file_path: Option<String>,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRequest {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: Option<ExecutionContext>,
    pub priority: ExecutionPriority,
    pub timeout_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub requested_by: Option<String>,
}

/// Execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub working_directory: Option<String>,
    pub environment_variables: HashMap<String, String>,
    pub input_files: Vec<String>,
    pub output_directory: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExecutionPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub request_id: Uuid,
    pub tool_id: Uuid,
    pub status: ExecutionStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub logs: Vec<LogEntry>,
    pub performance_metrics: PerformanceMetrics,
    pub caws_compliance_result: Option<CawsComplianceResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

/// Execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Timeout,
    Cancelled,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Log levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub disk_io_bytes: u64,
    pub network_io_bytes: u64,
    pub execution_time_ms: u64,
    pub queue_time_ms: u64,
}

/// CAWS compliance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsComplianceResult {
    pub is_compliant: bool,
    pub violations: Vec<CawsViolation>,
    pub compliance_score: f32,
    pub checked_at: DateTime<Utc>,
    pub rulebook_version: String,
}

/// Tool manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tool_type: ToolType,
    pub entry_point: String,
    pub dependencies: Vec<Dependency>,
    pub capabilities: Vec<ToolCapability>,
    pub parameters: ToolParameters,
    pub output_schema: serde_json::Value,
    pub caws_compliance: Option<CawsComplianceConfig>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub dependency_type: DependencyType,
    pub optional: bool,
}

/// Dependency types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    Runtime,
    Build,
    Development,
    Test,
}

/// CAWS compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsComplianceConfig {
    pub required_rules: Vec<String>,
    pub optional_rules: Vec<String>,
    pub strict_mode: bool,
    pub custom_validations: Vec<CustomValidation>,
}

/// Custom validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomValidation {
    pub name: String,
    pub description: String,
    pub validation_function: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// MCP server status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MCPServerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

/// MCP connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConnection {
    pub id: Uuid,
    pub client_id: Option<String>,
    pub connection_type: ConnectionType,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: ConnectionStatus,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Connection types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    WebSocket,
    HTTP,
    UnixSocket,
}

/// Connection status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
}

/// Tool discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDiscoveryResult {
    pub discovered_tools: Vec<MCPTool>,
    pub errors: Vec<DiscoveryError>,
    pub discovery_time_ms: u64,
    pub discovered_at: DateTime<Utc>,
}

/// Discovery error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryError {
    pub path: String,
    pub error_type: DiscoveryErrorType,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Discovery error types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryErrorType {
    FileNotFound,
    InvalidManifest,
    ParseError,
    ValidationError,
    PermissionError,
    NetworkError,
    Unknown,
}

/// Tool registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistryStats {
    pub total_tools: u64,
    pub active_tools: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub most_used_tools: Vec<ToolUsageStats>,
    pub last_updated: DateTime<Utc>,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub tool_id: Uuid,
    pub tool_name: String,
    pub usage_count: u64,
    pub success_rate: f32,
    pub average_execution_time_ms: f64,
    pub last_used: Option<DateTime<Utc>>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub server: ServerConfig,
    pub tool_discovery: ToolDiscoveryConfig,
    pub caws_integration: CawsIntegrationConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server_name: String,
    pub version: String,
    pub host: String,
    pub port: u16,
    pub enable_tls: bool,
    pub enable_http: bool,
    pub enable_websocket: bool,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
    pub enable_compression: bool,
    pub log_level: String,
    pub auth_api_key: Option<String>,
    pub requests_per_minute: Option<u32>,
}

/// Tool discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDiscoveryConfig {
    pub enable_auto_discovery: bool,
    pub discovery_paths: Vec<String>,
    pub manifest_patterns: Vec<String>,
    pub discovery_interval_seconds: u32,
    pub enable_validation: bool,
}

/// CAWS integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsIntegrationConfig {
    pub enable_caws_checking: bool,
    pub caws_rulebook_path: String,
    pub enable_provenance: bool,
    pub enable_quality_gates: bool,
    pub validation_strictness: ValidationStrictness,
}

/// Validation strictness levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStrictness {
    /// Strict validation - fail on any violation
    Strict,
    /// Moderate validation - warn on minor violations
    Moderate,
    /// Lenient validation - log violations but allow execution
    Lenient,
}
