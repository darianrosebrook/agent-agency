//! Planning Engine Contracts
//!
//! Core trait definitions for planning engines that can generate and execute
//! execution plans. This provides the hexagonal architecture boundary for
//! planning functionality.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
// Note: TaskDescriptor is defined in the orchestrator crate, not contracts
use crate::types::planning::PlanningStrategy;
use crate::ExecutionPlan;
use crate::PlanState;
// Use unified validation types
use crate::types::validation::ValidationResult;

/// Core planning engine trait
/// Defines the interface for planning engines that can generate and execute plans
#[async_trait]
pub trait PlanningEngine: Send + Sync {
    /// Generate an execution plan from a working spec and task context
    async fn generate_plan(
        &self,
        working_spec: &crate::WorkingSpec,
        task_context: HashMap<String, serde_json::Value>,
    ) -> Result<ExecutionPlan, PlanningError>;

    /// Validate a plan against constraints and requirements
    async fn validate_plan(&self, plan: &ExecutionPlan) -> Result<ValidationResult, PlanningError>;

    /// Execute a plan with milestone tracking and progress monitoring
    async fn execute_plan(&self, plan: ExecutionPlan)
        -> Result<PlanExecutionResult, PlanningError>;

    /// Get planning engine capabilities and supported features
    fn capabilities(&self) -> PlanningCapabilities;
}

/// Planning engine capabilities
/// Describes what features and constraints a planning engine supports
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningCapabilities {
    /// Supports parallel execution of independent milestones
    pub supports_parallel_execution: bool,

    /// Supports dependency graph analysis and cycle detection
    pub supports_dependency_analysis: bool,

    /// Supports adaptive planning with real-time adjustments
    pub supports_adaptive_planning: bool,

    /// Maximum number of milestones that can execute in parallel
    pub max_milestone_parallelism: usize,

    /// Supported planning strategies
    pub supported_strategies: Vec<PlanningStrategy>,

    /// Maximum plan complexity (estimated milestones)
    pub max_plan_complexity: usize,
}

/// Planning operation errors
#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Validation failed: {reason}")]
    ValidationError { reason: String },

    #[error("Dependency cycle detected: {:?}", cycle)]
    DependencyCycle { cycle: Vec<String> },

    #[error("Resource constraint violation: {constraint}")]
    ResourceConstraint { constraint: String },

    #[error("Scope violation: {violation}")]
    ScopeViolation { violation: String },

    #[error("Working spec incompatible: {reason}")]
    IncompatibleSpec { reason: String },

    #[error("Execution failed: {reason}")]
    ExecutionError { reason: String },

    #[error("Evidence validation failed: {reason}")]
    EvidenceError { reason: String },

    #[error("Council rejection: {reason}")]
    CouncilRejection { reason: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Plan execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanExecutionResult {
    /// Plan identifier
    #[schemars(with = "String")]
    pub plan_id: Uuid,

    /// Whether execution completed successfully
    pub success: bool,

    /// Number of milestones completed
    pub milestones_completed: usize,

    /// Total execution time in milliseconds
    pub total_duration_ms: u64,

    /// Execution evidence and artifacts
    pub evidence: ExecutionEvidence,

    /// Execution metrics and statistics
    pub metrics: ExecutionMetrics,

    /// Final plan state
    pub final_state: PlanState,

    /// Execution timeline
    pub timeline: Vec<ExecutionEvent>,
}

/// Execution evidence bundle
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEvidence {
    /// Plan-level evidence
    pub plan_evidence: Vec<EvidenceArtifact>,

    /// Milestone-specific evidence
    pub milestone_evidence: HashMap<String, Vec<EvidenceArtifact>>,

    /// Quality gate validation results
    pub quality_validation: Vec<QualityValidationResult>,

    /// Council review results
    pub council_reviews: Vec<CouncilReviewResult>,
}

/// Individual evidence artifact
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceArtifact {
    /// Artifact type
    pub artifact_type: ArtifactType,

    /// Artifact data (JSON-serializable)
    pub data: serde_json::Value,

    /// Whether evidence validation passed
    pub verified: bool,

    /// Validation timestamp
    #[schemars(with = "String")]
    pub validated_at: DateTime<Utc>,

    /// Validation metadata
    pub metadata: HashMap<String, String>,
}

/// Types of evidence artifacts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ArtifactType {
    /// Test execution results
    TestResults,

    /// Code coverage report
    CoverageReport,

    /// Mutation testing results
    MutationScore,

    /// Security scan results
    SecurityScan,

    /// Performance benchmark results
    PerformanceMetrics,

    /// Code quality metrics
    CodeQuality,

    /// Documentation artifacts
    Documentation,

    /// Custom evidence type
    Custom(String),
}

/// Quality validation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityValidationResult {
    /// Quality gate that was validated
    pub gate_type: String,

    /// Whether validation passed
    pub passed: bool,

    /// Validation score (0.0-1.0)
    pub score: f64,

    /// Validation details
    pub details: HashMap<String, serde_json::Value>,

    /// Validation timestamp
    #[schemars(with = "String")]
    pub validated_at: DateTime<Utc>,
}

/// Council review result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CouncilReviewResult {
    /// Council session ID
    pub session_id: String,

    /// Review decision
    pub decision: String,

    /// Review score (0.0-1.0)
    pub score: f64,

    /// Review comments and feedback
    pub feedback: Vec<String>,

    /// Review timestamp
    #[schemars(with = "String")]
    pub reviewed_at: DateTime<Utc>,
}

/// Execution metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionMetrics {
    /// Total milestones attempted
    pub total_milestones: usize,

    /// Milestones completed successfully
    pub successful_milestones: usize,

    /// Milestones that failed
    pub failed_milestones: usize,

    /// Milestones skipped
    pub skipped_milestones: usize,

    /// Average milestone execution time
    pub avg_milestone_time_ms: f64,

    /// Total parallel execution time saved
    pub parallel_time_saved_ms: u64,

    /// Resource utilization statistics
    pub resource_utilization: ResourceUtilization,

    /// Quality metrics
    pub quality_metrics: QualityMetrics,

    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Resource utilization statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceUtilization {
    /// CPU utilization percentage
    pub cpu_utilization: f64,

    /// Memory utilization percentage
    pub memory_utilization: f64,

    /// Network I/O in bytes
    pub network_io_bytes: u64,

    /// Disk I/O in bytes
    pub disk_io_bytes: u64,

    /// Worker utilization statistics
    pub worker_utilization: HashMap<String, f64>,
}

/// Quality metrics from execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityMetrics {
    /// Average test coverage achieved
    pub avg_coverage: f64,

    /// Average mutation score
    pub avg_mutation_score: f64,

    /// Number of security issues found
    pub security_issues_found: usize,

    /// Number of performance regressions
    pub performance_regressions: usize,

    /// Code quality score
    pub code_quality_score: f64,
}

/// Detailed quality metrics for plan evaluation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetailedQualityMetrics {
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,

    /// Coverage metrics
    pub coverage: CoverageMetrics,

    /// Test quality metrics
    pub test_quality: TestQualityMetrics,

    /// Code quality metrics
    pub code_quality: CodeQualityMetrics,

    /// Documentation quality metrics
    pub documentation_quality: DocumentationQualityMetrics,

    /// Measured at timestamp
    #[schemars(with = "String")]
    pub measured_at: chrono::DateTime<chrono::Utc>,
}

/// Coverage metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoverageMetrics {
    /// Line coverage percentage (0.0 to 100.0)
    pub line_coverage_percent: f64,

    /// Branch coverage percentage (0.0 to 100.0)
    pub branch_coverage_percent: f64,

    /// Function coverage percentage (0.0 to 100.0)
    pub function_coverage_percent: f64,

    /// Mutation score (0.0 to 1.0)
    pub mutation_score: f64,
}

/// Test quality metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestQualityMetrics {
    /// Test to code ratio
    pub test_to_code_ratio: f64,

    /// Average test execution time in ms
    pub avg_test_execution_ms: f64,

    /// Test flakiness rate (0.0 to 1.0)
    pub flakiness_rate: f64,

    /// Integration test coverage (0.0 to 1.0)
    pub integration_coverage: f64,
}

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeQualityMetrics {
    /// Cyclomatic complexity average
    pub avg_cyclomatic_complexity: f64,

    /// Maintainability index
    pub maintainability_index: f64,

    /// Technical debt ratio
    pub technical_debt_ratio: f64,

    /// Code duplication percentage
    pub duplication_percent: f64,
}

/// Documentation quality metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationQualityMetrics {
    /// Documentation coverage percentage
    pub documentation_coverage_percent: f64,

    /// API documentation completeness
    pub api_docs_completeness: f64,

    /// Code comment quality score
    pub comment_quality_score: f64,

    /// README completeness score
    pub readme_completeness: f64,
}

/// Hardware resource requirements for plan execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HardwareResourceRequirements {
    /// Total CPU cores needed
    pub total_cpu_cores: usize,

    /// Peak memory needed (MB)
    pub peak_memory_mb: usize,

    /// Total disk space needed (MB)
    pub total_disk_mb: usize,

    /// Network requirements
    pub network_requirements: NetworkRequirements,

    /// Estimated execution time (milliseconds)
    pub estimated_duration_ms: u64,
}

impl Default for HardwareResourceRequirements {
    fn default() -> Self {
        Self {
            total_cpu_cores: 1,
            peak_memory_mb: 1024,
            total_disk_mb: 1024,
            network_requirements: NetworkRequirements::default(),
            estimated_duration_ms: 60000, // 1 minute
        }
    }
}

/// Network requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetworkRequirements {
    /// Peak bandwidth needed (Mbps)
    pub peak_bandwidth_mbps: f64,

    /// External services required
    pub external_services: Vec<String>,

    /// Network security requirements
    pub security_requirements: Vec<String>,
}

impl Default for NetworkRequirements {
    fn default() -> Self {
        Self {
            peak_bandwidth_mbps: 10.0,
            external_services: vec![],
            security_requirements: vec![],
        }
    }
}

/// Human resource requirements for task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HumanResourceRequirements {
    /// Number of engineers needed
    pub engineer_count: usize,

    /// Specialized skills required
    pub specialized_skills: Vec<String>,

    /// Infrastructure needs
    pub infrastructure_needs: Vec<String>,
}

/// Performance metrics from execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceMetrics {
    /// Total execution time
    pub total_time_ms: u64,

    /// Time spent waiting for dependencies
    pub dependency_wait_time_ms: u64,

    /// Time spent on parallel execution
    pub parallel_execution_time_ms: u64,

    /// Time spent on sequential execution
    pub sequential_execution_time_ms: u64,

    /// Efficiency ratio (parallel vs sequential)
    pub efficiency_ratio: f64,
}

/// Execution timeline event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEvent {
    /// Event type
    pub event_type: ExecutionEventType,

    /// Event timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,

    /// Milestone ID (if applicable)
    pub milestone_id: Option<String>,

    /// Event description
    pub description: String,

    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of execution events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionEventType {
    /// Plan execution started
    PlanStarted,

    /// Batch execution started
    BatchStarted,

    /// Milestone execution started
    MilestoneStarted,

    /// Milestone execution completed
    MilestoneCompleted,

    /// Batch execution completed
    BatchCompleted,

    /// Milestone execution failed
    MilestoneFailed,

    /// Dependency resolved
    DependencyResolved,

    /// Quality gate validated
    QualityGateValidated,

    /// Council review completed
    CouncilReviewCompleted,

    /// Worker assigned to milestone
    WorkerAssigned,

    /// Worker completed milestone
    WorkerCompleted,

    /// Evidence collected
    EvidenceCollected,

    /// Timeline updated
    TimelineUpdated,

    /// Risk assessed
    RiskAssessed,

    /// Plan execution completed
    PlanCompleted,

    /// Plan execution failed
    PlanFailed,

    /// Custom event type
    Custom(String),
}

impl From<&str> for ExecutionEventType {
    fn from(s: &str) -> Self {
        match s {
            "PlanStarted" => ExecutionEventType::PlanStarted,
            "BatchStarted" => ExecutionEventType::BatchStarted,
            "MilestoneStarted" => ExecutionEventType::MilestoneStarted,
            "MilestoneCompleted" => ExecutionEventType::MilestoneCompleted,
            "BatchCompleted" => ExecutionEventType::BatchCompleted,
            "MilestoneFailed" => ExecutionEventType::MilestoneFailed,
            "DependencyResolved" => ExecutionEventType::DependencyResolved,
            "QualityGateValidated" => ExecutionEventType::QualityGateValidated,
            "CouncilReviewCompleted" => ExecutionEventType::CouncilReviewCompleted,
            "WorkerAssigned" => ExecutionEventType::WorkerAssigned,
            "WorkerCompleted" => ExecutionEventType::WorkerCompleted,
            "EvidenceCollected" => ExecutionEventType::EvidenceCollected,
            "TimelineUpdated" => ExecutionEventType::TimelineUpdated,
            "RiskAssessed" => ExecutionEventType::RiskAssessed,
            "PlanCompleted" => ExecutionEventType::PlanCompleted,
            "PlanFailed" => ExecutionEventType::PlanFailed,
            _ => ExecutionEventType::Custom(s.to_string()),
        }
    }
}

/// Plan status enumeration (alias for PlanState for compatibility)
pub type PlanStatus = crate::PlanState;

/// Plan priority levels for execution scheduling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanPriority {
    /// Low priority - execute when resources available
    Low,

    /// Normal priority - standard execution priority
    Normal,

    /// Medium priority - elevated execution priority
    Medium,

    /// High priority - expedited execution
    High,

    /// Critical priority - immediate execution required
    Critical,
}

/// Plan execution state for detailed tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanExecutionState {
    /// Plan is queued for execution
    Queued,

    /// Plan is being prepared for execution
    Preparing,

    /// Plan is actively executing
    Executing,

    /// Plan execution is paused
    Paused,

    /// Plan execution completed successfully
    Completed,

    /// Plan execution failed
    Failed,

    /// Plan execution was cancelled
    Cancelled,
}

#[test]
fn test_planning_capabilities_serialization() {
    let capabilities = PlanningCapabilities {
        supports_parallel_execution: true,
        supports_dependency_analysis: true,
        supports_adaptive_planning: false,
        max_milestone_parallelism: 5,
        supported_strategies: vec![
            PlanningStrategy::TopDown,
            PlanningStrategy::DependencyDriven,
        ],
        max_plan_complexity: 20,
    };

    let serialized = serde_json::to_string(&capabilities).unwrap();
    let deserialized: PlanningCapabilities = serde_json::from_str(&serialized).unwrap();

    assert_eq!(
        capabilities.supports_parallel_execution,
        deserialized.supports_parallel_execution
    );
    assert_eq!(
        capabilities.max_milestone_parallelism,
        deserialized.max_milestone_parallelism
    );
}

#[test]
fn test_validation_result_creation() {
    let result: ValidationResult = ValidationResult {
        valid: true,
        score: 0.95,
        issues: vec![],
        warnings: vec!["Consider adding more tests".to_string()],
        suggestions: vec!["Add integration tests".to_string()],
        metadata: std::collections::HashMap::new(),
    };

    assert!(result.valid);
    assert_eq!(result.score, 0.95);
    assert!(result.issues.is_empty());
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.suggestions.len(), 1);
}

#[test]
fn test_execution_event_creation() {
    let event = ExecutionEvent {
        event_type: ExecutionEventType::PlanStarted,
        timestamp: Utc::now(),
        milestone_id: None,
        description: "Plan execution initiated".to_string(),
        metadata: HashMap::new(),
    };

    assert_eq!(event.event_type, ExecutionEventType::PlanStarted);
    assert!(event.milestone_id.is_none());
    assert_eq!(event.description, "Plan execution initiated");
}

#[test]
fn execution_event_type_from_all_variants() {
    // Test all 16 match arms
    assert!(matches!(
        ExecutionEventType::from("PlanStarted"),
        ExecutionEventType::PlanStarted
    ));
    assert!(matches!(
        ExecutionEventType::from("BatchStarted"),
        ExecutionEventType::BatchStarted
    ));
    assert!(matches!(
        ExecutionEventType::from("MilestoneStarted"),
        ExecutionEventType::MilestoneStarted
    ));
    assert!(matches!(
        ExecutionEventType::from("MilestoneCompleted"),
        ExecutionEventType::MilestoneCompleted
    ));
    assert!(matches!(
        ExecutionEventType::from("BatchCompleted"),
        ExecutionEventType::BatchCompleted
    ));
    assert!(matches!(
        ExecutionEventType::from("MilestoneFailed"),
        ExecutionEventType::MilestoneFailed
    ));
    assert!(matches!(
        ExecutionEventType::from("DependencyResolved"),
        ExecutionEventType::DependencyResolved
    ));
    assert!(matches!(
        ExecutionEventType::from("QualityGateValidated"),
        ExecutionEventType::QualityGateValidated
    ));
    assert!(matches!(
        ExecutionEventType::from("CouncilReviewCompleted"),
        ExecutionEventType::CouncilReviewCompleted
    ));
    assert!(matches!(
        ExecutionEventType::from("WorkerAssigned"),
        ExecutionEventType::WorkerAssigned
    ));
    assert!(matches!(
        ExecutionEventType::from("WorkerCompleted"),
        ExecutionEventType::WorkerCompleted
    ));
    assert!(matches!(
        ExecutionEventType::from("EvidenceCollected"),
        ExecutionEventType::EvidenceCollected
    ));
    assert!(matches!(
        ExecutionEventType::from("TimelineUpdated"),
        ExecutionEventType::TimelineUpdated
    ));
    assert!(matches!(
        ExecutionEventType::from("RiskAssessed"),
        ExecutionEventType::RiskAssessed
    ));
    assert!(matches!(
        ExecutionEventType::from("PlanCompleted"),
        ExecutionEventType::PlanCompleted
    ));
    assert!(matches!(
        ExecutionEventType::from("PlanFailed"),
        ExecutionEventType::PlanFailed
    ));
    // Test custom variant (catch-all)
    match ExecutionEventType::from("CustomEvent") {
        ExecutionEventType::Custom(s) => assert_eq!(s, "CustomEvent"),
        _ => panic!("Expected Custom variant"),
    }
}
