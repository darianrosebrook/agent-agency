//! Rust language analyzer for CAWS
//!
//! This module provides Rust-specific code analysis including
//! complexity calculation, violation detection, and best practices checking.

use super::LanguageAnalyzer;
use crate::caws::language_types::{LanguageAnalysisResult, ProgrammingLanguage, ViolationSeverity, LanguageViolation, LanguageWarning, SourceLocation};
use std::collections::HashMap;

/// Rust-specific analyzer
#[derive(Debug)]
pub struct RustAnalyzer;

impl RustAnalyzer {
    /// Create a new Rust analyzer
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn analyze(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        // TODO: Implement actual Rust analysis
        LanguageAnalysisResult {
            language: ProgrammingLanguage::Rust,
            complexity_score: 0.0,
            violations: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        }
    }

    fn language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Rust
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "rs")
    }
}


// Moved from caws_checker.rs: RustAnalyzer struct
#[derive(Debug)]
pub struct RustAnalyzer;

// REFACTOR: [send RustAnalyzer impl block to caws/analyzers/rust.rs]
impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }
}



// Moved from caws_checker.rs: RustAnalyzer impl block
impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }
}



// Moved from caws_checker.rs: LanguageAnalyzer impl for RustAnalyzer
impl LanguageAnalyzer for RustAnalyzer {
    fn analyze_file_modification(
        &self,
        modification: &CouncilFileModification,
    ) -> Result<LanguageAnalysisResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Analyze Rust-specific issues
        if let Some(content) = &modification.content {
            // Check for unsafe code
            if content.contains("unsafe") {
                violations.push(LanguageViolation {
                    rule: "Rust Unsafe Code".to_string(),
                    severity: ViolationSeverity::High,
                    description: "Unsafe code detected".to_string(),
                    location: None,
                    suggestion: Some(
                        "Review unsafe code usage and ensure proper justification".to_string(),
                    ),
                    constitutional_ref: None,
                });
            }

            // Check for unwrap() usage
            let unwrap_count = content.matches("unwrap()").count();
            if unwrap_count > 3 {
                warnings.push(LanguageWarning {
                    rule: "Rust Error Handling".to_string(),
                    description: format!("{} unwrap() calls detected", unwrap_count),
                    location: None,
                    suggestion: Some(
                        "Consider using proper error handling instead of unwrap()".to_string(),
                    ),
                });
            }
        }

        // Implement sophisticated code complexity analysis
        let complexity_score = if let Some(content) = &modification.content {
            self.analyze_code_complexity(content)
        } else {
            0.1
        };

        // Implement comprehensive surgical change analysis
        let surgical_change_score = if let Some(diff) = &modification.diff {
            self.analyze_surgical_change(diff)
        } else {
            0.5
        };

        // Calculate change complexity
        let change_complexity = self.calculate_change_complexity(
            modification.diff.as_deref().unwrap_or(""),
            modification.content.as_deref(),
        )?;

        Ok(LanguageAnalysisResult {
            violations,
            warnings,
            complexity_score,
            surgical_change_score,
            change_complexity,
        })
    }

    fn language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Rust
    }

    fn calculate_change_complexity(
        &self,
        diff: &str,
        _content: Option<&str>,
    ) -> Result<ChangeComplexity> {
        let diff_lines = diff.lines().count() as u32;
        let structural_changes =
            diff.matches("struct ").count() as u32 + diff.matches("impl ").count() as u32;
        let logical_changes = diff.matches("fn ").count() as u32;
        let dependency_changes =
            diff.matches("use ").count() as u32 + diff.matches("mod ").count() as u32;

        let complexity_score = (structural_changes as f32 * 0.4
            + logical_changes as f32 * 0.3
            + dependency_changes as f32 * 0.3)
            / 10.0;
        let is_surgical = complexity_score < 0.5 && diff_lines < 30;

        Ok(ChangeComplexity {
            structural_changes,
            logical_changes,
            dependency_changes,
            complexity_score,
            is_surgical,
        })
    }
}

