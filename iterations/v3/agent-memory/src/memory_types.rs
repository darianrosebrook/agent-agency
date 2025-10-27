//! Memory Types - Core data structures for the memory system
//!
//! This module defines the fundamental types used throughout the memory system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a memory
pub type MemoryId = Uuid;

/// Configuration for the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub workspace_config: WorkspaceConfig,
    pub graph_config: GraphConfig,
    pub decay_config: DecayConfig,
    pub context_config: ContextConfig,
    pub temporal_config: TemporalConfig,
    #[cfg(feature = "embeddings")]
    pub embedding_config: EmbeddingConfig,
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub access_config: WorkspaceAccessConfig,
}

/// Workspace access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAccessConfig {
    pub max_workspaces: usize,
    pub default_ttl_hours: u64,
}

/// Graph configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub max_entities: usize,
    pub max_relationships: usize,
}

/// Decay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    pub decay_rate: f32,
    pub min_importance: f32,
}

/// Context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_contexts: usize,
    pub fold_threshold: f32,
}

/// Temporal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub time_window_hours: u64,
    pub pattern_detection_enabled: bool,
}

/// Embedding configuration
#[cfg(feature = "embeddings")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimensions: usize,
}

/// Agent experience for episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExperience {
    pub id: MemoryId,
    pub agent_id: String,
    pub task_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task context for memory retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: String,
    pub agent_id: String,
    pub keywords: Vec<String>,
    pub entities: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

/// Contextual memory with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualMemory {
    pub memory: AgentExperience,
    pub relevance_score: f32,
    pub context_match: ContextMatch,
    pub reasoning_path: Vec<String>,
}

/// Type of context match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextMatch {
    Semantic,
    Graph(usize), // Path length
    Temporal,
    Keyword,
}

/// Reasoning query for multi-hop reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningQuery {
    pub start_entity: String,
    pub target_entity: String,
    pub max_hops: usize,
    pub relationship_types: Vec<String>,
}

/// Result of reasoning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    pub path: Vec<String>,
    pub confidence: f32,
    pub reasoning_steps: Vec<String>,
}

/// Time range for temporal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

/// Temporal analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysis {
    pub patterns: Vec<String>,
    pub performance_metrics: HashMap<String, f32>,
    pub recommendations: Vec<String>,
}

/// Context statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub total_contexts: usize,
    pub total_storage_size: u64,
    pub working_memory_contexts: usize,
    pub folded_contexts: usize,
    pub average_context_size: f32,
    pub recent_accesses: usize,
    pub oldest_context_age_hours: f32,
    pub compression_ratio: f32,
}

/// Folded context types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FoldedContext {
    Compressed {
        data: Vec<u8>,
        original_size: usize,
        compressed_size: usize,
        compression_ratio: f32,
    },
    Summarized(ContextSummary),
    Archived(ArchivedContext),
    Deleted,
}

/// Context summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub task_type: String,
    pub description: String,
    pub domain: Vec<String>,
    pub entity_count: usize,
    pub temporal_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub key_entities: Vec<String>,
    pub summary_created: chrono::DateTime<chrono::Utc>,
}

/// Archived context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedContext {
    pub context: TaskContext,
    pub archived_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub retention_policy: RetentionPolicy,
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    ShortTerm,
    LongTerm,
    Permanent,
}

/// Default implementation for TaskContext
impl Default for TaskContext {
    fn default() -> Self {
        Self {
            task_id: "default".to_string(),
            agent_id: "default".to_string(),
            keywords: vec![],
            entities: vec![],
            timestamp: chrono::Utc::now(),
            description: "default task".to_string(),
        }
    }
}

/// Workspace entry for registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
}

/// Workspace access information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAccess {
    pub workspace_id: String,
    pub access_type: WorkspaceAccessType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of workspace access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceAccessType {
    Read,
    Write,
    Create,
    Delete,
}

/// Workspace access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAccessControl {
    pub workspace_id: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}