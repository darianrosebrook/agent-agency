//! Milestone Contract Specifications
//!
//! Specialized contract structures for milestone execution, evidence collection,
//! and completion validation within the planning system.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Milestone execution contract
/// Defines the complete specification for executing a milestone
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MilestoneContract {
    /// Milestone identifier
    pub milestone_id: String,

    /// Contract version for compatibility
    pub version: String,

    /// Execution specification
    pub execution_spec: ExecutionSpec,

    /// Evidence collection requirements
    pub evidence_spec: EvidenceSpec,

    /// Quality validation requirements
    pub quality_spec: QualitySpec,

    /// Resource requirements and constraints
    pub resource_spec: ResourceSpec,

    /// Rollback and recovery specifications
    pub recovery_spec: RecoverySpec,

    /// Monitoring and observability requirements
    pub monitoring_spec: MonitoringSpec,

    /// Contract metadata
    pub metadata: ContractMetadata,
}

/// Execution specification for milestone
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionSpec {
    /// Execution strategy (parallel, sequential, conditional)
    pub strategy: ExecutionStrategy,

    /// Worker requirements and capabilities needed
    pub worker_requirements: Vec<WorkerRequirement>,

    /// Execution timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Retry backoff strategy
    pub retry_backoff: RetryBackoff,

    /// Execution environment requirements
    pub environment: ExecutionEnvironment,

    /// Pre-execution setup commands
    pub setup_commands: Vec<String>,

    /// Post-execution cleanup commands
    pub cleanup_commands: Vec<String>,
}

/// Execution strategy options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionStrategy {
    /// Execute all tasks in parallel
    Parallel,

    /// Execute tasks sequentially
    Sequential,

    /// Execute tasks based on conditions
    Conditional,

    /// Custom execution strategy
    Custom(String),
}

/// Worker capability requirement
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerRequirement {
    /// Required capability name
    pub capability: String,

    /// Minimum capability level (0.0-1.0)
    pub min_level: f64,

    /// Whether capability is mandatory
    pub mandatory: bool,

    /// Alternative capabilities that could substitute
    pub alternatives: Vec<String>,
}

/// Retry backoff configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetryBackoff {
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,

    /// Backoff multiplier
    pub multiplier: f64,

    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,

    /// Jitter factor (0.0-1.0)
    pub jitter_factor: f64,
}

/// Execution environment requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEnvironment {
    /// Required operating system
    pub os: Option<String>,

    /// Required CPU architecture
    pub architecture: Option<String>,

    /// Required memory in MB
    pub memory_mb: usize,

    /// Required disk space in MB
    pub disk_mb: usize,

    /// Required network access
    pub network_access: Vec<String>,

    /// Required environment variables
    pub env_vars: Vec<String>,

    /// Required external services
    pub external_services: Vec<String>,
}

/// Evidence collection specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceSpec {
    /// Evidence collection strategy
    pub collection_strategy: EvidenceCollectionStrategy,

    /// Required evidence artifacts
    pub required_artifacts: Vec<ArtifactRequirement>,

    /// Evidence validation rules
    pub validation_rules: Vec<ValidationRule>,

    /// Evidence storage configuration
    pub storage_config: EvidenceStorageConfig,

    /// Evidence retention policy
    pub retention_policy: EvidenceRetentionPolicy,
}

/// Evidence collection strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EvidenceCollectionStrategy {
    /// Collect all possible evidence
    Comprehensive,

    /// Collect only required evidence
    Minimal,

    /// Collect evidence based on risk level
    RiskBased,

    /// Custom collection strategy
    Custom(String),
}

/// Artifact requirement specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactRequirement {
    /// Artifact type identifier
    pub artifact_type: String,

    /// Artifact name for identification
    pub name: String,

    /// Collection method specification
    pub collection_method: String,

    /// Validation criteria
    pub validation_criteria: HashMap<String, serde_json::Value>,

    /// Whether artifact is mandatory
    pub mandatory: bool,

    /// Expected artifact size in bytes (approximate)
    pub expected_size_bytes: Option<u64>,

    /// Artifact format specification
    pub format_spec: Option<String>,
}

/// Validation rule for evidence
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationRule {
    /// Rule identifier
    pub rule_id: String,

    /// Rule type
    pub rule_type: ValidationRuleType,

    /// Rule expression or specification
    pub rule_spec: String,

    /// Rule severity if violated
    pub severity: ValidationSeverity,

    /// Whether rule failure blocks milestone completion
    pub blocking: bool,

    /// Rule metadata
    pub metadata: HashMap<String, String>,
}

/// Validation rule types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ValidationRuleType {
    /// Schema validation
    Schema,

    /// Content validation
    Content,

    /// Performance validation
    Performance,

    /// Security validation
    Security,

    /// Custom validation rule
    Custom(String),
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ValidationSeverity {
    /// Informational only
    Info,

    /// Warning level
    Warning,

    /// Error that may block completion
    Error,

    /// Critical error that blocks completion
    Critical,
}

/// Evidence storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceStorageConfig {
    /// Storage backend type
    pub backend: EvidenceStorageBackend,

    /// Storage location/path
    pub location: String,

    /// Compression settings
    pub compression: Option<CompressionConfig>,

    /// Encryption settings
    pub encryption: Option<EncryptionConfig>,

    /// Access control settings
    pub access_control: Option<AccessControlConfig>,
}

/// Evidence storage backend types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EvidenceStorageBackend {
    /// Local file system
    FileSystem,

    /// Database storage
    Database,

    /// Object storage (S3, etc.)
    ObjectStorage,

    /// Distributed storage
    Distributed,

    /// Custom storage backend
    Custom(String),
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: String,

    /// Compression level (0-9)
    pub level: u32,

    /// Whether to compress metadata separately
    pub compress_metadata: bool,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EncryptionConfig {
    /// Encryption algorithm
    pub algorithm: String,

    /// Key management approach
    pub key_management: String,

    /// Whether to encrypt metadata
    pub encrypt_metadata: bool,
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccessControlConfig {
    /// Access control model
    pub model: String,

    /// Required permissions for read access
    pub read_permissions: Vec<String>,

    /// Required permissions for write access
    pub write_permissions: Vec<String>,
}

/// Evidence retention policy
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceRetentionPolicy {
    /// Retention duration in days
    pub retention_days: u32,

    /// Retention strategy
    pub strategy: RetentionStrategy,

    /// Archive configuration
    pub archive_config: Option<ArchiveConfig>,

    /// Deletion configuration
    pub deletion_config: Option<DeletionConfig>,
}

/// Retention strategy options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RetentionStrategy {
    /// Keep all evidence for retention period
    KeepAll,

    /// Keep only validated evidence
    KeepValidatedOnly,

    /// Keep samples based on sampling rate
    Sampled,

    /// Custom retention strategy
    Custom(String),
}

/// Archive configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArchiveConfig {
    /// Archive location
    pub location: String,

    /// Archive format
    pub format: String,

    /// Archive compression
    pub compression: Option<String>,
}

/// Deletion configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeletionConfig {
    /// Deletion method
    pub method: String,

    /// Secure deletion (multiple passes)
    pub secure_deletion: bool,

    /// Deletion audit logging
    pub audit_deletion: bool,
}

/// Quality specification for milestone
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualitySpec {
    /// Code quality requirements
    pub code_quality: CodeQualityRequirements,

    /// Testing requirements
    pub testing: TestingRequirements,

    /// Security requirements
    pub security: SecurityRequirements,

    /// Performance requirements
    pub performance: PerformanceRequirements,

    /// Documentation requirements
    pub documentation: DocumentationRequirements,

    /// Overall quality score threshold (0.0-1.0)
    pub quality_threshold: f64,
}

/// Code quality requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeQualityRequirements {
    /// Minimum code coverage (0.0-1.0)
    pub min_coverage: f64,

    /// Maximum cyclomatic complexity
    pub max_complexity: u32,

    /// Maximum lines per function
    pub max_lines_per_function: usize,

    /// Required linting rules
    pub required_linting: Vec<String>,

    /// Code smell thresholds
    pub smell_thresholds: HashMap<String, u32>,
}

/// Testing requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestingRequirements {
    /// Required test types
    pub required_test_types: Vec<String>,

    /// Minimum test count
    pub min_test_count: usize,

    /// Test execution timeout
    pub test_timeout_ms: u64,

    /// Test environment requirements
    pub environment_requirements: Vec<String>,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecurityRequirements {
    /// Required security scans
    pub required_scans: Vec<String>,

    /// Maximum security vulnerabilities by severity
    pub max_vulnerabilities: HashMap<String, usize>,

    /// Required security controls
    pub required_controls: Vec<String>,

    /// Security audit requirements
    pub audit_requirements: Vec<String>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceRequirements {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,

    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Minimum throughput requirements
    pub min_throughput: Option<f64>,

    /// Performance regression thresholds
    pub regression_thresholds: HashMap<String, f64>,
}

/// Documentation requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationRequirements {
    /// Required documentation types
    pub required_types: Vec<String>,

    /// Documentation coverage minimum (0.0-1.0)
    pub min_coverage: f64,

    /// Required documentation formats
    pub required_formats: Vec<String>,

    /// Documentation quality checks
    pub quality_checks: Vec<String>,
}

/// Resource specification for milestone
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceSpec {
    /// CPU requirements
    pub cpu: CpuRequirements,

    /// Memory requirements
    pub memory: MemoryRequirements,

    /// Disk requirements
    pub disk: DiskRequirements,

    /// Network requirements
    pub network: NetworkRequirements,

    /// External service dependencies
    pub external_services: Vec<ServiceDependency>,

    /// Resource allocation strategy
    pub allocation_strategy: ResourceAllocationStrategy,
}

/// CPU requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CpuRequirements {
    /// Minimum CPU cores required
    pub min_cores: usize,

    /// Preferred CPU architecture
    pub preferred_architecture: Option<String>,

    /// CPU utilization limit (0.0-1.0)
    pub utilization_limit: f64,

    /// CPU affinity requirements
    pub affinity_requirements: Option<String>,
}

/// Memory requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryRequirements {
    /// Minimum memory in MB
    pub min_memory_mb: usize,

    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Memory allocation strategy
    pub allocation_strategy: String,

    /// Memory monitoring requirements
    pub monitoring_enabled: bool,
}

/// Disk requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiskRequirements {
    /// Minimum disk space in MB
    pub min_disk_mb: usize,

    /// Required disk I/O speed (MB/s)
    pub min_io_speed_mbps: Option<f64>,

    /// Disk type preferences
    pub preferred_disk_type: Option<String>,

    /// Disk access patterns
    pub access_patterns: Vec<String>,
}

/// Network requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetworkRequirements {
    /// Required network bandwidth (Mbps)
    pub min_bandwidth_mbps: Option<f64>,

    /// Required network latency (ms)
    pub max_latency_ms: Option<u64>,

    /// Required network connectivity
    pub required_connectivity: Vec<String>,

    /// Network security requirements
    pub security_requirements: Vec<String>,
}

/// Service dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceDependency {
    /// Service name
    pub service_name: String,

    /// Service type
    pub service_type: String,

    /// Service endpoint
    pub endpoint: String,

    /// Connection requirements
    pub connection_requirements: HashMap<String, String>,

    /// Service availability requirements
    pub availability_requirements: HashMap<String, String>,
}

/// Resource allocation strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ResourceAllocationStrategy {
    /// Allocate resources as needed
    OnDemand,

    /// Pre-allocate all required resources
    PreAllocated,

    /// Allocate resources dynamically
    Dynamic,

    /// Custom allocation strategy
    Custom(String),
}

/// Recovery specification for milestone
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoverySpec {
    /// Rollback strategy
    pub rollback_strategy: RollbackStrategy,

    /// Recovery procedures
    pub recovery_procedures: Vec<RecoveryProcedure>,

    /// Checkpoint configuration
    pub checkpoint_config: CheckpointConfig,

    /// Failure handling strategy
    pub failure_handling: FailureHandlingStrategy,

    /// Recovery time objectives (RTO)
    pub rto_ms: u64,

    /// Recovery point objectives (RPO)
    pub rpo_ms: u64,
}

/// Rollback strategy options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RollbackStrategy {
    /// No rollback possible
    None,

    /// Automatic rollback on failure
    Automatic,

    /// Manual rollback required
    Manual,

    /// Conditional rollback based on failure type
    Conditional,

    /// Custom rollback strategy
    Custom(String),
}

/// Recovery procedure specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryProcedure {
    /// Procedure name
    pub name: String,

    /// Procedure type
    pub procedure_type: RecoveryProcedureType,

    /// Procedure steps
    pub steps: Vec<String>,

    /// Estimated recovery time
    pub estimated_time_ms: u64,

    /// Required resources for recovery
    pub required_resources: Vec<String>,
}

/// Recovery procedure types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RecoveryProcedureType {
    /// Rollback to previous state
    Rollback,

    /// Retry the failed operation
    Retry,

    /// Skip and continue
    Skip,

    /// Manual intervention required
    Manual,

    /// Custom recovery procedure
    Custom(String),
}

/// Checkpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckpointConfig {
    /// Checkpoint frequency in milliseconds
    pub frequency_ms: u64,

    /// Checkpoint storage location
    pub storage_location: String,

    /// Maximum checkpoints to retain
    pub max_checkpoints: usize,

    /// Checkpoint compression
    pub compression_enabled: bool,
}

/// Failure handling strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FailureHandlingStrategy {
    /// Stop on first failure
    FailFast,

    /// Continue with degraded functionality
    Degraded,

    /// Attempt recovery and continue
    Recovery,

    /// Custom failure handling
    Custom(String),
}

/// Monitoring specification for milestone
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitoringSpec {
    /// Metrics to collect
    pub metrics: Vec<MetricSpec>,

    /// Logs to collect
    pub logs: Vec<LogSpec>,

    /// Traces to collect
    pub traces: Vec<TraceSpec>,

    /// Alerts to configure
    pub alerts: Vec<AlertSpec>,

    /// Dashboard configuration
    pub dashboard: Option<DashboardSpec>,
}

/// Metric specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricSpec {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: MetricType,

    /// Collection interval in milliseconds
    pub collection_interval_ms: u64,

    /// Metric labels
    pub labels: HashMap<String, String>,

    /// Aggregation method
    pub aggregation: Option<String>,
}

/// Metric types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum MetricType {
    /// Counter metric
    Counter,

    /// Gauge metric
    Gauge,

    /// Histogram metric
    Histogram,

    /// Summary metric
    Summary,
}

/// Log specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogSpec {
    /// Log name
    pub name: String,

    /// Log level
    pub level: LogLevel,

    /// Log format
    pub format: String,

    /// Log destination
    pub destination: String,

    /// Log retention period in days
    pub retention_days: u32,
}

/// Log levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum LogLevel {
    /// Debug level
    Debug,

    /// Info level
    Info,

    /// Warning level
    Warn,

    /// Error level
    Error,

    /// Critical level
    Critical,
}

/// Trace specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TraceSpec {
    /// Trace name
    pub name: String,

    /// Trace sampling rate (0.0-1.0)
    pub sampling_rate: f64,

    /// Trace attributes
    pub attributes: HashMap<String, String>,

    /// Trace exporter configuration
    pub exporter_config: HashMap<String, String>,
}

/// Alert specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlertSpec {
    /// Alert name
    pub name: String,

    /// Alert condition
    pub condition: String,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Alert channels
    pub channels: Vec<String>,

    /// Alert throttling configuration
    pub throttling: Option<AlertThrottling>,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AlertSeverity {
    /// Informational alert
    Info,

    /// Warning alert
    Warning,

    /// Error alert
    Error,

    /// Critical alert
    Critical,
}

/// Alert throttling configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlertThrottling {
    /// Throttling window in milliseconds
    pub window_ms: u64,

    /// Maximum alerts per window
    pub max_alerts_per_window: usize,

    /// Throttling strategy
    pub strategy: ThrottlingStrategy,
}

/// Throttling strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ThrottlingStrategy {
    /// Drop excess alerts
    Drop,

    /// Aggregate alerts
    Aggregate,

    /// Custom throttling
    Custom(String),
}

/// Dashboard specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardSpec {
    /// Dashboard name
    pub name: String,

    /// Dashboard type
    pub dashboard_type: String,

    /// Dashboard configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Dashboard refresh interval
    pub refresh_interval_ms: u64,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractMetadata {
    /// Contract creation timestamp
    pub created_at: DateTime<Utc>,

    /// Contract last modified timestamp
    pub modified_at: DateTime<Utc>,

    /// Contract author
    pub author: String,

    /// Contract version
    pub version: String,

    /// Contract description
    pub description: String,

    /// Contract tags
    pub tags: Vec<String>,

    /// Additional metadata
    pub additional_metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_contract_creation() {
        let contract = MilestoneContract {
            milestone_id: "M1".to_string(),
            version: "1.0.0".to_string(),
            execution_spec: ExecutionSpec {
                strategy: ExecutionStrategy::Parallel,
                worker_requirements: vec![],
                timeout_ms: 300000,
                max_retries: 3,
                retry_backoff: RetryBackoff {
                    initial_delay_ms: 1000,
                    multiplier: 2.0,
                    max_delay_ms: 30000,
                    jitter_factor: 0.1,
                },
                environment: ExecutionEnvironment {
                    os: Some("linux".to_string()),
                    architecture: Some("x86_64".to_string()),
                    memory_mb: 1024,
                    disk_mb: 1024,
                    network_access: vec![],
                    env_vars: vec![],
                    external_services: vec![],
                },
                setup_commands: vec![],
                cleanup_commands: vec![],
            },
            evidence_spec: EvidenceSpec {
                collection_strategy: EvidenceCollectionStrategy::Comprehensive,
                required_artifacts: vec![],
                validation_rules: vec![],
                storage_config: EvidenceStorageConfig {
                    backend: EvidenceStorageBackend::FileSystem,
                    location: "/tmp/evidence".to_string(),
                    compression: None,
                    encryption: None,
                    access_control: None,
                },
                retention_policy: EvidenceRetentionPolicy {
                    retention_days: 30,
                    strategy: RetentionStrategy::KeepAll,
                    archive_config: None,
                    deletion_config: None,
                },
            },
            quality_spec: QualitySpec {
                code_quality: CodeQualityRequirements {
                    min_coverage: 0.8,
                    max_complexity: 10,
                    max_lines_per_function: 50,
                    required_linting: vec![],
                    smell_thresholds: HashMap::new(),
                },
                testing: TestingRequirements {
                    required_test_types: vec!["unit".to_string()],
                    min_test_count: 5,
                    test_timeout_ms: 60000,
                    environment_requirements: vec![],
                },
                security: SecurityRequirements {
                    required_scans: vec![],
                    max_vulnerabilities: HashMap::new(),
                    required_controls: vec![],
                    audit_requirements: vec![],
                },
                performance: PerformanceRequirements {
                    max_execution_time_ms: 30000,
                    max_memory_mb: 512,
                    min_throughput: None,
                    regression_thresholds: HashMap::new(),
                },
                documentation: DocumentationRequirements {
                    required_types: vec![],
                    min_coverage: 0.5,
                    required_formats: vec![],
                    quality_checks: vec![],
                },
                quality_threshold: 0.8,
            },
            resource_spec: ResourceSpec {
                cpu: CpuRequirements {
                    min_cores: 1,
                    preferred_architecture: Some("x86_64".to_string()),
                    utilization_limit: 0.8,
                    affinity_requirements: None,
                },
                memory: MemoryRequirements {
                    min_memory_mb: 256,
                    max_memory_mb: 1024,
                    allocation_strategy: "dynamic".to_string(),
                    monitoring_enabled: true,
                },
                disk: DiskRequirements {
                    min_disk_mb: 100,
                    min_io_speed_mbps: Some(50.0),
                    preferred_disk_type: Some("ssd".to_string()),
                    access_patterns: vec![],
                },
                network: NetworkRequirements {
                    min_bandwidth_mbps: Some(10.0),
                    max_latency_ms: Some(100),
                    required_connectivity: vec![],
                    security_requirements: vec![],
                },
                external_services: vec![],
                allocation_strategy: ResourceAllocationStrategy::Dynamic,
            },
            recovery_spec: RecoverySpec {
                rollback_strategy: RollbackStrategy::Automatic,
                recovery_procedures: vec![],
                checkpoint_config: CheckpointConfig {
                    frequency_ms: 30000,
                    storage_location: "/tmp/checkpoints".to_string(),
                    max_checkpoints: 10,
                    compression_enabled: true,
                },
                failure_handling: FailureHandlingStrategy::Recovery,
                rto_ms: 300000,
                rpo_ms: 60000,
            },
            monitoring_spec: MonitoringSpec {
                metrics: vec![],
                logs: vec![],
                traces: vec![],
                alerts: vec![],
                dashboard: None,
            },
            metadata: ContractMetadata {
                created_at: Utc::now(),
                modified_at: Utc::now(),
                author: "test-author".to_string(),
                version: "1.0.0".to_string(),
                description: "Test milestone contract".to_string(),
                tags: vec![],
                additional_metadata: HashMap::new(),
            },
        };

        assert_eq!(contract.milestone_id, "M1");
        assert_eq!(contract.version, "1.0.0");
        assert!(matches!(contract.execution_spec.strategy, ExecutionStrategy::Parallel));
        assert_eq!(contract.quality_spec.quality_threshold, 0.8);
    }

    #[test]
    fn test_evidence_spec_validation() {
        let evidence_spec = EvidenceSpec {
            collection_strategy: EvidenceCollectionStrategy::Comprehensive,
            required_artifacts: vec![ArtifactRequirement {
                artifact_type: "test_results".to_string(),
                name: "Unit Test Results".to_string(),
                collection_method: "jest".to_string(),
                validation_criteria: HashMap::new(),
                mandatory: true,
                expected_size_bytes: Some(1024000),
                format_spec: Some("junit".to_string()),
            }],
            validation_rules: vec![ValidationRule {
                rule_id: "coverage-check".to_string(),
                rule_type: ValidationRuleType::Performance,
                rule_spec: "coverage >= 0.8".to_string(),
                severity: ValidationSeverity::Error,
                blocking: true,
                metadata: HashMap::new(),
            }],
            storage_config: EvidenceStorageConfig {
                backend: EvidenceStorageBackend::FileSystem,
                location: "/evidence".to_string(),
                compression: Some(CompressionConfig {
                    algorithm: "gzip".to_string(),
                    level: 6,
                    compress_metadata: true,
                }),
                encryption: None,
                access_control: None,
            },
            retention_policy: EvidenceRetentionPolicy {
                retention_days: 90,
                strategy: RetentionStrategy::KeepValidatedOnly,
                archive_config: Some(ArchiveConfig {
                    location: "/archive".to_string(),
                    format: "tar.gz".to_string(),
                    compression: Some("gzip".to_string()),
                }),
                deletion_config: Some(DeletionConfig {
                    method: "secure".to_string(),
                    secure_deletion: true,
                    audit_deletion: true,
                }),
            },
        };

        assert_eq!(evidence_spec.required_artifacts.len(), 1);
        assert_eq!(evidence_spec.validation_rules.len(), 1);
        assert!(matches!(evidence_spec.collection_strategy, EvidenceCollectionStrategy::Comprehensive));
        assert_eq!(evidence_spec.retention_policy.retention_days, 90);
    }

    #[test]
    fn test_resource_spec_requirements() {
        let resource_spec = ResourceSpec {
            cpu: CpuRequirements {
                min_cores: 2,
                preferred_architecture: Some("arm64".to_string()),
                utilization_limit: 0.9,
                affinity_requirements: Some("cpu:0-3".to_string()),
            },
            memory: MemoryRequirements {
                min_memory_mb: 2048,
                max_memory_mb: 4096,
                allocation_strategy: "pre-allocated".to_string(),
                monitoring_enabled: true,
            },
            disk: DiskRequirements {
                min_disk_mb: 10240,
                min_io_speed_mbps: Some(100.0),
                preferred_disk_type: Some("nvme".to_string()),
                access_patterns: vec!["random".to_string(), "sequential".to_string()],
            },
            network: NetworkRequirements {
                min_bandwidth_mbps: Some(100.0),
                max_latency_ms: Some(10),
                required_connectivity: vec!["internet".to_string()],
                security_requirements: vec!["tls".to_string()],
            },
            external_services: vec![ServiceDependency {
                service_name: "database".to_string(),
                service_type: "postgresql".to_string(),
                endpoint: "postgresql://localhost:5432".to_string(),
                connection_requirements: HashMap::new(),
                availability_requirements: HashMap::new(),
            }],
            allocation_strategy: ResourceAllocationStrategy::PreAllocated,
        };

        assert_eq!(resource_spec.cpu.min_cores, 2);
        assert_eq!(resource_spec.memory.min_memory_mb, 2048);
        assert_eq!(resource_spec.disk.min_disk_mb, 10240);
        assert_eq!(resource_spec.external_services.len(), 1);
        assert!(matches!(resource_spec.allocation_strategy, ResourceAllocationStrategy::PreAllocated));
    }
}
