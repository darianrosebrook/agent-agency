//! Database storage and persistence
//!
//! Database operations for embedding storage, index persistence,
//! and retrieval with connection pooling and health monitoring.

use super::super::embedding_types::*;
use anyhow::Result;
use sqlx::{PgPool, Row};
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
        sqlx::query(
            "INSERT INTO embeddings (id, content_id, embedding, model, dimensions, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
               embedding = EXCLUDED.embedding,
               updated_at = NOW()"
        )
        .bind(&record.id)
        .bind(&record.content_id)
        .bind(&record.embedding.values)
        .bind(&record.embedding.model)
        .bind(record.embedding.dimensions as i32)
        .bind(&record.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Retrieve embedding by ID
    pub async fn get_embedding(&self, id: Uuid) -> Result<Option<EmbeddingRecord>> {
        let row = sqlx::query(
            "SELECT id, content_id, embedding, model, dimensions, created_at, updated_at
             FROM embeddings WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r: sqlx::postgres::PgRow| EmbeddingRecord {
            id: r.get("id"),
            content_id: r.get("content_id"),
            embedding: EmbeddingVector {
                values: r.get("embedding"),
                model: r.get("model"),
                dimensions: r.get::<i32, _>("dimensions") as usize,
            },
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// Find similar embeddings using vector similarity
    pub async fn find_similar(&self, embedding: &EmbeddingVector, limit: usize) -> Result<Vec<EmbeddingSimilarity>> {
        // Placeholder - would use pgvector or similar extension
        let rows = sqlx::query(
            "SELECT id, content_id, embedding, model, dimensions,
                    1 - (embedding <=> $1) as similarity
             FROM embeddings
             ORDER BY embedding <=> $1
             LIMIT $2"
        )
        .bind(&embedding.values)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row: sqlx::postgres::PgRow| EmbeddingSimilarity {
            embedding_id: row.get("id"),
            content_id: row.get("content_id"),
            similarity: row.get::<Option<f64>, _>("similarity").unwrap_or(0.0),
        }).collect())
    }

    /// Store text document metadata
    pub async fn store_text_document(&self, doc: &super::text::TextDocument) -> Result<()> {
        sqlx::query(
            "INSERT INTO text_documents (id, title, content, metadata, term_frequencies, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
               title = EXCLUDED.title,
               content = EXCLUDED.content,
               metadata = EXCLUDED.metadata,
               term_frequencies = EXCLUDED.term_frequencies,
               updated_at = NOW()"
        )
        .bind(&doc.id)
        .bind(&doc.title)
        .bind(&doc.content)
        .bind(serde_json::to_value(&doc.metadata).unwrap())
        .bind(serde_json::to_value(&doc.term_frequencies).unwrap())
        .bind(chrono::Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Real full-text search implementation using PostgreSQL
    pub async fn search_text_documents(&self, query: &str, limit: usize) -> Result<Vec<super::text::TextDocument>> {
        use tracing::{info, debug};
        
        info!("Performing full-text search for query: '{}'", query);
        
        // Use PostgreSQL's full-text search capabilities
        let rows = sqlx::query(
            "SELECT id, title, content, metadata, term_frequencies,
                    ts_rank(to_tsvector('english', title || ' ' || content), plainto_tsquery('english', $1)) as rank
             FROM text_documents
             WHERE to_tsvector('english', title || ' ' || content) @@ plainto_tsquery('english', $1)
             ORDER BY rank DESC
             LIMIT $2"
        )
        .bind(query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results = rows.into_iter().map(|row: sqlx::postgres::PgRow| {
            super::text::TextDocument {
                id: row.get("id"),
                title: row.get("title"),
                content: row.get("content"),
                metadata: serde_json::from_value(row.get("metadata")).unwrap_or_default(),
                term_frequencies: serde_json::from_value(row.get("term_frequencies")).unwrap_or_default(),
            }
        }).collect();

        debug!("Found {} text documents matching query", results.len());
        Ok(results)
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
        let embedding_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM embeddings")
            .fetch_one(&self.pool)
            .await?;

        let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM text_documents")
            .fetch_one(&self.pool)
            .await?;

        Ok(DatabaseStats {
            embedding_count: embedding_count,
            document_count: document_count,
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


