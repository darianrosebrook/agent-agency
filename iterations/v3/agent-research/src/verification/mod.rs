// claim_extraction/verification/mod.rs
//! Multi-Modal Verification Engine (V3)
//!
//! This crate module composes the verification pipeline: claim extraction,
//! cross-reference validation, authority checks, context/coreference,
//! code/documentation/data analysis, and historical lookups.
//!
//! Structure:
//! - types.rs                : Core data structures used within verification
//! - verifier.rs             : Orchestrator (MultiModalVerificationEngine) & high-level APIs
//! - coreference.rs          : Coreference detection/resolution + caching
//! - disambiguation.rs       : Entity disambiguation strategies
//! - authority_validator.rs  : Source credibility/authority checks
//! - semantic_analyzer.rs    : Semantic parsing, intent & meaning analysis
//! - keyword_matcher.rs      : Keyword/context/fuzzy matching utilities
//! - code_extractor.rs       : Code parsing & code-derived claim extraction
//! - documentation_extractor.rs : Documentation parsing & doc-derived claims
//! - data_extractor.rs       : Data/statistics/parsing & data-derived claims
//! - spec_analysis.rs        : Specification discovery/coverage & relevance
//! - historical.rs           : Historical-claims lookup/aggregation (DB + fallback)
//! - fs_utils.rs             : Filesystem traversal & content helpers
//!
//! External/shared types (e.g., `AtomicClaim`, `VerificationResults`, etc.) are
//! imported from `system_configuration::types`.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::result_large_err)] // adjust as needed

// --- Submodules (internal implementation details) ---
mod authority_validator;
mod code_extractor;
mod coreference;
mod data_extractor;
mod disambiguation;
mod documentation_extractor;
mod fs_utils;
mod historical;
mod keyword_matcher;
mod semantic_analyzer;
mod spec_analysis;
mod types;
mod verification_types;
mod verifier;

// --- Public re-exports (crate-facing API surface) ---

// Re-export Entity types from verification_types
pub use verification_types::{Entity, EntityCandidate, EntityDisambiguation, EntityType};

// Engine / primary entrypoints
pub use verifier::MultiModalVerificationEngine;

// Internal verification models used by callers (stable API)
pub use types::{
    // Helpers
    CheckResult,
    // Code/doc/data outputs & specs
    CodeOutput,
    CodeSpecification,
    // Entity & coref
    CoreferenceChain,
    CoreferenceResolution,
    CoreferenceType,
    CorrelationResult,
    DataAnalysisOutput,
    DataSchema,
    // Disambiguation
    DisambiguationMethod,
    DocumentationOutput,
    DocumentationStandards,
    // Matching
    KeywordMatch,
    MatchType,
    PatternResult,
    // Pattern/statistics/correlations containers
    StatisticalResult,
};

// Re-export verification_types
pub use verification_types::{
    CodeStructure, DocumentationStructure, FunctionDefinition, ImplementationBlock,
    TestConsistency, TestCoverage, TestOutput, TestQuality, TestRelevance, TypeDefinition,
    UsageExample,
};

// Frequently-used utilities optionally exposed (keep narrow)
pub use coreference::resolve_coreferences;
pub use disambiguation::disambiguate_entity;

// If you want a small convenience prelude for downstream modules/tests:
pub mod prelude {
    pub use super::{
        CoreferenceResolution, CoreferenceType, DataAnalysisOutput, DataSchema,
        DisambiguationMethod, Entity, EntityCandidate, EntityDisambiguation, EntityType,
        KeywordMatch, MatchType, MultiModalVerificationEngine, PatternResult, StatisticalResult,
    };
}

// --- External/shared types imported (but not re-exported here) ---
// Keep shared domain types outside this module to avoid name clashes with our
// internal `types` module. Downstream code can import from `system_configuration::types` directly.
use system_configuration::types as shared_types;

// --- Feature gates / optional DB integration sketch ---
// If DB-specific pieces are behind a feature, surface them here as needed.
// #[cfg(feature = "db")]
// pub use historical::DbHandles;

// --- Crate-level tests for linkage/smoke ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_builds() {
        let _engine = MultiModalVerificationEngine::new();
    }
}
