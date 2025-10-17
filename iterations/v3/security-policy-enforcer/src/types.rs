use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityPolicyConfig {
    /// File access policies
    pub file_access: FileAccessPolicy,
    /// Command execution policies
    pub command_execution: CommandExecutionPolicy,
    /// Secrets detection policies
    pub secrets_detection: SecretsDetectionPolicy,
    /// Audit and logging configuration
    pub audit: AuditPolicy,
    /// Integration with council for security decisions
    pub council_integration: CouncilIntegrationConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingPolicy,
}

/// File access control policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileAccessPolicy {
    /// Allowed file patterns (glob patterns)
    pub allowed_patterns: Vec<String>,
    /// Denied file patterns (glob patterns)
    pub denied_patterns: Vec<String>,
    /// Sensitive file patterns that require special handling
    pub sensitive_patterns: Vec<String>,
    /// Maximum file size for operations (bytes)
    pub max_file_size: u64,
    /// Whether to allow symbolic links
    pub allow_symlinks: bool,
    /// Whether to allow hidden files
    pub allow_hidden_files: bool,
    /// Whether to allow files outside workspace
    pub allow_outside_workspace: bool,
}

/// Command execution control policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandExecutionPolicy {
    /// Allowed command patterns
    pub allowed_commands: Vec<String>,
    /// Denied command patterns
    pub denied_commands: Vec<String>,
    /// Dangerous command patterns that require approval
    pub dangerous_commands: Vec<String>,
    /// Maximum command execution time (seconds)
    pub max_execution_time: u64,
    /// Whether to allow network access
    pub allow_network_access: bool,
    /// Whether to allow file system modifications
    pub allow_file_modifications: bool,
    /// Whether to allow process spawning
    pub allow_process_spawning: bool,
}

/// Secrets detection policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretsDetectionPolicy {
    /// Enable secrets detection
    pub enabled: bool,
    /// Patterns for detecting secrets
    pub secret_patterns: Vec<SecretPattern>,
    /// Whether to block operations containing secrets
    pub block_on_secrets: bool,
    /// Whether to log secret detections
    pub log_secret_detections: bool,
    /// Whether to redact secrets in logs
    pub redact_secrets_in_logs: bool,
}

/// Secret detection pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretPattern {
    /// Pattern name
    pub name: String,
    /// Regex pattern for detection
    pub pattern: String,
    /// Severity level
    pub severity: SecretSeverity,
    /// Whether this is a false positive pattern
    pub is_false_positive: bool,
}

/// Secret severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecretSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Rate limiting policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimitingPolicy {
    /// Enable rate limiting
    pub enabled: bool,
    /// Maximum requests per window per IP
    pub requests_per_window: u32,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Maximum burst size
    pub burst_size: u32,
    /// Cleanup interval for expired entries (seconds)
    pub cleanup_interval_seconds: u64,
}

/// Rate limiting request context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRequest {
    /// Client identifier (IP, user ID, etc.)
    pub client_id: String,
    /// Request path or operation
    pub operation: String,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Rate limiting result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Current request count in window
    pub current_count: u32,
    /// Window reset time
    pub reset_time: DateTime<Utc>,
    /// Retry after seconds (if denied)
    pub retry_after_seconds: Option<u64>,
}

/// Audit policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditPolicy {
    /// Enable audit logging
    pub enabled: bool,
    /// Log file access events
    pub log_file_access: bool,
    /// Log command execution events
    pub log_command_execution: bool,
    /// Log security violations
    pub log_security_violations: bool,
    /// Log secret detections
    pub log_secret_detections: bool,
    /// Audit log retention period (days)
    pub retention_days: u32,
}

/// Council integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CouncilIntegrationConfig {
    /// Enable council integration for security decisions
    pub enabled: bool,
    /// Risk tier for security-related tasks
    pub security_risk_tier: u8,
    /// Whether to require council approval for dangerous operations
    pub require_council_approval: bool,
    /// Timeout for council decisions (seconds)
    pub council_timeout: u64,
}

/// Security violation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityViolationType {
    FileAccessDenied,
    CommandExecutionDenied,
    SecretDetected,
    DangerousOperation,
    PolicyViolation,
    UnauthorizedAccess,
    ResourceLimitExceeded,
}

/// Security violation details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityViolation {
    /// Unique violation ID
    pub id: Uuid,
    /// Violation type
    pub violation_type: SecurityViolationType,
    /// Severity level
    pub severity: SecretSeverity,
    /// Description of the violation
    pub description: String,
    /// Resource that triggered the violation
    pub resource: String,
    /// User/process that triggered the violation
    pub actor: String,
    /// Timestamp of the violation
    pub timestamp: DateTime<Utc>,
    /// Additional context
    pub context: HashMap<String, String>,
    /// Whether the violation was blocked
    pub blocked: bool,
    /// Council decision if applicable
    pub council_decision: Option<CouncilDecision>,
}

/// Council decision for security violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CouncilDecision {
    /// Decision ID
    pub decision_id: Uuid,
    /// Whether the operation was approved
    pub approved: bool,
    /// Reasoning for the decision
    pub reasoning: String,
    /// Conditions for approval
    pub conditions: Vec<String>,
    /// Timestamp of the decision
    pub timestamp: DateTime<Utc>,
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityAuditEvent {
    /// Event ID
    pub id: Uuid,
    /// Event type
    pub event_type: AuditEventType,
    /// Actor (user/process)
    pub actor: String,
    /// Resource affected
    pub resource: String,
    /// Action performed
    pub action: String,
    /// Result of the action
    pub result: AuditResult,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    FileAccess,
    CommandExecution,
    SecretDetection,
    PolicyViolation,
    CouncilDecision,
    SecurityCheck,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditResult {
    Allowed,
    Denied,
    Blocked,
    Approved,
    Rejected,
    Warning,
}

/// Source metadata for audit log ingestion
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AuditEventSource {
    /// Originating subsystem or service name
    pub system: String,
    /// Component within the subsystem that emitted the event
    pub component: String,
    /// Deployment environment descriptor (e.g., prod, staging)
    pub environment: String,
}

/// Structured audit log entry used for ingestion/analysis pipelines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditLogEntry {
    /// Schema version for forward compatibility
    pub schema_version: String,
    /// Source metadata describing the emitter
    #[serde(default)]
    pub source: AuditEventSource,
    /// Structured security audit event payload
    pub event: SecurityAuditEvent,
}

impl AuditLogEntry {
    /// Canonical schema version supported by the current parser
    pub const CURRENT_VERSION: &'static str = "1.0";

    /// Validate schema version compatibility
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.schema_version != Self::CURRENT_VERSION {
            anyhow::bail!("Unsupported audit schema version: {}", self.schema_version);
        }
        Ok(())
    }
}

/// Normalized severity level used by the analysis engine
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityLevel {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityLevel {
    /// Map domain-specific severity (e.g., secret severity) into normalized levels
    pub fn from_secret(severity: SecretSeverity) -> Self {
        match severity {
            SecretSeverity::Low => SeverityLevel::Low,
            SecretSeverity::Medium => SeverityLevel::Medium,
            SecretSeverity::High => SeverityLevel::High,
            SecretSeverity::Critical => SeverityLevel::Critical,
        }
    }
}

/// Quantitative severity score produced by the analysis engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeverityScore {
    /// Normalized severity band
    pub level: SeverityLevel,
    /// Numeric score within [0.0, 1.0]
    pub score: f32,
    /// Explanation of the contributing factors
    pub rationale: String,
    /// Event identifiers used to compute the score
    pub contributing_events: Vec<Uuid>,
}

/// Aggregated analysis derived from a batch of audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityAnalysis {
    /// Total number of processed events
    pub total_events: usize,
    /// Count of events grouped by result type
    pub events_by_result: HashMap<String, usize>,
    /// Count of events grouped by audit type
    pub events_by_type: HashMap<String, usize>,
    /// Highest severity score observed in the batch
    pub overall_severity: SeverityScore,
    /// Optional note for anomalies discovered during analysis
    pub notes: Vec<String>,
}

/// Security policy enforcement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEnforcementResult {
    /// Whether the operation was allowed
    pub allowed: bool,
    /// Violations found
    pub violations: Vec<SecurityViolation>,
    /// Audit events generated
    pub audit_events: Vec<SecurityAuditEvent>,
    /// Council decision if applicable
    pub council_decision: Option<CouncilDecision>,
    /// Enforcement time (milliseconds)
    pub enforcement_time_ms: u64,
}

/// File access request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessRequest {
    /// Request ID
    pub id: Uuid,
    /// File path
    pub file_path: String,
    /// Access type
    pub access_type: FileAccessType,
    /// Actor (user/process)
    pub actor: String,
    /// Context of the access
    pub context: HashMap<String, String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// File access types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileAccessType {
    Read,
    Write,
    Execute,
    Delete,
    Create,
    Modify,
}

/// Command execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionRequest {
    /// Request ID
    pub id: Uuid,
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub arguments: Vec<String>,
    /// Working directory
    pub working_directory: String,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Actor (user/process)
    pub actor: String,
    /// Context of the execution
    pub context: HashMap<String, String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Secrets scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsScanResult {
    /// Scan ID
    pub id: Uuid,
    /// File or content scanned
    pub target: String,
    /// Secrets found
    pub secrets_found: Vec<DetectedSecret>,
    /// Scan time (milliseconds)
    pub scan_time_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Detected secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSecret {
    /// Secret ID
    pub id: Uuid,
    /// Pattern that matched
    pub pattern: String,
    /// Severity level
    pub severity: SecretSeverity,
    /// Location of the secret
    pub location: SecretLocation,
    /// Context around the secret
    pub context: String,
    /// Whether this is a false positive
    pub is_false_positive: bool,
}

/// Secret location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretLocation {
    /// File path
    pub file_path: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
    /// Column number
    pub column_number: Option<u32>,
    /// Byte offset
    pub byte_offset: Option<usize>,
}

/// Security policy enforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    /// Total operations checked
    pub total_operations: u64,
    /// Operations allowed
    pub operations_allowed: u64,
    /// Operations denied
    pub operations_denied: u64,
    /// Operations blocked
    pub operations_blocked: u64,
    /// Violations detected
    pub violations_detected: u64,
    /// Secrets detected
    pub secrets_detected: u64,
    /// Council decisions requested
    pub council_decisions_requested: u64,
    /// Council decisions approved
    pub council_decisions_approved: u64,
    /// Average enforcement time (milliseconds)
    pub avg_enforcement_time_ms: f64,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}
