#![cfg(feature = "database")]
//! Memory Decay Engine - Importance weighting and decay schedules

use crate::memory_types::*;
use crate::workspace_registry;
use crate::MemoryResult;
use sqlx::{Row, PgPool};
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
/// Memory decay engine for managing importance and decay
#[derive(Debug)]
pub struct MemoryDecayEngine {
    db_pool: PgPool,
    config: DecayConfig,
    workspace_registry: Option<Arc<crate::workspace_registry::WorkspaceRegistry>>,
}

impl MemoryDecayEngine {
    /// Create a new decay engine
    pub async fn new(config: &DecayConfig, db_pool: PgPool) -> MemoryResult<Self> {
        Ok(Self {
            db_pool,
            config: config.clone(),
            workspace_registry: None,
        })
    }

    /// Create a new decay engine with workspace registry
    pub async fn new_with_workspace_registry(
        config: &DecayConfig,
        db_pool: PgPool,
        workspace_registry: Arc<crate::workspace_registry::WorkspaceRegistry>
    ) -> MemoryResult<Self> {
        Ok(Self {
            db_pool,
            config: config.clone(),
            workspace_registry: Some(workspace_registry),
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

        // Apply workspace-aware decay if registry is available
        if self.workspace_registry.is_some() {
            let workspace_decayed = self.apply_workspace_aware_decay(now).await?;
            total_updated += workspace_decayed;

            // Clean up unused workspaces
            let workspaces_cleaned = self.cleanup_unused_workspaces().await?;
            info!("Workspace cleanup: {} workspaces cleaned", workspaces_cleaned);
        }

        info!("Decay cycle completed: {} memories updated, {} boosted, {} consolidated",
              total_updated, boosted, consolidated);

        Ok(total_updated)
    }

    /// Apply workspace-aware decay based on workspace access patterns
    async fn apply_workspace_aware_decay(&self, now: DateTime<Utc>) -> MemoryResult<usize> {
        let registry = self.workspace_registry.as_ref().unwrap();
        let mut total_decayed = 0;

        // Get workspace access statistics
        let workspaces = registry.get_accessible_workspaces().await?;

        for workspace in workspaces {
            // Calculate workspace decay multiplier based on access patterns
            let workspace_decay_multiplier = self.calculate_workspace_decay_multiplier(&workspace, now);

            if workspace_decay_multiplier < 1.0 {
                // Apply accelerated decay to memories in infrequently accessed workspaces
                let updated = sqlx::query(
                    r#"
                    UPDATE memory_embeddings
                    SET decay_factor = GREATEST(
                        decay_factor * $2,
                        $3  -- minimum decay factor
                    )
                    WHERE workspace_id = $1
                      AND last_accessed < $4 - INTERVAL '24 hours'
                    "#,
                )
                .bind(workspace.id)
                .bind(workspace_decay_multiplier)
                .bind(self.config.minimum_memory_strength)
                .bind(now)
                .execute(&self.db_pool)
                .await?;

                total_decayed += updated.rows_affected() as usize;

                debug!("Applied workspace decay multiplier {:.2} to {} memories in workspace {}",
                        workspace_decay_multiplier, updated.rows_affected(), workspace.name);
            }
        }

        Ok(total_decayed)
    }

    /// Calculate decay multiplier based on workspace access patterns
    fn calculate_workspace_decay_multiplier(&self, workspace: &crate::memory_types::WorkspaceEntry, now: DateTime<Utc>) -> f64 {
        let duration_since_access = now.signed_duration_since(workspace.last_accessed);
        let hours_since_access = duration_since_access.num_hours() as f64;
        let access_frequency = workspace.access_count as f64;

        // Base decay: more aggressive for workspaces not accessed recently
        let base_decay = if hours_since_access < 24.0 {
            1.0 // No extra decay for recently accessed workspaces
        } else if hours_since_access < 168.0 {  // Week
            0.95 // Slight decay
        } else if hours_since_access < 720.0 {  // Month
            0.85 // Moderate decay
        } else {
            0.7 // Heavy decay for workspaces not accessed in a month
        };

        // Access frequency boost: frequently accessed workspaces decay slower
        let frequency_boost = if access_frequency > 100.0 {
            1.2 // High usage - slower decay
        } else if access_frequency > 50.0 {
            1.1 // Moderate usage - slight boost
        } else if access_frequency > 10.0 {
            1.0 // Normal decay
        } else {
            0.9 // Low usage - slightly faster decay
        };

        // Default workspaces get protection
        let default_protection = if workspace.is_default { 1.1 } else { 1.0 };

        f64::min(base_decay * frequency_boost * default_protection, 1.0f64)
    }

    /// Clean up workspaces that haven't been accessed for extended periods
    async fn cleanup_unused_workspaces(&self) -> MemoryResult<usize> {
        let registry = self.workspace_registry.as_ref().unwrap();
        let cutoff_date = Utc::now() - Duration::days(90); // 90 days of inactivity
        let mut cleaned_count = 0;

        // Find workspaces that haven't been accessed in 90+ days and aren't default
        let all_workspaces = registry.get_accessible_workspaces().await?;
        let unused_workspaces: Vec<_> = all_workspaces.into_iter()
            .filter(|w| w.last_accessed < cutoff_date && !w.is_default)
            .collect();

        for workspace in unused_workspaces {
            if !workspace.is_default {
                // Mark workspace as disabled
                registry.update_workspace_access(&workspace.id, crate::memory_types::WorkspaceAccess::Disabled).await?;

                // Aggressively decay memories in unused workspaces
                let updated = sqlx::query(
                    r#"
                    UPDATE memory_embeddings
                    SET decay_factor = GREATEST(decay_factor * 0.5, $2)
                    WHERE workspace_id = $1
                    "#,
                )
                .bind(workspace.id)
                .bind(self.config.minimum_memory_strength)
                .execute(&self.db_pool)
                .await?;

                cleaned_count += 1;

                info!("Cleaned up unused workspace '{}': {} memories decayed",
                      workspace.name, updated.rows_affected());
            }
        }

        Ok(cleaned_count)
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
        .execute(&self.db_pool)
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
        .execute(&self.db_pool)
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
        .execute(&self.db_pool)
        .await?;

        Ok(updated.rows_affected() as usize)
    }

    /// Apply custom decay formula (simplified implementation)
    async fn apply_custom_decay(&self, now: DateTime<Utc>, _formula: &str) -> MemoryResult<usize> {
        // PLACEHOLDER: Custom decay formula parsing and evaluation
        // Option 1: Use PostgreSQL's expression evaluation (requires careful validation to prevent SQL injection)
        //   - Use PostgreSQL's EXECUTE format() with validated formula
        //   - Validate formula syntax (whitelist allowed functions: importance_score, decay_factor, age_days, LN, EXP, etc.)
        //   - Prevent arbitrary SQL execution
        // Option 2: Use Rust formula parser library (e.g., fasteval, meval, or custom parser)
        //   - Parse mathematical expressions (e.g., "importance_score * (1 - decay_factor^2)")
        //   - Evaluate expressions with database values (importance_score, decay_factor, age_days)
        //   - Support time-based functions (e.g., age_days, last_accessed)
        //   - Validate formula syntax and safety (prevent SQL injection, infinite loops)
        // 
        // For now, fall back to exponential decay
        warn!("Custom decay formulas require formula parser dependency - using exponential decay fallback");
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
        .execute(&self.db_pool)
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
        .fetch_all(&self.db_pool)
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
            .execute(&self.db_pool)
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
        .execute(&self.db_pool)
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
        .execute(&self.db_pool)
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
        .fetch_one(&self.db_pool)
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
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

// Decay statistics
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
