#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

// CAWS Runtime Validator modules
pub mod policy;
pub mod validation_budget;
pub mod validator;
pub mod waiver;

// Development workflow modules
pub mod analyzers;
pub mod codemod;
pub mod integration;
pub mod templates;

// Minimal diff evaluator modules (consolidated)
pub mod ast_analyzer;
pub mod change_classifier;
pub mod evaluator;
pub mod evaluator_types;
pub mod impact_analyzer;
pub mod language_support;

// Re-export key functionality
pub use analyzers::{
    JavaScriptAnalyzer, LanguageAnalysisResult, LanguageAnalyzer, LanguageAnalyzerRegistry,
    LanguageViolation, LanguageWarning, ProgrammingLanguage, RustAnalyzer, SourceLocation,
    TypeScriptAnalyzer, ViolationSeverity,
};
pub use codemod::CodeModRunner;
pub use integration::{McpIntegration, OrchestrationIntegration};
pub use policy::{CawsPolicy, PolicyValidator};
pub use templates::TemplateManager;
pub use validation_budget::{BudgetChecker, BudgetLimits, BudgetState};
pub use validator::{CawsValidator, ValidationResult, Violation};
pub use waiver::{WaiverGenerator, WaiverManager};

// Minimal diff evaluator re-exports
pub use ast_analyzer::ASTAnalyzer;
pub use change_classifier::ChangeClassifier;
pub use evaluator::MinimalDiffEvaluator;
pub use impact_analyzer::ImpactAnalyzer;
pub use language_support::LanguageSupport;
