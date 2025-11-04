//! Core type definitions for the self-prompting agent system

use schemars::JsonSchema;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Simple evaluation report stub (replace with real evaluation when available)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvalReport {
    pub score: f64,
    pub status: EvalStatus,
    pub thresholds_met: Vec<String>,
    pub failed_criteria: Vec<String>,
}

/// Evaluation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum EvalStatus {
    Pass,
    Fail,
    Partial,
}

/// Execution modes for the autonomous agent

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionMode {
    Strict,    // Ask for approval before each change
    Auto,      // Apply changes automatically, promote only when gates pass
    DryRun,    // Simulate execution without making changes
}

/// Safety modes for the sandbox

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum SafetyMode {
    Strict,      // No file operations allowed
    Sandbox,     // Limited operations within workspace
    Autonomous,  // Full autonomous operations
}

/// Task definition for self-prompting execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Task {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub description: String,
    pub task_type: TaskType,
    pub target_files: Vec<String>,
    pub constraints: HashMap<String, String>,
    pub refinement_context: Vec<String>,
}

/// Task types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TaskType {
    CodeGeneration,
    CodeReview,
    CodeRefactor,
    Testing,
    Documentation,
    Research,
    Planning,
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
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResult {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub final_report: EvalReport,
    pub execution_time_ms: u64,
    pub artifacts: Vec<Artifact>,
}

/// Execution artifact
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Artifact {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub file_path: String,
    pub content: String,
    pub artifact_type: ArtifactType,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// Artifact types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ArtifactType {
    Code,
    Test,
    Documentation,
    Configuration,
    Report,
    Text,
}

/// Change set for task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangeSet {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub changes: Vec<FileChange>,
    pub rationale: String,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// File change in a change set
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileChange {
    pub file_path: String,
    pub change_type: ChangeType,
    pub content: Option<String>,
    pub line_number: Option<usize>,
}

/// Change types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
}

/// Self-prompting agent error

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
pub enum SelfPromptingAgentError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Evaluation error: {0}")]
    Evaluation(String),

    #[error("Model provider error: {0}")]
    ModelProvider(String),

    #[error("Sandbox error: {0}")]
    Sandbox(String),

    #[error("Task validation error: {0}")]
    Validation(String),

    #[error("Learning error: {0}")]
    Learning(String),

    #[error("Database error: {0}")]
    Database(String),
}
