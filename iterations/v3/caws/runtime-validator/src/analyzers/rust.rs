//! Rust language analyzer for CAWS runtime validation
//!
//! This module provides Rust-specific code analysis including
//! complexity calculation, violation detection, and best practices checking.
//! Extracted from workers/src/caws/analyzers/rust.rs.

use super::{LanguageAnalyzer, LanguageAnalysisResult, ProgrammingLanguage, ViolationSeverity, LanguageViolation, LanguageWarning, SourceLocation};
use std::collections::HashMap;

/// Rust-specific analyzer
#[derive(Debug)]
pub struct RustAnalyzer;

impl RustAnalyzer {
    /// Create a new Rust analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze Rust code for complexity and violations
    fn analyze_rust_code(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut metrics = HashMap::new();

        // Calculate basic complexity metrics
        let lines_of_code = code.lines().count() as f32;
        let function_count = code.matches("fn ").count() as f32;
        let struct_count = code.matches("struct ").count() as f32;
        let trait_count = code.matches("trait ").count() as f32;
        let impl_count = code.matches("impl ").count() as f32;

        // Store metrics
        metrics.insert("lines_of_code".to_string(), lines_of_code);
        metrics.insert("functions".to_string(), function_count);
        metrics.insert("structs".to_string(), struct_count);
        metrics.insert("traits".to_string(), trait_count);
        metrics.insert("implementations".to_string(), impl_count);

        // Calculate complexity score based on various factors
        let complexity_score = self.calculate_rust_complexity(&code, &metrics);

        // Check for common Rust violations
        self.check_rust_violations(&code, file_path, &mut violations, &mut warnings);

        LanguageAnalysisResult {
            language: ProgrammingLanguage::Rust,
            complexity_score,
            violations,
            warnings,
            metrics,
        }
    }

    /// Calculate Rust-specific complexity score
    fn calculate_rust_complexity(&self, code: &str, metrics: &HashMap<String, f32>) -> f32 {
        let lines = metrics.get("lines_of_code").unwrap_or(&0.0);
        let functions = metrics.get("functions").unwrap_or(&0.0);
        let structs = metrics.get("structs").unwrap_or(&0.0);
        let traits = metrics.get("traits").unwrap_or(&0.0);
        let impls = metrics.get("implementations").unwrap_or(&0.0);

        // Base complexity from lines of code
        let mut complexity = *lines * 0.1;

        // Add complexity for functions (higher weight for more functions)
        complexity += *functions * 2.0;

        // Add complexity for type definitions
        complexity += *structs * 1.5;
        complexity += *traits * 2.5;
        complexity += *impls * 3.0;

        // Check for complex patterns
        let match_count = code.matches("match ").count() as f32;
        let loop_count = code.matches("for ").count() as f32 + code.matches("while ").count() as f32;
        let async_count = code.matches("async ").count() as f32;
        let unsafe_count = code.matches("unsafe ").count() as f32;

        complexity += match_count * 2.0;
        complexity += loop_count * 1.5;
        complexity += async_count * 3.0;
        complexity += unsafe_count * 5.0;

        complexity
    }

    /// Check for common Rust violations and warnings
    fn check_rust_violations(&self, code: &str, file_path: &str, violations: &mut Vec<LanguageViolation>, warnings: &mut Vec<LanguageWarning>) {
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

            // Check for unwrap() usage (warning for non-test code)
            if line.contains(".unwrap()") && !file_path.contains("test") {
                warnings.push(LanguageWarning {
                    rule_id: "UNWRAP_USAGE".to_string(),
                    message: "Use of unwrap() found - consider proper error handling".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use proper error handling with Result or Option".to_string()),
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

            // Check for unsafe code (high severity)
            if line.contains("unsafe ") {
                violations.push(LanguageViolation {
                    rule_id: "UNSAFE_CODE".to_string(),
                    severity: ViolationSeverity::High,
                    message: "Unsafe code found - requires careful review".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Ensure unsafe code is properly documented and tested".to_string()),
                });
            }

            // Check for panic! usage (medium severity)
            if line.contains("panic!") {
                violations.push(LanguageViolation {
                    rule_id: "PANIC_USAGE".to_string(),
                    severity: ViolationSeverity::Medium,
                    message: "panic! found - consider proper error handling".to_string(),
                    location: SourceLocation {
                        file_path: file_path.to_string(),
                        line: line_number,
                        column: 0,
                        end_line: Some(line_number),
                        end_column: None,
                    },
                    suggestion: Some("Use Result types or proper error handling instead of panic!".to_string()),
                });
            }
        }

        // Check for missing documentation on public items
        if code.contains("pub ") && !code.contains("///") && !file_path.contains("test") {
            warnings.push(LanguageWarning {
                rule_id: "MISSING_DOCS".to_string(),
                message: "Public items should have documentation".to_string(),
                location: SourceLocation {
                    file_path: file_path.to_string(),
                    line: 1,
                    column: 0,
                    end_line: None,
                    end_column: None,
                },
                suggestion: Some("Add documentation comments for public items".to_string()),
            });
        }
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn analyze(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        self.analyze_rust_code(code, file_path)
    }

    fn language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Rust
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "rs")
    }

    fn calculate_change_complexity(&self, diff: &str, _content: Option<&str>) -> Result<f32, String> {
        // Calculate complexity based on diff content
        let added_lines = diff.lines().filter(|line| line.starts_with('+')).count() as f32;
        let removed_lines = diff.lines().filter(|line| line.starts_with('-')).count() as f32;
        let modified_lines = diff.lines().filter(|line| line.starts_with(' ')).count() as f32;

        // Base complexity from line changes
        let mut complexity = (added_lines + removed_lines) * 0.5;

        // Higher complexity for structural changes
        if diff.contains("fn ") || diff.contains("struct ") || diff.contains("trait ") || diff.contains("impl ") {
            complexity *= 2.0;
        }

        // Higher complexity for unsafe changes
        if diff.contains("unsafe ") {
            complexity *= 3.0;
        }

        Ok(complexity)
    }
}
