//! Type adapters for integration with other crates
//!
//! Provides conversion functions between different task representations
//! to enable integration between orchestration, research, and workers.

use schemars::JsonSchema;
use crate::prompting_types::{Task as ResearchTask, TaskType};
use std::collections::HashMap;

/// Convert a simple string task ID to a Research Task
///
/// This is a helper for benchmarks and testing that creates a minimal
/// research task from basic parameters.
pub fn create_research_task(
    id: &str,
    description: String,
    context: Option<String>,
) -> ResearchTask {
    ResearchTask {
        id: uuid::Uuid::parse_str(id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
        description,
        task_type: infer_task_type(&description),
        target_files: Vec::new(),
        constraints: HashMap::new(),
        refinement_context: context.map_or_else(Vec::new, |c| vec![c]),
    }
}

/// Infer task type from description
fn infer_task_type(description: &str) -> TaskType {
    let desc_lower = description.to_lowercase();
    
    if desc_lower.contains("test") || desc_lower.contains("spec") {
        TaskType::Testing
    } else if desc_lower.contains("refactor") || desc_lower.contains("restructure") {
        TaskType::CodeRefactor
    } else if desc_lower.contains("review") || desc_lower.contains("analyze") {
        TaskType::CodeReview
    } else if desc_lower.contains("document") || desc_lower.contains("comment") {
        TaskType::Documentation
    } else if desc_lower.contains("plan") || desc_lower.contains("design") {
        TaskType::Planning
    } else if desc_lower.contains("research") || desc_lower.contains("investigate") {
        TaskType::Research
    } else {
        TaskType::CodeGeneration
    }
}

/// Convert Research Task to simple benchmark-compatible format

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimpleTask {
    pub id: String,
    pub description: String,
    pub context: Option<String>,
}

impl From<ResearchTask> for SimpleTask {
    fn from(task: ResearchTask) -> Self {
        Self {
            id: task.id.to_string(),
            description: task.description,
            context: task.refinement_context.first().cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_inference() {
        assert_eq!(
            infer_task_type("Write unit tests for the API"),
            TaskType::Testing
        );
        
        assert_eq!(
            infer_task_type("Refactor the authentication module"),
            TaskType::CodeRefactor
        );
        
        assert_eq!(
            infer_task_type("Generate a user login component"),
            TaskType::CodeGeneration
        );
    }

    #[test]
    fn test_create_research_task() {
        let task = create_research_task(
            "test-id",
            "Test task description".to_string(),
            Some("Test context".to_string()),
        );
        
        assert_eq!(task.description, "Test task description");
        assert_eq!(task.refinement_context.len(), 1);
        assert_eq!(task.refinement_context[0], "Test context");
    }

    #[test]
    fn test_simple_task_conversion() {
        let research_task = ResearchTask {
            id: uuid::Uuid::new_v4(),
            description: "Test task".to_string(),
            task_type: TaskType::Testing,
            target_files: Vec::new(),
            constraints: HashMap::new(),
            refinement_context: vec!["context".to_string()],
        };
        
        let simple: SimpleTask = research_task.into();
        assert_eq!(simple.description, "Test task");
        assert_eq!(simple.context, Some("context".to_string()));
    }
}
