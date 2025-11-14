//! Evidence Collection Module
//!
//! Comprehensive evidence collection system for claim verification
//! with multiple verification methods and quality assessment.

pub mod code_analysis;
pub mod collector;
pub mod common;
pub mod constitutional;
pub mod documentation;
pub mod evidence_analysis;
pub mod evidence_types;
pub mod filtering;
pub mod performance;
pub mod security;
pub mod test_execution;
pub mod types;

// Re-export main types and functionality
pub use code_analysis::CodeAnalysisCollector;
pub use collector::EvidenceCollector;
pub use constitutional::ConstitutionalCollector;
pub use documentation::DocumentationCollector;
pub use filtering::EvidenceFilter;
pub use performance::PerformanceCollector;
pub use security::SecurityCollector;
pub use test_execution::TestExecutionCollector;
pub use types::EvidenceCollectorConfig;
pub use types::*;
