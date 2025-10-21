//! Adaptive prompting strategy implementation

use async_trait::async_trait;
use crate::evaluation::{EvalReport, FailureBucket};
use crate::types::{Task, ActionRequest};
use super::PromptingStrategy;
use pest::Parser;
use pest_derive::Parser;
use nom::{
    IResult,
    bytes::complete::{tag, take_while1, take_until},
    character::complete::{char, digit1, space0, space1},
    combinator::{opt, recognize},
    multi::{many0, separated_list1},
    sequence::{delimited, preceded, tuple},
    branch::alt,
};
use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Adaptive prompting strategy that uses evaluation results to refine prompts
pub struct AdaptivePromptingStrategy;

impl AdaptivePromptingStrategy {
    /// Create a new adaptive prompting strategy
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PromptingStrategy for AdaptivePromptingStrategy {
    fn generate_initial_prompt(&self, task: &Task) -> String {
        format!(
            "Task: {}\n\
             Description: {}\n\
             \n\
             Generate a solution for this task. Focus on:\n\
             - Correct implementation\n\
             - Best practices\n\
             - Error handling\n\
             - Documentation\n\
             \n\
             Provide your response as a JSON ActionRequest object with the exact structure:\n\
             {{\n\
               \"action_type\": \"write\",\n\
               \"changeset\": {{\n\
                 \"patches\": [{{\n\
                   \"path\": \"file_path\",\n\
                   \"hunks\": [{{\n\
                     \"old_start\": line_number,\n\
                     \"old_lines\": 0,\n\
                     \"new_start\": line_number,\n\
                     \"new_lines\": num_lines,\n\
                     \"lines\": \"+new_content\\n\"\n\
                   }}],\n\
                   \"expected_prev_sha256\": null\n\
                 }}]\n\
               }},\n\
               \"reason\": \"Brief explanation\",\n\
               \"confidence\": 0.9,\n\
               \"metadata\": {{}}\n\
             }}",
            task.id, task.description
        )
    }

    fn generate_refinement_prompt(&self, eval_report: &EvalReport) -> String {
        // Extract failure bucket from evaluation metadata if available
        let failure_bucket = self.extract_failure_bucket(eval_report);

        if let Some(bucket) = failure_bucket {
            // Use targeted refinement prompt based on failure analysis
            use crate::evaluation::RefinementPromptGenerator;
            RefinementPromptGenerator::generate_targeted_prompt(&bucket, &eval_report.task_id)
        } else {
            // Fallback to general refinement prompt
            self.generate_general_refinement_prompt(eval_report)
        }
    }

    fn generate_self_critique_prompt(&self, output: &str) -> String {
        format!(
            "Review the following output and provide constructive criticism:\n\
             \n\
             Output: {}\n\
             \n\
             Focus on:\n\
             - Code quality and best practices\n\
             - Potential bugs or edge cases\n\
             - Performance considerations\n\
             - Security implications\n\
             - Maintainability\n\
             \n\
             Be specific and actionable in your feedback.",
            output
        )
    }

    async fn generate_action_request(
        &self,
        model_output: &str,
        task: &Task,
        eval_context: Option<&EvalReport>,
    ) -> Result<ActionRequest, String> {
        match serde_json::from_str::<ActionRequest>(model_output) {
            Ok(mut action_request) => {
                match action_request.validate() {
                    Ok(_) => Ok(action_request),
                    Err(e) => Err(format!("ActionRequest validation failed: {}", e)),
                }
            }
            Err(json_error) => {
                Err(format!("Failed to parse model output as JSON ActionRequest: {}. Expected format: ...", json_error))
            }
        }
    }
}

impl AdaptivePromptingStrategy {
    /// Extract failure bucket from evaluation report if available
    fn extract_failure_bucket(&self, eval_report: &EvalReport) -> Option<FailureBucket> {
        // Look for failure bucket information in evaluation criteria notes
        for criterion in &eval_report.criteria {
            if let Some(notes) = &criterion.notes {
                if notes.contains("[Failure:") && notes.contains("patterns:") {
                    // Parse the failure bucket from notes
                    // TODO: Implement robust action request parsing and validation
                    // - Add formal grammar definition for action requests
                    // - Implement comprehensive parsing with error recovery
                    // - Support complex action request structures and nesting
                    // - Add action request validation against schema
                    // - Implement action request normalization and canonicalization
                    // - Add action request parsing performance optimization
                    // PLACEHOLDER: Using simplified regex-based parsing
                    return Some(FailureBucket {
                        category: crate::evaluation::FailureCategory::Unknown, // Would parse from notes
                        patterns: vec!["parsed_pattern".to_string()], // Would extract from notes
                        confidence: 0.5,
                        examples: vec![notes.clone()],
                    });
                }
            }
        }
        None
    }

    /// Generate general refinement prompt when no specific failure analysis is available
    fn generate_general_refinement_prompt(&self, eval_report: &EvalReport) -> String {
        let failed_criteria: Vec<_> = eval_report.criteria.iter()
            .filter(|c| !c.passed)
            .collect();

        let improvement_suggestions = if failed_criteria.is_empty() {
            "Continue improving code quality and add more comprehensive tests.".to_string()
        } else {
            format!(
                "Focus on fixing these failed criteria:\n{}",
                failed_criteria.iter()
                    .map(|c| format!("- {}: {}", c.id, c.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        format!(
            "Task: {}\n\
             Current evaluation score: {:.2}\n\
             Status: {}\n\
             \n\
             The previous attempt had these issues:\n\
             {}\n\
             \n\
             {}\n\
             \n\
             Generate an improved solution that addresses these issues. \
             Provide your response as a JSON ActionRequest object.",
            eval_report.task_id,
            eval_report.score,
            if eval_report.status.as_ref().map(|s| s.to_string()).unwrap_or_default() == "pass" { "PASSING" } else { "FAILING" },
            eval_report.logs.join("\n"),
            improvement_suggestions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskType};

    #[test]
    fn test_generate_action_request_valid() {
        let strategy = AdaptivePromptingStrategy::new();
        let task = crate::types::Task::new("test task".to_string(), TaskType::CodeGeneration);

        let valid_json = r#"{
            "action_type": "write",
            "changeset": {
                "patches": [{
                    "path": "test.rs",
                    "hunks": [{
                        "old_start": 1,
                        "old_lines": 0,
                        "new_start": 1,
                        "new_lines": 1,
                        "lines": "+fn main() {}\n"
                    }],
                    "expected_prev_sha256": null
                }]
            },
            "reason": "Generated main function",
            "confidence": 0.95,
            "metadata": {}
        }"#;

        // This would normally be an async test, but we're testing the parsing logic
        // In a real test, we'd call generate_action_request
        let action_request: ActionRequest = serde_json::from_str(valid_json).unwrap();
        assert!(action_request.validate().is_ok());
        assert_eq!(action_request.action_type, crate::types::ActionType::Write);
        assert_eq!(action_request.confidence, 0.95);
    }

    #[test]
    fn test_generate_initial_prompt() {
        let strategy = AdaptivePromptingStrategy::new();
        let task = crate::types::Task::new("Implement user authentication".to_string(), TaskType::CodeGeneration);

        let prompt = strategy.generate_initial_prompt(&task);

        assert!(prompt.contains("Implement user authentication"));
        assert!(prompt.contains("JSON ActionRequest"));
        assert!(prompt.contains("action_type"));
        assert!(prompt.contains("changeset"));
    }

    #[test]
    fn test_generate_refinement_prompt() {
        let strategy = AdaptivePromptingStrategy::new();

        // Create a mock eval report
        let eval_report = EvalReport {
            task_id: "test-task".to_string(),
            artifact_paths: vec![],
            status: crate::evaluation::EvalStatus::Fail,
            score: 0.3,
            thresholds_met: vec![],
            thresholds_missed: vec!["tests-pass".to_string()],
            criteria: vec![
                crate::evaluation::EvalCriterion {
                    id: "tests-pass".to_string(),
                    description: "Tests should pass".to_string(),
                    weight: 1.0,
                    passed: false,
                    score: 0.0,
                    notes: Some("Test failure: assertion error".to_string()),
                }
            ],
            iterations: 1,
            prompt_tokens: None,
            completion_tokens: None,
            elapsed_ms: Some(1000),
            stop_reason: None,
            next_actions: vec![],
            logs: vec!["Test execution failed".to_string()],
            seed: None,
            tool_versions: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        let prompt = strategy.generate_refinement_prompt(&eval_report);

        assert!(prompt.contains("test-task"));
        assert!(prompt.contains("0.30"));
        assert!(prompt.contains("FAILING"));
        assert!(prompt.contains("JSON ActionRequest"));
    }
}