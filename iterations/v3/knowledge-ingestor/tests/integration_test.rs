//! Integration tests for knowledge ingestion
//!
//! @author @darianrosebrook

use knowledge_ingestor::*;

#[test]
fn test_core_vocabulary_lookup() {
    assert!(core_vocabulary::is_core_vocabulary("database"));
    assert!(core_vocabulary::is_core_vocabulary("API"));
    assert!(!core_vocabulary::is_core_vocabulary("nonexistent_term"));
}

#[test]
fn test_knowledge_source_serialization() {
    let source = types::KnowledgeSource::Wikidata;
    let json = serde_json::to_string(&source).unwrap();
    assert_eq!(json, "\"wikidata\"");
    
    let deserialized: types::KnowledgeSource = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, source);
}

#[test]
fn test_ingestion_stats_merge() {
    let mut stats1 = IngestionStats {
        entities_processed: 100,
        entities_inserted: 90,
        entities_skipped: 10,
        vectors_generated: 90,
        relationships_created: 50,
        processing_time_ms: 1000,
        errors: vec!["error1".to_string()],
    };
    
    let stats2 = IngestionStats {
        entities_processed: 50,
        entities_inserted: 45,
        entities_skipped: 5,
        vectors_generated: 45,
        relationships_created: 25,
        processing_time_ms: 500,
        errors: vec!["error2".to_string()],
    };
    
    stats1.merge(stats2);
    
    assert_eq!(stats1.entities_processed, 150);
    assert_eq!(stats1.entities_inserted, 135);
    assert_eq!(stats1.entities_skipped, 15);
    assert_eq!(stats1.vectors_generated, 135);
    assert_eq!(stats1.relationships_created, 75);
    assert_eq!(stats1.processing_time_ms, 1500);
    assert_eq!(stats1.errors.len(), 2);
}

#[test]
fn test_domain_vocabulary() {
    let tech_vocab = core_vocabulary::get_domain_vocabulary("software");
    assert!(!tech_vocab.is_empty());
    assert!(tech_vocab.contains(&"api"));
    assert!(tech_vocab.contains(&"database"));
    
    let general_vocab = core_vocabulary::get_domain_vocabulary("general");
    assert!(general_vocab.len() > tech_vocab.len());
}

#[test]
fn test_priority_scores() {
    let api_score = core_vocabulary::get_priority_score("api");
    let person_score = core_vocabulary::get_priority_score("person");
    
    assert!(api_score > person_score);
    assert_eq!(api_score, 1.0);
    assert_eq!(person_score, 0.5);
}

