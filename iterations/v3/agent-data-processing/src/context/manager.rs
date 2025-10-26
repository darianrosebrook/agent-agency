//! Context Manager - Unified context preservation and working memory management
//!
//! This module combines the functionality from:
//! - context-preservation-engine (multi-tenant, full-featured)
//! - agent-memory (working memory folding)
//!
//! Provides a unified interface for context lifecycle management.

use crate::context::types::*;
use crate::DataProcessingResult;
use agent_agency_database::{DatabaseClient, DatabaseConfig};
use chrono::{DateTime, Utc, Duration};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Unified context manager for preservation and working memory
#[derive(Debug)]
pub struct ContextManager {
    /// Database client
    db_client: Arc<DatabaseClient>,
    /// Configuration
    config: ContextConfig,
    /// Working memory cache
    working_memory: Arc<RwLock<HashMap<Uuid, ContextData>>>,
    /// Statistics
    stats: Arc<RwLock<ContextStats>>,
}

impl ContextManager {
    /// Create a new unified context manager
    pub async fn new(config: ContextConfig) -> DataProcessingResult<Self> {
        let db_config = DatabaseConfig::default();
        let db_client = Arc::new(DatabaseClient::new(db_config).await?);

        let stats = Arc::new(RwLock::new(ContextStats {
            total_contexts: 0,
            total_storage_size: 0,
            working_memory_contexts: 0,
            folded_contexts: 0,
            average_context_size: 0,
            recent_accesses: 0,
            oldest_context_age_hours: 0,
            compression_ratio: 1.0,
        }));

        let manager = Self {
            db_client,
            config,
            working_memory: Arc::new(RwLock::new(HashMap::new())),
            stats,
        };

        // Initialize working memory cleanup task
        manager.start_cleanup_task();

        Ok(manager)
    }

    /// Preserve context data
    pub async fn preserve_context(
        &self,
        request: ContextPreservationRequest,
    ) -> DataProcessingResult<ContextPreservationResult> {
        let start_time = tokio::time::Instant::now();

        info!("Preserving context: {}", request.context_data.id);

        // Check storage limits
        self.check_storage_limits(&request.context_data).await?;

        // Store context
        let context_id = request.context_data.id;
        let size_bytes = self.calculate_context_size(&request.context_data);

        // Store in database
        self.store_context_in_db(&request.context_data).await?;

        // Add to working memory if enabled
        if self.config.working_memory.max_size > 0 {
            self.add_to_working_memory(request.context_data).await?;
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Update statistics
        self.update_stats_after_preservation(size_bytes).await?;

        Ok(ContextPreservationResult {
            success: true,
            context_id: Some(context_id),
            processing_time_ms: processing_time,
            processed_size_bytes: size_bytes,
            error_message: None,
        })
    }

    /// Retrieve context data
    pub async fn retrieve_context(
        &self,
        request: ContextRetrievalRequest,
    ) -> DataProcessingResult<ContextRetrievalResult> {
        let start_time = tokio::time::Instant::now();

        info!("Retrieving context: {}", request.context_id);

        // Check working memory first
        if let Some(context) = self.get_from_working_memory(&request.context_id).await? {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Update access statistics
            self.update_context_access(&request.context_id).await?;

            return Ok(ContextRetrievalResult {
                success: true,
                context_data: Some(context),
                processing_time_ms: processing_time,
                error_message: None,
            });
        }

        // Retrieve from database
        match self.retrieve_context_from_db(&request.context_id).await? {
            Some(mut context) => {
                // Update access time
                context.last_accessed_at = Utc::now();
                context.access_count += 1;

                // Store back to database
                self.update_context_in_db(&context).await?;

                // Add to working memory
                if self.config.working_memory.max_size > 0 {
                    self.add_to_working_memory(context.clone()).await?;
                }

                let processing_time = start_time.elapsed().as_millis() as u64;

                Ok(ContextRetrievalResult {
                    success: true,
                    context_data: Some(context),
                    processing_time_ms: processing_time,
                    error_message: None,
                })
            }
            None => Ok(ContextRetrievalResult {
                success: false,
                context_data: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Context not found: {}", request.context_id)),
            }),
        }
    }

    /// Fold context based on lifecycle rules
    pub async fn fold_context(&self, context_id: &Uuid) -> DataProcessingResult<FoldedContext> {
        info!("Folding context: {}", context_id);

        // Get context
        let context = match self.retrieve_context_from_db(context_id).await? {
            Some(ctx) => ctx,
            None => return Err(crate::DataProcessingError::Other(format!("Context not found: {}", context_id))),
        };

        // Determine folding strategy
        let strategy = self.determine_folding_strategy(&context).await?;

        let folded = match strategy {
            FoldingStrategy::Compress => self.compress_context(context).await,
            FoldingStrategy::Summarize => self.summarize_context(context).await,
            FoldingStrategy::Archive => self.archive_context(context).await,
            FoldingStrategy::Delete => Ok(FoldedContext::Deleted),
        };

        // Store folded result
        if let Ok(folded_context) = &folded {
            self.store_folded_context(context_id, folded_context).await?;
        }

        folded
    }

    /// Manage context lifecycle - automatic folding and cleanup
    pub async fn manage_context_lifecycle(&self) -> DataProcessingResult<()> {
        info!("Running context lifecycle management");

        // Find contexts that need folding
        let contexts_to_fold = self.find_contexts_needing_folding().await?;

        for context_id in contexts_to_fold {
            if let Err(e) = self.fold_context(&context_id).await {
                warn!("Failed to fold context {}: {}", context_id, e);
            }
        }

        // Clean up working memory
        self.cleanup_working_memory().await?;

        // Update statistics
        self.update_lifecycle_stats().await?;

        Ok(())
    }

    /// Get context statistics
    pub async fn get_stats(&self) -> DataProcessingResult<ContextStats> {
        let stats = self.stats.read().await.clone();
        Ok(stats)
    }

    // Private helper methods

    async fn check_storage_limits(&self, context: &ContextData) -> DataProcessingResult<()> {
        let size_bytes = self.calculate_context_size(context);

        // Check max context size
        if size_bytes > self.config.storage.max_context_size {
            return Err(crate::DataProcessingError::ResourceExhausted(
                format!("Context size {} exceeds limit {}", size_bytes, self.config.storage.max_context_size)
            ));
        }

        // Check total storage usage
        let current_usage = self.get_current_storage_usage().await?;
        if current_usage + size_bytes > self.config.storage.cache_size_limit {
            return Err(crate::DataProcessingError::ResourceExhausted(
                "Storage limit exceeded".to_string()
            ));
        }

        Ok(())
    }

    async fn store_context_in_db(&self, context: &ContextData) -> DataProcessingResult<()> {
        // TODO: Implement database storage
        // This would use the database client to store the context
        debug!("Storing context {} in database", context.id);
        Ok(())
    }

    async fn retrieve_context_from_db(&self, context_id: &Uuid) -> DataProcessingResult<Option<ContextData>> {
        // TODO: Implement database retrieval
        debug!("Retrieving context {} from database", context_id);
        Ok(None)
    }

    async fn update_context_in_db(&self, context: &ContextData) -> DataProcessingResult<()> {
        // TODO: Implement database update
        debug!("Updating context {} in database", context.id);
        Ok(())
    }

    async fn update_context_access(&self, context_id: &Uuid) -> DataProcessingResult<()> {
        // TODO: Update access statistics
        debug!("Updating access stats for context {}", context_id);
        Ok(())
    }

    async fn store_folded_context(&self, context_id: &Uuid, folded: &FoldedContext) -> DataProcessingResult<()> {
        // TODO: Store folded context
        debug!("Storing folded context {}", context_id);
        Ok(())
    }

    async fn add_to_working_memory(&self, context: ContextData) -> DataProcessingResult<()> {
        let mut working_memory = self.working_memory.write().await;

        // Check working memory limits
        if working_memory.len() >= self.config.working_memory.max_size {
            // Remove least recently accessed context
            if let Some((oldest_id, _)) = working_memory
                .iter()
                .min_by_key(|(_, ctx)| ctx.last_accessed_at) {
                let oldest_id = *oldest_id;
                working_memory.remove(&oldest_id);
                debug!("Removed context {} from working memory due to size limit", oldest_id);
            }
        }

        working_memory.insert(context.id, context);
        Ok(())
    }

    async fn get_from_working_memory(&self, context_id: &Uuid) -> DataProcessingResult<Option<ContextData>> {
        let mut working_memory = self.working_memory.write().await;

        if let Some(context) = working_memory.get_mut(context_id) {
            context.last_accessed_at = Utc::now();
            context.access_count += 1;
            return Ok(Some(context.clone()));
        }

        Ok(None)
    }

    async fn cleanup_working_memory(&self) -> DataProcessingResult<()> {
        let mut working_memory = self.working_memory.write().await;
        let mut to_remove = Vec::new();

        for (id, context) in working_memory.iter() {
            // Remove contexts older than retention period
            let age = Utc::now().signed_duration_since(context.last_accessed_at);
            if age > Duration::hours(self.config.storage.retention_hours as i64) {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            working_memory.remove(&id);
            debug!("Removed expired context {} from working memory", id);
        }

        Ok(())
    }

    async fn find_contexts_needing_folding(&self) -> DataProcessingResult<Vec<Uuid>> {
        // TODO: Query database for contexts needing folding
        // Based on age, access frequency, importance
        Ok(Vec::new())
    }

    async fn determine_folding_strategy(&self, context: &ContextData) -> DataProcessingResult<FoldingStrategy> {
        let age_hours = Utc::now().signed_duration_since(context.created_at).num_hours() as u32;
        let access_frequency = if context.access_count > 0 {
            let age_days = age_hours as f64 / 24.0;
            context.access_count as f64 / age_days
        } else {
            0.0
        };

        let importance_score = context.metadata.importance_score.unwrap_or(0.5);

        // Folding decision logic
        if age_hours >= self.config.folding.age_threshold_hours {
            Ok(self.config.folding.strategy.clone())
        } else if age_hours >= 1 && access_frequency < self.config.folding.access_frequency_threshold {
            Ok(FoldingStrategy::Compress)
        } else if importance_score < self.config.folding.importance_threshold {
            Ok(FoldingStrategy::Compress)
        } else {
            Ok(FoldingStrategy::Compress) // Default to compression for now
        }
    }

    async fn compress_context(&self, context: ContextData) -> DataProcessingResult<FoldedContext> {
        let json_data = serde_json::to_string(&context)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.config.storage.compression_level));
        encoder.write_all(json_data.as_bytes())?;
        let compressed = encoder.finish()?;

        Ok(FoldedContext::Compressed(compressed))
    }

    async fn summarize_context(&self, _context: ContextData) -> DataProcessingResult<FoldedContext> {
        // TODO: Implement AI-powered summarization
        // For now, return a simple summary
        Ok(FoldedContext::Summarized("Context summarized".to_string()))
    }

    async fn archive_context(&self, _context: ContextData) -> DataProcessingResult<FoldedContext> {
        // TODO: Implement archiving to cold storage
        Ok(FoldedContext::Archived("archived_location".to_string()))
    }

    fn calculate_context_size(&self, context: &ContextData) -> u64 {
        // Rough calculation - could be more accurate
        serde_json::to_string(context)
            .map(|s| s.len() as u64)
            .unwrap_or(1024)
    }

    async fn get_current_storage_usage(&self) -> DataProcessingResult<u64> {
        // TODO: Query database for current storage usage
        Ok(0)
    }

    async fn update_stats_after_preservation(&self, size_bytes: u64) -> DataProcessingResult<()> {
        let mut stats = self.stats.write().await;
        stats.total_contexts += 1;
        stats.total_storage_size += size_bytes;

        if stats.total_contexts > 0 {
            stats.average_context_size = stats.total_storage_size / stats.total_contexts;
        }

        Ok(())
    }

    async fn update_lifecycle_stats(&self) -> DataProcessingResult<()> {
        // TODO: Update comprehensive statistics
        Ok(())
    }

    fn start_cleanup_task(&self) {
        let config = self.config.clone();
        let working_memory = Arc::clone(&self.working_memory);
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(config.working_memory.cleanup_interval_minutes as u64 * 60)
            );

            loop {
                interval.tick().await;

                // Cleanup working memory
                let mut memory = working_memory.write().await;
                let mut to_remove = Vec::new();

                for (id, context) in memory.iter() {
                    let age = Utc::now().signed_duration_since(context.last_accessed_at);
                    if age > Duration::hours(config.storage.retention_hours as i64) {
                        to_remove.push(*id);
                    }
                }

                let removed_count = to_remove.len();
                for id in to_remove {
                    memory.remove(&id);
                }

                if removed_count > 0 {
                    debug!("Cleaned up {} expired contexts from working memory", removed_count);
                }

                // Update stats
                let mut stats_guard = stats.write().await;
                stats_guard.working_memory_contexts = memory.len();
            }
        });
    }
}
