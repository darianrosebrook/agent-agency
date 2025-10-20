//! Core type definitions for the self-prompting agent system

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
