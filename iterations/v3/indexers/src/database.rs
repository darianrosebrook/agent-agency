//! @darianrosebrook
//! Database persistence layer for vector storage and audit logs

use crate::types::{BlockVectorRecord, SearchAuditEntry};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::debug;
use uuid::Uuid;

/// Database connection pool wrapper
pub struct DatabasePool {
    pool: Pool<Postgres>,
}

impl DatabasePool {
    /// Create a new database pool from connection URL
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| anyhow!("Failed to create database pool: {}", e))?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;

        debug!(
            "Database pool initialized with max {} connections",
            max_connections
        );

        Ok(Self { pool })
    }

    /// Get number of active connections
    pub fn num_idle(&self) -> usize {
        self.pool.num_idle() as usize
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.pool.size() as usize
    }
}

/// Vector store trait for persistence
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store a block vector
    async fn store_vector(&self, record: BlockVectorRecord) -> Result<()>;

    /// Retrieve vectors for a block
    async fn get_block_vectors(&self, block_id: Uuid, model_id: &str) -> Result<Option<Vec<f32>>>;

    /// Search for similar vectors using approximate nearest neighbors
    async fn search_similar(
        &self,
        query_vector: &[f32],
        model_id: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<(Uuid, f32)>>;

    /// Store search audit log
    async fn log_search(&self, entry: SearchAuditEntry) -> Result<()>;

    /// Get search audit logs
    async fn get_search_logs(&self, limit: usize) -> Result<Vec<SearchAuditEntry>>;
}

/// PostgreSQL implementation of VectorStore
pub struct PostgresVectorStore {
    pool: Pool<Postgres>,
}

impl PostgresVectorStore {
    /// Create new PostgreSQL vector store
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// From existing DatabasePool
    pub fn from_pool(db_pool: &DatabasePool) -> Self {
        Self {
            pool: db_pool.pool.clone(),
        }
    }
}

#[async_trait]
impl VectorStore for PostgresVectorStore {
    async fn store_vector(&self, record: BlockVectorRecord) -> Result<()> {
        // Insert or update block vector with pgvector support
        sqlx::query(
            r#"
            INSERT INTO block_vectors (block_id, model_id, modality, vec)
            VALUES ($1, $2, $3, $4::vector)
            ON CONFLICT (block_id, model_id) DO UPDATE
            SET vec = EXCLUDED.vec, updated_at = NOW()
            "#
        )
        .bind(record.block_id)
        .bind(&record.model_id)
        .bind(&record.modality)
        .bind(&record.vector) // Vec<f32> → pgvector
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to store vector: {}", e))?;

        Ok(())
    }

    async fn get_block_vectors(&self, block_id: Uuid, model_id: &str) -> Result<Option<Vec<f32>>> {
        // Query vectors from database for a specific block and model
        let vector = sqlx::query_scalar::<_, Vec<f32>>(
            "SELECT vec FROM block_vectors WHERE block_id = $1 AND model_id = $2"
        )
        .bind(block_id)
        .bind(model_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch block vectors: {}", e))?;

        debug!(
            "Retrieved vectors for block {} with model {}",
            block_id, model_id
        );

        Ok(vector)
    }

    async fn search_similar(
        &self,
        query_vector: &[f32],
        model_id: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<(Uuid, f32)>> {
        // Determine similarity operator based on model type
        let operator = match self.get_model_metric(model_id).await?.as_str() {
            "cosine" => "<=>", // Cosine similarity (normalized)
            "ip" => "<#>",     // Inner product
            "l2" => "<->",     // L2 distance
            _ => "<=>",        // Default to cosine
        };

        // Build dynamic SQL based on operator
        let query_sql = format!(
            r#"
            SELECT bv.block_id, (bv.vec {} $1::vector) AS similarity_score
            FROM block_vectors bv
            JOIN blocks b ON bv.block_id = b.id
            JOIN segments s ON b.segment_id = s.id
            WHERE bv.model_id = $2
              AND bv.modality = $3
              AND (s.project_scope IS NULL OR s.project_scope = $4)
            ORDER BY similarity_score {}
            LIMIT $5
            "#,
            operator,
            if operator == "<->" { "ASC" } else { "DESC" } // L2 distance ascending, others descending
        );

        let results = sqlx::query_as::<_, (Uuid, f32)>(&query_sql)
            .bind(query_vector)
            .bind(model_id)
            .bind("text") // Default modality for text search
            .bind(project_scope)
            .bind(k as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Vector search failed: {}", e))?;

        Ok(results)
    }

    async fn log_search(&self, entry: SearchAuditEntry) -> Result<()> {
        // Insert search audit log with results and features
        sqlx::query(
            r#"
            INSERT INTO search_logs (query, results, features, created_at)
            VALUES ($1, $2::jsonb, $3::jsonb, NOW())
            "#,
        )
        .bind(&entry.query)
        .bind(serde_json::to_value(&entry.results)?)
        .bind(serde_json::to_value(&entry.features)?)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to log search: {}", e))?;

        Ok(())
    }

    async fn get_search_logs(&self, limit: usize) -> Result<Vec<SearchAuditEntry>> {
        // Query recent search logs from audit table
        let logs = sqlx::query_as::<_, SearchAuditEntry>(
            "SELECT id, query, created_at, results, features
             FROM search_logs 
             ORDER BY created_at DESC 
             LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch search logs: {}", e))?;

        debug!("Retrieved {} search logs", logs.len());

        Ok(logs)
    }
}

impl PostgresVectorStore {
    // Helper method to get metric type for a model
    async fn get_model_metric(&self, model_id: &str) -> Result<String> {
        let metric = sqlx::query_scalar::<_, String>(
            "SELECT metric FROM embedding_models WHERE model_id = $1",
        )
        .bind(model_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch model metric: {}", e))?
        .unwrap_or_else(|| "cosine".to_string());

        Ok(metric)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_vector_record_creation() {
        let record = BlockVectorRecord {
            block_id: Uuid::new_v4(),
            model_id: "e5-small-v2".to_string(),
            modality: "text".to_string(),
            vector: vec![0.1, 0.2, 0.3],
        };

        assert_eq!(record.modality, "text");
        assert_eq!(record.vector.len(), 3);
    }
}
