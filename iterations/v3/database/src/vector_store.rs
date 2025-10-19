//! Vector store integration for multimodal RAG
//! 
//! Provides database-backed vector storage using pgvector extension
//! with HNSW indices for efficient similarity search.

use anyhow::{Context, Result};
use indexers::database::{PostgresVectorStore, VectorStore};
use indexers::types::{BlockVectorRecord, SearchAuditEntry};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Database-backed vector store for multimodal RAG
pub struct DatabaseVectorStore {
    /// PostgreSQL connection pool
    pool: Arc<PgPool>,
    /// Vector store implementation
    vector_store: PostgresVectorStore,
}

impl DatabaseVectorStore {
    /// Create new database vector store
    pub fn new(pool: Arc<PgPool>) -> Self {
        let vector_store = PostgresVectorStore::new((*pool).clone());
        Self {
            pool,
            vector_store,
        }
    }

    /// Store a block vector in the database
    ///
    /// # Arguments
    /// * `record` - Block vector record to store
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn store_vector(&self, record: BlockVectorRecord) -> Result<()> {
        debug!("Storing vector for block: {}", record.block_id);
        
        let block_id = record.block_id;
        self.vector_store
            .store_vector(record)
            .await
            .context("Failed to store vector in database")?;

        info!("Successfully stored vector for block: {}", block_id);
        Ok(())
    }

    /// Search for similar vectors
    ///
    /// # Arguments
    /// * `query_vector` - Query vector for similarity search
    /// * `model_id` - Embedding model identifier
    /// * `k` - Number of results to return
    /// * `project_scope` - Optional project scope filter
    ///
    /// # Returns
    /// Vector of (block_id, similarity_score) pairs
    pub async fn search_similar(
        &self,
        query_vector: &[f32],
        model_id: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<(Uuid, f32)>> {
        debug!(
            "Searching for similar vectors: model={}, k={}, scope={:?}",
            model_id, k, project_scope
        );

        let results = self
            .vector_store
            .search_similar(query_vector, model_id, k, project_scope)
            .await
            .context("Vector similarity search failed")?;

        info!(
            "Found {} similar vectors for model: {}",
            results.len(),
            model_id
        );

        Ok(results)
    }

    /// Log search operation for audit trail
    ///
    /// # Arguments
    /// * `query` - Search query text
    /// * `results` - Search results
    /// * `features` - Search features used
    pub async fn log_search(
        &self,
        query: &str,
        results: &[Uuid],
        features: &serde_json::Value,
    ) -> Result<()> {
        debug!("Logging search operation: query={}", query);

        // Convert results to SearchResult structs with default values
        let search_results: Vec<indexers::SearchResult> = results
            .iter()
            .enumerate()
            .map(|(i, block_id)| indexers::SearchResult {
                block_id: *block_id,
                score: 1.0 - (i as f32 * 0.1), // Decreasing scores for results
                text_snippet: String::new(),
                modality: "unknown".to_string(),
            })
            .collect();

        // Extract features from JSON if possible, or use empty HashMap
        let feature_map: std::collections::HashMap<String, f32> = features
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| {
                        v.as_f64().map(|f| (k.clone(), f as f32))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let entry = SearchAuditEntry {
            query: query.to_string(),
            results: search_results,
            features: feature_map,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.vector_store
            .log_search(entry)
            .await
            .context("Failed to log search operation")?;

        Ok(())
    }

    /// Get vector store statistics
    ///
    /// # Returns
    /// Statistics about the vector store
    pub async fn get_stats(&self) -> Result<VectorStoreStats> {
        debug!("Retrieving vector store statistics");

        // Count total vectors
        let total_vectors = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM block_vectors"
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count total vectors")?;

        // Count vectors by model
        let model_counts = sqlx::query_as::<_, (String, i64)>(
            "SELECT model_id, COUNT(*) FROM block_vectors GROUP BY model_id"
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to count vectors by model")?;

        // Count vectors by modality
        let modality_counts = sqlx::query_as::<_, (String, i64)>(
            "SELECT modality, COUNT(*) FROM block_vectors GROUP BY modality"
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to count vectors by modality")?;

        let stats = VectorStoreStats {
            total_vectors: total_vectors as u64,
            model_counts: model_counts.into_iter().collect(),
            modality_counts: modality_counts.into_iter().collect(),
        };

        info!("Retrieved vector store statistics: {} total vectors", stats.total_vectors);
        Ok(stats)
    }

    /// Verify pgvector extension is enabled
    ///
    /// # Returns
    /// True if pgvector is enabled, false otherwise
    pub async fn verify_pgvector(&self) -> Result<bool> {
        debug!("Verifying pgvector extension");

        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')"
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to check pgvector extension")?;

        if result {
            info!("pgvector extension is enabled");
        } else {
            error!("pgvector extension is not enabled");
        }

        Ok(result)
    }

    /// Get connection pool reference
    pub fn pool(&self) -> &Arc<PgPool> {
        &self.pool
    }
}

/// Vector store statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorStoreStats {
    /// Total number of vectors stored
    pub total_vectors: u64,
    /// Vector count by model ID
    pub model_counts: Vec<(String, i64)>,
    /// Vector count by modality
    pub modality_counts: Vec<(String, i64)>,
}

impl VectorStoreStats {
    /// Get count for specific model
    pub fn get_model_count(&self, model_id: &str) -> i64 {
        self.model_counts
            .iter()
            .find(|(model, _)| model == model_id)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    }

    /// Get count for specific modality
    pub fn get_modality_count(&self, modality: &str) -> i64 {
        self.modality_counts
            .iter()
            .find(|(r#mod, _)| r#mod == modality)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_vector_store_creation() {
        // This would require a test database setup
        // For now, just test the struct creation
        let stats = VectorStoreStats {
            total_vectors: 100,
            model_counts: vec![("e5-small-v2".to_string(), 50), ("clip-vit-b32".to_string(), 50)],
            modality_counts: vec![("text".to_string(), 60), ("image".to_string(), 40)],
        };

        assert_eq!(stats.total_vectors, 100);
        assert_eq!(stats.get_model_count("e5-small-v2"), 50);
        assert_eq!(stats.get_modality_count("text"), 60);
    }

    #[test]
    fn test_stats_methods() {
        let stats = VectorStoreStats {
            total_vectors: 200,
            model_counts: vec![
                ("e5-small-v2".to_string(), 100),
                ("clip-vit-b32".to_string(), 50),
                ("e5-multilingual-large".to_string(), 50),
            ],
            modality_counts: vec![
                ("text".to_string(), 120),
                ("image".to_string(), 60),
                ("video".to_string(), 20),
            ],
        };

        assert_eq!(stats.get_model_count("e5-small-v2"), 100);
        assert_eq!(stats.get_model_count("nonexistent"), 0);
        assert_eq!(stats.get_modality_count("text"), 120);
        assert_eq!(stats.get_modality_count("audio"), 0);
    }
}
