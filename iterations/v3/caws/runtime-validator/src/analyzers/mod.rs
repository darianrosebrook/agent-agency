//! Language-specific analyzers for CAWS runtime validation
//!
//! This module contains analyzers for different programming languages,
//! extracted from workers/src/caws_checker.rs to provide centralized
//! language analysis capabilities for the CAWS runtime validator.

pub mod rust;
pub mod typescript;
pub mod javascript;

#[cfg(test)]
mod test;

// Re-export analyzers
pub use rust::RustAnalyzer;
pub use typescript::TypeScriptAnalyzer;
pub use javascript::JavaScriptAnalyzer;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysisResult {
    pub language: ProgrammingLanguage,
    pub complexity_score: f32,
    pub violations: Vec<LanguageViolation>,
    pub warnings: Vec<LanguageWarning>,
    pub metrics: HashMap<String, f32>,
}

/// Language-specific violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageViolation {
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub location: SourceLocation,
    pub suggestion: Option<String>,
}

/// Language-specific warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageWarning {
    pub rule_id: String,
    pub message: String,
    pub location: SourceLocation,
    pub suggestion: Option<String>,
}

/// Source code location
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Trait for language-specific analysis
pub trait LanguageAnalyzer: Send + Sync + std::fmt::Debug {
    /// Analyze code for violations and complexity
    fn analyze(&self, code: &str, file_path: &str) -> LanguageAnalysisResult;
    
    /// Get the programming language this analyzer handles
    fn language(&self) -> ProgrammingLanguage;
    
    /// Check if the analyzer supports the given file extension
    fn supports_extension(&self, ext: &str) -> bool;

    /// Calculate change complexity for a diff
    fn calculate_change_complexity(&self, diff: &str, content: Option<&str>) -> Result<f32, String>;
}

/// Registry for managing language analyzers
#[derive(Debug)]
pub struct LanguageAnalyzerRegistry {
    analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>>,
}

impl LanguageAnalyzerRegistry {
    /// Create a new registry with default analyzers
    pub fn new() -> Self {
        let mut analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>> = HashMap::new();
        
        // Register default analyzers
        analyzers.insert(ProgrammingLanguage::Rust, Box::new(RustAnalyzer::new()));
        analyzers.insert(ProgrammingLanguage::TypeScript, Box::new(TypeScriptAnalyzer::new()));
        analyzers.insert(ProgrammingLanguage::JavaScript, Box::new(JavaScriptAnalyzer::new()));

        Self { analyzers }
    }

    /// Get analyzer for a specific language
    pub fn get_analyzer(&self, language: &ProgrammingLanguage) -> Option<&dyn LanguageAnalyzer> {
        self.analyzers.get(language).map(|analyzer| analyzer.as_ref())
    }

    /// Get analyzer for a file extension
    pub fn get_analyzer_for_extension(&self, ext: &str) -> Option<&dyn LanguageAnalyzer> {
        let language = ProgrammingLanguage::from_extension(ext);
        self.get_analyzer(&language)
    }

    /// Register a custom analyzer
    pub fn register_analyzer(&mut self, language: ProgrammingLanguage, analyzer: Box<dyn LanguageAnalyzer>) {
        self.analyzers.insert(language, analyzer);
    }
}

impl Default for LanguageAnalyzerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
