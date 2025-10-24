//! Prompt Frame Structure
//!
//! Reproducible prompt envelope that enables replay, cross-model consistency,
//! and bandit learning. Same inputs produce byte-identical prompt frames.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Budget limits for file changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Budgets {
    pub max_files: usize,
    pub max_loc: usize,
}

/// Evidence bundle containing failure information from previous iterations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub failing_tests: Vec<TestFailure>,
    pub lint_errors: Vec<LintError>,
    pub type_errors: Vec<TypeError>,
    pub prior_diffs_summary: Vec<DiffSummary>,
}

impl Default for EvidenceBundle {
    fn default() -> Self {
        Self {
            failing_tests: Vec::new(),
            lint_errors: Vec::new(),
            type_errors: Vec::new(),
            prior_diffs_summary: Vec::new(),
        }
    }
}

/// Test failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub test_name: String,
    pub failure_message: String,
    pub file_path: Option<PathBuf>,
    pub line_number: Option<usize>,
}

/// Lint error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintError {
    pub rule_id: String,
    pub message: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: Option<usize>,
    pub severity: LintSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

/// Type error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeError {
    pub error_code: String,
    pub message: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: Option<usize>,
}

/// Summary of a previous diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub iteration: usize,
    pub files_changed: Vec<PathBuf>,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub score_achieved: f64,
}

/// Prompt frame structure for deterministic prompt generation
///
/// This structure ensures that the same inputs produce byte-identical
/// prompt frames across runs, enabling replay and debugging.
#[derive(Debug, Clone, Serialize)]
pub struct PromptFrame<'a> {
    /// System-level invariants that must be followed
    pub system_invariants: &'a [String],
    
    /// Task description with goal and constraints
    pub task_brief: &'a str,
    
    /// Allow-listed paths that can be modified
    pub scope: &'a [PathBuf],
    
    /// Budget limits for this task
    pub budgets: Budgets,
    
    /// Evidence from previous iterations
    pub evidence: EvidenceBundle,
    
    /// Current iteration index (0-based)
    pub iteration_index: usize,
    
    /// JSON Schema for tool-call validation
    pub tool_schema: &'a serde_json::Value,
}

impl<'a> PromptFrame<'a> {
    /// Create a new prompt frame
    pub fn new(
        system_invariants: &'a [String],
        task_brief: &'a str,
        scope: &'a [PathBuf],
        budgets: Budgets,
        evidence: EvidenceBundle,
        iteration_index: usize,
        tool_schema: &'a serde_json::Value,
    ) -> Self {
        Self {
            system_invariants,
            task_brief,
            scope,
            budgets,
            evidence,
            iteration_index,
            tool_schema,
        }
    }
    
    /// Serialize to JSON for deterministic output
    ///
    /// Uses stable field ordering to ensure byte-identical output
    /// for the same inputs across runs.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        // Use stable serialization with sorted keys
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(
            &mut buf,
            serde_json::ser::PrettyFormatter::with_indent(b"  "),
        );
        self.serialize(&mut ser)?;
        
        Ok(String::from_utf8(buf).expect("JSON should be valid UTF-8"))
    }
    
    /// Generate the full prompt text for the model
    pub fn generate_prompt(&self) -> String {
        let mut prompt = String::new();
        
        // System invariants
        prompt.push_str("# System Invariants\n\n");
        for invariant in self.system_invariants {
            prompt.push_str("- ");
            prompt.push_str(invariant);
            prompt.push('\n');
        }
        prompt.push('\n');
        
        // Task brief
        prompt.push_str("# Task\n\n");
        prompt.push_str(self.task_brief);
        prompt.push_str("\n\n");
        
        // Scope
        prompt.push_str("# Allowed Paths\n\n");
        for path in self.scope {
            prompt.push_str("- ");
            prompt.push_str(&path.display().to_string());
            prompt.push('\n');
        }
        prompt.push_str("\n");
        
        // Budgets
        prompt.push_str("# Budget Limits\n\n");
        prompt.push_str(&format!("- Max files: {}\n", self.budgets.max_files));
        prompt.push_str(&format!("- Max lines of code: {}\n", self.budgets.max_loc));
        prompt.push_str("\n");
        
        // Evidence (if any)
        if !self.evidence.failing_tests.is_empty()
            || !self.evidence.lint_errors.is_empty()
            || !self.evidence.type_errors.is_empty()
        {
            prompt.push_str("# Evidence from Previous Iteration\n\n");
            
            if !self.evidence.failing_tests.is_empty() {
                prompt.push_str("## Failing Tests\n\n");
                for test in &self.evidence.failing_tests {
                    prompt.push_str(&format!("- {}: {}\n", test.test_name, test.failure_message));
                }
                prompt.push('\n');
            }
            
            if !self.evidence.lint_errors.is_empty() {
                prompt.push_str("## Lint Errors\n\n");
                for error in &self.evidence.lint_errors {
                    prompt.push_str(&format!(
                        "- {}:{} [{}] {}\n",
                        error.file_path.display(),
                        error.line_number,
                        error.rule_id,
                        error.message
                    ));
                }
                prompt.push('\n');
            }
            
            if !self.evidence.type_errors.is_empty() {
                prompt.push_str("## Type Errors\n\n");
                for error in &self.evidence.type_errors {
                    prompt.push_str(&format!(
                        "- {}:{} [{}] {}\n",
                        error.file_path.display(),
                        error.line_number,
                        error.error_code,
                        error.message
                    ));
                }
                prompt.push('\n');
            }
        }
        
        // Iteration context
        prompt.push_str(&format!("# Iteration {}\n\n", self.iteration_index + 1));
        
        // Tool schema
        prompt.push_str("# Tool Call Format\n\n");
        prompt.push_str("You must respond with a valid JSON tool call matching this schema:\n\n");
        prompt.push_str("```json\n");
        prompt.push_str(&serde_json::to_string_pretty(self.tool_schema).unwrap_or_default());
        prompt.push_str("\n```\n");
        
        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_prompt_frame_deterministic() {
        let invariants = vec![
            "Edit only allow-listed paths".to_string(),
            "Use JSON tool-call format only".to_string(),
        ];
        let task = "Fix the syntax error in src/main.rs";
        let scope = vec![PathBuf::from("src/")];
        let budgets = Budgets {
            max_files: 5,
            max_loc: 100,
        };
        let evidence = EvidenceBundle::default();
        let schema = json!({
            "type": "patch",
            "changes": []
        });
        
        let frame1 = PromptFrame::new(
            &invariants,
            task,
            &scope,
            budgets.clone(),
            evidence.clone(),
            0,
            &schema,
        );
        
        let frame2 = PromptFrame::new(
            &invariants,
            task,
            &scope,
            budgets,
            evidence,
            0,
            &schema,
        );
        
        // Same inputs should produce identical JSON
        assert_eq!(frame1.to_json().unwrap(), frame2.to_json().unwrap());
    }
    
    #[test]
    fn test_prompt_generation() {
        let invariants = vec!["Test invariant".to_string()];
        let task = "Test task";
        let scope = vec![PathBuf::from("src/")];
        let budgets = Budgets {
            max_files: 5,
            max_loc: 100,
        };
        let evidence = EvidenceBundle::default();
        let schema = json!({});
        
        let frame = PromptFrame::new(
            &invariants,
            task,
            &scope,
            budgets,
            evidence,
            0,
            &schema,
        );
        
        let prompt = frame.generate_prompt();
        
        // Verify key sections are present
        assert!(prompt.contains("# System Invariants"));
        assert!(prompt.contains("# Task"));
        assert!(prompt.contains("# Allowed Paths"));
        assert!(prompt.contains("# Budget Limits"));
        assert!(prompt.contains("# Iteration 1"));
        assert!(prompt.contains("# Tool Call Format"));
    }
}

