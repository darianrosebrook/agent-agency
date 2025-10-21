//! Main CAWS checker implementation
//!
//! This module contains the main CawsChecker struct and its implementation.
//! It orchestrates the various analyzers and compliance checking.
//!
//! DEPRECATION NOTICE: This implementation is being migrated to caws-runtime-validator
//! See: iterations/v3/caws/runtime-validator/src/analyzers/
//! TODO: Remove after migration complete (target: Phase 4.2)

use crate::caws::{
    language_types::ProgrammingLanguage,
    diff_analysis::{DiffAnalyzer, DiffAnalysisResult},
    violation_mapper::ViolationCodeMapper,
    compliance::{CawsWaiver, CawsValidationResult, CawsViolation},
    analyzers::{LanguageAnalyzer, RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer},
};

// NEW: Runtime-validator integration
use caws_runtime_validator::{
    CawsValidator,
    analyzers::{
        LanguageAnalyzer as RuntimeLanguageAnalyzer,
        LanguageAnalyzerRegistry,
        ProgrammingLanguage as RuntimeProgrammingLanguage,
        LanguageAnalysisResult,
    },
};

use agent_agency_database::DatabaseClient;
use std::collections::HashMap;
use std::sync::Arc;

/// Main CAWS checker for compliance validation
///
/// DEPRECATED: This implementation is being migrated to caws-runtime-validator
/// The runtime-validator provides centralized CAWS validation with better
/// performance and consistency across all components.
#[derive(Debug)]
pub struct CawsChecker {
    pub db_client: Option<Arc<DatabaseClient>>,
    
    // DEPRECATED: Legacy analyzers (being replaced by runtime-validator)
    pub analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>>,
    pub diff_analyzer: DiffAnalyzer,
    pub violation_mapper: ViolationCodeMapper,
    
    // NEW: Runtime-validator integration
    pub runtime_validator: Arc<CawsValidator>,
    pub runtime_analyzers: Arc<LanguageAnalyzerRegistry>,
}

impl CawsChecker {
    /// Create a new CAWS checker
    pub fn new(db_client: DatabaseClient) -> Self {
        // DEPRECATED: Legacy analyzers (kept for backward compatibility during migration)
        let mut analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>> = HashMap::new();
        analyzers.insert(ProgrammingLanguage::Rust, Box::new(RustAnalyzer::new()));
        analyzers.insert(ProgrammingLanguage::TypeScript, Box::new(TypeScriptAnalyzer::new()));
        analyzers.insert(ProgrammingLanguage::JavaScript, Box::new(JavaScriptAnalyzer::new()));

        // NEW: Runtime-validator integration
        let runtime_validator = Arc::new(CawsValidator::new(caws_runtime_validator::CawsPolicy::default()));
        let runtime_analyzers = Arc::new(LanguageAnalyzerRegistry::new());

        Self {
            db_client: Some(Arc::new(db_client)),
            
            // DEPRECATED: Legacy components
            analyzers,
            diff_analyzer: DiffAnalyzer::new(),
            violation_mapper: ViolationCodeMapper::new(),
            
            // NEW: Runtime-validator components
            runtime_validator,
            runtime_analyzers,
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
        // DEPRECATED: Legacy components
        let analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>> = HashMap::new();
        
        // NEW: Runtime-validator components
        let runtime_validator = Arc::new(CawsValidator::new(caws_runtime_validator::CawsPolicy::default()));
        let runtime_analyzers = Arc::new(LanguageAnalyzerRegistry::new());

        Self {
            db_client: None,
            
            // DEPRECATED: Legacy components
            analyzers,
            diff_analyzer: DiffAnalyzer::new(),
            violation_mapper: ViolationCodeMapper::new(),
            
            // NEW: Runtime-validator components
            runtime_validator,
            runtime_analyzers,
        }
    }
}

impl CawsChecker {
    /// NEW: Validate worker output using runtime-validator
    pub async fn validate_worker_output_runtime(
        &self,
        output: &str,
        file_path: &str,
        language: RuntimeProgrammingLanguage,
    ) -> Result<LanguageAnalysisResult, String> {
        // Use runtime-validator analyzers for primary validation
        if let Some(analyzer) = self.runtime_analyzers.get_analyzer(&language) {
            Ok(analyzer.analyze(output, file_path))
        } else {
            Err(format!("No analyzer available for language: {:?}", language))
        }
    }

    /// NEW: Get runtime-validator analyzer for a specific language
    pub fn get_runtime_analyzer(&self, language: &RuntimeProgrammingLanguage) -> Option<&dyn RuntimeLanguageAnalyzer> {
        self.runtime_analyzers.get_analyzer(language)
    }

    /// NEW: Calculate change complexity using runtime-validator
    pub fn calculate_change_complexity_runtime(
        &self,
        diff: &str,
        content: Option<&str>,
        language: RuntimeProgrammingLanguage,
    ) -> Result<f32, String> {
        if let Some(analyzer) = self.runtime_analyzers.get_analyzer(&language) {
            analyzer.calculate_change_complexity(diff, content)
        } else {
            Err(format!("No analyzer available for language: {:?}", language))
        }
    }

    /// DEPRECATED: Legacy method (use validate_worker_output_runtime instead)
    #[deprecated(note = "Use validate_worker_output_runtime with runtime-validator")]
    pub async fn validate_worker_output_legacy(
        &self,
        _output: &str,
        _file_path: &str,
        _language: ProgrammingLanguage,
    ) -> CawsValidationResult {
        // DEPRECATED: Legacy validation logic
        CawsValidationResult::new()
    }

    /// DEPRECATED: Legacy method (use calculate_change_complexity_runtime instead)
    #[deprecated(note = "Use calculate_change_complexity_runtime with runtime-validator")]
    pub fn analyze_diff_legacy(&self, _old_content: &str, _new_content: &str) -> DiffAnalysisResult {
        // DEPRECATED: Legacy diff analysis
        self.diff_analyzer.analyze_changes(_old_content, _new_content)
    }

    /// DEPRECATED: Legacy method (use get_runtime_analyzer instead)
    #[deprecated(note = "Use get_runtime_analyzer with runtime-validator")]
    pub fn get_analyzer_legacy(&self, language: &ProgrammingLanguage) -> Option<&dyn LanguageAnalyzer> {
        self.analyzers.get(language).map(|a| a.as_ref())
    }
}
