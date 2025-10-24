//! Tests for core type definitions

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use self_prompting_agent::types::*;

#[test]
fn test_eval_status_variants() {
    assert_eq!(EvalStatus::Pass, EvalStatus::Pass);
    assert_eq!(EvalStatus::Fail, EvalStatus::Fail);
    assert_eq!(EvalStatus::Partial, EvalStatus::Partial);
    assert_ne!(EvalStatus::Pass, EvalStatus::Fail);
}

#[test]
fn test_execution_mode_variants() {
    assert_eq!(ExecutionMode::Strict, ExecutionMode::Strict);
    assert_eq!(ExecutionMode::Auto, ExecutionMode::Auto);
    assert_eq!(ExecutionMode::DryRun, ExecutionMode::DryRun);
    assert_ne!(ExecutionMode::Strict, ExecutionMode::Auto);
}

#[test]
fn test_safety_mode_variants() {
    assert_eq!(SafetyMode::Strict, SafetyMode::Strict);
    assert_eq!(SafetyMode::Sandbox, SafetyMode::Sandbox);
    assert_eq!(SafetyMode::Autonomous, SafetyMode::Autonomous);
    assert_ne!(SafetyMode::Strict, SafetyMode::Sandbox);
}

#[test]
fn test_task_type_variants() {
    assert_eq!(TaskType::CodeGeneration, TaskType::CodeGeneration);
    assert_eq!(TaskType::CodeReview, TaskType::CodeReview);
    assert_eq!(TaskType::CodeRefactor, TaskType::CodeRefactor);
    assert_eq!(TaskType::Testing, TaskType::Testing);
    assert_eq!(TaskType::Documentation, TaskType::Documentation);
    assert_eq!(TaskType::Research, TaskType::Research);
    assert_eq!(TaskType::Planning, TaskType::Planning);
}

#[test]
fn test_artifact_type_variants() {
    assert_eq!(ArtifactType::Code, ArtifactType::Code);
    assert_eq!(ArtifactType::Test, ArtifactType::Test);
    assert_eq!(ArtifactType::Documentation, ArtifactType::Documentation);
    assert_eq!(ArtifactType::Configuration, ArtifactType::Configuration);
    assert_eq!(ArtifactType::Report, ArtifactType::Report);
}

#[test]
fn test_change_type_variants() {
    assert_eq!(ChangeType::Create, ChangeType::Create);
    assert_eq!(ChangeType::Modify, ChangeType::Modify);
    assert_eq!(ChangeType::Delete, ChangeType::Delete);
}

#[test]
fn test_task_new() {
    let description = "Test task description".to_string();
    let task_type = TaskType::CodeGeneration;

    let task = Task::new(description.clone(), task_type.clone());

    // Check that ID is generated
    assert!(!task.id.to_string().is_empty());

    // Check other fields
    assert_eq!(task.description, description);
    assert_eq!(task.task_type, task_type);
    assert!(task.target_files.is_empty());
    assert!(task.constraints.is_empty());
    assert!(task.refinement_context.is_empty());
}

#[test]
fn test_task_creation_with_different_types() {
    let task1 = Task::new("Code generation".to_string(), TaskType::CodeGeneration);
    let task2 = Task::new("Code review".to_string(), TaskType::CodeReview);
    let task3 = Task::new("Testing".to_string(), TaskType::Testing);

    assert_eq!(task1.task_type, TaskType::CodeGeneration);
    assert_eq!(task2.task_type, TaskType::CodeReview);
    assert_eq!(task3.task_type, TaskType::Testing);

    // IDs should be different
    assert_ne!(task1.id, task2.id);
    assert_ne!(task2.id, task3.id);
    assert_ne!(task1.id, task3.id);
}

#[test]
fn test_eval_report_creation() {
    let report = EvalReport {
        score: 0.85,
        status: EvalStatus::Pass,
        thresholds_met: vec!["threshold1".to_string(), "threshold2".to_string()],
        failed_criteria: vec!["criterion1".to_string()],
    };

    assert_eq!(report.score, 0.85);
    assert_eq!(report.status, EvalStatus::Pass);
    assert_eq!(report.thresholds_met.len(), 2);
    assert_eq!(report.failed_criteria.len(), 1);
}

#[test]
fn test_task_result_creation() {
    let task_id = Uuid::new_v4();
    let final_report = EvalReport {
        score: 0.9,
        status: EvalStatus::Pass,
        thresholds_met: vec!["all".to_string()],
        failed_criteria: vec![],
    };

    let artifacts = vec![
        Artifact {
            id: Uuid::new_v4(),
            file_path: "test.rs".to_string(),
            content: "test content".to_string(),
            artifact_type: ArtifactType::Code,
            created_at: Utc::now(),
        }
    ];

    let result = TaskResult {
        task_id,
        task_type: TaskType::CodeGeneration,
        final_report,
        execution_time_ms: 1500,
        artifacts,
    };

    assert_eq!(result.task_id, task_id);
    assert_eq!(result.task_type, TaskType::CodeGeneration);
    assert_eq!(result.execution_time_ms, 1500);
    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.final_report.score, 0.9);
}

#[test]
fn test_artifact_creation() {
    let id = Uuid::new_v4();
    let created_at = Utc::now();

    let artifact = Artifact {
        id,
        file_path: "src/main.rs".to_string(),
        content: "fn main() {}".to_string(),
        artifact_type: ArtifactType::Code,
        created_at,
    };

    assert_eq!(artifact.id, id);
    assert_eq!(artifact.file_path, "src/main.rs");
    assert_eq!(artifact.content, "fn main() {}");
    assert_eq!(artifact.artifact_type, ArtifactType::Code);
    assert_eq!(artifact.created_at, created_at);
}

#[test]
fn test_change_set_creation() {
    let id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let created_at = Utc::now();

    let changes = vec![
        FileChange {
            file_path: "src/lib.rs".to_string(),
            change_type: ChangeType::Modify,
            content: Some("new content".to_string()),
            line_number: Some(10),
        }
    ];

    let changeset = ChangeSet {
        id,
        task_id,
        changes,
        rationale: "Test rationale".to_string(),
        created_at,
    };

    assert_eq!(changeset.id, id);
    assert_eq!(changeset.task_id, task_id);
    assert_eq!(changeset.changes.len(), 1);
    assert_eq!(changeset.rationale, "Test rationale");
    assert_eq!(changeset.created_at, created_at);
}

#[test]
fn test_file_change_creation() {
    let change = FileChange {
        file_path: "Cargo.toml".to_string(),
        change_type: ChangeType::Create,
        content: Some("version = \"1.0.0\"".to_string()),
        line_number: None,
    };

    assert_eq!(change.file_path, "Cargo.toml");
    assert_eq!(change.change_type, ChangeType::Create);
    assert_eq!(change.content, Some("version = \"1.0.0\"".to_string()));
    assert_eq!(change.line_number, None);
}

#[test]
fn test_self_prompting_agent_error_variants() {
    let config_err = SelfPromptingAgentError::Configuration("config failed".to_string());
    let exec_err = SelfPromptingAgentError::Execution("exec failed".to_string());
    let eval_err = SelfPromptingAgentError::Evaluation("eval failed".to_string());
    let model_err = SelfPromptingAgentError::ModelProvider("model failed".to_string());
    let sandbox_err = SelfPromptingAgentError::Sandbox("sandbox failed".to_string());
    let validation_err = SelfPromptingAgentError::Validation("validation failed".to_string());

    // Test that they display correctly
    assert!(format!("{}", config_err).contains("Configuration error"));
    assert!(format!("{}", exec_err).contains("Execution error"));
    assert!(format!("{}", eval_err).contains("Evaluation error"));
    assert!(format!("{}", model_err).contains("Model provider error"));
    assert!(format!("{}", sandbox_err).contains("Sandbox error"));
    assert!(format!("{}", validation_err).contains("Task validation error"));
}

#[test]
fn test_serialization_deserialization() {
    // Test that key types can be serialized/deserialized
    let task = Task::new("test".to_string(), TaskType::CodeGeneration);

    // This tests that the structs are properly marked with Serialize/Deserialize
    let serialized = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&serialized).unwrap();

    assert_eq!(task.description, deserialized.description);
    assert_eq!(task.task_type, deserialized.task_type);
}

#[test]
fn test_debug_formatting() {
    // Test that types implement Debug
    let task = Task::new("debug test".to_string(), TaskType::Testing);
    let debug_str = format!("{:?}", task);
    assert!(debug_str.contains("debug test"));
    assert!(debug_str.contains("Testing"));

    let status = EvalStatus::Pass;
    let debug_str = format!("{:?}", status);
    assert_eq!(debug_str, "Pass");

    let mode = ExecutionMode::Auto;
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "Auto");
}
