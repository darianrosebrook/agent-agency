//! Configuration for quality gates

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityGateConfig {
    /// Maximum lines per file
    pub max_lines_per_file: usize,
    /// Maximum lines per function
    pub max_lines_per_function: usize,
    /// Maximum struct fields
    pub max_struct_fields: usize,
    /// Maximum enum variants
    pub max_enum_variants: usize,
    /// Maximum duplicate names before warning
    pub max_duplicate_names: usize,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Directory patterns to exclude
    pub exclude_dirs: Vec<String>,
    /// Custom rules
    pub custom_rules: HashMap<String, serde_json::Value>,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            max_lines_per_file: 1000,
            max_lines_per_function: 50,
            max_struct_fields: 20,
            max_enum_variants: 15,
            max_duplicate_names: 5,
            exclude_patterns: vec![
                "*.pb.rs".to_string(),        // Generated protobuf files
                "*generated*.rs".to_string(), // Generated files
                "target/".to_string(),
                "node_modules/".to_string(),
            ],
            exclude_dirs: vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                ".git/".to_string(),
            ],
            custom_rules: HashMap::new(),
        }
    }
}

/// Quality gate severity levels
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// Quality violation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityViolation {
    pub rule: String,
    pub severity: Severity,
    pub file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Quality gate results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityGateResults {
    pub violations: Vec<QualityViolation>,
    pub passed: bool,
    pub total_files_checked: usize,
    pub execution_time_ms: u64,
}

impl QualityGateResults {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            passed: true,
            total_files_checked: 0,
            execution_time_ms: 0,
        }
    }

    pub fn add_violation(&mut self, violation: QualityViolation) {
        if violation.severity == Severity::Error {
            self.passed = false;
        }
        self.violations.push(violation);
    }

    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Warning)
            .count()
    }

    pub fn info_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Info)
            .count()
    }
}
