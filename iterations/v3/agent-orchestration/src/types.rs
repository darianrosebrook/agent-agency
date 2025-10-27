//! Core types for the Agent Orchestration system
//!
//! This module contains all the core data structures used by the orchestration system
//! including configuration, task scopes, budgets, and execution results.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task scope definition for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    /// Files and directories included in this task scope
    pub in_scope: Vec<String>,
    /// Files and directories explicitly excluded from this task scope
    pub out_scope: Vec<String>,
}

/// Change budget for orchestration constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeBudget {
    /// Maximum number of files that can be changed
    pub max_files: u32,
    /// Maximum lines of code that can be changed
    pub max_loc: u32,
}

/// Blast radius for orchestration impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    /// Modules that will be affected by the orchestration
    pub modules: Vec<String>,
    /// Whether data migration is required
    pub data_migration: bool,
    /// External dependencies that will be affected
    pub external_deps: Vec<String>,
}

/// Memory-informed orchestration decision
#[derive(Debug, Clone)]
pub struct MemoryInformedDecision {
    /// Whether parallel execution is preferred based on historical success
    pub prefers_parallel: bool,
    /// Suggested worker IDs based on past performance
    pub suggested_workers: Vec<String>,
    /// Expected success rate for the preferred strategy
    pub expected_success_rate: f32,
    /// Confidence level in the decision (0.0 to 1.0)
    pub confidence: f32,
}

/// Result of task execution orchestration
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    /// The final working specification after orchestration
    pub working_spec: Option<String>, // Simplified for now - was agent_agency_contracts::working_spec::WorkingSpec
    /// Execution artifacts produced during orchestration
    pub artifacts: ExecutionArtifacts,
    /// Quality report from orchestration
    pub quality_report: Option<QualityReport>,
}

/// Execution artifacts produced during task orchestration
#[derive(Debug, Clone)]
pub struct ExecutionArtifacts {
    /// Unique execution ID
    pub execution_id: String,
    /// Worker ID that executed the task
    pub worker_id: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Output from execution
    pub output: Option<String>,
    /// Error message if execution failed
    pub error: Option<String>,
}

/// Execution status for tasks
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Running,
    Skipped,
    Completed,
    Failed,
    Cancelled,
}

/// Quality report from orchestration
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Overall quality score (0.0 to 1.0)
    pub score: f32,
    /// Quality metrics
    pub metrics: HashMap<String, f32>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Recovery timeout in seconds
    pub recovery_timeout_seconds: u64,
    /// Success threshold to close circuit
    pub success_threshold: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Task descriptor for orchestration
#[derive(Debug, Clone)]
pub struct TaskDescriptor {
    /// Unique task identifier
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Task scope
    pub scope_in: TaskScope,
    /// Out of scope areas
    pub scope_out: Option<TaskScope>,
    /// Change budget constraints
    pub change_budget: ChangeBudget,
    /// Blast radius analysis
    pub blast_radius: BlastRadius,
    /// Task priority
    pub priority: TaskPriority,
    /// Execution mode
    pub execution_mode: crate::ExecutionMode,
    /// Task type/category
    pub task_type: String,
    /// Risk tier assessment
    pub risk_tier: Option<crate::council_types::RiskTier>,
    /// Acceptance criteria
    pub acceptance: Option<String>,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Working specification for tasks
#[derive(Debug, Clone)]
pub struct WorkingSpec {
    /// Specification ID
    pub id: String,
    /// Specification title
    pub title: String,
    /// Risk tier (1-3)
    pub risk_tier: u8,
    /// Mode (feature, refactor, fix, etc.)
    pub mode: String,
    /// Change budget
    pub change_budget: ChangeBudget,
    /// Blast radius
    pub blast_radius: BlastRadius,
    /// Scope definition
    pub scope: TaskScope,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

/// Acceptance criterion for tasks
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct DiffStats {
    /// Number of files changed
    pub files_changed: u32,
    /// Lines of code added
    pub lines_added: u32,
    /// Lines of code removed
    pub lines_removed: u32,
    /// Lines of code modified
    pub lines_modified: u32,
}
