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
//! imported from `crate::types`.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::result_large_err)] // adjust as needed

// --- Submodules (internal implementation details) ---
mod types;
mod verifier;
mod coreference;
mod disambiguation;
mod authority_validator;
mod semantic_analyzer;
mod keyword_matcher;
mod code_extractor;
mod documentation_extractor;
mod data_extractor;
mod spec_analysis;
mod historical;
mod fs_utils;

// --- Public re-exports (crate-facing API surface) ---

// Engine / primary entrypoints
pub use verifier::MultiModalVerificationEngine;

// Internal verification models used by callers (stable API)
pub use types::{
    // Entity & coref
    Entity, EntityType, CoreferenceChain, CoreferenceResolution, CoreferenceType,
    // Disambiguation
    EntityDisambiguation, EntityCandidate, DisambiguationMethod,
    // Code/doc/data outputs & specs
    CodeOutput, CodeSpecification, DocumentationOutput, DocumentationStandards,
    DataAnalysisOutput, DataSchema,
    // Pattern/statistics/correlations containers
    StatisticalResult, PatternResult, CorrelationResult,
    // Matching
    KeywordMatch, MatchType,
    // Helpers
    CheckResult,
};

// Frequently-used utilities optionally exposed (keep narrow)
pub use coreference::resolve_coreferences;
pub use disambiguation::disambiguate_entity;

// If you want a small convenience prelude for downstream modules/tests:
pub mod prelude {
    pub use super::{
        CoreferenceResolution, CoreferenceType, DataAnalysisOutput, DataSchema,
        DisambiguationMethod, Entity, EntityCandidate, EntityDisambiguation, EntityType,
        KeywordMatch, MatchType, MultiModalVerificationEngine, PatternResult,
        StatisticalResult,
    };
}

// --- External/shared types imported (but not re-exported here) ---
// Keep shared domain types outside this module to avoid name clashes with our
// internal `types` module. Downstream code can import from `crate::types` directly.
use crate::types as shared_types;

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
