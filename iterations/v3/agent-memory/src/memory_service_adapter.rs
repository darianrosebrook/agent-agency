//! Postgres-backed implementation of `MemoryService`
//!
//! Connects to the real database via `sqlx::PgPool` and persists records
//! using `memory_embeddings` plus joins to `agent_experiences` when available.
//!
//! Author: @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::{info, warn};

use system_common_interfaces::memory::{
    MemoryError, MemoryId, MemoryQuery, MemoryRecord, MemoryService, ScoredMemory, WorkspaceId,
};

#[derive(Debug, Clone)]
pub struct PgMemoryService {
    db_pool: Arc<PgPool>,
}

impl PgMemoryService {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl MemoryService for PgMemoryService {
    async fn create(&self, record: MemoryRecord) -> Result<MemoryRecord, MemoryError> {
        // Insert embedding row first; content is stored in agent_experiences in the broader system.
        // Now persisting content and metadata to agent_experiences directly using the same database connection.

        let memory_uuid = uuid::Uuid::parse_str(&record.id.0)
            .map_err(|e| MemoryError::Configuration(format!("Invalid MemoryId UUID: {e}")))?;
        let workspace_uuid = if !record.workspace_id.0.is_empty() {
            Some(uuid::Uuid::parse_str(&record.workspace_id.0).map_err(|e| {
                MemoryError::Configuration(format!("Invalid WorkspaceId UUID: {e}"))
            })?)
        } else {
            None
        };

        // We tolerate NULL embedding if not provided
        let embedding: Option<Vec<f32>> = record.embedding.clone();
        let importance = record.importance as f64;
        let decay = record.decay_factor as f64;

        // Insert into memory_embeddings table
        let query = r#"
            INSERT INTO memory_embeddings (memory_id, embedding, workspace_id, importance_score, decay_factor, last_accessed)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (memory_id) DO UPDATE SET
                embedding = EXCLUDED.embedding,
                workspace_id = EXCLUDED.workspace_id,
                importance_score = EXCLUDED.importance_score,
                decay_factor = EXCLUDED.decay_factor,
                last_accessed = EXCLUDED.last_accessed
        "#;

        sqlx::query(query)
            .bind(memory_uuid)
            .bind(embedding.as_ref())
            .bind(workspace_uuid)
            .bind(importance)
            .bind(decay)
            .bind(record.last_accessed.unwrap_or_else(Utc::now))
            .execute(&*self.db_pool)
            .await
            .map_err(|e| MemoryError::Query(format!("Failed to insert memory_embeddings: {e}")))?;

        // Extract fields from metadata for agent_experiences
        let agent_id = record.metadata
            .get("agent_id")
            .and_then(|v| v.as_str())
            .unwrap_or("memory_service")
            .to_string();
        let task_id = record.metadata
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| &record.id.0)
            .to_string();
        
        // Extract input/output from metadata or use content
        let input = record.metadata
            .get("input")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| record.content.clone());
        let output = record.metadata
            .get("output")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| record.content.clone());
        
        // Extract context and outcome from metadata, or create defaults
        let context_json = record.metadata
            .get("context")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({
                "description": format!("Memory record: {}", record.content.chars().take(100).collect::<String>()),
                "domain": vec!["memory_service"]
            }));
        let outcome_json = record.metadata
            .get("outcome")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({
                "success": true,
                "performance_score": record.importance as f64,
                "quality_score": record.importance as f64
            }));
        
        // Extract memory_type from metadata (default to Episodic)
        let memory_type_int = record.metadata
            .get("memory_type")
            .and_then(|v| v.as_i64())
            .unwrap_or(0i64) as i32;

        // Insert into agent_experiences table (with ON CONFLICT to handle race conditions)
        // Note: Schema matches MemoryManager.store_experience() which uses memory_type and metadata columns
        let experience_query = r#"
            INSERT INTO agent_experiences (
                id, agent_id, task_id, context, input, output, outcome,
                memory_type, timestamp, metadata, workspace_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                context = EXCLUDED.context,
                input = EXCLUDED.input,
                output = EXCLUDED.output,
                outcome = EXCLUDED.outcome,
                metadata = EXCLUDED.metadata,
                workspace_id = EXCLUDED.workspace_id
        "#;

        sqlx::query(experience_query)
            .bind(memory_uuid)
            .bind(&agent_id)
            .bind(&task_id)
            .bind(&context_json)
            .bind(&input)
            .bind(&output)
            .bind(&outcome_json)
            .bind(memory_type_int)
            .bind(record.created_at)
            .bind(&serde_json::to_value(&record.metadata).unwrap_or_else(|_| serde_json::json!({})))
            .bind(workspace_uuid)
            .execute(&*self.db_pool)
            .await
            .map_err(|e| MemoryError::Query(format!("Failed to insert agent_experiences: {e}")))?;

        info!("memory_id = {} persisted to memory_embeddings and agent_experiences", record.id.0);

        Ok(record)
    }

    async fn update(&self, record: MemoryRecord) -> Result<MemoryRecord, MemoryError> {
        // Update embedding row and scoring attributes
        let memory_uuid = uuid::Uuid::parse_str(&record.id.0)
            .map_err(|e| MemoryError::Configuration(format!("Invalid MemoryId UUID: {e}")))?;
        let workspace_uuid = if !record.workspace_id.0.is_empty() {
            Some(uuid::Uuid::parse_str(&record.workspace_id.0).map_err(|e| {
                MemoryError::Configuration(format!("Invalid WorkspaceId UUID: {e}"))
            })?)
        } else {
            None
        };

        let embedding: Option<Vec<f32>> = record.embedding.clone();

        let query = r#"
            UPDATE memory_embeddings
            SET embedding = $2,
                workspace_id = $3,
                importance_score = $4,
                decay_factor = $5,
                last_accessed = $6
            WHERE memory_id = $1
        "#;

        let rows = sqlx::query(query)
            .bind(memory_uuid)
            .bind(embedding.as_ref())
            .bind(workspace_uuid)
            .bind(record.importance as f64)
            .bind(record.decay_factor as f64)
            .bind(record.last_accessed.unwrap_or_else(Utc::now))
            .execute(&*self.db_pool)
            .await
            .map_err(|e| MemoryError::Query(format!("Failed to update memory_embeddings: {e}")))?;

        if rows.rows_affected() == 0 {
            return Err(MemoryError::Query("No rows updated in memory_embeddings".into()));
        }

        Ok(record)
    }

    async fn get(&self, id: &MemoryId) -> Result<Option<MemoryRecord>, MemoryError> {
        let memory_uuid = uuid::Uuid::parse_str(&id.0)
            .map_err(|e| MemoryError::Configuration(format!("Invalid MemoryId UUID: {e}")))?;

        // Join with agent_experiences if present to retrieve content/metadata
        let row = sqlx::query(
            r#"
            SELECT me.memory_id, me.workspace_id, me.embedding, me.importance_score, me.decay_factor,
                   me.last_accessed,
                   ae.content, ae.metadata, ae.created_at, ae.updated_at
            FROM memory_embeddings me
            LEFT JOIN agent_experiences ae ON ae.id = me.memory_id
            WHERE me.memory_id = $1
        "#,
        )
        .bind(memory_uuid)
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| MemoryError::Query(format!("Failed to query memory: {e}")))?;

        if let Some(row) = row {
            let wid: Option<uuid::Uuid> = row.try_get("workspace_id").ok();
            // embedding may be NULL if not generated yet
            let embedding: Option<Vec<f32>> = row.try_get("embedding").ok();
            let importance: f64 = row.try_get("importance_score").unwrap_or(1.0);
            let decay: f64 = row.try_get("decay_factor").unwrap_or(1.0);
            let last_accessed: Option<DateTime<Utc>> = row.try_get("last_accessed").ok();
            let content: String = row.try_get("content").unwrap_or_default();
            let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();
            let created_at: Option<DateTime<Utc>> = row.try_get("created_at").ok();
            let updated_at: Option<DateTime<Utc>> = row.try_get("updated_at").ok();

            let record = MemoryRecord {
                id: MemoryId(id.0.clone()),
                workspace_id: WorkspaceId(wid.map(|u| u.to_string()).unwrap_or_default()),
                embedding,
                content,
                metadata: metadata
                    .and_then(|v| serde_json::from_value::<HashMap<String, serde_json::Value>>(v).ok())
                    .unwrap_or_default(),
                created_at: created_at.unwrap_or_else(Utc::now),
                updated_at: updated_at.unwrap_or_else(Utc::now),
                last_accessed,
                importance: importance as f32,
                decay_factor: decay as f32,
            };
            return Ok(Some(record));
        }

        Ok(None)
    }

    async fn search(&self, query: MemoryQuery) -> Result<Vec<ScoredMemory>, MemoryError> {
        // Vector search path via find_similar_memories()
        if let Some(vector) = &query.vector {
            let workspace_uuid = match &query.workspace_id {
                Some(w) if !w.0.is_empty() => Some(uuid::Uuid::parse_str(&w.0).map_err(|e| {
                    MemoryError::Configuration(format!("Invalid WorkspaceId UUID: {e}"))
                })?),
                _ => None,
            };

            // Safety: pgvector expects fixed dimension; DB enforces dimensions
            let similarity_threshold = 0.7_f64; // Reasonable default; can be made configurable
            let max_results = query.top_k.unwrap_or(10) as i32;

            let rows = sqlx::query(
                r#"
                SELECT r.memory_id, r.workspace_id, r.similarity_score, r.importance_score, r.relevance_score,
                       ae.content, ae.metadata, ae.created_at, ae.updated_at
                FROM find_similar_memories($1, $2, $3, $4) r
                LEFT JOIN agent_experiences ae ON ae.id = r.memory_id
                ORDER BY r.relevance_score DESC, r.similarity_score DESC
            "#,
            )
            .bind(vector)
            .bind(workspace_uuid)
            .bind(similarity_threshold)
            .bind(max_results)
            .fetch_all(&*self.db_pool)
            .await
            .map_err(|e| MemoryError::Query(format!("Vector search failed: {e}")))?;

            let mut results = Vec::with_capacity(rows.len());
            for row in rows {
                let memory_id: uuid::Uuid = row.try_get("memory_id").unwrap();
                let wid: Option<uuid::Uuid> = row.try_get("workspace_id").ok();
                let similarity: f64 = row.try_get("similarity_score").unwrap_or(0.0);
                let importance: f64 = row.try_get("importance_score").unwrap_or(1.0);
                let content: String = row.try_get("content").unwrap_or_default();
                let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();
                let created_at: Option<DateTime<Utc>> = row.try_get("created_at").ok();
                let updated_at: Option<DateTime<Utc>> = row.try_get("updated_at").ok();

                let record = MemoryRecord {
                    id: MemoryId(memory_id.to_string()),
                    workspace_id: WorkspaceId(wid.map(|u| u.to_string()).unwrap_or_default()),
                    embedding: None, // Not returned in search
                    content,
                    metadata: metadata
                        .and_then(|v| serde_json::from_value::<HashMap<String, serde_json::Value>>(v).ok())
                        .unwrap_or_default(),
                    created_at: created_at.unwrap_or_else(Utc::now),
                    updated_at: updated_at.unwrap_or_else(Utc::now),
                    last_accessed: None,
                    importance: importance as f32,
                    decay_factor: 1.0,
                };
                results.push(ScoredMemory { record, score: similarity as f32 });
            }
            return Ok(results);
        }

        // Text search path (fallback): simple ILIKE on content. Note: requires agent_experiences
        if let Some(text) = &query.text {
            let pattern = format!("%{}%", text);
            let rows = sqlx::query(
                r#"
                SELECT ae.id as memory_id, me.workspace_id, ae.content, ae.metadata, ae.created_at, ae.updated_at
                FROM agent_experiences ae
                LEFT JOIN memory_embeddings me ON me.memory_id = ae.id
                WHERE ae.content ILIKE $1
                ORDER BY ae.updated_at DESC
                LIMIT $2
            "#,
            )
            .bind(&pattern)
            .bind((query.top_k.unwrap_or(10)) as i64)
            .fetch_all(&*self.db_pool)
            .await
            .map_err(|e| MemoryError::Query(format!("Text search failed: {e}")))?;

            let mut results = Vec::with_capacity(rows.len());
            for row in rows {
                let memory_id: uuid::Uuid = row.try_get("memory_id").unwrap();
                let wid: Option<uuid::Uuid> = row.try_get("workspace_id").ok();
                let content: String = row.try_get("content").unwrap_or_default();
                let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();
                let created_at: Option<DateTime<Utc>> = row.try_get("created_at").ok();
                let updated_at: Option<DateTime<Utc>> = row.try_get("updated_at").ok();

                let record = MemoryRecord {
                    id: MemoryId(memory_id.to_string()),
                    workspace_id: WorkspaceId(wid.map(|u| u.to_string()).unwrap_or_default()),
                    embedding: None,
                    content,
                    metadata: metadata
                        .and_then(|v| serde_json::from_value::<HashMap<String, serde_json::Value>>(v).ok())
                        .unwrap_or_default(),
                    created_at: created_at.unwrap_or_else(Utc::now),
                    updated_at: updated_at.unwrap_or_else(Utc::now),
                    last_accessed: None,
                    importance: 1.0,
                    decay_factor: 1.0,
                };
                results.push(ScoredMemory { record, score: 0.0 });
            }
            return Ok(results);
        }

        Ok(vec![])
    }

    async fn touch(&self, id: &MemoryId, when: DateTime<Utc>) -> Result<(), MemoryError> {
        let memory_uuid = uuid::Uuid::parse_str(&id.0)
            .map_err(|e| MemoryError::Configuration(format!("Invalid MemoryId UUID: {e}")))?;

        let _ = sqlx::query("SELECT update_memory_access($1)")
            .bind(memory_uuid)
            .execute(&*self.db_pool)
            .await
            .map_err(|e| MemoryError::Query(format!("Failed to update access: {e}")))?;

        // Also update last_accessed explicitly to the provided time if different
        let _ = sqlx::query(
            "UPDATE memory_embeddings SET last_accessed = $2 WHERE memory_id = $1",
        )
        .bind(memory_uuid)
        .bind(when)
        .execute(&*self.db_pool)
        .await
        .map_err(|e| MemoryError::Query(format!("Failed to set last_accessed: {e}")))?;

        Ok(())
    }
}


