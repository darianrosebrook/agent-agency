#![cfg(feature = "database")]
//! Embedding Integration - Vector embeddings for memory with decay/importance

use crate::memory_types::*;
use crate::{MemoryResult, MemoryError};
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use reqwest::Client;
use anyhow::{Context, Result};
use std::collections::HashMap;
use sqlx::{PgPool, Row};
/// Memory embedding with decay information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MemoryEmbedding {
    #[schemars(with = "String")]
    pub memory_id: MemoryId,
    pub embedding: Vec<f32>,
    pub importance_score: f32,
    pub decay_factor: f32,
    #[schemars(with = "String")]
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// Real HTTP-based embedding service
pub struct HttpEmbeddingService {
    client: Client,
    base_url: String,
    model_name: String,
    timeout_ms: u64,
}

impl HttpEmbeddingService {
    pub fn new(base_url: String, model_name: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            model_name,
            timeout_ms: 30000,
        }
    }

    /// Generate embedding via HTTP call to external service
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/v1/embeddings", self.base_url);
        
        let payload = serde_json::json!({
            "model": self.model_name,
            "input": text,
            "encoding_format": "float"
        });

        debug!("Generating embedding for text: {}...", &text[..text.len().min(100)]);

        let response = self.client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .send()
            .await
            .context("Failed to send embedding request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Embedding service error {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await
            .context("Failed to parse embedding response")?;

        // Extract embedding vector from response
        let embedding = result["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid embedding response format"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        debug!("Generated embedding with {} dimensions", embedding.len());
        Ok(embedding)
    }

    /// Health check for embedding service
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        let url = format!("{}/api/v1/stats", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to get embedding service stats")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get stats: {}", response.status()));
        }

        let stats: HashMap<String, serde_json::Value> = response.json().await
            .context("Failed to parse stats response")?;

        Ok(stats)
    }
}

/// Embedding integration for memory operations
pub struct EmbeddingIntegration {
    embedding_service: Arc<HttpEmbeddingService>,
    db_pool: Arc<PgPool>,
    config: EmbeddingConfig,
}

impl EmbeddingIntegration {
    /// Create a new embedding integration
    pub async fn new(config: &EmbeddingConfig) -> MemoryResult<Self> {
        // Get embedding service URL from environment or use default
        let embedding_url = std::env::var("EMBEDDING_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        
        let model_name = std::env::var("EMBEDDING_MODEL_NAME")
            .unwrap_or_else(|_| "embeddinggemma".to_string());

        info!("Initializing HTTP embedding service at: {} with model: {}", embedding_url, model_name);

        // Create HTTP-based embedding service
        let embedding_service = Arc::new(HttpEmbeddingService::new(embedding_url, model_name));
        
        // Test connection
        if let Err(e) = embedding_service.health_check().await {
            warn!("Embedding service health check failed: {}", e);
        } else {
            info!("Embedding service health check passed");
        }

        // Get database URL from environment or use default
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        let db_pool = Arc::new(
            sqlx::PgPool::connect(&database_url)
                .await
                .context("Failed to connect to database for embedding integration")?
        );

        Ok(Self {
            embedding_service,
            db_pool,
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

        // Generate embedding via HTTP call
        let embedding = self.embedding_service.generate_embedding(&text_representation).await
            .map_err(|e| MemoryError::Embedding(e.to_string()))?;

        Ok(embedding)
    }

    /// Generate embedding for task context
    pub async fn generate_context_embedding(&self, context: &TaskContext) -> MemoryResult<Vec<f32>> {
        let text_representation = format!(
            "Task '{}': {} with keywords: {}. Entities: {}",
            context.task_type,
            context.description,
            context.keywords.join(", "),
            context.entities.join(", ")
        );

        // Generate embedding via HTTP call
        let embedding = self.embedding_service.generate_embedding(&text_representation).await
            .map_err(|e| MemoryError::Embedding(e.to_string()))?;

        Ok(embedding)
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
        .execute(self.db_pool.as_ref())
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
        .fetch_all(self.db_pool.as_ref())
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

    /// Generate embedding for arbitrary text (for workspace file embeddings)
    pub async fn generate_text_embedding(&self, text: &str) -> MemoryResult<Vec<f32>> {
        self.embedding_service.generate_embedding(text).await
            .map_err(|e| MemoryError::Embedding(e.to_string()))
    }
    
    /// Semantic search for general text queries
    pub async fn semantic_search_text(&self, query: &str, limit: usize) -> MemoryResult<Vec<(MemoryId, f32)>> {
        // Generate embedding via HTTP call
        let query_embedding = self.embedding_service.generate_embedding(query).await
            .map_err(|e| MemoryError::Embedding(e.to_string()))?;

        let rows = sqlx::query(
            r#"
            SELECT memory_id, embedding <=> $1 as similarity,
                   importance_score, decay_factor
            FROM memory_embeddings
            ORDER BY (embedding <=> $1) * (importance_score * decay_factor) ASC
            LIMIT $2
            "#,
        )
        .bind(&query_embedding)
        .bind(limit as i32)
        .fetch_all(self.db_pool.as_ref())
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
        .execute(self.db_pool.as_ref())
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
        .execute(self.db_pool.as_ref())
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
        .execute(self.db_pool.as_ref())
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
        .execute(self.db_pool.as_ref())
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
        .fetch_one(self.db_pool.as_ref())
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

    /// Get embedding service statistics
    pub async fn get_service_stats(&self) -> MemoryResult<HashMap<String, serde_json::Value>> {
        self.embedding_service.get_stats().await
            .map_err(|e| MemoryError::Embedding(e.to_string()))
    }

    /// Check embedding service health
    pub async fn health_check(&self) -> MemoryResult<bool> {
        self.embedding_service.health_check().await
            .map_err(|e| MemoryError::Embedding(e.to_string()))
    }

    /// Store file embedding in block_vectors table
    pub async fn store_file_embedding(
        &self,
        file_path: &std::path::Path,
        content: &str,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> MemoryResult<()> {
        use uuid::Uuid;
        
        // Generate block_id from file path hash for consistency
        let path_str = file_path.to_string_lossy();
        let block_id = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            path_str.as_bytes(),
        );
        
        // Determine file type/modality
        let modality = match file_path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref() {
            Some("md") | Some("txt") | Some("rs") | Some("ts") | Some("js") | Some("py") | Some("go") | Some("java") | Some("cpp") | Some("c") | Some("h") => "text",
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") | Some("webp") => "image",
            Some("mp3") | Some("wav") | Some("ogg") | Some("flac") => "audio",
            Some("mp4") | Some("avi") | Some("mov") | Some("webm") => "video",
            _ => "text", // Default to text
        };
        
        let embedding_model_id = std::env::var("EMBEDDING_MODEL_NAME")
            .unwrap_or_else(|_| "embeddinggemma".to_string());
        
        let metadata_json = metadata.unwrap_or_else(|| serde_json::json!({}));
        
        // Insert or update embedding in block_vectors
        // First check if block_id exists, then update or insert
        let existing = sqlx::query(
            "SELECT id FROM block_vectors WHERE block_id = $1 LIMIT 1"
        )
        .bind(block_id)
        .fetch_optional(self.db_pool.as_ref())
        .await?;
        
        if existing.is_some() {
            // Update existing
            sqlx::query(
                r#"
                UPDATE block_vectors SET
                    content = $2,
                    embedding = $3,
                    metadata = $4,
                    updated_at = NOW()
                WHERE block_id = $1
                "#,
            )
            .bind(block_id)
            .bind(content)
            .bind(&embedding)
            .bind(&metadata_json)
            .execute(self.db_pool.as_ref())
            .await?;
        } else {
            // Insert new
            sqlx::query(
                r#"
                INSERT INTO block_vectors (
                    block_id, content, modality, embedding_model_id, embedding, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(block_id)
            .bind(content)
            .bind(modality)
            .bind(&embedding_model_id)
            .bind(&embedding)
            .bind(&metadata_json)
            .execute(self.db_pool.as_ref())
            .await?;
        }
        
        debug!("Stored file embedding for: {}", file_path.display());
        Ok(())
    }
    
    /// Search files by semantic similarity using block_vectors
    pub async fn search_files_by_similarity(
        &self,
        query: &str,
        limit: usize,
    ) -> MemoryResult<Vec<(std::path::PathBuf, f32)>> {
        // Generate query embedding
        let query_embedding = self.generate_text_embedding(query).await?;
        
        // Search block_vectors table
        let rows = sqlx::query(
            r#"
            SELECT block_id, content, metadata, embedding <=> $1 as similarity
            FROM block_vectors
            WHERE modality = 'text'
            ORDER BY embedding <=> $1 ASC
            LIMIT $2
            "#,
        )
        .bind(&query_embedding)
        .bind(limit as i32)
        .fetch_all(self.db_pool.as_ref())
        .await?;
        
        let mut results = Vec::new();
        for row in rows {
            let metadata: serde_json::Value = row.try_get("metadata")?;
            let similarity: f32 = 1.0 - row.try_get::<f64, _>("similarity")? as f32;
            
            // Extract file_path from metadata
            if let Some(file_path_str) = metadata.get("file_path")
                .and_then(|v| v.as_str()) {
                let file_path = std::path::PathBuf::from(file_path_str);
                results.push((file_path, similarity));
            }
        }
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
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
