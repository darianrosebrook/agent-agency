use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Task intake request schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    /// Unique request ID
    pub id: Uuid,
    /// Natural language task description
    pub description: String,
    /// Optional context information
    pub context: Option<TaskContext>,
    /// Optional constraints and requirements
    pub constraints: Option<TaskConstraints>,
    /// Requested risk tier (1-3, will be validated)
    pub risk_tier: Option<u8>,
    /// API version for compatibility
    pub api_version: String,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Task intake response schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    /// Unique task ID for tracking
    pub task_id: Uuid,
    /// Generated working spec
    pub working_spec: Option<WorkingSpec>,
    /// Current task status
    pub status: TaskStatus,
    /// URL for real-time tracking (WebSocket/HTTP)
    pub tracking_url: String,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// Error information if failed
    pub error: Option<TaskError>,
}

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task accepted and queued for planning
    Accepted,
    /// Planning in progress
    Planning,
    /// Working spec generated and validated
    SpecReady,
    /// Execution in progress
    Executing,
    /// Quality checking in progress
    QualityCheck,
    /// Awaiting manual approval
    AwaitingApproval,
    /// Refinement in progress
    Refining,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task cancelled
    Cancelled,
}

/// Task error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Working spec schema for CAWS compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    /// Unique spec identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Risk tier (1=Critical, 2=High, 3=Standard)
    pub risk_tier: u8,
    /// Scope boundaries
    pub scope: Option<WorkingSpecScope>,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Test plan
    pub test_plan: Option<TestPlan>,
    /// Rollback plan
    pub rollback_plan: Option<RollbackPlan>,
    /// Implementation constraints
    pub constraints: Vec<String>,
    /// Estimated effort in hours
    pub estimated_effort_hours: f64,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Context hash for provenance
    pub context_hash: String,
}

/// Working spec scope boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpecScope {
    /// Paths included in scope
    pub included: Vec<String>,
    /// Paths excluded from scope
    pub excluded: Vec<String>,
}

/// Acceptance criterion with priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
    pub priority: CriterionPriority,
}

/// Criterion priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionPriority {
    MustHave,
    ShouldHave,
    CouldHave,
}

/// Test plan specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlan {
    pub unit_tests: Vec<String>,
    pub integration_tests: Vec<String>,
    pub e2e_tests: Vec<String>,
    pub coverage_target: f64,
    pub mutation_score_target: f64,
}

/// Rollback plan specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub steps: Vec<String>,
    pub data_backup_required: bool,
    pub downtime_estimate: std::time::Duration,
    pub risk_level: RollbackRisk,
}

/// Rollback risk levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackRisk {
    Low,
    Medium,
    High,
    Critical,
}

/// Task context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub repository: Option<RepositoryContext>,
    pub team: Option<TeamContext>,
    pub technical: Option<TechnicalContext>,
}

/// Repository context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContext {
    pub name: String,
    pub description: Option<String>,
    pub primary_language: String,
    pub size_kb: u64,
    pub contributors: Vec<String>,
}

/// Team context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamContext {
    pub constraints: Vec<String>,
    pub preferences: Vec<String>,
    pub availability: Vec<String>,
}

/// Technical context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalContext {
    pub stack: TechStack,
    pub patterns: Vec<String>,
    pub constraints: Vec<String>,
}

/// Technology stack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub deployment: Vec<String>,
}

/// Task constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraints {
    pub time_budget_hours: Option<f64>,
    pub priority: Option<TaskPriority>,
    pub dependencies: Vec<String>,
    pub blockers: Vec<String>,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Execution artifacts schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionArtifacts {
    /// Unique artifact set ID
    pub id: Uuid,
    /// Associated task ID
    pub task_id: Uuid,
    /// Code changes and diffs
    pub code_changes: Vec<CodeChange>,
    /// Test execution results
    pub test_results: TestResults,
    /// Code coverage data
    pub coverage: CoverageReport,
    /// Mutation testing results
    pub mutation: MutationReport,
    /// Linting results
    pub lint: LintReport,
    /// Type checking results
    pub types: TypeCheckReport,
    /// Provenance and audit trail
    pub provenance: ProvenanceRecord,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Code change artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub change_type: ChangeType,
    pub diff: String,
    pub lines_added: u32,
    pub lines_removed: u32,
}

/// Type of code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Test execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub total: u32,
    pub duration_ms: u64,
    pub coverage_percentage: f64,
}

/// Code coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub lines_covered: u32,
    pub lines_total: u32,
    pub branches_covered: u32,
    pub branches_total: u32,
    pub functions_covered: u32,
    pub functions_total: u32,
    pub coverage_percentage: f64,
}

/// Mutation testing report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationReport {
    pub mutants_generated: u32,
    pub mutants_killed: u32,
    pub mutants_survived: u32,
    pub mutation_score: f64,
    pub duration_ms: u64,
}

/// Linting report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    pub errors: u32,
    pub warnings: u32,
    pub issues: Vec<LintIssue>,
}

/// Individual lint issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub rule: String,
    pub severity: LintSeverity,
    pub message: String,
}

/// Lint severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

/// Type checking report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCheckReport {
    pub errors: u32,
    pub warnings: u32,
    pub issues: Vec<TypeIssue>,
}

/// Individual type issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeIssue {
    pub file: String,
    pub line: u32,
    pub message: String,
    pub error_code: Option<String>,
}

/// Provenance and audit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub entries: Vec<ProvenanceEntry>,
    pub hash: String,
}

/// Individual provenance entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub details: serde_json::Value,
}

/// Quality report schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,
    /// Individual gate results
    pub gates: Vec<QualityGate>,
    /// Quality deltas from previous iteration
    pub deltas: Vec<QualityDelta>,
    /// Risk tier thresholds met
    pub tier_thresholds_met: bool,
    /// Satisficing criteria met
    pub satisficing_met: bool,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Individual quality gate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub name: String,
    pub status: GateStatus,
    pub score: f64,
    pub threshold: f64,
    pub details: serde_json::Value,
}

/// Quality gate status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

/// Quality delta from previous iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDelta {
    pub metric: String,
    pub previous_value: f64,
    pub current_value: f64,
    pub change: f64,
    pub trend: Trend,
}

/// Quality trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Declining,
    Stable,
}

/// Refinement decision schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementDecision {
    pub task_id: Uuid,
    pub iteration: u32,
    pub decision: RefinementAction,
    pub rationale: String,
    pub confidence: f64,
    pub council_votes: Vec<CouncilVote>,
    pub suggested_changes: Vec<String>,
    pub max_additional_iterations: u32,
    pub decided_at: DateTime<Utc>,
}

/// Type of refinement action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefinementAction {
    Accept,
    Refine,
    Reject,
}

/// Individual council member vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilVote {
    pub judge: String,
    pub vote: RefinementAction,
    pub rationale: String,
    pub confidence: f64,
}

/// Execution event types for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// Execution started
    ExecutionStarted {
        task_id: Uuid,
        working_spec_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Worker assignment started
    WorkerAssignmentStarted {
        task_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Worker assigned to task
    WorkerAssigned {
        task_id: Uuid,
        worker_id: Uuid,
        worker_name: String,
        timestamp: DateTime<Utc>,
    },

    /// Execution phase started
    ExecutionPhaseStarted {
        task_id: Uuid,
        phase: String,
        timestamp: DateTime<Utc>,
    },

    /// Execution progress update
    ExecutionProgress {
        task_id: Uuid,
        phase: String,
        progress_percentage: f32,
        message: String,
        timestamp: DateTime<Utc>,
    },

    /// Execution phase completed
    ExecutionPhaseCompleted {
        task_id: Uuid,
        phase: String,
        success: bool,
        timestamp: DateTime<Utc>,
    },

    /// Artifact produced
    ArtifactProduced {
        task_id: Uuid,
        artifact_path: String,
        artifact_type: String,
        size_bytes: u64,
        timestamp: DateTime<Utc>,
    },

    /// Quality check completed
    QualityCheckCompleted {
        task_id: Uuid,
        passed: bool,
        score: f64,
        issues: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    /// Execution completed successfully
    ExecutionCompleted {
        task_id: Uuid,
        success: bool,
        artifacts_summary: String,
        execution_time_ms: u64,
        timestamp: DateTime<Utc>,
    },

    /// Execution failed
    ExecutionFailed {
        task_id: Uuid,
        error: String,
        timestamp: DateTime<Utc>,
    },

    /// Self-prompting iteration started
    SelfPromptingIterationStarted {
        task_id: Uuid,
        iteration: usize,
        model_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Self-prompting evaluation completed
    SelfEvaluationCompleted {
        task_id: Uuid,
        iteration: usize,
        score: f64,
        status: String,
        should_continue: bool,
        timestamp: DateTime<Utc>,
    },

    /// Model swapped during execution
    ModelSwapped {
        task_id: Uuid,
        old_model: String,
        new_model: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Self-prompting loop completed
    SelfPromptingLoopCompleted {
        task_id: Uuid,
        total_iterations: usize,
        final_score: f64,
        stop_reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Arbiter adjudication started
    AdjudicationStarted {
        task_id: Uuid,
        output_count: usize,
        timestamp: DateTime<Utc>,
    },

    /// Arbiter adjudication completed
    AdjudicationCompleted {
        task_id: Uuid,
        verdict_status: String,
        confidence: f64,
        waiver_required: bool,
        timestamp: DateTime<Utc>,
    },
}
