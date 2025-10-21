//! Language-specific types and enums for CAWS analysis
//!
//! This module contains types related to programming language detection,
//! analysis results, and language-specific violations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Programming language types for AST analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    Cpp,
    C,
    Sql,
    Markdown,
    YAML,
    JSON,
    TOML,
    Unknown,
}

impl ProgrammingLanguage {
    /// Detect programming language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => ProgrammingLanguage::Rust,
            "ts" | "tsx" => ProgrammingLanguage::TypeScript,
            "js" | "jsx" | "mjs" | "cjs" => ProgrammingLanguage::JavaScript,
            "py" => ProgrammingLanguage::Python,
            "go" => ProgrammingLanguage::Go,
            "java" => ProgrammingLanguage::Java,
            "cpp" | "cc" | "cxx" => ProgrammingLanguage::Cpp,
            "c" => ProgrammingLanguage::C,
            "sql" => ProgrammingLanguage::Sql,
            "md" => ProgrammingLanguage::Markdown,
            "yml" | "yaml" => ProgrammingLanguage::YAML,
            "json" => ProgrammingLanguage::JSON,
            "toml" => ProgrammingLanguage::TOML,
            _ => ProgrammingLanguage::Unknown,
        }
    }
}

/// Language analysis result
#[derive(Debug, Clone)]
pub struct LanguageAnalysisResult {
    pub language: ProgrammingLanguage,
    pub complexity_score: f32,
    pub violations: Vec<LanguageViolation>,
    pub warnings: Vec<LanguageWarning>,
    pub metrics: HashMap<String, f32>,
}

/// Language-specific violation
#[derive(Debug, Clone)]
pub struct LanguageViolation {
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub location: SourceLocation,
    pub suggestion: Option<String>,
}

/// Language-specific warning
#[derive(Debug, Clone)]
pub struct LanguageWarning {
    pub rule_id: String,
    pub message: String,
    pub location: SourceLocation,
    pub suggestion: Option<String>,
}

/// Source code location
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
