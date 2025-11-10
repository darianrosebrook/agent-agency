//! Planning Types for Orchestration
//!
//! Extended types for the planning system that build on the contracts
//! but include orchestration-specific details and runtime state.
//!
//! @author @darianrosebrook

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use agent_agency_contracts::{
    planning_io::{
        ExecutionPlan as ContractExecutionPlan, PlanState as ContractPlanState,
        DependencyGraph as ContractDependencyGraph,
    },
    types::validation::ValidationResult,
    ChangeBudget, WorkingSpec,
};

/// Extended execution plan with orchestration state
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

impl Default for OrchestrationMetadata {
    fn default() -> Self {
        Self {
            orchestrator_id: "default-orchestrator".to_string(),
            worker_pool_id: "default-pool".to_string(),
            council_session_id: None,
            audit_correlation_id: Uuid::new_v4(),
            planning_engine: "default-engine".to_string(),
            planning_version: "1.0.0".to_string(),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            session_start: Utc::now(),
            working_directory: ".".to_string(),
            environment: HashMap::new(),
            available_resources: ResourceInventory::default(),
            worker_assignments: HashMap::new(),
            parallel_batches: Vec::new(),
        }
    }
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        Self {
            contract_plan: ContractExecutionPlan {
                id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                working_spec_id: "default-spec".to_string(),
                contract_plan: WorkingSpec {
                    version: "1.0".to_string(),
                    id: Uuid::new_v4().to_string(),
                    title: "Default Working Spec".to_string(),
                    description: "Default working specification".to_string(),
                    goals: vec!["default".to_string()],
                    risk_tier: 2,
                    constraints: agent_agency_contracts::WorkingSpecConstraints {
                        max_duration_minutes: Some(60),
                        max_iterations: Some(10),
                        budget_limits: None,
                        scope_restrictions: None,
                    },
                    acceptance_criteria: vec![],
                    test_plan: agent_agency_contracts::TestPlan {
                        unit_tests: vec![],
                        integration_tests: vec![],
                        e2e_scenarios: vec![],
                        coverage_targets: None,
                    },
                    rollback_plan: agent_agency_contracts::RollbackPlan::default(),
                    context: agent_agency_contracts::WorkingSpecContext {
                        workspace_root: ".".to_string(),
                        git_branch: "main".to_string(),
                        recent_changes: vec![],
                        dependencies: std::collections::HashMap::new(),
                        environment: agent_agency_contracts::Environment::Development,
                    },
                    non_functional_requirements: None,
                    validation_results: None,
                    quality_gates: Some(crate::planning::quality_gates::default_quality_gates()),
                    scope: vec![],
                    file_changes: vec![],
                    metadata: Some(agent_agency_contracts::WorkingSpecMetadata {
                        created_at: Utc::now(),
                        created_by: None,
                        last_modified: Some(Utc::now()),
                        version: Some(1),
                        tags: vec![],
                    }),
                    milestones: vec![],
                    change_budget: ChangeBudget {
                        max_files: 100,
                        max_loc: 1000,
                        max_migrations: 5,
                        allow_breaking_changes: false,
                        allow_new_dependencies: false,
                        enforcement_mode: agent_agency_contracts::BudgetEnforcement::Strict,
                    },
                    coverage_targets: None,
                    overview: "Default working specification".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                title: "Default Plan".to_string(),
                overview: "Default execution plan".to_string(),
                state: ContractPlanState::Draft,
                milestones: vec![],
                dependency_graph: ContractDependencyGraph {
                    nodes: HashMap::new(),
                    edges: vec![],
                    critical_path: vec![],
                    parallel_groups: vec![],
                    cycles: vec![],
                    has_cycles: false,
                },
                change_budget: agent_agency_contracts::ChangeBudget {
                    max_files: 100,
                    max_loc: 1000,
                    max_migrations: 5,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode: agent_agency_contracts::BudgetEnforcement::Strict,
                },
                quality_gates: crate::planning::quality_gates::default_quality_gates(),
                evidence_requirements: vec![],
                active_waivers: vec![],
                metadata: agent_agency_contracts::planning_io::PlanMetadata {
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    approved_at: None,
                    completed_at: None,
                    created_by: agent_agency_contracts::planning_io::PlanCreator::AI {
                        model: "default".to_string(),
                        version: "1.0".to_string(),
                    },
                    version: "1.0".to_string(),
                    source: "default".to_string(),
                    confidence_score: Some(0.8),
                    generation_time_ms: Some(1000),
                    model_used: Some("default-model".to_string()),
                    fallback_used: false,
                    strategy: agent_agency_contracts::types::planning::PlanningStrategy::Hybrid,
                    confidence: 0.8,
                    estimated_duration_ms: 60000,
                    estimated_cost_cents: 100,
                    adaptive: false,
                    engine_version: "1.0".to_string(),
                    additional_metadata: std::collections::HashMap::new(),
                },
                execution_context: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                approved_at: None,
                completed_at: None,
            },
            orchestration_meta: OrchestrationMetadata::default(),
            execution_context: ExecutionContext::default(),
            execution_state: None,
        }
    }
}

/// Orchestration-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrationMetadata {
    /// Orchestrator instance that created this plan
    pub orchestrator_id: String,

    /// Worker pool assigned to this plan
    pub worker_pool_id: String,

    /// Council session that approved this plan
    pub council_session_id: Option<String>,

    /// Audit trail correlation ID
    #[schemars(with = "String")]
    pub audit_correlation_id: Uuid,

    /// Planning engine used
    pub planning_engine: String,

    /// Planning version
    pub planning_version: String,
}

/// Current execution context
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionContext {
    /// Session start time
    #[schemars(with = "String")]

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerAssignment {
    /// Worker ID assigned
    #[schemars(with = "String")]
    pub worker_id: Uuid,

    /// Assigned at timestamp
    #[schemars(with = "String")]

    pub assigned_at: DateTime<Utc>,

    /// Assignment status
    pub status: AssignmentStatus,

    /// Assignment priority
    pub priority: AssignmentPriority,

    /// Resource allocation for this assignment
    pub resource_allocation: ResourceAllocation,
}

/// Assignment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParallelBatch {
    /// Batch index
    pub batch_index: usize,

    /// Milestone IDs in this batch
    pub milestone_ids: Vec<String>,

    /// Batch start time
    #[schemars(with = "Option<String>")]
    pub started_at: Option<DateTime<Utc>>,

    /// Batch completion time
    #[schemars(with = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Batch status
    pub status: BatchStatus,

    /// Resource requirements for batch
    pub resource_requirements: ResourceRequirements,
}

/// Batch execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum BatchStatus {
    /// Batch queued for execution
    Queued,

    /// Batch pending additional requirements
    Pending,

    /// Batch currently executing
    Executing,

    /// Batch completed successfully
    Completed,

    /// Batch partially completed (some tasks succeeded, some failed)
    PartiallyCompleted,

    /// Batch failed
    Failed,

    /// Batch cancelled
    Cancelled,
}

// ResourceRequirements and NetworkRequirements are now imported from agent_agency_contracts
// Type alias for backward compatibility
pub type ResourceRequirements = agent_agency_contracts::HardwareResourceRequirements;

/// Resource inventory available
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

impl Default for ResourceInventory {
    fn default() -> Self {
        Self {
            available_cpu_cores: 8,
            available_memory_mb: 16384, // 16GB
            available_disk_mb: 102400, // 100GB
            available_network_mbps: 1000.0, // 1Gbps
            available_workers: HashMap::new(),
        }
    }
}

/// Execution progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceStatus {
    /// Evidence type
    pub evidence_type: String,

    /// Collection status
    pub status: EvidenceCollectionStatus,

    /// Collection start time
    #[schemars(with = "String")]

    pub started_at: DateTime<Utc>,

    /// Collection completion time
    #[schemars(with = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Evidence size (bytes)
    pub size_bytes: Option<u64>,

    /// Evidence quality score (0.0-1.0)
    pub quality_score: Option<f64>,
}

/// Evidence collection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceFailure {
    /// Milestone ID
    pub milestone_id: String,

    /// Evidence type that failed
    pub evidence_type: String,

    /// Failure reason
    pub reason: String,

    /// Failure timestamp
    #[schemars(with = "String")]

    pub failed_at: DateTime<Utc>,

    /// Retry count
    pub retry_count: u32,

    /// Whether failure is recoverable
    pub recoverable: bool,
}

/// Evidence bundle collected for a milestone
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceBundle {
    /// Milestone ID this evidence is for
    pub milestone_id: String,

    /// Plan ID this evidence belongs to
    #[schemars(with = "String")]
    pub plan_id: Uuid,

    /// Collection timestamp
    #[schemars(with = "String")]

    pub collected_at: DateTime<Utc>,

    /// Evidence artifacts
    pub artifacts: Vec<EvidenceArtifact>,

    /// Quality score of evidence
    pub quality_score: Option<f64>,

    /// Whether evidence meets quality gates
    pub meets_quality_gates: bool,

    /// Collection metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Individual evidence artifact
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceArtifact {
    /// Artifact ID
    #[schemars(with = "String")]
    pub id: Uuid,

    /// Artifact type (test_results, coverage, security_scan, etc.)
    pub artifact_type: String,

    /// Artifact path or content
    pub content: EvidenceContent,

    /// Artifact quality score
    pub quality_score: f64,

    /// Collection timestamp
    #[schemars(with = "String")]

    pub collected_at: DateTime<Utc>,

    /// Artifact metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Evidence content types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EvidenceContent {
    /// File path to evidence
    FilePath(String),

    /// Inline content as JSON
    InlineJson(serde_json::Value),

    /// Inline content as text
    InlineText(String),

    /// Structured data (HashMap/JSON object)
    Structured(std::collections::HashMap<String, serde_json::Value>),

    /// Binary data (base64 encoded)
    Binary(String),
}

/// Planning strategy options

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanGenerationStrategy {
    /// Use AI-assisted planning with human oversight
    AIAssisted,
    /// Use fully automated planning
    Automated,
    /// Use human-guided planning
    HumanGuided,
}

/// Plan generation context
pub struct PlanGenerationContext {
    /// Working spec to plan for
    pub working_spec_provider: Box<dyn WorkingSpecProvider>,

    /// Task descriptor
    pub task_descriptor: Box<dyn TaskDescriptorProvider>,

    /// Available resources
    pub resource_inventory: ResourceInventory,

    /// Planning constraints
    pub constraints: PlanningConstraints,

    /// Historical planning data
    pub historical_data: Option<HistoricalPlanningData>,

    /// Planning constraints (alias for constraints - added for compatibility)
    pub planning_constraints: PlanningConstraints,

    /// Execution mode for this planning operation
    pub execution_mode: agent_agency_contracts::types::planning::ExecutionMode,

    /// Planning strategy to use
    pub planning_strategy: PlanGenerationStrategy,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

impl Default for PlanningConstraints {
    fn default() -> Self {
        Self {
            max_planning_time_ms: 300000, // 5 minutes
            max_complexity: 100,
            risk_tolerance: RiskTolerance::Balanced,
            cost_limits: None,
            quality_requirements: QualityRequirements::default(),
            parallel_preferences: ParallelPreferences::default(),
        }
    }
}

/// Risk tolerance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RiskTolerance {
    /// Conservative - prefer proven approaches
    Conservative,

    /// Balanced - mix of proven and experimental
    Balanced,

    /// Aggressive - allow experimental approaches
    Aggressive,
}

/// Cost limits
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CostLimits {
    /// Maximum cost in cents
    pub max_cost_cents: u32,

    /// Cost per millisecond budget
    pub cost_per_ms_budget: f64,

    /// Cost optimization priority
    pub optimization_priority: CostOptimizationPriority,
}

/// Cost optimization priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CostOptimizationPriority {
    /// Minimize cost
    MinimizeCost,

    /// Balance cost and performance
    Balanced,

    /// Maximize performance within budget
    MaximizePerformance,
}

/// Quality requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_coverage: 0.8, // 80% coverage
            min_mutation_score: 0.5, // 50% mutation score
            security_scan_required: true,
            manual_review_required: false,
            council_approval_required: false,
        }
    }
}

/// Parallel execution preferences
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

impl Default for ParallelPreferences {
    fn default() -> Self {
        Self {
            max_parallelism: 4,
            prefer_parallel: true,
            allow_resource_contention: false,
            load_balancing: LoadBalancingStrategy::Even,
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalPlan {
    /// Plan ID
    #[schemars(with = "String")]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
            contract_plan: WorkingSpec {
                version: "1.0".to_string(),
                id: "test-spec".to_string(),
                title: "Test Spec".to_string(),
                description: "Test description".to_string(),
                goals: vec![],
                risk_tier: 2,
                constraints: agent_agency_contracts::WorkingSpecConstraints {
                    max_duration_minutes: None,
                    max_iterations: None,
                    budget_limits: None,
                    scope_restrictions: None,
                },
                acceptance_criteria: vec![],
                test_plan: agent_agency_contracts::TestPlan {
                    unit_tests: vec![],
                    integration_tests: vec![],
                    e2e_scenarios: vec![],
                    coverage_targets: None,
                },
                rollback_plan: agent_agency_contracts::RollbackPlan {
                    strategy: agent_agency_contracts::RollbackStrategy::GitRevert,
                    automated_steps: vec![],
                    manual_steps: vec![],
                    data_impact: agent_agency_contracts::DataImpact::None,
                    downtime_required: None,
                    rollback_window_minutes: None,
                },
                context: agent_agency_contracts::WorkingSpecContext {
                    workspace_root: "/tmp".to_string(),
                    git_branch: "main".to_string(),
                    recent_changes: vec![],
                    dependencies: std::collections::HashMap::new(),
                    environment: agent_agency_contracts::task_request::Environment::Development,
                },
                non_functional_requirements: None,
                validation_results: None,
                quality_gates: None,
                scope: vec![],
                metadata: None,
                milestones: vec![],
                change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                    max_files: 10,
                    max_loc: 100,
                    max_migrations: 0,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                },
                file_changes: vec![],
                coverage_targets: None,
                overview: "Test overview".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            title: "Test Plan".to_string(),
            overview: "Test overview".to_string(),
            state: ContractPlanState::Draft,
            milestones: vec![],
            evidence_requirements: vec![],
            active_waivers: vec![],
            dependency_graph: ContractDependencyGraph {
                nodes: HashMap::new(),
                edges: vec![],
                critical_path: vec![],
                parallel_groups: vec![],
                has_cycles: false,
                cycles: vec![],
            },
                change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                    max_files: 10,
                    max_loc: 100,
                    max_migrations: 0,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                },
                quality_gates: agent_agency_contracts::planning_io::QualityGates {
                    coverage_requirements: std::collections::HashMap::new(),
                    mutation_requirements: agent_agency_contracts::planning_io::MutationRequirements {
                        required: false,
                        min_score: 0.0,
                        operators: vec![],
                    },
                    security_requirements: agent_agency_contracts::planning_io::SecurityRequirements {
                        scan_required: false,
                        max_issues_by_severity: std::collections::HashMap::new(),
                        required_controls: vec![],
                    },
                    performance_requirements: agent_agency_contracts::planning_io::PerformanceRequirements {
                        max_regressions: 0,
                        required_benchmarks: vec![],
                        slas: vec![],
                    },
                    documentation_requirements: agent_agency_contracts::planning_io::DocumentationRequirements {
                        api_docs_required: false,
                        code_docs_required: false,
                        architecture_docs_required: false,
                        required_formats: vec![],
                        required_types: vec![],
                        min_coverage: 0.0,
                        quality_checks: vec![],
                    },
                    requires_manual_review: false,
                    requires_council_approval: false,
                    min_coverage: None,
                    min_mutation_score_percent: None,
                },
                metadata: agent_agency_contracts::planning_io::PlanMetadata {
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    approved_at: None,
                    completed_at: None,
                    created_by: agent_agency_contracts::planning_io::PlanCreator::AI {
                        model: "default-model".to_string(),
                        version: "1.0".to_string(),
                    },
                    version: "1.0".to_string(),
                    source: "default".to_string(),
                    confidence_score: Some(0.5),
                    generation_time_ms: Some(100),
                    model_used: Some("default-model".to_string()),
                    fallback_used: false,
                    strategy: agent_agency_contracts::types::planning::PlanningStrategy::AIAssisted,
                    confidence: 0.5,
                    estimated_duration_ms: 0,
                    estimated_cost_cents: 0,
                    adaptive: false,
                    engine_version: "1.0".to_string(),
                    additional_metadata: std::collections::HashMap::new(),
                },
                execution_context: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
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
                network_requirements: agent_agency_contracts::planning::NetworkRequirements {
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

/// Request for plan generation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanGenerationRequest {
    /// Working spec to generate plan for
    pub working_spec: agent_agency_contracts::WorkingSpec,

    /// Planning context and constraints
    pub planning_context: PlanningContext,

    /// Optional planning session to continue
    pub existing_session: Option<PlanningSession>,
}

/// Planning context with orchestration details
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningContext {
    /// Available workers and their capabilities
    pub worker_capabilities: HashMap<String, WorkerCapabilities>,

    /// Current system resource availability
    pub system_resources: SystemResources,

    /// Planning constraints and preferences
    pub planning_constraints: ExecutionPlanningConstraints,
}

/// Worker capabilities for planning
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerCapabilities {
    /// Worker specialization
    pub specialization: Vec<String>,

    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,

    /// Supported task types
    pub supported_task_types: Vec<String>,

    /// Performance characteristics
    pub performance_profile: PerformanceProfile,
}

/// System resource availability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemResources {
    /// Total available CPU cores
    pub total_cpu_cores: usize,

    /// Total available memory in MB
    pub total_memory_mb: usize,

    /// Total available disk space in MB
    pub total_disk_mb: usize,

    /// Network bandwidth in Mbps
    pub network_bandwidth_mbps: f64,
}

/// Execution planning constraints
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPlanningConstraints {
    /// Maximum plan execution time in minutes
    pub max_execution_minutes: Option<u32>,

    /// Maximum parallel execution limit
    pub max_parallel_execution: Option<usize>,

    /// Required quality gates
    pub required_quality_gates: Vec<String>,

    /// Resource allocation preferences
    pub resource_preferences: ResourcePreferences,
}

impl Default for ExecutionPlanningConstraints {
    fn default() -> Self {
        Self {
            max_execution_minutes: Some(60),
            max_parallel_execution: Some(3),
            required_quality_gates: vec![],
            resource_preferences: ResourcePreferences::default(),
        }
    }
}

/// Resource allocation preferences
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourcePreferences {
    /// Prefer CPU-intensive tasks
    pub prefer_cpu_intensive: bool,

    /// Prefer memory-intensive tasks
    pub prefer_memory_intensive: bool,

    /// Allow network-heavy tasks
    pub allow_network_heavy: bool,

    /// Require fast storage access
    pub require_fast_storage: bool,
}

impl Default for ResourcePreferences {
    fn default() -> Self {
        Self {
            prefer_cpu_intensive: false,
            prefer_memory_intensive: false,
            allow_network_heavy: true,
            require_fast_storage: false,
        }
    }
}

/// Performance profile for workers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceProfile {
    /// Average task completion time in seconds
    pub avg_completion_seconds: f64,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Resource efficiency score (0.0 to 1.0)
    pub resource_efficiency: f64,

    /// Specialization match score (0.0 to 1.0)
    pub specialization_score: f64,
}

/// Planning session state
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningSession {
    /// Session unique identifier
    #[schemars(with = "String")]
    pub session_id: Uuid,

    /// Working spec being planned
    pub working_spec: agent_agency_contracts::WorkingSpec,

    /// Planning start time
    #[schemars(with = "String")]

    pub started_at: DateTime<Utc>,

    /// Current planning phase
    pub current_phase: PlanningPhase,

    /// Planning progress (0.0 to 1.0)
    pub progress: f64,

    /// Generated execution plan (when complete)
    pub execution_plan: Option<ExecutionPlan>,

    /// Planning metrics and telemetry
    pub metrics: PlanningMetrics,
}

/// Planning phase enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanningPhase {
    /// Initial analysis phase
    Analysis,
    /// Milestone decomposition
    Decomposition,
    /// Dependency analysis
    DependencyAnalysis,
    /// Resource allocation
    ResourceAllocation,
    /// Quality gate validation
    Validation,
    /// Final optimization
    Optimization,
    /// Plan generation complete
    Complete,
}

/// Planning metrics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningMetrics {
    /// Total planning time in milliseconds
    pub total_time_ms: u64,

    /// Time spent in each phase
    pub phase_times_ms: HashMap<String, u64>,

    /// Number of milestones generated
    pub milestones_generated: usize,

    /// Number of dependencies identified
    pub dependencies_identified: usize,

    /// Resource allocation efficiency (0.0 to 1.0)
    pub resource_efficiency: f64,

    /// Planning quality score (0.0 to 1.0)
    pub quality_score: f64,
}

/// Todo integration for task management
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoIntegration {
    /// Integration identifier
    #[schemars(with = "String")]
    pub integration_id: Uuid,

    /// Todo system type
    pub system_type: TodoSystemType,

    /// Connection configuration
    pub connection_config: TodoConnectionConfig,

    /// Synchronization settings
    pub sync_settings: TodoSyncSettings,

    /// Current sync state
    pub sync_state: TodoSyncState,
}

/// Todo system types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TodoSystemType {
    /// GitHub Issues
    GitHub,
    /// Jira
    Jira,
    /// Linear
    Linear,
    /// Trello
    Trello,
    /// Asana
    Asana,
    /// Custom system
    Custom,
}

/// Todo connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoConnectionConfig {
    /// API endpoint URL
    pub endpoint_url: String,

    /// Authentication token/API key
    pub auth_token: String,

    /// Project/repository identifier
    pub project_id: String,

    /// Additional configuration
    pub additional_config: HashMap<String, String>,
}

/// Todo synchronization settings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoSyncSettings {
    /// Sync direction
    pub sync_direction: SyncDirection,

    /// Auto-sync enabled
    pub auto_sync: bool,

    /// Sync interval in minutes
    pub sync_interval_minutes: u32,

    /// Field mappings
    pub field_mappings: HashMap<String, String>,

    /// Status mappings
    pub status_mappings: HashMap<String, String>,
}

/// Sync direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SyncDirection {
    /// Bidirectional sync
    Bidirectional,
    /// Only push to todo system
    PushOnly,
    /// Only pull from todo system
    PullOnly,
}

/// Todo synchronization state
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoSyncState {
    /// Last successful sync time
    #[schemars(with = "Option<String>")]
    pub last_sync_at: Option<DateTime<Utc>>,

    /// Sync status
    pub status: SyncStatus,

    /// Items synchronized
    pub synced_items: usize,

    /// Failed sync attempts
    pub failed_sync_attempts: usize,

    /// Last error message
    pub last_error: Option<String>,
}

/// Synchronization status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SyncStatus {
    /// Never synchronized
    NeverSynced,
    /// Currently synchronizing
    Syncing,
    /// Last sync successful
    Synced,
    /// Last sync failed
    Failed,
    /// Sync disabled
    Disabled,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceUtilization {
    /// CPU utilization percentage (0.0 to 100.0)
    pub cpu_percent: f64,

    /// Memory utilization in MB
    pub memory_mb: f64,

    /// Disk utilization in MB
    pub disk_mb: f64,

    /// Network utilization in Mbps
    pub network_mbps: f64,

    /// GPU utilization if available
    pub gpu_percent: Option<f64>,

    /// Timestamp of measurement
    #[schemars(with = "String")]

    pub measured_at: DateTime<Utc>,

    /// Associated milestone or task
    pub associated_with: Option<String>,
}

// Quality metrics types are now imported from agent_agency_contracts
