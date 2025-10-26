//! JavaScript language analyzer for CAWS runtime validation
//!
//! This module provides JavaScript-specific code analysis including
//! complexity calculation, violation detection, and best practices checking.
//! Extracted from workers/src/caws/analyzers/javascript.rs.

use super::{LanguageAnalyzer, LanguageAnalysisResult, ProgrammingLanguage, ViolationSeverity, LanguageViolation, LanguageWarning, SourceLocation};
use std::collections::HashMap;

/// JavaScript-specific analyzer
#[derive(Debug)]
pub struct JavaScriptAnalyzer;

impl JavaScriptAnalyzer {
    /// Create a new JavaScript analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze JavaScript code for complexity and violations
    fn analyze_javascript_code(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut metrics = HashMap::new();

        // Calculate basic complexity metrics
        let lines_of_code = code.lines().count() as f32;
        let function_count = code.matches("function ").count() as f32 + code.matches("=>").count() as f32;
        let class_count = code.matches("class ").count() as f32;
        let var_count = code.matches("var ").count() as f32;
        let let_count = code.matches("let ").count() as f32;
        let const_count = code.matches("const ").count() as f32;
        let import_count = code.matches("import ").count() as f32 + code.matches("require(").count() as f32;

        // Store metrics
        metrics.insert("lines_of_code".to_string(), lines_of_code);
        metrics.insert("functions".to_string(), function_count);
        metrics.insert("classes".to_string(), class_count);
        metrics.insert("variables".to_string(), var_count + let_count + const_count);
        metrics.insert("imports".to_string(), import_count);

        // Calculate complexity score based on various factors
        let complexity_score = self.calculate_javascript_complexity(&code, &metrics);

        // Check for common JavaScript violations
        self.check_javascript_violations(&code, file_path, &mut violations, &mut warnings);

        LanguageAnalysisResult {
            language: ProgrammingLanguage::JavaScript,
            complexity_score,
            violations,
            warnings,
            metrics,
        }
    }

    /// Calculate JavaScript-specific complexity score
    fn calculate_javascript_complexity(&self, code: &str, metrics: &HashMap<String, f32>) -> f32 {
        let lines = metrics.get("lines_of_code").unwrap_or(&0.0);
        let functions = metrics.get("functions").unwrap_or(&0.0);
        let classes = metrics.get("classes").unwrap_or(&0.0);

        // Base complexity from lines of code
        let mut complexity = *lines * 0.1;

        // Add complexity for functions (higher weight for more functions)
        complexity += *functions * 1.5;

        // Add complexity for classes
        complexity += *classes * 2.0;

        // Check for complex patterns
        let async_count = code.matches("async ").count() as f32;
        let await_count = code.matches("await ").count() as f32;
        let promise_count = code.matches("Promise").count() as f32;
        let callback_count = code.matches("=>").count() as f32;
        let closure_count = code.matches("function").count() as f32;

        complexity += async_count * 2.0;
        complexity += await_count * 1.5;
        complexity += promise_count * 2.0;
        complexity += callback_count * 1.0;
        complexity += closure_count * 1.2;

        complexity
    }

    /// Check for common JavaScript violations and warnings
    fn check_javascript_violations(&self, code: &str, file_path: &str, violations: &mut Vec<LanguageViolation>, warnings: &mut Vec<LanguageWarning>) {
        let lines: Vec<&str> = code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_number = (line_num + 1) as u32;

            // Check for TODO comments (warnings)
            if line.contains("TODO") || line.contains("FIXME") {
                warnings.push(LanguageWarning {
                    rule_id: "TODO_COMMENT".to_string(),
                    message: "TODO or FIXME comment found".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Remove TODO/FIXME comments before production".to_string()),
                });
            }

            // Check for var usage (warning)
            if line.contains("var ") {
                warnings.push(LanguageWarning {
                    rule_id: "VAR_USAGE".to_string(),
                    message: "Use of 'var' found - prefer 'let' or 'const'".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use 'let' or 'const' instead of 'var'".to_string()),
                });
            }

            // Check for console.log usage (warning for non-test code)
            if line.contains("console.log") && !file_path.contains("test") {
                warnings.push(LanguageWarning {
                    rule_id: "CONSOLE_LOG".to_string(),
                    message: "console.log found - consider proper logging".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use proper logging library instead of console.log".to_string()),
                });
            }

            // Check for hardcoded strings (warning)
            if line.contains("\"http://") || line.contains("\"https://") {
                warnings.push(LanguageWarning {
                    rule_id: "HARDCODED_URL".to_string(),
                    message: "Hardcoded URL found".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use environment variables or configuration for URLs".to_string()),
                });
            }

            // Check for == usage (warning)
            if line.contains(" == ") && !line.contains(" === ") {
                warnings.push(LanguageWarning {
                    rule_id: "LOOSE_EQUALITY".to_string(),
                    message: "Use of == found - prefer ===".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use strict equality (===) instead of loose equality (==)".to_string()),
                });
            }

            // Check for eval usage (high severity)
            if line.contains("eval(") {
                violations.push(LanguageViolation {
                    rule_id: "EVAL_USAGE".to_string(),
                    severity: ViolationSeverity::High,
                    message: "eval() usage found - security risk".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Avoid eval() usage due to security risks".to_string()),
                });
            }

            // Check for with statement usage (high severity)
            if line.contains("with ") {
                violations.push(LanguageViolation {
                    rule_id: "WITH_STATEMENT".to_string(),
                    severity: ViolationSeverity::High,
                    message: "with statement found - avoid usage".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Avoid with statement - use proper variable scoping".to_string()),
                });
            }

            // Check for missing semicolons (low severity)
            if !line.trim().is_empty() && !line.trim().starts_with("//") && !line.trim().starts_with("/*") && !line.trim().starts_with("*") && !line.trim().starts_with("*/") && !line.trim().ends_with(";") && !line.trim().ends_with("{") && !line.trim().ends_with("}") && !line.contains("if ") && !line.contains("else ") && !line.contains("for ") && !line.contains("while ") {
                warnings.push(LanguageWarning {
                    rule_id: "MISSING_SEMICOLON".to_string(),
                    message: "Missing semicolon at end of statement".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Add semicolon at end of statement".to_string()),
                });
            }
        }

        // Check for missing JSDoc on exported functions
        if code.contains("module.exports") && !code.contains("/**") && !file_path.contains("test") {
            warnings.push(LanguageWarning {
                rule_id: "MISSING_JSDOC".to_string(),
                message: "Exported functions should have JSDoc comments".to_string(),
                location: SourceLocation {
                    file_path: file_path.to_string(),
                    line: 1,
                    column: 0,
                    end_line: None,
                    end_column: None,
                },
                suggestion: Some("Add JSDoc comments for exported functions".to_string()),
            });
        }
    }
}

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn analyze(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        self.analyze_javascript_code(code, file_path)
    }

    fn language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::JavaScript
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "js" | "jsx" | "mjs" | "cjs")
    }

    fn calculate_change_complexity(&self, diff: &str, _content: Option<&str>) -> Result<f32, String> {
        // Calculate complexity based on diff content
        let added_lines = diff.lines().filter(|line| line.starts_with('+')).count() as f32;
        let removed_lines = diff.lines().filter(|line| line.starts_with('-')).count() as f32;

        // Base complexity from line changes
        let mut complexity = (added_lines + removed_lines) * 0.5;

        // Higher complexity for structural changes
        if diff.contains("function ") || diff.contains("class ") || diff.contains("=>") {
            complexity *= 2.0;
        }

        // Higher complexity for async changes
        if diff.contains("async ") || diff.contains("await ") || diff.contains("Promise") {
            complexity *= 2.5;
        }

        // Higher complexity for callback changes
        if diff.contains("=>") || diff.contains("callback") {
            complexity *= 1.5;
        }

        Ok(complexity)
    }
}
