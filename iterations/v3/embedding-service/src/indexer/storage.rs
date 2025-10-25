//! Database storage and persistence
//!
//! Database operations for embedding storage, index persistence,
//! and retrieval with connection pooling and health monitoring.

use super::super::types::*;
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Database client for embedding operations
#[derive(Debug)]
pub struct EmbeddingStorage {
    pool: PgPool,
    config: DatabaseConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub database_url: String,
    pub health_check_enabled: bool,
}

impl EmbeddingStorage {
    /// Create new storage client
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url)
            .await?;

        Ok(Self { pool, config })
    }

    /// Store embedding record
    pub async fn store_embedding(&self, record: &EmbeddingRecord) -> Result<()> {
        sqlx::query!(
            "INSERT INTO embeddings (id, content_id, embedding, model, dimensions, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
               embedding = EXCLUDED.embedding,
               updated_at = NOW()",
            record.id,
            record.content_id,
            &record.embedding.values,
            record.embedding.model,
            record.embedding.dimensions as i32,
            record.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Retrieve embedding by ID
    pub async fn get_embedding(&self, id: Uuid) -> Result<Option<EmbeddingRecord>> {
        let row = sqlx::query!(
            "SELECT id, content_id, embedding, model, dimensions, created_at, updated_at
             FROM embeddings WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| EmbeddingRecord {
            id: r.id,
            content_id: r.content_id,
            embedding: EmbeddingVector {
                values: r.embedding,
                model: r.model,
                dimensions: r.dimensions as usize,
            },
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Find similar embeddings using vector similarity
    pub async fn find_similar(&self, embedding: &EmbeddingVector, limit: usize) -> Result<Vec<EmbeddingSimilarity>> {
        // Placeholder - would use pgvector or similar extension
        let rows = sqlx::query!(
            "SELECT id, content_id, embedding, model, dimensions,
                    1 - (embedding <=> $1) as similarity
             FROM embeddings
             ORDER BY embedding <=> $1
             LIMIT $2",
            &embedding.values,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| EmbeddingSimilarity {
            embedding_id: row.id,
            content_id: row.content_id,
            similarity: row.similarity.unwrap_or(0.0),
        }).collect())
    }

    /// Store text document metadata
    pub async fn store_text_document(&self, doc: &super::text::TextDocument) -> Result<()> {
        sqlx::query!(
            "INSERT INTO text_documents (id, title, content, metadata, term_frequencies, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
               title = EXCLUDED.title,
               content = EXCLUDED.content,
               metadata = EXCLUDED.metadata,
               term_frequencies = EXCLUDED.term_frequencies,
               updated_at = NOW()",
            doc.id,
            doc.title,
            doc.content,
            serde_json::to_value(&doc.metadata).unwrap(),
            serde_json::to_value(&doc.term_frequencies).unwrap(),
            chrono::Utc::now()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get text documents by BM25 search
    pub async fn search_text_documents(&self, query: &str, limit: usize) -> Result<Vec<super::text::TextDocument>> {
        // Placeholder - would use full-text search
        let rows = sqlx::query!(
            "SELECT id, title, content, metadata, term_frequencies
             FROM text_documents
             WHERE content ILIKE $1
             LIMIT $2",
            format!("%{}%", query),
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| super::text::TextDocument {
            id: row.id,
            content: row.content,
            title: row.title,
            metadata: serde_json::from_value(row.metadata).unwrap_or_default(),
            term_frequencies: serde_json::from_value(row.term_frequencies).unwrap_or_default(),
        }).collect())
    }

    /// Health check for database connectivity
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let embedding_count = sqlx::query_scalar!("SELECT COUNT(*) FROM embeddings")
            .fetch_one(&self.pool)
            .await?;

        let document_count = sqlx::query_scalar!("SELECT COUNT(*) FROM text_documents")
            .fetch_one(&self.pool)
            .await?;

        Ok(DatabaseStats {
            embedding_count: embedding_count.unwrap_or(0),
            document_count: document_count.unwrap_or(0),
            pool_size: self.pool.size() as u32,
            idle_connections: self.pool.num_idle() as u32,
        })
    }
}

/// Embedding similarity result
#[derive(Debug)]
pub struct EmbeddingSimilarity {
    pub embedding_id: Uuid,
    pub content_id: Uuid,
    pub similarity: f64,
}

/// Database statistics
#[derive(Debug)]
pub struct DatabaseStats {
    pub embedding_count: i64,
    pub document_count: i64,
    pub pool_size: u32,
    pub idle_connections: u32,
}

/// Embedding record for database storage
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmbeddingRecord {
    pub id: Uuid,
    pub content_id: Uuid,
    pub embedding: EmbeddingVector,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
