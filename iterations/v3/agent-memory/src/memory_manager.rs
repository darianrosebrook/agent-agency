//! Memory Manager - Central coordinator for all memory operations

use crate::types::*;
use crate::MemoryResult;
use crate::MemoryError;
use agent_agency_database::{DatabaseClient, DatabaseConfig, Row};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

/// Central memory manager coordinating all memory operations
#[derive(Debug)]
pub struct MemoryManager {
    db_client: Arc<DatabaseClient>,
    config: MemoryConfig,
}

impl MemoryManager {
    /// Create a new memory manager
    pub async fn new(config: MemoryConfig) -> MemoryResult<Self> {
        let db_config = agent_agency_database::DatabaseConfig::default();
        let db_client = Arc::new(DatabaseClient::new(db_config).await?);

        Ok(Self {
            db_client,
            config,
        })
    }

    /// Store an agent experience
    pub async fn store_experience(&self, experience: AgentExperience) -> MemoryResult<MemoryId> {
        let memory_id = experience.id;

        // Insert into agent_experiences table
        sqlx::query(
            r#"
            INSERT INTO agent_experiences (
                id, agent_id, task_id, context, input, output, outcome,
                memory_type, timestamp, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(memory_id)
        .bind(&experience.agent_id)
        .bind(&experience.task_id)
        .bind(serde_json::to_value(&experience.context)?)
        .bind(&experience.input)
        .bind(&experience.output)
        .bind(serde_json::to_value(&experience.outcome)?)
        .bind(experience.memory_type as i32)
        .bind(experience.timestamp)
        .bind(serde_json::to_value(&experience.metadata)?)
        .execute(self.db_client.pool())
        .await?;

        info!("Stored agent experience: {} for agent {}", memory_id, experience.agent_id);
        Ok(memory_id)
    }

    /// Retrieve a memory by ID
    pub async fn retrieve_memory(&self, memory_id: MemoryId) -> MemoryResult<AgentExperience> {
        let row = sqlx::query(
            r#"
            SELECT id, agent_id, task_id, context, input, output, outcome,
                   memory_type, timestamp, metadata
            FROM agent_experiences
            WHERE id = $1
            "#,
        )
        .bind(memory_id)
        .fetch_optional(self.db_client.pool())
        .await?
        .ok_or_else(|| MemoryError::NotFound(format!("Memory not found: {}", memory_id)))?;

        let experience = AgentExperience {
            id: row.try_get("id")?,
            agent_id: row.try_get("agent_id")?,
            task_id: row.try_get("task_id")?,
            context: serde_json::from_value(row.try_get("context")?)?,
            input: row.try_get("input")?,
            output: row.try_get("output")?,
            outcome: serde_json::from_value(row.try_get("outcome")?)?,
            memory_type: MemoryType::try_from(row.try_get::<i32, _>("memory_type")?).unwrap_or_else(|_| MemoryType::Episodic),
            timestamp: row.try_get("timestamp")?,
            metadata: serde_json::from_value(row.try_get("metadata")?).unwrap_or_default(),
        };

        Ok(experience)
    }

    /// Search memories by various criteria
    pub async fn search_memories(&self, query: MemoryQuery) -> MemoryResult<Vec<AgentExperience>> {
        // For now, use a simple query - can be optimized later with proper query builders
        let mut sql_query = sqlx::query(
            r#"
            SELECT id, agent_id, task_id, context, input, output, outcome,
                   memory_type, timestamp, metadata
            FROM agent_experiences
            WHERE ($1::text IS NULL OR agent_id = $1)
              AND ($2::text IS NULL OR context->>'task_type' = $2)
              AND ($3::integer IS NULL OR memory_type = $3)
              AND ($4::timestamptz IS NULL OR timestamp >= $4)
              AND ($5::timestamptz IS NULL OR timestamp <= $5)
            ORDER BY timestamp DESC
            LIMIT $6
            "#,
        )
        .bind(&query.agent_id)
        .bind(&query.task_type)
        .bind(query.memory_type.map(|mt| mt as i32))
        .bind(query.time_range.as_ref().map(|tr| tr.start))
        .bind(query.time_range.as_ref().map(|tr| tr.end))
        .bind(query.limit.unwrap_or(100) as i32);

        let rows = sql_query.fetch_all(self.db_client.pool()).await?;

        let mut experiences = Vec::new();
        for row in rows {
            let experience = AgentExperience {
                id: row.try_get("id")?,
                agent_id: row.try_get("agent_id")?,
                task_id: row.try_get("task_id")?,
                context: serde_json::from_value(row.try_get("context")?)?,
                input: row.try_get("input")?,
                output: row.try_get("output")?,
                outcome: serde_json::from_value(row.try_get("outcome")?)?,
                memory_type: MemoryType::try_from(row.try_get::<i32, _>("memory_type")?).unwrap_or_else(|_| MemoryType::Episodic),
                timestamp: row.try_get("timestamp")?,
                metadata: serde_json::from_value(row.try_get("metadata")?).unwrap_or_default(),
            };
            experiences.push(experience);
        }

        Ok(experiences)
    }

    /// Update memory metadata
    pub async fn update_memory_metadata(&self, memory_id: MemoryId, metadata: HashMap<String, serde_json::Value>) -> MemoryResult<()> {
        sqlx::query(
            r#"
            UPDATE agent_experiences
            SET metadata = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(memory_id)
        .bind(serde_json::to_value(metadata)?)
        .execute(self.db_client.pool())
        .await?;

        debug!("Updated metadata for memory: {}", memory_id);
        Ok(())
    }

    /// Delete a memory
    pub async fn delete_memory(&self, memory_id: MemoryId) -> MemoryResult<()> {
        let result = sqlx::query(
            "DELETE FROM agent_experiences WHERE id = $1",
        )
        .bind(memory_id)
        .execute(self.db_client.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(MemoryError::NotFound(format!("Memory not found: {}", memory_id)));
        }

        info!("Deleted memory: {}", memory_id);
        Ok(())
    }

    /// Consolidate related memories (merge similar experiences)
    pub async fn consolidate_memories(&self) -> MemoryResult<usize> {
        // Find memories with similar contexts that can be consolidated
        let consolidatable = sqlx::query(
            r#"
            SELECT agent_id, context->>'task_type' as task_type, COUNT(*) as count
            FROM agent_experiences
            WHERE timestamp > NOW() - INTERVAL '24 hours'
            GROUP BY agent_id, context->>'task_type'
            HAVING COUNT(*) > 5
            "#,
        )
        .fetch_all(self.db_client.pool())
        .await?;

        let mut consolidated_count = 0;

        for row in consolidatable {
            let agent_id: String = row.try_get("agent_id")?;
            let task_type: String = row.try_get("task_type")?;
            let count: i64 = row.try_get("count")?;

            // Create consolidated memory
            let consolidated_id = MemoryId::new_v4();
            sqlx::query(
                r#"
                INSERT INTO agent_experiences (
                    id, agent_id, task_id, context, input, output, outcome,
                    memory_type, timestamp, metadata
                )
                SELECT $1, agent_id, 'consolidated-' || context->>'task_type',
                       jsonb_build_object('task_type', context->>'task_type',
                                        'consolidated_count', $3),
                       '{}'::jsonb, '{}'::jsonb,
                       jsonb_build_object('success', true, 'consolidated', true),
                       $4, NOW(),
                       jsonb_build_object('consolidated_from', array_agg(id))
                FROM agent_experiences
                WHERE agent_id = $2 AND context->>'task_type' = $5
                  AND timestamp > NOW() - INTERVAL '24 hours'
                GROUP BY agent_id, context->>'task_type'
                "#,
            )
            .bind(consolidated_id)
            .bind(&agent_id)
            .bind(count)
            .bind(MemoryType::Semantic as i32)
            .bind(&task_type)
            .execute(self.db_client.pool())
            .await?;

            consolidated_count += 1;
        }

        if consolidated_count > 0 {
            info!("Consolidated {} memory groups", consolidated_count);
        }

        Ok(consolidated_count)
    }

    /// Clean up expired memories based on decay rules
    pub async fn cleanup_expired_memories(&self) -> MemoryResult<usize> {
        // Delete memories older than retention period (90 days default)
        let retention_days = 90;
        let result = sqlx::query(
            "DELETE FROM agent_experiences WHERE timestamp < NOW() - INTERVAL '90 days'",
        )
        .execute(self.db_client.pool())
        .await?;

        let deleted_count = result.rows_affected() as usize;

        if deleted_count > 0 {
            info!("Cleaned up {} expired memories", deleted_count);
        }

        Ok(deleted_count)
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self) -> MemoryResult<MemoryStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_memories,
                COUNT(DISTINCT agent_id) as unique_agents,
                AVG(EXTRACT(EPOCH FROM (NOW() - timestamp))) as avg_age_seconds,
                MIN(timestamp) as oldest_memory,
                MAX(timestamp) as newest_memory
            FROM agent_experiences
            "#,
        )
        .fetch_one(self.db_client.pool())
        .await?;

        let total_memories: i64 = row.try_get("total_memories")?;
        let unique_agents: i64 = row.try_get("unique_agents")?;
        let avg_age_seconds: Option<f64> = row.try_get("avg_age_seconds")?;
        let oldest_memory: Option<DateTime<Utc>> = row.try_get("oldest_memory")?;
        let newest_memory: Option<DateTime<Utc>> = row.try_get("newest_memory")?;

        Ok(MemoryStats {
            total_memories: total_memories as usize,
            unique_agents: unique_agents as usize,
            avg_age_seconds: avg_age_seconds.unwrap_or(0.0),
            oldest_memory,
            newest_memory,
            memory_types_distribution: HashMap::new(), // TODO: implement
        })
    }
}

/// Memory query structure
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub agent_id: Option<String>,
    pub task_type: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<usize>,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            agent_id: None,
            task_type: None,
            memory_type: None,
            time_range: None,
            limit: Some(50),
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub unique_agents: usize,
    pub avg_age_seconds: f64,
    pub oldest_memory: Option<DateTime<Utc>>,
    pub newest_memory: Option<DateTime<Utc>>,
    pub memory_types_distribution: HashMap<MemoryType, usize>,
}
