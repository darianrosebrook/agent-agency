//! Disambiguation module for claim extraction
//!
//! This module provides functionality to identify and resolve ambiguities in text
//! to prepare it for claim extraction. It uses a multi-stage approach:
//!
//! 1. **Detection**: Identify pronouns, technical terms, scope boundaries, and temporal references
//! 2. **Context Resolution**: Use domain hints and surrounding context to resolve ambiguities
//! 3. **Entity Recognition**: Extract and disambiguate named entities (optional)
//!
//! ## Feature Flags
//!
//! - `simulated_kb`: Enable simulated knowledge base for development (default: on)
//! - `embedding`: Enable real embedding provider integration
//! - `kb`: Enable real knowledge base integration
//! - `ingest`: Enable knowledge ingestion capabilities

pub mod context;
pub mod detection;
pub mod disambiguation_types;
pub mod entities;
pub mod patterns;
pub mod stage;
pub mod types;

// Re-export public API
pub use types::{
    // Ambiguity types
    Ambiguity,
    AmbiguityType,
    ContextAwareDisambiguation,
    DisambiguationResult,
    DomainIntegration,
    // Traits
    EmbeddingProvider,
    EntityMatch,
    EntityRelationship,
    // Entity types
    EntityType,
    // Supporting types
    ExternalKnowledgeEntity,
    // Analysis helpers
    HistoricalEntityAnalysis,
    IngestionCacheEntry,
    IngestionCandidate,
    IngestionChannel,
    IngestionPipelineStats,
    // Knowledge base types
    KbSource,
    KnowledgeBase,
    KnowledgeBaseResult,
    KnowledgeIngest,
    // Core types
    Language,
    NamedEntity,
    ReferentInfo,
    RelatedEntity,
    ResolvedEntity,
    UnresolvableAmbiguity,
    UnresolvableReason,
};

pub use context::ContextResolver;
pub use detection::AmbiguityDetector;
pub use entities::NamedEntityRecognizer;
pub use patterns::EntityPatterns;
pub use stage::DisambiguationStage;

// Convenience constructors for common configurations

/// Create a minimal disambiguation stage with no optional integrations
///
/// This provides basic regex-based disambiguation without external services.
/// Suitable for simple use cases or when external dependencies are not available.
pub fn minimal_stage() -> DisambiguationStage {
    DisambiguationStage::minimal()
}

/// Create a disambiguation stage with simulated services for development
///
/// Uses simulated implementations of embedding and knowledge base services.
/// Useful for development and testing when real services are not available.
#[cfg(feature = "simulated_kb")]
pub fn simulated_stage() -> DisambiguationStage {
    // Simulated implementations would be provided here
    DisambiguationStage::minimal()
}

/// Create a fully configured disambiguation stage with real services
///
/// Requires all optional features to be enabled. Use this in production
/// environments where all external services are available.
#[cfg(all(feature = "embedding", feature = "kb", feature = "ingest"))]
pub fn full_stage(
    embedding_provider: std::sync::Arc<dyn EmbeddingProvider>,
    knowledge_base: std::sync::Arc<dyn KnowledgeBase>,
    knowledge_ingest: std::sync::Arc<dyn KnowledgeIngest>,
) -> DisambiguationStage {
    DisambiguationStage::with_services(
        Some(embedding_provider),
        Some(knowledge_base),
        Some(knowledge_ingest),
    )
}

// Re-export async_trait for convenience when implementing traits
pub use async_trait::async_trait;
