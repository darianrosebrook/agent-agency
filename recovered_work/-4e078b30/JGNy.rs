//! Knowledge Ingestor for Wikidata and WordNet
//!
//! This crate provides functionality to parse, normalize, and ingest external knowledge
//! sources (Wikidata lexemes and WordNet synsets) into the v3 database with vector embeddings.
//!
//! @author @darianrosebrook

pub mod core_vocabulary;
pub mod cross_reference;
pub mod on_demand;
pub mod types;
pub mod wikidata;
pub mod wordnet;

// Re-export database types to avoid conflicts
pub use agent_agency_database::models::*;

use agent_agency_database as database;
use anyhow::Result;
use database::DatabaseClient;
#[cfg(feature = "embeddings")]
use embedding_service::EmbeddingService;
use std::sync::Arc;

/// Configuration for knowledge ingestion
#[derive(Debug, Clone)]
pub struct IngestionConfig {
    /// Maximum number of entities to ingest
    pub limit: Option<usize>,
    /// Preferred languages (BCP47 tags) in priority order
    pub languages: Vec<String>,
    /// Embedding model ID to use
    pub model_id: String,
    /// Minimum confidence threshold for ingestion
    pub min_confidence: f64,
    /// Batch size for database inserts
    pub batch_size: usize,
    /// Enable parallel processing
    pub parallel: bool,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            limit: Some(10_000),
            languages: vec!["en".to_string()],
            model_id: "kb-text-default".to_string(),
            min_confidence: 0.5,
            batch_size: 100,
            parallel: true,
        }
    }
}

/// Main knowledge ingestor orchestrator
pub struct KnowledgeIngestor {
    db_client: Arc<DatabaseClient>,
    embedding_service: Arc<dyn EmbeddingService>,
    config: IngestionConfig,
}

impl KnowledgeIngestor {
    /// Create a new knowledge ingestor
    pub fn new(
        db_client: Arc<DatabaseClient>,
        embedding_service: Arc<dyn EmbeddingService>,
        config: IngestionConfig,
    ) -> Self {
        Self {
            db_client,
            embedding_service,
            config,
        }
    }

    /// Get the database client
    pub fn db_client(&self) -> &DatabaseClient {
        &self.db_client
    }

    /// Get the embedding service
    pub fn embedding_service(&self) -> &dyn EmbeddingService {
        self.embedding_service.as_ref()
    }

    /// Get the configuration
    pub fn config(&self) -> &IngestionConfig {
        &self.config
    }
}

/// Statistics from ingestion process
#[derive(Debug, Clone, Default)]
pub struct IngestionStats {
    /// Number of entities processed
    pub entities_processed: usize,
    /// Number of entities inserted
    pub entities_inserted: usize,
    /// Number of entities skipped (duplicates or low confidence)
    pub entities_skipped: usize,
    /// Number of vectors generated
    pub vectors_generated: usize,
    /// Number of relationships created
    pub relationships_created: usize,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Errors encountered
    pub errors: Vec<String>,
}

impl IngestionStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another stats object into this one
    pub fn merge(&mut self, other: IngestionStats) {
        self.entities_processed += other.entities_processed;
        self.entities_inserted += other.entities_inserted;
        self.entities_skipped += other.entities_skipped;
        self.vectors_generated += other.vectors_generated;
        self.relationships_created += other.relationships_created;
        self.processing_time_ms += other.processing_time_ms;
        self.errors.extend(other.errors);
    }

    /// Print a summary of the stats
    pub fn print_summary(&self) {
        println!("\n=== Ingestion Statistics ===");
        println!("Entities processed: {}", self.entities_processed);
        println!("Entities inserted: {}", self.entities_inserted);
        println!("Entities skipped: {}", self.entities_skipped);
        println!("Vectors generated: {}", self.vectors_generated);
        println!("Relationships created: {}", self.relationships_created);
        println!("Processing time: {}ms", self.processing_time_ms);
        if !self.errors.is_empty() {
            println!("Errors encountered: {}", self.errors.len());
            for (i, error) in self.errors.iter().take(5).enumerate() {
                println!("  {}. {}", i + 1, error);
            }
            if self.errors.len() > 5 {
                println!("  ... and {} more", self.errors.len() - 5);
            }
        }
        println!("============================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

