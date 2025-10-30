//! Planning Engine Contracts
//!
//! Core trait definitions for planning engines that can generate and execute
//! execution plans. This provides the hexagonal architecture boundary for
//! planning functionality.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
// Note: TaskDescriptor is defined in the orchestrator crate, not contracts
use crate::ExecutionPlan;
use crate::PlanState;

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
    async fn validate_plan(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<ValidationResult, PlanningError>;

    /// Execute a plan with milestone tracking and progress monitoring
    async fn execute_plan(
        &self,
        plan: ExecutionPlan,
    ) -> Result<PlanExecutionResult, PlanningError>;

    /// Get planning engine capabilities and supported features
    fn capabilities(&self) -> PlanningCapabilities;
}

/// Planning engine capabilities
/// Describes what features and constraints a planning engine supports
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Planning strategies available to engines
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanningStrategy {
    /// Top-down decomposition from high-level requirements
    TopDown,

    /// Bottom-up composition from existing tool chains
    BottomUp,

    /// Dependency-driven analysis identifying critical paths
    DependencyDriven,

    /// Risk-based prioritization of high-risk milestones
    RiskBased,

    /// Hybrid approach combining multiple strategies
    Hybrid,

    /// AI-assisted planning with human oversight
    AIAssisted,

    /// Template-based planning from proven patterns
    TemplateBased,
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

/// Validation result with detailed feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,

    /// Validation score (0.0-1.0)
    pub score: f64,

    /// Detailed validation issues
    pub issues: Vec<ValidationIssue>,

    /// Validation warnings (non-blocking)
    pub warnings: Vec<String>,

    /// Suggested improvements
    pub suggestions: Vec<String>,
}

/// Individual validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue severity
    pub severity: ValidationSeverity,

    /// Issue category
    pub category: ValidationCategory,

    /// Human-readable description
    pub description: String,

    /// Affected milestone or component
    pub affected_component: Option<String>,

    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical issue preventing execution
    Critical,

    /// High-priority issue requiring attention
    High,

    /// Medium-priority issue
    Medium,

    /// Low-priority issue
    Low,

    /// Informational note
    Info,
}

/// Validation categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationCategory {
    /// Dependency-related issues
    Dependency,

    /// Scope boundary violations
    Scope,

    /// Resource constraint issues
    Resource,

    /// Quality gate violations
    Quality,

    /// Evidence requirement issues
    Evidence,

    /// Council compliance issues
    Council,

    /// Performance constraint issues
    Performance,

    /// Security requirement violations
    Security,
}

/// Plan execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExecutionResult {
    /// Plan identifier
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    /// Artifact type
    pub artifact_type: ArtifactType,

    /// Artifact data (JSON-serializable)
    pub data: serde_json::Value,

    /// Whether evidence validation passed
    pub verified: bool,

    /// Validation timestamp
    pub validated_at: DateTime<Utc>,

    /// Validation metadata
    pub metadata: HashMap<String, String>,
}

/// Types of evidence artifacts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub validated_at: DateTime<Utc>,
}

/// Council review result
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub reviewed_at: DateTime<Utc>,
}

/// Execution metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Performance metrics from execution
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    /// Event type
    pub event_type: ExecutionEventType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Milestone ID (if applicable)
    pub milestone_id: Option<String>,

    /// Event description
    pub description: String,

    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of execution events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEventType {
    /// Plan execution started
    PlanStarted,

    /// Milestone execution started
    MilestoneStarted,

    /// Milestone execution completed
    MilestoneCompleted,

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

    /// Evidence collected
    EvidenceCollected,

    /// Plan execution completed
    PlanCompleted,

    /// Plan execution failed
    PlanFailed,

    /// Custom event type
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planning_capabilities_serialization() {
        let capabilities = PlanningCapabilities {
            supports_parallel_execution: true,
            supports_dependency_analysis: true,
            supports_adaptive_planning: false,
            max_milestone_parallelism: 5,
            supported_strategies: vec![PlanningStrategy::TopDown, PlanningStrategy::DependencyDriven],
            max_plan_complexity: 20,
        };

        let serialized = serde_json::to_string(&capabilities).unwrap();
        let deserialized: PlanningCapabilities = serde_json::from_str(&serialized).unwrap();

        assert_eq!(capabilities.supports_parallel_execution, deserialized.supports_parallel_execution);
        assert_eq!(capabilities.max_milestone_parallelism, deserialized.max_milestone_parallelism);
    }

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult {
            valid: true,
            score: 0.95,
            issues: vec![],
            warnings: vec!["Consider adding more tests".to_string()],
            suggestions: vec!["Add integration tests".to_string()],
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
}
