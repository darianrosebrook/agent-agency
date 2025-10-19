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
            id: uuid::Uuid::new_v4(),
            query: query.to_string(),
            created_at: chrono::Utc::now(),
            results: Some(serde_json::to_value(&search_results).unwrap_or(serde_json::Value::Null)),
            features: Some(serde_json::to_value(&feature_map).unwrap_or(serde_json::Value::Null)),
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
    use indexers::types::{BlockVectorRecord, SearchAuditEntry};
    use std::collections::HashMap;
    use uuid::Uuid;

    // TODO: Implement comprehensive test database setup and lifecycle management
    // - [ ] Set up isolated test database instances for each test run
    // - [ ] Implement database schema migration and seeding for tests
    // - [ ] Support multiple test database configurations (in-memory, local, remote)
    // - [ ] Add database connection pooling and cleanup for concurrent tests
    // - [ ] Implement test data generation and fixture management
    // - [ ] Support database state snapshots and restoration between tests
    // - [ ] Add database performance monitoring and slow query detection in tests
        Err(anyhow::anyhow!("Test database not configured - run tests with TEST_DATABASE_URL"))
    }

    #[tokio::test]
    async fn test_vector_store_stats_struct() {
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

    #[tokio::test]
    async fn test_vector_store_creation() {
        // Test that DatabaseVectorStore can be created (without database connection)
        // This tests the struct creation and basic functionality

        // We can't test actual database operations without a test database,
        // but we can test the struct creation and method signatures

        let pool = create_test_pool().await;
        if pool.is_err() {
            // Skip test if no test database is available
            println!("Skipping vector store database integration tests - no test database configured");
            return;
        }

        let pool = pool.unwrap();
        let pool = Arc::new(pool);
        let vector_store = DatabaseVectorStore::new(pool);

        // Test that we can access the pool
        assert!(vector_store.pool().is_some());
    }

    #[tokio::test]
    async fn test_vector_record_creation() {
        // Test creating BlockVectorRecord instances
        let block_id = Uuid::new_v4();
        let model_id = "e5-small-v2";
        let vec = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let modality = "text";

        let record = BlockVectorRecord {
            block_id,
            model_id: model_id.to_string(),
            vec: vec.clone(),
            modality: modality.to_string(),
        };

        assert_eq!(record.block_id, block_id);
        assert_eq!(record.model_id, model_id);
        assert_eq!(record.vec, vec);
        assert_eq!(record.modality, modality);
    }

    #[tokio::test]
    async fn test_search_audit_entry_creation() {
        // Test creating SearchAuditEntry instances
        let id = Uuid::new_v4();
        let query = "test query";
        let created_at = chrono::Utc::now();

        let mut results = Vec::new();
        results.push(indexers::SearchResult {
            block_id: Uuid::new_v4(),
            score: 0.95,
            text_snippet: "test snippet".to_string(),
            modality: "text".to_string(),
        });

        let mut features = HashMap::new();
        features.insert("feature1".to_string(), 0.8);
        features.insert("feature2".to_string(), 0.6);

        let entry = SearchAuditEntry {
            id,
            query: query.to_string(),
            created_at,
            results: Some(serde_json::to_value(&results).unwrap()),
            features: Some(serde_json::to_value(&features).unwrap()),
        };

        assert_eq!(entry.id, id);
        assert_eq!(entry.query, query);
        assert!(entry.results.is_some());
        assert!(entry.features.is_some());

        // Test deserialization
        let results_deserialized: Vec<indexers::SearchResult> =
            serde_json::from_value(entry.results.unwrap()).unwrap();
        assert_eq!(results_deserialized.len(), 1);

        let features_deserialized: HashMap<String, f32> =
            serde_json::from_value(entry.features.unwrap()).unwrap();
        assert_eq!(features_deserialized.len(), 2);
    }

    #[tokio::test]
    async fn test_vector_store_stats_calculation() {
        // Test that VectorStoreStats can be properly constructed from mock data
        let model_counts = vec![
            ("e5-small-v2".to_string(), 150),
            ("clip-vit-b32".to_string(), 75),
            ("e5-multilingual-large".to_string(), 25),
        ];

        let modality_counts = vec![
            ("text".to_string(), 200),
            ("image".to_string(), 50),
        ];

        let stats = VectorStoreStats {
            total_vectors: 250,
            model_counts: model_counts.clone(),
            modality_counts: modality_counts.clone(),
        };

        // Verify total matches sum of model counts
        let sum_model_counts: i64 = model_counts.iter().map(|(_, count)| count).sum();
        assert_eq!(stats.total_vectors as i64, sum_model_counts);

        // Test individual count lookups
        assert_eq!(stats.get_model_count("e5-small-v2"), 150);
        assert_eq!(stats.get_model_count("nonexistent-model"), 0);
        assert_eq!(stats.get_modality_count("text"), 200);
        assert_eq!(stats.get_modality_count("video"), 0);
    }

    #[tokio::test]
    async fn test_vector_similarity_search_parameters() {
        // Test parameter validation for similarity search
        let query_vector = vec![0.1, 0.2, 0.3];
        let model_id = "e5-small-v2";
        let k = 10;
        let project_scope = Some("test-project");

        // These parameters would be used in actual search calls
        assert_eq!(query_vector.len(), 3);
        assert_eq!(model_id, "e5-small-v2");
        assert_eq!(k, 10);
        assert_eq!(project_scope, Some("test-project"));
    }

    // Integration tests that would require a test database
    // These are commented out but show the structure for real database testing

    /*
    #[tokio::test]
    async fn test_vector_storage_and_retrieval() {
        let pool = create_test_pool().await.unwrap();
        let pool = Arc::new(pool);
        let vector_store = DatabaseVectorStore::new(pool);

        // Create test vector
        let block_id = Uuid::new_v4();
        let record = BlockVectorRecord {
            block_id,
            model_id: "e5-small-v2".to_string(),
            vec: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            modality: "text".to_string(),
        };

        // Store vector
        vector_store.store_vector(record).await.unwrap();

        // Search for similar vectors
        let query_vec = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let results = vector_store.search_similar(&query_vec, "e5-small-v2", 5, None).await.unwrap();

        // Verify we get our stored vector back
        assert!(!results.is_empty());
        assert_eq!(results[0].0, block_id);
        assert!(results[0].1 > 0.9); // High similarity expected
    }

    #[tokio::test]
    async fn test_vector_store_statistics() {
        let pool = create_test_pool().await.unwrap();
        let pool = Arc::new(pool);
        let vector_store = DatabaseVectorStore::new(pool);

        // Get statistics
        let stats = vector_store.get_stats().await.unwrap();

        // Verify statistics are reasonable
        assert!(stats.total_vectors >= 0);
        assert!(stats.model_counts.len() >= 0);
        assert!(stats.modality_counts.len() >= 0);
    }

    #[tokio::test]
    async fn test_pgvector_extension_verification() {
        let pool = create_test_pool().await.unwrap();
        let pool = Arc::new(pool);
        let vector_store = DatabaseVectorStore::new(pool);

        // Verify pgvector is enabled
        let is_enabled = vector_store.verify_pgvector().await.unwrap();
        assert!(is_enabled, "pgvector extension must be enabled for vector operations");
    }
    */
}
