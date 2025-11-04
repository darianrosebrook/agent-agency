//! Evidence Collection Module
//!
//! Comprehensive evidence collection system for claim verification
//! with multiple verification methods and quality assessment.

pub mod types;
pub mod evidence_types;
pub mod collector;
pub mod code_analysis;
pub mod test_execution;
pub mod documentation;
pub mod performance;
pub mod security;
pub mod constitutional;
pub mod filtering;
pub mod evidence_analysis;

// Re-export main types and functionality
pub use types::*;
pub use collector::EvidenceCollector;
pub use types::EvidenceCollectorConfig;
pub use code_analysis::CodeAnalysisCollector;
pub use test_execution::TestExecutionCollector;
pub use documentation::DocumentationCollector;
pub use performance::PerformanceCollector;
pub use security::SecurityCollector;
pub use constitutional::ConstitutionalCollector;
pub use filtering::EvidenceFilter;
