//! Core types for the verdict storage and management system
//!
//! This module contains the fundamental data structures and types
//! used throughout the verdict storage system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{VerdictId, TaskId, ConsensusResult, DebateSession};

/// Verdict record with metadata and storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictRecord {
    pub verdict_id: VerdictId,
    pub consensus_result: ConsensusResult,
    pub debate_session: Option<DebateSession>,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u64,
    pub storage_location: Option<String>,
}

/// Cache configuration for verdict storage
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_cached_verdicts: usize,
    pub cache_ttl_seconds: u64,
    pub enable_persistence: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cached_verdicts: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            enable_persistence: true,
        }
    }
}

/// Storage statistics for verdict persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_verdicts: u64,
    pub total_debates: u64,
    pub storage_size_bytes: u64,
    pub oldest_verdict: Option<DateTime<Utc>>,
    pub newest_verdict: Option<DateTime<Utc>>,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub eviction_count: u64,
    pub last_access: Option<DateTime<Utc>>,
}

/// Verdict store statistics combining storage and cache stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictStoreStats {
    pub storage_stats: StorageStats,
    pub cache_stats: CacheStats,
    pub uptime_seconds: u64,
    pub operations_count: u64,
}

/// Storage operation result with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOperation {
    pub operation_type: StorageOperationType,
    pub verdict_id: Option<VerdictId>,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Types of storage operations for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOperationType {
    Store,
    Load,
    LoadByTask,
    LoadByTimeRange,
    Delete,
    GetStats,
}

/// Verdict query parameters for flexible searching
#[derive(Debug, Clone)]
pub struct VerdictQuery {
    pub task_id: Option<TaskId>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_debates: bool,
}

/// Verdict query result with pagination information
#[derive(Debug, Clone)]
pub struct VerdictQueryResult {
    pub records: Vec<VerdictRecord>,
    pub total_count: u64,
    pub has_more: bool,
    pub query_duration_ms: u64,
}

/// Verdict cleanup policy for automatic maintenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    pub max_age_days: u32,
    pub max_storage_size_bytes: u64,
    pub min_retention_count: u32,
    pub enable_auto_cleanup: bool,
}

/// Verdict storage health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String, critical: bool },
}

/// Storage health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub status: StorageHealth,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}
