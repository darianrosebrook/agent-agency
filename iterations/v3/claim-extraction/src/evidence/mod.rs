//! Evidence Collection Module
//!
//! Comprehensive evidence collection system for claim verification
//! with multiple verification methods and quality assessment.

pub mod types;
pub mod collector;
pub mod code_analysis;
pub mod test_execution;
pub mod documentation;
pub mod performance;
pub mod security;
pub mod constitutional;
pub mod filtering;
pub mod analysis;

// Re-export main types and functionality
pub use types::*;
pub use collector::EvidenceCollector;
pub use types::EvidenceCollectorConfig;
