//! Language-specific analyzers for CAWS
//!
//! This module contains analyzers for different programming languages,
//! each implementing the LanguageAnalyzer trait.

pub mod rust;
pub mod typescript;
pub mod javascript;

// Re-export analyzers
pub use rust::RustAnalyzer;
pub use typescript::TypeScriptAnalyzer;
pub use javascript::JavaScriptAnalyzer;

use crate::caws::language_types::{LanguageAnalysisResult, ProgrammingLanguage};

/// Trait for language-specific analysis
pub trait LanguageAnalyzer: Send + Sync + std::fmt::Debug {
    /// Analyze code for violations and complexity
    fn analyze(&self, code: &str, file_path: &str) -> LanguageAnalysisResult;
    
    /// Get the programming language this analyzer handles
    fn language(&self) -> ProgrammingLanguage;
    
    /// Check if the analyzer supports the given file extension
    fn supports_extension(&self, ext: &str) -> bool;
}


// Moved from caws_checker.rs: LanguageAnalyzer trait
pub trait LanguageAnalyzer: Send + Sync + std::fmt::Debug {
    /// Analyze a file modification for language-specific issues
    fn analyze_file_modification(
        &self,
        modification: &CouncilFileModification,
    ) -> Result<LanguageAnalysisResult>;

    /// Get the programming language this analyzer handles
    fn language(&self) -> ProgrammingLanguage;

    /// Calculate change complexity for a diff
    fn calculate_change_complexity(
        &self,
        diff: &str,
        content: Option<&str>,
    ) -> Result<ChangeComplexity>;
}

