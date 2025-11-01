//! Memory Types - Core data structures for the memory system
//!
//! This module defines the fundamental types used throughout the memory system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Memory type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    Episodic,  // Event-based memories
    Semantic,  // Factual knowledge
    Procedural, // Skill-based memories
    Working,   // Short-term working memory
}


/// Temporal context for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration: Option<chrono::Duration>,
    pub sequence_number: Option<u64>,
    pub priority: TaskPriority,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Experience outcome classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceOutcome {
    pub success: bool,
    pub quality_score: f64,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub performance_score: Option<f32>,
    pub execution_time_ms: Option<u64>,
    pub learned_capabilities: Vec<String>,
}

/// Agent feedback for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFeedback {
    pub feedback_type: String,
    pub rating: f64,
    pub comment: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Experience context for memory storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceContext {
    pub description: String,
    pub domain: Vec<String>,
    pub task_type: String,
    pub temporal_context: Option<TemporalContext>,
}

/// Unique identifier for a memory
pub type MemoryId = Uuid;

/// Configuration for the memory system
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceConfig {
    pub access_config: WorkspaceAccessConfig,
    pub current_workspace_id: String,
    pub isolation_level: String,
    pub enable_cross_workspace_access: bool,
}

/// Workspace access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAccessConfig {
    pub max_workspaces: usize,
    pub default_ttl_hours: u64,
    pub discovery_paths: Vec<String>,
    pub default_access: WorkspaceAccess,
    pub default_workspaces: Vec<String>,
    pub blocked_workspaces: Vec<String>,
}

impl Default for WorkspaceAccessConfig {
    fn default() -> Self {
        Self {
            max_workspaces: 10,
            default_ttl_hours: 24,
            discovery_paths: vec![],
            default_access: WorkspaceAccess::Enabled,
            default_workspaces: vec![],
            blocked_workspaces: vec![],
        }
    }
}

/// Graph configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphConfig {
    pub max_entities: usize,
    pub max_relationships: usize,
}

/// Decay configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecayConfig {
    pub decay_rate: f32,
    pub min_importance: f32,
    pub decay_schedule: DecaySchedule,
    pub minimum_memory_strength: f32,
    pub base_decay_rate: f32,
    pub importance_boost_factor: f32,
}

/// Context configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextConfig {
    pub max_contexts: usize,
    pub fold_threshold: f32,
}

/// Temporal configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalConfig {
    pub time_window_hours: u64,
    pub pattern_detection_enabled: bool,
    pub change_point_sensitivity: f32,
}

/// Embedding configuration
#[cfg(feature = "embeddings")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimensions: usize,
    pub similarity_threshold: f32,
}

/// Experience context
/// Agent experience for episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExperience {
    pub id: MemoryId,
    pub agent_id: String,
    pub task_id: String,
    pub content: String,
    pub input: String,
    pub output: String,
    pub context: ExperienceContext,
    pub outcome: ExperienceOutcome,
    pub memory_type: MemoryType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task context for memory retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: String,
    pub agent_id: String,
    pub task_type: String,
    pub keywords: Vec<String>,
    pub entities: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

/// Contextual memory with relevance scoring

/// Type of context match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextMatch {
    Exact,
    Semantic,
    SemanticScore(f32),
    Graph(usize), // Path length
    Temporal,
    TemporalScore(f32),
    Keyword,
}

/// Reasoning query for multi-hop reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningQuery {
    pub start_entity: String,
    pub target_entity: String,
    pub start_entities: Vec<String>,
    pub target_entities: Vec<String>,
    pub max_hops: usize,
    pub min_confidence: f32,
    pub relationship_types: Vec<String>,
}

/// Result of reasoning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    pub path: Vec<String>,
    pub paths: Vec<Vec<String>>,
    pub confidence: f32,
    pub confidence_score: f32,
    pub reasoning_steps: Vec<String>,
    pub reasoning_time_ms: u32,
    pub entities_discovered: Vec<String>,
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
    pub time_range: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    pub trends: Vec<TrendDirection>,
    pub change_points: Vec<chrono::DateTime<chrono::Utc>>,
    pub causality_links: Vec<(String, String, f32)>,
    pub performance_summary: String,
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
            task_type: "default".to_string(),
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
    pub path: std::path::PathBuf,
    pub access: WorkspaceAccess,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub is_default: bool,
}

/// Workspace access information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAccessInfo {
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


impl TryFrom<i32> for MemoryType {
    type Error = ();
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MemoryType::Episodic),
            1 => Ok(MemoryType::Semantic),
            2 => Ok(MemoryType::Procedural),
            3 => Ok(MemoryType::Working),
            _ => Err(()),
        }
    }
}

/// Trend direction for temporal analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

/// Decay schedule types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DecaySchedule {
    #[default]
    Exponential,
    PowerLaw,
    Logarithmic,
    Custom(String), // Formula as string
}

/// Workspace access levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkspaceAccess {
    #[default]
    Enabled,
    Disabled,
    ReadOnly,
    Blocked,
}

/// Workspace isolation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceIsolationLevel {
    Strict,        // Only current workspace
    WorkspaceFirst, // Prefer current workspace
    GlobalFirst,   // Allow global access
    Unrestricted,  // Allow all workspaces
}

/// Memory content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryContent {
    Text(String),
    Structured(serde_json::Value),
}

/// Core memory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: MemoryId,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub importance_score: f32,
    pub importance: f32,
    pub tags: Option<Vec<String>>,
    pub memory_type: MemoryType,
}

/// Contextual memory with context matching information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualMemory {
    pub memory: AgentExperience,
    pub context_match: ContextMatch,
    pub relevance_score: f32,
    pub reasoning_path: Vec<String>,
}


/// Reinforcement context for serde support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementContext {
    pub context_type: String,
    pub context_data: HashMap<String, serde_json::Value>,
}

/// Context data structure (imported from agent-data-processing)
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
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub last_accessed_at: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// Context size (bytes)
    pub size_bytes: u64,
}

/// Context metadata (imported from agent-data-processing)
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

/// Context preservation request (imported from agent-data-processing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationRequest {
    /// Context data to preserve
    pub context_data: ContextData,
    /// Preservation options
    pub options: PreservationOptions,
}

/// Preservation options (imported from agent-data-processing)
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

/// Preservation priority (imported from agent-data-processing)
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

/// Retrieval options (imported from agent-data-processing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalOptions {
    /// Include metadata
    pub include_metadata: bool,
    /// Decompress if compressed
    pub decompress: bool,
    /// Validate checksum
    pub validate_checksum: bool,
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

/// Context retrieval request (imported from agent-data-processing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalRequest {
    /// Context ID to retrieve
    pub context_id: Uuid,
    /// Retrieval options
    pub options: RetrievalOptions,
}

/// Reasoning path for multi-hop reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPath {
    /// Entities in the path
    pub entities: Vec<String>,
    /// Relationships between entities
    pub relationships: Vec<String>,
    /// Confidence score for this path
    pub confidence: f32,
    /// Path length (number of hops)
    pub length: usize,
}

/// Temporal trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTrend {
    /// Metric name
    pub metric: String,
    /// Trend direction
    pub direction: TrendDirection,
    /// Confidence in the trend (0.0-1.0)
    pub confidence: f32,
    /// Time period analyzed
    pub time_range: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    /// Change magnitude
    pub magnitude: f32,
}

/// Change point detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePoint {
    /// Timestamp when change occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Confidence in the change point (0.0-1.0)
    pub confidence: f32,
    /// Type of change detected
    pub change_type: ChangeType,
    /// Magnitude of the change
    pub magnitude: f32,
}

/// Type of change detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Sudden increase
    Spike,
    /// Sudden decrease
    Drop,
    /// Gradual change
    Trend,
    /// Level shift
    Shift,
}

/// Causality link between events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityLink {
    /// Cause event
    pub cause: String,
    /// Effect event
    pub effect: String,
    /// Confidence in the causal relationship (0.0-1.0)
    pub confidence: f32,
    /// Time lag between cause and effect
    pub time_lag_seconds: i64,
    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Performance summary for temporal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Overall performance score (0.0-1.0)
    pub overall_score: f32,
    /// Individual metric scores
    pub metric_scores: HashMap<String, f32>,
    /// Performance trends
    pub trends: Vec<TemporalTrend>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    /// Analysis timestamp
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

/// Capability evolution over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEvolution {
    /// Capability name
    pub capability: String,
    /// Evolution timeline
    pub timeline: Vec<EvolutionPoint>,
    /// Current capability level (0.0-1.0)
    pub current_level: f32,
    /// Predicted future level (0.0-1.0)
    pub predicted_level: f32,
    /// Learning rate assessment
    pub learning_rate: f32,
}

/// Evolution point in capability development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPoint {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Capability level at this point (0.0-1.0)
    pub level: f32,
    /// Context or trigger for this evolution
    pub context: String,
    /// Performance metrics at this point
    pub metrics: HashMap<String, f32>,
}

