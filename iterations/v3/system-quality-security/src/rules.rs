//! Quality gate rules

use crate::config::{QualityViolation, Severity};
use regex::Regex;
use std::collections::HashMap;

/// Trait for quality rules
pub trait QualityRule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn check_file(&self, file_path: &str, content: &str, config: &crate::config::QualityGateConfig) -> Vec<QualityViolation>;
}

/// God object detection rule
pub struct GodObjectRule;

impl QualityRule for GodObjectRule {
    fn name(&self) -> &str {
        "god-object"
    }

    fn description(&self) -> &str {
        "Detects files that are too large (god objects)"
    }

    fn check_file(&self, file_path: &str, content: &str, config: &crate::config::QualityGateConfig) -> Vec<QualityViolation> {
        let mut violations = Vec::new();
        let line_count = content.lines().count();

        if line_count > config.max_lines_per_file {
            violations.push(QualityViolation {
                rule: self.name().to_string(),
                severity: Severity::Error,
                file: file_path.to_string(),
                line: None,
                column: None,
                message: format!("File has {} lines, exceeds maximum of {}", line_count, config.max_lines_per_file),
                suggestion: Some("Consider splitting this file into smaller, focused modules".to_string()),
                details: Some({
                    let mut details = HashMap::new();
                    details.insert("line_count".to_string(), line_count.into());
                    details.insert("max_lines".to_string(), config.max_lines_per_file.into());
                    details
                }),
            });
        }

        violations
    }
}

/// Duplicate name detection rule
pub struct DuplicateNameRule;

impl QualityRule for DuplicateNameRule {
    fn name(&self) -> &str {
        "duplicate-names"
    }

    fn description(&self) -> &str {
        "Detects excessive duplicate struct/enum names"
    }

    fn check_file(&self, _file_path: &str, _content: &str, _config: &crate::config::QualityGateConfig) -> Vec<QualityViolation> {
        // This rule needs global analysis across all files
        // Will be implemented in the runner
        Vec::new()
    }
}

/// Function complexity rule
pub struct FunctionComplexityRule;

impl QualityRule for FunctionComplexityRule {
    fn name(&self) -> &str {
        "function-complexity"
    }

    fn description(&self) -> &str {
        "Detects functions that are too long"
    }

    fn check_file(&self, file_path: &str, content: &str, config: &crate::config::QualityGateConfig) -> Vec<QualityViolation> {
        let mut violations = Vec::new();
        let function_regex = Regex::new(r"fn\s+\w+\s*\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if function_regex.is_match(line) {
                // Count lines until the next function or end of block
                let mut brace_count = 0;
                let mut function_lines = 0;
                let mut in_function = false;

                for check_line in content.lines().skip(line_num) {
                    function_lines += 1;

                    for ch in check_line.chars() {
                        match ch {
                            '{' => {
                                brace_count += 1;
                                in_function = true;
                            }
                            '}' => {
                                brace_count -= 1;
                                if brace_count == 0 && in_function {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if brace_count == 0 && in_function {
                        break;
                    }
                }

                if function_lines > config.max_lines_per_function {
                    violations.push(QualityViolation {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        file: file_path.to_string(),
                        line: Some(line_num + 1),
                        column: None,
                        message: format!("Function has {} lines, exceeds maximum of {}", function_lines, config.max_lines_per_function),
                        suggestion: Some("Consider breaking this function into smaller, focused functions".to_string()),
                        details: Some({
                            let mut details = HashMap::new();
                            details.insert("function_lines".to_string(), function_lines.into());
                            details.insert("max_lines".to_string(), config.max_lines_per_function.into());
                            details
                        }),
                    });
                }
            }
        }

        violations
    }
}

/// Struct complexity rule
pub struct StructComplexityRule;

impl QualityRule for StructComplexityRule {
    fn name(&self) -> &str {
        "struct-complexity"
    }

    fn description(&self) -> &str {
        "Detects structs with too many fields"
    }

    fn check_file(&self, file_path: &str, content: &str, config: &crate::config::QualityGateConfig) -> Vec<QualityViolation> {
        let mut violations = Vec::new();
        let struct_regex = Regex::new(r"pub struct\s+(\w+)").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = struct_regex.captures(line) {
                let struct_name = captures.get(1).unwrap().as_str();

                // Count fields until the closing brace
                let mut field_count = 0;
                let mut brace_count = 0;
                let mut in_struct = false;

                for check_line in content.lines().skip(line_num) {
                    for ch in check_line.chars() {
                        match ch {
                            '{' => {
                                brace_count += 1;
                                in_struct = true;
                            }
                            '}' => {
                                brace_count -= 1;
                                if brace_count == 0 && in_struct {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if in_struct && brace_count > 0 {
                        // Count pub fields
                        if check_line.contains("pub ") {
                            field_count += 1;
                        }
                    }

                    if brace_count == 0 && in_struct {
                        break;
                    }
                }

                if field_count > config.max_struct_fields {
                    violations.push(QualityViolation {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        file: file_path.to_string(),
                        line: Some(line_num + 1),
                        column: None,
                        message: format!("Struct '{}' has {} fields, exceeds maximum of {}", struct_name, field_count, config.max_struct_fields),
                        suggestion: Some("Consider grouping related fields or splitting into smaller structs".to_string()),
                        details: Some({
                            let mut details = HashMap::new();
                            details.insert("struct_name".to_string(), struct_name.into());
                            details.insert("field_count".to_string(), field_count.into());
                            details.insert("max_fields".to_string(), config.max_struct_fields.into());
                            details
                        }),
                    });
                }
            }
        }

        violations
    }
}

/// Placeholder detection rule
pub struct PlaceholderRule;

impl QualityRule for PlaceholderRule {
    fn name(&self) -> &str {
        "placeholders"
    }

    fn description(&self) -> &str {
        "Detects TODO, PLACEHOLDER, and MOCK comments"
    }

    fn check_file(&self, file_path: &str, content: &str, _config: &crate::config::QualityGateConfig) -> Vec<QualityViolation> {
        let mut violations = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            if line_lower.contains("// todo") ||
               line_lower.contains("// placeholder") ||
               line_lower.contains("// mock") ||
               line_lower.contains("// fixme") ||
               line_lower.contains("// hack") {

                let severity = if line_lower.contains("// todo") && !line_lower.contains("critical") {
                    Severity::Info
                } else {
                    Severity::Warning
                };

                violations.push(QualityViolation {
                    rule: self.name().to_string(),
                    severity,
                    file: file_path.to_string(),
                    line: Some(line_num + 1),
                    column: None,
                    message: format!("Found placeholder comment: {}", line.trim()),
                    suggestion: Some("Replace with actual implementation or remove if no longer needed".to_string()),
                    details: Some({
                        let mut details = HashMap::new();
                        details.insert("line_content".to_string(), line.trim().to_string().into());
                        details
                    }),
                });
            }
        }

        violations
    }
}
