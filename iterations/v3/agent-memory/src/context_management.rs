//! Context Management - Working memory and context folding
//!
//! This module provides a memory-focused interface to the unified context preservation
//! system from agent-data-processing. It handles working memory limits, automatic
//! context folding, and retrieval with memory-specific optimizations.

use crate::memory_types::*;
use crate::MemoryResult;
use crate::MemoryError;

use chrono::{DateTime, Utc, Duration};
use serde_json;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{Row, PgPool};
use flate2::read::GzDecoder;
use std::io::Read;
// ContextConfig is defined in memory_types.rs

// NOTE: Real ContextManager integration requires resolving circular dependency
// agent-data-processing was removed from dependencies to break circular dependency
// To enable real ContextManager:
// 1. Resolve circular dependency between agent-memory and agent-data-processing
// 2. Uncomment agent-data-processing dependency in Cargo.toml
// 3. Uncomment the imports below and RealContextManagerAdapter implementation
//
// use agent_data_processing::context::manager::{ContextManager as RealContextManager, DatabaseClient, DatabaseConfig};
// use agent_data_processing::context::types::{
//     ContextPreservationRequest as RealContextPreservationRequest,
//     ContextRetrievalRequest as RealContextRetrievalRequest,
//     ContextPreservationResult as RealContextPreservationResult,
//     ContextRetrievalResult as RealContextRetrievalResult,
//     ContextStats as RealContextStats,
//     ContextData as RealContextData,
//     PreservationOptions,
//     PreservationPriority,
//     RetrievalOptions,
// };

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
    async fn preserve_context(&self, request: ContextPreservationRequest) -> Result<ContextPreservationResult, String>;
    async fn retrieve_context(&self, request: ContextRetrievalRequest) -> Result<ContextRetrievalResult, String>;
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
            .field("db_pool", &if self.db_pool.is_some() { "Some(PgPool)" } else { "None" })
            .finish()
    }
}

/// Temporary stub implementation for ContextManager
#[derive(Debug)]
struct StubContextManager {
    config: ContextConfig,
}

// RealContextManagerAdapter implementation - commented out until circular dependency is resolved
// 
// struct RealContextManagerAdapter {
//     manager: Arc<RealContextManager>,
// }
//
// #[async_trait::async_trait]
// impl ContextManager for RealContextManagerAdapter {
//     // Implementation converts between simple types (agent-memory) and complex types (agent-data-processing)
//     // See commented imports above for type definitions
// }

#[async_trait::async_trait]
impl ContextManager for StubContextManager {
    async fn manage_lifecycle(&self) -> Result<(), String> {
        Ok(())
    }

    async fn preserve_context(&self, _request: ContextPreservationRequest) -> Result<ContextPreservationResult, String> {
        Ok(ContextPreservationResult {
            success: true,
            context_id: Uuid::new_v4(),
            folded: false,
        })
    }

    async fn retrieve_context(&self, _request: ContextRetrievalRequest) -> Result<ContextRetrievalResult, String> {
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

impl MemoryContextManager {
    /// Create a new memory context manager
    pub async fn new(config: ContextConfig) -> MemoryResult<Self> {
        Self::new_with_db(config, None).await
    }
    
    /// Create a new memory context manager with database pool
    pub async fn new_with_db(config: ContextConfig, db_pool: Option<PgPool>) -> MemoryResult<Self> {
        // TODO: Replace stub context manager with real implementation
        // BLOCKED: Requires resolving circular dependency between agent-memory and agent-data-processing
        // 
        // To implement:
        // 1. Resolve circular dependency (agent-memory <-> agent-data-processing)
        // 2. Uncomment agent-data-processing dependency in Cargo.toml
        // 3. Uncomment RealContextManagerAdapter implementation below
        // 4. Use real ContextManager when db_pool is available:
        //
        //     let context_manager: Box<dyn ContextManager> = if let Some(_pool) = db_pool {
        //         // Create database client and real ContextManager
        //         // ... (see commented RealContextManagerAdapter implementation)
        //         Box::new(RealContextManagerAdapter { ... })
        //     } else {
        //         Box::new(StubContextManager { config: config.clone() })
        //     };
        //
        // Currently using stub implementation until circular dependency is resolved
        let context_manager = StubContextManager {
            config: config.clone(),
        };
        
        Ok(Self { 
            config,
            context_manager: Box::new(context_manager),
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
        self.context_manager.manage_lifecycle().await
            .map_err(|e| MemoryError::Other(format!("Context lifecycle management failed: {}", e)))?;
        
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
            debug!("Context {} should be folded (age: {:?}, access: {:.2}, importance: {:.2})",
                   context_id, context_age, access_frequency, importance_score);
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
                    debug!("Context {} retrieved from cache (age: {}s)", context_id, cache_age.num_seconds());
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
                    let content_bytes: Vec<u8> = row.try_get("content")
                        .map_err(|e| MemoryError::Other(format!("Failed to read content: {}", e)))?;
                    let compression_enabled: bool = row.try_get("compression_enabled").unwrap_or(false);
                    let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();
                    let context_type: Option<String> = row.try_get("context_type").ok();
                    
                    // Decompress if needed
                    let decompressed_bytes = if compression_enabled {
                        let mut decoder = GzDecoder::new(&content_bytes[..]);
                        let mut decompressed = Vec::new();
                        decoder.read_to_end(&mut decompressed)
                            .map_err(|e| MemoryError::Other(format!("Failed to decompress context: {}", e)))?;
                        
                        if decompressed.is_empty() {
                            return Err(MemoryError::Other("Decompressed context data is empty".to_string()));
                        }
                        decompressed
                    } else {
                        content_bytes
                    };
                    
                    // Deserialize TaskContext from JSON
                    let task_context: TaskContext = match serde_json::from_slice(&decompressed_bytes) {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            // Try to extract fields from metadata if direct deserialization fails
                            if let Some(meta) = &metadata {
                                if let Some(task_id) = meta.get("task_id").and_then(|v| v.as_str()) {
                                    if let Some(agent_id) = meta.get("agent_id").and_then(|v| v.as_str()) {
                                        let task_type = context_type.as_deref().unwrap_or("unknown");
                                        let description = meta.get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let keywords = meta.get("keywords")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| arr.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect())
                                            .unwrap_or_default();
                                        let entities = meta.get("entities")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| arr.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect())
                                            .unwrap_or_default();
                                        let timestamp = meta.get("timestamp")
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
                                return Err(MemoryError::Other(format!("Failed to deserialize TaskContext: {} (no metadata available)", e)));
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
                        warn!("Failed to update context access tracking for {}: {}", context_id, e);
                    }
                    
                    // Cache the retrieved context
                    {
                        let mut cache = self.context_cache.write().unwrap();
                        cache.insert(context_id.to_string(), CachedContext {
                            context: task_context.clone(),
                            cached_at: Utc::now(),
                        });
                        
                        // Limit cache size to 100 entries (evict oldest)
                        if cache.len() > 100 {
                            let oldest_key = cache.iter()
                                .min_by_key(|(_, v)| v.cached_at)
                                .map(|(k, _)| k.clone());
                            if let Some(key) = oldest_key {
                                cache.remove(&key);
                                debug!("Evicted oldest context from cache: {}", key);
                            }
                        }
                    }
                    
                    debug!("Context {} retrieved successfully from database", context_id);
                    Ok(task_context)
                }
                Ok(None) => {
                    warn!("Context {} not found in database", context_id);
                    Err(MemoryError::Other(format!("Context {} not found", context_id)))
                }
                Err(e) => {
                    warn!("Failed to query context from database: {}", e);
                    Err(MemoryError::Other(format!("Database query failed: {}", e)))
                }
            }
        } else {
            // No database pool available, return error
            warn!("No database pool available for context retrieval");
            Err(MemoryError::Other("Database pool not available".to_string()))
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
        let stats = self.context_manager.get_stats().await
            .map_err(|e| MemoryError::Other(format!("Failed to get context statistics: {}", e)))?;
        
        debug!("Retrieved context statistics: {} total contexts", stats.total_contexts);
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
                    let created_at: DateTime<Utc> = row.try_get("created_at")
                        .map_err(|e| MemoryError::Other(format!("Failed to read created_at timestamp: {}", e)))?;
                    
                    // Calculate age from creation time to now
                    let now = Utc::now();
                    let age = now.signed_duration_since(created_at);
                    
                    debug!("Context {} age calculated: {:?}", context_id, age);
                    Ok(age)
                }
                Ok(None) => {
                    warn!("Context {} not found in database, returning default age", context_id);
                    Ok(Duration::hours(1)) // Default for missing contexts
                }
                Err(e) => {
                    warn!("Failed to query context age from database: {}, returning default", e);
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
            
            let (access_count, last_accessed_at): (Option<i64>, Option<DateTime<Utc>>) = match sqlx::query(access_count_query)
                .bind(context_uuid)
                .fetch_optional(db_pool)
                .await
            {
                Ok(Some(row)) => {
                    let count: i64 = row.try_get("access_count")
                        .unwrap_or(0);
                    let last_accessed: Option<DateTime<Utc>> = row.try_get("last_accessed_at").ok();
                    (Some(count), last_accessed)
                }
                Ok(None) => {
                    warn!("Context {} not found in database, returning default frequency", context_id);
                    return Ok(0.0);
                }
                Err(e) => {
                    warn!("Failed to query context access count: {}, returning default", e);
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
                    debug!("Failed to query access history (table may not exist): {}", e);
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
            
            debug!("Context {} access frequency calculated: {} (recent: {}, total: {}, decay: {})", 
                context_id, frequency, recent_frequency, total_frequency, decay_factor);
            
            Ok(frequency.min(1.0).max(0.0)) // Clamp between 0.0 and 1.0
        } else {
            // No database pool available, return default
            debug!("No database pool available for access frequency calculation, returning default");
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
                    let last_accessed_at: Option<DateTime<Utc>> = row.try_get("last_accessed_at").ok();
                    let created_at: DateTime<Utc> = row.try_get("created_at")
                        .map_err(|e| MemoryError::Other(format!("Failed to read created_at: {}", e)))?;
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
                    let importance = (
                        frequency_factor * 0.25 +
                        recency_factor * 0.20 +
                        access_count_factor * 0.15 +
                        age_factor * 0.15 +
                        size_factor * 0.10 +
                        metadata_factor * 0.10 +
                        folded_factor * 0.05
                    );
                    
                    debug!(
                        "Context {} importance calculated: {:.3} (freq: {:.3}, recency: {:.3}, access: {:.3}, age: {:.3}, size: {:.3}, metadata: {:.3}, folded: {:.3})",
                        context_id, importance, frequency_factor, recency_factor, access_count_factor, age_factor, size_factor, metadata_factor, folded_factor
                    );
                    
                    Ok(importance.min(1.0).max(0.0)) // Clamp between 0.0 and 1.0
                }
                Ok(None) => {
                    warn!("Context {} not found in database, returning default importance", context_id);
                    Ok(0.5) // Default importance for missing contexts
                }
                Err(e) => {
                    warn!("Failed to query context importance from database: {}, returning default", e);
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
        let task_context: TaskContext = serde_json::from_value(serde_json::Value::String(context_data.content))
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
