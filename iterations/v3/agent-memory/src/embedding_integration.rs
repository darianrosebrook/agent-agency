//! Embedding Integration - Vector embeddings for memory with decay/importance

use crate::types::*;
use crate::MemoryResult;
use embedding_service::{EmbeddingService, EmbeddingConfig as ESConfig, ContentType};
use agent_agency_database::{DatabaseClient, DatabaseConfig, Row};
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Memory embedding with decay information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEmbedding {
    pub memory_id: MemoryId,
    pub embedding: Vec<f32>,
    pub importance_score: f32,
    pub decay_factor: f32,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub created_at: DateTime<Utc>,
}

/// Embedding integration for memory operations
pub struct EmbeddingIntegration {
    embedding_service: Arc<dyn EmbeddingService>,
    db_client: Arc<DatabaseClient>,
    config: EmbeddingConfig,
}

impl EmbeddingIntegration {
    /// Create a new embedding integration
    pub async fn new(config: &EmbeddingConfig) -> MemoryResult<Self> {
        // Create embedding service config
        let es_config = ESConfig {
            model_name: "embeddinggemma".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_ms: 30000,
            batch_size: 32,
            cache_size: 1000,
            dimension: 768,
        };

        // For now, create a placeholder embedding provider
        // TODO: Get proper provider injection
        let provider = embedding_service::OllamaEmbeddingProvider::new(&es_config);
        let embedding_service = Arc::new(embedding_service::EmbeddingServiceImpl::new(Arc::new(provider), es_config));
        let db_config = agent_agency_database::DatabaseConfig::default();
        let db_client = Arc::new(DatabaseClient::new(db_config).await?);

        Ok(Self {
            embedding_service,
            db_client,
            config: config.clone(),
        })
    }

    /// Generate embedding for an agent experience
    pub async fn generate_experience_embedding(&self, experience: &AgentExperience) -> MemoryResult<Vec<f32>> {
        // Create a text representation of the experience
        let text_representation = format!(
            "Agent {} performed task '{}': {}. Context: {}. Outcome: {}. Learned: {}",
            experience.agent_id,
            experience.task_id,
            experience.context.description,
            serde_json::to_string(&experience.context).unwrap_or_default(),
            serde_json::to_string(&experience.outcome).unwrap_or_default(),
            experience.outcome.learned_capabilities.join(", ")
        );

        let stored_embedding = self.embedding_service.generate_embedding(
            &text_representation,
            ContentType::Knowledge,
            "agent_memory_experience"
        ).await?;
        Ok(stored_embedding.vector)
    }

    /// Generate embedding for task context
    pub async fn generate_context_embedding(&self, context: &TaskContext) -> MemoryResult<Vec<f32>> {
        let text_representation = format!(
            "Task '{}': {} in domain(s): {}. Entities: {}",
            context.task_type,
            context.description,
            context.domain.join(", "),
            context.entities.join(", ")
        );

        let stored_embedding = self.embedding_service.generate_embedding(
            &text_representation,
            ContentType::TaskDescription,
            "agent_memory_context"
        ).await?;
        Ok(stored_embedding.vector)
    }

    /// Store embedding with metadata
    pub async fn store_embedding(&self, memory_id: MemoryId, embedding: Vec<f32>) -> MemoryResult<()> {
        let memory_embedding = MemoryEmbedding {
            memory_id,
            embedding,
            importance_score: 1.0, // Default importance
            decay_factor: 1.0,      // No decay initially
            last_accessed: Utc::now(),
            access_count: 0,
            created_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO memory_embeddings (
                memory_id, embedding, importance_score, decay_factor,
                last_accessed, access_count, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (memory_id) DO UPDATE SET
                embedding = EXCLUDED.embedding,
                last_accessed = EXCLUDED.last_accessed
            "#,
        )
        .bind(memory_id)
        .bind(&memory_embedding.embedding)
        .bind(memory_embedding.importance_score)
        .bind(memory_embedding.decay_factor)
        .bind(memory_embedding.last_accessed)
        .bind(memory_embedding.access_count as i32)
        .bind(memory_embedding.created_at)
        .execute(self.db_client.pool())
        .await?;

        debug!("Stored embedding for memory: {}", memory_id);
        Ok(())
    }

    /// Semantic search for memories similar to context
    pub async fn semantic_search_context(&self, context: &TaskContext, limit: usize) -> MemoryResult<Vec<(MemoryId, f32)>> {
        let context_embedding = self.generate_context_embedding(context).await?;

        // Find similar embeddings using cosine similarity
        let rows = sqlx::query(
            r#"
            SELECT memory_id, embedding <=> $1 as similarity,
                   importance_score, decay_factor, access_count
            FROM memory_embeddings
            WHERE (importance_score * decay_factor) > 0.1  -- Only consider relevant memories
            ORDER BY (embedding <=> $1) * (importance_score * decay_factor) ASC
            LIMIT $2
            "#,
        )
        .bind(&context_embedding)
        .bind(limit as i32)
        .fetch_all(self.db_client.pool())
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let memory_id: MemoryId = row.try_get("memory_id")?;
            let similarity: f32 = 1.0 - row.try_get::<f64, _>("similarity")? as f32; // Convert distance to similarity
            let importance_score: f32 = row.try_get("importance_score")?;
            let decay_factor: f32 = row.try_get("decay_factor")?;

            // Apply importance and decay weighting
            let weighted_similarity = similarity * importance_score * decay_factor;

            if weighted_similarity > self.config.similarity_threshold {
                results.push((memory_id, weighted_similarity));

                // Update access statistics
                self.update_access_stats(memory_id).await?;
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// Semantic search for general text queries
    pub async fn semantic_search_text(&self, query: &str, limit: usize) -> MemoryResult<Vec<(MemoryId, f32)>> {
        let query_embedding = self.embedding_service.generate_embedding(
            query,
            ContentType::Knowledge,
            "memory_search"
        ).await?;

        let rows = sqlx::query(
            r#"
            SELECT memory_id, embedding <=> $1 as similarity,
                   importance_score, decay_factor
            FROM memory_embeddings
            ORDER BY (embedding <=> $1) * (importance_score * decay_factor) ASC
            LIMIT $2
            "#,
        )
        .bind(&query_embedding.vector)
        .bind(limit as i32)
        .fetch_all(self.db_client.pool())
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let memory_id: MemoryId = row.try_get("memory_id")?;
            let similarity: f32 = 1.0 - row.try_get::<f64, _>("similarity")? as f32;
            let importance_score: f32 = row.try_get("importance_score")?;
            let decay_factor: f32 = row.try_get("decay_factor")?;

            let weighted_similarity = similarity * importance_score * decay_factor;
            if weighted_similarity > self.config.similarity_threshold {
                results.push((memory_id, weighted_similarity));
                self.update_access_stats(memory_id).await?;
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// Update importance score for a memory
    pub async fn update_importance(&self, memory_id: MemoryId, new_importance: f32) -> MemoryResult<()> {
        sqlx::query(
            "UPDATE memory_embeddings SET importance_score = $2 WHERE memory_id = $1",
        )
        .bind(memory_id)
        .bind(new_importance)
        .execute(self.db_client.pool())
        .await?;

        debug!("Updated importance score for memory {} to {}", memory_id, new_importance);
        Ok(())
    }

    /// Apply decay to all embeddings based on time and access patterns
    pub async fn apply_decay(&self) -> MemoryResult<usize> {
        let now = Utc::now();

        // Calculate decay based on time since last access and access frequency
        let updated = sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET decay_factor = GREATEST(
                decay_factor * POWER(0.99, EXTRACT(EPOCH FROM ($1 - last_accessed)) / 86400),
                0.1  -- Minimum decay factor
            ),
            last_accessed = $1
            WHERE last_accessed < $1 - INTERVAL '1 day'
            "#,
        )
        .bind(now)
        .execute(self.db_client.pool())
        .await?;

        let updated_count = updated.rows_affected() as usize;

        if updated_count > 0 {
            info!("Applied decay to {} embeddings", updated_count);
        }

        Ok(updated_count)
    }

    /// Boost importance of recently accessed memories
    pub async fn boost_recent_accesses(&self, hours: i64) -> MemoryResult<usize> {
        let cutoff = Utc::now() - Duration::hours(hours);

        let updated = sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET importance_score = LEAST(importance_score * 1.1, 2.0),
                decay_factor = LEAST(decay_factor * 1.05, 1.0)
            WHERE last_accessed > $1 AND access_count > 0
            "#,
        )
        .bind(cutoff)
        .execute(self.db_client.pool())
        .await?;

        let updated_count = updated.rows_affected() as usize;

        if updated_count > 0 {
            debug!("Boosted importance of {} recently accessed memories", updated_count);
        }

        Ok(updated_count)
    }

    /// Update access statistics when a memory is retrieved
    async fn update_access_stats(&self, memory_id: MemoryId) -> MemoryResult<()> {
        sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET access_count = access_count + 1,
                last_accessed = $2
            WHERE memory_id = $1
            "#,
        )
        .bind(memory_id)
        .bind(Utc::now())
        .execute(self.db_client.pool())
        .await?;

        Ok(())
    }

    /// Get embedding statistics
    pub async fn get_embedding_stats(&self) -> MemoryResult<EmbeddingStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_embeddings,
                AVG(importance_score) as avg_importance,
                AVG(decay_factor) as avg_decay,
                AVG(access_count) as avg_access_count,
                MIN(created_at) as oldest_embedding,
                MAX(last_accessed) as newest_access
            FROM memory_embeddings
            "#,
        )
        .fetch_one(self.db_client.pool())
        .await?;

        Ok(EmbeddingStats {
            total_embeddings: row.try_get::<i64, _>("total_embeddings").unwrap_or(0) as usize,
            avg_importance: row.try_get::<Option<f64>, _>("avg_importance")?.unwrap_or(0.0) as f32,
            avg_decay: row.try_get::<Option<f64>, _>("avg_decay")?.unwrap_or(0.0) as f32,
            avg_access_count: row.try_get::<Option<f64>, _>("avg_access_count")?.unwrap_or(0.0) as f32,
            oldest_embedding: row.try_get("oldest_embedding")?,
            newest_access: row.try_get("newest_access")?,
        })
    }

    /// Clean up embeddings for deleted memories
    pub async fn cleanup_orphaned_embeddings(&self) -> MemoryResult<usize> {
        let deleted = sqlx::query(
            r#"
            DELETE FROM memory_embeddings
            WHERE memory_id NOT IN (
                SELECT id FROM agent_experiences
            )
            "#,
        )
        .execute(self.db_client.pool())
        .await?;

        let deleted_count = deleted.rows_affected() as usize;

        if deleted_count > 0 {
            info!("Cleaned up {} orphaned embeddings", deleted_count);
        }

        Ok(deleted_count)
    }
}

/// Embedding statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStats {
    pub total_embeddings: usize,
    pub avg_importance: f32,
    pub avg_decay: f32,
    pub avg_access_count: f32,
    pub oldest_embedding: Option<DateTime<Utc>>,
    pub newest_access: Option<DateTime<Utc>>,
}
