#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

// CAWS Runtime Validator modules
pub mod policy;
pub mod validator;
pub mod validation_budget;
pub mod waiver;

// Development workflow modules
pub mod integration;
pub mod analyzers;
pub mod codemod;
pub mod templates;

// Minimal diff evaluator modules (consolidated)
pub mod ast_analyzer;
pub mod change_classifier;
pub mod evaluator;
pub mod evaluator_types;
pub mod impact_analyzer;
pub mod language_support;

// Re-export key functionality
pub use policy::{CawsPolicy, PolicyValidator};
pub use validator::{CawsValidator, ValidationResult, Violation};
pub use validation_budget::{BudgetChecker, BudgetLimits, BudgetState};
pub use waiver::{WaiverGenerator, WaiverManager};
pub use integration::{McpIntegration, OrchestrationIntegration};
pub use analyzers::{
    LanguageAnalyzer, LanguageAnalyzerRegistry, LanguageAnalysisResult,
    ProgrammingLanguage, LanguageViolation, LanguageWarning, SourceLocation, ViolationSeverity,
    RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer,
};
pub use codemod::CodeModRunner;
pub use templates::TemplateManager;

// Minimal diff evaluator re-exports
pub use ast_analyzer::ASTAnalyzer;
pub use change_classifier::ChangeClassifier;
pub use evaluator::MinimalDiffEvaluator;
pub use impact_analyzer::ImpactAnalyzer;
pub use language_support::LanguageSupport;
