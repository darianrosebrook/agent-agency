//! CAWS (Coding Agent Workflow System) module
//!
//! This module provides CAWS compliance checking and validation for worker outputs.
//! It has been refactored from a monolithic 2,700+ line file into focused modules.

pub mod language_types;
pub mod diff_analysis;
pub mod violation_mapper;
pub mod analyzers;
pub mod compliance;
pub mod checker;

// Re-export main types for backward compatibility
pub use checker::CawsChecker;
pub use compliance::{CawsWaiver, CawsValidationResult};
pub use language_types::ProgrammingLanguage;
pub use diff_analysis::{DiffAnalyzer, DiffAnalysisResult};
pub use violation_mapper::ViolationCodeMapper;
