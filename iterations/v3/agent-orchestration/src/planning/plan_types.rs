//! Planning Types for Orchestration
//!
//! Extended types for the planning system that build on the contracts
//! but include orchestration-specific details and runtime state.
//!
//! @author @darianrosebrook

use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use agent_agency_contracts::{
    planning_io::{
        ExecutionPlan as ContractExecutionPlan, PlanState as ContractPlanState,
        Milestone as ContractMilestone, MilestoneState as ContractMilestoneState,
        DependencyGraph as ContractDependencyGraph,
    },
    planning::{PlanningCapabilities, ValidationResult},
};

/// Extended execution plan with orchestration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Base contract plan
    pub contract_plan: ContractExecutionPlan,

    /// Orchestration-specific metadata
    pub orchestration_meta: OrchestrationMetadata,

    /// Current execution context
    pub execution_context: ExecutionContext,

    /// Active execution state
    pub execution_state: Option<ActiveExecutionState>,
}

/// Orchestration-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationMetadata {
    /// Orchestrator instance that created this plan
    pub orchestrator_id: String,

    /// Worker pool assigned to this plan
    pub worker_pool_id: String,

    /// Council session that approved this plan
    pub council_session_id: Option<String>,

    /// Audit trail correlation ID
    pub audit_correlation_id: Uuid,

    /// Planning engine used
    pub planning_engine: String,

    /// Planning version
    pub planning_version: String,
}

/// Current execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Session start time
    pub session_start: DateTime<Utc>,

    /// Current working directory
    pub working_directory: String,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Available resources
    pub available_resources: ResourceInventory,

    /// Active worker assignments
    pub worker_assignments: HashMap<String, WorkerAssignment>,

    /// Current parallel execution batches
    pub parallel_batches: Vec<ParallelBatch>,
}

/// Active execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveExecutionState {
    /// Currently executing milestones
    pub executing_milestones: HashSet<String>,

    /// Completed milestones
    pub completed_milestones: HashSet<String>,

    /// Failed milestones with reasons
    pub failed_milestones: HashMap<String, String>,

    /// Blocked milestones with dependencies
    pub blocked_milestones: HashMap<String, HashSet<String>>,

    /// Current parallel batch being executed
    pub current_batch: Option<usize>,

    /// Execution progress metrics
    pub progress: ExecutionProgress,

    /// Active evidence collection
    pub evidence_collection: EvidenceCollectionState,
}

/// Worker assignment for milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAssignment {
    /// Worker ID assigned
    pub worker_id: Uuid,

    /// Assigned at timestamp
    pub assigned_at: DateTime<Utc>,

    /// Assignment status
    pub status: AssignmentStatus,

    /// Assignment priority
    pub priority: AssignmentPriority,

    /// Resource allocation for this assignment
    pub resource_allocation: ResourceAllocation,
}

/// Assignment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    /// Worker assigned but not yet started
    Assigned,

    /// Worker actively executing
    Active,

    /// Worker completed assignment
    Completed,

    /// Worker failed assignment
    Failed,

    /// Assignment cancelled
    Cancelled,

    /// Worker reassigned to different milestone
    Reassigned,
}

/// Assignment priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentPriority {
    /// Low priority background work
    Low,

    /// Normal priority work
    Normal,

    /// High priority time-sensitive work
    High,

    /// Critical priority immediate execution
    Critical,
}

/// Resource allocation for assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// CPU cores allocated
    pub cpu_cores: usize,

    /// Memory allocated (MB)
    pub memory_mb: usize,

    /// Disk space allocated (MB)
    pub disk_mb: usize,

    /// Network bandwidth allocated (Mbps)
    pub network_mbps: Option<f64>,

    /// Time limit (milliseconds)
    pub time_limit_ms: Option<u64>,
}

/// Parallel execution batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelBatch {
    /// Batch index
    pub batch_index: usize,

    /// Milestone IDs in this batch
    pub milestone_ids: Vec<String>,

    /// Batch start time
    pub started_at: Option<DateTime<Utc>>,

    /// Batch completion time
    pub completed_at: Option<DateTime<Utc>>,

    /// Batch status
    pub status: BatchStatus,

    /// Resource requirements for batch
    pub resource_requirements: ResourceRequirements,
}

/// Batch execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Batch queued for execution
    Queued,

    /// Batch currently executing
    Executing,

    /// Batch completed successfully
    Completed,

    /// Batch failed
    Failed,

    /// Batch cancelled
    Cancelled,
}

/// Resource requirements for batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
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

/// Network requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequirements {
    /// Peak bandwidth needed (Mbps)
    pub peak_bandwidth_mbps: f64,

    /// External services required
    pub external_services: Vec<String>,

    /// Network security requirements
    pub security_requirements: Vec<String>,
}

/// Resource inventory available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInventory {
    /// Available CPU cores
    pub available_cpu_cores: usize,

    /// Available memory (MB)
    pub available_memory_mb: usize,

    /// Available disk space (MB)
    pub available_disk_mb: usize,

    /// Available network bandwidth (Mbps)
    pub available_network_mbps: f64,

    /// Available worker count by type
    pub available_workers: HashMap<String, usize>,
}

/// Execution progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProgress {
    /// Overall completion percentage (0.0-1.0)
    pub overall_completion: f64,

    /// Milestones completed
    pub milestones_completed: usize,

    /// Total milestones
    pub total_milestones: usize,

    /// Estimated time remaining (milliseconds)
    pub estimated_time_remaining_ms: Option<u64>,

    /// Current execution rate (milestones per hour)
    pub current_execution_rate: f64,

    /// Bottleneck analysis
    pub bottlenecks: Vec<String>,

    /// Parallel efficiency (0.0-1.0)
    pub parallel_efficiency: f64,
}

/// Evidence collection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCollectionState {
    /// Evidence collected so far
    pub collected_evidence: HashMap<String, EvidenceStatus>,

    /// Evidence collection failures
    pub collection_failures: Vec<EvidenceFailure>,

    /// Evidence validation results
    pub validation_results: HashMap<String, ValidationResult>,

    /// Evidence storage locations
    pub storage_locations: HashMap<String, String>,
}

/// Evidence collection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStatus {
    /// Evidence type
    pub evidence_type: String,

    /// Collection status
    pub status: EvidenceCollectionStatus,

    /// Collection start time
    pub started_at: DateTime<Utc>,

    /// Collection completion time
    pub completed_at: Option<DateTime<Utc>>,

    /// Evidence size (bytes)
    pub size_bytes: Option<u64>,

    /// Evidence quality score (0.0-1.0)
    pub quality_score: Option<f64>,
}

/// Evidence collection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceCollectionStatus {
    /// Collection not started
    NotStarted,

    /// Collection in progress
    InProgress,

    /// Collection completed successfully
    Completed,

    /// Collection failed
    Failed,

    /// Collection skipped
    Skipped,
}

/// Evidence collection failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceFailure {
    /// Milestone ID
    pub milestone_id: String,

    /// Evidence type that failed
    pub evidence_type: String,

    /// Failure reason
    pub reason: String,

    /// Failure timestamp
    pub failed_at: DateTime<Utc>,

    /// Retry count
    pub retry_count: u32,

    /// Whether failure is recoverable
    pub recoverable: bool,
}

/// Evidence bundle collected for a milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// Milestone ID this evidence is for
    pub milestone_id: String,

    /// Plan ID this evidence belongs to
    pub plan_id: Uuid,

    /// Collection timestamp
    pub collected_at: DateTime<Utc>,

    /// Evidence artifacts
    pub artifacts: Vec<EvidenceArtifact>,

    /// Quality score of evidence
    pub quality_score: f64,

    /// Whether evidence meets quality gates
    pub meets_quality_gates: bool,

    /// Collection metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Individual evidence artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    /// Artifact ID
    pub id: Uuid,

    /// Artifact type (test_results, coverage, security_scan, etc.)
    pub artifact_type: String,

    /// Artifact path or content
    pub content: EvidenceContent,

    /// Artifact quality score
    pub quality_score: f64,

    /// Collection timestamp
    pub collected_at: DateTime<Utc>,

    /// Artifact metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Evidence content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceContent {
    /// File path to evidence
    FilePath(String),

    /// Inline content as JSON
    InlineJson(serde_json::Value),

    /// Inline content as text
    InlineText(String),

    /// Binary data (base64 encoded)
    Binary(String),
}

/// Plan generation context
#[derive(Debug, Clone)]
pub struct PlanGenerationContext {
    /// Working spec to plan for
    pub working_spec: Box<dyn WorkingSpecProvider>,

    /// Task descriptor
    pub task_descriptor: Box<dyn TaskDescriptorProvider>,

    /// Available resources
    pub resource_inventory: ResourceInventory,

    /// Planning constraints
    pub constraints: PlanningConstraints,

    /// Historical planning data
    pub historical_data: Option<HistoricalPlanningData>,
}

/// Working spec provider trait
#[async_trait::async_trait]
pub trait WorkingSpecProvider: Send + Sync {
    /// Get the working spec
    async fn get_working_spec(&self) -> Result<agent_agency_contracts::WorkingSpec, anyhow::Error>;
}

/// Task descriptor provider trait
#[async_trait::async_trait]
pub trait TaskDescriptorProvider: Send + Sync {
    /// Get the task descriptor
    async fn get_task_descriptor(&self) -> Result<agent_agency_contracts::TaskDescriptor, anyhow::Error>;
}

/// Planning constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningConstraints {
    /// Maximum planning time (milliseconds)
    pub max_planning_time_ms: u64,

    /// Maximum plan complexity
    pub max_complexity: usize,

    /// Risk tolerance level
    pub risk_tolerance: RiskTolerance,

    /// Cost constraints
    pub cost_limits: Option<CostLimits>,

    /// Quality requirements
    pub quality_requirements: QualityRequirements,

    /// Parallel execution preferences
    pub parallel_preferences: ParallelPreferences,
}

/// Risk tolerance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTolerance {
    /// Conservative - prefer proven approaches
    Conservative,

    /// Balanced - mix of proven and experimental
    Balanced,

    /// Aggressive - allow experimental approaches
    Aggressive,
}

/// Cost limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostLimits {
    /// Maximum cost in cents
    pub max_cost_cents: u32,

    /// Cost per millisecond budget
    pub cost_per_ms_budget: f64,

    /// Cost optimization priority
    pub optimization_priority: CostOptimizationPriority,
}

/// Cost optimization priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostOptimizationPriority {
    /// Minimize cost
    MinimizeCost,

    /// Balance cost and performance
    Balanced,

    /// Maximize performance within budget
    MaximizePerformance,
}

/// Quality requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    /// Minimum coverage required
    pub min_coverage: f64,

    /// Minimum mutation score required
    pub min_mutation_score: f64,

    /// Security scan required
    pub security_scan_required: bool,

    /// Manual review required
    pub manual_review_required: bool,

    /// Council approval required
    pub council_approval_required: bool,
}

/// Parallel execution preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelPreferences {
    /// Preferred maximum parallelism
    pub max_parallelism: usize,

    /// Prefer parallel over sequential
    pub prefer_parallel: bool,

    /// Allow resource contention
    pub allow_resource_contention: bool,

    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,
}

/// Load balancing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Even distribution
    Even,

    /// Workload-based balancing
    WorkloadBased,

    /// Capability-based assignment
    CapabilityBased,

    /// Custom strategy
    Custom,
}

/// Historical planning data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPlanningData {
    /// Previous similar plans
    pub similar_plans: Vec<HistoricalPlan>,

    /// Average execution times by milestone type
    pub avg_execution_times: HashMap<String, u64>,

    /// Success rates by planning strategy
    pub success_rates: HashMap<String, f64>,

    /// Common failure patterns
    pub failure_patterns: Vec<FailurePattern>,
}

/// Historical plan data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPlan {
    /// Plan ID
    pub plan_id: Uuid,

    /// Plan complexity score
    pub complexity_score: f64,

    /// Total execution time
    pub execution_time_ms: u64,

    /// Success status
    pub successful: bool,

    /// Planning strategy used
    pub strategy: String,

    /// Lessons learned
    pub lessons: Vec<String>,
}

/// Failure pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// Pattern description
    pub description: String,

    /// Frequency of occurrence
    pub frequency: usize,

    /// Impact severity
    pub severity: FailureSeverity,

    /// Mitigation strategies
    pub mitigations: Vec<String>,
}

/// Failure severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureSeverity {
    /// Low impact failures
    Low,

    /// Medium impact failures
    Medium,

    /// High impact failures
    High,

    /// Critical system failures
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_plan_extension() {
        let contract_plan = ContractExecutionPlan {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            working_spec_id: "test-spec".to_string(),
            title: "Test Plan".to_string(),
            overview: "Test overview".to_string(),
            state: ContractPlanState::Draft,
            milestones: vec![],
            dependency_graph: ContractDependencyGraph {
                nodes: HashMap::new(),
                edges: vec![],
                critical_path: vec![],
                parallel_groups: vec![],
                has_cycles: false,
                cycles: vec![],
            },
            change_budget: Default::default(),
            quality_gates: Default::default(),
            evidence_requirements: vec![],
            active_waivers: vec![],
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            approved_at: None,
            completed_at: None,
        };

        let plan = ExecutionPlan {
            contract_plan,
            orchestration_meta: OrchestrationMetadata {
                orchestrator_id: "test-orchestrator".to_string(),
                worker_pool_id: "test-pool".to_string(),
                council_session_id: Some("test-session".to_string()),
                audit_correlation_id: Uuid::new_v4(),
                planning_engine: "test-engine".to_string(),
                planning_version: "1.0.0".to_string(),
            },
            execution_context: ExecutionContext {
                session_start: Utc::now(),
                working_directory: "/tmp".to_string(),
                environment: HashMap::new(),
                available_resources: ResourceInventory {
                    available_cpu_cores: 4,
                    available_memory_mb: 8192,
                    available_disk_mb: 102400,
                    available_network_mbps: 100.0,
                    available_workers: HashMap::new(),
                },
                worker_assignments: HashMap::new(),
                parallel_batches: vec![],
            },
            execution_state: None,
        };

        assert_eq!(plan.orchestration_meta.orchestrator_id, "test-orchestrator");
        assert_eq!(plan.execution_context.available_resources.available_cpu_cores, 4);
    }

    #[test]
    fn test_worker_assignment_status() {
        let assignment = WorkerAssignment {
            worker_id: Uuid::new_v4(),
            assigned_at: Utc::now(),
            status: AssignmentStatus::Active,
            priority: AssignmentPriority::High,
            resource_allocation: ResourceAllocation {
                cpu_cores: 2,
                memory_mb: 1024,
                disk_mb: 5120,
                network_mbps: Some(50.0),
                time_limit_ms: Some(300000),
            },
        };

        assert!(matches!(assignment.status, AssignmentStatus::Active));
        assert!(matches!(assignment.priority, AssignmentPriority::High));
        assert_eq!(assignment.resource_allocation.cpu_cores, 2);
    }

    #[test]
    fn test_parallel_batch_execution() {
        let batch = ParallelBatch {
            batch_index: 0,
            milestone_ids: vec!["M1".to_string(), "M2".to_string()],
            started_at: Some(Utc::now()),
            completed_at: None,
            status: BatchStatus::Executing,
            resource_requirements: ResourceRequirements {
                total_cpu_cores: 4,
                peak_memory_mb: 2048,
                total_disk_mb: 10240,
                network_requirements: NetworkRequirements {
                    peak_bandwidth_mbps: 100.0,
                    external_services: vec![],
                    security_requirements: vec![],
                },
                estimated_duration_ms: 600000,
            },
        };

        assert_eq!(batch.batch_index, 0);
        assert_eq!(batch.milestone_ids.len(), 2);
        assert!(matches!(batch.status, BatchStatus::Executing));
        assert_eq!(batch.resource_requirements.total_cpu_cores, 4);
    }

    #[test]
    fn test_evidence_collection_state() {
        let evidence_state = EvidenceCollectionState {
            collected_evidence: HashMap::new(),
            collection_failures: vec![EvidenceFailure {
                milestone_id: "M1".to_string(),
                evidence_type: "test_results".to_string(),
                reason: "Test execution failed".to_string(),
                failed_at: Utc::now(),
                retry_count: 1,
                recoverable: true,
            }],
            validation_results: HashMap::new(),
            storage_locations: HashMap::new(),
        };

        assert_eq!(evidence_state.collection_failures.len(), 1);
        assert_eq!(evidence_state.collection_failures[0].milestone_id, "M1");
        assert!(evidence_state.collection_failures[0].recoverable);
    }
}
