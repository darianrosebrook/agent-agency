//! Memory System Types - DTOs for memory operations
//!
//! Defines the data transfer objects used by the memory system port.
//! These types enable clean communication between orchestration and memory services.
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Memory type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(with = "String")]
pub enum MemoryType {
    Episodic,   // Event-based memories
    Semantic,   // Factual knowledge
    Procedural, // Skill-based memories
    Working,    // Short-term working memory
}

/// Re-export TaskPriority from planning module for consistency
pub use super::planning::TaskPriority;

/// Unique identifier for a memory/experience
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct MemoryId(#[schemars(with = "String")] pub Uuid);

impl std::fmt::Display for MemoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for MemoryId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MemoryId(Uuid::parse_str(s)?))
    }
}

/// Temporal context for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemporalContext {
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub sequence_number: Option<u64>,
    pub priority: TaskPriority,
}

/// Experience outcome classification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExperienceOutcome {
    pub success: bool,
    pub quality_score: f64,
    pub error_message: Option<String>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub performance_score: Option<f32>,
    pub execution_time_ms: Option<u64>,
    pub learned_capabilities: Vec<String>,
}

/// Query for temporal context retrieval
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemporalQuery {
    #[schemars(with = "Option<String>")]
    pub start_time: Option<DateTime<Utc>>,
    #[schemars(with = "Option<String>")]
    pub end_time: Option<DateTime<Utc>>,
    pub priority_filter: Option<TaskPriority>,
    pub memory_type_filter: Option<MemoryType>,
    pub limit: Option<usize>,
}

/// Experience data for storage
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Experience {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub description: String,
    pub memory_type: MemoryType,
    pub temporal_context: Option<TemporalContext>,
    pub outcome: ExperienceOutcome,
    pub domain: Vec<String>,
    pub task_type: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}
