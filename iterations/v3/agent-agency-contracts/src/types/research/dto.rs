//! Data Transfer Objects for research operations
//! 
//! These types cross crate boundaries and must be serializable.
//! They do NOT contain runtime types (Arc<dyn Trait>, etc.)

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use schemars::JsonSchema;

/// Opaque key for entities, can evolve to {ns, id}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityKey(pub String);

impl EntityKey {
    pub fn new(key: String) -> Self {
        Self(key)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Technology,
    Concept,
    Code,
    // Additional variants used in disambiguation
    Date,
    TechnicalTerm,
    Money,
    Percent,
    // Additional variants from verification module
    CodeEntity, // Alias for Code - used in disambiguation
    SystemComponent, // System components like APIs, services
    Documentation,
    Data,
    Other(String), // Allows custom entity types
}

// Manual Hash implementation since Other(String) prevents auto-derive
impl std::hash::Hash for EntityType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        if let EntityType::Other(ref s) = self {
            s.hash(state);
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct EntityMatch {
    pub entity: String,
    pub entity_type: EntityType,
    /// Confidence score in range [0.0, 1.0]
    pub confidence: f64,
    /// Start position (byte index in UTF-8)
    pub start_pos: usize,
    /// End position (byte index in UTF-8)
    pub end_pos: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct UnresolvableAmbiguity {
    pub ambiguity: String,
    pub suggested_context: Option<String>,
    pub reason: UnresolvableReason,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationMethod {
    CodeAnalysis,
    TestExecution,
    PerformanceMeasurement,
    Measurement,
    LogicalAnalysis,
    ProcessAnalysis,
    // Additional variants used in agent-research
    DocumentationReview,
    SecurityScan,
    ConstitutionalCheck,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnresolvableReason {
    SubjectiveLanguage,
    InsufficientContext,
    AmbiguousReference,
    MissingInformation,
    ConflictingEvidence,
}

/// Opaque embedding vector - prevents infra types from leaking through
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct Embedding(pub Vec<f32>);

impl Embedding {
    pub fn into_vec(self) -> Vec<f32> {
        self.0
    }
    
    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }
}

/// Query type for research and multimodal retrieval operations
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueryType {
    /// General knowledge search
    Knowledge,
    /// Code-specific research
    Code,
    /// Documentation search
    Documentation,
    /// API reference lookup
    ApiReference,
    /// Error troubleshooting
    Troubleshooting,
    /// Best practices research
    BestPractices,
    /// Technical research
    Technical,
    /// Text-based query
    Text,
    /// Image-based query
    Image,
    /// Visual search query
    Visual,
    /// Hybrid text and image query
    Hybrid,
    /// Timestamp-anchored search
    TimestampAnchored,
    /// Text search (data processing)
    TextSearch,
    /// Semantic search (data processing)
    SemanticSearch,
    /// Entity search (data processing)
    EntitySearch,
    /// Hybrid search (data processing)
    HybridSearch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_match_invariants() {
        let m = EntityMatch {
            entity: "test".into(),
            entity_type: EntityType::Concept,
            confidence: 0.9,
            start_pos: 2,
            end_pos: 5,
        };

        assert!(m.start_pos < m.end_pos, "start_pos must be < end_pos");
        assert!(
            (0.0..=1.0).contains(&m.confidence),
            "confidence must be in [0.0, 1.0]"
        );
    }

    #[test]
    fn entity_match_confidence_boundary() {
        let cases = vec![
            (0.0, true),
            (0.5, true),
            (1.0, true),
            (-0.1, false),
            (1.1, false),
        ];

        for (confidence, valid) in cases {
            let m = EntityMatch {
                entity: "test".into(),
                entity_type: EntityType::Concept,
                confidence,
                start_pos: 0,
                end_pos: 4,
            };
            assert_eq!(
                (0.0..=1.0).contains(&m.confidence),
                valid,
                "confidence {} should be {}",
                confidence,
                if valid { "valid" } else { "invalid" }
            );
        }
    }
}

