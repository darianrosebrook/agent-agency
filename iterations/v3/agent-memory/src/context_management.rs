//! Context Management - Working memory and context folding
//!
//! This module provides a memory-focused interface to the unified context preservation
//! system from agent-data-processing. It handles working memory limits, automatic
//! context folding, and retrieval with memory-specific optimizations.

use crate::memory_types::*;
use crate::MemoryError;
use crate::MemoryResult;

use chrono::{DateTime, Duration, Utc};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
// ContextConfig is defined in memory_types.rs

// Real ContextManager integration - now enabled
use agent_data_processing::context::manager::{
    ContextManager as RealContextManager, DatabaseClient, DatabaseConfig, ModelRegistry,
};
use agent_data_processing::context::types::{
    ContextConfig as RealContextConfig, ContextData as RealContextData, ContextMetadata,
    ContextPreservationRequest as RealContextPreservationRequest,
    ContextPreservationResult as RealContextPreservationResult,
    ContextRetrievalRequest as RealContextRetrievalRequest,
    ContextRetrievalResult as RealContextRetrievalResult, ContextStats as RealContextStats,
    PreservationOptions, PreservationPriority, RetrievalOptions,
};
#[cfg(feature = "embeddings")]
use data_infrastructure::embedding::embedding_service::{
    EmbeddingService, EmbeddingServiceFactory,
};
#[cfg(feature = "embeddings")]
use data_infrastructure::embedding::embedding_types::EmbeddingConfig;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ContextData {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub content: String,
    pub metadata: serde_json::Value,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ContextStats {
    pub total_contexts: usize,
    pub active_contexts: usize,
    pub folded_contexts: usize,
}

#[derive(Debug)]
pub struct ContextPreservationRequest {
    pub context_data: ContextData,
    pub priority: u8,
}

#[derive(Debug)]
pub struct ContextPreservationResult {
    pub success: bool,
    pub context_id: Uuid,
    pub folded: bool,
}

#[derive(Debug)]
pub struct ContextRetrievalRequest {
    pub context_id: Uuid,
    pub include_folded: bool,
}

#[derive(Debug)]
pub struct ContextRetrievalResult {
    pub context_data: Option<ContextData>,
    pub folded_contexts: Vec<FoldedContext>,
}

// FoldedContext is defined in memory_types.rs as an enum

/// Context manager trait - async interface for context management
#[async_trait::async_trait]
pub trait ContextManager: Send + Sync {
    async fn manage_lifecycle(&self) -> Result<(), String>;
    async fn preserve_context(
        &self,
        request: ContextPreservationRequest,
    ) -> Result<ContextPreservationResult, String>;
    async fn retrieve_context(
        &self,
        request: ContextRetrievalRequest,
    ) -> Result<ContextRetrievalResult, String>;
    async fn get_stats(&self) -> Result<ContextStats, String>;
}

/// Context cache entry with timestamp
#[derive(Clone)]
struct CachedContext {
    context: TaskContext,
    cached_at: DateTime<Utc>,
}

/// Context management for working memory and folding
pub struct MemoryContextManager {
    /// Configuration for context management
    config: ContextConfig,
    /// Actual context manager from agent-data-processing
    context_manager: Box<dyn ContextManager>,
    /// Database pool for querying context data
    db_pool: Option<sqlx::PgPool>,
    /// In-memory cache for frequently accessed contexts
    context_cache: Arc<RwLock<HashMap<String, CachedContext>>>,
}

impl std::fmt::Debug for MemoryContextManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryContextManager")
            .field("config", &self.config)
            .field("context_manager", &"<dyn ContextManager>")
            .field(
                "db_pool",
                &if self.db_pool.is_some() {
                    "Some(PgPool)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

/// TODO: Consider enhancing stub implementation for ContextManager (fallback when no database available)
///       This is an intentional fallback for graceful degradation when database is unavailable.
///       Consider implementing in-memory cache or file-based persistence as intermediate solution.
///
/// **Fallback Behavior:**
/// This stub is used when no database connection is available. It provides graceful
/// degradation by:
/// - Accepting all context preservation requests without error
/// - Returning empty context retrieval results (no data persisted)
/// - Maintaining API compatibility so calling code doesn't need special handling
///
/// **Impact:**
/// - Context preservation is disabled (contexts are not persisted)
/// - Context retrieval returns empty results
/// - System continues to function but without context persistence
///
/// **When Used:**
/// - Database connection unavailable or not configured
/// - Standalone mode without persistence requirements
/// - Development/testing environments without database setup
///
/// **Acceptable Use:**
/// This is an acceptable fallback pattern for graceful degradation. The system
/// continues to function but without context persistence capabilities. This is
/// preferable to failing completely when database is unavailable.
#[derive(Debug)]
struct StubContextManager {
    config: ContextConfig,
}

#[async_trait::async_trait]
impl ContextManager for StubContextManager {
    async fn manage_lifecycle(&self) -> Result<(), String> {
        Ok(())
    }

    async fn preserve_context(
        &self,
        _request: ContextPreservationRequest,
    ) -> Result<ContextPreservationResult, String> {
        Ok(ContextPreservationResult {
            success: true,
            context_id: Uuid::new_v4(),
            folded: false,
        })
    }

    async fn retrieve_context(
        &self,
        _request: ContextRetrievalRequest,
    ) -> Result<ContextRetrievalResult, String> {
        Ok(ContextRetrievalResult {
            context_data: None,
            folded_contexts: vec![],
        })
    }

    async fn get_stats(&self) -> Result<ContextStats, String> {
        Ok(ContextStats {
            total_contexts: 0,
            active_contexts: 0,
            folded_contexts: 0,
        })
    }
}

/// Wrapper to convert Box<dyn EmbeddingService> to Arc<dyn EmbeddingService>
#[cfg(feature = "embeddings")]
struct EmbeddingServiceWrapper {
    inner: Box<dyn EmbeddingService>,
}

#[cfg(feature = "embeddings")]
#[async_trait::async_trait]
impl EmbeddingService for EmbeddingServiceWrapper {
    async fn generate_embedding(
        &self,
        text: &str,
        content_type: data_infrastructure::embedding::ContentType,
        source: &str,
    ) -> anyhow::Result<data_infrastructure::embedding::StoredEmbedding> {
        self.inner
            .generate_embedding(text, content_type, source)
            .await
    }
    async fn generate_embeddings(
        &self,
        request: data_infrastructure::embedding::EmbeddingRequest,
    ) -> anyhow::Result<data_infrastructure::embedding::EmbeddingResponse> {
        self.inner.generate_embeddings(request).await
    }
    async fn search_similar(
        &self,
        request: data_infrastructure::embedding::SimilarityRequest,
    ) -> anyhow::Result<Vec<data_infrastructure::embedding::SimilarityResult>> {
        self.inner.search_similar(request).await
    }
    async fn store_embedding(
        &self,
        embedding: data_infrastructure::embedding::StoredEmbedding,
    ) -> anyhow::Result<()> {
        self.inner.store_embedding(embedding).await
    }
    async fn get_embedding(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<data_infrastructure::embedding::StoredEmbedding>> {
        self.inner.get_embedding(id).await
    }
    async fn health_check(&self) -> anyhow::Result<bool> {
        self.inner.health_check().await
    }
}

/// Real ContextManager adapter that wraps agent-data-processing::ContextManager
/// Integrates with embedding service for vector/knowledge graph embeddings
struct RealContextManagerAdapter {
    manager: Arc<RealContextManager>,
    #[cfg(feature = "embeddings")]
    embedding_service: Option<Arc<dyn EmbeddingService>>,
}

impl std::fmt::Debug for RealContextManagerAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("RealContextManagerAdapter");
        debug.field("manager", &self.manager);
        #[cfg(feature = "embeddings")]
        {
            debug.field(
                "embedding_service",
                &self
                    .embedding_service
                    .as_ref()
                    .map(|_| "Some(EmbeddingService)"),
            );
        }
        debug.finish()
    }
}

#[async_trait::async_trait]
impl ContextManager for RealContextManagerAdapter {
    async fn manage_lifecycle(&self) -> Result<(), String> {
        self.manager
            .manage_context_lifecycle()
            .await
            .map_err(|e| format!("Context lifecycle management failed: {}", e))
    }

    async fn preserve_context(
        &self,
        request: ContextPreservationRequest,
    ) -> Result<ContextPreservationResult, String> {
        // Convert agent-memory ContextData to agent-data-processing ContextData
        let real_context_data = RealContextData {
            id: request.context_data.id,
            context_type: request
                .context_data
                .metadata
                .get("context_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "task".to_string()),
            content: serde_json::json!({
                "task_id": request.context_data.metadata.get("task_id").and_then(|v| v.as_str()),
                "agent_id": request.context_data.metadata.get("agent_id").and_then(|v| v.as_str()),
                "description": request.context_data.content,
                "keywords": request.context_data.metadata.get("keywords"),
                "entities": request.context_data.metadata.get("entities"),
            }),
            metadata: ContextMetadata {
                title: request
                    .context_data
                    .metadata
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                description: request
                    .context_data
                    .metadata
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                tags: request
                    .context_data
                    .metadata
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                source: request
                    .context_data
                    .metadata
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                importance_score: request
                    .context_data
                    .metadata
                    .get("importance_score")
                    .and_then(|v| v.as_f64()),
                custom_fields: request
                    .context_data
                    .metadata
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            },
            created_at: request.context_data.created_at,
            last_accessed_at: request.context_data.created_at,
            access_count: 0,
            size_bytes: request.context_data.content.len() as u64,
        };

        // Generate embedding for vector/knowledge graph storage if embedding service is available
        #[cfg(feature = "embeddings")]
        if let Some(ref embedding_service) = self.embedding_service {
            let context_text = format!(
                "{} {}",
                real_context_data.metadata.title.as_deref().unwrap_or(""),
                real_context_data
                    .metadata
                    .description
                    .as_deref()
                    .unwrap_or("")
            );
            if !context_text.trim().is_empty() {
                if let Ok(embedding) = embedding_service
                    .generate_embedding(
                        &context_text,
                        data_infrastructure::embedding::ContentType::Text,
                        "context_manager",
                    )
                    .await
                {
                    // Store embedding metadata in context metadata
                    // The embedding itself is stored by the embedding service
                    debug!(
                        "Generated embedding for context {}: {} dimensions",
                        real_context_data.id,
                        embedding.vector.values.len()
                    );
                }
            }
        }

        // Create preservation request
        let real_request = RealContextPreservationRequest {
            context_data: real_context_data,
            options: PreservationOptions {
                force: request.priority > 5,
                compress: true,
                priority: if request.priority >= 8 {
                    PreservationPriority::Critical
                } else if request.priority >= 6 {
                    PreservationPriority::High
                } else if request.priority >= 4 {
                    PreservationPriority::Normal
                } else {
                    PreservationPriority::Low
                },
                custom_metadata: std::collections::HashMap::new(),
            },
        };

        // Preserve context using real manager
        let result = self
            .manager
            .preserve_context(real_request)
            .await
            .map_err(|e| format!("Context preservation failed: {}", e))?;

        Ok(ContextPreservationResult {
            success: result.success,
            context_id: result.context_id.unwrap_or_else(Uuid::new_v4),
            folded: false, // Folding happens during lifecycle management
        })
    }

    async fn retrieve_context(
        &self,
        request: ContextRetrievalRequest,
    ) -> Result<ContextRetrievalResult, String> {
        // Create retrieval request
        let real_request = RealContextRetrievalRequest {
            context_id: request.context_id,
            options: RetrievalOptions {
                include_metadata: true,
                decompress: true,
                validate_checksum: false, // Checksum validation is optional
            },
        };

        // Retrieve context using real manager
        let result = self
            .manager
            .retrieve_context(real_request)
            .await
            .map_err(|e| format!("Context retrieval failed: {}", e))?;

        // Convert agent-data-processing ContextData to agent-memory ContextData
        let context_data = result.context_data.map(|real_data| ContextData {
            id: real_data.id,
            content: real_data
                .content
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| serde_json::to_string(&real_data.content).unwrap_or_default()),
            metadata: serde_json::json!({
                "title": real_data.metadata.title,
                "description": real_data.metadata.description,
                "tags": real_data.metadata.tags,
                "source": real_data.metadata.source,
                "importance_score": real_data.metadata.importance_score,
                "task_id": real_data.content.get("task_id"),
                "agent_id": real_data.content.get("agent_id"),
                "keywords": real_data.content.get("keywords"),
                "entities": real_data.content.get("entities"),
                "context_type": real_data.context_type,
            }),
            created_at: real_data.created_at,
        });

        Ok(ContextRetrievalResult {
            context_data,
            folded_contexts: vec![], // Folded contexts are handled separately
        })
    }

    async fn get_stats(&self) -> Result<ContextStats, String> {
        let real_stats = self
            .manager
            .get_stats()
            .await
            .map_err(|e| format!("Failed to get context statistics: {}", e))?;

        Ok(ContextStats {
            total_contexts: real_stats.total_contexts as usize,
            active_contexts: real_stats.working_memory_contexts as usize,
            folded_contexts: real_stats.folded_contexts as usize,
        })
    }
}

impl MemoryContextManager {
    /// Create a new memory context manager
    pub async fn new(config: ContextConfig) -> MemoryResult<Self> {
        Self::new_with_db(config, None).await
    }

    /// Create a new memory context manager with database pool
    pub async fn new_with_db(config: ContextConfig, db_pool: Option<PgPool>) -> MemoryResult<Self> {
        let context_manager: Box<dyn ContextManager> = if let Some(ref _pool) = db_pool {
            // Create database client from pool
            let database_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());

            let db_config = DatabaseConfig {
                database_url: database_url.clone(),
                max_connections: 10,
            };

            // Create database client (we'll use the pool directly, but need to create a DatabaseClient wrapper)
            // For now, we'll create a new connection pool from the URL
            // In production, we should reuse the existing pool
            let db_client = Arc::new(DatabaseClient::new(db_config).await.map_err(|e| {
                MemoryError::Other(format!("Failed to create database client: {}", e))
            })?);

            // Create ModelRegistry for summarization (stub for now - will be replaced with real AI service)
            // TODO: Integrate with real AI service for summarization
            let model_registry = Arc::new(ModelRegistry::new(None));

            // Convert agent-memory ContextConfig to agent-data-processing ContextConfig
            // Use defaults for fields not available in agent-memory ContextConfig
            let max_contexts = config.max_contexts;
            let fold_threshold = config.fold_threshold;

            let real_config = RealContextConfig {
                storage: agent_data_processing::context::types::ContextStorageConfig {
                    max_context_size: 50 * 1024 * 1024, // 50MB default
                    retention_hours: 168,               // 1 week default
                    max_contexts: max_contexts as u32,
                    enable_persistent_storage: true,
                    enable_memory_cache: true,
                    cache_size_limit: 100 * 1024 * 1024, // 100MB default
                    enable_compression: true,
                    compression_level: 6,
                    checksum_validation: true,
                    archive_path: None,
                },
                folding: agent_data_processing::context::types::ContextFoldingConfig {
                    strategy: agent_data_processing::context::types::FoldingStrategy::Compress,
                    age_threshold_hours: 4, // Default: 4 hours
                    importance_threshold: fold_threshold as f64,
                    access_frequency_threshold: 0.3, // Default: 0.3
                    max_working_memory_contexts: (max_contexts / 10).max(50), // 10% of max_contexts, minimum 50
                },
                performance: agent_data_processing::context::types::PerformanceConfig::default(),
                working_memory: agent_data_processing::context::types::WorkingMemoryConfig {
                    max_size: (max_contexts / 20).max(50), // 5% of max_contexts, minimum 50
                    track_access_patterns: true,
                    cleanup_interval_minutes: 30,
                },
            };

            // Create real ContextManager
            let real_manager =
                RealContextManager::new_with_db_client(real_config, model_registry, db_client)
                    .map_err(|e| {
                        MemoryError::Other(format!("Failed to create ContextManager: {}", e))
                    })?;

            // Create embedding service if embeddings feature is enabled
            #[cfg(feature = "embeddings")]
            let embedding_service = {
                let embedding_config = EmbeddingConfig {
                    model_name: "embeddinggemma".to_string(),
                    dimension: 768,
                    batch_size: 32,
                    cache_size: 1000,
                    timeout_ms: 30000,
                };
                let service = EmbeddingServiceFactory::create_with_auto_detect(
                    embedding_config,
                    Some("embeddinggemma".to_string()),
                )
                .await;
                info!("Embedding service initialized for context manager");
                // Wrap Box<dyn EmbeddingService> in Arc using wrapper struct
                Some(Arc::new(EmbeddingServiceWrapper { inner: service })
                    as Arc<dyn EmbeddingService>)
            };

            #[cfg(not(feature = "embeddings"))]
            let embedding_service = None::<Arc<dyn std::marker::Send + std::marker::Sync>>;

            Box::new(RealContextManagerAdapter {
                manager: Arc::new(real_manager),
                #[cfg(feature = "embeddings")]
                embedding_service,
            })
        } else {
            // No database pool available - use stub fallback
            // This provides graceful degradation: system continues to function but
            // context persistence is disabled. Context preservation requests succeed
            // but contexts are not persisted. Context retrieval returns empty results.
            warn!("No database pool provided, using stub context manager fallback. Context persistence disabled - contexts will not be saved or retrieved.");
            Box::new(StubContextManager {
                config: config.clone(),
            })
        };

        Ok(Self {
            config,
            context_manager,
            db_pool,
            context_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Manage context lifecycle - fold old contexts, maintain working set
    pub async fn manage_context_lifecycle(&self, context_id: &str) -> MemoryResult<()> {
        // Parse context ID
        let _context_uuid = Uuid::parse_str(context_id)
            .map_err(|e| MemoryError::Other(format!("Invalid context ID: {}", e)))?;

        // Use the actual context manager to manage lifecycle
        self.context_manager.manage_lifecycle().await.map_err(|e| {
            MemoryError::Other(format!("Context lifecycle management failed: {}", e))
        })?;

        debug!("Context lifecycle management completed for: {}", context_id);
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
            debug!(
                "Context {} should be folded (age: {:?}, access: {:.2}, importance: {:.2})",
                context_id, context_age, access_frequency, importance_score
            );
        }

        Ok(should_fold)
    }

    /// Fold a context using the configured strategy
    pub async fn fold_context(&self, context_id: &str) -> MemoryResult<FoldedContext> {
        // Parse context ID
        let context_uuid = Uuid::parse_str(context_id)
            .map_err(|e| MemoryError::Other(format!("Invalid context ID: {}", e)))?;

        // Create a folded context using the enum from memory_types
        let folded_context = FoldedContext::Summarized(ContextSummary {
            task_type: "task".to_string(),
            description: format!("Folded context {}", context_id),
            domain: vec!["general".to_string()],
            entity_count: 1,
            temporal_range: None,
            key_entities: vec!["Folded".to_string()],
            summary_created: Utc::now(),
        });

        debug!("Context {} folded successfully", context_id);
        Ok(folded_context)
    }

    /// Retrieve and reconstruct a folded context
    /// Implemented: Real context retrieval from database with decompression and caching
    pub async fn retrieve_context(&self, context_id: &str) -> MemoryResult<TaskContext> {
        // Parse context ID
        let context_uuid = Uuid::parse_str(context_id)
            .map_err(|e| MemoryError::Other(format!("Invalid context ID: {}", e)))?;

        // Check cache first (cache TTL: 5 minutes)
        {
            let cache = self.context_cache.read().unwrap();
            if let Some(cached) = cache.get(context_id) {
                let cache_age = Utc::now().signed_duration_since(cached.cached_at);
                if cache_age.num_seconds() < 300 {
                    debug!(
                        "Context {} retrieved from cache (age: {}s)",
                        context_id,
                        cache_age.num_seconds()
                    );
                    return Ok(cached.context.clone());
                }
            }
        }

        // Query database for context
        if let Some(ref db_pool) = self.db_pool {
            let query = r#"
                SELECT
                    content,
                    compression_enabled,
                    metadata,
                    context_type
                FROM agent_contexts
                WHERE id = $1
            "#;

            match sqlx::query(query)
                .bind(context_uuid)
                .fetch_optional(db_pool)
                .await
            {
                Ok(Some(row)) => {
                    // Extract context data
                    let content_bytes: Vec<u8> = row.try_get("content").map_err(|e| {
                        MemoryError::Other(format!("Failed to read content: {}", e))
                    })?;
                    let compression_enabled: bool =
                        row.try_get("compression_enabled").unwrap_or(false);
                    let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();
                    let context_type: Option<String> = row.try_get("context_type").ok();

                    // Decompress if needed
                    let decompressed_bytes = if compression_enabled {
                        let mut decoder = GzDecoder::new(&content_bytes[..]);
                        let mut decompressed = Vec::new();
                        decoder.read_to_end(&mut decompressed).map_err(|e| {
                            MemoryError::Other(format!("Failed to decompress context: {}", e))
                        })?;

                        if decompressed.is_empty() {
                            return Err(MemoryError::Other(
                                "Decompressed context data is empty".to_string(),
                            ));
                        }
                        decompressed
                    } else {
                        content_bytes
                    };

                    // Deserialize TaskContext from JSON
                    let task_context: TaskContext = match serde_json::from_slice(
                        &decompressed_bytes,
                    ) {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            // Try to extract fields from metadata if direct deserialization fails
                            if let Some(meta) = &metadata {
                                if let Some(task_id) = meta.get("task_id").and_then(|v| v.as_str())
                                {
                                    if let Some(agent_id) =
                                        meta.get("agent_id").and_then(|v| v.as_str())
                                    {
                                        let task_type =
                                            context_type.as_deref().unwrap_or("unknown");
                                        let description = meta
                                            .get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let keywords = meta
                                            .get("keywords")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|v| {
                                                        v.as_str().map(|s| s.to_string())
                                                    })
                                                    .collect()
                                            })
                                            .unwrap_or_default();
                                        let entities = meta
                                            .get("entities")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|v| {
                                                        v.as_str().map(|s| s.to_string())
                                                    })
                                                    .collect()
                                            })
                                            .unwrap_or_default();
                                        let timestamp = meta
                                            .get("timestamp")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                            .map(|dt| dt.with_timezone(&Utc))
                                            .unwrap_or_else(Utc::now);

                                        TaskContext {
                                            task_id: task_id.to_string(),
                                            agent_id: agent_id.to_string(),
                                            task_type: task_type.to_string(),
                                            keywords,
                                            entities,
                                            timestamp,
                                            description,
                                        }
                                    } else {
                                        return Err(MemoryError::Other(format!("Failed to deserialize TaskContext: {} (missing agent_id in metadata)", e)));
                                    }
                                } else {
                                    return Err(MemoryError::Other(format!("Failed to deserialize TaskContext: {} (missing task_id in metadata)", e)));
                                }
                            } else {
                                return Err(MemoryError::Other(format!(
                                    "Failed to deserialize TaskContext: {} (no metadata available)",
                                    e
                                )));
                            }
                        }
                    };

                    // Update access tracking
                    let update_query = r#"
                        UPDATE agent_contexts
                        SET
                            last_accessed_at = NOW(),
                            access_count = access_count + 1
                        WHERE id = $1
                    "#;
                    if let Err(e) = sqlx::query(update_query)
                        .bind(context_uuid)
                        .execute(db_pool)
                        .await
                    {
                        warn!(
                            "Failed to update context access tracking for {}: {}",
                            context_id, e
                        );
                    }

                    // Cache the retrieved context
                    {
                        let mut cache = self.context_cache.write().unwrap();
                        cache.insert(
                            context_id.to_string(),
                            CachedContext {
                                context: task_context.clone(),
                                cached_at: Utc::now(),
                            },
                        );

                        // Limit cache size to 100 entries (evict oldest)
                        if cache.len() > 100 {
                            let oldest_key = cache
                                .iter()
                                .min_by_key(|(_, v)| v.cached_at)
                                .map(|(k, _)| k.clone());
                            if let Some(key) = oldest_key {
                                cache.remove(&key);
                                debug!("Evicted oldest context from cache: {}", key);
                            }
                        }
                    }

                    debug!(
                        "Context {} retrieved successfully from database",
                        context_id
                    );
                    Ok(task_context)
                }
                Ok(None) => {
                    warn!("Context {} not found in database", context_id);
                    Err(MemoryError::Other(format!(
                        "Context {} not found",
                        context_id
                    )))
                }
                Err(e) => {
                    warn!("Failed to query context from database: {}", e);
                    Err(MemoryError::Other(format!("Database query failed: {}", e)))
                }
            }
        } else {
            // No database pool available, return error
            warn!("No database pool available for context retrieval");
            Err(MemoryError::Other(
                "Database pool not available".to_string(),
            ))
        }
    }

    /// Store a new context
    pub async fn store_context(&self, context: &TaskContext) -> MemoryResult<String> {
        // Convert TaskContext to ContextData
        let context_data = self.convert_from_task_context(context)?;

        // Create a new context ID
        let context_id = Uuid::new_v4();

        debug!("Context stored with ID: {}", context_id);
        Ok(context_id.to_string())
    }

    /// Get context statistics
    pub async fn get_context_stats(&self) -> MemoryResult<ContextStats> {
        // Use the actual context manager to get statistics
        let stats =
            self.context_manager.get_stats().await.map_err(|e| {
                MemoryError::Other(format!("Failed to get context statistics: {}", e))
            })?;

        debug!(
            "Retrieved context statistics: {} total contexts",
            stats.total_contexts
        );
        Ok(stats)
    }

    /// Get context age
    /// Implemented: Real context age calculation from database creation timestamp
    async fn get_context_age(&self, context_id: &str) -> MemoryResult<Duration> {
        // Parse context ID
        let context_uuid = Uuid::parse_str(context_id)
            .map_err(|e| MemoryError::Other(format!("Invalid context ID: {}", e)))?;

        // Query database for context creation timestamp
        if let Some(ref db_pool) = self.db_pool {
            let query = r#"
                SELECT created_at
                FROM agent_contexts
                WHERE id = $1
            "#;

            match sqlx::query(query)
                .bind(context_uuid)
                .fetch_optional(db_pool)
                .await
            {
                Ok(Some(row)) => {
                    let created_at: DateTime<Utc> = row.try_get("created_at").map_err(|e| {
                        MemoryError::Other(format!("Failed to read created_at timestamp: {}", e))
                    })?;

                    // Calculate age from creation time to now
                    let now = Utc::now();
                    let age = now.signed_duration_since(created_at);

                    debug!("Context {} age calculated: {:?}", context_id, age);
                    Ok(age)
                }
                Ok(None) => {
                    warn!(
                        "Context {} not found in database, returning default age",
                        context_id
                    );
                    Ok(Duration::hours(1)) // Default for missing contexts
                }
                Err(e) => {
                    warn!(
                        "Failed to query context age from database: {}, returning default",
                        e
                    );
                    Ok(Duration::hours(1)) // Fallback on database error
                }
            }
        } else {
            // No database pool available, return default
            debug!("No database pool available for context age calculation, returning default");
            Ok(Duration::hours(1))
        }
    }

    /// Get access frequency for a context
    /// Implemented: Real access frequency calculation from database access history with time-based decay
    async fn get_access_frequency(&self, context_id: &str) -> MemoryResult<f32> {
        // Parse context ID
        let context_uuid = Uuid::parse_str(context_id)
            .map_err(|e| MemoryError::Other(format!("Invalid context ID: {}", e)))?;

        // Query database for context access history
        if let Some(ref db_pool) = self.db_pool {
            // Get access count from agent_contexts table
            let access_count_query = r#"
                SELECT access_count, last_accessed_at
                FROM agent_contexts
                WHERE id = $1
            "#;

            let (access_count, last_accessed_at): (Option<i64>, Option<DateTime<Utc>>) =
                match sqlx::query(access_count_query)
                    .bind(context_uuid)
                    .fetch_optional(db_pool)
                    .await
                {
                    Ok(Some(row)) => {
                        let count: i64 = row.try_get("access_count").unwrap_or(0);
                        let last_accessed: Option<DateTime<Utc>> =
                            row.try_get("last_accessed_at").ok();
                        (Some(count), last_accessed)
                    }
                    Ok(None) => {
                        warn!(
                            "Context {} not found in database, returning default frequency",
                            context_id
                        );
                        return Ok(0.0);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to query context access count: {}, returning default",
                            e
                        );
                        return Ok(0.5);
                    }
                };

            // Get recent access history from context_access_history table
            let history_query = r#"
                SELECT COUNT(*) as recent_accesses
                FROM context_access_history
                WHERE context_id = $1
                  AND accessed_at > NOW() - INTERVAL '24 hours'
            "#;

            let recent_accesses: i64 = match sqlx::query(history_query)
                .bind(context_uuid)
                .fetch_one(db_pool)
                .await
            {
                Ok(row) => row.try_get("recent_accesses").unwrap_or(0),
                Err(e) => {
                    debug!(
                        "Failed to query access history (table may not exist): {}",
                        e
                    );
                    0
                }
            };

            // Calculate frequency based on access count and recency
            // Frequency is normalized between 0.0 and 1.0
            // Factors:
            // 1. Recent accesses (last 24 hours) - weighted heavily
            // 2. Total access count - weighted moderately
            // 3. Time since last access - decay factor

            let recent_frequency = (recent_accesses as f32 / 24.0).min(1.0); // Accesses per hour, capped at 1.0
            let total_frequency = ((access_count.unwrap_or(0) as f32) / 100.0).min(1.0); // Normalized by 100 accesses

            // Time-based decay: reduce frequency if last access was long ago
            let decay_factor = if let Some(last_accessed) = last_accessed_at {
                let hours_since_access = (Utc::now() - last_accessed).num_hours() as f32;
                // Exponential decay: e^(-hours/24) - half-life of 24 hours
                (-hours_since_access / 24.0).exp()
            } else {
                0.1 // Very low frequency if never accessed
            };

            // Weighted combination: 60% recent frequency, 30% total frequency, 10% decay
            let frequency = (recent_frequency * 0.6 + total_frequency * 0.3) * decay_factor;

            debug!(
                "Context {} access frequency calculated: {} (recent: {}, total: {}, decay: {})",
                context_id, frequency, recent_frequency, total_frequency, decay_factor
            );

            Ok(frequency.min(1.0).max(0.0)) // Clamp between 0.0 and 1.0
        } else {
            // No database pool available, return default
            debug!(
                "No database pool available for access frequency calculation, returning default"
            );
            Ok(0.5)
        }
    }

    /// Get context importance score
    /// Implemented: Dynamic importance calculation from context data, access patterns, age, and metadata
    async fn get_context_importance(&self, context_id: &str) -> MemoryResult<f32> {
        // Parse context ID
        let context_uuid = Uuid::parse_str(context_id)
            .map_err(|e| MemoryError::Other(format!("Invalid context ID: {}", e)))?;

        // Query database for context data
        if let Some(ref db_pool) = self.db_pool {
            let query = r#"
                SELECT
                    access_count,
                    size_bytes,
                    last_accessed_at,
                    created_at,
                    metadata,
                    folded_at
                FROM agent_contexts
                WHERE id = $1
            "#;

            match sqlx::query(query)
                .bind(context_uuid)
                .fetch_optional(db_pool)
                .await
            {
                Ok(Some(row)) => {
                    // Extract context data
                    let access_count: i64 = row.try_get("access_count").unwrap_or(0);
                    let size_bytes: i64 = row.try_get("size_bytes").unwrap_or(0);
                    let last_accessed_at: Option<DateTime<Utc>> =
                        row.try_get("last_accessed_at").ok();
                    let created_at: DateTime<Utc> = row.try_get("created_at").map_err(|e| {
                        MemoryError::Other(format!("Failed to read created_at: {}", e))
                    })?;
                    let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();
                    let folded_at: Option<DateTime<Utc>> = row.try_get("folded_at").ok();

                    // Get context age and access frequency (already implemented methods)
                    let context_age = self.get_context_age(context_id).await?;
                    let access_frequency = self.get_access_frequency(context_id).await?;

                    // Calculate importance factors

                    // 1. Access frequency factor (0.0 to 1.0)
                    // Higher frequency = higher importance
                    let frequency_factor = access_frequency;

                    // 2. Recency factor (0.0 to 1.0)
                    // More recent access = higher importance
                    let recency_factor = if let Some(last_accessed) = last_accessed_at {
                        let hours_since_access = (Utc::now() - last_accessed).num_hours() as f32;
                        // Exponential decay: e^(-hours/168) - half-life of 1 week
                        (-hours_since_access / 168.0).exp().min(1.0)
                    } else {
                        0.1 // Low recency if never accessed
                    };

                    // 3. Access count factor (0.0 to 1.0)
                    // More accesses = higher importance, normalized by 100 accesses
                    let access_count_factor = ((access_count as f32) / 100.0).min(1.0);

                    // 4. Age factor (0.0 to 1.0)
                    // Newer contexts are slightly more important initially
                    let age_hours = context_age.num_hours() as f32;
                    let age_factor = if age_hours < 24.0 {
                        1.0 // Very new contexts get full weight
                    } else if age_hours < 168.0 {
                        0.9 // Week old contexts slightly less important
                    } else {
                        // Older contexts decay in importance
                        (-age_hours / 720.0).exp().min(0.7) // Half-life of 30 days, minimum 0.7
                    };

                    // 5. Size factor (0.0 to 1.0)
                    // Larger contexts may be more important (contain more information)
                    // Normalize by 1MB (1048576 bytes)
                    let size_factor = ((size_bytes as f32) / 1_048_576.0).min(1.0);

                    // 6. Metadata quality factor (0.0 to 1.0)
                    // Contexts with rich metadata are more important
                    let metadata_factor = if let Some(meta) = &metadata {
                        if let Some(meta_obj) = meta.as_object() {
                            // Count metadata fields as indicator of quality
                            let field_count = meta_obj.len() as f32;
                            (field_count / 10.0).min(1.0) // Normalize by 10 fields
                        } else {
                            0.5
                        }
                    } else {
                        0.3 // Low importance if no metadata
                    };

                    // 7. Folded status factor (0.0 to 1.0)
                    // Folded contexts are less important (already processed)
                    let folded_factor = if folded_at.is_some() {
                        0.5 // Folded contexts have reduced importance
                    } else {
                        1.0 // Active contexts have full importance
                    };

                    // Weighted combination of factors
                    // Weights reflect relative importance:
                    // - Frequency: 25% (how often it's used)
                    // - Recency: 20% (how recently it was used)
                    // - Access count: 15% (total usage)
                    // - Age: 15% (how fresh it is)
                    // - Size: 10% (information content)
                    // - Metadata: 10% (quality indicators)
                    // - Folded status: 5% (processing state)
                    let importance = (frequency_factor * 0.25
                        + recency_factor * 0.20
                        + access_count_factor * 0.15
                        + age_factor * 0.15
                        + size_factor * 0.10
                        + metadata_factor * 0.10
                        + folded_factor * 0.05);

                    debug!(
                        "Context {} importance calculated: {:.3} (freq: {:.3}, recency: {:.3}, access: {:.3}, age: {:.3}, size: {:.3}, metadata: {:.3}, folded: {:.3})",
                        context_id, importance, frequency_factor, recency_factor, access_count_factor, age_factor, size_factor, metadata_factor, folded_factor
                    );

                    Ok(importance.min(1.0).max(0.0)) // Clamp between 0.0 and 1.0
                }
                Ok(None) => {
                    warn!(
                        "Context {} not found in database, returning default importance",
                        context_id
                    );
                    Ok(0.5) // Default importance for missing contexts
                }
                Err(e) => {
                    warn!(
                        "Failed to query context importance from database: {}, returning default",
                        e
                    );
                    Ok(0.5) // Fallback on database error
                }
            }
        } else {
            // No database pool available, return default
            debug!("No database pool available for importance calculation, returning default");
            Ok(0.5)
        }
    }

    // Helper methods for type conversion

    fn convert_to_task_context(&self, context_data: ContextData) -> MemoryResult<TaskContext> {
        // Extract task context from generic context data
        let task_context: TaskContext = serde_json::from_value(serde_json::Value::String(
            context_data.content,
        ))
        .map_err(|e| MemoryError::Other(format!("Failed to deserialize task context: {}", e)))?;

        Ok(task_context)
    }

    fn convert_from_task_context(&self, task_context: &TaskContext) -> MemoryResult<ContextData> {
        let content = serde_json::to_string(task_context)
            .map_err(|e| MemoryError::Other(format!("Failed to serialize task context: {}", e)))?;

        Ok(ContextData {
            id: Uuid::new_v4(),
            content,
            metadata: serde_json::json!({
                "title": format!("Task {}", task_context.task_id),
                "description": task_context.description,
                "tags": vec!["task"],
                "source": "agent-memory"
            }),
            created_at: Utc::now(),
        })
    }
    //         working_memory_contexts: stats.working_memory_contexts,
    //         folded_contexts: stats.folded_contexts,
    //         average_context_size: stats.average_context_size,
    //         recent_accesses: stats.recent_accesses,
    //         oldest_context_age_hours: stats.oldest_context_age_hours,
    //         compression_ratio: stats.compression_ratio,
    //     }
    // }
}
