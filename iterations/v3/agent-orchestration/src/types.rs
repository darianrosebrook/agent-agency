//! Core types for the Agent Orchestration system
//!
//! This module contains all the core data structures used by the orchestration system
//! including configuration, task scopes, budgets, and execution results.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

// Import contracts types directly
use agent_agency_contracts::WorkingSpec as ContractsWorkingSpec;
use agent_agency_contracts::types::planning::{TaskPriority, BlastRadius};
use agent_agency_contracts::planning_io::ChangeBudget as ContractsChangeBudget;

/// Task scope definition for orchestration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskScope {
    /// Files and directories included in this task scope
    pub in_scope: Vec<String>,
    /// Files and directories explicitly excluded from this task scope
    pub out_scope: Vec<String>,
}

// ChangeBudget and BlastRadius are now defined in agent-agency-contracts
// Use agent_agency_contracts::prelude::* to access them

/// Memory-informed orchestration decision

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct MemoryInformedDecision {
    /// Whether parallel execution is preferred based on historical success
    pub prefers_parallel: bool,
    /// Suggested worker IDs based on past performance
    pub suggested_workers: Vec<String>,
    /// Expected success rate for the preferred strategy
    pub expected_success_rate: f32,
    /// Confidence level in the decision (0.0 to 1.0)
    pub confidence: f32,
}

// TaskExecutionResult is now in agent_agency_contracts::task_executor::TaskExecutionResult
// Use agent_agency_contracts::task_executor::TaskExecutionResult instead
// WorkingSpec, ExecutionArtifacts, and QualityReport should be stored/retrieved separately

// ExecutionArtifacts is now imported from agent_agency_contracts
// Use agent_agency_contracts::ExecutionArtifacts

/// Execution status for tasks
#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Running,
    Skipped,
    Completed,
    Failed,
    Cancelled,
    /// Task is starting up
    Starting,
    /// Task is paused by user
    Paused,
    /// Task is awaiting approval
    AwaitingApproval,
    /// Task is in planning phase
    Planning,
    /// Task is in consensus phase
    Consensus,
    /// Task is in execution phase
    Execution,
}

/// Quality report from orchestration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct QualityReport {
    /// Overall quality score (0.0 to 1.0)
    pub score: f32,
    /// Quality metrics
    pub metrics: HashMap<String, f32>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestratorConfig {
    /// Maximum time allowed for orchestration (in seconds)
    pub max_orchestration_time_seconds: u64,
    /// Whether to enable parallel execution
    pub enable_parallel_execution: bool,
    /// Whether to enable memory-informed decisions
    pub enable_memory_decisions: bool,
    /// Whether to enable ARM optimization
    pub enable_arm_optimization: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Recovery timeout in seconds
    pub recovery_timeout_seconds: u64,
    /// Success threshold to close circuit
    pub success_threshold: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f32,
    /// Whether to add jitter to delays
    pub jitter: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_orchestration_time_seconds: 300, // 5 minutes
            enable_parallel_execution: true,
            enable_memory_decisions: true,
            enable_arm_optimization: cfg!(target_arch = "aarch64"),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            retry_config: RetryConfig::default(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_seconds: 60,
            success_threshold: 3,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

// TaskDescriptor is now imported from agent_agency_contracts::types::planning::TaskDescriptor
// (removed duplicate definition)


// ExecutionMode is now imported from agent_agency_contracts::types::planning
// (removed duplicate definition)

/// Task type classification

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum TaskType {
    Feature,
    BugFix,
    Refactor,
    Documentation,
    Maintenance,
}

/// Working specification for tasks
/// 
/// # Deprecation Notice
/// This local type is being phased out in favor of `agent_agency_contracts::WorkingSpec`.
/// Use the type adapters in `crate::planning::type_adapters` for conversion during migration.
/// 
/// # Migration
/// ```rust
/// // Old (deprecated):
/// use crate::types::WorkingSpec;
/// 
/// // New (preferred):
/// use agent_agency_contracts::WorkingSpec;
/// // or
/// use agent_agency_contracts::working_spec::WorkingSpec;
/// ```
#[derive(Debug, Clone)]
#[deprecated(note = "Use agent_agency_contracts::WorkingSpec instead. See type_adapters for migration.")]
pub struct WorkingSpec {
    /// Specification ID
    pub id: String,
    /// Specification title
    pub title: String,
    /// Risk tier (1-3)
    pub risk_tier: u8,
    /// Mode (feature, refactor, fix, etc.)
    pub mode: String,
    /// Change budget (uses contracts ChangeBudget)
    pub change_budget: ContractsChangeBudget,
    /// Blast radius
    pub blast_radius: BlastRadius,
    /// Scope definition
    pub scope: TaskScope,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

/// Acceptance criterion for tasks

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCriterion {
    /// Criterion ID
    pub id: String,
    /// Given condition
    pub given: String,
    /// When condition
    pub when: String,
    /// Then expected outcome
    pub then: String,
}

/// Diff statistics for change tracking

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiffStats {
    /// Number of files changed
    pub files_changed: u32,
    /// Lines of code added
    pub lines_added: u32,
    /// Lines of code removed
    pub lines_removed: u32,
    /// Lines of code modified
    pub lines_modified: u32,
    /// Number of files added
    pub files_added: u32,
    /// Number of files modified
    pub files_modified: u32,
    /// Number of files deleted
    pub files_deleted: u32,
    /// Lines of code deleted
    pub lines_deleted: u32,
    /// Number of binary files changed
    pub binary_files_changed: u32,
}

/// Multimodal task for processing different content types

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultimodalTask {
    /// Task ID
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Content type (text, image, audio, video)
    pub content_type: String,
    /// Task data
    pub data: Vec<u8>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Result of multimodal processing

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultimodalProcessingResult {
    /// Task ID
    pub task_id: String,
    /// Processing status (uses contracts ExecutionStatus for standard statuses, local for orchestration-specific)
    pub status: agent_agency_contracts::ExecutionStatus,
    /// Processed content
    pub processed_content: Option<Vec<u8>>,
    /// Extracted features
    pub features: HashMap<String, serde_json::Value>,
    /// Error message if processing failed
    pub error: Option<String>,
}

/// Working specification scope

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WorkingSpecScope {
    /// Files and directories included in scope
    pub in_scope: Vec<String>,
    /// Files and directories excluded from scope
    pub out_scope: Vec<String>,
}
