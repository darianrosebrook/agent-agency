//! TypeScript language analyzer for CAWS runtime validation
//!
//! This module provides TypeScript-specific code analysis including
//! complexity calculation, violation detection, and best practices checking.
//! Extracted from workers/src/caws/analyzers/typescript.rs.

use super::{LanguageAnalyzer, LanguageAnalysisResult, ProgrammingLanguage, ViolationSeverity, LanguageViolation, LanguageWarning, SourceLocation};
use std::collections::HashMap;

/// TypeScript-specific analyzer
#[derive(Debug)]
pub struct TypeScriptAnalyzer;

impl TypeScriptAnalyzer {
    /// Create a new TypeScript analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze TypeScript code for complexity and violations
    fn analyze_typescript_code(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut metrics = HashMap::new();

        // Calculate basic complexity metrics
        let lines_of_code = code.lines().count() as f32;
        let function_count = code.matches("function ").count() as f32 + code.matches("=>").count() as f32;
        let class_count = code.matches("class ").count() as f32;
        let interface_count = code.matches("interface ").count() as f32;
        let type_count = code.matches("type ").count() as f32;
        let import_count = code.matches("import ").count() as f32;

        // Store metrics
        metrics.insert("lines_of_code".to_string(), lines_of_code);
        metrics.insert("functions".to_string(), function_count);
        metrics.insert("classes".to_string(), class_count);
        metrics.insert("interfaces".to_string(), interface_count);
        metrics.insert("types".to_string(), type_count);
        metrics.insert("imports".to_string(), import_count);

        // Calculate complexity score based on various factors
        let complexity_score = self.calculate_typescript_complexity(&code, &metrics);

        // Check for common TypeScript violations
        self.check_typescript_violations(&code, file_path, &mut violations, &mut warnings);

        LanguageAnalysisResult {
            language: ProgrammingLanguage::TypeScript,
            complexity_score,
            violations,
            warnings,
            metrics,
        }
    }

    /// Calculate TypeScript-specific complexity score
    fn calculate_typescript_complexity(&self, code: &str, metrics: &HashMap<String, f32>) -> f32 {
        let lines = metrics.get("lines_of_code").unwrap_or(&0.0);
        let functions = metrics.get("functions").unwrap_or(&0.0);
        let classes = metrics.get("classes").unwrap_or(&0.0);
        let interfaces = metrics.get("interfaces").unwrap_or(&0.0);
        let types = metrics.get("types").unwrap_or(&0.0);

        // Base complexity from lines of code
        let mut complexity = *lines * 0.1;

        // Add complexity for functions (higher weight for more functions)
        complexity += *functions * 1.5;

        // Add complexity for type definitions
        complexity += *classes * 2.0;
        complexity += *interfaces * 1.5;
        complexity += *types * 1.0;

        // Check for complex patterns
        let async_count = code.matches("async ").count() as f32;
        let await_count = code.matches("await ").count() as f32;
        let promise_count = code.matches("Promise<").count() as f32;
        let generic_count = code.matches("<").count() as f32;
        let union_count = code.matches("|").count() as f32;
        let intersection_count = code.matches("&").count() as f32;

        complexity += async_count * 2.0;
        complexity += await_count * 1.5;
        complexity += promise_count * 2.5;
        complexity += generic_count * 0.5;
        complexity += union_count * 0.3;
        complexity += intersection_count * 0.5;

        complexity
    }

    /// Check for common TypeScript violations and warnings
    fn check_typescript_violations(&self, code: &str, file_path: &str, violations: &mut Vec<LanguageViolation>, warnings: &mut Vec<LanguageWarning>) {
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

            // Check for any type usage (warning)
            if line.contains(": any") || line.contains("any[]") {
                warnings.push(LanguageWarning {
                    rule_id: "ANY_TYPE".to_string(),
                    message: "Use of 'any' type found - consider more specific typing".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use more specific types instead of 'any'".to_string()),
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

            // Check for missing return types (warning)
            if line.contains("function ") && !line.contains("): ") && !line.contains("=> ") {
                warnings.push(LanguageWarning {
                    rule_id: "MISSING_RETURN_TYPE".to_string(),
                    message: "Function missing return type annotation".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Add explicit return type annotation".to_string()),
                });
            }

            // Check for non-null assertion usage (medium severity)
            if line.contains("!") && (line.contains(".") || line.contains("[")) {
                violations.push(LanguageViolation {
                    rule_id: "NON_NULL_ASSERTION".to_string(),
                    severity: ViolationSeverity::Medium,
                    message: "Non-null assertion operator (!) found - ensure null safety".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use proper null checking instead of non-null assertion".to_string()),
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
        }

        // Check for missing JSDoc on exported functions
        if code.contains("export ") && !code.contains("/**") && !file_path.contains("test") {
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

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn analyze(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        self.analyze_typescript_code(code, file_path)
    }

    fn language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::TypeScript
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "ts" | "tsx")
    }

    fn calculate_change_complexity(&self, diff: &str, _content: Option<&str>) -> Result<f32, String> {
        // Calculate complexity based on diff content
        let added_lines = diff.lines().filter(|line| line.starts_with('+')).count() as f32;
        let removed_lines = diff.lines().filter(|line| line.starts_with('-')).count() as f32;

        // Base complexity from line changes
        let mut complexity = (added_lines + removed_lines) * 0.5;

        // Higher complexity for structural changes
        if diff.contains("function ") || diff.contains("class ") || diff.contains("interface ") || diff.contains("type ") {
            complexity *= 2.0;
        }

        // Higher complexity for type changes
        if diff.contains(": ") || diff.contains("<") || diff.contains("|") || diff.contains("&") {
            complexity *= 1.5;
        }

        // Higher complexity for async changes
        if diff.contains("async ") || diff.contains("await ") || diff.contains("Promise<") {
            complexity *= 2.5;
        }

        Ok(complexity)
    }
}
