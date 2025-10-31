//! Memory Service Interface
//!
//! Defines interfaces and core types for agent memory systems to avoid
//! circular dependencies. Concrete implementations (e.g., Postgres-backed)
//! should live in implementation crates and be injected via this trait.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::Result;

/// Unique identifier types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId(pub String);

/// Memory record stored/retrieved by the memory service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: MemoryId,
    pub workspace_id: WorkspaceId,
    pub embedding: Option<Vec<f32>>, // Optional if not using vector search
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub importance: f32,
    pub decay_factor: f32,
}

/// Query for retrieving memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub workspace_id: Option<WorkspaceId>,
    pub text: Option<String>,
    pub vector: Option<Vec<f32>>, // If provided, use vector similarity
    pub top_k: Option<usize>,
    pub metadata_filters: HashMap<String, serde_json::Value>,
}

/// Result item for similarity searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub record: MemoryRecord,
    pub score: f32,
}

/// Errors for memory operations
#[derive(thiserror::Error, Debug)]
pub enum MemoryError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Memory service interface to be implemented by concrete backends
#[async_trait]
pub trait MemoryService: Send + Sync + std::fmt::Debug {
    /// Create a new memory record
    async fn create(&self, record: MemoryRecord) -> std::result::Result<MemoryRecord, MemoryError>;

    /// Update an existing memory record
    async fn update(&self, record: MemoryRecord) -> std::result::Result<MemoryRecord, MemoryError>;

    /// Get a memory record by id
    async fn get(&self, id: &MemoryId) -> std::result::Result<Option<MemoryRecord>, MemoryError>;

    /// Search memories by text/vector and filters
    async fn search(&self, query: MemoryQuery) -> std::result::Result<Vec<ScoredMemory>, MemoryError>;

    /// Record access time and optionally adjust importance/decay
    async fn touch(&self, id: &MemoryId, when: DateTime<Utc>) -> std::result::Result<(), MemoryError>;
}


