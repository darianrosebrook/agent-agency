//! Core type definitions for the self-prompting agent system

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Simple evaluation report stub (replace with real evaluation when available)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub score: f64,
    pub status: EvalStatus,
    pub thresholds_met: Vec<String>,
    pub failed_criteria: Vec<String>,
}

/// Evaluation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvalStatus {
    Pass,
    Fail,
    Partial,
}

/// Execution modes for the autonomous agent
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Strict,    // Ask for approval before each change
    Auto,      // Apply changes automatically, promote only when gates pass
    DryRun,    // Simulate execution without making changes
}

/// Safety modes for the sandbox
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyMode {
    Strict,      // No file operations allowed
    Sandbox,     // Limited operations within workspace
    Autonomous,  // Full autonomous operations
}

/// Task definition for self-prompting execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub task_type: TaskType,
    pub target_files: Vec<String>,
    pub constraints: HashMap<String, String>,
    pub refinement_context: Vec<String>,
}

impl Task {
    pub fn new(description: String, task_type: TaskType) -> Self {
        Self {
            id: Uuid::new_v4(),
            description,
            task_type,
            target_files: Vec::new(),
            constraints: HashMap::new(),
            refinement_context: Vec::new(),
        }
    }

    pub fn add_refinement_context(&mut self, context: String) {
        self.refinement_context.push(context);
    }
}

/// Task types supported by the agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    CodeFix,
    CodeGeneration,
    TextTransformation,
    DesignTokenApplication,
    DocumentationUpdate,
}

/// Result of a self-prompting task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub final_report: EvalReport,
    pub iterations: usize,
    pub stop_reason: StopReason,
    pub model_used: String,
    pub total_time_ms: u64,
    pub artifacts: Vec<Artifact>,
}

/// Artifacts produced during task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub file_path: String,
    pub content: String,
    pub artifact_type: ArtifactType,
    pub created_at: DateTime<Utc>,
}

/// Types of artifacts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    Code,
    Test,
    Documentation,
    Configuration,
    Diff,  // First-class diff artifacts for observability
}

/// Model response from a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub text: String,
    pub model_id: String,
    pub tokens_used: usize,
    pub latency_ms: u64,
    pub finish_reason: Option<String>,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub max_context: usize,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
}

/// Model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub capabilities: ModelCapabilities,
}

/// Model health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
}

/// Unified diff representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDiff {
    pub file_path: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<String>,
}

/// Stop reasons for task execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopReason {
    Satisficed,
    MaxIterations,
    QualityCeiling,
    FailedGates,
    Timeout,
    Error,
    NoProgress,  // Added for hysteresis and no-progress guards
    PatchFailure, // Added for patch application failures (addresses 75% of agent failures)
    ProgressStalled, // Added for quantitative progress plateau detection (addresses unproductive loops)
    Aborted, // Added for user-initiated task abortion
    Unknown,
}

/// Learning signals for reflexive learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelfPromptingSignal {
    /// Iteration efficiency patterns
    IterationEfficiency {
        iterations: usize,
        quality: f64,
        time: f64, // milliseconds per iteration
    },

    /// Model performance on specific tasks
    ModelPerformance {
        model_id: String,
        task_type: String,
        score: f64,
    },

    /// Effectiveness of satisficing decisions
    SatisficingEffectiveness {
        stopped_early: bool,
        quality_delta: f64,
        iterations_saved: usize,
    },
}

/// Quantitative progress metrics for iteration tracking (addresses unproductive loops)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationProgress {
    pub files_touched: usize,
    pub loc_changed: usize,
    pub test_pass_rate_delta: f64,
    pub lint_errors_delta: i32,
    pub score_improvement: f64,
    pub timestamp: DateTime<Utc>,
}

/// Context utilization metrics for preventing overload failures (addresses large codebase issues)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetrics {
    pub prompt_size_tokens: usize,
    pub context_window_utilization: f64, // 0.0 to 1.0
    pub files_in_scope: usize,
    pub dependency_depth: usize,
    pub timestamp: DateTime<Utc>,
}

/// Context overload detection and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMonitor {
    pub metrics: ContextMetrics,
    pub overload_threshold: f64, // e.g., 0.8 for 80% utilization
    pub max_files_threshold: usize,
    pub scope_reduction_strategy: ScopeReductionStrategy,
}

/// File change operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeOperation {
    /// Create a new file
    Create { content: String },
    /// Modify an existing file
    Modify { expected_content: String, new_content: String },
    /// Delete an existing file
    Delete { expected_content: String },
}

/// Individual file change in a changeset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: std::path::PathBuf,
    pub operation: ChangeOperation,
}

/// Atomic changeset for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub changes: Vec<FileChange>,
    pub rationale: String,
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl ChangeSet {
    pub fn new(changes: Vec<FileChange>, rationale: String) -> Self {
        Self {
            changes,
            rationale,
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
        }
    }

    /// Count total lines of code affected
    pub fn total_loc(&self) -> usize {
        self.changes.iter().map(|change| match &change.operation {
            ChangeOperation::Create { content } |
            ChangeOperation::Modify { new_content: content, .. } => {
                content.lines().count()
            }
            ChangeOperation::Delete { .. } => 0,
        }).sum()
    }

    /// Count files changed
    pub fn files_changed(&self) -> usize {
        self.changes.len()
    }
}

/// Receipt from applying a changeset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSetReceipt {
    pub changeset_id: Uuid,
    pub applied_at: DateTime<Utc>,
    pub files_changed: usize,
    pub loc_delta: i64, // Can be negative for deletions
    pub sha256_tree: String, // SHA256 of entire workspace tree
    pub checkpoint_id: String, // Git commit hash or snapshot ID
}

/// Strategies for reducing context scope when overloaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeReductionStrategy {
    /// Remove least recently modified files
    RemoveLeastRecent,
    /// Keep only files directly related to current task
    TaskRelevantOnly,
    /// Prioritize files with highest change frequency
    HighChangeFrequency,
    /// Manual intervention required
    ManualIntervention,
}

/// File metadata for scope reduction analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub last_modified: DateTime<Utc>,
    pub change_frequency: usize, // Number of changes in recent history
    pub task_relevance_score: f64, // 0.0 to 1.0 based on task description match
}

/// Task relevance analysis for files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRelevanceAnalysis {
    pub keywords: Vec<String>,
    pub file_extensions: Vec<String>,
    pub directory_patterns: Vec<String>,
}

/// Iteration context for maintaining state across loops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationContext {
    pub iteration: usize,
    pub previous_output: String,
    pub eval_report: EvalReport,
    pub refinement_prompt: String,
    pub timestamp: DateTime<Utc>,
}

/// Structured action request from model (tool-call envelope)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    /// Type of action to perform
    pub action_type: ActionType,
    /// File changes to apply (for patch/write actions)
    pub changeset: Option<file_ops::ChangeSet>,
    /// Human-readable reason for this action
    pub reason: String,
    /// Model's confidence in this action (0.0 to 1.0)
    pub confidence: f64,
    /// Additional metadata for the action
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ActionRequest {
    /// Create a new action request
    pub fn new(action_type: ActionType, reason: String, confidence: f64) -> Self {
        Self {
            action_type,
            changeset: None,
            reason,
            confidence,
            metadata: HashMap::new(),
        }
    }

    /// Create a patch action request
    pub fn patch(changeset: file_ops::ChangeSet, reason: String, confidence: f64) -> Self {
        Self {
            action_type: ActionType::Patch,
            changeset: Some(changeset),
            reason,
            confidence,
            metadata: HashMap::new(),
        }
    }

    /// Create a write action request
    pub fn write(changeset: file_ops::ChangeSet, reason: String, confidence: f64) -> Self {
        Self {
            action_type: ActionType::Write,
            changeset: Some(changeset),
            reason,
            confidence,
            metadata: HashMap::new(),
        }
    }

    /// Create a no-op action request
    pub fn noop(reason: String, confidence: f64) -> Self {
        Self {
            action_type: ActionType::NoOp,
            changeset: None,
            reason,
            confidence,
            metadata: HashMap::new(),
        }
    }

    /// Validate the action request
    pub fn validate(&self) -> Result<(), ActionValidationError> {
        // Validate confidence range
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ActionValidationError::InvalidConfidence(self.confidence));
        }

        // Validate changeset presence for patch/write actions
        match self.action_type {
            ActionType::Patch | ActionType::Write => {
                if self.changeset.is_none() {
                    return Err(ActionValidationError::MissingChangeset);
                }
            }
            ActionType::NoOp => {
                if self.changeset.is_some() {
                    return Err(ActionValidationError::UnexpectedChangeset);
                }
            }
        }

        Ok(())
    }

    /// Check if this action requires file changes
    pub fn requires_changes(&self) -> bool {
        matches!(self.action_type, ActionType::Patch | ActionType::Write)
    }

    /// Get the changeset if present
    pub fn changeset(&self) -> Option<&file_ops::ChangeSet> {
        self.changeset.as_ref()
    }
}

/// Types of actions that can be requested
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// Apply patches to existing files
    Patch,
    /// Write new files or overwrite existing ones
    Write,
    /// No operation needed (task is complete or no changes required)
    NoOp,
}

/// Errors that can occur during action request validation
#[derive(Debug, thiserror::Error)]
pub enum ActionValidationError {
    #[error("Invalid confidence value: {0} (must be between 0.0 and 1.0)")]
    InvalidConfidence(f64),

    #[error("Changeset required for patch/write actions")]
    MissingChangeset,

    #[error("Changeset not allowed for no-op actions")]
    UnexpectedChangeset,

    #[error("JSON schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("File operation validation failed: {0}")]
    FileOpsValidation(String),
}
