//! Core types for the Agent Memory System

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Unique identifier for memory operations
pub type MemoryId = Uuid;

/// Memory types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    /// Episodic memory - specific events and experiences
    Episodic,
    /// Semantic memory - general knowledge and facts
    Semantic,
    /// Procedural memory - how-to knowledge and skills
    Procedural,
    /// Working memory - temporary context for current tasks
    Working,
}

/// Agent experience data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExperience {
    pub id: MemoryId,
    pub agent_id: String,
    pub task_id: String,
    pub context: TaskContext,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub outcome: ExperienceOutcome,
    pub memory_type: MemoryType,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Outcome of an agent experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceOutcome {
    pub success: bool,
    pub performance_score: Option<f32>,
    pub learned_capabilities: Vec<String>,
    pub failure_reasons: Vec<String>,
    pub success_factors: Vec<String>,
    pub execution_time_ms: Option<i64>,
    pub tokens_used: Option<i32>,
    pub feedback: Option<AgentFeedback>,
}

/// Feedback from evaluation systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFeedback {
    pub quality_score: Option<f32>,
    pub relevance_score: Option<f32>,
    pub accuracy_score: Option<f32>,
    pub comments: Vec<String>,
    pub evaluator_id: Option<String>,
}

/// Task context for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: String,
    pub task_type: String,
    pub description: String,
    pub domain: Vec<String>,
    pub entities: Vec<String>,
    pub temporal_context: Option<TemporalContext>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Temporal context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub start_time: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub priority: TaskPriority,
    pub recurrence_pattern: Option<String>,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TryFrom<i32> for MemoryType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MemoryType::Episodic),
            1 => Ok(MemoryType::Semantic),
            2 => Ok(MemoryType::Procedural),
            3 => Ok(MemoryType::Working),
            _ => Err(format!("Invalid memory type value: {}", value)),
        }
    }
}

impl From<MemoryType> for i32 {
    fn from(memory_type: MemoryType) -> Self {
        match memory_type {
            MemoryType::Episodic => 0,
            MemoryType::Semantic => 1,
            MemoryType::Procedural => 2,
            MemoryType::Working => 3,
        }
    }
}

/// Capability evolution over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEvolution {
    pub capability: String,
    pub week: DateTime<Utc>,
    pub learned_count: i64,
    pub avg_performance: Option<f64>,
    pub improvement_rate: f64,
}

/// Contextual memory result from retrieval operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualMemory {
    pub memory: AgentExperience,
    pub relevance_score: f32,
    pub context_match: ContextMatch,
    pub reasoning_path: Vec<String>,
}

/// How a memory was matched to the context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextMatch {
    /// Semantic similarity match via embeddings
    Semantic,
    /// Graph-based match with hop distance
    Graph(usize),
    /// Temporal match (recent or time-related)
    Temporal,
    /// Direct entity match
    Entity(String),
    /// Combined multi-criteria match
    Multi(Vec<String>),
}

/// Time range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub graph_config: GraphConfig,
    pub embedding_config: EmbeddingConfig,
    pub temporal_config: TemporalConfig,
    pub decay_config: DecayConfig,
    pub context_config: ContextConfig,
    pub performance_config: PerformanceConfig,
}

/// Knowledge graph configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub max_entities: usize,
    pub max_relationships_per_entity: usize,
    pub similarity_threshold: f32,
    pub deduplication_enabled: bool,
    pub reasoning_depth: usize,
    pub cache_size: usize,
}

/// Embedding service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub cache_enabled: bool,
    pub cache_size: usize,
    pub similarity_threshold: f32,
}

/// Temporal reasoning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub analysis_window_days: i64,
    pub causality_enabled: bool,
    pub trend_detection_enabled: bool,
    pub forecasting_enabled: bool,
    pub change_point_sensitivity: f32,
}

/// Memory decay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    pub base_decay_rate: f32,
    pub importance_boost_factor: f32,
    pub access_recency_weight: f32,
    pub consolidation_interval_hours: i64,
    pub minimum_memory_strength: f32,
    pub decay_schedule: DecaySchedule,
}

/// Decay schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecaySchedule {
    /// Exponential decay: memory_strength *= (1 - decay_rate) ^ time_elapsed
    Exponential,
    /// Power law decay: memory_strength *= time_elapsed ^ (-decay_rate)
    PowerLaw,
    /// Logarithmic decay: memory_strength -= log(time_elapsed) * decay_rate
    Logarithmic,
    /// Custom decay function (for advanced users)
    Custom(String),
}

/// Context offloading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub compression_enabled: bool,
    pub compression_threshold_kb: usize,
    pub offload_strategy: OffloadStrategy,
    pub retrieval_boost_factor: f32,
    pub max_context_age_days: i64,
}

/// Context offloading strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OffloadStrategy {
    /// Compress old contexts but keep them accessible
    Compress,
    /// Summarize old contexts and store summaries
    Summarize,
    /// Archive old contexts to secondary storage
    Archive,
    /// Delete old contexts permanently
    Delete,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub metrics_enabled: bool,
    pub query_timeout_ms: u64,
    pub max_concurrent_queries: usize,
    pub memory_pressure_threshold_mb: usize,
    pub cache_enabled: bool,
}

/// Temporal reasoning query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalQuery {
    pub agent_id: Option<String>,
    pub time_range: Option<TimeRange>,
    pub analysis_type: TemporalAnalysisType,
}

/// Types of temporal analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalAnalysisType {
    PerformanceTrends,
    CapabilityEvolution,
    CausalityAnalysis,
    PatternRecognition,
}

/// Causality result from temporal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityResult {
    pub cause_event: String,
    pub effect_event: String,
    pub confidence: f32,
    pub time_delay: i64, // milliseconds
    pub evidence_count: usize,
}

/// Reasoning query for multi-hop analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningQuery {
    pub start_entities: Vec<String>,
    pub target_entities: Vec<String>,
    pub relationship_types: Vec<String>,
    pub max_hops: usize,
    pub min_confidence: f32,
    pub time_range: Option<TimeRange>,
}

/// Result of multi-hop reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    pub paths: Vec<ReasoningPath>,
    pub confidence_score: f32,
    pub reasoning_time_ms: i64,
    pub entities_discovered: Vec<String>,
}

/// A reasoning path through the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPath {
    pub entities: Vec<String>,
    pub relationships: Vec<String>,
    pub confidence: f32,
    pub hops: usize,
}

/// Temporal analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysis {
    pub time_range: TimeRange,
    pub trends: Vec<TemporalTrend>,
    pub change_points: Vec<ChangePoint>,
    pub causality_links: Vec<CausalityLink>,
    pub performance_summary: PerformanceSummary,
}

/// Temporal trend in agent performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTrend {
    pub metric: String,
    pub direction: TrendDirection,
    pub magnitude: f32,
    pub confidence: f32,
    pub time_range: TimeRange,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

/// Change point in performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePoint {
    pub timestamp: DateTime<Utc>,
    pub metric: String,
    pub change_magnitude: f32,
    pub confidence: f32,
    pub description: String,
}

/// Causality link between events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityLink {
    pub cause_event: String,
    pub effect_event: String,
    pub confidence: f32,
    pub time_delay_ms: Option<i64>,
    pub supporting_evidence: Vec<String>,
}

/// Performance summary over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub average_score: f32,
    pub best_score: f32,
    pub worst_score: f32,
    pub improvement_rate: f32,
    pub consistency_score: f32,
    pub total_samples: usize,
}

/// Memory operation types for provenance tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryOperation {
    Store,
    Retrieve,
    Update,
    Delete,
    Search,
    Reason,
    Consolidate,
    Decay,
    Offload,
}

/// Provenance record for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub operation: MemoryOperation,
    pub memory_id: Option<MemoryId>,
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, serde_json::Value>,
    pub reasoning: Vec<String>,
    pub confidence: Option<f32>,
    pub processing_time_ms: i64,
}


// Default implementations for configuration structs

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            graph_config: GraphConfig::default(),
            embedding_config: EmbeddingConfig::default(),
            temporal_config: TemporalConfig::default(),
            decay_config: DecayConfig::default(),
            context_config: ContextConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            base_decay_rate: 0.05,
            importance_boost_factor: 1.1,
            access_recency_weight: 0.3,
            consolidation_interval_hours: 24,
            minimum_memory_strength: 0.1,
            decay_schedule: DecaySchedule::Exponential,
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            compression_enabled: true,
            compression_threshold_kb: 50,
            offload_strategy: OffloadStrategy::Compress,
            retrieval_boost_factor: 1.2,
            max_context_age_days: 7,
        }
    }
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            max_entities: 10000,
            max_relationships_per_entity: 50,
            similarity_threshold: 0.8,
            deduplication_enabled: true,
            reasoning_depth: 3,
            cache_size: 1000,
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "embeddinggemma".to_string(),
            dimension: 768,
            batch_size: 32,
            cache_enabled: true,
            cache_size: 1000,
            similarity_threshold: 0.8,
        }
    }
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            analysis_window_days: 90,
            causality_enabled: true,
            trend_detection_enabled: true,
            forecasting_enabled: false,
            change_point_sensitivity: 0.7,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            query_timeout_ms: 5000,
            max_concurrent_queries: 100,
            memory_pressure_threshold_mb: 1024,
            cache_enabled: true,
        }
    }
}
