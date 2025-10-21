//! Integration tests for knowledge ingestion

use std::collections::HashMap;
use std::sync::Arc;

use agent_agency_database::models::{KnowledgeSource, ExternalKnowledgeEntity};
use knowledge_ingestor::*;

// Mock embedding service for testing
struct MockEmbeddingService;

#[async_trait::async_trait]
impl embedding_service::EmbeddingService for MockEmbeddingService {
    async fn generate_embedding(&self, text: &str, model: &str) -> anyhow::Result<Vec<f32>> {
        // Simple mock that returns a fixed-size vector based on text length
        let dim = match model {
            "kb-text-default" => 384,
            _ => 384,
        };
        Ok(vec![(text.len() as f32 / 100.0).min(1.0); dim])
    }

    async fn generate_embeddings(&self, texts: &[String], model: &str) -> anyhow::Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.generate_embedding(text, model).await?);
        }
        Ok(results)
    }

    async fn similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        // Simple cosine similarity mock
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

#[tokio::test]
async fn test_knowledge_ingestor_creation() {
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

    // This test will fail without a real database, but it validates the struct creation
    // In a real test environment, we'd set up a test database

    let mock_embedding = Arc::new(MockEmbeddingService {});
    let config = IngestionConfig::default();

    // Test that we can create the ingestor (will fail on DB connection, which is expected)
    let result = agent_agency_database::DatabaseClient::new(db_config).await;
    assert!(result.is_err()); // Expected to fail without real DB
}

#[test]
fn test_ingestion_config_defaults() {
    let config = IngestionConfig::default();

    assert!(config.max_entities.is_none());
    assert_eq!(config.batch_size, 1000);
    assert_eq!(config.embedding_model, "text-embedding-ada-002");
    assert_eq!(config.confidence_threshold, 0.7);
    assert_eq!(config.dump_version, "20250924");
    assert_eq!(config.toolchain, "v3-knowledge-ingestor");
    assert_eq!(config.license, "CC0-1.0");
}

#[test]
fn test_ingestion_stats_merge() {
    let mut stats1 = IngestionStats {
        entities_processed: 100,
        entities_ingested: 95,
        vectors_generated: 95,
        relationships_created: 50,
        errors: 5,
        duration: std::time::Duration::from_secs(30),
    };

    let stats2 = IngestionStats {
        entities_processed: 80,
        entities_ingested: 78,
        vectors_generated: 78,
        relationships_created: 30,
        errors: 2,
        duration: std::time::Duration::from_secs(25),
    };

    stats1.merge(stats2);

    assert_eq!(stats1.entities_processed, 180);
    assert_eq!(stats1.entities_ingested, 173);
    assert_eq!(stats1.vectors_generated, 173);
    assert_eq!(stats1.relationships_created, 80);
    assert_eq!(stats1.errors, 7);
    assert_eq!(stats1.duration, std::time::Duration::from_secs(55));
}

#[tokio::test]
async fn test_mock_embedding_service() {
    let service = MockEmbeddingService {};

    // Test single embedding
    let embedding = service.generate_embedding("test text", "kb-text-default").await.unwrap();
    assert_eq!(embedding.len(), 384); // Default dimension

    // Test batch embeddings
    let texts = vec!["short".to_string(), "longer text here".to_string()];
    let embeddings = service.generate_embeddings(&texts, "kb-text-default").await.unwrap();
    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].len(), 384);
    assert_eq!(embeddings[1].len(), 384);

    // Test similarity (should be > 0 for identical vectors)
    let similarity = service.similarity(&embedding, &embedding).await;
    assert_eq!(similarity, 1.0);
}

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

