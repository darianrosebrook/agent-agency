//! Main CAWS checker implementation
//!
//! This module contains the main CawsChecker struct and its implementation.
//! It orchestrates the various analyzers and compliance checking.

use crate::caws::{
    language_types::ProgrammingLanguage,
    diff_analysis::{DiffAnalyzer, DiffAnalysisResult},
    violation_mapper::ViolationCodeMapper,
    compliance::{CawsWaiver, CawsValidationResult, CawsViolation},
    analyzers::{LanguageAnalyzer, RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer},
};
use agent_agency_database::DatabaseClient;
use std::collections::HashMap;
use std::sync::Arc;

/// Main CAWS checker for compliance validation
#[derive(Debug)]
pub struct CawsChecker {
    pub db_client: Option<Arc<DatabaseClient>>,
    pub analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>>,
    pub diff_analyzer: DiffAnalyzer,
    pub violation_mapper: ViolationCodeMapper,
}

impl CawsChecker {
    /// Create a new CAWS checker
    pub fn new(db_client: DatabaseClient) -> Self {
        let mut analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>> = HashMap::new();
        
        // Register language analyzers
        analyzers.insert(ProgrammingLanguage::Rust, Box::new(RustAnalyzer::new()));
        analyzers.insert(ProgrammingLanguage::TypeScript, Box::new(TypeScriptAnalyzer::new()));
        analyzers.insert(ProgrammingLanguage::JavaScript, Box::new(JavaScriptAnalyzer::new()));

        Self {
            db_client: Some(Arc::new(db_client)),
            analyzers,
            diff_analyzer: DiffAnalyzer::new(),
            violation_mapper: ViolationCodeMapper::new(),
        }
    }

    /// Validate worker output for CAWS compliance
    pub async fn validate_worker_output(
        &self,
        _output: &str,
        _file_path: &str,
        _language: ProgrammingLanguage,
    ) -> CawsValidationResult {
        // TODO: Implement actual validation logic
        CawsValidationResult::new()
    }

    /// Analyze diff for compliance
    pub fn analyze_diff(&self, _old_content: &str, _new_content: &str) -> DiffAnalysisResult {
        self.diff_analyzer.analyze_changes(_old_content, _new_content)
    }

    /// Get analyzer for a specific language
    pub fn get_analyzer(&self, language: &ProgrammingLanguage) -> Option<&dyn LanguageAnalyzer> {
        self.analyzers.get(language).map(|a| a.as_ref())
    }
}

impl Default for CawsChecker {
    fn default() -> Self {
        Self {
            db_client: None,
            analyzers: HashMap::new(),
            diff_analyzer: DiffAnalyzer::new(),
            violation_mapper: ViolationCodeMapper::new(),
        }
    }
}
