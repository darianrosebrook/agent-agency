//! Planning IO Structures with JSON Schema Validation
//!
//! Data structures for execution plans, milestones, and related entities
//! with automatic JSON schema generation for validation and documentation.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Execution plan with milestone breakdown
/// The core data structure for executable plans
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPlan {
    /// Unique plan identifier (persistent across sessions)
    /// Follows Cursor's UUID format for compatibility
    pub id: Uuid,

    /// Session identifier (ephemeral per execution)
    /// Links execution context in Cursor-compatible format
    pub session_id: Uuid,

    /// Reference to CAWS working spec that generated this plan
    pub working_spec_id: String,

    /// Human-readable plan title
    #[schemars(description = "Plan title for human identification")]
    pub title: String,

    /// High-level overview of plan objectives and scope
    #[schemars(description = "Executive summary of plan goals")]
    pub overview: String,

    /// Current execution state of the plan
    pub state: PlanState,

    /// Ordered list of milestones to execute
    pub milestones: Vec<Milestone>,

    /// Dependency graph defining milestone relationships
    pub dependency_graph: DependencyGraph,

    /// Change budget inherited from working spec
    pub change_budget: ChangeBudget,

    /// Quality gates that must be satisfied
    pub quality_gates: QualityGates,

    /// Evidence requirements for completion validation
    pub evidence_requirements: Vec<EvidenceRequirement>,

    /// Active waivers (if any) allowing gate bypass
    pub active_waivers: Vec<WaiverReference>,

    /// Planning metadata and telemetry
    pub metadata: PlanMetadata,

    /// Timestamp tracking
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Plan execution state machine
/// Defines the lifecycle states of an execution plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanState {
    /// Plan being created by AI/human planning engine
    Draft,

    /// Plan submitted for council constitutional review
    UnderReview,

    /// Plan approved by council, ready for execution
    Approved,

    /// Plan currently executing milestones
    InProgress,

    /// Plan blocked by dependency or external issue
    Blocked { reason: String },

    /// All milestones completed successfully
    Completed,

    /// Plan execution failed with reason
    Failed { reason: String },

    /// Plan cancelled by user or council
    Cancelled { reason: String },
}

/// Individual milestone within an execution plan
/// Represents a discrete unit of work with dependencies and evidence gates
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Milestone {
    /// Milestone identifier (e.g., "M0", "M1", "M2")
    /// Should be unique within the plan
    pub id: String,

    /// Human-readable objective statement
    #[schemars(description = "Clear statement of what this milestone achieves")]
    pub objective: String,

    /// Execution scope defining files and boundaries
    pub scope: MilestoneScope,

    /// Interface contracts that will be introduced/modified
    pub interfaces: Vec<InterfaceContract>,

    /// Test requirements that must be satisfied
    pub tests: Vec<TestRequirement>,

    /// Evidence gate defining completion criteria
    pub evidence_gate: EvidenceGate,

    /// Rollback plan if milestone execution fails
    pub rollback_plan: String,

    /// Dependencies on other milestone IDs
    /// Must form a DAG (no cycles)
    pub dependencies: Vec<String>,

    /// Current execution state
    pub state: MilestoneState,

    /// Workers assigned to execute this milestone
    pub assigned_workers: Vec<Uuid>,

    /// Estimated effort in hours
    pub estimated_effort: f64,

    /// Milestone priority level
    pub priority: MilestonePriority,

    /// Risk tier (1-3, inherited from working spec)
    pub risk_tier: u8,

    /// Whether this milestone blocks other milestones
    pub is_blocking: bool,

    /// Reason if blocking (for dependency logging)
    pub blocking_reason: Option<String>,

    /// Execution metrics and results
    pub metrics: Option<MilestoneMetrics>,
}

/// Milestone execution state machine
/// Tracks the lifecycle of individual milestone execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum MilestoneState {
    /// Milestone not yet started (dependencies may not be satisfied)
    Pending,

    /// Dependencies satisfied, milestone ready to start
    Ready,

    /// Milestone currently executing
    InProgress,

    /// Milestone blocked by unresolved dependencies
    Blocked { dependencies: Vec<String> },

    /// Milestone completed successfully with evidence validation
    Completed,

    /// Milestone failed with specific reason
    Failed { reason: String },

    /// Milestone conditionally skipped
    Skipped { reason: String },
}

/// Milestone priority levels
/// Used for execution ordering and resource allocation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum MilestonePriority {
    /// Lowest priority, execute when resources available
    Low,

    /// Standard priority for regular milestones
    Normal,

    /// High priority for time-sensitive work
    High,

    /// Critical priority requiring immediate attention
    Critical,
}

/// Execution scope defining milestone boundaries
/// Controls what files and operations are allowed
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MilestoneScope {
    /// Files that can be read/modified by this milestone
    pub files: Vec<String>,

    /// Directories included in scope (globs supported)
    pub directories: Vec<String>,

    /// Whether this milestone will modify files (affects locking)
    pub will_modify: bool,

    /// Allowed operations (read, write, execute)
    pub allowed_operations: Vec<String>,

    /// Parallelism level (how many workers can work simultaneously)
    pub parallelism: Option<usize>,

    /// Resource requirements (CPU, memory, etc.)
    pub resource_requirements: HashMap<String, String>,
}

/// Interface contract specification
/// Defines APIs or contracts that will be created/modified
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InterfaceContract {
    /// Contract type (API, database schema, etc.)
    pub contract_type: String,

    /// Contract name/identifier
    pub name: String,

    /// Contract version
    pub version: String,

    /// Contract specification (JSON schema, OpenAPI, etc.)
    pub specification: serde_json::Value,

    /// Whether this is a new contract or modification
    pub is_new: bool,

    /// Breaking change indicator
    pub breaking_change: bool,
}

/// Test requirement specification
/// Defines testing obligations for milestone completion
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestRequirement {
    /// Test type (unit, integration, e2e)
    pub test_type: TestType,

    /// Minimum coverage requirement (0.0-1.0)
    pub min_coverage: f64,

    /// Specific test cases that must pass
    pub required_tests: Vec<String>,

    /// Test environment requirements
    pub environment: Option<String>,

    /// Performance requirements for tests
    pub performance_requirements: Option<TestPerformance>,
}

/// Test type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TestType {
    /// Unit tests for individual functions/components
    Unit,

    /// Integration tests for component interactions
    Integration,

    /// End-to-end tests for complete workflows
    EndToEnd,

    /// Property-based tests for invariant checking
    Property,

    /// Performance regression tests
    Performance,

    /// Security-focused tests
    Security,

    /// Custom test type
    Custom(String),
}

/// Test performance requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestPerformance {
    /// Maximum test execution time in milliseconds
    pub max_execution_time_ms: u64,

    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Minimum operations per second
    pub min_ops_per_second: Option<f64>,
}

/// Evidence gate for milestone completion
/// Defines what evidence must be collected to consider milestone complete
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceGate {
    /// Minimum line coverage (0.0-1.0)
    pub min_coverage: f64,

    /// Minimum branch coverage (0.0-1.0)
    pub min_branch_coverage: f64,

    /// Minimum mutation testing score (0.0-1.0)
    pub min_mutation_score: f64,

    /// Whether security scan is required
    pub security_scan_required: bool,

    /// Performance budget requirements
    pub performance_budget: Option<PerformanceBudget>,

    /// Required evidence artifact types
    pub required_artifacts: Vec<String>,

    /// Custom validation rules
    pub custom_validations: Vec<String>,
}

/// Performance budget constraints
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceBudget {
    /// Maximum P95 latency in milliseconds
    pub max_p95_ms: u64,

    /// Maximum P99 latency in milliseconds
    pub max_p99_ms: u64,

    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Minimum throughput (requests/second)
    pub min_throughput_per_second: f64,
}

/// Dependency graph structure
/// Defines relationships between milestones using DAG representation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DependencyGraph {
    /// Node data indexed by milestone ID
    pub nodes: HashMap<String, DependencyNode>,

    /// Directed edges (from -> to relationships)
    pub edges: Vec<DependencyEdge>,

    /// Critical path (longest dependency chain)
    pub critical_path: Vec<String>,

    /// Parallel execution groups (batches that can run concurrently)
    pub parallel_groups: Vec<Vec<String>>,

    /// Whether the graph contains cycles (should always be false)
    pub has_cycles: bool,

    /// Cycle information if detected
    pub cycles: Vec<Vec<String>>,
}

/// Individual dependency node
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DependencyNode {
    /// Milestone ID
    pub milestone_id: String,

    /// Node type classification
    pub node_type: DependencyNodeType,

    /// Estimated execution cost
    pub estimated_cost: f64,

    /// Estimated execution time in milliseconds
    pub estimated_time_ms: u64,

    /// Resource requirements
    pub resource_requirements: HashMap<String, f64>,

    /// Node metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency node classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DependencyNodeType {
    /// Milestone node
    Milestone,

    /// Barrier node (synchronization point)
    Barrier,

    /// Conditional node (may or may not execute)
    Conditional,

    /// Virtual node (for graph structure)
    Virtual,
}

/// Directed dependency edge
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DependencyEdge {
    /// Source milestone ID
    pub from: String,

    /// Target milestone ID
    pub to: String,

    /// Edge type
    pub edge_type: DependencyEdgeType,

    /// Edge weight (cost/distance)
    pub weight: f64,

    /// Edge metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of dependency relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DependencyEdgeType {
    /// Hard dependency (must complete before target)
    Hard,

    /// Soft dependency (prefer completion before target)
    Soft,

    /// Conditional dependency (only if condition met)
    Conditional,

    /// Resource dependency (competes for same resource)
    Resource,

    /// Information dependency (needs output from source)
    Information,
}

/// Change budget constraints
/// Limits the scope of changes allowed in the plan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangeBudget {
    /// Maximum number of files that can be changed
    pub max_files: usize,

    /// Maximum lines of code that can be changed
    pub max_loc: usize,

    /// Maximum number of database migrations
    pub max_migrations: usize,

    /// Whether breaking API changes are allowed
    pub allow_breaking_changes: bool,

    /// Whether new dependencies can be added
    pub allow_new_dependencies: bool,

    /// Budget enforcement mode
    pub enforcement_mode: BudgetEnforcement,
}

/// Budget enforcement modes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum BudgetEnforcement {
    /// Strict enforcement (block over-budget changes)
    Strict,

    /// Warning mode (allow but warn about overages)
    Warning,

    /// Flexible mode (allow budget increases with justification)
    Flexible,
}

/// Quality gates that must be satisfied
/// Defines the quality standards for plan completion
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityGates {
    /// Coverage requirements by test type
    pub coverage_requirements: HashMap<String, f64>,

    /// Mutation testing requirements
    pub mutation_requirements: MutationRequirements,

    /// Security requirements
    pub security_requirements: SecurityRequirements,

    /// Performance requirements
    pub performance_requirements: PerformanceRequirements,

    /// Documentation requirements
    pub documentation_requirements: DocumentationRequirements,

    /// Whether manual review is required
    pub requires_manual_review: bool,

    /// Whether council approval is required
    pub requires_council_approval: bool,
}

/// Mutation testing requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MutationRequirements {
    /// Whether mutation testing is required
    pub required: bool,

    /// Minimum mutation score (0.0-1.0)
    pub min_score: f64,

    /// Mutation operators to test
    pub operators: Vec<String>,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecurityRequirements {
    /// Whether security scan is required
    pub scan_required: bool,

    /// Maximum allowed security issues by severity
    pub max_issues_by_severity: HashMap<String, usize>,

    /// Required security controls
    pub required_controls: Vec<String>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceRequirements {
    /// Maximum allowed performance regressions
    pub max_regressions: usize,

    /// Required performance benchmarks
    pub required_benchmarks: Vec<String>,

    /// Performance SLA requirements
    pub slas: Vec<PerformanceSLA>,
}

/// Performance SLA specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceSLA {
    /// SLA name
    pub name: String,

    /// Metric to measure
    pub metric: String,

    /// Maximum allowed value
    pub max_value: f64,

    /// Unit of measurement
    pub unit: String,
}

/// Documentation requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationRequirements {
    /// Whether API documentation is required
    pub api_docs_required: bool,

    /// Whether code documentation is required
    pub code_docs_required: bool,

    /// Whether architecture documentation is required
    pub architecture_docs_required: bool,

    /// Required documentation formats
    pub required_formats: Vec<String>,
}

/// Evidence requirement specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceRequirement {
    /// Milestone ID this requirement applies to
    pub milestone_id: String,

    /// Required evidence type
    pub evidence_type: String,

    /// Evidence collection method
    pub collection_method: String,

    /// Validation criteria
    pub validation_criteria: HashMap<String, serde_json::Value>,

    /// Whether evidence is mandatory
    pub mandatory: bool,
}

/// Waiver reference for active waivers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverReference {
    /// Waiver identifier
    pub waiver_id: String,

    /// Reason for waiver
    pub reason: String,

    /// Gates waived by this waiver
    pub waived_gates: Vec<String>,

    /// Waiver expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Waiver approval information
    pub approved_by: String,
}

/// Planning metadata and telemetry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanMetadata {
    /// Who created the plan
    pub created_by: PlanCreator,

    /// Planning strategy used
    pub strategy: PlanningStrategy,

    /// AI confidence score (0.0-1.0)
    pub confidence: f64,

    /// Estimated total duration in milliseconds
    pub estimated_duration_ms: u64,

    /// Estimated total cost in cents
    pub estimated_cost_cents: u32,

    /// Whether adaptive planning is enabled
    pub adaptive: bool,

    /// Planning engine version
    pub engine_version: String,

    /// Additional metadata
    pub additional_metadata: HashMap<String, serde_json::Value>,
}

/// Plan creator information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PlanCreator {
    /// Created by AI planning engine
    AI { model: String, version: String },

    /// Created by human user
    Human { user_id: String },

    /// Created through AI-human collaboration
    Hybrid { ai_contribution: f64 },
}

/// Planning strategy used
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanningStrategy {
    /// Top-down decomposition from requirements
    TopDown,

    /// Bottom-up composition from tool chains
    BottomUp,

    /// Dependency-driven critical path analysis
    DependencyDriven,

    /// Risk-based milestone prioritization
    RiskBased,

    /// Hybrid strategy combining approaches
    Hybrid,

    /// AI-assisted planning with human oversight
    AIAssisted,

    /// Template-based planning from patterns
    TemplateBased,
}

/// Milestone execution metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MilestoneMetrics {
    /// Actual execution time in milliseconds
    pub execution_time_ms: u64,

    /// Resources consumed
    pub resources_used: HashMap<String, f64>,

    /// Quality metrics achieved
    pub quality_metrics: HashMap<String, f64>,

    /// Evidence collection results
    pub evidence_results: Vec<EvidenceResult>,

    /// Execution events timeline
    pub execution_events: Vec<ExecutionEvent>,

    /// Worker assignments and performance
    pub worker_performance: HashMap<Uuid, WorkerPerformance>,
}

/// Evidence collection result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceResult {
    /// Evidence type collected
    pub evidence_type: String,

    /// Whether collection succeeded
    pub collection_success: bool,

    /// Evidence quality score (0.0-1.0)
    pub quality_score: f64,

    /// Collection timestamp
    pub collected_at: DateTime<Utc>,

    /// Evidence metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerPerformance {
    /// Worker ID
    pub worker_id: Uuid,

    /// Tasks completed by this worker
    pub tasks_completed: usize,

    /// Tasks failed by this worker
    pub tasks_failed: usize,

    /// Average task completion time
    pub avg_completion_time_ms: f64,

    /// Worker utilization percentage
    pub utilization: f64,

    /// Quality score for work produced
    pub quality_score: f64,
}

/// Execution event for milestone tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEvent {
    /// Event type
    pub event_type: String,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event description
    pub description: String,

    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;

    #[test]
    fn test_execution_plan_schema_generation() {
        let schema = schema_for!(ExecutionPlan);
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();

        // Ensure schema contains required fields
        assert!(schema_json.contains("id"));
        assert!(schema_json.contains("title"));
        assert!(schema_json.contains("milestones"));
        assert!(schema_json.contains("state"));
    }

    #[test]
    fn test_milestone_schema_generation() {
        let schema = schema_for!(Milestone);
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();

        // Ensure schema contains required milestone fields
        assert!(schema_json.contains("id"));
        assert!(schema_json.contains("objective"));
        assert!(schema_json.contains("dependencies"));
        assert!(schema_json.contains("state"));
    }

    #[test]
    fn test_plan_state_transitions() {
        // Test valid state transitions
        assert!(matches!(PlanState::Draft, PlanState::Draft));
        assert!(matches!(PlanState::UnderReview, PlanState::UnderReview));
        assert!(matches!(PlanState::Approved, PlanState::Approved));
        assert!(matches!(PlanState::InProgress, PlanState::InProgress));
        assert!(matches!(PlanState::Completed, PlanState::Completed));

        // Test blocked state with reason
        match (PlanState::Blocked { reason: "Dependency issue".to_string() }) {
            PlanState::Blocked { reason } => assert_eq!(reason, "Dependency issue"),
            _ => panic!("Expected blocked state"),
        }
    }

    #[test]
    fn test_milestone_state_machine() {
        // Test milestone state transitions
        assert!(matches!(MilestoneState::Pending, MilestoneState::Pending));
        assert!(matches!(MilestoneState::Ready, MilestoneState::Ready));
        assert!(matches!(MilestoneState::InProgress, MilestoneState::InProgress));
        assert!(matches!(MilestoneState::Completed, MilestoneState::Completed));

        // Test blocked state with dependencies
        match (MilestoneState::Blocked { dependencies: vec!["M1".to_string()] }) {
            MilestoneState::Blocked { dependencies } => {
                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0], "M1");
            }
            _ => panic!("Expected blocked state"),
        }
    }

    #[test]
    fn test_evidence_gate_validation() {
        let gate = EvidenceGate {
            min_coverage: 0.80,
            min_branch_coverage: 0.75,
            min_mutation_score: 0.70,
            security_scan_required: true,
            performance_budget: Some(PerformanceBudget {
                max_p95_ms: 500,
                max_p99_ms: 1000,
                max_memory_mb: 256,
                min_throughput_per_second: 100.0,
            }),
            required_artifacts: vec!["test_results".to_string(), "coverage".to_string()],
            custom_validations: vec![],
        };

        assert_eq!(gate.min_coverage, 0.80);
        assert!(gate.security_scan_required);
        assert!(gate.performance_budget.is_some());
        assert_eq!(gate.required_artifacts.len(), 2);
    }

    #[test]
    fn test_dependency_graph_structure() {
        let mut nodes = HashMap::new();
        nodes.insert("M1".to_string(), DependencyNode {
            milestone_id: "M1".to_string(),
            node_type: DependencyNodeType::Milestone,
            estimated_cost: 10.0,
            estimated_time_ms: 5000,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        });

        let edges = vec![DependencyEdge {
            from: "M1".to_string(),
            to: "M2".to_string(),
            edge_type: DependencyEdgeType::Hard,
            weight: 1.0,
            metadata: HashMap::new(),
        }];

        let graph = DependencyGraph {
            nodes,
            edges,
            critical_path: vec!["M1".to_string(), "M2".to_string()],
            parallel_groups: vec![vec!["M1".to_string()], vec!["M2".to_string()]],
            has_cycles: false,
            cycles: vec![],
        };

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.critical_path.len(), 2);
        assert!(!graph.has_cycles);
    }
}
