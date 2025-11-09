//! Chain-of-Thought Tracing for Orchestration Visibility
//!
//! This module provides comprehensive tracing capabilities for orchestration
//! decision-making processes, enabling complete visibility into:
//! - Decision points and reasoning
//! - Worker coordination events
//! - Council evaluation processes
//! - End-to-end orchestration traces

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Decision types for orchestration decisions
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DecisionType {
    WorkerAssignment,
    JudgeSelection,
    ParallelCoordination,
    QualityGate,
    FailureRecovery,
    CouncilApproval,
    DependencyResolution,
    ResourceAllocation,
}

impl std::fmt::Display for DecisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionType::WorkerAssignment => write!(f, "WorkerAssignment"),
            DecisionType::JudgeSelection => write!(f, "JudgeSelection"),
            DecisionType::ParallelCoordination => write!(f, "ParallelCoordination"),
            DecisionType::QualityGate => write!(f, "QualityGate"),
            DecisionType::FailureRecovery => write!(f, "FailureRecovery"),
            DecisionType::CouncilApproval => write!(f, "CouncilApproval"),
            DecisionType::DependencyResolution => write!(f, "DependencyResolution"),
            DecisionType::ResourceAllocation => write!(f, "ResourceAllocation"),
        }
    }
}

/// Context information for decision making
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DecisionContext {
    #[schemars(with = "Option<String>")]
    pub task_id: Option<Uuid>,
    #[schemars(with = "Option<String>")]
    pub plan_id: Option<Uuid>,
    pub milestone_id: Option<String>,
    #[schemars(with = "Option<String>")]
    pub worker_id: Option<Uuid>,
    pub resource_constraints: HashMap<String, serde_json::Value>,
    #[schemars(with = "Option<String>")]
    pub time_constraints: Option<DateTime<Utc>>,
    pub priority_level: Option<String>,
}

/// Alternative option considered during decision making
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Alternative {
    pub option: String,
    pub score: f64,
    pub reasoning: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub confidence: f64,
}

/// Risk assessment for decision
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAssessment {
    pub risk_level: String,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub fallback_options: Vec<String>,
}

/// Complete decision point trace
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DecisionPoint {
    #[schemars(with = "String")]
    pub decision_id: Uuid,
    pub decision_type: DecisionType,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub context: DecisionContext,
    pub alternatives: Vec<Alternative>,
    pub chosen_option: String,
    pub reasoning: String,
    pub confidence: f64,
    pub risk_assessment: Option<RiskAssessment>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Coordination event types
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CoordinationEventType {
    WorkerAssigned,
    WorkerReleased,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    MilestoneStarted,
    MilestoneCompleted,
    DependencyResolved,
    ResourceAllocated,
    ResourceFreed,
    ParallelExecutionStarted,
    ParallelExecutionCompleted,
}

/// Individual coordination event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationEvent {
    #[schemars(with = "String")]
    pub event_id: Uuid,
    pub event_type: CoordinationEventType,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    #[schemars(with = "Option<String>")]
    pub task_id: Option<Uuid>,
    pub milestone_id: Option<String>,
    #[schemars(with = "Option<String>")]
    pub worker_id: Option<Uuid>,
    pub resource_id: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
}

/// Worker assignment record
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerAssignment {
    #[schemars(with = "String")]
    pub assignment_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub milestone_id: Option<String>,
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    #[schemars(with = "String")]
    pub assigned_at: DateTime<Utc>,
    pub capability_score: f64,
    pub load_factor: f64,
    pub expected_duration: Option<u64>, // milliseconds
    pub assignment_reason: String,
}

/// Execution timeline event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEvent {
    #[schemars(with = "String")]
    pub event_id: Uuid,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub milestone_id: Option<String>,
    pub event_type: String,
    pub details: HashMap<String, serde_json::Value>,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceMetrics {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub network_utilization: f64,
    pub disk_utilization: f64,
    pub worker_utilization: Vec<WorkerUtilization>,
}

/// Worker utilization data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerUtilization {
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    pub utilization_percentage: f64,
    pub active_tasks: usize,
    pub queued_tasks: usize,
}

/// Failure recovery information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailureRecovery {
    pub failure_type: String,
    pub recovery_strategy: String,
    pub recovery_duration_ms: u64,
    pub success: bool,
    pub fallback_used: bool,
    pub lessons_learned: Vec<String>,
}

/// Complete coordination trace
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationTrace {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub coordination_events: Vec<CoordinationEvent>,
    pub worker_assignments: Vec<WorkerAssignment>,
    pub execution_timeline: Vec<ExecutionEvent>,
    pub resource_utilization: ResourceMetrics,
    pub failure_recovery: Option<FailureRecovery>,
}

/// Judge selection trace
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeSelectionTrace {
    pub selection_criteria: HashMap<String, serde_json::Value>,
    pub available_judges: Vec<JudgeInfo>,
    pub selected_judges: Vec<JudgeInfo>,
    pub selection_reasoning: String,
    pub selection_confidence: f64,
}

/// Judge information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeInfo {
    pub judge_id: String,
    pub judge_type: String,
    pub specialization: Vec<String>,
    pub performance_score: f64,
    pub availability_score: f64,
    pub selection_score: f64,
}

/// Verdict from individual judge
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeVerdict {
    pub judge_id: String,
    pub verdict: String,
    pub confidence: f64,
    pub reasoning: String,
    pub evidence: Vec<String>,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
}

/// Aggregation process trace
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AggregationTrace {
    pub aggregation_method: String,
    pub consensus_threshold: f64,
    pub aggregation_steps: Vec<AggregationStep>,
    pub final_consensus_score: f64,
    pub dissenting_opinions: Vec<String>,
}

/// Step in aggregation process
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AggregationStep {
    pub step_number: usize,
    pub step_type: String,
    pub input_verdicts: Vec<String>,
    pub output_score: f64,
    pub reasoning: String,
}

/// Council evaluation trace
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CouncilEvaluationTrace {
    #[schemars(with = "String")]
    pub session_id: Uuid,
    pub review_context: serde_json::Value, // Context passed to council
    pub judge_selection: JudgeSelectionTrace,
    pub individual_verdicts: Vec<JudgeVerdict>,
    pub aggregation_process: AggregationTrace,
    pub final_decision: String,
    pub decision_rationale: String,
}

/// Execution phase in end-to-end trace
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPhase {
    #[schemars(with = "String")]
    pub phase_id: Uuid,
    pub phase_name: String,
    #[schemars(with = "String")]
    pub start_time: DateTime<Utc>,
    #[schemars(with = "Option<String>")]
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub milestones_completed: usize,
    pub milestones_failed: usize,
    pub decision_points: Vec<DecisionPoint>,
    pub coordination_events: Vec<CoordinationEvent>,
}

/// Quality gate result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityGateResult {
    pub gate_id: String,
    pub gate_type: String,
    pub passed: bool,
    pub score: f64,
    pub violations: Vec<String>,
    pub remediation_required: bool,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
}

/// Final plan outcome
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanOutcome {
    pub final_state: String,
    pub completion_percentage: f64,
    pub total_duration_ms: u64,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub lessons_learned: Vec<String>,
    pub recommendations: Vec<String>,
}

/// End-to-end orchestration trace
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndToEndTrace {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub execution_phases: Vec<ExecutionPhase>,
    pub decision_points: Vec<DecisionPoint>,
    pub coordination_events: Vec<CoordinationEvent>,
    pub council_evaluations: Vec<CouncilEvaluationTrace>,
    pub evidence_collection: Vec<serde_json::Value>, // Evidence bundles
    pub quality_gates: Vec<QualityGateResult>,
    pub final_outcome: PlanOutcome,
}

/// Chain-of-thought phase enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ChainOfThoughtPhase {
    PlanAnalysis,
    WorkerSelection,
    TaskExecution,
    CouncilReview,
    QualityValidation,
    Completion,
    FailureRecovery,
    DecisionMaking,
}

/// Progress indicator for operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ProgressIndicator {
    Percentage(f64),
    Steps { current: usize, total: usize },
    Phase(String),
    WaitingFor { resource: String, timeout: Option<u64> },
}

/// Error link in error propagation chain
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorLink {
    pub from_component: String,
    pub to_component: String,
    pub error_type: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, serde_json::Value>,
}

/// Silent failure detection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SilentFailure {
    #[schemars(with = "String")]
    pub operation_id: Uuid,
    pub component: String,
    pub failure_type: String,
    #[schemars(with = "String")]
    pub detected_at: DateTime<Utc>,
    pub detection_method: String,
    pub impact: String,
}

/// Stuck state types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum StuckState {
    NoProgress {
        duration_ms: u64,
        #[schemars(with = "String")]
        last_activity: DateTime<Utc>
    },
    WaitingForResource { resource: String, wait_duration_ms: u64 },
    DeadlockDetected {
        resources: Vec<String>,
        #[schemars(with = "Vec<String>")]
        participants: Vec<Uuid>
    },
    TimeoutImminent { elapsed_ms: u64, threshold_ms: u64 },
}
