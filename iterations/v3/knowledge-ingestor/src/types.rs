//! Common types for knowledge ingestion
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Knowledge source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeSource {
    Wikidata,
    WordNet,
}

impl KnowledgeSource {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            KnowledgeSource::Wikidata => "wikidata",
            KnowledgeSource::WordNet => "wordnet",
        }
    }
}

/// External knowledge entity for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalKnowledgeEntity {
    pub id: Option<Uuid>,
    pub source: KnowledgeSource,
    pub entity_key: String,
    pub canonical_name: String,
    pub lang: Option<String>,
    pub entity_type: Option<String>,
    pub properties: serde_json::Value,
    pub confidence: f64,
    pub dump_version: Option<String>,
    pub toolchain: Option<String>,
    pub license: Option<String>,
}

/// Knowledge relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelationship {
    pub id: Option<Uuid>,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub relationship_type: String,
    pub confidence: f64,
    pub metadata: Option<serde_json::Value>,
}

/// Wikidata lexeme structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikidataLexeme {
    pub id: String,
    pub lemmas: HashMap<String, String>,
    pub language: String,
    pub lexical_category: String,
    pub senses: Vec<WikidataSense>,
    pub forms: Vec<String>,
}

/// Wikidata sense (meaning) of a lexeme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikidataSense {
    pub id: String,
    pub glosses: HashMap<String, String>,
    pub examples: Vec<String>,
}

/// WordNet synset structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordNetSynset {
    pub id: String,
    pub pos: String,
    pub words: Vec<String>,
    pub definition: String,
    pub examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub hypernyms: Vec<String>,
    pub hyponyms: Vec<String>,
}

/// Parsed entity ready for ingestion
#[derive(Debug, Clone)]
pub struct ParsedEntity {
    pub entity: ExternalKnowledgeEntity,
    pub embedding_text: String,
}

/// Cross-reference match between Wikidata and WordNet
#[derive(Debug, Clone)]
pub struct CrossReferenceMatch {
    pub wikidata_id: Uuid,
    pub wordnet_id: Uuid,
    pub confidence: f64,
    pub matching_method: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_source_as_str() {
        assert_eq!(KnowledgeSource::Wikidata.as_str(), "wikidata");
        assert_eq!(KnowledgeSource::WordNet.as_str(), "wordnet");
    }

    #[test]
    fn test_knowledge_source_serialization() {
        let source = KnowledgeSource::Wikidata;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"wikidata\"");

        let deserialized: KnowledgeSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, source);
    }
}

