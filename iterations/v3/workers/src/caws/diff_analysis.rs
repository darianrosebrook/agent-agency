//! Diff analysis and complexity calculation for CAWS
//!
//! This module handles AST-based diff analysis, change complexity scoring,
//! and surgical change detection for better CAWS compliance assessment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AST-based diff analyzer for surgical change scoring
#[derive(Debug)]
pub struct DiffAnalyzer {
    // Configuration for diff analysis
    pub max_complexity: f32,
    pub complexity_weights: HashMap<String, f32>,
}

/// Change complexity metrics
#[derive(Debug, Clone)]
pub struct ChangeComplexity {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub nesting_depth: u32,
    pub parameter_count: u32,
    pub line_count: u32,
    pub total_score: f32,
}

/// Scope of changes in a diff
#[derive(Debug, Clone)]
pub struct DiffScope {
    pub files_modified: Vec<String>,
    pub functions_added: Vec<String>,
    pub functions_modified: Vec<String>,
    pub functions_removed: Vec<String>,
    pub classes_added: Vec<String>,
    pub classes_modified: Vec<String>,
    pub classes_removed: Vec<String>,
}

/// Type of change detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Addition,
    Modification,
    Deletion,
    Refactoring,
    BugFix,
    Feature,
    Breaking,
    Documentation,
    Test,
    Configuration,
}

/// Result of diff analysis
#[derive(Debug, Clone)]
pub struct DiffAnalysisResult {
    pub scope: DiffScope,
    pub complexity: ChangeComplexity,
    pub change_type: ChangeType,
    pub risk_score: f32,
    pub surgical_score: f32,
    pub recommendations: Vec<String>,
}

/// Recommended actions based on analysis
#[derive(Debug, Clone)]
pub enum RecommendedAction {
    Approve,
    RequestChanges,
    RequireReview,
    Block,
    Escalate,
}

impl DiffAnalyzer {
    /// Create a new diff analyzer
    pub fn new() -> Self {
        Self {
            max_complexity: 10.0,
            complexity_weights: HashMap::new(),
        }
    }

    /// Analyze changes and calculate complexity
    pub fn analyze_changes(&self, _old_content: &str, _new_content: &str) -> DiffAnalysisResult {
        // TODO: Implement actual diff analysis
        DiffAnalysisResult {
            scope: DiffScope {
                files_modified: vec![],
                functions_added: vec![],
                functions_modified: vec![],
                functions_removed: vec![],
                classes_added: vec![],
                classes_modified: vec![],
                classes_removed: vec![],
            },
            complexity: ChangeComplexity {
                cyclomatic_complexity: 0,
                cognitive_complexity: 0,
                nesting_depth: 0,
                parameter_count: 0,
                line_count: 0,
                total_score: 0.0,
            },
            change_type: ChangeType::Modification,
            risk_score: 0.0,
            surgical_score: 0.0,
            recommendations: vec![],
        }
    }
}


// Moved from caws_checker.rs: DiffAnalyzer struct
#[derive(Debug)]
pub struct DiffAnalyzer {
    // Configuration for diff analysis
    max_change_complexity: f32,
    surgical_change_threshold: f32,
}



// Moved from caws_checker.rs: ChangeComplexity struct
#[derive(Debug, Clone)]
pub struct ChangeComplexity {
    pub structural_changes: u32,
    pub logical_changes: u32,
    pub dependency_changes: u32,
    pub complexity_score: f32,
    pub is_surgical: bool,
    /// Cyclomatic complexity score from AST analysis
    pub cyclomatic_complexity: u32,
    /// Diff scope analysis (lines changed, files affected)
    pub diff_scope: DiffScope,
}



// Moved from caws_checker.rs: DiffScope struct
#[derive(Debug, Clone)]
pub struct DiffScope {
    /// Number of lines changed in the diff
    pub lines_changed: u32,
    /// Number of files affected
    pub files_affected: u32,
    /// Estimated blast radius (how many modules/components affected)
    pub blast_radius: u32,
    /// Change type classification
    pub change_type: ChangeType,
}



// Moved from caws_checker.rs: ChangeType enum
#[derive(Debug, Clone)]
pub enum ChangeType {
    /// Simple variable or constant changes
    Variable,
    /// Function or method modifications
    Function,
    /// Class or interface changes
    Structural,
    /// Import/export dependency changes
    Dependency,
    /// Configuration or build system changes
    Configuration,
    /// Test-only changes
    Test,
    /// Documentation changes
    Documentation,
    /// Mixed or complex changes
    Mixed,
}



// Moved from caws_checker.rs: DiffAnalysisResult struct
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



// Moved from caws_checker.rs: RecommendedAction enum
#[derive(Debug, Clone)]
pub enum RecommendedAction {
    Accept,
    RequestSmallerChanges,
    SplitIntoMultiplePRs,
    AddMoreTests,
    ImproveDocumentation,
    RequestReview,
}



// Moved from caws_checker.rs: DiffAnalyzer impl block
impl DiffAnalyzer {
    pub fn new() -> Self {
        Self {
            max_change_complexity: 0.7,
            surgical_change_threshold: 0.6,
        }
    }
}

