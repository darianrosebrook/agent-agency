//! Context Preservation Types - Unified context management for Agent Agency V3
//!
//! This module consolidates context preservation functionality from:
//! - context-preservation-engine (full-featured multi-tenant)
//! - agent-memory (working memory folding)
//! - reflexive-learning (context-aware learning)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Context preservation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Context storage configuration
    pub storage: ContextStorageConfig,
    /// Context folding configuration
    pub folding: ContextFoldingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Working memory configuration
    pub working_memory: WorkingMemoryConfig,
}

/// Context storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStorageConfig {
    /// Maximum context size (bytes)
    pub max_context_size: u64,
    /// Context retention period (hours)
    pub retention_hours: u32,
    /// Maximum number of contexts
    pub max_contexts: u32,
    /// Enable persistent storage
    pub enable_persistent_storage: bool,
    /// Enable in-memory caching
    pub enable_memory_cache: bool,
    /// Cache size limit (bytes)
    pub cache_size_limit: u64,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Enable checksum validation
    pub checksum_validation: bool,
}

impl Default for ContextStorageConfig {
    fn default() -> Self {
        Self {
            max_context_size: 50 * 1024 * 1024, // 50MB
            retention_hours: 168, // 1 week
            max_contexts: 1000,
            enable_persistent_storage: true,
            enable_memory_cache: true,
            cache_size_limit: 100 * 1024 * 1024, // 100MB
            enable_compression: true,
            compression_level: 6,
            checksum_validation: true,
        }
    }
}

/// Context folding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFoldingConfig {
    /// Folding strategy
    pub strategy: FoldingStrategy,
    /// Age threshold for folding (hours)
    pub age_threshold_hours: u32,
    /// Importance threshold for folding (0.0-1.0)
    pub importance_threshold: f64,
    /// Access frequency threshold for folding
    pub access_frequency_threshold: f64,
    /// Maximum working memory contexts
    pub max_working_memory_contexts: usize,
}

impl Default for ContextFoldingConfig {
    fn default() -> Self {
        Self {
            strategy: FoldingStrategy::Compress,
            age_threshold_hours: 4,
            importance_threshold: 0.5,
            access_frequency_threshold: 0.3,
            max_working_memory_contexts: 100,
        }
    }
}

/// Folding strategy for context compression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FoldingStrategy {
    /// Compress context using gzip
    Compress,
    /// Summarize context using AI/ML
    Summarize,
    /// Archive context to cold storage
    Archive,
    /// Delete context permanently
    Delete,
}

/// Working memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryConfig {
    /// Maximum working memory size (contexts)
    pub max_size: usize,
    /// Working memory access pattern tracking
    pub track_access_patterns: bool,
    /// Automatic cleanup interval (minutes)
    pub cleanup_interval_minutes: u32,
}

impl Default for WorkingMemoryConfig {
    fn default() -> Self {
        Self {
            max_size: 50,
            track_access_patterns: true,
            cleanup_interval_minutes: 30,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum preservation time (ms)
    pub max_preservation_time_ms: u64,
    /// Maximum retrieval time (ms)
    pub max_retrieval_time_ms: u64,
    /// Enable parallel processing
    pub enable_parallel_processing: bool,
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_preservation_time_ms: 5000,
            max_retrieval_time_ms: 2000,
            enable_parallel_processing: true,
            batch_size: 10,
        }
    }
}

/// Context data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// Unique context ID
    pub id: Uuid,
    /// Context type/category
    pub context_type: String,
    /// Context content
    pub content: serde_json::Value,
    /// Context metadata
    pub metadata: ContextMetadata,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub last_accessed_at: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
    /// Context size (bytes)
    pub size_bytes: u64,
}

/// Context metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Human-readable title
    pub title: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Source information
    pub source: Option<String>,
    /// Importance score (0.0-1.0)
    pub importance_score: Option<f64>,
    /// Custom metadata fields
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Context preservation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationRequest {
    /// Context data to preserve
    pub context_data: ContextData,
    /// Preservation options
    pub options: PreservationOptions,
}

/// Preservation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationOptions {
    /// Force preservation (ignore limits)
    pub force: bool,
    /// Enable compression
    pub compress: bool,
    /// Priority level
    pub priority: PreservationPriority,
    /// Custom metadata to add
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

/// Preservation priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreservationPriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Context preservation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationResult {
    /// Success status
    pub success: bool,
    /// Context ID (if preserved)
    pub context_id: Option<Uuid>,
    /// Processing time (ms)
    pub processing_time_ms: u64,
    /// Size after processing (bytes)
    pub processed_size_bytes: u64,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

/// Context retrieval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalRequest {
    /// Context ID to retrieve
    pub context_id: Uuid,
    /// Retrieval options
    pub options: RetrievalOptions,
}

/// Retrieval options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalOptions {
    /// Include metadata
    pub include_metadata: bool,
    /// Decompress if compressed
    pub decompress: bool,
    /// Validate checksum
    pub validate_checksum: bool,
}

/// Context retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalResult {
    /// Success status
    pub success: bool,
    /// Context data (if retrieved)
    pub context_data: Option<ContextData>,
    /// Processing time (ms)
    pub processing_time_ms: u64,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

/// Folded context result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FoldedContext {
    /// Context was compressed
    Compressed(Vec<u8>),
    /// Context was summarized
    Summarized(String),
    /// Context was archived
    Archived(String),
    /// Context was deleted
    Deleted,
}

/// Context folding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFoldingRequest {
    /// Context ID to fold
    pub context_id: Uuid,
    /// Folding strategy to use
    pub strategy: FoldingStrategy,
    /// Folding options
    pub options: FoldingOptions,
}

/// Folding options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldingOptions {
    /// Compression level (if compressing)
    pub compression_level: Option<u32>,
    /// Maximum summary length (if summarizing)
    pub max_summary_length: Option<usize>,
    /// Archive location (if archiving)
    pub archive_location: Option<String>,
}

/// Context statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    /// Total contexts stored
    pub total_contexts: u64,
    /// Total storage size (bytes)
    pub total_storage_size: u64,
    /// Working memory contexts
    pub working_memory_contexts: usize,
    /// Folded contexts
    pub folded_contexts: u64,
    /// Average context size (bytes)
    pub average_context_size: u64,
    /// Contexts accessed in last 24h
    pub recent_accesses: u64,
    /// Oldest context age (hours)
    pub oldest_context_age_hours: u64,
    /// Compression ratio
    pub compression_ratio: f64,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            storage: ContextStorageConfig::default(),
            folding: ContextFoldingConfig::default(),
            performance: PerformanceConfig::default(),
            working_memory: WorkingMemoryConfig::default(),
        }
    }
}

impl Default for PreservationOptions {
    fn default() -> Self {
        Self {
            force: false,
            compress: true,
            priority: PreservationPriority::Normal,
            custom_metadata: HashMap::new(),
        }
    }
}

impl Default for RetrievalOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            decompress: true,
            validate_checksum: true,
        }
    }
}

impl Default for FoldingOptions {
    fn default() -> Self {
        Self {
            compression_level: Some(6),
            max_summary_length: Some(1000),
            archive_location: None,
        }
    }
}
