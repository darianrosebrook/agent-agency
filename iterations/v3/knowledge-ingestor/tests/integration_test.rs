//! Integration tests for knowledge ingestion

use agent_agency_database::models::KnowledgeSource;

#[test]
fn test_knowledge_source_enum() {
    assert_eq!(KnowledgeSource::Wikidata.as_str(), "wikidata");
    assert_eq!(KnowledgeSource::WordNet.as_str(), "wordnet");

    // Test serialization round-trip
    let source = KnowledgeSource::Wikidata;
    let json = serde_json::to_string(&source).unwrap();
    let deserialized: KnowledgeSource = serde_json::from_str(&json).unwrap();
    assert_eq!(source, deserialized);
}

#[test]
fn test_knowledge_source_serialization() {
    let source = KnowledgeSource::Wikidata;
    let json = serde_json::to_string(&source).unwrap();
    assert_eq!(json, "\"wikidata\"");

    let deserialized: KnowledgeSource = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, source);
}

#[test]
fn test_database_config_creation() {
    let db_config = agent_agency_database::DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "test_db".to_string(),
        username: "test".to_string(),
        password: "test".to_string(),
        pool_min: 1,
        pool_max: 5,
        connection_timeout_seconds: 30,
        idle_timeout_seconds: 300,
        max_lifetime_seconds: 3600,
    };

    assert_eq!(db_config.host, "localhost");
    assert_eq!(db_config.port, 5432);
    assert_eq!(db_config.database, "test_db");
    assert_eq!(db_config.username, "test");
    assert_eq!(db_config.password, "test");
    assert_eq!(db_config.pool_min, 1);
    assert_eq!(db_config.pool_max, 5);
}

#[test]
fn test_external_knowledge_entity_structure() {
    use agent_agency_database::models::ExternalKnowledgeEntity;

    let entity = ExternalKnowledgeEntity {
        id: None,
        source: KnowledgeSource::Wikidata,
        entity_key: "Q12345".to_string(),
        canonical_name: "database".to_string(),
        lang: Some("en".to_string()),
        entity_type: Some("lexeme".to_string()),
        properties: serde_json::json!({
            "senses": [{"glosses": {"en": "organized collection of data"}}]
        }),
        confidence: 1.0,
        usage_count: 0,
        usage_decay: Some(1.0),
        last_accessed: None,
        created_at: None,
        dump_version: Some("2025-09-24".to_string()),
        toolchain: Some("v3-knowledge-ingestor".to_string()),
        license: Some("CC0".to_string()),
    };

    assert_eq!(entity.source, KnowledgeSource::Wikidata);
    assert_eq!(entity.entity_key, "Q12345");
    assert_eq!(entity.canonical_name, "database");
    assert_eq!(entity.confidence, 1.0);
    assert_eq!(entity.usage_count, 0);
}

#[test]
fn test_knowledge_relationship_structure() {
    use agent_agency_database::models::KnowledgeRelationship;
    use uuid::Uuid;

    let relationship = KnowledgeRelationship {
        id: None,
        source_entity_id: Uuid::new_v4(),
        target_entity_id: Uuid::new_v4(),
        relationship_type: "synonym".to_string(),
        confidence: 0.95,
        metadata: Some(serde_json::json!({"matching": "exact"})),
    };

    assert_eq!(relationship.relationship_type, "synonym");
    assert_eq!(relationship.confidence, 0.95);
    assert!(relationship.metadata.is_some());
}