//! Vector Embeddings with pgvector
//!
//! Provides embedding storage and similarity search for workspace content.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::error::PostgresError;
use crate::pool::ConnectionPool;

/// Embedding repository for vector operations
pub struct EmbeddingRepository {
    pool: ConnectionPool,
}

impl EmbeddingRepository {
    /// Create a new embedding repository
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Store an embedding vector
    pub async fn store_embedding(
        &self,
        source_type: &str,
        source_id: &str,
        content: &str,
        embedding: &[f32],
        model: &str,
    ) -> Result<uuid::Uuid, PostgresError> {
        let content_hash = Self::hash_content(content);

        // Format embedding as PostgreSQL vector literal
        let embedding_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let row = sqlx::query(
            r#"
            INSERT INTO embeddings (source_type, source_id, content_hash, embedding, model)
            VALUES ($1, $2, $3, $4::vector, $5)
            ON CONFLICT (source_type, source_id, content_hash, model) DO UPDATE
                SET embedding = EXCLUDED.embedding
            RETURNING id
            "#,
        )
        .bind(source_type)
        .bind(source_id)
        .bind(&content_hash)
        .bind(&embedding_str)
        .bind(model)
        .fetch_one(self.pool.pool())
        .await?;

        Ok(row.get("id"))
    }

    /// Find similar embeddings by vector similarity (cosine distance)
    pub async fn find_similar(
        &self,
        query_embedding: &[f32],
        limit: i32,
        min_similarity: f32,
    ) -> Result<Vec<SimilarityResult>, PostgresError> {
        let embedding_str = format!(
            "[{}]",
            query_embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        // Use cosine similarity (1 - cosine distance)
        let rows = sqlx::query(
            r#"
            SELECT
                e.id,
                e.source_type,
                e.source_id,
                e.content_hash,
                e.model,
                e.created_at,
                1 - (e.embedding <=> $1::vector) as similarity
            FROM embeddings e
            WHERE 1 - (e.embedding <=> $1::vector) >= $2
            ORDER BY e.embedding <=> $1::vector
            LIMIT $3
            "#,
        )
        .bind(&embedding_str)
        .bind(min_similarity)
        .bind(limit)
        .fetch_all(self.pool.pool())
        .await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(SimilarityResult {
                id: row.get("id"),
                source_type: row.get("source_type"),
                source_id: row.get("source_id"),
                content_hash: row.get("content_hash"),
                model: row.get("model"),
                similarity: row.get("similarity"),
                created_at: row.get("created_at"),
            });
        }

        Ok(results)
    }

    /// Find similar embeddings filtered by source type
    pub async fn find_similar_by_type(
        &self,
        query_embedding: &[f32],
        source_type: &str,
        limit: i32,
        min_similarity: f32,
    ) -> Result<Vec<SimilarityResult>, PostgresError> {
        let embedding_str = format!(
            "[{}]",
            query_embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let rows = sqlx::query(
            r#"
            SELECT
                e.id,
                e.source_type,
                e.source_id,
                e.content_hash,
                e.model,
                e.created_at,
                1 - (e.embedding <=> $1::vector) as similarity
            FROM embeddings e
            WHERE e.source_type = $2
              AND 1 - (e.embedding <=> $1::vector) >= $3
            ORDER BY e.embedding <=> $1::vector
            LIMIT $4
            "#,
        )
        .bind(&embedding_str)
        .bind(source_type)
        .bind(min_similarity)
        .bind(limit)
        .fetch_all(self.pool.pool())
        .await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(SimilarityResult {
                id: row.get("id"),
                source_type: row.get("source_type"),
                source_id: row.get("source_id"),
                content_hash: row.get("content_hash"),
                model: row.get("model"),
                similarity: row.get("similarity"),
                created_at: row.get("created_at"),
            });
        }

        Ok(results)
    }

    /// Delete embeddings for a source
    pub async fn delete_embeddings(
        &self,
        source_type: &str,
        source_id: &str,
    ) -> Result<u64, PostgresError> {
        let result = sqlx::query(
            "DELETE FROM embeddings WHERE source_type = $1 AND source_id = $2",
        )
        .bind(source_type)
        .bind(source_id)
        .execute(self.pool.pool())
        .await?;

        Ok(result.rows_affected())
    }

    /// Get embedding by ID
    pub async fn get_embedding(&self, id: uuid::Uuid) -> Result<Option<EmbeddingRecord>, PostgresError> {
        let row = sqlx::query(
            r#"
            SELECT id, source_type, source_id, content_hash, model, created_at
            FROM embeddings
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool.pool())
        .await?;

        Ok(row.map(|r| EmbeddingRecord {
            id: r.get("id"),
            source_type: r.get("source_type"),
            source_id: r.get("source_id"),
            content_hash: r.get("content_hash"),
            model: r.get("model"),
            created_at: r.get("created_at"),
        }))
    }

    /// Count embeddings
    pub async fn count(&self) -> Result<i64, PostgresError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM embeddings")
            .fetch_one(self.pool.pool())
            .await?;

        Ok(row.get("count"))
    }

    /// Count embeddings by source type
    pub async fn count_by_type(&self, source_type: &str) -> Result<i64, PostgresError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM embeddings WHERE source_type = $1")
            .bind(source_type)
            .fetch_one(self.pool.pool())
            .await?;

        Ok(row.get("count"))
    }

    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Similarity search result
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    /// Embedding ID
    pub id: uuid::Uuid,
    /// Source type (e.g., "file", "chunk", "memory")
    pub source_type: String,
    /// Source identifier
    pub source_id: String,
    /// Content hash
    pub content_hash: String,
    /// Model used for embedding
    pub model: String,
    /// Similarity score (0.0 to 1.0)
    pub similarity: f32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Embedding record without vector (for metadata queries)
#[derive(Debug, Clone)]
pub struct EmbeddingRecord {
    /// Embedding ID
    pub id: uuid::Uuid,
    /// Source type
    pub source_type: String,
    /// Source identifier
    pub source_id: String,
    /// Content hash
    pub content_hash: String,
    /// Model used
    pub model: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let hash1 = EmbeddingRepository::hash_content("test");
        let hash2 = EmbeddingRepository::hash_content("test");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_similarity_result() {
        let result = SimilarityResult {
            id: uuid::Uuid::new_v4(),
            source_type: "file".to_string(),
            source_id: "src/main.rs".to_string(),
            content_hash: "abc123".to_string(),
            model: "text-embedding-3-small".to_string(),
            similarity: 0.95,
            created_at: Utc::now(),
        };

        assert!(result.similarity > 0.9);
        assert_eq!(result.source_type, "file");
    }
}
