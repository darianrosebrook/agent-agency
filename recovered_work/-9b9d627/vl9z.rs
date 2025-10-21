//! CAWS (Constitutional AI Workflow System) compliance evaluator

use async_trait::async_trait;
use std::collections::HashSet;

use super::{Evaluator, EvalContext, EvalCriterion, EvaluationError};
use crate::types::{Artifact, TaskType};

/// CAWS compliance rules
#[derive(Debug, Clone)]
pub struct CawsRules {
    pub require_file_headers: bool,
    pub require_docstrings: bool,
    pub ban_todos_placeholders: bool,
    pub require_type_safety: bool,
    pub require_error_handling: bool,
    pub max_file_length: usize,
    pub max_function_length: usize,
}

impl Default for CawsRules {
    fn default() -> Self {
        Self {
            require_file_headers: true,
            require_docstrings: true,
            ban_todos_placeholders: true,
            require_type_safety: true,
            require_error_handling: true,
            max_file_length: 1000,
            max_function_length: 50,
        }
    }
}

/// CAWS compliance evaluator
pub struct CawsEvaluator {
    rules: CawsRules,
}

impl CawsEvaluator {
    /// Create a new CAWS evaluator
    pub fn new() -> Self {
        Self {
            rules: CawsRules::default(),
        }
    }

    /// Create with custom rules
    pub fn with_rules(rules: CawsRules) -> Self {
        Self { rules }
    }

    /// Check for file headers
    fn evaluate_file_headers(&self, artifacts: &[Artifact]) -> EvalCriterion {
        let mut violations = Vec::new();

        for artifact in artifacts {
            if self.rules.require_file_headers {
                let has_header = artifact.content.starts_with("/**")
                    || artifact.content.starts_with("//!")
                    || artifact.content.starts_with("// @author")
                    || artifact.content.starts_with("/*")
                    && artifact.content.contains("@author");

                if !has_header {
                    violations.push(format!("Missing header: {}", artifact.file_path));
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "file-headers-present".to_string(),
            description: "All files have proper headers".to_string(),
            weight: 0.15,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("All files have headers".to_string())
            } else {
                Some(format!("Missing headers in: {}", violations.len()))
            },
        }
    }

    /// Check for TODOs and PLACEHOLDERs
    fn evaluate_todos_placeholders(&self, artifacts: &[Artifact]) -> EvalCriterion {
        let mut violations = Vec::new();

        for artifact in artifacts {
            let content = artifact.content.to_lowercase();

            if self.rules.ban_todos_placeholders {
                let todo_patterns = [
                    "// todo:",
                    "// placeholder:",
                    "// mock data:",
                    "// fixme:",
                    "# todo",
                    "# placeholder",
                    "# mock data",
                    "# fixme",
                ];

                for pattern in &todo_patterns {
                    if content.contains(pattern) {
                        violations.push(format!("Found '{}' in {}", pattern, artifact.file_path));
                    }
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "no-todos-placeholders".to_string(),
            description: "No TODOs, PLACEHOLDERs, or MOCK DATA in production code".to_string(),
            weight: 0.2,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("No forbidden markers found".to_string())
            } else {
                Some(format!("Found {} violations", violations.len()))
            },
        }
    }

    /// Check for type safety (TypeScript/Rust)
    fn evaluate_type_safety(&self, artifacts: &[Artifact]) -> EvalCriterion {
        let mut violations = Vec::new();

        for artifact in artifacts {
            if self.rules.require_type_safety {
                let content = &artifact.content;

                // TypeScript checks
                if artifact.file_path.ends_with(".ts") || artifact.file_path.ends_with(".tsx") {
                    // Check for any: any
                    if content.contains(": any") || content.contains("<any>") {
                        violations.push(format!("TypeScript 'any' type in {}", artifact.file_path));
                    }

                    // Check for missing return types on functions
                    let function_regex = regex::Regex::new(r"function\s+\w+\s*\([^)]*\)\s*:").unwrap();
                    if !function_regex.is_match(content) && content.contains("function ") {
                        violations.push(format!("Missing return type annotation in {}", artifact.file_path));
                    }
                }

                // Rust checks
                if artifact.file_path.ends_with(".rs") {
                    // Check for unwrap() without expect()
                    if content.contains(".unwrap()") {
                        violations.push(format!("Unsafe unwrap() in {}", artifact.file_path));
                    }
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "type-safety".to_string(),
            description: "Code follows type safety guidelines".to_string(),
            weight: 0.2,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("Type safety checks passed".to_string())
            } else {
                Some(format!("Type violations: {}", violations.len()))
            },
        }
    }

    /// Check for error handling
    fn evaluate_error_handling(&self, artifacts: &[Artifact]) -> EvalCriterion {
        let mut violations = Vec::new();

        for artifact in artifacts {
            if self.rules.require_error_handling {
                let content = &artifact.content;

                // Check for try-catch in TypeScript/JavaScript
                if (artifact.file_path.ends_with(".ts") || artifact.file_path.ends_with(".js"))
                    && content.contains("async") && !content.contains("try") {
                    violations.push(format!("Async function without try-catch in {}", artifact.file_path));
                }

                // Check for Result handling in Rust
                if artifact.file_path.ends_with(".rs") {
                    // Look for .unwrap() or .expect() calls
                    let unwrap_count = content.matches(".unwrap()").count();
                    let expect_count = content.matches(".expect(").count();

                    if unwrap_count > 0 || expect_count > 0 {
                        violations.push(format!("Unsafe error handling in {} (unwrap: {}, expect: {})",
                            artifact.file_path, unwrap_count, expect_count));
                    }
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "error-handling".to_string(),
            description: "Proper error handling implemented".to_string(),
            weight: 0.15,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("Error handling checks passed".to_string())
            } else {
                Some(format!("Error handling issues: {}", violations.len()))
            },
        }
    }

    /// Check file and function lengths
    fn evaluate_code_size(&self, artifacts: &[Artifact]) -> EvalCriterion {
        let mut violations = Vec::new();

        for artifact in artifacts {
            let lines: Vec<&str> = artifact.content.lines().collect();
            let file_length = lines.len();

            if file_length > self.rules.max_file_length {
                violations.push(format!("File too long: {} lines (max {}) in {}",
                    file_length, self.rules.max_file_length, artifact.file_path));
            }

            // Check function lengths (basic heuristic)
            let mut current_function_lines = 0;
            let mut in_function = false;

            for line in &lines {
                let trimmed = line.trim();

                // Detect function starts
                if trimmed.starts_with("fn ") || trimmed.starts_with("function ")
                    || trimmed.starts_with("pub fn ") || trimmed.contains("=>") {
                    if in_function && current_function_lines > self.rules.max_function_length {
                        violations.push(format!("Function too long: {} lines in {}",
                            current_function_lines, artifact.file_path));
                    }
                    in_function = true;
                    current_function_lines = 0;
                } else if in_function {
                    current_function_lines += 1;

                    // Detect function ends
                    if trimmed.is_empty() && current_function_lines > self.rules.max_function_length {
                        violations.push(format!("Function too long: {} lines in {}",
                            current_function_lines, artifact.file_path));
                        in_function = false;
                    }
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "code-size-appropriate".to_string(),
            description: "Code size follows guidelines".to_string(),
            weight: 0.15,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("Code size within limits".to_string())
            } else {
                Some(format!("Size violations: {}", violations.len()))
            },
        }
    }

    /// Check for documentation
    fn evaluate_documentation(&self, artifacts: &[Artifact]) -> EvalCriterion {
        let mut undocumented_items = Vec::new();

        for artifact in artifacts {
            if self.rules.require_docstrings {
                let content = &artifact.content;

                // TypeScript/JavaScript function documentation
                if artifact.file_path.ends_with(".ts") || artifact.file_path.ends_with(".js") {
                    let function_regex = regex::Regex::new(r"(export\s+)?(?:async\s+)?function\s+(\w+)").unwrap();
                    for capture in function_regex.captures_iter(content) {
                        if let Some(func_name_match) = capture.get(2) {
                            let func_name = func_name_match.as_str();
                            let doc_pattern = format!("/\\*\\*\\s*\\n\\s*\\* {}\\s", regex::escape(func_name));

                            if let Ok(doc_regex) = regex::Regex::new(&doc_pattern) {
                                if !doc_regex.is_match(content) {
                                    undocumented_items.push(format!("Function {} in {}", func_name, artifact.file_path));
                                }
                            }
                        }
                    }
                }

                // Rust function documentation
                if artifact.file_path.ends_with(".rs") {
                    let function_regex = regex::Regex::new(r"(pub\s+)?(?:async\s+)?fn\s+(\w+)").unwrap();
                    for capture in function_regex.captures_iter(content) {
                        if let Some(func_name_match) = capture.get(2) {
                            let func_name = func_name_match.as_str();
                            let doc_pattern = format!("/// {}", regex::escape(func_name));

                            if !content.contains(&doc_pattern) && !content.contains("//!") {
                                undocumented_items.push(format!("Function {} in {}", func_name, artifact.file_path));
                            }
                        }
                    }
                }
            }
        }

        let passed = undocumented_items.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "documentation-complete".to_string(),
            description: "Public APIs are documented".to_string(),
            weight: 0.15,
            passed,
            score,
            notes: if undocumented_items.is_empty() {
                Some("All public items documented".to_string())
            } else {
                Some(format!("Undocumented items: {}", undocumented_items.len()))
            },
        }
    }
}

#[async_trait]
impl Evaluator for CawsEvaluator {
    async fn evaluate(&self, artifacts: &[Artifact], _context: &EvalContext) -> Result<Vec<EvalCriterion>, EvaluationError> {
        let mut criteria = Vec::new();

        criteria.push(self.evaluate_file_headers(artifacts));
        criteria.push(self.evaluate_todos_placeholders(artifacts));
        criteria.push(self.evaluate_type_safety(artifacts));
        criteria.push(self.evaluate_error_handling(artifacts));
        criteria.push(self.evaluate_code_size(artifacts));
        criteria.push(self.evaluate_documentation(artifacts));

        Ok(criteria)
    }

    fn applies_to(&self, task_type: &TaskType) -> bool {
        // CAWS compliance applies to all code-related tasks
        matches!(task_type, TaskType::CodeFix | TaskType::CodeGeneration | TaskType::DesignTokenApplication)
    }

    fn evaluator_type(&self) -> &'static str {
        "caws"
    }
}
