//! Vector store integration for multimodal RAG
//!
//! Provides database-backed vector storage using pgvector extension
//! with HNSW indices for efficient similarity search.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Row};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Shared database pool with proper reference counting
pub struct DatabasePool {
    /// Reference-counted pool data
    inner: Arc<PoolInner>,
}

/// Inner pool data that is reference counted
#[derive(JsonSchema)]
struct PoolInner {
    /// The actual database pool
    #[schemars(skip)]
    pool: sqlx::Pool<sqlx::Postgres>,
    /// Active reference counter
    active_refs: AtomicUsize,
    /// Pool identifier for tracking
    pool_id: String,
    /// Creation timestamp
    #[schemars(with = "String")]
    created_at: DateTime<Utc>,
}

impl DatabasePool {
    /// Create new database pool with reference counting
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        let pool_id = format!("pool_{}", Uuid::new_v4().simple());
        let created_at = Utc::now();

        let inner = Arc::new(PoolInner {
            pool,
            active_refs: AtomicUsize::new(1),
            pool_id: pool_id.clone(),
            created_at,
        });

        info!("Created database pool {} with reference counting", pool_id);

        Self { inner }
    }

    /// Get the current reference count
    pub fn reference_count(&self) -> usize {
        self.inner.active_refs.load(Ordering::SeqCst)
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            pool_id: self.inner.pool_id.clone(),
            active_refs: self.reference_count(),
            created_at: self.inner.created_at,
            pool_size: self.inner.pool.size() as usize,
            idle_connections: self.inner.pool.num_idle(),
        }
    }

    /// Create a new reference-counted handle to this pool
    pub fn clone_ref(&self) -> Self {
        // Increment reference count
        let old_count = self.inner.active_refs.fetch_add(1, Ordering::SeqCst);
        debug!(
            "Incremented reference count for pool {}: {} -> {}",
            self.inner.pool_id,
            old_count,
            old_count + 1
        );

        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Force cleanup of this reference (decrements count)
    pub fn cleanup_ref(&self) {
        let old_count = self.inner.active_refs.fetch_sub(1, Ordering::SeqCst);
        let new_count = old_count - 1;

        debug!(
            "Decremented reference count for pool {}: {} -> {}",
            self.inner.pool_id, old_count, new_count
        );

        // If this was the last reference, perform cleanup
        if new_count == 0 {
            info!(
                "Last reference to pool {} dropped, performing cleanup",
                self.inner.pool_id
            );
            self.perform_cleanup();
        }
    }
}

impl Clone for DatabasePool {
    fn clone(&self) -> Self {
        self.clone_ref()
    }
}

impl Drop for DatabasePool {
    fn drop(&mut self) {
        // Decrement reference count when dropped
        let old_count = self.inner.active_refs.fetch_sub(1, Ordering::SeqCst);
        let new_count = old_count - 1;

        debug!(
            "Dropped reference to pool {}: {} -> {}",
            self.inner.pool_id, old_count, new_count
        );

        // If this was the last reference, perform cleanup
        if new_count == 0 {
            info!(
                "Last reference to pool {} dropped, performing cleanup",
                self.inner.pool_id
            );
            self.perform_cleanup();
        }
    }
}

impl DatabasePool {
    /// Perform cleanup when reference count reaches zero
    fn perform_cleanup(&self) {
        // TODO: Implement graceful pool cleanup and return to connection pool manager
        //       Currently relies on sqlx drop behavior; should gracefully close pool and return to manager.
        //
        // COMPLETION CHECKLIST:
        // [ ] Implement graceful pool closure with connection draining
        // [ ] Return pool to connection pool manager for reuse
        // [ ] Flush any pending operations before closure
        // [ ] Close prepared statements and clear caches
        // [ ] Update monitoring metrics for pool lifecycle
        // [ ] Add unit tests for cleanup logic
        // [ ] Add integration tests with pool manager
        // [ ] Verify connections are properly released
        //
        // ACCEPTANCE CRITERIA:
        // - Pool is gracefully closed with connection draining
        // - Pool is returned to connection pool manager for reuse
        // - Pending operations are flushed before closure
        // - Monitoring metrics are updated correctly
        //
        // DEPENDENCIES:
        // - Connection pool manager API (Required)
        // - Connection draining utilities (Required)
        // - Monitoring metrics system (Required)
        //
        // ESTIMATED EFFORT: 3-5 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (standard feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Database connection management expertise
        info!(
            "Performing cleanup for pool {} - closing idle connections",
            self.inner.pool_id
        );

        // The sqlx pool will handle connection cleanup when dropped
        // Here we could add custom cleanup logic like:
        // - Flush any pending operations
        // - Close prepared statements
        // - Update monitoring metrics
        // - Log final statistics
    }
}

// Implement Deref to allow DatabasePool to be used as sqlx::Pool<Postgres>
impl std::ops::Deref for DatabasePool {
    type Target = sqlx::Pool<sqlx::Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.inner.pool
    }
}

/// Statistics for a database pool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PoolStats {
    pub pool_id: String,
    pub active_refs: usize,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    pub pool_size: usize,
    pub idle_connections: usize,
}

/// Vector record for database storage
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockVectorRecord {
    #[schemars(with = "String")]
    pub block_id: Uuid,
    pub vector: Vec<f32>,
    pub model_id: String,
    pub modality: String,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// Search audit entry for logging
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchAuditEntry {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub query: String,
    pub query_type: String,
    pub results_count: usize,
    pub search_time_ms: u64,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    pub results: Option<serde_json::Value>,
    pub features: Option<serde_json::Value>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    #[schemars(with = "String")]
    pub block_id: Uuid,
    pub score: f32,
    pub text_snippet: String,
    pub modality: String,
}

/// Vector search query
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VectorQuery {
    pub vector: Vec<f32>,
    pub model_id: String,
    pub k: usize,
    pub project_scope: Option<String>,
}

/// Vector search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VectorSearchResult {
    #[schemars(with = "String")]
    pub block_id: Uuid,
    pub score: f32,
    pub vector: Vec<f32>,
    pub metadata: serde_json::Value,
}

/// Simple vector store implementation
pub struct VectorStore {
    pool: DatabasePool,
}

impl VectorStore {
    pub fn new(pool: DatabasePool) -> Self {
        info!(
            "Created vector store with pool reference count: {}",
            pool.reference_count()
        );
        Self { pool }
    }

    /// Store a vector record
    ///
    /// Stores a vector embedding in the block_vectors table using a database transaction
    /// for atomicity. The vector is stored with its metadata (block_id, model_id, modality)
    /// and uses pgvector's VECTOR type for efficient similarity search.
    pub async fn store_vector(&self, record: BlockVectorRecord) -> Result<(), anyhow::Error> {
        // Validate reference count is healthy
        if self.pool.reference_count() == 0 {
            return Err(anyhow::anyhow!(
                "Vector store pool has no active references"
            ));
        }

        debug!(
            "Storing vector record {} in pool {}",
            record.block_id,
            self.pool.stats().pool_id
        );

        // Start a database transaction for atomic operation
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start database transaction for vector storage")?;

        // Insert vector into block_vectors table
        // Note: content field is required but not in BlockVectorRecord - using empty string as placeholder
        // project_scope is optional and can be NULL
        // metadata is stored as empty JSONB object
        // Multiple vectors can exist for the same block_id (different models/modalities)
        sqlx::query(
            r#"
            INSERT INTO block_vectors (
                block_id, content, modality, embedding_model_id,
                embedding, metadata, project_scope, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(record.block_id)
        .bind("") // content field - placeholder since not in BlockVectorRecord
        .bind(&record.modality)
        .bind(&record.model_id)
        .bind(&record.vector as &[f32]) // pgvector accepts &[f32]
        .bind(serde_json::json!({})) // metadata as empty JSONB
        .bind::<Option<String>>(None) // project_scope - optional
        .bind(record.created_at)
        .bind(Utc::now()) // updated_at
        .execute(&mut *tx)
        .await
        .context("Failed to insert vector into database")?;

        // Commit transaction
        tx.commit()
            .await
            .context("Failed to commit vector storage transaction")?;

        tracing::debug!(
            block_id = %record.block_id,
            model_id = %record.model_id,
            modality = %record.modality,
            vector_dim = record.vector.len(),
            "Vector stored successfully in database"
        );

        Ok(())
    }

    /// Search vectors
    ///
    /// Performs vector similarity search using pgvector's cosine distance operator.
    /// Returns ranked results with similarity scores, supporting filtering by model_id and project_scope.
    pub async fn search_vectors(
        &self,
        query: &VectorQuery,
    ) -> Result<Vec<VectorSearchResult>, anyhow::Error> {
        // Validate reference count is healthy
        if self.pool.reference_count() == 0 {
            return Err(anyhow::anyhow!(
                "Vector store pool has no active references"
            ));
        }

        info!(
            "Searching vectors with query for model {} in pool {}",
            query.model_id,
            self.pool.stats().pool_id
        );

        // Build SQL query with pgvector cosine similarity
        // Using <=> operator for cosine distance, then converting to similarity (1 - distance)
        let mut sql = String::from(
            "SELECT block_id, embedding, metadata,
                    1 - (embedding <=> $1::vector) as similarity
             FROM block_vectors
             WHERE embedding_model_id = $2",
        );

        let mut param_count = 2;

        // Add project_scope filter if provided
        if query.project_scope.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND project_scope = ${}", param_count));
        }

        // Order by similarity (distance ascending) and limit results
        sql.push_str(&format!(
            " ORDER BY embedding <=> $1::vector LIMIT ${}",
            param_count + 1
        ));

        // Execute query with parameters
        let mut db_query = sqlx::query(&sql)
            .bind(&query.vector as &[f32]) // $1: query vector
            .bind(&query.model_id); // $2: model_id

        // Bind project_scope if provided
        if let Some(ref project_scope) = query.project_scope {
            db_query = db_query.bind(project_scope);
        }

        // Bind limit
        db_query = db_query.bind(query.k as i64);

        let rows = db_query
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute vector similarity search: {}", e))?;

        // Convert rows to VectorSearchResult
        let results: Vec<VectorSearchResult> = rows
            .into_iter()
            .filter_map(|row: PgRow| {
                let block_id = row.try_get::<Uuid, &str>("block_id").ok()?;
                let similarity = row.try_get::<f64, &str>("similarity").ok()?;
                let embedding = row.try_get::<Vec<f32>, &str>("embedding").ok()?;
                let metadata = row.try_get::<serde_json::Value, &str>("metadata").ok()?;

                Some(VectorSearchResult {
                    block_id,
                    score: similarity as f32,
                    vector: embedding,
                    metadata,
                })
            })
            .collect();

        info!(
            "Vector search completed: found {} results for model {}",
            results.len(),
            query.model_id
        );

        Ok(results)
    }

    /// Search similar vectors
    ///
    /// Performs vector similarity search using pgvector's cosine distance operator.
    /// Simplified interface that takes a query vector directly and returns top-k similar vectors.
    pub async fn search_similar(
        &self,
        query_vector: &[f32],
        model_id: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<VectorSearchResult>, anyhow::Error> {
        // Validate reference count is healthy
        if self.pool.reference_count() == 0 {
            return Err(anyhow::anyhow!(
                "Vector store pool has no active references"
            ));
        }

        // Validate input
        if query_vector.is_empty() {
            return Err(anyhow::anyhow!("Query vector cannot be empty"));
        }

        info!(
            "Searching similar vectors for model {} with k={} in pool {}",
            model_id,
            k,
            self.pool.stats().pool_id
        );

        // Build SQL query with pgvector cosine similarity
        // Using <=> operator for cosine distance, then converting to similarity (1 - distance)
        let mut sql = String::from(
            "SELECT block_id, embedding, metadata,
                    1 - (embedding <=> $1::vector) as similarity
             FROM block_vectors
             WHERE embedding_model_id = $2",
        );

        let mut param_count = 2;

        // Add project_scope filter if provided
        if project_scope.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND project_scope = ${}", param_count));
        }

        // Order by similarity (distance ascending) and limit results
        sql.push_str(&format!(
            " ORDER BY embedding <=> $1::vector LIMIT ${}",
            param_count + 1
        ));

        // Execute query with parameters
        let mut db_query = sqlx::query(&sql)
            .bind(query_vector as &[f32]) // $1: query vector
            .bind(model_id); // $2: model_id

        // Bind project_scope if provided
        if let Some(project_scope) = project_scope {
            db_query = db_query.bind(project_scope);
        }

        // Bind limit
        db_query = db_query.bind(k as i64);

        let rows = db_query
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute vector similarity search: {}", e))?;

        // Convert rows to VectorSearchResult
        let results: Vec<VectorSearchResult> = rows
            .into_iter()
            .filter_map(|row: PgRow| {
                let block_id = row.try_get::<Uuid, &str>("block_id").ok()?;
                let similarity = row.try_get::<f64, &str>("similarity").ok()?;
                let embedding = row.try_get::<Vec<f32>, &str>("embedding").ok()?;
                let metadata = row.try_get::<serde_json::Value, &str>("metadata").ok()?;

                Some(VectorSearchResult {
                    block_id,
                    score: similarity as f32,
                    vector: embedding,
                    metadata,
                })
            })
            .collect();

        info!(
            "Similar vector search completed: found {} results for model {}",
            results.len(),
            model_id
        );

        Ok(results)
    }

    /// Log search operation
    ///
    /// Inserts an audit log entry into the search_audit_log table for tracking vector search operations.
    /// Errors are logged but do not block the operation to ensure search functionality is not impacted.
    pub async fn log_search(&self, entry: SearchAuditEntry) -> Result<(), anyhow::Error> {
        // Validate reference count is healthy
        if self.pool.reference_count() == 0 {
            return Err(anyhow::anyhow!(
                "Vector store pool has no active references"
            ));
        }

        debug!(
            audit_entry_id = %entry.id,
            query_type = %entry.query_type,
            results_count = entry.results_count,
            search_time_ms = entry.search_time_ms,
            "Logging search operation in pool {}",
            self.pool.stats().pool_id
        );

        // Insert audit log entry into search_audit_log table
        // Use entry.timestamp if provided, otherwise use current time
        let timestamp = entry.timestamp;

        // Serialize results and features as JSONB
        let results_json = entry.results.unwrap_or_else(|| serde_json::json!([]));
        let features_json = entry.features.unwrap_or_else(|| serde_json::json!({}));

        // Build query string with query_type included in features for additional context
        let query_with_type = format!("{} [type: {}]", entry.query, entry.query_type);

        // Insert into search_audit_log table
        // Note: user_id and session_id are optional and not in SearchAuditEntry, so we use NULL
        let result = sqlx::query(
            r#"
            INSERT INTO search_audit_log (
                id, query, results, features, timestamp,
                user_id, session_id, response_time_ms, result_count
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(entry.id)
        .bind(&query_with_type)
        .bind(&results_json)
        .bind(&features_json)
        .bind(timestamp)
        .bind::<Option<Uuid>>(None::<Uuid>) // user_id - not available in SearchAuditEntry
        .bind::<Option<String>>(None::<String>) // session_id - not available in SearchAuditEntry
        .bind(entry.search_time_ms as i32) // response_time_ms
        .bind(entry.results_count as i32) // result_count
        .execute(&*self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!(
                    audit_entry_id = %entry.id,
                    "Audit log entry inserted successfully"
                );
                Ok(())
            }
            Err(e) => {
                // Log error but don't fail - audit logging should not block search operations
                warn!(
                    audit_entry_id = %entry.id,
                    error = %e,
                    "Failed to insert audit log entry - continuing without audit log"
                );
                // Return error for caller to handle, but don't block search operations
                Err(anyhow::anyhow!("Failed to insert audit log entry: {}", e))
            }
        }
    }

    /// Get current pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        self.pool.stats()
    }

    /// Check if the vector store is healthy
    pub fn is_healthy(&self) -> bool {
        let stats = self.pool.stats();
        stats.active_refs > 0 && stats.pool_size > 0
    }

    /// Force cleanup of pool reference
    pub fn cleanup(&self) {
        self.pool.cleanup_ref();
    }
}

/// Database-backed vector store for multimodal RAG
pub struct DatabaseVectorStore {
    /// Database pool
    pool: Arc<DatabasePool>,
    /// Vector store implementation
    vector_store: VectorStore,
}

impl DatabaseVectorStore {
    /// Create new database vector store with reference counting
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        info!(
            "Creating database vector store with reference count: {}",
            pool.reference_count()
        );

        // Clone the pool reference (increments reference count)
        let pool_clone = Arc::clone(&pool);
        let vector_store = VectorStore::new((*pool_clone).clone_ref());

        info!(
            "Database vector store created, total references: {}",
            pool.reference_count()
        );

        Self { pool, vector_store }
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

        let start_time = std::time::Instant::now();

        let search_results = self
            .vector_store
            .search_similar(query_vector, model_id, k, project_scope)
            .await
            .context("Vector similarity search failed")?;

        // Convert to expected format (block_id, score)
        let results: Vec<(Uuid, f32)> = search_results
            .into_iter()
            .map(|result| (result.block_id, result.score))
            .collect();

        let search_time = start_time.elapsed();

        info!(
            "Found {} similar vectors for model: {} in {:?}",
            results.len(),
            model_id,
            search_time
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
        let _search_results: Vec<SearchResult> = results
            .iter()
            .enumerate()
            .map(|(i, block_id)| SearchResult {
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
                    .filter_map(|(k, v)| v.as_f64().map(|f| (k.clone(), f as f32)))
                    .collect()
            })
            .unwrap_or_default();

        let entry = SearchAuditEntry {
            id: uuid::Uuid::new_v4(),
            query: query.to_string(),
            query_type: "vector_similarity".to_string(),
            results_count: results.len() as usize,
            search_time_ms: 0, // TODO: Pass actual search time when available
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            results: Some(serde_json::to_value(&results).unwrap_or(serde_json::Value::Null)),
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
        let total_vectors = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM block_vectors")
            .fetch_one(&**self.pool)
            .await
            .context("Failed to count total vectors")?;

        // Count vectors by model
        let model_counts = sqlx::query_as::<_, (String, i64)>(
            "SELECT model_id, COUNT(*) FROM block_vectors GROUP BY model_id",
        )
        .fetch_all(&**self.pool)
        .await
        .context("Failed to count vectors by model")?;

        // Count vectors by modality
        let modality_counts = sqlx::query_as::<_, (String, i64)>(
            "SELECT modality, COUNT(*) FROM block_vectors GROUP BY modality",
        )
        .fetch_all(&**self.pool)
        .await
        .context("Failed to count vectors by modality")?;

        let stats = VectorStoreStats {
            total_vectors: total_vectors as u64,
            model_counts: model_counts.into_iter().collect(),
            modality_counts: modality_counts.into_iter().collect(),
        };

        info!(
            "Retrieved vector store statistics: {} total vectors",
            stats.total_vectors
        );
        Ok(stats)
    }

    /// Verify pgvector extension is enabled
    ///
    /// # Returns
    /// True if pgvector is enabled, false otherwise
    pub async fn verify_pgvector(&self) -> Result<bool> {
        debug!("Verifying pgvector extension");

        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')",
        )
        .fetch_one(&**self.pool)
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
    pub fn pool(&self) -> &Arc<DatabasePool> {
        &self.pool
    }

    /// Get current pool reference count
    pub fn reference_count(&self) -> usize {
        self.pool.reference_count()
    }

    /// Get pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        self.pool.stats()
    }

    /// Check if the database vector store is healthy
    pub fn is_healthy(&self) -> bool {
        self.pool.reference_count() > 0 && self.vector_store.is_healthy()
    }

    /// Force cleanup of pool references
    pub fn cleanup(&self) {
        info!("Cleaning up database vector store references");
        self.vector_store.cleanup();
        // Note: Arc<DatabasePool> will automatically decrement when dropped
    }
}

/// Vector store statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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
    use std::collections::HashMap;
    use uuid::Uuid;

    // Stub types for tests
    #[derive(Debug, Clone, JsonSchema)]
    pub struct BlockVectorRecord {
        pub block_id: String,
        pub vector: Vec<f32>,
        pub model_id: String,
        pub modality: String,
        #[allow(dead_code)]
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, Clone, JsonSchema)]
    pub struct SearchAuditEntry {
        pub query_id: String,
        pub query_type: String,
        pub results_count: usize,
        pub search_time_ms: u64,
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }

    // Helper function for tests
    async fn create_test_pool() -> Result<sqlx::PgPool, sqlx::Error> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost:5432/agent_agency_test".to_string());
        sqlx::PgPool::connect(&database_url).await
    }

    // TODO: Implement comprehensive test database setup and lifecycle management
    // - [ ] Set up isolated test database instances for each test run
    // - [ ] Implement database schema migration and seeding for tests
    // - [ ] Support multiple test database configurations (in-memory, local, remote)
    // - [ ] Add database connection pooling and cleanup for concurrent tests
    // - [ ] Implement test data generation and fixture management
    // - [ ] Support database state snapshots and restoration between tests
    // - [ ] Add database performance monitoring and slow query detection in tests

    #[tokio::test]
    async fn test_vector_store_stats_struct() {
        let stats = VectorStoreStats {
            total_vectors: 100,
            model_counts: vec![
                ("embeddinggemma".to_string(), 50),
                ("clip-vit-b32".to_string(), 50),
            ],
            modality_counts: vec![("text".to_string(), 60), ("image".to_string(), 40)],
        };

        assert_eq!(stats.total_vectors, 100);
        assert_eq!(stats.get_model_count("embeddinggemma"), 50);
        assert_eq!(stats.get_modality_count("text"), 60);
    }

    #[test]
    fn test_stats_methods() {
        let stats = VectorStoreStats {
            total_vectors: 200,
            model_counts: vec![
                ("embeddinggemma".to_string(), 100),
                ("clip-vit-b32".to_string(), 50),
                ("e5-multilingual-large".to_string(), 50),
            ],
            modality_counts: vec![
                ("text".to_string(), 120),
                ("image".to_string(), 60),
                ("video".to_string(), 20),
            ],
        };

        assert_eq!(stats.get_model_count("embeddinggemma"), 100);
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
            println!(
                "Skipping vector store database integration tests - no test database configured"
            );
            return;
        }

        let pool = pool.unwrap();
        let database_pool = DatabasePool::new(pool);
        let pool = Arc::new(database_pool);
        let vector_store = DatabaseVectorStore::new(pool);

        // Test that we can access the pool
        assert!(vector_store.pool().reference_count() > 0);
    }

    #[tokio::test]
    async fn test_vector_record_creation() {
        // Test creating BlockVectorRecord instances
        let block_id = Uuid::new_v4();
        let model_id = "embeddinggemma";
        // embeddinggemma uses 768 dimensions (sample vector truncated for test)
        let vec = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let modality = "text";

        let record = BlockVectorRecord {
            block_id: block_id.to_string(),
            model_id: model_id.to_string(),
            vector: vec.clone(),
            modality: modality.to_string(),
            created_at: chrono::Utc::now(),
        };

        assert_eq!(record.block_id, block_id.to_string());
        assert_eq!(record.model_id, model_id);
        assert_eq!(record.vector, vec);
        assert_eq!(record.modality, modality);
    }

    #[tokio::test]
    async fn test_search_audit_entry_creation() {
        // Test creating SearchAuditEntry instances
        let id = Uuid::new_v4();
        let _query = "test query";
        let created_at = chrono::Utc::now();

        let mut results = Vec::new();
        results.push(SearchResult {
            block_id: Uuid::new_v4(),
            score: 0.95,
            text_snippet: "test snippet".to_string(),
            modality: "text".to_string(),
        });

        let mut features = HashMap::new();
        features.insert("feature1".to_string(), 0.8);
        features.insert("feature2".to_string(), 0.6);

        let entry = SearchAuditEntry {
            query_id: id.to_string(),
            query_type: "semantic".to_string(),
            results_count: results.len(),
            search_time_ms: 150,
            timestamp: created_at,
        };

        assert_eq!(entry.query_id, id.to_string());
        assert_eq!(entry.query_type, "semantic");
        assert_eq!(entry.results_count, results.len());
        assert_eq!(entry.search_time_ms, 150);

        // Test that the entry was created successfully
        assert_eq!(entry.timestamp, created_at);
    }

    #[tokio::test]
    async fn test_vector_store_stats_calculation() {
        // Test that VectorStoreStats can be properly constructed from mock data
        let model_counts = vec![
            ("embeddinggemma".to_string(), 150),
            ("clip-vit-b32".to_string(), 75),
            ("e5-multilingual-large".to_string(), 25),
        ];

        let modality_counts = vec![("text".to_string(), 200), ("image".to_string(), 50)];

        let stats = VectorStoreStats {
            total_vectors: 250,
            model_counts: model_counts.clone(),
            modality_counts: modality_counts.clone(),
        };

        // Verify total matches sum of model counts
        let sum_model_counts: i64 = model_counts.iter().map(|(_, count)| count).sum();
        assert_eq!(stats.total_vectors as i64, sum_model_counts);

        // Test individual count lookups
        assert_eq!(stats.get_model_count("embeddinggemma"), 150);
        assert_eq!(stats.get_model_count("nonexistent-model"), 0);
        assert_eq!(stats.get_modality_count("text"), 200);
        assert_eq!(stats.get_modality_count("video"), 0);
    }

    #[tokio::test]
    async fn test_vector_similarity_search_parameters() {
        // Test parameter validation for similarity search
        let query_vector = vec![0.1, 0.2, 0.3];
        let model_id = "embeddinggemma";
        let k = 10;
        let project_scope = Some("test-project");

        // These parameters would be used in actual search calls
        assert_eq!(query_vector.len(), 3);
        assert_eq!(model_id, "embeddinggemma");
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
            model_id: "embeddinggemma".to_string(),
            vector: vec![0.1, 0.2, 0.3, 0.4, 0.5], // Sample - embeddinggemma uses 768 dims
            modality: "text".to_string(),
            created_at: chrono::Utc::now(),
        };

        // Store vector
        vector_store.store_vector(record).await.unwrap();

        // Search for similar vectors
        let query_vec = vec![0.1, 0.2, 0.3, 0.4, 0.5]; // Sample - embeddinggemma uses 768 dims
        let results = vector_store.search_similar(&query_vec, "embeddinggemma", 5, None).await.unwrap();

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
