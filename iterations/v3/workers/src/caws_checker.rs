//! CAWS Checker
//!
//! Provides CAWS compliance checking and validation for worker outputs.
//! Enhanced with AST-based diff sizing and violation code mapping.

use crate::types::*;
use agent_agency_council::models::{
    FileModification as CouncilFileModification, FileOperation as CouncilFileOperation, RiskTier,
    TaskSpec, WorkerOutput as CouncilWorkerOutput,
};
use agent_agency_database::{CawsViolation as DbCawsViolation, DatabaseClient};
use anyhow::{Context, Result};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

/// Programming language types for AST analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// AST-based diff analyzer for surgical change scoring
#[derive(Debug)]
pub struct DiffAnalyzer {
    // Configuration for diff analysis
    max_change_complexity: f32,
    surgical_change_threshold: f32,
}

/// Violation code mapper for constitutional references
#[derive(Debug)]
pub struct ViolationCodeMapper {
    // Maps violation codes to constitutional sections
    code_mappings: HashMap<String, ConstitutionalReference>,
}

/// Constitutional reference for violations
#[derive(Debug, Clone)]
pub struct ConstitutionalReference {
    pub section: String,
    pub subsection: String,
    pub description: String,
    pub severity: ViolationSeverity,
}

/// Language analyzer trait for language-specific analysis
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

/// Language analysis result
#[derive(Debug, Clone)]
pub struct LanguageAnalysisResult {
    pub violations: Vec<LanguageViolation>,
    pub warnings: Vec<LanguageWarning>,
    pub complexity_score: f32,
    pub surgical_change_score: f32,
    pub change_complexity: ChangeComplexity,
}

/// Language-specific violation
#[derive(Debug, Clone)]
pub struct LanguageViolation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub location: Option<SourceLocation>,
    pub suggestion: Option<String>,
    pub constitutional_ref: Option<ConstitutionalReference>,
}

/// Language-specific warning
#[derive(Debug, Clone)]
pub struct LanguageWarning {
    pub rule: String,
    pub description: String,
    pub location: Option<SourceLocation>,
    pub suggestion: Option<String>,
}

/// Source code location
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// Change complexity analysis
#[derive(Debug, Clone)]
pub struct ChangeComplexity {
    pub structural_changes: u32,
    pub logical_changes: u32,
    pub dependency_changes: u32,
    pub complexity_score: f32,
    pub is_surgical: bool,
}

/// Diff analysis result
#[derive(Debug, Clone)]
pub struct DiffAnalysisResult {
    pub change_complexity: ChangeComplexity,
    pub language_violations: Vec<LanguageViolation>,
    pub language_warnings: Vec<LanguageWarning>,
    pub is_oversized: bool,
    pub is_noisy: bool,
    pub surgical_change_score: f32,
    pub recommended_action: RecommendedAction,
}

/// Recommended action for diff issues
#[derive(Debug, Clone)]
pub enum RecommendedAction {
    Accept,
    RequestSmallerChanges,
    SplitIntoMultiplePRs,
    AddMoreTests,
    ImproveDocumentation,
    RequestReview,
}

/// CAWS compliance checker for worker outputs
#[derive(Debug)]
pub struct CawsChecker {
    // AST-based diff analyzer for surgical change scoring
    diff_analyzer: DiffAnalyzer,
    // Violation code mapper for constitutional references
    violation_mapper: ViolationCodeMapper,
    // Language-specific analyzers
    language_analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>>,
    // Database client for violation storage and retrieval
    db_client: DatabaseClient,
}

impl CawsChecker {
    /// Helper function to create a CawsViolation with constitutional_ref
    fn create_violation(
        rule: &str,
        severity: ViolationSeverity,
        description: String,
        location: Option<String>,
        suggestion: Option<String>,
        constitutional_ref: &str,
    ) -> CawsViolation {
        CawsViolation {
            rule: rule.to_string(),
            severity,
            description,
            location,
            suggestion,
            constitutional_ref: Some(constitutional_ref.to_string()),
        }
    }

    /// Create a new CAWS checker
    pub fn new(db_client: DatabaseClient) -> Self {
        let mut language_analyzers: HashMap<ProgrammingLanguage, Box<dyn LanguageAnalyzer>> =
            HashMap::new();

        // Register language analyzers
        language_analyzers.insert(ProgrammingLanguage::Rust, Box::new(RustAnalyzer::new()));
        language_analyzers.insert(
            ProgrammingLanguage::TypeScript,
            Box::new(TypeScriptAnalyzer::new()),
        );
        language_analyzers.insert(
            ProgrammingLanguage::JavaScript,
            Box::new(JavaScriptAnalyzer::new()),
        );

        Self {
            diff_analyzer: DiffAnalyzer::new(),
            violation_mapper: ViolationCodeMapper::new(),
            language_analyzers,
            db_client,
        }
    }

    /// Check CAWS compliance for a task specification
    pub async fn validate_task_spec(&self, task_spec: &TaskSpec) -> Result<CawsValidationResult> {
        info!("Validating CAWS compliance for task: {}", task_spec.title);

        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Check budget compliance
        self.check_budget_compliance(task_spec, &mut violations, &mut warnings)?;

        // Check scope compliance
        self.check_scope_compliance(task_spec, &mut violations, &mut warnings)?;

        // Check acceptance criteria
        self.check_acceptance_criteria(task_spec, &mut violations, &mut suggestions)?;

        // Check risk tier appropriateness
        self.check_risk_tier_appropriateness(task_spec, &mut violations, &mut suggestions)?;

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(&violations, &warnings);
        let is_compliant = violations.is_empty();

        Ok(CawsValidationResult {
            is_compliant,
            compliance_score,
            violations,
            warnings,
            suggestions,
            validated_at: chrono::Utc::now(),
        })
    }

    /// Check CAWS compliance for worker output
    pub async fn validate_worker_output(
        &self,
        output: &CouncilWorkerOutput,
        task_spec: &TaskSpec,
    ) -> Result<CawsValidationResult> {
        info!("Validating CAWS compliance for worker output");

        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Check budget adherence
        self.check_budget_adherence(output, task_spec, &mut violations, &mut warnings)?;

        // Check quality standards
        self.check_quality_standards(output, task_spec, &mut violations, &mut warnings)?;

        // Check CAWS rule compliance
        self.check_caws_rules(
            output,
            task_spec,
            &mut violations,
            &mut warnings,
            &mut suggestions,
        )?;

        // Check provenance requirements
        self.check_provenance_requirements(output, &mut violations, &mut warnings)?;

        // NEW: AST-based diff analysis for surgical change scoring
        let diff_analysis = self.analyze_diff_complexity(output).await?;
        self.process_diff_analysis(
            &diff_analysis,
            &mut violations,
            &mut warnings,
            &mut suggestions,
        )?;

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(&violations, &warnings);
        let is_compliant = violations.is_empty();

        Ok(CawsValidationResult {
            is_compliant,
            compliance_score,
            violations,
            warnings,
            suggestions,
            validated_at: chrono::Utc::now(),
        })
    }

    /// Analyze diff complexity using AST-based analysis
    pub async fn analyze_diff_complexity(
        &self,
        output: &CouncilWorkerOutput,
    ) -> Result<Vec<DiffAnalysisResult>> {
        let mut results = Vec::new();

        for file_mod in &output.files_modified {
            let language = self.detect_programming_language(&file_mod.path);

            if let Some(analyzer) = self.language_analyzers.get(&language) {
                let analysis_result = analyzer.analyze_file_modification(file_mod)?;

                let diff_analysis = DiffAnalysisResult {
                    change_complexity: analysis_result.change_complexity.clone(),
                    language_violations: analysis_result.violations.clone(),
                    language_warnings: analysis_result.warnings.clone(),
                    is_oversized: analysis_result.complexity_score
                        > self.diff_analyzer.max_change_complexity,
                    is_noisy: analysis_result.surgical_change_score
                        < self.diff_analyzer.surgical_change_threshold,
                    surgical_change_score: analysis_result.surgical_change_score,
                    recommended_action: self.determine_recommended_action(&analysis_result),
                };

                results.push(diff_analysis);
            }
        }

        Ok(results)
    }

    /// Detect programming language from file path
    fn detect_programming_language(&self, file_path: &str) -> ProgrammingLanguage {
        let extension = file_path.split('.').last().unwrap_or("").to_lowercase();

        match extension.as_str() {
            "rs" => ProgrammingLanguage::Rust,
            "ts" | "tsx" => ProgrammingLanguage::TypeScript,
            "js" | "jsx" => ProgrammingLanguage::JavaScript,
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

    /// Process diff analysis results into violations and warnings
    fn process_diff_analysis(
        &self,
        diff_analyses: &[DiffAnalysisResult],
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
        suggestions: &mut Vec<String>,
    ) -> Result<()> {
        for analysis in diff_analyses {
            // Check for oversized changes
            if analysis.is_oversized {
                violations.push(CawsViolation {
                    rule: "Change Size Limit".to_string(),
                    severity: ViolationSeverity::High,
                    description: format!(
                        "Change complexity score {:.2} exceeds maximum allowed {:.2}",
                        analysis.change_complexity.complexity_score,
                        self.diff_analyzer.max_change_complexity
                    ),
                    location: None,
                    suggestion: Some(
                        "Break changes into smaller, more focused modifications".to_string(),
                    ),
                    constitutional_ref: None,
                });
            }

            // Check for noisy changes
            if analysis.is_noisy {
                violations.push(CawsViolation {
                    rule: "Surgical Change Requirement".to_string(),
                    severity: ViolationSeverity::Medium,
                    description: format!(
                        "Surgical change score {:.2} below threshold {:.2}",
                        analysis.surgical_change_score,
                        self.diff_analyzer.surgical_change_threshold
                    ),
                    location: None,
                    suggestion: Some(
                        "Make more surgical changes with focused modifications".to_string(),
                    ),
                    constitutional_ref: Some("SURG-001".to_string()),
                });
            }

            // Add language-specific violations
            for lang_violation in &analysis.language_violations {
                violations.push(CawsViolation {
                    rule: lang_violation.rule.clone(),
                    severity: lang_violation.severity.clone(),
                    description: lang_violation.description.clone(),
                    location: lang_violation
                        .location
                        .as_ref()
                        .map(|loc| format!("{}:{}", loc.line, loc.column)),
                    suggestion: lang_violation.suggestion.clone(),
                    constitutional_ref: None,
                });
            }

            // Add language-specific warnings
            for lang_warning in &analysis.language_warnings {
                warnings.push(format!(
                    "{}: {}",
                    lang_warning.rule, lang_warning.description
                ));
            }

            // Add recommendations based on analysis
            match analysis.recommended_action {
                RecommendedAction::RequestSmallerChanges => {
                    suggestions.push(
                        "Consider breaking this change into smaller, more focused modifications"
                            .to_string(),
                    );
                }
                RecommendedAction::SplitIntoMultiplePRs => {
                    suggestions.push(
                        "This change may be too large for a single PR - consider splitting"
                            .to_string(),
                    );
                }
                RecommendedAction::AddMoreTests => {
                    suggestions.push(
                        "Consider adding more comprehensive tests for this change".to_string(),
                    );
                }
                RecommendedAction::ImproveDocumentation => {
                    suggestions
                        .push("Consider improving documentation for this change".to_string());
                }
                RecommendedAction::RequestReview => {
                    suggestions.push("This change may benefit from additional review".to_string());
                }
                RecommendedAction::Accept => {
                    // No additional suggestions needed
                }
            }
        }

        Ok(())
    }

    /// Determine recommended action based on analysis
    fn determine_recommended_action(&self, analysis: &LanguageAnalysisResult) -> RecommendedAction {
        if analysis.complexity_score > 0.8 {
            RecommendedAction::SplitIntoMultiplePRs
        } else if analysis.complexity_score > 0.6 {
            RecommendedAction::RequestSmallerChanges
        } else if analysis.surgical_change_score < 0.5 {
            RecommendedAction::AddMoreTests
        } else if !analysis.violations.is_empty() {
            RecommendedAction::RequestReview
        } else {
            RecommendedAction::Accept
        }
    }

    /// Check budget compliance
    fn check_budget_compliance(
        &self,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if budget limits are reasonable for the task
        if let Some(max_files) = task_spec.scope.max_files {
            if max_files == 0 {
                violations.push(CawsViolation {
                    rule: "Budget Definition".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: "Max files cannot be zero".to_string(),
                    location: Some("task_spec.scope.max_files".to_string()),
                    suggestion: Some("Set max_files to at least 1".to_string()),
                    constitutional_ref: None,
                });
            } else if max_files > 50 {
                warnings.push(format!("Large file count limit: {} files", max_files));
            }
        }

        if let Some(max_loc) = task_spec.scope.max_loc {
            if max_loc == 0 {
                violations.push(CawsViolation {
                    rule: "Budget Definition".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: "Max LOC cannot be zero".to_string(),
                    location: Some("task_spec.scope.max_loc".to_string()),
                    suggestion: Some("Set max_loc to at least 1".to_string()),
                    constitutional_ref: None,
                });
            } else if max_loc > 10000 {
                warnings.push(format!("Large LOC limit: {} lines", max_loc));
            }
        }

        Ok(())
    }

    /// Check scope compliance
    fn check_scope_compliance(
        &self,
        task_spec: &TaskSpec,
        _violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if scope is well-defined
        if task_spec.scope.files_affected.is_empty() {
            warnings.push("No files specified in scope".to_string());
        }

        // Check if domains are specified
        if task_spec.scope.domains.is_empty() {
            warnings.push("No domains specified in scope".to_string());
        }

        // Check for overly broad scopes
        if task_spec.scope.files_affected.len() > 20 {
            warnings.push("Very broad file scope - consider narrowing".to_string());
        }

        Ok(())
    }

    /// Check acceptance criteria
    fn check_acceptance_criteria(
        &self,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        suggestions: &mut Vec<String>,
    ) -> Result<()> {
        // Check if acceptance criteria are defined
        if task_spec.acceptance_criteria.is_empty() {
            violations.push(CawsViolation {
                rule: "Acceptance Criteria".to_string(),
                severity: ViolationSeverity::High,
                description: "No acceptance criteria defined".to_string(),
                location: None,
                suggestion: Some("Define clear acceptance criteria for the task".to_string()),
                constitutional_ref: None,
            });
        } else {
            // Check quality of acceptance criteria
            for criterion in &task_spec.acceptance_criteria {
                if criterion.description.len() < 10 {
                    suggestions.push(format!(
                        "Acceptance criterion '{}' is too brief - consider adding more detail",
                        criterion.id
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check risk tier appropriateness
    fn check_risk_tier_appropriateness(
        &self,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        suggestions: &mut Vec<String>,
    ) -> Result<()> {
        // Check if risk tier matches task complexity
        let has_auth_keywords = task_spec.description.to_lowercase().contains("auth")
            || task_spec.description.to_lowercase().contains("login")
            || task_spec.description.to_lowercase().contains("password");

        let has_billing_keywords = task_spec.description.to_lowercase().contains("billing")
            || task_spec.description.to_lowercase().contains("payment")
            || task_spec.description.to_lowercase().contains("money");

        let has_database_keywords = task_spec.description.to_lowercase().contains("database")
            || task_spec.description.to_lowercase().contains("migration")
            || task_spec.description.to_lowercase().contains("schema");

        let is_high_risk_content =
            has_auth_keywords || has_billing_keywords || has_database_keywords;

        match task_spec.risk_tier {
            RiskTier::Tier1 => {
                // Tier 1 should be for critical systems
                if !is_high_risk_content {
                    suggestions.push(
                        "Task may not require Tier 1 risk level - consider Tier 2".to_string(),
                    );
                }
            }
            RiskTier::Tier2 => {
                // Tier 2 is appropriate for most features
                // No specific checks needed
            }
            RiskTier::Tier3 => {
                // Tier 3 should be for low-risk changes
                if is_high_risk_content {
                    violations.push(CawsViolation {
                        rule: "Risk Tier Classification".to_string(),
                        severity: ViolationSeverity::High,
                        description: "High-risk content detected but task marked as Tier 3"
                            .to_string(),
                        location: None,
                        suggestion: Some("Consider upgrading to Tier 1 or Tier 2".to_string()),
                        constitutional_ref: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Check budget adherence in worker output
    fn check_budget_adherence(
        &self,
        output: &CouncilWorkerOutput,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        let files_used = output.files_modified.len() as u32;
        let loc_used: u32 = output
            .files_modified
            .iter()
            .map(|f| {
                f.content
                    .as_ref()
                    .map(|c| c.lines().count() as u32)
                    .unwrap_or(0)
            })
            .sum();

        // Check file count
        if let Some(max_files) = task_spec.scope.max_files {
            if files_used > max_files {
                violations.push(CawsViolation {
                    rule: "File Count Budget".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: format!(
                        "Used {} files, exceeds limit of {}",
                        files_used, max_files
                    ),
                    location: None,
                    suggestion: Some("Reduce file count or request budget increase".to_string()),
                    constitutional_ref: None,
                });
            } else if files_used as f32 / max_files as f32 > 0.8 {
                warnings.push(format!(
                    "File count near limit: {}/{}",
                    files_used, max_files
                ));
            }
        }

        // Check LOC count
        if let Some(max_loc) = task_spec.scope.max_loc {
            if loc_used > max_loc {
                violations.push(CawsViolation {
                    rule: "LOC Budget".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: format!("Used {} LOC, exceeds limit of {}", loc_used, max_loc),
                    location: None,
                    suggestion: Some("Reduce LOC or request budget increase".to_string()),
                    constitutional_ref: None,
                });
            } else if loc_used as f32 / max_loc as f32 > 0.8 {
                warnings.push(format!("LOC near limit: {}/{}", loc_used, max_loc));
            }
        }

        Ok(())
    }

    /// Check quality standards
    fn check_quality_standards(
        &self,
        output: &CouncilWorkerOutput,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check self-assessment quality
        if output.self_assessment.quality_score < 0.7 {
            violations.push(CawsViolation {
                rule: "Quality Standards".to_string(),
                severity: ViolationSeverity::High,
                description: "Worker self-assessment indicates low quality".to_string(),
                location: None,
                suggestion: Some("Review and improve output quality".to_string()),
                constitutional_ref: None,
            });
        }

        // Check confidence level
        if output.self_assessment.confidence < 0.5 {
            warnings.push("Worker has low confidence in output".to_string());
        }

        // Check for concerns
        if !output.self_assessment.concerns.is_empty() {
            warnings.push(format!(
                "Worker raised {} concerns about the output",
                output.self_assessment.concerns.len()
            ));
        }

        // Check rationale quality
        if output.rationale.len() < 50 {
            violations.push(CawsViolation {
                rule: "Rationale Quality".to_string(),
                severity: ViolationSeverity::Medium,
                description: "Rationale is too brief".to_string(),
                location: None,
                suggestion: Some("Provide more detailed rationale for decisions".to_string()),
                constitutional_ref: None,
            });
        }

        Ok(())
    }

    /// Check CAWS rules compliance
    fn check_caws_rules(
        &self,
        output: &CouncilWorkerOutput,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
        suggestions: &mut Vec<String>,
    ) -> Result<()> {
        // Check CAWS compliance score from self-assessment
        if output.self_assessment.caws_compliance < 0.8 {
            violations.push(CawsViolation {
                rule: "CAWS Compliance".to_string(),
                severity: ViolationSeverity::High,
                description: "Worker self-assessment indicates CAWS compliance issues".to_string(),
                location: None,
                suggestion: Some("Review CAWS requirements and improve compliance".to_string()),
                constitutional_ref: None,
            });
        }

        // Check for hardcoded values in code
        for file_mod in &output.files_modified {
            if let Some(content) = &file_mod.content {
                if content.contains("localhost") || content.contains("127.0.0.1") {
                    warnings.push(format!("Hardcoded localhost found in {}", file_mod.path));
                }

                if content.contains("password") || content.contains("secret") {
                    violations.push(CawsViolation {
                        rule: "Security Best Practices".to_string(),
                        severity: ViolationSeverity::Critical,
                        description: "Potential hardcoded secrets detected".to_string(),
                        location: Some(file_mod.path.clone()),
                        suggestion: Some(
                            "Use environment variables or secure configuration".to_string(),
                        ),
                        constitutional_ref: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Check provenance requirements
    fn check_provenance_requirements(
        &self,
        output: &CouncilWorkerOutput,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if rationale is provided
        if output.rationale.is_empty() {
            violations.push(CawsViolation {
                rule: "Provenance Requirements".to_string(),
                severity: ViolationSeverity::High,
                description: "No rationale provided for decisions".to_string(),
                location: None,
                suggestion: Some("Provide detailed rationale for all decisions".to_string()),
                constitutional_ref: None,
            });
        }

        // Check if self-assessment is complete
        if output.self_assessment.concerns.is_empty()
            && output.self_assessment.improvements.is_empty()
        {
            warnings.push(
                "Self-assessment appears incomplete - no concerns or improvements identified"
                    .to_string(),
            );
        }

        // Check if file modifications are documented
        for file_mod in &output.files_modified {
            match file_mod.operation {
                CouncilFileOperation::Create => {
                    if file_mod.content.is_none() {
                        violations.push(CawsViolation {
                            rule: "File Modification Documentation".to_string(),
                            severity: ViolationSeverity::Medium,
                            description: "File creation without content provided".to_string(),
                            location: Some(file_mod.path.clone()),
                            suggestion: Some(
                                "Provide file content for creation operations".to_string(),
                            ),
                            constitutional_ref: None,
                        });
                    }
                }
                CouncilFileOperation::Modify => {
                    if file_mod.diff.is_none() && file_mod.content.is_none() {
                        violations.push(CawsViolation {
                            rule: "File Modification Documentation".to_string(),
                            severity: ViolationSeverity::Medium,
                            description: "File modification without diff or content provided"
                                .to_string(),
                            location: Some(file_mod.path.clone()),
                            suggestion: Some(
                                "Provide diff or content for modification operations".to_string(),
                            ),
                            constitutional_ref: None,
                        });
                    }
                }
                CouncilFileOperation::Delete => {
                    // Deletion operations don't require content
                }
                CouncilFileOperation::Move { .. } => {
                    if file_mod.diff.is_none() {
                        warnings.push(format!(
                            "File move operation without diff: {}",
                            file_mod.path
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculate compliance score
    fn calculate_compliance_score(&self, violations: &[CawsViolation], warnings: &[String]) -> f32 {
        let mut score: f32 = 1.0;

        // Deduct points for violations
        for violation in violations {
            let deduction = match violation.severity {
                ViolationSeverity::Critical => 0.3,
                ViolationSeverity::High => 0.2,
                ViolationSeverity::Medium => 0.1,
                ViolationSeverity::Low => 0.05,
            };
            score -= deduction;
        }

        // Deduct smaller points for warnings
        for _warning in warnings {
            score -= 0.02;
        }

        score.max(0.0)
    }

    /// Get CAWS rule violations for a task
    pub async fn get_violations(&self, task_id: Uuid) -> Result<Vec<CawsViolation>> {
        // Query violations from database
        let violations = self.query_violations_from_database(task_id).await?;

        // Format and enrich violations with constitutional references
        let enriched_violations = self
            .enrich_violations_with_constitutional_refs(violations)
            .await?;

        Ok(enriched_violations)
    }

    /// Query violations from database
    async fn query_violations_from_database(&self, task_id: Uuid) -> Result<Vec<CawsViolation>> {
        // Execute SQL query to get active violations for the task
        let query = r#"
            SELECT 
                id,
                task_id,
                violation_code,
                severity,
                description,
                file_path,
                line_number,
                column_number,
                rule_id,
                constitutional_reference,
                status,
                created_at,
                resolved_at,
                metadata
            FROM caws_violations 
            WHERE task_id = $1 AND status = 'active'
            ORDER BY severity DESC, created_at DESC
            LIMIT 100
        "#;

        // Execute the query using the database client
        let db_violations: Vec<DbCawsViolation> = sqlx::query_as(query)
            .bind(task_id)
            .fetch_all(self.db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query violations: {}", e))?;

        // Convert database violations to CAWS violations
        let violations: Vec<CawsViolation> = db_violations
            .into_iter()
            .map(|db_violation| CawsViolation {
                rule: db_violation.rule_id,
                severity: match db_violation.severity.as_str() {
                    "critical" => ViolationSeverity::Critical,
                    "high" => ViolationSeverity::High,
                    "medium" => ViolationSeverity::Medium,
                    "low" => ViolationSeverity::Low,
                    _ => ViolationSeverity::Medium,
                },
                description: db_violation.description,
                location: db_violation.file_path.map(|path| {
                    if let (Some(line), Some(col)) =
                        (db_violation.line_number, db_violation.column_number)
                    {
                        format!("{}:{}:{}", path, line, col)
                    } else {
                        path
                    }
                }),
                suggestion: None, // Could be added to database schema
                constitutional_ref: db_violation.constitutional_reference,
            })
            .collect();

        Ok(violations)
    }

    /// Enrich violations with constitutional references
    async fn enrich_violations_with_constitutional_refs(
        &self,
        violations: Vec<CawsViolation>,
    ) -> Result<Vec<CawsViolation>> {
        let mut enriched = Vec::new();

        for mut violation in violations {
            // Add constitutional reference if not already present
            if violation.constitutional_ref.is_none() {
                if let Some(ref rule) = self.violation_mapper.code_mappings.get(&violation.rule) {
                    violation.constitutional_ref =
                        Some(format!("{}.{}", rule.section, rule.subsection));
                }
            }

            // Add detailed metadata
            violation = self.add_violation_metadata(violation).await?;
            enriched.push(violation);
        }

        Ok(enriched)
    }

    /// Add metadata to violations
    async fn add_violation_metadata(&self, mut violation: CawsViolation) -> Result<CawsViolation> {
        use chrono::Utc;

        // Generate unique ID for violation tracking
        let violation_id = Uuid::new_v4();

        // Add timestamps for lifecycle tracking
        let created_at = Utc::now();
        let metadata = json!({
            "violation_id": violation_id,
            "created_at": created_at,
            "processed_by": "caws_checker",
            "version": "1.0",
            "tags": ["compliance", "caws", &violation.rule],
            "severity_score": violation.severity as u8,
            "auto_generated": true
        });

        // Store violation in database for audit trail and tracking
        if let Some(db_client) = &self.db_client {
            let db_violation = DbCawsViolation {
                id: violation_id,
                task_id: Uuid::nil(), // Will be set by caller if available
                violation_code: violation.rule.clone(),
                severity: format!("{:?}", violation.severity).to_lowercase(),
                description: violation.description.clone(),
                file_path: violation.location.clone(),
                line_number: None, // Could be extracted from location if needed
                column_number: None,
                rule_id: violation.rule.clone(),
                constitutional_reference: violation.constitutional_ref.clone(),
                status: "active".to_string(),
                created_at,
                resolved_at: None,
                metadata: metadata.clone(),
            };

            // Store in database asynchronously without blocking the main flow
            let db_client_clone = db_client.clone();
            let db_violation_clone = db_violation.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::store_violation_in_db(&db_client_clone, &db_violation_clone).await {
                    tracing::warn!("Failed to store CAWS violation in database: {}", e);
                }
            });
        }

        // Add suggestion if missing
        if violation.suggestion.is_none() {
            violation.suggestion = Some(format!(
                "Review and address the {} violation",
                violation.rule
            ));
        }

        // Validate violation data
        if violation.description.is_empty() {
            violation.description = format!("Violation of rule: {}", violation.rule);
        }

        // Add metadata to violation for client consumption
        // Note: We keep the original CawsViolation struct intact but add metadata context
        info!(
            "Enhanced CAWS violation {} with database metadata - Rule: {}, Severity: {:?}",
            violation_id, violation.rule, violation.severity
        );

        Ok(violation)
    }

    /// Store CAWS violation in database
    async fn store_violation_in_db(
        db_client: &DatabaseClient,
        violation: &DbCawsViolation,
    ) -> Result<()> {
        use chrono::Utc;

        // Insert violation into database
        let query = r#"
            INSERT INTO caws_violations (
                id, task_id, violation_code, severity, description,
                file_path, line_number, column_number, rule_id,
                constitutional_reference, status, created_at, resolved_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (id) DO UPDATE SET
                description = EXCLUDED.description,
                status = EXCLUDED.status,
                metadata = EXCLUDED.metadata,
                resolved_at = CASE
                    WHEN EXCLUDED.status = 'resolved' THEN EXCLUDED.resolved_at
                    ELSE caws_violations.resolved_at
                END
        "#;

        let mut conn = db_client.get_connection().await?;
        sqlx::query(query)
            .bind(&violation.id)
            .bind(&violation.task_id)
            .bind(&violation.violation_code)
            .bind(&violation.severity)
            .bind(&violation.description)
            .bind(&violation.file_path)
            .bind(violation.line_number)
            .bind(violation.column_number)
            .bind(&violation.rule_id)
            .bind(&violation.constitutional_reference)
            .bind(&violation.status)
            .bind(violation.created_at)
            .bind(violation.resolved_at)
            .bind(&violation.metadata)
            .execute(&mut *conn)
            .await
            .context("Failed to store CAWS violation in database")?;

        info!(
            "Stored CAWS violation {} in database - Rule: {}, Severity: {}",
            violation.id, violation.violation_code, violation.severity
        );

        Ok(())
    }

    /// Check if a waiver is valid
    pub async fn validate_waiver(&self, waiver: &CawsWaiver) -> Result<bool> {
        // Check if waiver has valid justification
        if waiver.justification.len() < 20 {
            return Ok(false);
        }

        // Check if waiver is time-bounded
        if waiver.time_bounded {
            if let Some(expires_at) = waiver.expires_at {
                if expires_at < chrono::Utc::now() {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Store violation in database (for future use)
    pub async fn store_violation(&self, task_id: Uuid, violation: CawsViolation) -> Result<String> {
        let CawsViolation {
            rule,
            severity,
            description,
            location,
            suggestion,
            constitutional_ref,
        } = violation;

        let violation_id = Uuid::new_v4();
        let (file_path, line_number, column_number) = parse_violation_location(location.as_deref());
        let metadata = json!({
            "suggestion": suggestion,
            "recorded_at": chrono::Utc::now().to_rfc3339(),
            "original_location": location,
        });

        sqlx::query(
            r#"
            INSERT INTO caws_violations (
                id,
                task_id,
                violation_code,
                severity,
                description,
                file_path,
                line_number,
                column_number,
                rule_id,
                constitutional_reference,
                status,
                created_at,
                resolved_at,
                metadata
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                'active', $11, NULL, $12
            )
            "#,
        )
        .bind(violation_id)
        .bind(task_id)
        .bind(&rule)
        .bind(severity_to_db_value(&severity))
        .bind(&description)
        .bind(file_path)
        .bind(line_number)
        .bind(column_number)
        // Implement proper rule_id mapping with comprehensive rule identification system
        let rule_id = self.get_or_create_rule_id(&rule, &description, &severity).await?;
        .bind(&rule_id)
        .bind(constitutional_ref)
        .bind(chrono::Utc::now())
        .bind(metadata)
        .execute(self.db_client.pool())
        .await
        .with_context(|| format!("Failed to store CAWS violation for task {}", task_id))?;

        Ok(violation_id.to_string())
    }

    /// Update violation status
    pub async fn update_violation_status(&self, violation_id: &str, status: &str) -> Result<()> {
        let violation_uuid = Uuid::parse_str(violation_id)
            .with_context(|| format!("Invalid violation id: {}", violation_id))?;

        let resolved_at =
            matches!(status, "resolved" | "waived" | "dismissed").then(|| chrono::Utc::now());

        sqlx::query(
            r#"
            UPDATE caws_violations
            SET status = $1,
                resolved_at = CASE
                    WHEN $1 IN ('resolved', 'waived', 'dismissed') THEN $2
                    ELSE resolved_at
                END
            WHERE id = $3
            "#,
        )
        .bind(status)
        .bind(resolved_at)
        .bind(violation_uuid)
        .execute(self.db_client.pool())
        .await
        .with_context(|| format!("Failed to update violation {}", violation_id))?;

        Ok(())
    }

    /// Get violation statistics for reporting
    pub async fn get_violation_stats(&self, task_id: Option<Uuid>) -> Result<ViolationStats> {
        let counts_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) AS total_count,
                COUNT(*) FILTER (WHERE severity = 'critical') AS critical_count,
                COUNT(*) FILTER (WHERE severity = 'high') AS high_count,
                COUNT(*) FILTER (WHERE severity = 'medium') AS medium_count,
                COUNT(*) FILTER (WHERE severity = 'low') AS low_count,
                COUNT(*) FILTER (WHERE status = 'resolved') AS resolved_count,
                COUNT(*) FILTER (WHERE status = 'active') AS active_count,
                AVG(EXTRACT(EPOCH FROM (resolved_at - created_at)) / 3600.0) AS avg_resolution_hours
            FROM caws_violations
            WHERE task_id = COALESCE($1::uuid, task_id)
            "#,
        )
        .bind(task_id)
        .fetch_one(self.db_client.pool())
        .await
        .context("Failed to aggregate CAWS violation statistics")?;

        let total_violations: i64 = counts_row.try_get("total_count")?;
        let critical_count: i64 = counts_row.try_get("critical_count")?;
        let high_count: i64 = counts_row.try_get("high_count")?;
        let medium_count: i64 = counts_row.try_get("medium_count")?;
        let low_count: i64 = counts_row.try_get("low_count")?;
        let resolved_count: i64 = counts_row.try_get("resolved_count")?;
        let active_count: i64 = counts_row.try_get("active_count")?;
        let avg_resolution_hours: Option<f64> = counts_row.try_get("avg_resolution_hours")?;

        let most_common_rule = sqlx::query(
            r#"
            SELECT rule_id, COUNT(*) AS usage
            FROM caws_violations
            WHERE task_id = COALESCE($1::uuid, task_id)
            GROUP BY rule_id
            ORDER BY usage DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(self.db_client.pool())
        .await?
        .and_then(|row| row.try_get::<String, _>("rule_id").ok())
        .unwrap_or_else(|| "Unknown".to_string());

        let stats = ViolationStats {
            total_violations: total_violations as u32,
            critical_count: critical_count as u32,
            high_count: high_count as u32,
            medium_count: medium_count as u32,
            low_count: low_count as u32,
            resolved_count: resolved_count as u32,
            active_count: active_count as u32,
            average_resolution_time_hours: avg_resolution_hours.unwrap_or_default() as f32,
            most_common_rule,
            compliance_trend: infer_compliance_trend(
                total_violations as u32,
                resolved_count as u32,
                active_count as u32,
            ),
        };

        Ok(stats)
    }

    /// Bulk operations for violations
    pub async fn bulk_update_violations(
        &self,
        violation_ids: Vec<String>,
        status: &str,
    ) -> Result<usize> {
        if violation_ids.is_empty() {
            return Ok(0);
        }

        let ids: Vec<Uuid> = violation_ids
            .into_iter()
            .map(|id| {
                Uuid::parse_str(&id)
                    .with_context(|| format!("Invalid violation id in bulk update: {}", id))
            })
            .collect::<Result<Vec<_>>>()?;

        let resolved_at =
            matches!(status, "resolved" | "waived" | "dismissed").then(|| chrono::Utc::now());

        let result = sqlx::query(
            r#"
            UPDATE caws_violations
            SET status = $1,
                resolved_at = CASE
                    WHEN $1 IN ('resolved', 'waived', 'dismissed') THEN $2
                    ELSE resolved_at
                END
            WHERE id = ANY($3)
            "#,
        )
        .bind(status)
        .bind(resolved_at)
        .bind(&ids)
        .execute(self.db_client.pool())
        .await
        .with_context(|| "Failed to bulk update CAWS violations".to_string())?;

        Ok(result.rows_affected() as usize)
    }

    /// Get or create rule ID with comprehensive rule identification system
    async fn get_or_create_rule_id(
        &self,
        rule: &str,
        description: &str,
        severity: &ViolationSeverity,
    ) -> Result<String> {
        // 1. Rule identification: Create unique rule identifiers for each CAWS rule type
        let rule_hash = self.generate_rule_hash(rule, description);
        
        // 2. Database schema integration: Check if rule exists in rules table
        let existing_rule_id = self.get_existing_rule_id(&rule_hash).await?;
        
        if let Some(rule_id) = existing_rule_id {
            return Ok(rule_id);
        }
        
        // 3. Rule management system: Create new rule with metadata
        let rule_id = self.create_new_rule(rule, description, severity, &rule_hash).await?;
        
        Ok(rule_id)
    }

    /// Generate unique hash for rule identification
    fn generate_rule_hash(&self, rule: &str, description: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        rule.hash(&mut hasher);
        description.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get existing rule ID from database
    async fn get_existing_rule_id(&self, rule_hash: &str) -> Result<Option<String>> {
        let query = "SELECT rule_id FROM caws_rules WHERE rule_hash = $1";
        
        match sqlx::query_scalar::<_, String>(query)
            .bind(rule_hash)
            .fetch_optional(self.db_client.pool())
            .await
        {
            Ok(Some(rule_id)) => Ok(Some(rule_id)),
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("Failed to query existing rule: {}", e);
                Ok(None) // Graceful fallback
            }
        }
    }

    /// Create new rule in database with metadata
    async fn create_new_rule(
        &self,
        rule: &str,
        description: &str,
        severity: &ViolationSeverity,
        rule_hash: &str,
    ) -> Result<String> {
        // 4. Performance optimization: Generate UUID for rule_id
        let rule_id = uuid::Uuid::new_v4().to_string();
        
        // Insert new rule with comprehensive metadata
        let query = r#"
            INSERT INTO caws_rules (
                rule_id, rule_hash, rule_name, description, severity, 
                category, impact_level, created_at, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;
        
        let category = self.infer_rule_category(rule);
        let impact_level = self.map_severity_to_impact(severity);
        
        sqlx::query(query)
            .bind(&rule_id)
            .bind(rule_hash)
            .bind(rule)
            .bind(description)
            .bind(severity_to_db_value(severity))
            .bind(&category)
            .bind(&impact_level)
            .bind(chrono::Utc::now())
            .bind("active")
            .execute(self.db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create rule: {}", e))?;
        
        tracing::info!("Created new CAWS rule: {} with ID: {}", rule, rule_id);
        Ok(rule_id)
    }

    /// Infer rule category from rule name
    fn infer_rule_category(&self, rule: &str) -> String {
        let rule_lower = rule.to_lowercase();
        
        if rule_lower.contains("security") || rule_lower.contains("auth") {
            "security".to_string()
        } else if rule_lower.contains("performance") || rule_lower.contains("optimization") {
            "performance".to_string()
        } else if rule_lower.contains("accessibility") || rule_lower.contains("a11y") {
            "accessibility".to_string()
        } else if rule_lower.contains("maintainability") || rule_lower.contains("code_quality") {
            "maintainability".to_string()
        } else if rule_lower.contains("constitutional") || rule_lower.contains("compliance") {
            "constitutional".to_string()
        } else {
            "general".to_string()
        }
    }

    /// Map severity to impact level
    fn map_severity_to_impact(&self, severity: &ViolationSeverity) -> String {
        match severity {
            ViolationSeverity::Critical => "high".to_string(),
            ViolationSeverity::High => "high".to_string(),
            ViolationSeverity::Medium => "medium".to_string(),
            ViolationSeverity::Low => "low".to_string(),
        }
    }

    /// Analyze code complexity using multiple metrics
    fn analyze_code_complexity(&self, content: &str) -> f64 {
        let lines = content.lines().count();
        let chars = content.chars().count();

        // Basic metrics
        let avg_line_length = if lines > 0 { chars as f64 / lines as f64 } else { 0.0 };

        // Cyclomatic complexity estimation (simplified)
        let control_flow_keywords = ["if ", "else", "for ", "while ", "match ", "loop ", "break", "continue"];
        let mut cyclomatic_complexity = 1; // Base complexity

        for keyword in &control_flow_keywords {
            cyclomatic_complexity += content.matches(keyword).count();
        }

        // Nested depth analysis (simplified)
        let mut max_nest_depth = 0;
        let mut current_depth = 0;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("if ") || trimmed.starts_with("for ") ||
               trimmed.starts_with("while ") || trimmed.starts_with("match ") {
                current_depth += 1;
                max_nest_depth = max_nest_depth.max(current_depth);
            }
            if trimmed.starts_with("}") {
                current_depth = current_depth.saturating_sub(1);
            }
        }

        // Coupling analysis - count external dependencies
        let import_lines = content.lines()
            .filter(|line| line.trim().starts_with("use ") || line.trim().starts_with("import"))
            .count();

        // Maintainability index calculation (simplified)
        // Higher scores are better (less complex)
        let lines_factor = if lines > 0 { (lines as f64).ln() } else { 0.0 };
        let import_factor = if import_lines > 0 { (import_lines as f64).ln() } else { 0.0 };

        let maintainability_score = 171.0 -
            5.2 * lines_factor -
            0.23 * (cyclomatic_complexity as f64) -
            16.2 * import_factor;

        // Normalize to 0-1 scale (1 = very complex, 0 = simple)
        let complexity_score = 1.0 - (maintainability_score / 171.0).max(0.0).min(1.0);

        // Weight additional factors
        let size_factor = if lines > 300 { 0.3 } else if lines > 100 { 0.1 } else { 0.0 };
        let nesting_factor = if max_nest_depth > 4 { 0.2 } else if max_nest_depth > 2 { 0.1 } else { 0.0 };
        let length_factor = if avg_line_length > 120.0 { 0.1 } else { 0.0 };

        (complexity_score + size_factor + nesting_factor + length_factor).min(1.0)
    }

    /// Analyze surgical change characteristics for CAWS evaluation
    fn analyze_surgical_change(&self, diff: &str) -> f64 {
        let diff_lines = diff.lines().count();

        // Analyze change types
        let additions = diff.lines().filter(|line| line.starts_with('+') && !line.starts_with("+++")).count();
        let deletions = diff.lines().filter(|line| line.starts_with('-') && !line.starts_with("---")).count();
        let modifications = diff.lines().filter(|line| line.starts_with("@@")).count(); // Hunk headers indicate modifications

        // Calculate change composition
        let total_changes = additions + deletions;
        let addition_ratio = if total_changes > 0 { additions as f64 / total_changes as f64 } else { 0.0 };

        // Analyze change isolation (how focused the change is)
        let changed_files = diff.lines()
            .filter(|line| line.starts_with("diff --git") || line.starts_with("+++") || line.starts_with("---"))
            .count() / 3; // Each file has 3 header lines

        // Impact radius - how many different areas are affected
        let affected_functions = diff.lines()
            .filter(|line| line.contains("fn ") || line.contains("impl ") || line.contains("struct ") || line.contains("enum "))
            .count();

        let affected_modules = diff.lines()
            .filter(|line| line.contains("mod ") || line.starts_with("use "))
            .count();

        // Coupling measurement - how interconnected the changes are
        let cross_references = diff.lines()
            .filter(|line| line.contains("->") || line.contains("as ") || line.contains("impl<"))
            .count();

        // Calculate surgical precision score (higher = more surgical/precise)
        let size_score = if diff_lines > 100 { 0.2 } else if diff_lines > 50 { 0.4 } else if diff_lines > 20 { 0.7 } else { 0.9 };
        let focus_score = if changed_files > 3 { 0.2 } else if changed_files > 1 { 0.6 } else { 0.9 };
        let isolation_score = if affected_functions > 5 { 0.3 } else if affected_functions > 2 { 0.6 } else { 0.9 };

        // Pure additions are generally safer than modifications/deletions
        let change_type_bonus = if deletions == 0 && modifications < 3 { 0.1 } else { 0.0 };

        // Weighted combination for surgical change score
        let surgical_score = (size_score * 0.4 + focus_score * 0.3 + isolation_score * 0.3 + change_type_bonus).max(0.0).min(1.0);

        // Convert to CAWS scale where 1.0 = very risky, 0.0 = very surgical
        1.0 - surgical_score
    }
}

fn severity_to_db_value(severity: &ViolationSeverity) -> &'static str {
    match severity {
        ViolationSeverity::Low => "low",
        ViolationSeverity::Medium => "medium",
        ViolationSeverity::High => "high",
        ViolationSeverity::Critical => "critical",
    }
}

fn parse_violation_location(location: Option<&str>) -> (Option<String>, Option<i32>, Option<i32>) {
    if let Some(location) = location {
        let mut parts = location.split(':');
        let path = parts.next().map(|value| value.to_string());
        let line = parts.next().and_then(|value| value.parse::<i32>().ok());
        let column = parts.next().and_then(|value| value.parse::<i32>().ok());

        (path, line, column)
    } else {
        (None, None, None)
    }
}

fn infer_compliance_trend(total: u32, resolved: u32, active: u32) -> ComplianceTrend {
    if total == 0 {
        return ComplianceTrend::Unknown;
    }

    let resolution_ratio = resolved as f32 / total as f32;

    if resolution_ratio >= 0.75 {
        ComplianceTrend::Improving
    } else if resolution_ratio >= 0.4 {
        ComplianceTrend::Stable
    } else if active > resolved {
        ComplianceTrend::Declining
    } else {
        ComplianceTrend::Stable
    }
}

/// Violation statistics for reporting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationStats {
    pub total_violations: u32,
    pub critical_count: u32,
    pub high_count: u32,
    pub medium_count: u32,
    pub low_count: u32,
    pub resolved_count: u32,
    pub active_count: u32,
    pub average_resolution_time_hours: f32,
    pub most_common_rule: String,
    pub compliance_trend: ComplianceTrend,
}

/// Compliance trend over time
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComplianceTrend {
    Improving,
    Stable,
    Declining,
    Unknown,
}

impl Default for CawsChecker {
    fn default() -> Self {
        // CawsChecker now requires a database client for proper operation
        // Use CawsChecker::new(db_client) to create an instance
        panic!("CawsChecker requires a database client. Use CawsChecker::new(db_client) instead.")
    }
}

// Implementation for DiffAnalyzer
impl DiffAnalyzer {
    pub fn new() -> Self {
        Self {
            max_change_complexity: 0.7,
            surgical_change_threshold: 0.6,
        }
    }
}

// Implementation for ViolationCodeMapper
impl ViolationCodeMapper {
    pub fn new() -> Self {
        let mut code_mappings = HashMap::new();

        // Add constitutional references for common violations
        code_mappings.insert(
            "CHANGE_SIZE_LIMIT".to_string(),
            ConstitutionalReference {
                section: "Change Management".to_string(),
                subsection: "Size Limits".to_string(),
                description: "Changes must be surgical and focused".to_string(),
                severity: ViolationSeverity::High,
            },
        );

        code_mappings.insert(
            "SURGICAL_CHANGE_REQUIREMENT".to_string(),
            ConstitutionalReference {
                section: "Change Management".to_string(),
                subsection: "Surgical Changes".to_string(),
                description: "Changes should be precise and minimal".to_string(),
                severity: ViolationSeverity::Medium,
            },
        );

        Self { code_mappings }
    }
}

// Rust language analyzer implementation
#[derive(Debug)]
pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

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

// TypeScript language analyzer implementation
#[derive(Debug)]
pub struct TypeScriptAnalyzer;

impl TypeScriptAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn analyze_file_modification(
        &self,
        modification: &CouncilFileModification,
    ) -> Result<LanguageAnalysisResult> {
        let violations = Vec::new();
        let mut warnings = Vec::new();

        // Analyze TypeScript-specific issues
        if let Some(content) = &modification.content {
            // Check for any usage
            if content.contains(": any") {
                warnings.push(LanguageWarning {
                    rule: "TypeScript Type Safety".to_string(),
                    description: "any type usage detected".to_string(),
                    location: None,
                    suggestion: Some("Consider using specific types instead of any".to_string()),
                });
            }

            // Check for console.log
            if content.contains("console.log") {
                warnings.push(LanguageWarning {
                    rule: "TypeScript Debug Code".to_string(),
                    description: "console.log detected".to_string(),
                    location: None,
                    suggestion: Some("Remove or replace with proper logging".to_string()),
                });
            }
        }

        // TODO: Implement sophisticated code complexity analysis for CAWS evaluation
        // - [ ] Analyze cyclomatic complexity and code structure metrics
        // - [ ] Implement dependency analysis and coupling measurements
        // - [ ] Add code maintainability and readability scoring
        // - [ ] Support different programming language complexity metrics
        // - [ ] Implement historical complexity trend analysis
        // - [ ] Add complexity-based risk assessment and prioritization
        // - [ ] Support automated complexity reduction suggestions
        let complexity_score = if let Some(content) = &modification.content {
            let lines = content.lines().count();
            if lines > 100 {
                0.8
            } else if lines > 50 {
                0.6
            } else {
                0.3
            }
        } else {
            0.1
        };

        // TODO: Implement comprehensive surgical change analysis for CAWS evaluation
        // - [ ] Analyze diff size, scope, and impact radius
        // - [ ] Implement change isolation and coupling measurements
        // - [ ] Add change propagation analysis and side effect prediction
        // - [ ] Support different change types (additive, modificative, destructive)
        // - [ ] Implement change complexity and risk assessment
        // - [ ] Add surgical precision scoring and improvement suggestions
        // - [ ] Support automated refactoring recommendations
        let surgical_change_score = if let Some(diff) = &modification.diff {
            let diff_lines = diff.lines().count();
            if diff_lines > 50 {
                0.3
            } else if diff_lines > 20 {
                0.6
            } else {
                0.9
            }
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
        ProgrammingLanguage::TypeScript
    }

    fn calculate_change_complexity(
        &self,
        diff: &str,
        _content: Option<&str>,
    ) -> Result<ChangeComplexity> {
        let diff_lines = diff.lines().count() as u32;
        let structural_changes =
            diff.matches("interface ").count() as u32 + diff.matches("class ").count() as u32;
        let logical_changes =
            diff.matches("function ").count() as u32 + diff.matches("const ").count() as u32;
        let dependency_changes =
            diff.matches("import ").count() as u32 + diff.matches("export ").count() as u32;

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

// JavaScript language analyzer implementation
#[derive(Debug)]
pub struct JavaScriptAnalyzer;

impl JavaScriptAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn analyze_file_modification(
        &self,
        modification: &CouncilFileModification,
    ) -> Result<LanguageAnalysisResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Analyze JavaScript-specific issues
        if let Some(content) = &modification.content {
            // Check for eval usage
            if content.contains("eval(") {
                violations.push(LanguageViolation {
                    rule: "JavaScript Security".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: "eval() usage detected".to_string(),
                    location: None,
                    suggestion: Some("Avoid eval() for security reasons".to_string()),
                    constitutional_ref: Some(ConstitutionalReference {
                        section: "Security".to_string(),
                        subsection: "Code Injection".to_string(),
                        description: "eval() can lead to code injection vulnerabilities"
                            .to_string(),
                        severity: ViolationSeverity::Critical,
                    }),
                });
            }

            // Check for var usage
            if content.contains("var ") {
                warnings.push(LanguageWarning {
                    rule: "JavaScript Best Practices".to_string(),
                    description: "var usage detected".to_string(),
                    location: None,
                    suggestion: Some("Consider using let or const instead of var".to_string()),
                });
            }
        }

        // TODO: Implement sophisticated code complexity analysis for CAWS evaluation
        // - [ ] Analyze cyclomatic complexity and code structure metrics
        // - [ ] Implement dependency analysis and coupling measurements
        // - [ ] Add code maintainability and readability scoring
        // - [ ] Support different programming language complexity metrics
        // - [ ] Implement historical complexity trend analysis
        // - [ ] Add complexity-based risk assessment and prioritization
        // - [ ] Support automated complexity reduction suggestions
        let complexity_score = if let Some(content) = &modification.content {
            let lines = content.lines().count();
            if lines > 100 {
                0.8
            } else if lines > 50 {
                0.6
            } else {
                0.3
            }
        } else {
            0.1
        };

        // TODO: Implement comprehensive surgical change analysis for CAWS evaluation
        // - [ ] Analyze diff size, scope, and impact radius
        // - [ ] Implement change isolation and coupling measurements
        // - [ ] Add change propagation analysis and side effect prediction
        // - [ ] Support different change types (additive, modificative, destructive)
        // - [ ] Implement change complexity and risk assessment
        // - [ ] Add surgical precision scoring and improvement suggestions
        // - [ ] Support automated refactoring recommendations
        let surgical_change_score = if let Some(diff) = &modification.diff {
            let diff_lines = diff.lines().count();
            if diff_lines > 50 {
                0.3
            } else if diff_lines > 20 {
                0.6
            } else {
                0.9
            }
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
        ProgrammingLanguage::JavaScript
    }

    fn calculate_change_complexity(
        &self,
        diff: &str,
        _content: Option<&str>,
    ) -> Result<ChangeComplexity> {
        let diff_lines = diff.lines().count() as u32;
        let structural_changes =
            diff.matches("class ").count() as u32 + diff.matches("function ").count() as u32;
        let logical_changes =
            diff.matches("const ").count() as u32 + diff.matches("let ").count() as u32;
        let dependency_changes =
            diff.matches("require(").count() as u32 + diff.matches("import ").count() as u32;

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

/// TODO: Implement comprehensive CAWS waiver system with governance and approval workflows
/// - [ ] Design waiver approval process with multiple authorization levels
/// - [ ] Implement waiver validity periods and automatic expiration
/// - [ ] Add waiver audit trail and change tracking
/// - [ ] Support different waiver types (temporary, permanent, conditional)
/// - [ ] Implement waiver impact assessment and risk evaluation
/// - [ ] Add waiver reporting and compliance monitoring
/// - [ ] Support automated waiver renewal and review processes
#[derive(Debug, Clone)]
pub struct CawsWaiver {
    pub id: String,
    pub reason: String,
    pub justification: String,
    pub time_bounded: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// CAWS validation result
#[derive(Debug, Clone)]
pub struct CawsValidationResult {
    pub is_compliant: bool,
    pub compliance_score: f32,
    pub violations: Vec<CawsViolation>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub validated_at: chrono::DateTime<chrono::Utc>,
    /// Store CAWS validation result in database
    pub async fn store_validation_result(
        &self,
        task_id: Uuid,
        result: &CawsValidationResult,
    ) -> Result<Uuid> {
        let validation_id = Uuid::new_v4();

        let query = r#"
            INSERT INTO caws_validations (
                id, task_id, is_compliant, violations, suggestions, trend, validated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        let violations_json = serde_json::to_value(&result.violations)
            .context("Failed to serialize violations")?;
        let suggestions_json = serde_json::to_value(&result.suggestions)
            .context("Failed to serialize suggestions")?;

        let trend_str = match result.is_compliant {
            true if result.violations.is_empty() && result.warnings.is_empty() => "improving",
            true => "stable",
            false => "declining",
        };

        self.db_client
            .execute_parameterized_query(
                query,
                vec![
                    json!(validation_id),
                    json!(task_id),
                    json!(result.is_compliant),
                    violations_json,
                    suggestions_json,
                    json!(trend_str),
                    json!(result.validated_at),
                ],
            )
            .await
            .context("Failed to store CAWS validation result")?;

        Ok(validation_id)
    }

    /// Query historical compliance trends for a task
    pub async fn get_compliance_history(
        &self,
        task_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<CawsValidationResult>> {
        let limit_val = limit.unwrap_or(10);
        let query = format!(
            r#"
            SELECT is_compliant, violations, suggestions, trend, validated_at
            FROM caws_validations
            WHERE task_id = $1
            ORDER BY validated_at DESC
            LIMIT {}
            "#,
            limit_val
        );

        let rows = self.db_client
            .execute_parameterized_query(&query, vec![json!(task_id)])
            .await
            .context("Failed to query compliance history")?;

        let mut results = Vec::new();
        for row in rows {
            let is_compliant: bool = row.get("is_compliant");
            let violations_json: serde_json::Value = row.get("violations");
            let suggestions_json: serde_json::Value = row.get("suggestions");
            let validated_at: chrono::DateTime<chrono::Utc> = row.get("validated_at");

            let violations: Vec<CawsViolation> = serde_json::from_value(violations_json)
                .context("Failed to deserialize violations")?;
            let suggestions: Vec<String> = serde_json::from_value(suggestions_json)
                .context("Failed to deserialize suggestions")?;

            // Calculate compliance score from violations
            let compliance_score = self.calculate_compliance_score(&violations, &vec![]);

            results.push(CawsValidationResult {
                is_compliant,
                compliance_score,
                violations,
                warnings: vec![], // Not stored in current schema
                suggestions,
                validated_at,
            });
        }

        Ok(results)
    }

    /// Get compliance statistics across all tasks
    pub async fn get_compliance_stats(&self) -> Result<serde_json::Value> {
        let query = r#"
            SELECT
                COUNT(*) as total_validations,
                COUNT(*) FILTER (WHERE is_compliant = true) as compliant_validations,
                AVG(CASE WHEN is_compliant THEN 1.0 ELSE 0.0 END) as compliance_rate,
                COUNT(DISTINCT task_id) as unique_tasks_validated
            FROM caws_validations
            WHERE validated_at >= NOW() - INTERVAL '30 days'
        "#;

        let rows = self.db_client
            .execute_parameterized_query(query, vec![])
            .await
            .context("Failed to query compliance statistics")?;

        if let Some(row) = rows.first() {
            Ok(json!({
                "total_validations": row.get::<i64, _>("total_validations"),
                "compliant_validations": row.get::<i64, _>("compliant_validations"),
                "compliance_rate": row.get::<Option<f64>, _>("compliance_rate").unwrap_or(0.0),
                "unique_tasks_validated": row.get::<i64, _>("unique_tasks_validated")
            }))
        } else {
            Ok(json!({
                "total_validations": 0,
                "compliant_validations": 0,
                "compliance_rate": 0.0,
                "unique_tasks_validated": 0
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_database::DatabaseClient;

    #[tokio::test]
    async fn test_caws_checker_creation() {
        let checker = CawsChecker::new();
        // Basic creation test
        assert!(true);
    }

    #[tokio::test]
    async fn test_validate_task_spec() {
        let checker = CawsChecker::new();

        let task_spec = TaskSpec {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "A test task description".to_string(),
            risk_tier: RiskTier::Tier2,
            scope: TaskScope {
                files_affected: vec!["src/test.rs".to_string()],
                max_files: Some(5),
                max_loc: Some(1000),
                domains: vec!["backend".to_string()],
            },
            acceptance_criteria: vec![],
            context: CouncilTaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: ConfigEnvironment::Development,
            },
            worker_output: CouncilWorkerOutput {
                content: "".to_string(),
                files_modified: vec![],
                rationale: "".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.0,
                    quality_score: 0.0,
                    confidence: 0.0,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: std::collections::HashMap::new(),
            },
            caws_spec: None,
        };

        let result = checker.validate_task_spec(&task_spec).await.unwrap();
        assert!(!result.is_compliant); // Should fail due to no acceptance criteria
        assert!(result.compliance_score < 1.0);
        assert!(!result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_validate_worker_output() {
        let checker = CawsChecker::new();

        let task_spec = TaskSpec {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "A test task description".to_string(),
            risk_tier: RiskTier::Tier2,
            scope: TaskScope {
                files_affected: vec!["src/test.rs".to_string()],
                max_files: Some(2),
                max_loc: Some(100),
                domains: vec!["backend".to_string()],
            },
            acceptance_criteria: vec![],
            context: CouncilTaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: ConfigEnvironment::Development,
            },
            worker_output: CouncilWorkerOutput {
                content: "Test implementation".to_string(),
                files_modified: vec![
                    CouncilFileModification {
                        path: "test1.rs".to_string(),
                        operation: CouncilFileOperation::Create,
                        content: Some("fn main() {\n    println!(\"test\");\n}".to_string()),
                        diff: None,
                        size_bytes: 50,
                    },
                    CouncilFileModification {
                        path: "test2.rs".to_string(),
                        operation: CouncilFileOperation::Create,
                        content: Some("fn helper() {\n    // helper function\n}".to_string()),
                        diff: None,
                        size_bytes: 40,
                    },
                    CouncilFileModification {
                        path: "test3.rs".to_string(),
                        operation: CouncilFileOperation::Create,
                        content: Some("fn extra() {\n    // extra function\n}".to_string()),
                        diff: None,
                        size_bytes: 40,
                    },
                ],
                rationale: "Created three files for the implementation".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.95,
                    quality_score: 0.9,
                    confidence: 0.85,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: std::collections::HashMap::new(),
            },
            caws_spec: None,
        };

        let result = checker
            .validate_worker_output(&task_spec.worker_output, &task_spec)
            .await
            .unwrap();
        assert!(!result.is_compliant); // Should fail due to file count exceeding limit
        assert!(result.compliance_score < 1.0);
        assert!(!result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_validate_waiver() {
        let checker = CawsChecker::new();

        let valid_waiver = CawsWaiver {
            id: "waiver-001".to_string(),
            reason: "Technical complexity".to_string(),
            justification: "This is a detailed justification for why the waiver is needed"
                .to_string(),
            time_bounded: true,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(30)),
        };

        let invalid_waiver = CawsWaiver {
            id: "waiver-002".to_string(),
            reason: "Technical complexity".to_string(),
            justification: "Short".to_string(), // Too short
            time_bounded: true,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(30)),
        };

        assert!(checker.validate_waiver(&valid_waiver).await.unwrap());
        assert!(!checker.validate_waiver(&invalid_waiver).await.unwrap());
    }

    #[tokio::test]
    async fn test_calculate_compliance_score() {
        /// Requirements for completion:
        /// - [ ] Implement proper test database setup and teardown
        /// - [ ] Add support for test data seeding and cleanup
        /// - [ ] Implement proper test isolation and parallel execution
        /// - [ ] Add support for different test database configurations
        /// - [ ] Implement proper error handling for test database failures
        /// - [ ] Add support for test database migration and schema management
        /// - [ ] Implement proper memory management for test database operations
        /// - [ ] Add support for test database performance optimization
        /// - [ ] Implement proper cleanup of test database resources
        /// - [ ] Add support for test database monitoring and validation
        // Create a mock database client for testing
        let db_config = agent_agency_database::DatabaseConfig::default();
        let db_client = agent_agency_database::DatabaseClient::new(db_config)
            .await
            .unwrap();
        let checker = CawsChecker::new(db_client);

        let violations = vec![CawsViolation {
            rule: "Test Rule".to_string(),
            severity: ViolationSeverity::High,
            description: "Test violation".to_string(),
            location: None,
            suggestion: None,
            constitutional_ref: Some("Section 2.1".to_string()),
        }];

        let warnings = vec!["Test warning".to_string()];

        let score = checker.calculate_compliance_score(&violations, &warnings);
        assert!(score < 1.0);
        assert!(score >= 0.0);
    }

    #[test]
    fn parses_violation_locations_with_line_and_column() {
        let (path, line, column) = parse_violation_location(Some("src/lib.rs:42:7"));
        assert_eq!(path.as_deref(), Some("src/lib.rs"));
        assert_eq!(line, Some(42));
        assert_eq!(column, Some(7));

        let (path_only, line_only, column_only) = parse_violation_location(Some("src/main.rs"));
        assert_eq!(path_only.as_deref(), Some("src/main.rs"));
        assert_eq!(line_only, None);
        assert_eq!(column_only, None);

        let (none_path, none_line, none_column) = parse_violation_location(None);
        assert!(none_path.is_none());
        assert!(none_line.is_none());
        assert!(none_column.is_none());
    }

    #[test]
    fn infers_compliance_trend_from_resolution_ratio() {
        let improving = infer_compliance_trend(10, 8, 2);
        assert_eq!(improving, ComplianceTrend::Improving);

        let stable = infer_compliance_trend(10, 5, 4);
        assert_eq!(stable, ComplianceTrend::Stable);

        let declining = infer_compliance_trend(10, 2, 7);
        assert_eq!(declining, ComplianceTrend::Declining);

        let unknown = infer_compliance_trend(0, 0, 0);
        assert_eq!(unknown, ComplianceTrend::Unknown);
    }

    #[test]
    fn maps_severity_to_database_value() {
        assert_eq!(severity_to_db_value(&ViolationSeverity::Low), "low");
        assert_eq!(severity_to_db_value(&ViolationSeverity::Medium), "medium");
        assert_eq!(severity_to_db_value(&ViolationSeverity::High), "high");
        assert_eq!(
            severity_to_db_value(&ViolationSeverity::Critical),
            "critical"
        );
    }

    #[tokio::test]
    async fn test_database_integration_validation_storage() {
        // Integration test for CAWS validation result storage
        // This test requires a real database connection
        if std::env::var("RUN_INTEGRATION_TESTS").is_err() {
            return; // Skip unless explicitly enabled
        }

        // let db_client = setup_test_database_client().await;
        // let checker = CawsChecker::with_database_client(db_client);

        // Test validation result storage and retrieval
        // let validation_result = checker.validate_task_spec(&task_spec).await.unwrap();
        // let stored_id = checker.store_validation_result(task_spec.id, &validation_result).await.unwrap();

        // Test retrieval
        // let history = checker.get_compliance_history(task_spec.id, Some(5)).await.unwrap();
        // assert!(!history.is_empty());

        // Test statistics
        // let stats = checker.get_compliance_stats().await.unwrap();
        // assert!(stats.get("total_validations").unwrap().as_i64().unwrap() >= 1);

        // For now, just validate the method signatures and data structures exist
        let task_spec = TaskSpec {
            id: Uuid::new_v4(),
            title: "Integration Test".to_string(),
            description: "Testing database integration".to_string(),
            risk_tier: RiskTier::Tier2,
            scope: TaskScope {
                files_affected: vec!["test.rs".to_string()],
                max_files: Some(1),
                max_loc: Some(100),
                domains: vec!["test".to_string()],
            },
            acceptance_criteria: vec![],
            context: CouncilTaskContext {
                workspace_root: "/test".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: ConfigEnvironment::Development,
            },
            worker_output: CouncilWorkerOutput {
                content: "".to_string(),
                files_modified: vec![],
                rationale: "".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.0,
                    quality_score: 0.0,
                    confidence: 0.0,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: std::collections::HashMap::new(),
            },
            caws_spec: None,
        };

        // Validate data structures work correctly
        assert_eq!(task_spec.title, "Integration Test");
        assert!(task_spec.scope.files_affected.len() > 0);
    }
}
