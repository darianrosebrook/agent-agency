//! Context Management - Working memory and context folding
//!
//! This module implements the context preservation and folding functionality
//! from the context-preservation-engine, adapted for the agent memory system.
//! It handles working memory limits, automatic context folding, and retrieval.

use crate::types::*;
use crate::MemoryResult;
use crate::MemoryError;
use agent_agency_database::{DatabaseClient, DatabaseConfig, Row};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

/// Context management for working memory and folding
#[derive(Debug)]
pub struct ContextManager {
    db_client: Arc<DatabaseClient>,
    config: ContextConfig,
}

impl ContextManager {
    /// Create a new context manager
    pub async fn new(config: &ContextConfig) -> MemoryResult<Self> {
        let db_config = agent_agency_database::DatabaseConfig::default();
        let db_client = Arc::new(DatabaseClient::new(db_config).await?);

        Ok(Self {
            db_client,
            config: config.clone(),
        })
    }

    /// Manage context lifecycle - fold old contexts, maintain working set
    pub async fn manage_context_lifecycle(&self, context_id: &str) -> MemoryResult<()> {
        // Check if context should be folded
        if self.should_fold_context(context_id).await? {
            self.fold_context(context_id).await?;
        }

        // Maintain working memory limits
        self.enforce_working_memory_limits().await?;

        Ok(())
    }

    /// Determine if a context should be folded based on age and importance
    async fn should_fold_context(&self, context_id: &str) -> MemoryResult<bool> {
        // Get context age and access patterns
        let context_age = self.get_context_age(context_id).await?;
        let access_frequency = self.get_access_frequency(context_id).await?;
        let importance_score = self.get_context_importance(context_id).await?;

        // Folding decision based on v4 context folding strategy
        let should_fold = if context_age > Duration::hours(4) {
            // Old contexts get folded
            true
        } else if context_age > Duration::hours(1) && access_frequency < 0.3 {
            // Moderately old, low access contexts get folded
            true
        } else if importance_score < 0.5 {
            // Low importance contexts get folded even if recent
            true
        } else {
            false
        };

        if should_fold {
            debug!("Context {} should be folded (age: {:?}, access: {:.2}, importance: {:.2})",
                   context_id, context_age, access_frequency, importance_score);
        }

        Ok(should_fold)
    }

    /// Fold a context using the configured strategy
    pub async fn fold_context(&self, context_id: &str) -> MemoryResult<FoldedContext> {
        let context = self.retrieve_full_context(context_id).await?;

        let folded = match self.config.offload_strategy {
            OffloadStrategy::Compress => self.compress_context(context).await,
            OffloadStrategy::Summarize => self.summarize_context(context).await,
            OffloadStrategy::Archive => self.archive_context(context).await,
            OffloadStrategy::Delete => Ok(FoldedContext::Deleted),
        };

        // Store folded context
        if let Ok(folded_context) = &folded {
            self.store_folded_context(context_id, folded_context).await?;
            self.update_context_metadata(context_id, folded_context).await?;
        }

        folded
    }

    /// Compress context using gzip
    async fn compress_context(&self, context: TaskContext) -> MemoryResult<FoldedContext> {
        let json_data = serde_json::to_string(&context)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        std::io::copy(&mut json_data.as_bytes(), &mut encoder)?;
        let compressed = encoder.finish()?;

        let compressed_size = compressed.len();
        let compression_ratio = json_data.len() as f32 / compressed_size as f32;

        Ok(FoldedContext::Compressed {
            data: compressed.clone(),
            original_size: json_data.len(),
            compressed_size,
            compression_ratio,
        })
    }

    /// Summarize context for long-term storage
    async fn summarize_context(&self, context: TaskContext) -> MemoryResult<FoldedContext> {
        // Create a summary of the context
        let summary = ContextSummary {
            task_type: context.task_type,
            description: context.description.chars().take(200).collect(),
            domain: context.domain,
            entity_count: context.entities.len(),
            temporal_range: context.temporal_context.as_ref().map(|tc| {
                TemporalRange {
                    start: tc.start_time,
                    end: tc.deadline.unwrap_or(tc.start_time + Duration::hours(1)),
                }
            }),
            key_entities: context.entities.into_iter().take(5).collect(),
            summary_created: Utc::now(),
        };

        Ok(FoldedContext::Summarized(summary))
    }

    /// Archive context for long-term storage
    async fn archive_context(&self, context: TaskContext) -> MemoryResult<FoldedContext> {
        let archived = ArchivedContext {
            context,
            archived_at: Utc::now(),
            access_count: 0,
            last_accessed: None,
            retention_policy: RetentionPolicy::LongTerm,
        };

        Ok(FoldedContext::Archived(archived))
    }

    /// Retrieve and reconstruct a folded context
    pub async fn reconstruct_context(&self, context_id: &str) -> MemoryResult<TaskContext> {
        let folded = self.retrieve_folded_context(context_id).await?;

        match folded {
            FoldedContext::Compressed { data, original_size, .. } => {
                let mut decoder = GzDecoder::new(&data[..]);
                let mut decompressed = Vec::new();
                std::io::copy(&mut decoder, &mut decompressed)?;
                let decompressed_str = String::from_utf8(decompressed)?;
                let context: TaskContext = serde_json::from_str(&decompressed_str)?;
                Ok(context)
            }
            FoldedContext::Summarized(summary) => {
                // Reconstruct a minimal context from summary
                Ok(TaskContext {
                    task_id: context_id.to_string(),
                    task_type: summary.task_type,
                    description: summary.description,
                    domain: summary.domain,
                    entities: summary.key_entities,
                    temporal_context: summary.temporal_range.map(|tr| TemporalContext {
                        start_time: tr.start,
                        deadline: Some(tr.end),
                        priority: TaskPriority::Medium, // Default
                        recurrence_pattern: None,
                    }),
                    metadata: HashMap::new(),
                })
            }
            FoldedContext::Archived(archived) => {
                // Update access statistics
                self.update_archived_access(&archived, context_id).await?;
                Ok(archived.context)
            }
            FoldedContext::Deleted => {
                Err(MemoryError::NotFound(format!("Context {} has been deleted", context_id)))
            }
        }
    }

    /// Enforce working memory limits
    async fn enforce_working_memory_limits(&self) -> MemoryResult<usize> {
        // Get current working memory contexts
        let working_contexts = self.get_working_memory_contexts().await?;

        if working_contexts.len() <= 10 { // Default working memory limit
            return Ok(0);
        }

        // Sort by access recency and importance and filter for folding
        let mut contexts_to_fold = Vec::new();
        for ctx in working_contexts {
            if self.should_fold_context(&ctx.task_id).await.unwrap_or(false) {
                contexts_to_fold.push(ctx);
            }
        }

        contexts_to_fold.sort_by(|a, b| {
            // Sort by importance (lower first) then by access time (older first)
            a.metadata.get("importance_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5)
                .partial_cmp(&b.metadata.get("importance_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5))
                .unwrap()
        });

        // Fold oldest/lowest importance contexts
        let mut folded_count = 0;
        for context in contexts_to_fold.into_iter().take(5) { // Fold up to 5 at a time
            if let Ok(_) = self.fold_context(&context.task_id).await {
                folded_count += 1;
            }
        }

        if folded_count > 0 {
            info!("Folded {} contexts to maintain working memory limits", folded_count);
        }

        Ok(folded_count)
    }

    // Helper methods for context metadata
    async fn get_context_age(&self, context_id: &str) -> MemoryResult<Duration> {
        // Query database for context creation time
        // This would be implemented with actual DB queries
        Ok(Duration::hours(2)) // Placeholder
    }

    async fn get_access_frequency(&self, _context_id: &str) -> MemoryResult<f32> {
        // Calculate access frequency over time window
        Ok(0.5) // Placeholder
    }

    async fn get_context_importance(&self, _context_id: &str) -> MemoryResult<f32> {
        // Get importance score from metadata
        Ok(0.7) // Placeholder
    }

    async fn retrieve_full_context(&self, _context_id: &str) -> MemoryResult<TaskContext> {
        // Retrieve full context from storage
        // This would be implemented with actual DB queries
        Err(MemoryError::NotFound("Context not found".to_string())) // Placeholder
    }

    async fn store_folded_context(&self, _context_id: &str, _folded: &FoldedContext) -> MemoryResult<()> {
        // Store folded context in database
        Ok(()) // Placeholder
    }

    async fn update_context_metadata(&self, _context_id: &str, _folded: &FoldedContext) -> MemoryResult<()> {
        // Update context metadata with folding information
        Ok(()) // Placeholder
    }

    async fn retrieve_folded_context(&self, _context_id: &str) -> MemoryResult<FoldedContext> {
        // Retrieve folded context from storage
        Err(MemoryError::NotFound("Folded context not found".to_string())) // Placeholder
    }

    async fn update_archived_access(&self, _archived: &ArchivedContext, _context_id: &str) -> MemoryResult<()> {
        // Update access statistics for archived context
        Ok(()) // Placeholder
    }

    async fn get_working_memory_contexts(&self) -> MemoryResult<Vec<TaskContext>> {
        // Get all contexts currently in working memory
        Ok(vec![]) // Placeholder
    }
}

/// Folded context representations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FoldedContext {
    /// Gzip-compressed full context
    Compressed {
        data: Vec<u8>,
        original_size: usize,
        compressed_size: usize,
        compression_ratio: f32,
    },
    /// Summarized context for quick access
    Summarized(ContextSummary),
    /// Archived full context for long-term storage
    Archived(ArchivedContext),
    /// Context deleted (no longer available)
    Deleted,
}

/// Context summary for folded contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub task_type: String,
    pub description: String,
    pub domain: Vec<String>,
    pub entity_count: usize,
    pub temporal_range: Option<TemporalRange>,
    pub key_entities: Vec<String>,
    pub summary_created: DateTime<Utc>,
}

/// Archived context with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedContext {
    pub context: TaskContext,
    pub archived_at: DateTime<Utc>,
    pub access_count: u32,
    pub last_accessed: Option<DateTime<Utc>>,
    pub retention_policy: RetentionPolicy,
}

/// Temporal range for context summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Retention policy for archived contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    /// Keep indefinitely
    Permanent,
    /// Keep for specified period
    LongTerm,
    /// Keep until manually deleted
    Manual,
    /// Auto-delete after period
    Temporary,
}
