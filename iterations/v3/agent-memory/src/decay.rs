//! Memory Decay Engine - Importance weighting and decay schedules

use crate::types::*;
use crate::MemoryResult;
use agent_agency_database::{DatabaseClient, DatabaseConfig, Row};
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

/// Memory decay engine for managing importance and decay
#[derive(Debug)]
pub struct MemoryDecayEngine {
    db_client: Arc<DatabaseClient>,
    config: DecayConfig,
}

impl MemoryDecayEngine {
    /// Create a new decay engine
    pub async fn new(config: &DecayConfig) -> MemoryResult<Self> {
        let db_config = agent_agency_database::DatabaseConfig::default();
        let db_client = Arc::new(DatabaseClient::new(db_config).await?);

        Ok(Self {
            db_client,
            config: config.clone(),
        })
    }

    /// Run a full decay cycle on all memories
    pub async fn run_decay_cycle(&self) -> MemoryResult<usize> {
        let now = Utc::now();
        let mut total_updated = 0;

        // Apply decay based on the configured schedule
        match self.config.decay_schedule {
            DecaySchedule::Exponential => {
                total_updated = self.apply_exponential_decay(now).await?;
            }
            DecaySchedule::PowerLaw => {
                total_updated = self.apply_power_law_decay(now).await?;
            }
            DecaySchedule::Logarithmic => {
                total_updated = self.apply_logarithmic_decay(now).await?;
            }
            DecaySchedule::Custom(ref formula) => {
                total_updated = self.apply_custom_decay(now, formula).await?;
            }
        }

        // Boost importance of recently accessed memories
        let boosted = self.boost_recently_accessed().await?;
        total_updated += boosted;

        // Consolidate important memories
        let consolidated = self.consolidate_important_memories().await?;
        total_updated += consolidated;

        info!("Decay cycle completed: {} memories updated, {} boosted, {} consolidated",
              total_updated, boosted, consolidated);

        Ok(total_updated)
    }

    /// Apply exponential decay: importance *= (1 - decay_rate) ^ time_elapsed
    async fn apply_exponential_decay(&self, now: DateTime<Utc>) -> MemoryResult<usize> {
        let updated = sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET decay_factor = GREATEST(
                decay_factor * POWER(1.0 - $1, EXTRACT(EPOCH FROM ($2 - last_accessed)) / 86400),
                $3  -- minimum decay factor
            )
            WHERE last_accessed < $2 - INTERVAL '1 hour'
            "#,
        )
        .bind(self.config.base_decay_rate)
        .bind(now)
        .bind(self.config.minimum_memory_strength)
        .execute(self.db_client.pool())
        .await?;

        Ok(updated.rows_affected() as usize)
    }

    /// Apply power law decay: importance *= time_elapsed ^ (-decay_rate)
    async fn apply_power_law_decay(&self, now: DateTime<Utc>) -> MemoryResult<usize> {
        let updated = sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET decay_factor = GREATEST(
                decay_factor * POWER(EXTRACT(EPOCH FROM ($1 - last_accessed)) / 86400, -$2),
                $3
            )
            WHERE last_accessed < $1 - INTERVAL '1 hour'
            "#,
        )
        .bind(now)
        .bind(self.config.base_decay_rate)
        .bind(self.config.minimum_memory_strength)
        .execute(self.db_client.pool())
        .await?;

        Ok(updated.rows_affected() as usize)
    }

    /// Apply logarithmic decay: importance -= log(time_elapsed) * decay_rate
    async fn apply_logarithmic_decay(&self, now: DateTime<Utc>) -> MemoryResult<usize> {
        let updated = sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET decay_factor = GREATEST(
                decay_factor - (LN(EXTRACT(EPOCH FROM ($1 - last_accessed)) / 86400 + 1) * $2),
                $3
            )
            WHERE last_accessed < $1 - INTERVAL '1 hour'
            "#,
        )
        .bind(now)
        .bind(self.config.base_decay_rate)
        .bind(self.config.minimum_memory_strength)
        .execute(self.db_client.pool())
        .await?;

        Ok(updated.rows_affected() as usize)
    }

    /// Apply custom decay formula (simplified implementation)
    async fn apply_custom_decay(&self, now: DateTime<Utc>, _formula: &str) -> MemoryResult<usize> {
        // For now, fall back to exponential decay
        // In a full implementation, this would parse and evaluate custom formulas
        warn!("Custom decay formulas not fully implemented, using exponential decay");
        self.apply_exponential_decay(now).await
    }

    /// Boost importance of recently accessed memories
    async fn boost_recently_accessed(&self) -> MemoryResult<usize> {
        let cutoff = Utc::now() - Duration::hours(24); // Last 24 hours

        let updated = sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET importance_score = LEAST(
                importance_score * (1.0 + ($1 * access_count / 10.0)),
                2.0  -- Maximum importance boost
            ),
            decay_factor = LEAST(decay_factor * 1.1, 1.0)
            WHERE last_accessed > $2 AND access_count > 0
            "#,
        )
        .bind(self.config.importance_boost_factor)
        .bind(cutoff)
        .execute(self.db_client.pool())
        .await?;

        Ok(updated.rows_affected() as usize)
    }

    /// Consolidate important memories to prevent information loss
    async fn consolidate_important_memories(&self) -> MemoryResult<usize> {
        // Find memories that are important but decaying
        let important_decaying = sqlx::query(
            r#"
            SELECT memory_id, importance_score, decay_factor
            FROM memory_embeddings
            WHERE importance_score > 1.5
              AND decay_factor < 0.7
              AND last_accessed < NOW() - INTERVAL '7 days'
            "#,
        )
        .fetch_all(self.db_client.pool())
        .await?;

        let mut consolidated = 0;

        for row in important_decaying {
            let memory_id: MemoryId = row.try_get("memory_id")?;
            let importance: f32 = row.try_get("importance_score")?;
            let decay: f32 = row.try_get("decay_factor")?;

            // Calculate consolidation boost
            let consolidation_boost = (importance * (1.0 - decay)).min(0.5);

            sqlx::query(
                r#"
                UPDATE memory_embeddings
                SET decay_factor = LEAST(decay_factor + $2, 1.0),
                    importance_score = importance_score * 0.95  -- Slight importance decay after consolidation
                WHERE memory_id = $1
                "#,
            )
            .bind(memory_id)
            .bind(consolidation_boost)
            .execute(self.db_client.pool())
            .await?;

            consolidated += 1;
        }

        Ok(consolidated)
    }

    /// Apply temporal weighting to contextual memories
    pub async fn apply_temporal_weighting(&self, memories: &mut Vec<ContextualMemory>) -> MemoryResult<()> {
        let now = Utc::now();

        for memory in memories.iter_mut() {
            let age_hours = (now - memory.memory.timestamp).num_hours() as f32;

            // Recency boost: newer memories get higher weight
            let recency_boost = if age_hours < 24.0 {
                1.0 + (24.0 - age_hours) / 48.0  // Up to 1.5x boost for very recent
            } else if age_hours < 168.0 {  // Week
                1.0 + (168.0 - age_hours) / 336.0  // Up to 1.25x boost
            } else {
                1.0  // No boost for older memories
            };

            // Apply temporal weighting
            memory.relevance_score *= recency_boost;

            // Add temporal reasoning to path
            if recency_boost > 1.0 {
                memory.reasoning_path.push(format!("Temporal boost: {:.2}x ({}h ago)",
                                                 recency_boost, age_hours));
            }
        }

        Ok(())
    }

    /// Manually boost importance of specific memories
    pub async fn boost_memory_importance(&self, memory_id: MemoryId, boost_factor: f32) -> MemoryResult<()> {
        sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET importance_score = LEAST(importance_score * $2, 3.0),
                decay_factor = LEAST(decay_factor * 1.2, 1.0)
            WHERE memory_id = $1
            "#,
        )
        .bind(memory_id)
        .bind(boost_factor)
        .execute(self.db_client.pool())
        .await?;

        info!("Boosted importance of memory {} by factor {}", memory_id, boost_factor);
        Ok(())
    }

    /// Protect important memories from decay
    pub async fn protect_important_memories(&self, min_importance: f32) -> MemoryResult<usize> {
        let updated = sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET decay_factor = 1.0
            WHERE importance_score >= $1 AND decay_factor < 0.8
            "#,
        )
        .bind(min_importance)
        .execute(self.db_client.pool())
        .await?;

        let protected_count = updated.rows_affected() as usize;

        if protected_count > 0 {
            info!("Protected {} important memories from decay", protected_count);
        }

        Ok(protected_count)
    }

    /// Get decay statistics
    pub async fn get_decay_stats(&self) -> MemoryResult<DecayStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_memories,
                AVG(importance_score) as avg_importance,
                AVG(decay_factor) as avg_decay,
                MIN(decay_factor) as min_decay,
                MAX(importance_score) as max_importance,
                COUNT(CASE WHEN decay_factor < 0.5 THEN 1 END) as heavily_decayed,
                COUNT(CASE WHEN importance_score > 1.5 THEN 1 END) as highly_important
            FROM memory_embeddings
            "#,
        )
        .fetch_one(self.db_client.pool())
        .await?;

        Ok(DecayStats {
            total_memories: row.try_get::<i64, _>("total_memories").unwrap_or(0) as usize,
            avg_importance: row.try_get::<Option<f64>, _>("avg_importance")?.unwrap_or(0.0) as f32,
            avg_decay: row.try_get::<Option<f64>, _>("avg_decay")?.unwrap_or(0.0) as f32,
            min_decay: row.try_get::<Option<f64>, _>("min_decay")?.unwrap_or(0.0) as f32,
            max_importance: row.try_get::<Option<f64>, _>("max_importance")?.unwrap_or(0.0) as f32,
            heavily_decayed: row.try_get::<i64, _>("heavily_decayed").unwrap_or(0) as usize,
            highly_important: row.try_get::<i64, _>("highly_important").unwrap_or(0) as usize,
        })
    }

    /// Reset decay for testing purposes
    #[cfg(test)]
    pub async fn reset_decay_for_testing(&self) -> MemoryResult<()> {
        sqlx::query(
            r#"
            UPDATE memory_embeddings
            SET decay_factor = 1.0, importance_score = 1.0, access_count = 0
            "#,
        )
        .execute(self.db_client.pool())
        .await?;

        Ok(())
    }
}

/// Decay statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayStats {
    pub total_memories: usize,
    pub avg_importance: f32,
    pub avg_decay: f32,
    pub min_decay: f32,
    pub max_importance: f32,
    pub heavily_decayed: usize,
    pub highly_important: usize,
}
